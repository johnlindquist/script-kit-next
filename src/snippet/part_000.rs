#[derive(Debug, Clone, PartialEq)]
pub enum SnippetPart {
    /// Literal text (no special meaning)
    Text(String),
    /// A tabstop position
    Tabstop {
        /// Tabstop index: 0 = final cursor, 1+ = navigation order
        index: usize,
        /// Default placeholder text (from `${1:text}` syntax)
        placeholder: Option<String>,
        /// Choice options (from `${1|a,b,c|}` syntax)
        choices: Option<Vec<String>>,
        /// Byte range in the expanded text where this tabstop appears
        range: (usize, usize),
    },
}
/// Information about a tabstop, with all occurrences of the same index merged
#[derive(Debug, Clone, PartialEq)]
pub struct TabstopInfo {
    /// Tabstop index
    pub index: usize,
    /// All byte ranges where this tabstop appears (for linked editing)
    pub ranges: Vec<(usize, usize)>,
    /// Placeholder text (if any)
    pub placeholder: Option<String>,
    /// Choice options (if any)
    pub choices: Option<Vec<String>>,
}
/// A fully parsed snippet with expanded text and tabstop metadata
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSnippet {
    /// Sequential parts of the snippet (text and tabstops interleaved)
    pub parts: Vec<SnippetPart>,
    /// Fully expanded text with placeholders filled in
    pub text: String,
    /// Tabstops sorted by navigation order (1, 2, 3... then 0)
    pub tabstops: Vec<TabstopInfo>,
}
impl ParsedSnippet {
    /// Parse a VSCode snippet template string into a structured representation
    ///
    /// # Examples
    ///
    /// ```
    /// use script_kit_gpui::snippet::ParsedSnippet;
    ///
    /// let snippet = ParsedSnippet::parse("Hello $1!");
    /// assert_eq!(snippet.text, "Hello !");
    /// assert_eq!(snippet.tabstops.len(), 1);
    /// ```
    pub fn parse(template: &str) -> Self {
        let mut parts = Vec::new();
        let mut text = String::new();
        let mut char_count: usize = 0; // Track char count for char-based indices
        let mut chars = template.chars().peekable();
        let mut current_text = String::new();

        while let Some(c) = chars.next() {
            if c == '$' {
                match chars.peek() {
                    // Escaped dollar: $$ -> $
                    Some('$') => {
                        chars.next();
                        current_text.push('$');
                    }
                    // Tabstop with braces: ${...}
                    Some('{') => {
                        // Flush current text
                        if !current_text.is_empty() {
                            text.push_str(&current_text);
                            char_count += current_text.chars().count();
                            parts.push(SnippetPart::Text(current_text.clone()));
                            current_text.clear();
                        }
                        chars.next(); // consume '{'

                        let tabstop = Self::parse_braced_tabstop(&mut chars, char_count);
                        let placeholder_text = tabstop
                            .placeholder
                            .as_deref()
                            .or(tabstop
                                .choices
                                .as_ref()
                                .and_then(|c| c.first().map(|s| s.as_str())))
                            .unwrap_or("");

                        text.push_str(placeholder_text);
                        char_count += placeholder_text.chars().count();
                        parts.push(SnippetPart::Tabstop {
                            index: tabstop.index,
                            placeholder: tabstop.placeholder,
                            choices: tabstop.choices,
                            range: tabstop.range,
                        });
                    }
                    // Simple tabstop: $N
                    Some(d) if d.is_ascii_digit() => {
                        // Flush current text
                        if !current_text.is_empty() {
                            text.push_str(&current_text);
                            char_count += current_text.chars().count();
                            parts.push(SnippetPart::Text(current_text.clone()));
                            current_text.clear();
                        }

                        let mut num_str = String::new();
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_digit() {
                                num_str.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }

                        let index: usize = num_str.parse().unwrap_or(0);
                        // Simple tabstop has empty placeholder, so range is (char_count, char_count)
                        parts.push(SnippetPart::Tabstop {
                            index,
                            placeholder: None,
                            choices: None,
                            range: (char_count, char_count),
                        });
                    }
                    // Just a lone $ at end or followed by non-special char
                    _ => {
                        current_text.push('$');
                    }
                }
            } else {
                current_text.push(c);
            }
        }

        // Flush remaining text
        if !current_text.is_empty() {
            text.push_str(&current_text);
            parts.push(SnippetPart::Text(current_text));
        }

        // Build tabstop info, merging same indices
        let tabstops = Self::build_tabstop_info(&parts);

        Self {
            parts,
            text,
            tabstops,
        }
    }

    /// Parse a braced tabstop: `{1}`, `{1:default}`, or `{1|a,b,c|}`
    ///
    /// `char_offset` is the current position in char indices (not bytes).
    fn parse_braced_tabstop(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        char_offset: usize,
    ) -> TabstopParseResult {
        let mut index_str = String::new();

        // Parse index number
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                index_str.push(c);
                chars.next();
            } else {
                break;
            }
        }

        let index: usize = index_str.parse().unwrap_or(0);

        // Check what follows the index
        match chars.peek() {
            // Placeholder: ${1:text}
            Some(':') => {
                chars.next(); // consume ':'
                let placeholder = Self::parse_until_close_brace(chars);
                // Use char count, not byte length
                let placeholder_char_len = placeholder.chars().count();
                let range = (char_offset, char_offset + placeholder_char_len);
                TabstopParseResult {
                    index,
                    placeholder: Some(placeholder),
                    choices: None,
                    range,
                }
            }
            // Choices: ${1|a,b,c|}
            Some('|') => {
                chars.next(); // consume '|'
                let choices = Self::parse_choices(chars);
                // Use char count of first choice, not byte length
                let first_choice_char_len = choices.first().map(|s| s.chars().count()).unwrap_or(0);
                let range = (char_offset, char_offset + first_choice_char_len);
                TabstopParseResult {
                    index,
                    placeholder: None,
                    choices: Some(choices),
                    range,
                }
            }
            // Simple: ${1}
            Some('}') => {
                chars.next(); // consume '}'
                TabstopParseResult {
                    index,
                    placeholder: None,
                    choices: None,
                    range: (char_offset, char_offset),
                }
            }
            // Unexpected - consume until }
            _ => {
                Self::parse_until_close_brace(chars);
                TabstopParseResult {
                    index,
                    placeholder: None,
                    choices: None,
                    range: (char_offset, char_offset),
                }
            }
        }
    }

    /// Parse content until closing brace, handling nested braces
    #[allow(clippy::while_let_on_iterator)]
    fn parse_until_close_brace(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut result = String::new();
        let mut brace_depth = 1;

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.peek().copied() {
                    Some('{') | Some('}') | Some('$') | Some('\\') => {
                        if let Some(escaped) = chars.next() {
                            result.push(escaped);
                        }
                    }
                    Some(_) | None => result.push('\\'),
                }
                continue;
            }

            match c {
                '{' => {
                    brace_depth += 1;
                    result.push(c);
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        break;
                    }
                    result.push(c);
                }
                _ => result.push(c),
            }
        }

        result
    }

    /// Parse choice options: `a,b,c|}`
    fn parse_choices(chars: &mut std::iter::Peekable<std::str::Chars>) -> Vec<String> {
        let mut choices = Vec::new();
        let mut current = String::new();

        #[allow(clippy::while_let_on_iterator)]
        while let Some(c) = chars.next() {
            match c {
                ',' => {
                    choices.push(current.clone());
                    current.clear();
                }
                '|' => {
                    // End of choices, expect }
                    choices.push(current);
                    // Consume the closing }
                    if chars.peek() == Some(&'}') {
                        chars.next();
                    }
                    break;
                }
                '\\' => {
                    match chars.peek().copied() {
                        Some(',') | Some('|') | Some('\\') => {
                            if let Some(next) = chars.next() {
                                current.push(next);
                            }
                        }
                        Some(_) | None => current.push('\\'),
                    }
                }
                _ => current.push(c),
            }
        }

        choices
    }

    /// Build TabstopInfo from parts, merging same indices
    fn build_tabstop_info(parts: &[SnippetPart]) -> Vec<TabstopInfo> {
        use std::collections::BTreeMap;

        let mut tabstop_map: BTreeMap<usize, TabstopInfo> = BTreeMap::new();

        for part in parts {
            if let SnippetPart::Tabstop {
                index,
                placeholder,
                choices,
                range,
            } = part
            {
                tabstop_map
                    .entry(*index)
                    .and_modify(|info| {
                        info.ranges.push(*range);
                        // Keep first placeholder/choices found
                        if info.placeholder.is_none() && placeholder.is_some() {
                            info.placeholder = placeholder.clone();
                        }
                        if info.choices.is_none() && choices.is_some() {
                            info.choices = choices.clone();
                        }
                    })
                    .or_insert_with(|| TabstopInfo {
                        index: *index,
                        ranges: vec![*range],
                        placeholder: placeholder.clone(),
                        choices: choices.clone(),
                    });
            }
        }

        // Sort: all non-zero indices in order, then 0 (final cursor) at end.
        // Keep merged ranges for every index, including repeated $0.
        let mut result: Vec<TabstopInfo> = tabstop_map.into_values().collect();
        result.sort_by_key(|info| (info.index == 0, info.index));

        result
    }

    /// Get tabstop info by index
    #[allow(dead_code)]
    pub fn get_tabstop(&self, index: usize) -> Option<&TabstopInfo> {
        self.tabstops.iter().find(|t| t.index == index)
    }

    /// Get the navigation order of tabstops (1, 2, 3... then 0)
    #[allow(dead_code)]
    pub fn tabstop_order(&self) -> Vec<usize> {
        self.tabstops.iter().map(|t| t.index).collect()
    }

    /// Update tabstop ranges after an edit operation.
    ///
    /// This method adjusts all tabstop ranges to account for text changes in the document.
    /// Ranges are stored as char indices (not byte offsets) to match editor cursor positions.
    ///
    /// # Arguments
    /// * `current_tabstop_idx` - Index into self.tabstops of the tabstop currently being edited.
    ///   Ranges within this tabstop that contain the edit point will be resized.
    ///   Pass `usize::MAX` if editing outside any tabstop.
    /// * `edit_start` - Char index where the edit begins
    /// * `old_len` - Number of chars that were removed
    /// * `new_len` - Number of chars that were inserted
    ///
    /// # Behavior
    /// - Ranges **after** the edit point are shifted by `delta = new_len - old_len`
    /// - Ranges **containing** the edit point (within current tabstop) are resized by `delta`
    /// - Ranges **before** the edit point are unchanged
    ///
    /// Note: This is prepared for linked editing support (when the same tabstop appears
    /// multiple times in a template, edits should sync across all occurrences).
    #[allow(dead_code)]
    pub fn update_tabstops_after_edit(
        &mut self,
        current_tabstop_idx: usize,
        edit_start: usize,
        old_len: usize,
        new_len: usize,
    ) {
        let delta = new_len as isize - old_len as isize;
        if delta == 0 {
            return;
        }

        let shift = |value: usize| -> usize {
            if delta >= 0 {
                value.saturating_add(delta as usize)
            } else {
                value.saturating_sub((-delta) as usize)
            }
        };

        let edit_end = edit_start + old_len;

        for (tabstop_idx, tabstop) in self.tabstops.iter_mut().enumerate() {
            for range in tabstop.ranges.iter_mut() {
                let (range_start, range_end) = *range;

                // Case 1: Range is entirely before the edit - no change
                if range_end <= edit_start {
                    continue;
                }

                // Case 2: Range is entirely after the edit - shift by delta
                if range_start > edit_end
                    || (range_start == edit_end && tabstop_idx != current_tabstop_idx)
                {
                    *range = (shift(range_start), shift(range_end));
                    continue;
                }

                // Case 3: Edit is within or at the boundary of this range
                // For the current tabstop, we resize (keep start, adjust end)
                // For other tabstops, the edit should not overlap (they're not being edited)
                if tabstop_idx == current_tabstop_idx {
                    // Edit is within this range - keep start, resize end
                    *range = (range_start, shift(range_end).max(range_start));
                } else {
                    // This range starts at or after the edit point but before edit_end
                    // This means it overlaps with the edit region
                    // Shift the entire range by delta
                    *range = (shift(range_start), shift(range_end));
                }
            }
        }
    }
}
/// Internal helper for parsing braced tabstops
struct TabstopParseResult {
    index: usize,
    placeholder: Option<String>,
    choices: Option<Vec<String>>,
    range: (usize, usize),
}

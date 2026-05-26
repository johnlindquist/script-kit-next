use super::types::*;

const PROMPT_BUILDER_SIGILS: &[char] = &['@', '/', '|', '.'];
const CAPTURE_SIGIL: char = ';';
const FILTER_SIGIL: char = ':';
const MODE_EXIT_SIGILS: &[char] = &['~', '>', '?'];

/// Parse an input string into a sequence of Spine segments.
///
/// The parser splits the input at sigil boundaries while preserving free text.
/// It does NOT validate segments against known entities — that is the resolution
/// step, done separately by catalog lookups.
pub fn parse_spine(input: &str) -> SpineParse {
    if input.is_empty() {
        return SpineParse {
            segments: vec![],
            input: input.to_string(),
        };
    }

    // Mode exit: if the input starts with ~, >, or ?, the entire input is one
    // ModeExit segment. These sigils exit the grammar flow entirely.
    if let Some(first) = input.chars().next() {
        if MODE_EXIT_SIGILS.contains(&first) {
            let rest = &input[first.len_utf8()..];
            return SpineParse {
                segments: vec![SpineSegment {
                    kind: SpineSegmentKind::ModeExit {
                        sigil: first,
                        rest: rest.to_string(),
                    },
                    byte_range: 0..input.len(),
                    raw: input.to_string(),
                    resolution: SpineSegmentResolution::Unresolved,
                }],
                input: input.to_string(),
            };
        }
    }

    let segments = split_segments(input);

    SpineParse {
        segments,
        input: input.to_string(),
    }
}

/// Given a parsed `SpineParse` and a cursor byte offset, compute which segment
/// the cursor is inside and what query to use for list projection.
pub fn project_cursor(parse: &SpineParse, cursor_byte: usize) -> SpineCursorProjection {
    if parse.segments.is_empty() {
        return SpineCursorProjection {
            active_segment_index: 0,
            active_segment_kind: SpineSegmentKind::FreeText,
            active_query: String::new(),
            is_tail: false,
            has_prompt_segments: false,
        };
    }

    let has_prompt_segments = parse.segments.iter().any(|s| is_prompt_builder_segment(s));

    // Find the segment containing the cursor. If cursor is at a boundary,
    // it belongs to the segment whose range includes it (left-biased for
    // range end, since ranges are half-open).
    let mut active_index = parse.segments.len() - 1;
    for (i, seg) in parse.segments.iter().enumerate() {
        if cursor_byte >= seg.byte_range.start && cursor_byte <= seg.byte_range.end {
            active_index = i;
            break;
        }
    }

    let active_seg = &parse.segments[active_index];
    let active_query = extract_active_query(active_seg, cursor_byte);
    let is_tail = matches!(active_seg.kind, SpineSegmentKind::FreeText) && has_prompt_segments;

    SpineCursorProjection {
        active_segment_index: active_index,
        active_segment_kind: active_seg.kind.clone(),
        active_query,
        is_tail,
        has_prompt_segments,
    }
}

fn is_prompt_builder_segment(seg: &SpineSegment) -> bool {
    matches!(
        seg.kind,
        SpineSegmentKind::ContextMention { .. }
            | SpineSegmentKind::SlashCommand { .. }
            | SpineSegmentKind::Profile { .. }
            | SpineSegmentKind::Style { .. }
    )
}

/// Split the input into segments at sigil boundaries.
///
/// Prompt-builder sigils (`@`, `/`, `|`, `.`) claim only the sigil + the
/// immediately following whitespace-delimited word.  Greedy sigils (`;`, `:`)
/// consume until the next boundary-sigil or end of input.  Everything else is
/// free text.  Segments do NOT include inter-segment whitespace.
fn split_segments(input: &str) -> Vec<SpineSegment> {
    let mut segments = Vec::new();
    let bytes = input.as_bytes();
    let len = input.len();
    let mut pos = 0;

    while pos < len {
        // Skip whitespace between segments
        if bytes[pos] == b' ' {
            pos += 1;
            continue;
        }

        let ch = input[pos..].chars().next().unwrap();
        let ch_len = ch.len_utf8();
        let at_boundary = pos == 0 || bytes[pos - 1] == b' ';

        let is_prompt_sigil = PROMPT_BUILDER_SIGILS.contains(&ch) && at_boundary;
        let is_greedy_sigil = (ch == CAPTURE_SIGIL || ch == FILTER_SIGIL) && at_boundary;

        if is_prompt_sigil {
            // Prompt-builder sigil: claim sigil + one word (up to next space)
            let seg_start = pos;
            pos += ch_len;
            // Consume non-space chars (the value word)
            while pos < len && bytes[pos] != b' ' {
                pos += input[pos..].chars().next().unwrap().len_utf8();
            }
            let raw = input[seg_start..pos].to_string();
            let rest = &input[seg_start + ch_len..pos];
            let kind = classify_sigil_segment(ch, rest);
            segments.push(SpineSegment {
                kind,
                byte_range: seg_start..pos,
                raw,
                resolution: SpineSegmentResolution::Unresolved,
            });
        } else if is_greedy_sigil {
            // Greedy sigil (`;`, `:`): consume until next boundary-sigil or end
            let seg_start = pos;
            pos += ch_len;
            while pos < len {
                let next_ch = input[pos..].chars().next().unwrap();
                let next_at_boundary = bytes[pos - 1] == b' ';
                let next_is_any_sigil = PROMPT_BUILDER_SIGILS.contains(&next_ch)
                    || next_ch == CAPTURE_SIGIL
                    || next_ch == FILTER_SIGIL;
                if next_is_any_sigil && next_at_boundary {
                    break;
                }
                pos += next_ch.len_utf8();
            }
            // Trim trailing whitespace from greedy segment
            let mut seg_end = pos;
            while seg_end > seg_start && bytes[seg_end - 1] == b' ' {
                seg_end -= 1;
            }
            let raw = input[seg_start..seg_end].to_string();
            let rest = &input[seg_start + ch_len..seg_end];
            let kind = classify_sigil_segment(ch, rest);
            segments.push(SpineSegment {
                kind,
                byte_range: seg_start..seg_end,
                raw,
                resolution: SpineSegmentResolution::Unresolved,
            });
        } else {
            // Free text: consume words until we hit a boundary-sigil
            let seg_start = pos;
            while pos < len {
                if bytes[pos] == b' ' {
                    // Check if the next non-space char is a sigil at boundary
                    let peek = pos + 1;
                    if peek < len {
                        let next_ch = input[peek..].chars().next().unwrap();
                        let next_is_sigil = PROMPT_BUILDER_SIGILS.contains(&next_ch)
                            || next_ch == CAPTURE_SIGIL
                            || next_ch == FILTER_SIGIL;
                        if next_is_sigil {
                            break;
                        }
                    }
                }
                pos += input[pos..].chars().next().unwrap().len_utf8();
            }
            // Trim trailing whitespace
            let mut seg_end = pos;
            while seg_end > seg_start && bytes[seg_end - 1] == b' ' {
                seg_end -= 1;
            }
            let raw = input[seg_start..seg_end].to_string();
            segments.push(SpineSegment {
                kind: SpineSegmentKind::FreeText,
                byte_range: seg_start..seg_end,
                raw,
                resolution: SpineSegmentResolution::Unresolved,
            });
        }
    }

    segments
}

/// Classify a sigil segment by its leading character.
fn classify_sigil_segment(sigil: char, rest: &str) -> SpineSegmentKind {
    match sigil {
        '@' => {
            // Check for sub-search pattern: @type:query
            if let Some(colon_pos) = rest.find(':') {
                let context_type = rest[..colon_pos].to_string();
                let sub_query = rest[colon_pos + 1..].to_string();
                SpineSegmentKind::ContextMention {
                    context_type,
                    sub_query: if sub_query.is_empty() {
                        None
                    } else {
                        Some(sub_query)
                    },
                }
            } else {
                SpineSegmentKind::ContextMention {
                    context_type: rest.to_string(),
                    sub_query: None,
                }
            }
        }
        '/' => SpineSegmentKind::SlashCommand {
            command: rest.to_string(),
        },
        '|' => SpineSegmentKind::Profile {
            profile_id: rest.to_string(),
        },
        '.' => SpineSegmentKind::Style {
            style_id: rest.to_string(),
        },
        ';' => {
            let first_space = rest.find(' ');
            match first_space {
                Some(pos) => SpineSegmentKind::Capture {
                    target: rest[..pos].to_string(),
                    args: rest[pos + 1..].to_string(),
                },
                None => SpineSegmentKind::Capture {
                    target: rest.to_string(),
                    args: String::new(),
                },
            }
        }
        ':' => SpineSegmentKind::ListFilter {
            query: rest.to_string(),
        },
        _ => SpineSegmentKind::FreeText,
    }
}

/// Extract the query text from the active segment at the given cursor position.
/// This is what drives list filtering for the projected rows.
fn extract_active_query(seg: &SpineSegment, cursor_byte: usize) -> String {
    match &seg.kind {
        SpineSegmentKind::FreeText => {
            // Return text from segment start to cursor
            let offset = cursor_byte.saturating_sub(seg.byte_range.start);
            let clamped = offset.min(seg.raw.len());
            // Find a valid char boundary
            let boundary = find_char_boundary(&seg.raw, clamped);
            seg.raw[..boundary].trim().to_string()
        }
        SpineSegmentKind::ContextMention {
            sub_query: Some(sq),
            ..
        } => {
            // For @file:readme, the query is the sub-query portion
            sq.clone()
        }
        SpineSegmentKind::ContextMention {
            context_type,
            sub_query: None,
        } => context_type.clone(),
        SpineSegmentKind::SlashCommand { command } => command.clone(),
        SpineSegmentKind::Profile { profile_id } => profile_id.clone(),
        SpineSegmentKind::Style { style_id } => style_id.clone(),
        SpineSegmentKind::Capture { target, .. } => target.clone(),
        SpineSegmentKind::ListFilter { query } => query.clone(),
        SpineSegmentKind::ModeExit { rest, .. } => rest.clone(),
    }
}

/// Find the nearest valid UTF-8 char boundary at or before `pos` in `s`.
fn find_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut p = pos;
    while p > 0 && !s.is_char_boundary(p) {
        p -= 1;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let parse = parse_spine("");
        assert!(parse.segments.is_empty());
    }

    #[test]
    fn plain_text_only() {
        let parse = parse_spine("hello world");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(parse.segments[0].kind, SpineSegmentKind::FreeText));
        assert_eq!(parse.segments[0].raw, "hello world");
    }

    #[test]
    fn single_context_mention() {
        let parse = parse_spine("@selection");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ContextMention { context_type, sub_query: None }
            if context_type == "selection"
        ));
    }

    #[test]
    fn context_mention_with_sub_query() {
        let parse = parse_spine("@file:readme.md");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ContextMention { context_type, sub_query: Some(sq) }
            if context_type == "file" && sq == "readme.md"
        ));
    }

    #[test]
    fn context_mention_sub_search_empty_query() {
        let parse = parse_spine("@file:");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ContextMention { context_type, sub_query: None }
            if context_type == "file"
        ));
    }

    #[test]
    fn slash_command() {
        let parse = parse_spine("/rewrite");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::SlashCommand { command }
            if command == "rewrite"
        ));
    }

    #[test]
    fn profile() {
        let parse = parse_spine("|creative");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::Profile { profile_id }
            if profile_id == "creative"
        ));
    }

    #[test]
    fn style() {
        let parse = parse_spine(".professional");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::Style { style_id }
            if style_id == "professional"
        ));
    }

    #[test]
    fn capture_with_args() {
        let parse = parse_spine(";todo buy milk");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::Capture { target, args }
            if target == "todo" && args == "buy milk"
        ));
    }

    #[test]
    fn capture_no_args() {
        let parse = parse_spine(";todo");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::Capture { target, args }
            if target == "todo" && args.is_empty()
        ));
    }

    #[test]
    fn list_filter() {
        let parse = parse_spine(":type:script");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ListFilter { query }
            if query == "type:script"
        ));
    }

    #[test]
    fn mode_exit_tilde() {
        let parse = parse_spine("~/Documents");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ModeExit { sigil: '~', .. }
        ));
    }

    #[test]
    fn mode_exit_greater() {
        let parse = parse_spine(">ls -la");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ModeExit { sigil: '>', .. }
        ));
    }

    #[test]
    fn mode_exit_question() {
        let parse = parse_spine("?");
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ModeExit { sigil: '?', .. }
        ));
    }

    #[test]
    fn full_prompt_builder() {
        let input = "@file:readme |creative /rewrite make it punchier";
        let parse = parse_spine(input);

        assert_eq!(parse.segments.len(), 4);

        // @file:readme
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ContextMention { context_type, sub_query: Some(sq) }
            if context_type == "file" && sq == "readme"
        ));

        // |creative
        assert!(matches!(
            &parse.segments[1].kind,
            SpineSegmentKind::Profile { profile_id }
            if profile_id == "creative"
        ));

        // /rewrite
        assert!(matches!(
            &parse.segments[2].kind,
            SpineSegmentKind::SlashCommand { command }
            if command == "rewrite"
        ));

        // make it punchier (free text tail)
        assert!(matches!(parse.segments[3].kind, SpineSegmentKind::FreeText));
        assert_eq!(parse.segments[3].raw, "make it punchier");
    }

    #[test]
    fn cursor_in_context_mention() {
        let input = "@file:readme |creative /rewrite make it punchier";
        let parse = parse_spine(input);

        // Cursor inside "readme" (byte 6-11)
        let proj = project_cursor(&parse, 8);
        assert_eq!(proj.active_segment_index, 0);
        assert!(matches!(
            &proj.active_segment_kind,
            SpineSegmentKind::ContextMention { .. }
        ));
        assert_eq!(proj.active_query, "readme");
        assert!(!proj.is_tail);
        assert!(proj.has_prompt_segments);
    }

    #[test]
    fn cursor_in_slash_command() {
        let input = "@file:readme |creative /rewrite make it punchier";
        let parse = parse_spine(input);

        // Cursor inside "/rewrite" — find its byte range
        let rewrite_seg = &parse.segments[2];
        let proj = project_cursor(&parse, rewrite_seg.byte_range.start + 2);
        assert_eq!(proj.active_segment_index, 2);
        assert!(matches!(
            &proj.active_segment_kind,
            SpineSegmentKind::SlashCommand { command }
            if command == "rewrite"
        ));
    }

    #[test]
    fn cursor_in_free_text_tail() {
        let input = "@file:readme |creative /rewrite make it punchier";
        let parse = parse_spine(input);

        // Cursor at end of input
        let proj = project_cursor(&parse, input.len());
        assert_eq!(proj.active_segment_index, parse.segments.len() - 1);
        assert!(matches!(
            proj.active_segment_kind,
            SpineSegmentKind::FreeText
        ));
        assert!(proj.is_tail);
        assert!(proj.has_prompt_segments);
    }

    #[test]
    fn free_text_without_prompt_segments_is_not_tail() {
        let input = "hello world";
        let parse = parse_spine(input);

        let proj = project_cursor(&parse, input.len());
        assert!(!proj.is_tail);
        assert!(!proj.has_prompt_segments);
    }

    #[test]
    fn unknown_segment_gets_preflight_instruction() {
        let input = "@unknownThing";
        let parse = parse_spine(input);
        assert_eq!(parse.segments.len(), 1);

        // Resolution is Unresolved by default — resolution happens at catalog lookup time.
        // The parser only classifies; it doesn't resolve.
        assert!(matches!(
            parse.segments[0].resolution,
            SpineSegmentResolution::Unresolved
        ));
    }

    #[test]
    fn unicode_input_safety() {
        let input = "@file:日本語 /rewrite 文章を修正";
        let parse = parse_spine(input);

        assert_eq!(parse.segments.len(), 3);

        // Verify byte ranges are valid UTF-8 boundaries
        for seg in &parse.segments {
            assert!(input.is_char_boundary(seg.byte_range.start));
            assert!(input.is_char_boundary(seg.byte_range.end));
            assert_eq!(&input[seg.byte_range.clone()], seg.raw);
        }

        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ContextMention { context_type, sub_query: Some(sq) }
            if context_type == "file" && sq == "日本語"
        ));
    }

    #[test]
    fn sigil_in_middle_of_word_is_not_a_segment() {
        let input = "hello@world.com";
        let parse = parse_spine(input);
        // The @ is not at a word boundary, so this should be plain free text
        assert_eq!(parse.segments.len(), 1);
        assert!(matches!(parse.segments[0].kind, SpineSegmentKind::FreeText));
    }

    #[test]
    fn multiple_context_mentions() {
        let input = "@selection @clipboard @file:notes.md";
        let parse = parse_spine(input);

        assert_eq!(parse.segments.len(), 3);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::ContextMention { context_type, .. }
            if context_type == "selection"
        ));
        assert!(matches!(
            &parse.segments[1].kind,
            SpineSegmentKind::ContextMention { context_type, .. }
            if context_type == "clipboard"
        ));
        assert!(matches!(
            &parse.segments[2].kind,
            SpineSegmentKind::ContextMention { context_type, sub_query: Some(sq) }
            if context_type == "file" && sq == "notes.md"
        ));
    }

    #[test]
    fn style_sugar_with_tail() {
        let input = ".professional make it shorter";
        let parse = parse_spine(input);

        // `.professional` is one prompt-builder segment, "make it shorter" is free text
        assert_eq!(parse.segments.len(), 2);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::Style { style_id }
            if style_id == "professional"
        ));
        assert!(matches!(parse.segments[1].kind, SpineSegmentKind::FreeText));
        assert_eq!(parse.segments[1].raw, "make it shorter");
    }

    #[test]
    fn capture_with_context_attachment() {
        let input = ";todo buy milk @clipboard";
        let parse = parse_spine(input);

        // ";todo buy milk" is a capture segment, then "@clipboard" is a context mention
        assert_eq!(parse.segments.len(), 2);
        assert!(matches!(
            &parse.segments[0].kind,
            SpineSegmentKind::Capture { target, args }
            if target == "todo" && args == "buy milk"
        ));
        assert!(matches!(
            &parse.segments[1].kind,
            SpineSegmentKind::ContextMention { context_type, .. }
            if context_type == "clipboard"
        ));
    }

    #[test]
    fn byte_ranges_are_ordered_and_non_overlapping() {
        let input = "@file:readme |creative /rewrite make it punchier";
        let parse = parse_spine(input);

        // Verify segments are ordered, non-overlapping, and their raw text
        // matches the byte range slice. Inter-segment whitespace is unowned.
        let mut prev_end = 0;
        for seg in &parse.segments {
            assert!(
                seg.byte_range.start >= prev_end,
                "Segment starts before previous end: {} < {}",
                seg.byte_range.start,
                prev_end
            );
            assert_eq!(
                &input[seg.byte_range.clone()],
                seg.raw,
                "Byte range doesn't match raw text"
            );
            prev_end = seg.byte_range.end;
        }
        assert!(prev_end <= input.len(), "Last segment extends past input");
    }
}

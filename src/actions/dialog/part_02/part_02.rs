impl ActionsDialog {

    /// Set actions from SDK (replaces built-in actions)
    ///
    /// Converts `ProtocolAction` items to internal `Action` format and updates
    /// the actions list. Filters out actions with `visible: false`.
    /// The `has_action` field on each action determines routing:
    /// - `has_action=true`: Send ActionTriggered back to SDK
    /// - `has_action=false`: Submit value directly
    pub fn set_sdk_actions(&mut self, actions: Vec<ProtocolAction>) {
        let total_count = actions.len();
        let mut sdk_action_indices = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();
        let mut duplicate_names = Vec::new();

        let converted: Vec<Action> = actions
            .iter()
            .enumerate()
            .filter_map(|(protocol_index, pa)| {
                if !pa.is_visible() {
                    return None;
                }
                if !seen_names.insert(pa.name.clone()) {
                    duplicate_names.push(pa.name.clone());
                }
                sdk_action_indices.push(protocol_index);
                let shortcut = pa.shortcut.as_ref().map(|s| Self::format_shortcut_hint(s));
                Some(Action {
                    id: pa.name.clone(),
                    title: pa.name.clone(),
                    description: pa.description.clone(),
                    category: ActionCategory::ScriptContext,
                    shortcut: shortcut.clone(),
                    has_action: pa.has_action,
                    value: pa.value.clone(),
                    icon: None,    // SDK actions don't currently have icons
                    section: None, // SDK actions don't currently have sections
                    // Pre-compute lowercase for fast filtering (performance optimization)
                    title_lower: pa.name.to_lowercase(),
                    description_lower: pa.description.as_ref().map(|d| d.to_lowercase()),
                    shortcut_lower: shortcut.as_ref().map(|s| s.to_lowercase()),
                })
            })
            .collect();
        let visible_count = converted.len();

        if !duplicate_names.is_empty() {
            tracing::warn!(
                target: "script_kit::actions",
                duplicate_names = ?duplicate_names,
                "SDK actions contain duplicate names; using selected row index for protocol mapping"
            );
        }

        logging::log(
            "ACTIONS",
            &format!(
                "SDK actions set: {} visible of {} total",
                visible_count, total_count
            ),
        );

        self.actions = converted;
        self.filtered_actions = (0..self.actions.len()).collect();
        self.search_text.clear();
        self.sdk_actions = Some(actions);
        self.sdk_action_indices = sdk_action_indices;
        // Rebuild grouped items and reset selection
        self.rebuild_grouped_items();
        self.selected_index = initial_selection_index(&self.grouped_items);
    }

    /// Format a keyboard shortcut for display (e.g., "cmd+c" → "⌘C")
    pub(crate) fn format_shortcut_hint(shortcut: &str) -> String {
        format_shortcut_hint_shared(shortcut)
    }

    /// Clear SDK actions and restore built-in actions
    pub fn clear_sdk_actions(&mut self) {
        if self.sdk_actions.is_some() {
            logging::log(
                "ACTIONS",
                "Clearing SDK actions, restoring built-in actions",
            );
            self.sdk_actions = None;
            self.sdk_action_indices.clear();
            self.actions = Self::build_actions(&self.focused_script, &self.focused_scriptlet);
            self.filtered_actions = (0..self.actions.len()).collect();
            self.search_text.clear();
            // Rebuild grouped items and reset selection
            self.rebuild_grouped_items();
            self.selected_index = initial_selection_index(&self.grouped_items);
        }
    }

    /// Check if SDK actions are currently active
    pub fn has_sdk_actions(&self) -> bool {
        self.sdk_actions.is_some()
    }

    /// Get the currently selected action (for external handling)
    pub fn get_selected_action(&self) -> Option<&Action> {
        self.selected_action_index()
            .and_then(|action_idx| self.actions.get(action_idx))
    }

    /// Count the number of section headers in the filtered action list
    /// A section header appears when an action's section differs from the previous action's section
    pub fn count_section_headers(&self) -> usize {
        if self.filtered_actions.is_empty() {
            return 0;
        }

        let mut count = 0;
        let mut prev_section: Option<&Option<String>> = None;

        for &idx in &self.filtered_actions {
            if let Some(action) = self.actions.get(idx) {
                let current_section = &action.section;
                // Count as header if: first item with a section, or section changed
                if current_section.is_some() {
                    match prev_section {
                        None => count += 1,                                  // First item with a section
                        Some(prev) if prev != current_section => count += 1, // Section changed
                        _ => {}
                    }
                }
                prev_section = Some(current_section);
            }
        }

        count
    }

    /// Build the complete actions list based on focused script and optional scriptlet
    fn build_actions(
        focused_script: &Option<ScriptInfo>,
        focused_scriptlet: &Option<Scriptlet>,
    ) -> Vec<Action> {
        let mut actions = Vec::new();

        // Add script-specific actions first if a script is focused
        if let Some(script) = focused_script {
            // If this is a scriptlet with custom actions, use the enhanced builder
            if script.is_scriptlet && focused_scriptlet.is_some() {
                actions.extend(get_scriptlet_context_actions_with_custom(
                    script,
                    focused_scriptlet.as_ref(),
                ));
            } else {
                // Use standard actions for regular scripts
                actions.extend(get_script_context_actions(script));
            }
        }

        // Add global actions
        actions.extend(get_global_actions());

        actions
    }

    /// Update the focused script and rebuild actions
    pub fn set_focused_script(&mut self, script: Option<ScriptInfo>) {
        self.focused_script = script;
        self.focused_scriptlet = None; // Clear scriptlet when only setting script
        self.actions = Self::build_actions(&self.focused_script, &self.focused_scriptlet);
        self.refilter();
    }

    /// Update both the focused script and scriptlet for custom actions
    ///
    /// Use this when the focused item is a scriptlet with H3-defined custom actions.
    /// The scriptlet's actions will appear in the Actions Menu.
    pub fn set_focused_scriptlet(
        &mut self,
        script: Option<ScriptInfo>,
        scriptlet: Option<Scriptlet>,
    ) {
        self.focused_script = script;
        self.focused_scriptlet = scriptlet;
        self.actions = Self::build_actions(&self.focused_script, &self.focused_scriptlet);
        self.refilter();

        logging::log(
            "ACTIONS",
            &format!(
                "Set focused scriptlet with {} custom actions",
                self.focused_scriptlet
                    .as_ref()
                    .map(|s| s.actions.len())
                    .unwrap_or(0)
            ),
        );
    }

    /// Update the theme when hot-reloading
    /// Call this from the parent when theme changes to ensure dialog reflects new colors
    pub fn update_theme(&mut self, theme: Arc<theme::Theme>) {
        let is_dark = theme.should_use_dark_vibrancy();
        logging::log(
            "ACTIONS_THEME",
            &format!(
                "Theme updated in ActionsDialog (mode={}, keycap_base=#{:06x})",
                if is_dark { "dark" } else { "light" },
                if is_dark {
                    theme.colors.ui.border
                } else {
                    theme.colors.text.secondary
                }
            ),
        );
        self.theme = theme;
    }

    /// Refilter actions based on current search_text using ranked fuzzy matching.
    ///
    /// Scoring system:
    /// - Prefix match on title: +100 (strongest signal)
    /// - Fuzzy match on title: +50 + character bonus
    /// - Contains match on description: +25
    /// - Results are sorted by score (descending)
    fn refilter(&mut self) {
        // Preserve selection if possible (track which action was selected)
        // NOTE: selected_index is an index into grouped_items, not filtered_actions.
        // We must extract the filter_idx from the GroupedActionItem first.
        let previously_selected = match self.grouped_items.get(self.selected_index) {
            Some(GroupedActionItem::Item(filter_idx)) => self
                .filtered_actions
                .get(*filter_idx)
                .and_then(|&idx| self.actions.get(idx).map(|a| a.id.clone())),
            _ => None,
        };

        if self.search_text.is_empty() {
            self.filtered_actions = (0..self.actions.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();

            // Score each action and collect (index, score) pairs
            let mut scored: Vec<(usize, i32)> = self
                .actions
                .iter()
                .enumerate()
                .filter_map(|(idx, action)| {
                    let score = Self::score_action(action, &search_lower);
                    if score > 0 {
                        Some((idx, score))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by score descending
            scored.sort_by(|a, b| b.1.cmp(&a.1));

            // Extract just the indices
            self.filtered_actions = scored.into_iter().map(|(idx, _)| idx).collect();
        }

        // Rebuild grouped items after filter change
        self.rebuild_grouped_items();

        // Preserve selection if the same action is still in results
        // NOTE: We must find the position in grouped_items, not filtered_actions,
        // because grouped_items may include section headers that offset the indices.
        if let Some(prev_id) = previously_selected {
            // First find the filter_idx in filtered_actions
            if let Some(filter_idx) = self.filtered_actions.iter().position(|&idx| {
                self.actions
                    .get(idx)
                    .map(|a| a.id == prev_id)
                    .unwrap_or(false)
            }) {
                // Now find the position in grouped_items that contains Item(filter_idx)
                if let Some(grouped_idx) = self
                    .grouped_items
                    .iter()
                    .position(|item| matches!(item, GroupedActionItem::Item(i) if *i == filter_idx))
                {
                    self.selected_index = grouped_idx;
                } else {
                    // Fallback: coerce to first valid item
                    self.selected_index =
                        coerce_action_selection(&self.grouped_items, 0).unwrap_or(0);
                }
            } else {
                // Action no longer in results, select first valid item
                self.selected_index = coerce_action_selection(&self.grouped_items, 0).unwrap_or(0);
            }
        } else {
            // No previous selection, select first valid item
            self.selected_index = coerce_action_selection(&self.grouped_items, 0).unwrap_or(0);
        }

        // Only scroll if we have results
        if !self.grouped_items.is_empty() {
            self.list_state.scroll_to_reveal_item(self.selected_index);
        }

        logging::log_debug(
            "ACTIONS_SCROLL",
            &format!(
                "Filter changed: {} results, selected={}",
                self.filtered_actions.len(),
                self.selected_index
            ),
        );
    }

    /// Rebuild grouped_items from current filtered_actions
    fn rebuild_grouped_items(&mut self) {
        self.grouped_items = build_grouped_items_static(
            &self.actions,
            &self.filtered_actions,
            self.config.section_style,
        );
        // Update list state item count
        let old_count = self.list_state.item_count();
        let new_count = self.grouped_items.len();
        self.list_state.splice(0..old_count, new_count);
    }

    fn selected_action_index(&self) -> Option<usize> {
        let filter_idx = self.get_selected_filtered_index()?;
        self.filtered_actions.get(filter_idx).copied()
    }

    fn grouped_index_for_action_index(&self, action_idx: usize) -> Option<usize> {
        let filter_idx = self
            .filtered_actions
            .iter()
            .position(|&idx| idx == action_idx)?;
        self.grouped_items
            .iter()
            .position(|item| matches!(item, GroupedActionItem::Item(i) if *i == filter_idx))
    }

    /// Get the filtered_actions index for the current selection
    /// Returns None if selection is on a section header
    pub fn get_selected_filtered_index(&self) -> Option<usize> {
        match self.grouped_items.get(self.selected_index) {
            Some(GroupedActionItem::Item(filter_idx)) => Some(*filter_idx),
            _ => None,
        }
    }

    /// Score an action against a search query.
    /// Returns 0 if no match, higher scores for better matches.
    ///
    /// PERFORMANCE: Uses pre-computed lowercase fields (title_lower, description_lower,
    /// shortcut_lower) to avoid repeated to_lowercase() calls on every keystroke.
    pub(crate) fn score_action(action: &Action, search_lower: &str) -> i32 {
        let mut score = 0;

        // Prefix match on title (strongest) - use cached lowercase
        if action.title_lower.starts_with(search_lower) {
            score += 100;
        }
        // Contains match on title
        else if action.title_lower.contains(search_lower) {
            score += 50;
        }
        // Fuzzy match on title (character-by-character subsequence)
        else if Self::fuzzy_match(&action.title_lower, search_lower) {
            score += 25;
        }

        // Description match (bonus) - use cached lowercase
        if let Some(ref desc_lower) = action.description_lower {
            if desc_lower.contains(search_lower) {
                score += 15;
            }
        }

        // Shortcut match (bonus) - use cached lowercase
        if let Some(ref shortcut_lower) = action.shortcut_lower {
            if shortcut_lower.contains(search_lower) {
                score += 10;
            }
        }

        score
    }

    /// Simple fuzzy matching: check if all characters in needle appear in haystack in order.
    pub(crate) fn fuzzy_match(haystack: &str, needle: &str) -> bool {
        let mut haystack_chars = haystack.chars();
        for needle_char in needle.chars() {
            loop {
                match haystack_chars.next() {
                    Some(h) if h == needle_char => break,
                    Some(_) => continue,
                    None => return false,
                }
            }
        }
        true
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }
}

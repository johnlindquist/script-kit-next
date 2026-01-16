//! Actions Dialog
//!
//! The main ActionsDialog struct and its implementation, providing a searchable
//! action menu as a compact overlay popup.

#![allow(dead_code)]

use crate::components::scrollbar::{Scrollbar, ScrollbarColors};
use crate::designs::{get_tokens, DesignColors, DesignVariant};
use crate::logging;
use crate::protocol::ProtocolAction;
use crate::theme;
use gpui::{
    div, list, prelude::*, px, rgb, rgba, svg, App, BoxShadow, Context, ElementId, FocusHandle,
    Focusable, ListAlignment, ListState, Render, SharedString, Window,
};
use std::sync::Arc;

use super::builders::{
    get_chat_context_actions, get_clipboard_history_context_actions, get_file_context_actions,
    get_global_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, ChatPromptInfo, ClipboardEntryInfo,
};
use super::constants::{
    ACTION_ITEM_HEIGHT, ACTION_ROW_INSET, HEADER_HEIGHT, KEYCAP_HEIGHT, KEYCAP_MIN_WIDTH,
    POPUP_MAX_HEIGHT, POPUP_WIDTH, SEARCH_INPUT_HEIGHT, SECTION_HEADER_HEIGHT, SELECTION_RADIUS,
};
use crate::file_search::FileInfo;
use crate::scriptlets::Scriptlet;

// Keep ACCENT_BAR_WIDTH for backwards compatibility during transition
#[allow(unused_imports)]
use super::constants::ACCENT_BAR_WIDTH;
#[allow(unused_imports)] // AnchorPosition reserved for future use
use super::types::{
    Action, ActionCallback, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo,
    SearchPosition, SectionStyle,
};
use crate::prompts::PathInfo;

/// Helper function to combine a hex color with an alpha value
/// Delegates to DesignColors::hex_with_alpha for DRY
#[inline]
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    DesignColors::hex_with_alpha(hex, alpha)
}

/// Grouped action item for variable-height list rendering
/// Section headers are 24px, action items are 44px
#[derive(Clone, Debug)]
pub enum GroupedActionItem {
    /// A section header (e.g., "Actions", "Navigation")
    SectionHeader(String),
    /// An action item - usize is the index in filtered_actions
    Item(usize),
}

/// Coerce action selection to skip section headers during navigation
///
/// When the given index lands on a header:
/// 1. First tries searching DOWN to find the next Item
/// 2. If not found, searches UP to find the previous Item
/// 3. If still not found, returns None
fn coerce_action_selection(rows: &[GroupedActionItem], ix: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    let ix = ix.min(rows.len() - 1);

    // If already on a selectable item, done
    if matches!(rows[ix], GroupedActionItem::Item(_)) {
        return Some(ix);
    }

    // Search down for next selectable
    for (j, item) in rows.iter().enumerate().skip(ix + 1) {
        if matches!(item, GroupedActionItem::Item(_)) {
            return Some(j);
        }
    }

    // Search up for previous selectable
    for (j, item) in rows.iter().enumerate().take(ix).rev() {
        if matches!(item, GroupedActionItem::Item(_)) {
            return Some(j);
        }
    }

    None
}

/// Build grouped items from actions and filtered_actions
/// This is a static helper used during construction to avoid borrowing issues
fn build_grouped_items_static(
    actions: &[Action],
    filtered_actions: &[usize],
    section_style: SectionStyle,
) -> Vec<GroupedActionItem> {
    let mut grouped = Vec::new();

    if filtered_actions.is_empty() {
        return grouped;
    }

    let mut prev_section: Option<String> = None;
    let mut prev_category: Option<ActionCategory> = None;

    for (filter_idx, &action_idx) in filtered_actions.iter().enumerate() {
        if let Some(action) = actions.get(action_idx) {
            match section_style {
                SectionStyle::Headers => {
                    // Add section header when section changes
                    if let Some(ref section) = action.section {
                        if prev_section.as_ref() != Some(section) {
                            grouped.push(GroupedActionItem::SectionHeader(section.clone()));
                            prev_section = Some(section.clone());
                        }
                    }
                }
                SectionStyle::Separators | SectionStyle::None => {
                    // For separators, we track category changes but don't add headers
                    // (separators are rendered inline in the item renderer)
                    prev_category = Some(action.category.clone());
                }
            }
            grouped.push(GroupedActionItem::Item(filter_idx));
        }
    }

    // Suppress unused variable warning
    let _ = prev_category;

    grouped
}

/// ActionsDialog - Compact overlay popup for quick actions
/// Implements Raycast-style design with individual keycap shortcuts
///
/// # Configuration
/// Use `ActionsDialogConfig` to customize appearance:
/// - `search_position`: Top (AI chat style) or Bottom (main menu style)
/// - `section_style`: Headers (text labels) or Separators (subtle lines)
/// - `anchor`: Top (list grows down) or Bottom (list grows up)
/// - `show_icons`: Display icons next to actions
/// - `show_footer`: Show keyboard hint footer
pub struct ActionsDialog {
    pub actions: Vec<Action>,
    pub filtered_actions: Vec<usize>, // Indices into actions
    pub selected_index: usize,        // Index within grouped_items (visual row index)
    pub search_text: String,
    pub focus_handle: FocusHandle,
    pub on_select: ActionCallback,
    /// Currently focused script for context-aware actions
    pub focused_script: Option<ScriptInfo>,
    /// Currently focused scriptlet (for H3-defined custom actions)
    pub focused_scriptlet: Option<Scriptlet>,
    /// List state for variable-height list (section headers 24px, items 44px)
    pub list_state: ListState,
    /// Grouped items for list rendering (includes section headers)
    pub grouped_items: Vec<GroupedActionItem>,
    /// Theme for consistent color styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling (defaults to Default for theme-based styling)
    pub design_variant: DesignVariant,
    /// Cursor visibility for blinking (controlled externally)
    pub cursor_visible: bool,
    /// When true, hide the search input (used when rendered inline in main.rs header)
    pub hide_search: bool,
    /// SDK-provided actions (when present, replaces built-in actions)
    pub sdk_actions: Option<Vec<ProtocolAction>>,
    /// Context title shown in the header (e.g., "Activity Monitor", script name)
    pub context_title: Option<String>,
    /// Configuration for appearance and behavior
    pub config: ActionsDialogConfig,
    /// When true, skip track_focus in render (parent handles focus, e.g., ActionsWindow)
    pub skip_track_focus: bool,
}

impl ActionsDialog {
    pub fn new(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        Self::with_script_and_design(focus_handle, on_select, None, theme, DesignVariant::Default)
    }

    pub fn with_script(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        focused_script: Option<ScriptInfo>,
        theme: Arc<theme::Theme>,
    ) -> Self {
        Self::with_script_and_design(
            focus_handle,
            on_select,
            focused_script,
            theme,
            DesignVariant::Default,
        )
    }

    pub fn with_design(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        Self::with_script_and_design(focus_handle, on_select, None, theme, design_variant)
    }

    /// Create ActionsDialog for a path (file/folder) with path-specific actions
    pub fn with_path(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        path_info: &PathInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_path_context_actions(path_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let config = ActionsDialogConfig::default();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(grouped_items.len(), ListAlignment::Top, px(100.));

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for path: {} (is_dir={}) with {} actions",
                path_info.path,
                path_info.is_dir,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            list_state,
            grouped_items,
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(path_info.path.clone()),
            config,
            skip_track_focus: false,
        }
    }

    /// Create ActionsDialog for a file search result with file-specific actions
    /// Actions: Open, Show in Finder, Quick Look, Open With..., Show Info, Copy Path
    pub fn with_file(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        file_info: &FileInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_file_context_actions(file_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let config = ActionsDialogConfig::default();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(grouped_items.len(), ListAlignment::Top, px(100.));

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for file: {} (is_dir={}) with {} actions",
                file_info.path,
                file_info.is_dir,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            list_state,
            grouped_items,
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(file_info.name.clone()),
            config,
            skip_track_focus: false,
        }
    }

    /// Create ActionsDialog for a clipboard history entry with clipboard-specific actions
    /// Actions: Paste, Copy, Paste and Keep Open, Share, Attach to AI, Pin/Unpin, Delete, etc.
    pub fn with_clipboard_entry(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        entry_info: &ClipboardEntryInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_clipboard_history_context_actions(entry_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let config = ActionsDialogConfig::default();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(grouped_items.len(), ListAlignment::Top, px(100.));

        let context_title = if entry_info.preview.len() > 30 {
            format!("{}...", &entry_info.preview[..27])
        } else {
            entry_info.preview.clone()
        };

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for clipboard entry: {} (type={:?}, pinned={}) with {} actions",
                entry_info.id,
                entry_info.content_type,
                entry_info.pinned,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            list_state,
            grouped_items,
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(context_title),
            config,
            skip_track_focus: false,
        }
    }

    /// Create ActionsDialog for a chat prompt with chat-specific actions
    /// Actions: Model selection, Continue in Chat, Copy Response, Clear Conversation
    pub fn with_chat(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        chat_info: &ChatPromptInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_chat_context_actions(chat_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let config = ActionsDialogConfig::default();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(grouped_items.len(), ListAlignment::Top, px(100.));

        let context_title = chat_info
            .current_model
            .clone()
            .unwrap_or_else(|| "Chat".to_string());

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for chat prompt: model={:?} with {} actions",
                chat_info.current_model,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            list_state,
            grouped_items,
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(context_title),
            config,
            skip_track_focus: false,
        }
    }

    pub fn with_script_and_design(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        focused_script: Option<ScriptInfo>,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        let actions = Self::build_actions(&focused_script, &None);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let config = ActionsDialogConfig::default();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(grouped_items.len(), ListAlignment::Top, px(100.));

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created with {} actions, script: {:?}, design: {:?}",
                actions.len(),
                focused_script.as_ref().map(|s| &s.name),
                design_variant
            ),
        );

        // Log theme color configuration for debugging
        logging::log("ACTIONS_THEME", &format!(
            "Theme colors applied: bg_main=#{:06x}, bg_search=#{:06x}, text_primary=#{:06x}, accent_selected=#{:06x}",
            theme.colors.background.main,
            theme.colors.background.search_box,
            theme.colors.text.primary,
            theme.colors.accent.selected
        ));

        // Extract context title from focused script if available
        let context_title = focused_script.as_ref().map(|s| s.name.clone());

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script,
            focused_scriptlet: None,
            list_state,
            grouped_items,
            theme,
            design_variant,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title,
            config,
            skip_track_focus: false,
        }
    }

    /// Update cursor visibility (called from parent's blink timer)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Hide the search input (for inline mode where header has search)
    pub fn set_hide_search(&mut self, hide: bool) {
        self.hide_search = hide;
    }

    /// Set the context title shown in the header
    pub fn set_context_title(&mut self, title: Option<String>) {
        self.context_title = title;
    }

    /// Set the configuration for appearance and behavior
    pub fn set_config(&mut self, config: ActionsDialogConfig) {
        self.config = config;
        // Update hide_search based on config for backwards compatibility
        self.hide_search = matches!(self.config.search_position, SearchPosition::Hidden);
    }

    /// Set skip_track_focus to let parent handle focus (used by ActionsWindow)
    pub fn set_skip_track_focus(&mut self, skip: bool) {
        self.skip_track_focus = skip;
    }

    /// Create ActionsDialog with custom configuration and actions
    ///
    /// Use this for contexts like AI chat that need different appearance:
    /// - Search at top instead of bottom
    /// - Section headers instead of separators
    /// - Icons next to actions
    pub fn with_config(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        actions: Vec<Action>,
        theme: Arc<theme::Theme>,
        config: ActionsDialogConfig,
    ) -> Self {
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(grouped_items.len(), ListAlignment::Top, px(100.));

        // Coerce initial selection to skip section headers
        let initial_selection = coerce_action_selection(&grouped_items, 0).unwrap_or(0);

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created with config: {} actions, search={:?}, section_style={:?}, initial_selection={}",
                actions.len(),
                config.search_position,
                config.section_style,
                initial_selection
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: initial_selection,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            list_state,
            grouped_items,
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: matches!(config.search_position, SearchPosition::Hidden),
            sdk_actions: None,
            context_title: None,
            config,
            skip_track_focus: false,
        }
    }

    /// Parse a shortcut string into individual keycap characters
    /// e.g., "⌘↵" → vec!["⌘", "↵"], "⌘I" → vec!["⌘", "I"]
    fn parse_shortcut_keycaps(shortcut: &str) -> Vec<String> {
        let mut keycaps = Vec::new();

        for ch in shortcut.chars() {
            // Handle modifier symbols (single character)
            match ch {
                '⌘' | '⌃' | '⌥' | '⇧' | '↵' | '⎋' | '⇥' | '⌫' | '␣' | '↑' | '↓' | '←' | '→' =>
                {
                    keycaps.push(ch.to_string());
                }
                // Regular characters (letters, numbers)
                _ => {
                    keycaps.push(ch.to_uppercase().to_string());
                }
            }
        }

        keycaps
    }

    /// Set actions from SDK (replaces built-in actions)
    ///
    /// Converts `ProtocolAction` items to internal `Action` format and updates
    /// the actions list. Filters out actions with `visible: false`.
    /// The `has_action` field on each action determines routing:
    /// - `has_action=true`: Send ActionTriggered back to SDK
    /// - `has_action=false`: Submit value directly
    pub fn set_sdk_actions(&mut self, actions: Vec<ProtocolAction>) {
        let total_count = actions.len();
        let visible_actions: Vec<&ProtocolAction> =
            actions.iter().filter(|a| a.is_visible()).collect();
        let visible_count = visible_actions.len();

        let converted: Vec<Action> = visible_actions
            .iter()
            .map(|pa| Action {
                id: pa.name.clone(),
                title: pa.name.clone(),
                description: pa.description.clone(),
                category: ActionCategory::ScriptContext,
                shortcut: pa.shortcut.as_ref().map(|s| Self::format_shortcut_hint(s)),
                has_action: pa.has_action,
                value: pa.value.clone(),
                icon: None,    // SDK actions don't currently have icons
                section: None, // SDK actions don't currently have sections
            })
            .collect();

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
        // Rebuild grouped items and reset selection
        self.rebuild_grouped_items();
        self.selected_index = coerce_action_selection(&self.grouped_items, 0).unwrap_or(0);
    }

    /// Format a keyboard shortcut for display (e.g., "cmd+c" → "⌘C")
    fn format_shortcut_hint(shortcut: &str) -> String {
        let mut result = String::new();
        let parts: Vec<&str> = shortcut.split('+').collect();

        for (i, part) in parts.iter().enumerate() {
            let part_lower = part.trim().to_lowercase();
            let formatted = match part_lower.as_str() {
                // Modifier keys → symbols
                "cmd" | "command" | "meta" | "super" => "⌘",
                "ctrl" | "control" => "⌃",
                "alt" | "opt" | "option" => "⌥",
                "shift" => "⇧",
                // Special keys
                "enter" | "return" => "↵",
                "escape" | "esc" => "⎋",
                "tab" => "⇥",
                "backspace" | "delete" => "⌫",
                "space" => "␣",
                "up" | "arrowup" => "↑",
                "down" | "arrowdown" => "↓",
                "left" | "arrowleft" => "←",
                "right" | "arrowright" => "→",
                // Regular letters/numbers → uppercase
                _ => {
                    // Check if it's the last part (the actual key)
                    if i == parts.len() - 1 {
                        // Uppercase single characters, keep others as-is
                        result.push_str(&part.trim().to_uppercase());
                        continue;
                    }
                    part.trim()
                }
            };
            result.push_str(formatted);
        }

        result
    }

    /// Clear SDK actions and restore built-in actions
    pub fn clear_sdk_actions(&mut self) {
        if self.sdk_actions.is_some() {
            logging::log(
                "ACTIONS",
                "Clearing SDK actions, restoring built-in actions",
            );
            self.sdk_actions = None;
            self.actions = Self::build_actions(&self.focused_script, &self.focused_scriptlet);
            self.filtered_actions = (0..self.actions.len()).collect();
            self.search_text.clear();
            // Rebuild grouped items and reset selection
            self.rebuild_grouped_items();
            self.selected_index = coerce_action_selection(&self.grouped_items, 0).unwrap_or(0);
        }
    }

    /// Check if SDK actions are currently active
    pub fn has_sdk_actions(&self) -> bool {
        self.sdk_actions.is_some()
    }

    /// Get the currently selected action (for external handling)
    pub fn get_selected_action(&self) -> Option<&Action> {
        // Get the filter_idx from grouped_items, then action_idx from filtered_actions
        match self.grouped_items.get(self.selected_index) {
            Some(GroupedActionItem::Item(filter_idx)) => self
                .filtered_actions
                .get(*filter_idx)
                .and_then(|&idx| self.actions.get(idx)),
            _ => None, // Section headers are not selectable
        }
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
        logging::log("ACTIONS_THEME", "Theme updated in ActionsDialog");
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
        let previously_selected = self
            .filtered_actions
            .get(self.selected_index)
            .and_then(|&idx| self.actions.get(idx).map(|a| a.id.clone()));

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

        // Preserve selection if the same action is still in results
        if let Some(prev_id) = previously_selected {
            if let Some(new_idx) = self.filtered_actions.iter().position(|&idx| {
                self.actions
                    .get(idx)
                    .map(|a| a.id == prev_id)
                    .unwrap_or(false)
            }) {
                self.selected_index = new_idx;
            } else {
                self.selected_index = 0;
            }
        } else {
            self.selected_index = 0;
        }

        // Rebuild grouped items after filter change
        self.rebuild_grouped_items();

        // Coerce selection to skip section headers
        if let Some(valid_idx) = coerce_action_selection(&self.grouped_items, self.selected_index) {
            self.selected_index = valid_idx;
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
    fn score_action(action: &Action, search_lower: &str) -> i32 {
        let title_lower = action.title.to_lowercase();
        let mut score = 0;

        // Prefix match on title (strongest)
        if title_lower.starts_with(search_lower) {
            score += 100;
        }
        // Contains match on title
        else if title_lower.contains(search_lower) {
            score += 50;
        }
        // Fuzzy match on title (character-by-character subsequence)
        else if Self::fuzzy_match(&title_lower, search_lower) {
            score += 25;
        }

        // Description match (bonus)
        if let Some(ref desc) = action.description {
            let desc_lower = desc.to_lowercase();
            if desc_lower.contains(search_lower) {
                score += 15;
            }
        }

        // Shortcut match (bonus)
        if let Some(ref shortcut) = action.shortcut {
            if shortcut.to_lowercase().contains(search_lower) {
                score += 10;
            }
        }

        score
    }

    /// Simple fuzzy matching: check if all characters in needle appear in haystack in order.
    fn fuzzy_match(haystack: &str, needle: &str) -> bool {
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

    /// Move selection up, skipping section headers
    ///
    /// When moving up and landing on a section header, we must search UPWARD
    /// (not downward) to find the previous selectable item. This ensures
    /// navigation past section headers works correctly.
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index == 0 {
            return;
        }

        // Search backwards from current position to find the previous selectable item
        // This correctly skips section headers when moving up
        for i in (0..self.selected_index).rev() {
            if matches!(self.grouped_items.get(i), Some(GroupedActionItem::Item(_))) {
                self.selected_index = i;
                self.list_state.scroll_to_reveal_item(self.selected_index);
                logging::log_debug(
                    "ACTIONS_SCROLL",
                    &format!("Up: selected_index={}", self.selected_index),
                );
                cx.notify();
                return;
            }
        }
    }

    /// Move selection down, skipping section headers
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.grouped_items.len().saturating_sub(1) {
            let new_index = self.selected_index + 1;
            // Skip section headers - search forward
            for i in new_index..self.grouped_items.len() {
                if matches!(self.grouped_items.get(i), Some(GroupedActionItem::Item(_))) {
                    self.selected_index = i;
                    self.list_state.scroll_to_reveal_item(self.selected_index);
                    logging::log_debug(
                        "ACTIONS_SCROLL",
                        &format!("Down: selected_index={}", self.selected_index),
                    );
                    cx.notify();
                    break;
                }
            }
        }
    }

    /// Get the currently selected action ID (for external handling)
    pub fn get_selected_action_id(&self) -> Option<String> {
        // Get the filter_idx from grouped_items, then action_idx from filtered_actions
        match self.grouped_items.get(self.selected_index) {
            Some(GroupedActionItem::Item(filter_idx)) => {
                if let Some(&action_idx) = self.filtered_actions.get(*filter_idx) {
                    if let Some(action) = self.actions.get(action_idx) {
                        return Some(action.id.clone());
                    }
                }
                None
            }
            _ => None, // Section headers are not selectable
        }
    }

    /// Get the currently selected ProtocolAction (for checking close behavior)
    /// Returns the original ProtocolAction from sdk_actions if this is an SDK action,
    /// or None for built-in actions.
    pub fn get_selected_protocol_action(&self) -> Option<&ProtocolAction> {
        let action_id = self.get_selected_action_id()?;
        self.sdk_actions
            .as_ref()?
            .iter()
            .find(|a| a.name == action_id)
    }

    /// Check if the currently selected action should close the dialog
    /// Returns true if the action has close: true (or no close field, which defaults to true)
    /// Returns true for built-in actions (they always close)
    pub fn selected_action_should_close(&self) -> bool {
        if let Some(protocol_action) = self.get_selected_protocol_action() {
            protocol_action.should_close()
        } else {
            // Built-in actions always close
            true
        }
    }

    /// Submit the selected action
    pub fn submit_selected(&mut self) {
        // Get action from grouped_items -> filtered_actions -> actions chain
        if let Some(action) = self.get_selected_action() {
            let action_id = action.id.clone();
            logging::log("ACTIONS", &format!("Action selected: {}", action_id));
            (self.on_select)(action_id);
        }
    }

    /// Cancel - close the dialog
    pub fn submit_cancel(&mut self) {
        logging::log("ACTIONS", "Actions dialog cancelled");
        (self.on_select)("__cancel__".to_string());
    }

    /// Dismiss the dialog when user clicks outside its bounds.
    /// This is a public method called from the parent container's click-outside handler.
    /// Logs the event and triggers the cancel callback.
    pub fn dismiss_on_click_outside(&mut self) {
        tracing::info!(
            target: "script_kit::actions",
            "ActionsDialog dismiss-on-click-outside triggered"
        );
        logging::log("ACTIONS", "Actions dialog dismissed (click outside)");
        self.submit_cancel();
    }

    /// Create box shadow for the overlay popup
    /// When rendered in a separate vibrancy window, no shadow is needed
    /// (the window vibrancy provides visual separation)
    pub(super) fn create_popup_shadow() -> Vec<BoxShadow> {
        // No shadow - vibrancy window provides visual separation
        vec![]
    }

    /// Get colors for the search box based on design variant
    /// Returns: (search_box_bg, border_color, muted_text, dimmed_text, secondary_text)
    pub(super) fn get_search_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        // Use theme opacity for input background to support vibrancy
        let opacity = self.theme.get_opacity();
        let input_alpha = (opacity.input * 255.0) as u8;

        if self.design_variant == DesignVariant::Default {
            (
                rgba(hex_with_alpha(
                    self.theme.colors.background.search_box,
                    input_alpha,
                )),
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x80)),
                rgb(self.theme.colors.text.muted),
                rgb(self.theme.colors.text.dimmed),
                rgb(self.theme.colors.text.secondary),
            )
        } else {
            (
                rgba(hex_with_alpha(colors.background_secondary, input_alpha)),
                rgba(hex_with_alpha(colors.border, 0x80)),
                rgb(colors.text_muted),
                rgb(colors.text_dimmed),
                rgb(colors.text_secondary),
            )
        }
    }

    /// Get colors for the main container based on design variant
    /// Returns: (main_bg, container_border, container_text)
    pub(super) fn get_container_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        // Vibrancy-aware dialog background:
        // - When vibrancy enabled: ~50% opacity to show blur but remain visible
        // - When vibrancy disabled: ~95% opacity for near-solid appearance
        let use_vibrancy = self.theme.is_vibrancy_enabled();
        let dialog_alpha = if use_vibrancy {
            // Dialogs need higher opacity than main window (0.37) to stand out
            (0.50 * 255.0) as u8
        } else {
            // Near-opaque when vibrancy disabled
            (0.95 * 255.0) as u8
        };

        if self.design_variant == DesignVariant::Default {
            (
                rgba(hex_with_alpha(
                    self.theme.colors.background.main,
                    dialog_alpha,
                )),
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x80)),
                rgb(self.theme.colors.text.secondary),
            )
        } else {
            (
                rgba(hex_with_alpha(colors.background, dialog_alpha)),
                rgba(hex_with_alpha(colors.border, 0x80)),
                rgb(colors.text_secondary),
            )
        }
    }
}

impl Focusable for ActionsDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ActionsDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get design tokens for the current design variant
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let visual = tokens.visual();

        // NOTE: Key handling is done by the parent (ScriptListApp in main.rs)
        // which routes all keyboard events to this dialog's methods.
        // We do NOT attach our own on_key_down handler to avoid double-processing.

        // Render search input - compact version
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Use helper method for design/theme color extraction
        let (search_box_bg, border_color, _muted_text, dimmed_text, _secondary_text) =
            self.get_search_colors(&colors);

        // Get primary text color for cursor (matches main list styling)
        let primary_text = if self.design_variant == DesignVariant::Default {
            rgb(self.theme.colors.text.primary)
        } else {
            rgb(colors.text_primary)
        };

        // Get accent color for the search input focus indicator
        let accent_color_hex = if self.design_variant == DesignVariant::Default {
            self.theme.colors.accent.selected
        } else {
            colors.accent
        };
        let accent_color = rgb(accent_color_hex);

        // Focus border color (accent with transparency)
        let focus_border_color = rgba(hex_with_alpha(accent_color_hex, 0x60));

        // Input container with fixed height and width to prevent any layout shifts
        // The entire row is constrained to prevent resizing when text is entered
        let input_container = div()
            .w(px(POPUP_WIDTH)) // Match parent width exactly
            .min_w(px(POPUP_WIDTH))
            .max_w(px(POPUP_WIDTH))
            .h(px(SEARCH_INPUT_HEIGHT)) // Fixed height for the input row
            .min_h(px(SEARCH_INPUT_HEIGHT))
            .max_h(px(SEARCH_INPUT_HEIGHT))
            .overflow_hidden() // Prevent any content from causing shifts
            .px(px(spacing.item_padding_x))
            .py(px(spacing.item_padding_y + 2.0)) // Slightly more vertical padding
            .bg(search_box_bg)
            .border_t_1() // Border on top since input is now at bottom
            .border_color(border_color)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.gap_md))
            .child(
                // Search icon or indicator - fixed width to prevent shifts
                div()
                    .w(px(24.0)) // Fixed width for the icon container
                    .min_w(px(24.0))
                    .text_color(dimmed_text)
                    .text_xs()
                    .child("⌘K"),
            )
            .child(
                // Search input field with focus indicator
                // CRITICAL: Use flex_shrink_0 to prevent flexbox from shrinking this container
                // The border/bg MUST stay at fixed width regardless of content
                div()
                    .flex_shrink_0() // PREVENT flexbox from shrinking this!
                    .w(px(240.0))
                    .min_w(px(240.0))
                    .max_w(px(240.0))
                    .h(px(28.0)) // Fixed height too
                    .min_h(px(28.0))
                    .max_h(px(28.0))
                    .overflow_hidden()
                    .px(px(spacing.padding_sm))
                    .py(px(spacing.padding_xs))
                    // ALWAYS show background - just vary intensity
                    .bg(if self.design_variant == DesignVariant::Default {
                        rgba(hex_with_alpha(
                            self.theme.colors.background.main,
                            if self.search_text.is_empty() {
                                0x20
                            } else {
                                0x40
                            },
                        ))
                    } else {
                        rgba(hex_with_alpha(
                            colors.background,
                            if self.search_text.is_empty() {
                                0x20
                            } else {
                                0x40
                            },
                        ))
                    })
                    .rounded(px(visual.radius_sm))
                    .border_1()
                    // ALWAYS show border - just vary intensity
                    .border_color(if !self.search_text.is_empty() {
                        focus_border_color
                    } else {
                        border_color
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    .text_color(if self.search_text.is_empty() {
                        dimmed_text
                    } else {
                        primary_text
                    })
                    // ALWAYS render cursor div with consistent margin to prevent layout shift
                    // When empty, cursor is at the start before placeholder text
                    .when(self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .mr(px(2.)) // Use consistent 2px margin
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    })
                    .child(search_display.clone())
                    // When has text, cursor is at the end after the text
                    .when(!self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .ml(px(2.)) // Consistent 2px margin
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    }),
            );

        // Render action list using list() for variable-height items
        // Section headers are 24px, action items are 44px
        let actions_container = if self.grouped_items.is_empty() {
            // Empty state: fixed height matching one action item row
            div()
                .w_full()
                .h(px(ACTION_ITEM_HEIGHT))
                .flex()
                .items_center()
                .px(px(spacing.item_padding_x))
                .text_color(dimmed_text)
                .text_sm()
                .child("No actions match your search")
                .into_any_element()
        } else {
            // Clone data needed for the list closure
            let grouped_items_clone = self.grouped_items.clone();
            let design_variant = self.design_variant;

            // Calculate scrollbar parameters
            // Container height for actions (excluding search box)
            let search_box_height = if self.hide_search {
                0.0
            } else {
                SEARCH_INPUT_HEIGHT
            };

            // Count section headers and items for accurate height calculation
            let mut header_count = 0_usize;
            let mut item_count = 0_usize;
            for item in &self.grouped_items {
                match item {
                    GroupedActionItem::SectionHeader(_) => header_count += 1,
                    GroupedActionItem::Item(_) => item_count += 1,
                }
            }
            let total_content_height = (header_count as f32 * SECTION_HEADER_HEIGHT)
                + (item_count as f32 * ACTION_ITEM_HEIGHT);
            let container_height = total_content_height.min(POPUP_MAX_HEIGHT - search_box_height);

            // Estimate visible items based on average item height
            let avg_item_height = if self.grouped_items.is_empty() {
                ACTION_ITEM_HEIGHT
            } else {
                total_content_height / self.grouped_items.len() as f32
            };
            let visible_items = (container_height / avg_item_height).ceil() as usize;

            // Get scroll offset from list state
            let scroll_offset = self.list_state.logical_scroll_top().item_ix;

            // Get scrollbar colors from theme for consistent styling
            let scrollbar_colors = ScrollbarColors::from_theme(&self.theme);

            // Create scrollbar (only visible if content overflows)
            let scrollbar = Scrollbar::new(
                self.grouped_items.len(),
                visible_items,
                scroll_offset,
                scrollbar_colors,
            )
            .container_height(container_height);

            // Capture entity handle for use in the render closure
            let entity = cx.entity();

            let variable_height_list = list(self.list_state.clone(), move |ix, _window, cx| {
                // Access entity state inside the closure
                entity.update(cx, |this, _cx| {
                    let current_selected = this.selected_index;

                    if let Some(grouped_item) = grouped_items_clone.get(ix) {
                        match grouped_item {
                            GroupedActionItem::SectionHeader(label) => {
                                // Section header at 24px height
                                let header_text = if this.design_variant == DesignVariant::Default {
                                    rgb(this.theme.colors.text.dimmed)
                                } else {
                                    let tokens = get_tokens(this.design_variant);
                                    rgb(tokens.colors().text_dimmed)
                                };
                                let border_color = if this.design_variant == DesignVariant::Default
                                {
                                    rgba(hex_with_alpha(this.theme.colors.ui.border, 0x40))
                                } else {
                                    let tokens = get_tokens(this.design_variant);
                                    rgba(hex_with_alpha(tokens.colors().border, 0x40))
                                };

                                div()
                                    .id(ElementId::NamedInteger("section-header".into(), ix as u64))
                                    .h(px(SECTION_HEADER_HEIGHT))
                                    .w_full()
                                    .px(px(16.0))
                                    .flex()
                                    .items_center()
                                    .when(ix > 0, |d| d.border_t_1().border_color(border_color))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(header_text)
                                            .child(label.clone()),
                                    )
                                    .into_any_element()
                            }
                            GroupedActionItem::Item(filter_idx) => {
                                // Action item at 44px height
                                if let Some(&action_idx) = this.filtered_actions.get(*filter_idx) {
                                    if let Some(action) = this.actions.get(action_idx) {
                                        let is_selected = ix == current_selected;

                                        // Get tokens for styling
                                        let item_tokens = get_tokens(design_variant);
                                        let item_colors = item_tokens.colors();
                                        let item_spacing = item_tokens.spacing();

                                        // Extract colors for list items - use theme opacity for vibrancy
                                        let theme_opacity = this.theme.get_opacity();
                                        let selected_alpha =
                                            (theme_opacity.selected * 255.0) as u32;
                                        let hover_alpha = (theme_opacity.hover * 255.0) as u32;

                                        let (
                                            selected_bg,
                                            hover_bg,
                                            primary_text,
                                            secondary_text,
                                            dimmed_text,
                                        ) = if design_variant == DesignVariant::Default {
                                            (
                                                rgba(
                                                    (this.theme.colors.accent.selected_subtle << 8)
                                                        | selected_alpha,
                                                ),
                                                rgba(
                                                    (this.theme.colors.accent.selected_subtle << 8)
                                                        | hover_alpha,
                                                ),
                                                rgb(this.theme.colors.text.primary),
                                                rgb(this.theme.colors.text.secondary),
                                                rgb(this.theme.colors.text.dimmed),
                                            )
                                        } else {
                                            (
                                                rgba(
                                                    (item_colors.background_selected << 8)
                                                        | selected_alpha,
                                                ),
                                                rgba(
                                                    (item_colors.background_selected << 8)
                                                        | hover_alpha,
                                                ),
                                                rgb(item_colors.text_primary),
                                                rgb(item_colors.text_secondary),
                                                rgb(item_colors.text_dimmed),
                                            )
                                        };

                                        // Title color: bright when selected, secondary when not
                                        let title_color = if is_selected {
                                            primary_text
                                        } else {
                                            secondary_text
                                        };
                                        let shortcut_color = dimmed_text;

                                        // Keycap colors
                                        let keycap_bg = if design_variant == DesignVariant::Default
                                        {
                                            rgba(hex_with_alpha(this.theme.colors.ui.border, 0x80))
                                        } else {
                                            rgba(hex_with_alpha(item_colors.border, 0x80))
                                        };
                                        let keycap_border = if design_variant
                                            == DesignVariant::Default
                                        {
                                            rgba(hex_with_alpha(this.theme.colors.ui.border, 0xA0))
                                        } else {
                                            rgba(hex_with_alpha(item_colors.border, 0xA0))
                                        };

                                        // Inner row with pill-style selection
                                        let inner_row = div()
                                            .w_full()
                                            .flex_1()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .px(px(item_spacing.item_padding_x))
                                            .rounded(px(SELECTION_RADIUS))
                                            .bg(if is_selected {
                                                selected_bg
                                            } else {
                                                rgba(0x00000000)
                                            })
                                            .hover(|s| s.bg(hover_bg))
                                            .cursor_pointer();

                                        // Content: optional icon + label + shortcuts
                                        let show_icons = this.config.show_icons;
                                        let action_icon = action.icon;

                                        let mut left_side =
                                            div().flex().flex_row().items_center().gap(px(12.0));

                                        // Add icon if enabled and present
                                        if show_icons {
                                            if let Some(icon) = action_icon {
                                                left_side = left_side.child(
                                                    svg()
                                                        .external_path(icon.external_path())
                                                        .size(px(16.0))
                                                        .text_color(if is_selected {
                                                            primary_text
                                                        } else {
                                                            dimmed_text
                                                        }),
                                                );
                                            }
                                        }

                                        // Add title
                                        left_side = left_side.child(
                                            div()
                                                .text_color(title_color)
                                                .text_sm()
                                                .font_weight(if is_selected {
                                                    gpui::FontWeight::MEDIUM
                                                } else {
                                                    gpui::FontWeight::NORMAL
                                                })
                                                .child(action.title.clone()),
                                        );

                                        let mut content = div()
                                            .flex_1()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .child(left_side);

                                        // Right side: keyboard shortcuts as keycaps
                                        if let Some(ref shortcut) = action.shortcut {
                                            let keycaps =
                                                ActionsDialog::parse_shortcut_keycaps(shortcut);
                                            let mut keycap_row =
                                                div().flex().flex_row().items_center().gap(px(3.));

                                            for keycap in keycaps {
                                                keycap_row = keycap_row.child(
                                                    div()
                                                        .min_w(px(KEYCAP_MIN_WIDTH))
                                                        .h(px(KEYCAP_HEIGHT))
                                                        .px(px(6.))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .bg(keycap_bg)
                                                        .border_1()
                                                        .border_color(keycap_border)
                                                        .rounded(px(5.))
                                                        .text_xs()
                                                        .text_color(shortcut_color)
                                                        .child(keycap),
                                                );
                                            }

                                            content = content.child(keycap_row);
                                        }

                                        div()
                                            .id(ElementId::NamedInteger(
                                                "action-item".into(),
                                                ix as u64,
                                            ))
                                            .h(px(ACTION_ITEM_HEIGHT))
                                            .w_full()
                                            .px(px(ACTION_ROW_INSET))
                                            .py(px(2.0))
                                            .flex()
                                            .flex_col()
                                            .justify_center()
                                            .child(inner_row.child(content))
                                            .into_any_element()
                                    } else {
                                        // Fallback for missing action
                                        div().h(px(ACTION_ITEM_HEIGHT)).into_any_element()
                                    }
                                } else {
                                    // Fallback for missing filtered index
                                    div().h(px(ACTION_ITEM_HEIGHT)).into_any_element()
                                }
                            }
                        }
                    } else {
                        // Fallback for out-of-bounds index
                        div().h(px(ACTION_ITEM_HEIGHT)).into_any_element()
                    }
                })
            })
            .flex_1()
            .w_full();

            // Wrap list in a relative container with scrollbar overlay
            div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .h_full()
                .overflow_hidden()
                .child(variable_height_list)
                .child(scrollbar)
                .into_any_element()
        };

        // Use helper method for container colors
        let (main_bg, container_border, container_text) = self.get_container_colors(&colors);

        // Calculate dynamic height based on number of items
        // Each item is ACTION_ITEM_HEIGHT, plus search box height (SEARCH_INPUT_HEIGHT), plus padding
        // When hide_search is true, we don't include the search box height
        // Now also includes HEADER_HEIGHT when context_title is set
        // NOTE: Add border_thin * 2 for border (top + bottom from .border_1()) to prevent
        // content from being clipped and causing unnecessary scrolling
        let num_items = self.filtered_actions.len();
        let search_box_height = if self.hide_search {
            0.0
        } else {
            SEARCH_INPUT_HEIGHT
        };
        let header_height = if self.context_title.is_some() {
            HEADER_HEIGHT
        } else {
            0.0
        };
        let border_height = visual.border_thin * 2.0; // top + bottom border
                                                      // When no actions, still need space for "No actions match" message
        let min_items_height = if num_items == 0 {
            ACTION_ITEM_HEIGHT
        } else {
            0.0
        };
        let items_height = (num_items as f32 * ACTION_ITEM_HEIGHT)
            .max(min_items_height)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let total_height = items_height + search_box_height + header_height + border_height;

        // Build header row (section header style - non-interactive label)
        // Styled to match render_section_header() from list_item.rs:
        // - Smaller font (text_xs)
        // - Semibold weight
        // - Dimmed color (visually distinct from actionable items)
        let header_container = self.context_title.as_ref().map(|title| {
            let header_text = if self.design_variant == DesignVariant::Default {
                rgb(self.theme.colors.text.dimmed)
            } else {
                rgb(colors.text_dimmed)
            };
            let header_border = if self.design_variant == DesignVariant::Default {
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x40))
            } else {
                rgba(hex_with_alpha(colors.border, 0x40))
            };

            div()
                .w_full()
                .h(px(HEADER_HEIGHT))
                .px(px(16.0)) // Match section header padding from list_item.rs
                .pt(px(8.0)) // Top padding for visual separation
                .pb(px(4.0)) // Bottom padding
                .flex()
                .flex_col()
                .justify_center()
                .border_b_1()
                .border_color(header_border)
                .child(
                    div()
                        .text_xs() // Smaller font like section headers
                        .font_weight(gpui::FontWeight::SEMIBOLD) // Semibold like section headers
                        .text_color(header_text)
                        .child(title.clone()),
                )
        });

        // Main overlay popup container
        // Fixed width, dynamic height based on content, rounded corners, shadow
        // NOTE: Using visual.radius_lg from design tokens for consistency with child item rounding
        //
        // VIBRANCY: Background is handled in get_container_colors() with vibrancy-aware opacity
        // (~50% when vibrancy enabled, ~95% when disabled)

        // Build footer with keyboard hints (if enabled)
        let footer_height = if self.config.show_footer { 32.0 } else { 0.0 };
        let footer_container = if self.config.show_footer {
            let footer_text = if self.design_variant == DesignVariant::Default {
                rgb(self.theme.colors.text.dimmed)
            } else {
                rgb(colors.text_dimmed)
            };
            let footer_border = if self.design_variant == DesignVariant::Default {
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x40))
            } else {
                rgba(hex_with_alpha(colors.border, 0x40))
            };

            Some(
                div()
                    .w_full()
                    .h(px(32.0))
                    .px(px(16.0))
                    .border_t_1()
                    .border_color(footer_border)
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .text_xs()
                    .text_color(footer_text)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("↑↓")
                            .child("Navigate"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("↵")
                            .child("Select"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("esc")
                            .child("Close"),
                    ),
            )
        } else {
            None
        };

        // Recalculate total height including footer
        let total_height = total_height + footer_height;

        // Get search position from config
        let search_at_top = matches!(self.config.search_position, SearchPosition::Top);
        let show_search =
            !matches!(self.config.search_position, SearchPosition::Hidden) && !self.hide_search;

        // Modify input_container for top position (border at bottom instead of top)
        let input_container_top = if search_at_top && show_search {
            Some(
                div()
                    .w(px(POPUP_WIDTH))
                    .min_w(px(POPUP_WIDTH))
                    .max_w(px(POPUP_WIDTH))
                    .h(px(SEARCH_INPUT_HEIGHT))
                    .min_h(px(SEARCH_INPUT_HEIGHT))
                    .max_h(px(SEARCH_INPUT_HEIGHT))
                    .overflow_hidden()
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.item_padding_y + 2.0))
                    .bg(search_box_bg)
                    .border_b_1() // Border at bottom for top-positioned search
                    .border_color(border_color)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.gap_md))
                    .child(
                        div()
                            .w(px(24.0))
                            .min_w(px(24.0))
                            .text_color(dimmed_text)
                            .text_xs()
                            .child("⌘K"),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .w(px(240.0))
                            .min_w(px(240.0))
                            .max_w(px(240.0))
                            .h(px(28.0))
                            .min_h(px(28.0))
                            .max_h(px(28.0))
                            .overflow_hidden()
                            .px(px(spacing.padding_sm))
                            .py(px(spacing.padding_xs))
                            .bg(if self.design_variant == DesignVariant::Default {
                                rgba(hex_with_alpha(
                                    self.theme.colors.background.main,
                                    if self.search_text.is_empty() {
                                        0x20
                                    } else {
                                        0x40
                                    },
                                ))
                            } else {
                                rgba(hex_with_alpha(
                                    colors.background,
                                    if self.search_text.is_empty() {
                                        0x20
                                    } else {
                                        0x40
                                    },
                                ))
                            })
                            .rounded(px(visual.radius_sm))
                            .border_1()
                            .border_color(if !self.search_text.is_empty() {
                                focus_border_color
                            } else {
                                border_color
                            })
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_sm()
                            .text_color(if self.search_text.is_empty() {
                                dimmed_text
                            } else {
                                primary_text
                            })
                            .when(self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .mr(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            })
                            .child(search_display.clone())
                            .when(!self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .ml(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            }),
                    ),
            )
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .w(px(POPUP_WIDTH))
            .h(px(total_height)) // Use calculated height including footer
            .bg(main_bg) // Always apply background with vibrancy-aware opacity
            .rounded(px(visual.radius_lg))
            .shadow(Self::create_popup_shadow())
            .border_1()
            .border_color(container_border)
            .overflow_hidden()
            .text_color(container_text)
            .key_context("actions_dialog")
            // Only track focus if not delegated to parent (ActionsWindow sets skip_track_focus=true)
            .when(!self.skip_track_focus, |d| {
                d.track_focus(&self.focus_handle)
            })
            // NOTE: No on_key_down here - parent handles all keyboard input
            // Search input at top (if config.search_position == Top)
            .when_some(input_container_top, |d, input| d.child(input))
            // Header row (if context_title is set)
            .when_some(header_container, |d, header| d.child(header))
            // Actions list
            .child(actions_container)
            // Search input at bottom (if config.search_position == Bottom)
            .when(show_search && !search_at_top, |d| d.child(input_container))
            // Footer with keyboard hints (if config.show_footer)
            .when_some(footer_container, |d, footer| d.child(footer))
    }
}

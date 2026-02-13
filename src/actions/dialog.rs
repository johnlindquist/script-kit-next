#![allow(dead_code)]

// --- merged from part_01.rs ---
// Actions Dialog
//
// The main ActionsDialog struct and its implementation, providing a searchable
// action menu as a compact overlay popup.

use crate::components::scrollbar::{Scrollbar, ScrollbarColors};
use crate::designs::{get_tokens, DesignColors, DesignVariant};
use crate::logging;
use crate::protocol::ProtocolAction;
use crate::theme;
use gpui::{
    div, list, prelude::*, px, rgb, rgba, svg, App, BoxShadow, Context, ElementId, FocusHandle,
    Focusable, ListAlignment, ListState, Render, SharedString, Window,
};
use std::collections::HashSet;
use std::sync::Arc;

use super::builders::{
    format_shortcut_hint as format_shortcut_hint_shared, get_chat_context_actions,
    get_clipboard_history_context_actions, get_file_context_actions, get_global_actions,
    get_path_context_actions, get_script_context_actions,
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
    Action, ActionCallback, ActionCategory, ActionsDialogConfig, AnchorPosition, CloseCallback,
    ScriptInfo, SearchPosition, SectionStyle,
};
use crate::prompts::PathInfo;

/// Helper function to combine a hex color with an alpha value
/// Delegates to DesignColors::hex_with_alpha for DRY
#[inline]
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    DesignColors::hex_with_alpha(hex, alpha)
}

/// Action subtitle text shown in the popup row, if any.
pub(crate) fn action_subtitle_for_display(action: &Action) -> Option<&str> {
    action.description.as_deref()
}

/// Whether an action should render with destructive styling.
///
/// We key off stable action IDs first, then fall back to title prefixes for
/// dynamic or SDK-defined destructive actions.
pub(crate) fn is_destructive_action(action: &Action) -> bool {
    let id = action.id.as_str();

    if id == "move_to_trash"
        || id == "reset_ranking"
        || id == "clear_conversation"
        || id.starts_with("remove_")
        || id.starts_with("delete_")
        || id.contains("_delete")
        || id.contains("_trash")
    {
        return true;
    }

    action.title_lower.starts_with("remove ")
        || action.title_lower.starts_with("delete ")
        || action.title_lower.starts_with("clear ")
        || action.title_lower.starts_with("move to trash")
}

/// Grouped action item for variable-height list rendering
/// Section headers are 22px, action items are 36px
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
pub(super) fn coerce_action_selection(rows: &[GroupedActionItem], ix: usize) -> Option<usize> {
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

/// Compute the initial selected row for grouped items.
///
/// Constructors should use this helper so initial selection behavior remains
/// consistent across all dialog entry points.
pub(super) fn initial_selection_index(rows: &[GroupedActionItem]) -> usize {
    coerce_action_selection(rows, 0).unwrap_or(0)
}

/// Whether config changes require rebuilding grouped rows.
///
/// Grouped rows depend on section style because `Headers` injects extra rows.
pub(super) fn should_rebuild_grouped_items_for_config_change(
    previous: &ActionsDialogConfig,
    next: &ActionsDialogConfig,
) -> bool {
    previous.section_style != next.section_style
}

/// Resolve a selected protocol action index from the selected visible action index.
///
/// `sdk_action_indices` maps visible action indices to indices in the original
/// SDK protocol action array.
pub(super) fn resolve_selected_protocol_action_index(
    selected_action_index: Option<usize>,
    sdk_action_indices: &[usize],
) -> Option<usize> {
    selected_action_index.and_then(|action_idx| sdk_action_indices.get(action_idx).copied())
}

/// Build grouped items from actions and filtered_actions
/// This is a static helper used during construction to avoid borrowing issues
pub(super) fn build_grouped_items_static(
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

/// Whether a separator line should be shown before a filtered item index.
///
/// Used for `SectionStyle::Separators` so we can visually group sections
/// without injecting explicit header rows.
pub(super) fn should_render_section_separator(
    actions: &[Action],
    filtered_actions: &[usize],
    filter_idx: usize,
) -> bool {
    if filter_idx == 0 {
        return false;
    }

    let current_action = filtered_actions
        .get(filter_idx)
        .and_then(|&idx| actions.get(idx));
    let previous_action = filtered_actions
        .get(filter_idx - 1)
        .and_then(|&idx| actions.get(idx));

    match (previous_action, current_action) {
        (Some(prev), Some(curr)) => prev.section != curr.section,
        _ => false,
    }
}

const ACTIONS_DIALOG_FOOTER_HEIGHT: f32 = 32.0;

/// Calculate the list viewport height used for scrollbar geometry.
///
/// This must mirror popup layout constraints so the scrollbar thumb represents
/// the visible list region (excluding search/header/footer chrome).
pub(super) fn actions_dialog_scrollbar_viewport_height(
    total_content_height: f32,
    show_search: bool,
    has_header: bool,
    show_footer: bool,
) -> f32 {
    let search_height = if show_search {
        SEARCH_INPUT_HEIGHT
    } else {
        0.0
    };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let footer_height = if show_footer {
        ACTIONS_DIALOG_FOOTER_HEIGHT
    } else {
        0.0
    };
    let available_viewport_height =
        (POPUP_MAX_HEIGHT - search_height - header_height - footer_height).max(0.0);

    total_content_height.min(available_viewport_height)
}

/// Resolve empty-state copy based on whether a search query is active.
pub(super) fn actions_dialog_empty_state_message(search_text: &str) -> &'static str {
    if search_text.trim().is_empty() {
        "No actions available"
    } else {
        "No actions match your search"
    }
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
    /// List state for variable-height list (section headers 22px, items 36px)
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
    /// Visible action index -> original protocol action index mapping.
    /// Keeps SDK action resolution deterministic when names collide.
    sdk_action_indices: Vec<usize>,
    /// Context title shown in the header (e.g., "Activity Monitor", script name)
    pub context_title: Option<String>,
    /// Configuration for appearance and behavior
    pub config: ActionsDialogConfig,
    /// When true, skip track_focus in render (parent handles focus, e.g., ActionsWindow)
    pub skip_track_focus: bool,
    /// Callback for when the dialog is closed (escape pressed, window dismissed)
    /// Used to notify the main app to restore focus
    pub on_close: Option<CloseCallback>,
}

#[cfg(test)]
mod empty_state_message_tests {
    use super::actions_dialog_empty_state_message;

    #[test]
    fn test_actions_dialog_empty_state_message_returns_available_when_search_is_empty() {
        assert_eq!(
            actions_dialog_empty_state_message(""),
            "No actions available"
        );
        assert_eq!(
            actions_dialog_empty_state_message("   "),
            "No actions available"
        );
    }

    #[test]
    fn test_actions_dialog_empty_state_message_returns_no_match_when_search_has_text() {
        assert_eq!(
            actions_dialog_empty_state_message("open"),
            "No actions match your search"
        );
    }
}

// --- merged from part_02.rs ---
// --- merged from part_01.rs ---
const ACTIONS_DIALOG_LIST_OVERDRAW_PX: f32 = 100.0;

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

    #[allow(clippy::too_many_arguments)]
    fn from_actions_with_context(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        actions: Vec<Action>,
        focused_script: Option<ScriptInfo>,
        focused_scriptlet: Option<Scriptlet>,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
        context_title: Option<String>,
        config: ActionsDialogConfig,
    ) -> Self {
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(
            grouped_items.len(),
            ListAlignment::Top,
            px(ACTIONS_DIALOG_LIST_OVERDRAW_PX),
        );
        let selected_index = initial_selection_index(&grouped_items);

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script,
            focused_scriptlet,
            list_state,
            grouped_items,
            theme,
            design_variant,
            cursor_visible: true,
            hide_search: matches!(config.search_position, SearchPosition::Hidden),
            sdk_actions: None,
            sdk_action_indices: Vec::new(),
            context_title,
            config,
            skip_track_focus: false,
            on_close: None,
        }
    }

    /// Create ActionsDialog for a path (file/folder) with path-specific actions
    pub fn with_path(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        path_info: &PathInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_path_context_actions(path_info);
        let config = ActionsDialogConfig::default();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for path: {} (is_dir={}) with {} actions",
                path_info.path,
                path_info.is_dir,
                actions.len()
            ),
        );

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(path_info.path.clone()),
            config,
        )
    }

    /// Create ActionsDialog for a file search result with file-specific actions
    /// Actions: Open, Reveal in Finder, Quick Look, Open With..., Show Info, Copy Path
    pub fn with_file(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        file_info: &FileInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_file_context_actions(file_info);
        let config = ActionsDialogConfig::default();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for file: {} (is_dir={}) with {} actions",
                file_info.path,
                file_info.is_dir,
                actions.len()
            ),
        );

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(file_info.name.clone()),
            config,
        )
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
        let config = ActionsDialogConfig::default();

        let context_title = Self::clipboard_context_title(&entry_info.preview);

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

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(context_title),
            config,
        )
    }

    fn clipboard_context_title(preview: &str) -> String {
        const CONTEXT_TITLE_MAX_CHARS: usize = 30;
        const CONTEXT_TITLE_TRUNCATE_CHARS: usize = 27;

        if preview.chars().count() > CONTEXT_TITLE_MAX_CHARS {
            let truncated: String = preview.chars().take(CONTEXT_TITLE_TRUNCATE_CHARS).collect();
            format!("{truncated}...")
        } else {
            preview.to_string()
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
        let config = ActionsDialogConfig::default();

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

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(context_title),
            config,
        )
    }

    pub fn with_script_and_design(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        focused_script: Option<ScriptInfo>,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        let actions = Self::build_actions(&focused_script, &None);
        let config = ActionsDialogConfig::default();

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

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            focused_script,
            None,
            theme,
            design_variant,
            context_title,
            config,
        )
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
        let should_rebuild = should_rebuild_grouped_items_for_config_change(&self.config, &config);
        let previously_selected_action_index = self.selected_action_index();

        self.config = config;
        // Update hide_search based on config for backwards compatibility
        self.hide_search = matches!(self.config.search_position, SearchPosition::Hidden);

        if should_rebuild {
            self.rebuild_grouped_items();
            self.selected_index = previously_selected_action_index
                .and_then(|action_idx| self.grouped_index_for_action_index(action_idx))
                .unwrap_or_else(|| initial_selection_index(&self.grouped_items));
            if !self.grouped_items.is_empty() {
                self.list_state.scroll_to_reveal_item(self.selected_index);
            }
        }
    }

    /// Set skip_track_focus to let parent handle focus (used by ActionsWindow)
    pub fn set_skip_track_focus(&mut self, skip: bool) {
        self.skip_track_focus = skip;
    }

    /// Set the callback for when the dialog is closed (escape pressed, window dismissed)
    /// Used to notify the main app to restore focus
    pub fn set_on_close(&mut self, callback: CloseCallback) {
        self.on_close = Some(callback);
    }

    /// Call the on_close callback if set
    /// Returns true if a callback was called, false otherwise
    pub fn trigger_on_close(&self, cx: &mut gpui::App) -> bool {
        if let Some(ref callback) = self.on_close {
            callback(cx);
            true
        } else {
            false
        }
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
        let filtered_actions_preview: Vec<usize> = (0..actions.len()).collect();
        let grouped_items_preview =
            build_grouped_items_static(&actions, &filtered_actions_preview, config.section_style);
        let initial_selection = initial_selection_index(&grouped_items_preview);

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

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            None,
            config,
        )
    }

    /// Parse a shortcut string into individual keycap characters
    /// e.g., "⌘↵" → vec!["⌘", "↵"], "⌘I" → vec!["⌘", "I"]
    pub(crate) fn parse_shortcut_keycaps(shortcut: &str) -> Vec<String> {
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
}

#[cfg(test)]
mod unicode_keycap_safety_tests {
    use super::ActionsDialog;

    #[test]
    fn test_clipboard_context_title_does_not_panic_when_preview_contains_multibyte_unicode() {
        let preview = "😀".repeat(31);
        let title = ActionsDialog::clipboard_context_title(&preview);

        assert_eq!(title.chars().count(), 30);
        assert_eq!(title, format!("{}...", "😀".repeat(27)));
    }

    #[test]
    fn test_parse_shortcut_keycaps_does_not_panic_when_shortcut_contains_multibyte_unicode() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘😀");
        let expected = vec!["⌘".to_string(), "😀".to_string()];

        assert_eq!(keycaps, expected);
    }
}

// --- merged from part_02.rs ---
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
        let old_count = self.list_state.item_count();
        let new_count = self.grouped_items.len();

        // Workaround for GPUI ListState stale layout: when transitioning
        // from 0 items back to N items (e.g., type "nice" → 0 results,
        // then delete all chars → all items restored), splice(0..0, N)
        // doesn't fully recalculate the list's internal layout heights.
        // Recreating the ListState forces a clean layout pass.
        if old_count == 0 && new_count > 0 {
            self.list_state = ListState::new(
                new_count,
                ListAlignment::Top,
                px(ACTIONS_DIALOG_LIST_OVERDRAW_PX),
            );
        } else {
            self.list_state.splice(0..old_count, new_count);
        }
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

// --- merged from part_03.rs ---
const ACTIONS_DIALOG_COLOR_ALPHA_MAX: f32 = 255.0;
const ACTIONS_DIALOG_SEARCH_BORDER_ALPHA_SCALE: f32 = 2.0;
const ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA: u8 = 0x80;
const ACTIONS_DIALOG_VIBRANT_DIALOG_MIN_OPACITY: f32 = 0.50;
const ACTIONS_DIALOG_OPAQUE_DIALOG_MIN_OPACITY: f32 = 0.95;

fn actions_dialog_alpha_u8(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * ACTIONS_DIALOG_COLOR_ALPHA_MAX) as u8
}

fn actions_dialog_search_border_alpha(border_inactive_opacity: f32) -> u8 {
    let scaled_border_opacity =
        (border_inactive_opacity * ACTIONS_DIALOG_SEARCH_BORDER_ALPHA_SCALE).min(1.0);
    actions_dialog_alpha_u8(scaled_border_opacity)
}

fn actions_dialog_container_border_alpha(border_inactive_opacity: f32) -> u8 {
    actions_dialog_search_border_alpha(border_inactive_opacity)
        .max(ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA)
}

fn actions_dialog_container_background_alpha(dialog_opacity: f32, use_vibrancy: bool) -> u8 {
    let minimum_dialog_opacity = if use_vibrancy {
        ACTIONS_DIALOG_VIBRANT_DIALOG_MIN_OPACITY
    } else {
        ACTIONS_DIALOG_OPAQUE_DIALOG_MIN_OPACITY
    };
    actions_dialog_alpha_u8(dialog_opacity.max(minimum_dialog_opacity))
}

fn actions_dialog_rgba_with_alpha(hex: u32, alpha: u8) -> gpui::Rgba {
    rgba(hex_with_alpha(hex, alpha))
}

impl ActionsDialog {
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
        self.get_selected_action().map(|action| action.id.clone())
    }

    /// Get the currently selected ProtocolAction (for checking close behavior)
    /// Returns the original ProtocolAction from sdk_actions if this is an SDK action,
    /// or None for built-in actions.
    pub fn get_selected_protocol_action(&self) -> Option<&ProtocolAction> {
        let protocol_action_index = resolve_selected_protocol_action_index(
            self.selected_action_index(),
            &self.sdk_action_indices,
        )?;
        self.sdk_actions.as_ref()?.get(protocol_action_index)
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

    /// Create box shadow for the overlay popup
    pub(super) fn create_popup_shadow() -> Vec<BoxShadow> {
        vec![BoxShadow {
            color: gpui::Hsla::from(rgba(0x00000066)),
            offset: gpui::point(px(0.), px(4.)),
            blur_radius: px(12.),
            spread_radius: px(1.),
        }]
    }

    /// Get colors for the search box based on design variant
    /// Returns: (search_box_bg, border_color, muted_text, dimmed_text, secondary_text)
    pub(super) fn get_search_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        // Use theme opacity for input background to support vibrancy
        let opacity = self.theme.get_opacity();
        let input_alpha = actions_dialog_alpha_u8(opacity.input);
        // Keep search and container borders on the same opacity scaling path.
        let border_alpha = actions_dialog_search_border_alpha(opacity.border_inactive);
        let (search_box_background, search_box_border, muted_text, dimmed_text, secondary_text) =
            if self.design_variant == DesignVariant::Default {
                (
                    self.theme.colors.background.search_box,
                    self.theme.colors.ui.border,
                    self.theme.colors.text.muted,
                    self.theme.colors.text.dimmed,
                    self.theme.colors.text.secondary,
                )
            } else {
                (
                    colors.background_secondary,
                    colors.border,
                    colors.text_muted,
                    colors.text_dimmed,
                    colors.text_secondary,
                )
            };

        (
            actions_dialog_rgba_with_alpha(search_box_background, input_alpha),
            actions_dialog_rgba_with_alpha(search_box_border, border_alpha),
            rgb(muted_text),
            rgb(dimmed_text),
            rgb(secondary_text),
        )
    }

    /// Get colors for the main container based on design variant
    /// Returns: (main_bg, container_border, container_text)
    pub(super) fn get_container_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        let opacity = self.theme.get_opacity();
        let use_vibrancy = self.theme.is_vibrancy_enabled();
        // Keep the current readability floors while allowing stronger user-configured dialog
        // opacity values to take effect.
        let dialog_alpha = actions_dialog_container_background_alpha(opacity.dialog, use_vibrancy);
        let border_alpha = actions_dialog_container_border_alpha(opacity.border_inactive);
        let (main_background, container_border, container_text) =
            if self.design_variant == DesignVariant::Default {
                (
                    self.theme.colors.background.main,
                    self.theme.colors.ui.border,
                    self.theme.colors.text.secondary,
                )
            } else {
                (colors.background, colors.border, colors.text_secondary)
            };

        (
            actions_dialog_rgba_with_alpha(main_background, dialog_alpha),
            actions_dialog_rgba_with_alpha(container_border, border_alpha),
            rgb(container_text),
        )
    }
}

#[cfg(test)]
mod actions_dialog_opacity_consistency_tests {
    use super::{
        actions_dialog_container_background_alpha, actions_dialog_container_border_alpha,
        actions_dialog_rgba_with_alpha, actions_dialog_search_border_alpha,
        ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA,
    };
    use gpui::rgba;

    #[test]
    fn test_actions_dialog_search_border_alpha_scales_border_inactive_opacity() {
        assert_eq!(actions_dialog_search_border_alpha(0.20), 102);
    }

    #[test]
    fn test_actions_dialog_container_border_alpha_enforces_minimum_contrast() {
        assert_eq!(
            actions_dialog_container_border_alpha(0.10),
            ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA
        );
    }

    #[test]
    fn test_actions_dialog_container_background_alpha_keeps_vibrancy_floor() {
        assert_eq!(actions_dialog_container_background_alpha(0.15, true), 127);
    }

    #[test]
    fn test_actions_dialog_container_background_alpha_keeps_non_vibrancy_floor() {
        assert_eq!(actions_dialog_container_background_alpha(0.15, false), 242);
    }

    #[test]
    fn test_actions_dialog_container_background_alpha_uses_higher_theme_value() {
        assert_eq!(actions_dialog_container_background_alpha(0.80, true), 204);
    }

    #[test]
    fn test_actions_dialog_rgba_with_alpha_combines_hex_and_alpha_channels() {
        assert_eq!(
            actions_dialog_rgba_with_alpha(0x112233, 0x44),
            rgba(0x11223344)
        );
    }
}

// --- merged from part_03.rs ---

impl Focusable for ActionsDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// --- merged from dialog_part_04_rewire.rs ---
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
        let (_search_box_bg, border_color, _muted_text, dimmed_text, _secondary_text) =
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

        // Focus border color (accent with theme-aware transparency)
        // Use border_active opacity for focused state, scaled for visibility
        let opacity = self.theme.get_opacity();
        let focus_border_alpha = ((opacity.border_active * 1.5).min(1.0) * 255.0) as u8;
        let focus_border_color = rgba(hex_with_alpha(accent_color_hex, focus_border_alpha));

        // Raycast-style footer search input: minimal styling, full-width, top separator line
        // No boxed input field - just text on a clean background with a thin top border
        // Use theme colors for both light and dark mode
        // Light mode derives from the same theme tokens as dark mode
        let separator_color = border_color;
        let hint_text_color = dimmed_text;
        let input_text_color = primary_text;

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
            // No background - clean/transparent to match Raycast
            .border_t_1() // Top separator line only
            .border_color(separator_color)
            .flex()
            .flex_row()
            .items_center()
            .child(
                // Full-width search input - no box styling, just text
                div()
                    .flex_1() // Take full width
                    .h(px(28.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    // Placeholder or input text color
                    .text_color(if self.search_text.is_empty() {
                        hint_text_color
                    } else {
                        input_text_color
                    })
                    // Cursor at start when empty
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
                    // Cursor at end when has text
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
            );

        // Render action list using list() for variable-height items
        // Section headers are 22px, action items are 36px
        //
        // IMPORTANT: Always render the list() component, even when empty.
        // Switching between a static empty-state div and the list component
        // causes the GPUI ListState to lose sync with the render tree,
        // resulting in stale layout when items are restored after filtering
        // to zero results (e.g., type "nice" then delete all characters).
        let actions_container = {
            // Clone data needed for the list closure
            let grouped_items_clone = self.grouped_items.clone();
            let design_variant = self.design_variant;
            let is_empty = self.grouped_items.is_empty();

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

            // Keep scrollbar viewport aligned with actual list viewport by
            // excluding non-list chrome (search/header/footer) from max height.
            let show_search =
                !matches!(self.config.search_position, SearchPosition::Hidden) && !self.hide_search;
            let container_height = actions_dialog_scrollbar_viewport_height(
                if is_empty {
                    ACTION_ITEM_HEIGHT
                } else {
                    total_content_height
                },
                show_search,
                self.context_title.is_some(),
                self.config.show_footer,
            );

            // Estimate visible items based on average item height
            let avg_item_height = if is_empty {
                ACTION_ITEM_HEIGHT
            } else {
                total_content_height / self.grouped_items.len() as f32
            };
            let visible_items = if is_empty {
                0
            } else {
                (container_height / avg_item_height)
                    .ceil()
                    .max(1.0)
                    .min(self.grouped_items.len() as f32) as usize
            };

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
                                // Section header at 22px height
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
                                    .px(px(crate::actions::constants::ACTION_PADDING_X))
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
                                // Action item at 36px height
                                if let Some(&action_idx) = this.filtered_actions.get(*filter_idx) {
                                    if let Some(action) = this.actions.get(action_idx) {
                                        let is_selected = ix == current_selected;
                                        let filter_ix = *filter_idx;
                                        let show_section_separator = matches!(
                                            this.config.section_style,
                                            SectionStyle::Separators
                                        )
                                            && should_render_section_separator(
                                                &this.actions,
                                                &this.filtered_actions,
                                                filter_ix,
                                            );
                                        let is_destructive = is_destructive_action(action);

                                        // Get tokens for styling
                                        let item_tokens = get_tokens(design_variant);
                                        let item_colors = item_tokens.colors();
                                        let item_spacing = item_tokens.spacing();

                                        // Extract colors for list items - theme-aware selection
                                        // Light mode: Use light gray (like POC: 0xE8E8E8 at 80%)
                                        // Dark mode: Use white at low opacity for subtle brightening
                                        let is_dark_mode = this.theme.should_use_dark_vibrancy();

                                        let (
                                            selected_bg,
                                            hover_bg,
                                            primary_text,
                                            secondary_text,
                                            dimmed_text,
                                        ) = if design_variant == DesignVariant::Default {
                                            // Use theme opacity for both light and dark mode
                                            // Light mode uses same derivation pattern as dark mode
                                            let theme_opacity = this.theme.get_opacity();
                                            let selected_alpha =
                                                (theme_opacity.selected * 255.0) as u32;
                                            let hover_alpha = (theme_opacity.hover * 255.0) as u32;
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
                                            let theme_opacity = this.theme.get_opacity();
                                            let selected_alpha =
                                                (theme_opacity.selected * 255.0) as u32;
                                            let hover_alpha = (theme_opacity.hover * 255.0) as u32;
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

                                        let destructive_text =
                                            if design_variant == DesignVariant::Default {
                                                rgb(this.theme.colors.ui.error)
                                            } else {
                                                rgb(item_colors.error)
                                            };
                                        let destructive_selected_bg =
                                            if design_variant == DesignVariant::Default {
                                                rgba(hex_with_alpha(
                                                    this.theme.colors.ui.error,
                                                    if is_dark_mode { 0x45 } else { 0x2A },
                                                ))
                                            } else {
                                                rgba(hex_with_alpha(
                                                    item_colors.error,
                                                    if is_dark_mode { 0x45 } else { 0x2A },
                                                ))
                                            };
                                        let destructive_hover_bg =
                                            if design_variant == DesignVariant::Default {
                                                rgba(hex_with_alpha(
                                                    this.theme.colors.ui.error,
                                                    if is_dark_mode { 0x2E } else { 0x1F },
                                                ))
                                            } else {
                                                rgba(hex_with_alpha(
                                                    item_colors.error,
                                                    if is_dark_mode { 0x2E } else { 0x1F },
                                                ))
                                            };
                                        let section_separator_color = if design_variant
                                            == DesignVariant::Default
                                        {
                                            rgba(hex_with_alpha(this.theme.colors.ui.border, 0x60))
                                        } else {
                                            rgba(hex_with_alpha(item_colors.border, 0x60))
                                        };

                                        // Title color: bright when selected, secondary when not
                                        let title_color = if is_selected {
                                            primary_text
                                        } else {
                                            secondary_text
                                        };
                                        // Keycap colors: derive from theme for both light and dark mode
                                        // Uses theme border color with appropriate alpha values
                                        let (mut keycap_bg, mut keycap_border, mut shortcut_color) =
                                            if design_variant == DesignVariant::Default {
                                                // Use theme-derived colors for both modes
                                                // Light mode: higher alpha for visibility on light bg
                                                // Dark mode: lower alpha for subtlety on dark bg
                                                let bg_alpha: u8 =
                                                    if is_dark_mode { 0x80 } else { 0xCC };
                                                let border_alpha: u8 =
                                                    if is_dark_mode { 0xA0 } else { 0xDD };
                                                (
                                                    rgba(hex_with_alpha(
                                                        this.theme.colors.ui.border,
                                                        bg_alpha,
                                                    )),
                                                    rgba(hex_with_alpha(
                                                        this.theme.colors.ui.border,
                                                        border_alpha,
                                                    )),
                                                    rgb(this.theme.colors.text.secondary),
                                                )
                                            } else {
                                                (
                                                    rgba(hex_with_alpha(item_colors.border, 0x80)),
                                                    rgba(hex_with_alpha(item_colors.border, 0xA0)),
                                                    dimmed_text,
                                                )
                                            };

                                        let title_color = if is_destructive {
                                            destructive_text
                                        } else {
                                            title_color
                                        };
                                        if is_destructive {
                                            keycap_bg = if design_variant == DesignVariant::Default
                                            {
                                                rgba(hex_with_alpha(
                                                    this.theme.colors.ui.error,
                                                    if is_dark_mode { 0x40 } else { 0x2A },
                                                ))
                                            } else {
                                                rgba(hex_with_alpha(
                                                    item_colors.error,
                                                    if is_dark_mode { 0x40 } else { 0x2A },
                                                ))
                                            };
                                            keycap_border =
                                                if design_variant == DesignVariant::Default {
                                                    rgba(hex_with_alpha(
                                                        this.theme.colors.ui.error,
                                                        if is_dark_mode { 0x90 } else { 0xB0 },
                                                    ))
                                                } else {
                                                    rgba(hex_with_alpha(
                                                        item_colors.error,
                                                        if is_dark_mode { 0x90 } else { 0xB0 },
                                                    ))
                                                };
                                            shortcut_color = destructive_text;
                                        }

                                        // Inner row with pill-style selection
                                        let on_select = this.on_select.clone();
                                        let action_id_for_click = action.id.clone();

                                        let inner_row = div()
                                            .w_full()
                                            .flex_1()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .px(px(item_spacing.item_padding_x))
                                            .rounded(px(SELECTION_RADIUS))
                                            .bg(if is_selected {
                                                if is_destructive {
                                                    destructive_selected_bg
                                                } else {
                                                    selected_bg
                                                }
                                            } else {
                                                rgba(0x00000000)
                                            })
                                            .hover(move |s| {
                                                s.bg(if is_destructive {
                                                    destructive_hover_bg
                                                } else {
                                                    hover_bg
                                                })
                                            })
                                            .cursor_pointer();

                                        // Content: optional icon + title + shortcuts
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
                                                        .text_color(if is_destructive {
                                                            destructive_text
                                                        } else if is_selected {
                                                            primary_text
                                                        } else {
                                                            dimmed_text
                                                        }),
                                                );
                                            }
                                        }

                                        // Add title + optional description stack
                                        let mut text_stack =
                                            div().flex().flex_col().justify_center().gap(px(1.0));
                                        text_stack = text_stack.child(
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

                                        if let Some(description) =
                                            action_subtitle_for_display(action)
                                        {
                                            text_stack = text_stack.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(if is_selected {
                                                        secondary_text
                                                    } else {
                                                        dimmed_text
                                                    })
                                                    .text_ellipsis()
                                                    .child(description.to_string()),
                                            );
                                        }

                                        left_side = left_side.child(text_stack);

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
                                            .when(show_section_separator, |d| {
                                                d.border_t_1().border_color(section_separator_color)
                                            })
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                move |_, _, _| {
                                                    (on_select)(action_id_for_click.clone());
                                                },
                                            )
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
            // Note: Using flex_1() to fill remaining space in flex column.
            // Do NOT use h_full() here as it can conflict with flex layout
            // and cause the search bar to be pushed off-screen.
            let empty_message = actions_dialog_empty_state_message(&self.search_text);
            div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .overflow_hidden()
                // Always render the list to keep ListState in the render tree
                .child(variable_height_list)
                .child(scrollbar)
                // Overlay empty state message when no items match
                .when(is_empty, |d| {
                    d.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .w_full()
                            .h(px(ACTION_ITEM_HEIGHT))
                            .flex()
                            .items_center()
                            .px(px(spacing.item_padding_x))
                            .text_color(dimmed_text)
                            .text_sm()
                            .child(empty_message),
                    )
                })
                .into_any_element()
        };

        // Use helper method for container colors
        let (main_bg, _container_border, container_text) = self.get_container_colors(&colors);

        // Get search position from config before height calculations
        let search_at_top = matches!(self.config.search_position, SearchPosition::Top);
        let show_search =
            !matches!(self.config.search_position, SearchPosition::Hidden) && !self.hide_search;
        let search_box_height = if show_search {
            SEARCH_INPUT_HEIGHT
        } else {
            0.0
        };
        let header_height = if self.context_title.is_some() {
            HEADER_HEIGHT
        } else {
            0.0
        };
        let footer_height = if self.config.show_footer { 32.0 } else { 0.0 };
        let border_height = visual.border_thin * 2.0; // top + bottom border

        // Count items and section headers separately for accurate height calculation
        let mut section_header_count = 0_usize;
        let mut action_item_count = 0_usize;
        for item in &self.grouped_items {
            match item {
                GroupedActionItem::SectionHeader(_) => section_header_count += 1,
                GroupedActionItem::Item(_) => action_item_count += 1,
            }
        }

        // When no actions, still need space for "No actions match" message
        let min_items_height = if action_item_count == 0 {
            ACTION_ITEM_HEIGHT
        } else {
            0.0
        };

        // Calculate content height including both items and section headers
        let content_height = (action_item_count as f32 * ACTION_ITEM_HEIGHT)
            + (section_header_count as f32 * SECTION_HEADER_HEIGHT);
        let items_height = content_height
            .max(min_items_height)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height - footer_height);
        let total_height =
            items_height + search_box_height + header_height + border_height + footer_height;

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
                .px(px(crate::actions::constants::ACTION_PADDING_X)) // Match section header padding from list_item.rs
                .pt(px(crate::actions::constants::ACTION_PADDING_TOP)) // Top padding for visual separation
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

        // Top-positioned search input - clean Raycast-style matching the bottom search
        // No boxed input field, no ⌘K prefix - just text on a clean background with bottom separator
        let input_container_top = if search_at_top && show_search {
            Some(
                div()
                    .w(px(POPUP_WIDTH)) // Match parent width exactly
                    .min_w(px(POPUP_WIDTH))
                    .max_w(px(POPUP_WIDTH))
                    .h(px(SEARCH_INPUT_HEIGHT)) // Fixed height for the input row
                    .min_h(px(SEARCH_INPUT_HEIGHT))
                    .max_h(px(SEARCH_INPUT_HEIGHT))
                    .overflow_hidden() // Prevent any content from causing shifts
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.item_padding_y + 2.0)) // Slightly more vertical padding
                    // No background - clean/transparent to match Raycast
                    .border_b_1() // Bottom separator line
                    .border_color(separator_color)
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        // Full-width search input - no box styling, just text
                        div()
                            .flex_1() // Take full width
                            .h(px(28.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_sm()
                            // Placeholder or input text color
                            .text_color(if self.search_text.is_empty() {
                                hint_text_color
                            } else {
                                input_text_color
                            })
                            // Cursor at start when empty
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
                            // Cursor at end when has text
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
            .border_color(focus_border_color)
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

// --- merged from part_05.rs ---

#[cfg(test)]
mod tests {
    use super::{
        action_subtitle_for_display, actions_dialog_scrollbar_viewport_height,
        is_destructive_action, should_render_section_separator, ActionsDialog,
    };
    use crate::actions::types::{Action, ActionCategory};
    use gpui::px;

    #[test]
    fn destructive_detection_matches_known_ids() {
        let remove_action = Action::new(
            "remove_alias",
            "Remove Alias",
            Some("Remove alias".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(is_destructive_action(&remove_action));

        let trash_action = Action::new(
            "move_to_trash",
            "Move to Trash",
            Some("Move item to Trash".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(is_destructive_action(&trash_action));
    }

    #[test]
    fn destructive_detection_matches_title_prefix_fallback() {
        let delete_action = Action::new(
            "custom_action",
            "Delete Export Cache",
            Some("Delete cached export".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(is_destructive_action(&delete_action));

        let safe_action = Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy path".to_string()),
            ActionCategory::ScriptContext,
        );
        assert!(!is_destructive_action(&safe_action));
    }

    #[test]
    fn section_separator_only_shows_on_section_boundary() {
        let actions = vec![
            Action::new(
                "run_script",
                "Run Script",
                Some("Run".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Actions"),
            Action::new(
                "edit_script",
                "Edit Script",
                Some("Edit".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Edit"),
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Share"),
            Action::new(
                "copy_deeplink",
                "Copy Deeplink",
                Some("Copy".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Share"),
        ];
        let filtered_actions = vec![0, 1, 2, 3];

        assert!(!should_render_section_separator(
            &actions,
            &filtered_actions,
            0
        ));
        assert!(should_render_section_separator(
            &actions,
            &filtered_actions,
            1
        ));
        assert!(should_render_section_separator(
            &actions,
            &filtered_actions,
            2
        ));
        assert!(!should_render_section_separator(
            &actions,
            &filtered_actions,
            3
        ));
    }

    #[test]
    fn test_scrollbar_viewport_subtracts_header_footer_and_search_height() {
        let total_content_height = 500.0;
        let viewport_height =
            actions_dialog_scrollbar_viewport_height(total_content_height, true, true, true);

        // POPUP_MAX_HEIGHT (400) - SEARCH_INPUT_HEIGHT (44) - HEADER_HEIGHT (24) - footer (32)
        assert_eq!(viewport_height, 300.0);
    }

    #[test]
    fn test_scrollbar_viewport_clamps_to_content_when_content_shorter_than_viewport() {
        let total_content_height = 120.0;
        let viewport_height =
            actions_dialog_scrollbar_viewport_height(total_content_height, true, true, true);

        assert_eq!(viewport_height, 120.0);
    }

    #[test]
    fn test_action_subtitle_for_display_returns_description_when_present() {
        let action_with_description = Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the selected path".to_string()),
            ActionCategory::ScriptContext,
        );
        let action_without_description = Action::new(
            "run_script",
            "Run Script",
            None,
            ActionCategory::ScriptContext,
        );

        assert_eq!(
            action_subtitle_for_display(&action_with_description),
            Some("Copy the selected path")
        );
        assert_eq!(
            action_subtitle_for_display(&action_without_description),
            None
        );
    }

    #[test]
    fn test_create_popup_shadow_returns_visible_shadow() {
        let shadows = ActionsDialog::create_popup_shadow();

        assert_eq!(shadows.len(), 1);
        assert_eq!(shadows[0].offset, gpui::point(px(0.), px(4.)));
        assert_eq!(shadows[0].blur_radius, px(12.));
        assert_eq!(shadows[0].spread_radius, px(1.));
    }
}

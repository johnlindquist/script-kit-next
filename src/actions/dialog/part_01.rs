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
///
/// We intentionally suppress subtitle/description rendering to keep action rows
/// visually focused on title + shortcut + icon.
pub(crate) fn action_subtitle_for_display(_action: &Action) -> Option<&str> {
    None
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

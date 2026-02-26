//! Notes Actions Panel
//!
//! Modal overlay panel triggered by Cmd+K in the Notes window.
//! Provides searchable action list for note operations.
//!
//! ## Actions
//! - New Note (⌘N) - Create a new note
//! - Duplicate Note (⌘D) - Create a copy of the current note
//! - Browse Notes (⌘P) - Open note browser/picker
//! - Find in Note (⌘F) - Search within current note
//! - Copy Note As... (⇧⌘C) - Copy note in a chosen format
//! - Copy Deeplink (⇧⌘D) - Copy a deeplink to the note
//! - Create Quicklink (⇧⌘L) - Copy a quicklink to the note
//! - Export... (⇧⌘E) - Export note content
//! - Move List Item Up (⌃⌘↑) - Reorder notes list (disabled)
//! - Move List Item Down (⌃⌘↓) - Reorder notes list (disabled)
//! - Format... (⇧⌘T) - Formatting commands
//!
//! ## Keyboard Navigation
//! - Arrow Up/Down: Navigate actions
//! - Home/End: Jump to first/last selectable action
//! - Page Up/Page Down: Jump by 8 actions
//! - Enter: Execute selected action
//! - Escape: Close panel
//! - Type to search/filter actions

use super::window::OPACITY_DISABLED;
use crate::actions::ActionsDialog;
use crate::designs::icon_variations::IconName;
use crate::protocol::ProtocolAction;
use gpui::{
    div, point, prelude::*, px, rgba, svg, uniform_list, AnyElement, App, BoxShadow, Context,
    FocusHandle, Focusable, Hsla, KeyDownEvent, MouseButton, Render, ScrollStrategy, SharedString,
    UniformListScrollHandle, Window,
};
use gpui_component::theme::{ActiveTheme, Theme};
use std::sync::Arc;
use tracing::debug;

/// Action invocation emitted by `NotesActionsPanel`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotesActionInvocation {
    BuiltIn(NotesAction),
    Sdk {
        action_name: String,
        protocol_index: usize,
    },
}

/// Callback type for action execution.
pub type NotesActionCallback = Arc<dyn Fn(NotesActionInvocation) + Send + Sync>;

/// Available actions in the Notes actions panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotesAction {
    /// Create a new note
    NewNote,
    /// Duplicate the current note
    DuplicateNote,
    /// Open the note browser/picker
    BrowseNotes,
    /// Search within the current note
    FindInNote,
    /// Copy note content as a formatted export
    CopyNoteAs,
    /// Copy deeplink to the current note
    CopyDeeplink,
    /// Copy quicklink to the current note
    CreateQuicklink,
    /// Export note content
    Export,
    /// Move list item up (disabled placeholder)
    MoveListItemUp,
    /// Move list item down (disabled placeholder)
    MoveListItemDown,
    /// Open formatting commands
    Format,
    /// Enable auto-sizing (window grows/shrinks with content)
    EnableAutoSizing,
    /// Panel was cancelled (Escape pressed)
    Cancel,
}

impl NotesAction {
    /// Get all available actions (excluding Cancel)
    pub fn all() -> &'static [NotesAction] {
        &[
            NotesAction::NewNote,
            NotesAction::DuplicateNote,
            NotesAction::BrowseNotes,
            NotesAction::FindInNote,
            NotesAction::CopyNoteAs,
            NotesAction::CopyDeeplink,
            NotesAction::CreateQuicklink,
            NotesAction::Export,
            NotesAction::MoveListItemUp,
            NotesAction::MoveListItemDown,
            NotesAction::Format,
        ]
    }

    /// Get the display label for this action
    pub fn label(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "New Note",
            NotesAction::DuplicateNote => "Duplicate Note",
            NotesAction::BrowseNotes => "Browse Notes",
            NotesAction::FindInNote => "Find in Note",
            NotesAction::CopyNoteAs => "Copy Note As...",
            NotesAction::CopyDeeplink => "Copy Deeplink",
            NotesAction::CreateQuicklink => "Create Quicklink",
            NotesAction::Export => "Export...",
            NotesAction::MoveListItemUp => "Move List Item Up",
            NotesAction::MoveListItemDown => "Move List Item Down",
            NotesAction::Format => "Format...",
            NotesAction::EnableAutoSizing => "Enable Auto-Sizing",
            NotesAction::Cancel => "Cancel",
        }
    }

    /// Get the keyboard shortcut key (without modifier)
    pub fn shortcut_key(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "N",
            NotesAction::DuplicateNote => "D",
            NotesAction::BrowseNotes => "P",
            NotesAction::FindInNote => "F",
            NotesAction::CopyNoteAs => "C",
            NotesAction::CopyDeeplink => "D",
            NotesAction::CreateQuicklink => "L",
            NotesAction::Export => "E",
            NotesAction::MoveListItemUp => "↑",
            NotesAction::MoveListItemDown => "↓",
            NotesAction::Format => "T",
            NotesAction::EnableAutoSizing => "A",
            NotesAction::Cancel => "Esc",
        }
    }

    /// Get shortcut keys for keycap rendering
    pub fn shortcut_keys(&self) -> &'static [&'static str] {
        const CMD_N: [&str; 2] = ["⌘", "N"];
        const CMD_D: [&str; 2] = ["⌘", "D"];
        const CMD_P: [&str; 2] = ["⌘", "P"];
        const CMD_F: [&str; 2] = ["⌘", "F"];
        const SHIFT_CMD_C: [&str; 3] = ["⇧", "⌘", "C"];
        const SHIFT_CMD_D: [&str; 3] = ["⇧", "⌘", "D"];
        const SHIFT_CMD_L: [&str; 3] = ["⇧", "⌘", "L"];
        const SHIFT_CMD_E: [&str; 3] = ["⇧", "⌘", "E"];
        const CTRL_CMD_UP: [&str; 3] = ["⌃", "⌘", "↑"];
        const CTRL_CMD_DOWN: [&str; 3] = ["⌃", "⌘", "↓"];
        const SHIFT_CMD_T: [&str; 3] = ["⇧", "⌘", "T"];
        const CMD_A: [&str; 2] = ["⌘", "A"];
        const ESC: [&str; 1] = ["Esc"];

        match self {
            NotesAction::NewNote => &CMD_N,
            NotesAction::DuplicateNote => &CMD_D,
            NotesAction::BrowseNotes => &CMD_P,
            NotesAction::FindInNote => &CMD_F,
            NotesAction::CopyNoteAs => &SHIFT_CMD_C,
            NotesAction::CopyDeeplink => &SHIFT_CMD_D,
            NotesAction::CreateQuicklink => &SHIFT_CMD_L,
            NotesAction::Export => &SHIFT_CMD_E,
            NotesAction::MoveListItemUp => &CTRL_CMD_UP,
            NotesAction::MoveListItemDown => &CTRL_CMD_DOWN,
            NotesAction::Format => &SHIFT_CMD_T,
            NotesAction::EnableAutoSizing => &CMD_A,
            NotesAction::Cancel => &ESC,
        }
    }

    /// Get the formatted shortcut display string
    pub fn shortcut_display(&self) -> String {
        if self.shortcut_keys().is_empty() {
            return String::new();
        }

        self.shortcut_keys().join("")
    }

    /// Get the icon for this action (uses local IconName from designs module)
    pub fn icon(&self) -> IconName {
        match self {
            NotesAction::NewNote => IconName::Plus,
            NotesAction::DuplicateNote => IconName::Copy,
            NotesAction::BrowseNotes => IconName::FolderOpen,
            NotesAction::FindInNote => IconName::MagnifyingGlass,
            NotesAction::CopyNoteAs => IconName::Copy,
            NotesAction::CopyDeeplink => IconName::ArrowRight,
            NotesAction::CreateQuicklink => IconName::Star,
            NotesAction::Export => IconName::ArrowRight,
            NotesAction::MoveListItemUp => IconName::ArrowUp,
            NotesAction::MoveListItemDown => IconName::ArrowDown,
            NotesAction::Format => IconName::Code,
            NotesAction::EnableAutoSizing => IconName::ArrowRight,
            NotesAction::Cancel => IconName::Close,
        }
    }

    /// Get action ID for lookup
    pub fn id(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "new_note",
            NotesAction::DuplicateNote => "duplicate_note",
            NotesAction::BrowseNotes => "browse_notes",
            NotesAction::FindInNote => "find_in_note",
            NotesAction::CopyNoteAs => "copy_note_as",
            NotesAction::CopyDeeplink => "copy_deeplink",
            NotesAction::CreateQuicklink => "create_quicklink",
            NotesAction::Export => "export",
            NotesAction::MoveListItemUp => "move_list_item_up",
            NotesAction::MoveListItemDown => "move_list_item_down",
            NotesAction::Format => "format",
            NotesAction::EnableAutoSizing => "enable_auto_sizing",
            NotesAction::Cancel => "cancel",
        }
    }
}

/// Action list sections for visual grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotesActionSection {
    Primary,
    Actions,
    Move,
    Format,
    Utility,
}

impl NotesActionSection {
    fn for_action(action: NotesAction) -> Self {
        match action {
            NotesAction::NewNote | NotesAction::DuplicateNote | NotesAction::BrowseNotes => {
                NotesActionSection::Primary
            }
            NotesAction::FindInNote
            | NotesAction::CopyNoteAs
            | NotesAction::CopyDeeplink
            | NotesAction::CreateQuicklink
            | NotesAction::Export => NotesActionSection::Actions,
            NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => NotesActionSection::Move,
            NotesAction::Format => NotesActionSection::Format,
            NotesAction::EnableAutoSizing | NotesAction::Cancel => NotesActionSection::Utility,
        }
    }
}

/// Action entry with enabled state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotesActionItem {
    pub action: NotesAction,
    pub enabled: bool,
}

impl NotesActionItem {
    fn section(&self) -> NotesActionSection {
        NotesActionSection::for_action(self.action)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NotesSdkActionRow {
    protocol_index: usize,
    name: String,
    description: Option<String>,
    shortcut_keys: Vec<String>,
    enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotesPanelRowKey {
    BuiltIn(NotesAction),
    Sdk(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NotesPanelRow {
    BuiltIn(NotesActionItem),
    Sdk(NotesSdkActionRow),
}

impl NotesPanelRow {
    fn key(&self) -> NotesPanelRowKey {
        match self {
            Self::BuiltIn(item) => NotesPanelRowKey::BuiltIn(item.action),
            Self::Sdk(row) => NotesPanelRowKey::Sdk(row.protocol_index),
        }
    }

    fn label(&self) -> &str {
        match self {
            Self::BuiltIn(item) => item.action.label(),
            Self::Sdk(row) => &row.name,
        }
    }

    fn is_enabled(&self) -> bool {
        match self {
            Self::BuiltIn(item) => item.enabled,
            Self::Sdk(row) => row.enabled,
        }
    }

    fn icon(&self) -> IconName {
        match self {
            Self::BuiltIn(item) => item.action.icon(),
            Self::Sdk(_) => IconName::Code,
        }
    }

    fn section(&self) -> Option<NotesActionSection> {
        match self {
            Self::BuiltIn(item) => Some(item.section()),
            Self::Sdk(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
struct NotesActionSearchCache {
    label_lower: String,
    id_lower: String,
    shortcut_lower: String,
    description_lower: String,
}

impl NotesActionSearchCache {
    fn from_row(row: &NotesPanelRow) -> Self {
        match row {
            NotesPanelRow::BuiltIn(item) => Self {
                label_lower: item.action.label().to_lowercase(),
                id_lower: item.action.id().to_lowercase(),
                shortcut_lower: item.action.shortcut_display().to_lowercase(),
                description_lower: String::new(),
            },
            NotesPanelRow::Sdk(row) => Self {
                label_lower: row.name.to_lowercase(),
                id_lower: format!("sdk_action_{}_{}", row.protocol_index, row.name).to_lowercase(),
                shortcut_lower: row.shortcut_keys.join("").to_lowercase(),
                description_lower: row.description.as_deref().unwrap_or("").to_lowercase(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotesNavigationIntent {
    Home,
    End,
    PageUp,
    PageDown,
}

/// Panel dimensions and styling constants (matches main ActionsDialog)
pub const PANEL_WIDTH: f32 = 320.0;
/// Standardized to match main ActionsDialog POPUP_MAX_HEIGHT (was 580.0)
pub const PANEL_MAX_HEIGHT: f32 = 400.0;
pub const PANEL_CORNER_RADIUS: f32 = 12.0;
pub const ACTION_ITEM_HEIGHT: f32 = 36.0;
pub const PANEL_SEARCH_HEIGHT: f32 = 44.0;
pub const PANEL_BORDER_HEIGHT: f32 = 2.0;
/// Horizontal inset for action rows (creates rounded pill appearance)
pub const ACTION_ROW_INSET: f32 = 6.0;
/// Corner radius for selected row background
pub const SELECTION_RADIUS: f32 = 8.0;
/// Number of rows to jump for PageUp/PageDown navigation.
const NOTES_PANEL_PAGE_JUMP: usize = 8;

// =============================================================================
// Shadow tokens — drop shadow for floating panel when vibrancy is off
// =============================================================================

/// Secondary shadow opacity as a multiplier of the configured primary opacity.
const SHADOW_SECONDARY_ALPHA_SCALE: f32 = 0.5;
/// Primary shadow vertical offset (px).
const SHADOW_PRIMARY_OFFSET_Y: f32 = 4.0;
/// Primary shadow blur radius (px).
const SHADOW_PRIMARY_BLUR: f32 = 16.0;
/// Secondary shadow vertical offset (px).
const SHADOW_SECONDARY_OFFSET_Y: f32 = 8.0;
/// Secondary shadow blur radius (px).
const SHADOW_SECONDARY_BLUR: f32 = 32.0;
/// Secondary shadow inward spread (px, negative = inset).
const SHADOW_SECONDARY_SPREAD: f32 = -4.0;

// =============================================================================
// Search input layout tokens
// =============================================================================

/// Horizontal padding inside the search input row (px).
const SEARCH_ROW_PX: f32 = 12.0;
/// Vertical padding inside the search input row (px).
const SEARCH_ROW_PY: f32 = 8.0;
/// Height of the inner search text field (px).
const SEARCH_FIELD_HEIGHT: f32 = 28.0;
/// Horizontal padding inside the inner search field (px).
const SEARCH_FIELD_PX: f32 = 8.0;
/// Corner radius for the inner search field (px).
const SEARCH_FIELD_RADIUS: f32 = 4.0;
/// Cursor beam width (px).
const CURSOR_WIDTH: f32 = 2.0;
/// Cursor beam height (px).
const CURSOR_HEIGHT: f32 = 16.0;
/// Cursor beam margin from adjacent text (px).
const CURSOR_MARGIN: f32 = 2.0;
/// Cursor beam corner radius (px).
const CURSOR_RADIUS: f32 = 1.0;

/// "No results" row padding (px).
const NO_RESULTS_PY: f32 = 16.0;
/// "No results" row horizontal padding (px).
const NO_RESULTS_PX: f32 = 12.0;

/// Inner action row height deduction for vertical padding (px).
const ACTION_ROW_INNER_PAD: f32 = 8.0;
/// Inner action row horizontal padding (px).
const ACTION_ROW_INNER_PX: f32 = 8.0;
pub fn panel_height_for_rows(row_count: usize) -> f32 {
    let items_height = (row_count as f32 * ACTION_ITEM_HEIGHT)
        .min(PANEL_MAX_HEIGHT - (PANEL_SEARCH_HEIGHT + 16.0));
    items_height + PANEL_SEARCH_HEIGHT + PANEL_BORDER_HEIGHT
}

fn find_selectable_forward(selectable: &[bool], start: usize) -> Option<usize> {
    if start >= selectable.len() {
        return None;
    }

    (start..selectable.len()).find(|&idx| selectable[idx])
}

fn find_selectable_backward(selectable: &[bool], start: usize) -> Option<usize> {
    if selectable.is_empty() {
        return None;
    }

    let clamped_start = start.min(selectable.len() - 1);
    (0..=clamped_start).rev().find(|&idx| selectable[idx])
}

fn find_page_up_selectable(selectable: &[bool], selected_index: usize) -> Option<usize> {
    if selectable.is_empty() {
        return None;
    }

    let clamped_selected = selected_index.min(selectable.len() - 1);
    let jump_target = clamped_selected.saturating_sub(NOTES_PANEL_PAGE_JUMP);
    find_selectable_backward(selectable, jump_target).or_else(|| {
        jump_target
            .checked_add(1)
            .and_then(|next_start| find_selectable_forward(selectable, next_start))
    })
}

fn find_page_down_selectable(selectable: &[bool], selected_index: usize) -> Option<usize> {
    if selectable.is_empty() {
        return None;
    }

    let clamped_selected = selected_index.min(selectable.len() - 1);
    let jump_target = clamped_selected
        .saturating_add(NOTES_PANEL_PAGE_JUMP)
        .min(selectable.len() - 1);
    find_selectable_forward(selectable, jump_target).or_else(|| {
        jump_target
            .checked_sub(1)
            .and_then(|prev_start| find_selectable_backward(selectable, prev_start))
    })
}

fn resolve_navigation_intent(key: &str) -> Option<NotesNavigationIntent> {
    match key {
        "home" | "Home" => Some(NotesNavigationIntent::Home),
        "end" | "End" => Some(NotesNavigationIntent::End),
        "pageup" | "PageUp" => Some(NotesNavigationIntent::PageUp),
        "pagedown" | "PageDown" => Some(NotesNavigationIntent::PageDown),
        _ => None,
    }
}

fn format_protocol_shortcut_keys(shortcut: Option<&str>) -> Vec<String> {
    shortcut
        .map(ActionsDialog::format_shortcut_hint)
        .map(|hint| ActionsDialog::parse_shortcut_keycaps(&hint))
        .unwrap_or_default()
}

fn build_sdk_rows(actions: &[ProtocolAction]) -> Vec<NotesPanelRow> {
    actions
        .iter()
        .enumerate()
        .filter_map(|(protocol_index, action)| {
            if !action.is_visible() {
                return None;
            }

            Some(NotesPanelRow::Sdk(NotesSdkActionRow {
                protocol_index,
                name: action.name.clone(),
                description: action.description.clone(),
                shortcut_keys: format_protocol_shortcut_keys(action.shortcut.as_deref()),
                enabled: true,
            }))
        })
        .collect()
}

fn row_invocation(row: &NotesPanelRow) -> Option<NotesActionInvocation> {
    match row {
        NotesPanelRow::BuiltIn(item) if item.enabled => {
            Some(NotesActionInvocation::BuiltIn(item.action))
        }
        NotesPanelRow::Sdk(action) if action.enabled => Some(NotesActionInvocation::Sdk {
            action_name: action.name.clone(),
            protocol_index: action.protocol_index,
        }),
        _ => None,
    }
}

fn score_notes_action(cache: &NotesActionSearchCache, query_lower: &str) -> i32 {
    if query_lower.is_empty() {
        return 1;
    }

    let label_score = score_match_candidate(&cache.label_lower, query_lower, 180, 120, 80);
    let id_score = score_match_candidate(&cache.id_lower, query_lower, 70, 45, 30);
    let shortcut_score = score_match_candidate(&cache.shortcut_lower, query_lower, 35, 25, 20);
    let description_score =
        score_match_candidate(&cache.description_lower, query_lower, 25, 15, 10);

    let total = label_score + id_score + shortcut_score + description_score;
    if total > 0 {
        total
    } else {
        0
    }
}

fn score_match_candidate(
    candidate_lower: &str,
    query_lower: &str,
    prefix_base: i32,
    contains_base: i32,
    fuzzy_base: i32,
) -> i32 {
    if candidate_lower.is_empty() || query_lower.is_empty() {
        return 0;
    }

    let mut best = 0;

    if candidate_lower.starts_with(query_lower) {
        best = best.max(prefix_base + 30);
    }

    if let Some(position) = candidate_lower.find(query_lower) {
        let position_bonus = (24_i32 - position as i32).max(0);
        let boundary_bonus = if position == 0
            || candidate_lower
                .as_bytes()
                .get(position.saturating_sub(1))
                .is_some_and(|byte| !byte.is_ascii_alphanumeric())
        {
            12
        } else {
            0
        };
        best = best.max(contains_base + position_bonus + boundary_bonus);
    }

    if let Some(fuzzy) = fuzzy_match_score(candidate_lower, query_lower) {
        best = best.max(fuzzy_base + fuzzy);
    }

    best
}

fn fuzzy_match_score(haystack: &str, needle: &str) -> Option<i32> {
    if needle.is_empty() {
        return Some(0);
    }

    let haystack_chars: Vec<char> = haystack.chars().collect();
    if haystack_chars.is_empty() {
        return None;
    }

    let mut score = 0_i32;
    let mut haystack_index = 0_usize;
    let mut first_match: Option<usize> = None;
    let mut last_match: Option<usize> = None;
    let mut previous_match: Option<usize> = None;
    let mut consecutive_streak = 0_i32;
    let mut all_matches_at_word_start = true;

    for needle_char in needle.chars() {
        let mut found_index = None;
        while haystack_index < haystack_chars.len() {
            let ch = haystack_chars[haystack_index];
            if ch == needle_char {
                found_index = Some(haystack_index);
                haystack_index += 1;
                break;
            }
            haystack_index += 1;
        }

        let matched_index = found_index?;
        first_match.get_or_insert(matched_index);
        last_match = Some(matched_index);
        score += 6;

        let is_word_start = matched_index == 0
            || haystack_chars
                .get(matched_index.saturating_sub(1))
                .is_some_and(|ch| !ch.is_alphanumeric());
        if is_word_start {
            score += 10;
        } else {
            all_matches_at_word_start = false;
        }

        if let Some(previous_index) = previous_match {
            if matched_index == previous_index + 1 {
                consecutive_streak += 1;
                score += 9 + (consecutive_streak * 2);
            } else {
                consecutive_streak = 0;
                let gap_penalty = (matched_index - previous_index - 1) as i32;
                score -= gap_penalty.min(6);
            }
        } else {
            score += (18_i32 - matched_index as i32).max(0);
        }

        previous_match = Some(matched_index);
    }

    if let (Some(first), Some(last)) = (first_match, last_match) {
        let span = (last - first + 1) as i32;
        let needle_len = needle.chars().count() as i32;
        score += (needle_len * 4 - span).max(0);

        if all_matches_at_word_start {
            let word_start_count = haystack_chars
                .iter()
                .enumerate()
                .filter(|(index, ch)| {
                    ch.is_alphanumeric()
                        && (*index == 0
                            || !haystack_chars[index.saturating_sub(1)].is_alphanumeric())
                })
                .count() as i32;
            if needle_len == word_start_count {
                score += 12;
            }
        }
    }

    Some(score.max(1))
}

fn find_filtered_index_for_row_key(
    filtered_indices: &[usize],
    rows: &[NotesPanelRow],
    target_key: NotesPanelRowKey,
) -> Option<usize> {
    filtered_indices.iter().position(|&row_index| {
        rows.get(row_index)
            .is_some_and(|row| row.key() == target_key)
    })
}

/// Notes Actions Panel - Modal overlay for note operations
pub struct NotesActionsPanel {
    /// Built-in notes actions (restored when SDK actions are cleared).
    actions: Vec<NotesActionItem>,
    /// Active rows (built-in or SDK).
    rows: Vec<NotesPanelRow>,
    /// Precomputed lowercase caches for ranked filtering.
    search_cache: Vec<NotesActionSearchCache>,
    /// Filtered action indices
    filtered_indices: Vec<usize>,
    /// Currently selected index (within filtered)
    selected_index: usize,
    /// Search text
    search_text: String,
    /// Focus handle
    focus_handle: FocusHandle,
    /// Callback for action selection
    on_action: NotesActionCallback,
    /// Scroll handle for virtualization
    scroll_handle: UniformListScrollHandle,
    /// Cursor blink visibility
    cursor_visible: bool,
    /// SDK-provided actions (when present, replaces built-in actions)
    pub sdk_actions: Option<Vec<ProtocolAction>>,
}

impl NotesActionsPanel {
    /// Create a new NotesActionsPanel
    pub fn new(
        focus_handle: FocusHandle,
        actions: Vec<NotesActionItem>,
        on_action: NotesActionCallback,
    ) -> Self {
        let rows: Vec<NotesPanelRow> = actions
            .iter()
            .copied()
            .map(NotesPanelRow::BuiltIn)
            .collect();
        let search_cache = rows.iter().map(NotesActionSearchCache::from_row).collect();
        let filtered_indices: Vec<usize> = (0..rows.len()).collect();
        let selected_index = rows.iter().position(NotesPanelRow::is_enabled).unwrap_or(0);

        debug!(action_count = actions.len(), "Notes actions panel created");

        Self {
            actions,
            rows,
            search_cache,
            filtered_indices,
            selected_index,
            search_text: String::new(),
            focus_handle,
            on_action,
            scroll_handle: UniformListScrollHandle::new(),
            cursor_visible: true,
            sdk_actions: None,
        }
    }

    /// Set cursor visibility (for blink animation)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Set SDK-provided actions (replaces built-in actions when present)
    pub fn set_sdk_actions(&mut self, actions: Vec<ProtocolAction>) {
        self.sdk_actions = Some(actions);
        self.rebuild_rows();
        self.refilter();
    }

    /// Clear SDK actions and restore built-in actions
    pub fn clear_sdk_actions(&mut self) {
        self.sdk_actions = None;
        self.rebuild_rows();
        self.refilter();
    }

    /// Check if SDK actions are currently active
    pub fn has_sdk_actions(&self) -> bool {
        self.sdk_actions.is_some()
    }

    fn rebuild_rows(&mut self) {
        self.rows = if let Some(sdk_actions) = self.sdk_actions.as_ref() {
            build_sdk_rows(sdk_actions)
        } else {
            self.actions
                .iter()
                .copied()
                .map(NotesPanelRow::BuiltIn)
                .collect()
        };
        self.search_cache = self
            .rows
            .iter()
            .map(NotesActionSearchCache::from_row)
            .collect();
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
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

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        self.move_selection(-1, cx);
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        self.move_selection(1, cx);
    }

    /// Select the first enabled action in the filtered list.
    pub fn select_first(&mut self, cx: &mut Context<Self>) {
        let selectable = self.selectable_flags();
        if let Some(index) = find_selectable_forward(&selectable, 0) {
            self.apply_selection(index, cx);
        }
    }

    /// Select the last enabled action in the filtered list.
    pub fn select_last(&mut self, cx: &mut Context<Self>) {
        let selectable = self.selectable_flags();
        if let Some(index) = selectable
            .len()
            .checked_sub(1)
            .and_then(|last| find_selectable_backward(&selectable, last))
        {
            self.apply_selection(index, cx);
        }
    }

    /// Jump selection up by one page while keeping selection on an enabled action.
    pub fn select_page_up(&mut self, cx: &mut Context<Self>) {
        let selectable = self.selectable_flags();
        if let Some(index) = find_page_up_selectable(&selectable, self.selected_index) {
            self.apply_selection(index, cx);
        }
    }

    /// Jump selection down by one page while keeping selection on an enabled action.
    pub fn select_page_down(&mut self, cx: &mut Context<Self>) {
        let selectable = self.selectable_flags();
        if let Some(index) = find_page_down_selectable(&selectable, self.selected_index) {
            self.apply_selection(index, cx);
        }
    }

    /// Handle panel-specific navigation keys not handled elsewhere.
    pub fn handle_navigation_key(&mut self, key: &str, cx: &mut Context<Self>) -> bool {
        match resolve_navigation_intent(key) {
            Some(NotesNavigationIntent::Home) => {
                self.select_first(cx);
                true
            }
            Some(NotesNavigationIntent::End) => {
                self.select_last(cx);
                true
            }
            Some(NotesNavigationIntent::PageUp) => {
                self.select_page_up(cx);
                true
            }
            Some(NotesNavigationIntent::PageDown) => {
                self.select_page_down(cx);
                true
            }
            None => false,
        }
    }

    /// Submit the selected action
    pub fn submit_selected(&mut self) {
        if let Some(&row_idx) = self.filtered_indices.get(self.selected_index) {
            if let Some(row) = self.rows.get(row_idx) {
                if let Some(invocation) = row_invocation(row) {
                    debug!(?invocation, "Notes action selected");
                    (self.on_action)(invocation);
                }
            }
        }
    }

    /// Cancel and close
    pub fn cancel(&mut self) {
        debug!("Notes actions panel cancelled");
        (self.on_action)(NotesActionInvocation::BuiltIn(NotesAction::Cancel));
    }

    /// Get currently selected action
    pub fn get_selected_action(&self) -> Option<NotesAction> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&row_idx| self.rows.get(row_idx))
            .and_then(|row| match row {
                NotesPanelRow::BuiltIn(item) if item.enabled => Some(item.action),
                _ => None,
            })
    }

    /// Refilter actions based on search text
    fn refilter(&mut self) {
        let previously_selected = self.selected_row_key();

        if self.search_text.is_empty() {
            self.filtered_indices = (0..self.rows.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();
            let mut scored: Vec<(usize, i32)> = self
                .search_cache
                .iter()
                .enumerate()
                .filter_map(|(idx, cache)| {
                    let score = score_notes_action(cache, &search_lower);
                    if score > 0 {
                        Some((idx, score))
                    } else {
                        None
                    }
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            self.filtered_indices = scored.into_iter().map(|(idx, _)| idx).collect();
        }

        self.restore_selection(previously_selected);

        // Scroll to keep selection visible
        if !self.filtered_indices.is_empty() {
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
        }
    }

    fn selected_row_key(&self) -> Option<NotesPanelRowKey> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&row_index| self.rows.get(row_index))
            .map(NotesPanelRow::key)
    }

    fn restore_selection(&mut self, previously_selected: Option<NotesPanelRowKey>) {
        if let Some(previous_key) = previously_selected {
            if let Some(index) =
                find_filtered_index_for_row_key(&self.filtered_indices, &self.rows, previous_key)
            {
                self.selected_index = index;
                self.ensure_valid_selection();
                return;
            }
        }

        self.ensure_valid_selection();
    }

    fn ensure_valid_selection(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected_index = 0;
            return;
        }

        if self.selected_index >= self.filtered_indices.len()
            || !self.is_selectable(self.selected_index)
        {
            if let Some(index) =
                (0..self.filtered_indices.len()).find(|&idx| self.is_selectable(idx))
            {
                self.selected_index = index;
            } else {
                self.selected_index = 0;
            }
        }
    }

    fn is_selectable(&self, filtered_idx: usize) -> bool {
        self.filtered_indices
            .get(filtered_idx)
            .and_then(|&row_index| self.rows.get(row_index))
            .map(NotesPanelRow::is_enabled)
            .unwrap_or(false)
    }

    fn selectable_flags(&self) -> Vec<bool> {
        (0..self.filtered_indices.len())
            .map(|idx| self.is_selectable(idx))
            .collect()
    }

    fn apply_selection(&mut self, selected_index: usize, cx: &mut Context<Self>) {
        self.selected_index = selected_index;
        self.scroll_handle
            .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
        cx.notify();
    }

    fn move_selection(&mut self, delta: i32, cx: &mut Context<Self>) {
        let filtered_len = self.filtered_indices.len();
        if filtered_len == 0 {
            return;
        }

        let mut next_index = self.selected_index as i32;
        loop {
            next_index += delta;
            if next_index < 0 || next_index >= filtered_len as i32 {
                break;
            }

            let next = next_index as usize;
            if self.is_selectable(next) {
                self.selected_index = next;
                self.scroll_handle
                    .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
                cx.notify();
                return;
            }
        }
    }

    // =====================================================
    // Vibrancy Helper Functions
    // =====================================================
    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Get background color with vibrancy opacity applied
    ///
    /// Uses cached theme to avoid file I/O on every render.
    fn get_vibrancy_background() -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get search box background with vibrancy opacity
    ///
    /// Uses cached theme to avoid file I/O on every render.
    fn get_vibrancy_search_background() -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.search_box;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.search_box,
        ))
    }

    /// Create box shadow for the overlay
    /// Returns empty vec when vibrancy is enabled - shadows block vibrancy blur
    ///
    /// Uses cached theme to avoid file I/O on every render.
    fn build_theme_drop_shadows(shadow_color_hex: u32, primary_alpha: f32) -> Vec<BoxShadow> {
        let primary_alpha = primary_alpha.clamp(0.0, 1.0);
        let secondary_alpha = (primary_alpha * SHADOW_SECONDARY_ALPHA_SCALE).clamp(0.0, 1.0);

        vec![
            BoxShadow {
                color: crate::ui_foundation::hex_to_hsla_with_alpha(
                    shadow_color_hex,
                    primary_alpha,
                ),
                offset: point(px(0.0), px(SHADOW_PRIMARY_OFFSET_Y)),
                blur_radius: px(SHADOW_PRIMARY_BLUR),
                spread_radius: px(0.0),
            },
            BoxShadow {
                color: crate::ui_foundation::hex_to_hsla_with_alpha(
                    shadow_color_hex,
                    secondary_alpha,
                ),
                offset: point(px(0.0), px(SHADOW_SECONDARY_OFFSET_Y)),
                blur_radius: px(SHADOW_SECONDARY_BLUR),
                spread_radius: px(SHADOW_SECONDARY_SPREAD),
            },
        ]
    }

    fn create_shadow() -> Vec<BoxShadow> {
        let sk_theme = crate::theme::service::get_cached_theme();
        if sk_theme.is_vibrancy_enabled() {
            return vec![]; // No shadows for vibrancy - matches POC behavior
        }

        let shadow_config = sk_theme.get_drop_shadow();
        if !shadow_config.enabled {
            return vec![];
        }

        Self::build_theme_drop_shadows(shadow_config.color, shadow_config.opacity)
    }
}

impl Focusable for NotesActionsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotesActionsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Vibrancy-aware colors using Script Kit theme hex values
        let bg_color = Self::get_vibrancy_background();
        let search_bg_color = Self::get_vibrancy_search_background();
        let border_color = theme.border;
        let text_primary = theme.foreground;
        let text_muted = theme.muted_foreground;
        let accent_color = theme.accent;

        // Search display
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search for actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Build search input row - Raycast style: no search icon, just placeholder with cursor
        let search_input = div()
            .w_full()
            .h(px(PANEL_SEARCH_HEIGHT))
            .px(px(SEARCH_ROW_PX))
            .py(px(SEARCH_ROW_PY))
            .bg(search_bg_color) // Vibrancy-aware search area
            .border_b_1()
            .border_color(border_color)
            .flex()
            .flex_row()
            .items_center()
            // Search field - full width, no icon
            .child(
                div()
                    .flex_1()
                    .h(px(SEARCH_FIELD_HEIGHT))
                    .px(px(SEARCH_FIELD_PX))
                    .bg(search_bg_color) // Vibrancy-aware input
                    .rounded(px(SEARCH_FIELD_RADIUS))
                    .border_1()
                    .border_color(if self.search_text.is_empty() {
                        border_color
                    } else {
                        accent_color
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    .text_color(if self.search_text.is_empty() {
                        text_muted
                    } else {
                        text_primary
                    })
                    // Cursor when empty
                    .when(self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(CURSOR_WIDTH))
                                .h(px(CURSOR_HEIGHT))
                                .mr(px(CURSOR_MARGIN))
                                .rounded(px(CURSOR_RADIUS))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    })
                    .child(search_display)
                    // Cursor when has text
                    .when(!self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(CURSOR_WIDTH))
                                .h(px(CURSOR_HEIGHT))
                                .ml(px(CURSOR_MARGIN))
                                .rounded(px(CURSOR_RADIUS))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    }),
            );

        // Build actions list
        let selected_index = self.selected_index;
        let filtered_len = self.filtered_indices.len();

        let actions_list = if self.filtered_indices.is_empty() {
            div()
                .flex_1()
                .w_full()
                .py(px(NO_RESULTS_PY))
                .px(px(NO_RESULTS_PX))
                .text_color(text_muted)
                .text_sm()
                .child("No actions match your search")
                .into_any_element()
        } else {
            uniform_list(
                "notes-actions-list",
                filtered_len,
                cx.processor(
                    move |this: &mut NotesActionsPanel, visible_range, _window, cx| {
                        let theme = cx.theme();
                        let mut items = Vec::new();

                        for idx in visible_range {
                            if let Some(&row_idx) = this.filtered_indices.get(idx) {
                                if let Some(row) = this.rows.get(row_idx) {
                                    let row: &NotesPanelRow = row;
                                    let is_enabled = row.is_enabled();
                                    let is_selected = idx == selected_index && is_enabled;
                                    let is_section_start = if idx > 0 {
                                        let previous_section = this
                                            .filtered_indices
                                            .get(idx - 1)
                                            .and_then(|&prev_row_idx| this.rows.get(prev_row_idx))
                                            .and_then(NotesPanelRow::section);
                                        let current_section = row.section();
                                        current_section.is_some() && current_section != previous_section
                                    } else {
                                        false
                                    };
                                    let row_label = row.label().to_string();

                                    let shortcut_badges: AnyElement = match row {
                                        NotesPanelRow::BuiltIn(item) => {
                                            render_shortcut_keys(item.action.shortcut_keys(), theme)
                                        }
                                        NotesPanelRow::Sdk(action) => {
                                            render_shortcut_keys_dynamic(&action.shortcut_keys, theme)
                                        }
                                    };

                                    let transparent = Hsla::transparent_black();

                                    // Raycast-style: rounded pill selection, no left accent bar
                                    // Outer wrapper provides horizontal inset for the rounded background
                                    let action_row = div()
                                        .id(idx)
                                        .w_full()
                                        .h(px(ACTION_ITEM_HEIGHT))
                                        .px(px(ACTION_ROW_INSET))
                                        .flex()
                                        .flex_col()
                                        .justify_center()
                                        // Section divider as top border
                                        .when(is_section_start, |d| {
                                            d.border_t_1().border_color(theme.border)
                                        })
                                        // Inner row with rounded background
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(ACTION_ITEM_HEIGHT - ACTION_ROW_INNER_PAD))
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(ACTION_ROW_INNER_PX))
                                                .rounded(px(SELECTION_RADIUS))
                                                .bg(if is_selected {
                                                    theme.list_active
                                                } else {
                                                    transparent
                                                })
                                                .when(is_enabled, |d| {
                                                    d.hover(|s| s.bg(theme.list_hover))
                                                })
                                                .when(is_enabled, |d| d.cursor_pointer())
                                                .when(!is_enabled, |d| d.opacity(OPACITY_DISABLED))
                                                // Content row: icon + label + shortcuts
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .justify_between()
                                                        // Left: icon + label
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .gap(px(10.0))
                                                                // Icon
                                                                .child(
                                                                    svg()
                                                                        .external_path(row.icon().external_path())
                                                                        .size(px(16.))
                                                                        .text_color(if is_enabled {
                                                                            theme.foreground
                                                                        } else {
                                                                            theme.muted_foreground
                                                                        }),
                                                                )
                                                                // Label
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .text_color(if is_enabled {
                                                                            theme.foreground
                                                                        } else {
                                                                            theme.muted_foreground
                                                                        })
                                                                        .font_weight(
                                                                            if is_selected {
                                                                                gpui::FontWeight::MEDIUM
                                                                            } else {
                                                                                gpui::FontWeight::NORMAL
                                                                            },
                                                                        )
                                                                        .child(row_label.clone()),
                                                                ),
                                                        )
                                                        // Right: shortcut badge
                                                        .child(shortcut_badges),
                                                ),
                                        )
                                        .when(is_enabled, |d| {
                                            d.on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.selected_index = idx;
                                                    this.submit_selected();
                                                    cx.notify();
                                                }),
                                            )
                                        });

                                    items.push(action_row);
                                }
                            }
                        }
                        items
                    },
                ),
            )
            .flex_1()
            .w_full()
            .track_scroll(&self.scroll_handle)
            .into_any_element()
        };

        // Calculate dynamic height
        let items_height = (filtered_len as f32 * ACTION_ITEM_HEIGHT)
            .min(PANEL_MAX_HEIGHT - (PANEL_SEARCH_HEIGHT + 16.0));
        let total_height = items_height + PANEL_SEARCH_HEIGHT + PANEL_BORDER_HEIGHT;

        // Main container
        div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(total_height))
            .bg(bg_color)
            .rounded(px(PANEL_CORNER_RADIUS))
            .shadow(Self::create_shadow())
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if this.handle_navigation_key(event.keystroke.key.as_str(), cx) {
                    cx.stop_propagation();
                }
            }))
            .child(search_input)
            .child(actions_list)
    }
}

fn render_shortcut_badges<'a>(
    keys: impl IntoIterator<Item = &'a str>,
    theme: &Theme,
) -> AnyElement {
    let mut row = div().flex().flex_row().items_center().gap(px(4.0));
    let mut has_keys = false;

    for key in keys {
        has_keys = true;
        row = row.child(
            div()
                .min_w(px(18.0))
                .px(px(6.0))
                .py(px(2.0))
                .bg(theme.muted)
                .border_1()
                .border_color(theme.border)
                .rounded(px(5.0))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(key.to_string()),
        );
    }

    if has_keys {
        row.into_any_element()
    } else {
        div().into_any_element()
    }
}

fn render_shortcut_keys(keys: &[&'static str], theme: &Theme) -> AnyElement {
    render_shortcut_badges(keys.iter().copied(), theme)
}

fn render_shortcut_keys_dynamic(keys: &[String], theme: &Theme) -> AnyElement {
    render_shortcut_badges(keys.iter().map(String::as_str), theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes_action_labels() {
        assert_eq!(NotesAction::NewNote.label(), "New Note");
        assert_eq!(NotesAction::DuplicateNote.label(), "Duplicate Note");
        assert_eq!(NotesAction::BrowseNotes.label(), "Browse Notes");
        assert_eq!(NotesAction::FindInNote.label(), "Find in Note");
        assert_eq!(NotesAction::CopyNoteAs.label(), "Copy Note As...");
        assert_eq!(NotesAction::CopyDeeplink.label(), "Copy Deeplink");
        assert_eq!(NotesAction::CreateQuicklink.label(), "Create Quicklink");
        assert_eq!(NotesAction::Export.label(), "Export...");
        assert_eq!(NotesAction::MoveListItemUp.label(), "Move List Item Up");
        assert_eq!(NotesAction::MoveListItemDown.label(), "Move List Item Down");
        assert_eq!(NotesAction::Format.label(), "Format...");
    }

    #[test]
    fn test_notes_action_shortcuts() {
        assert_eq!(NotesAction::NewNote.shortcut_display(), "⌘N");
        assert_eq!(NotesAction::DuplicateNote.shortcut_display(), "⌘D");
        assert_eq!(NotesAction::BrowseNotes.shortcut_display(), "⌘P");
        assert_eq!(NotesAction::FindInNote.shortcut_display(), "⌘F");
        assert_eq!(NotesAction::CopyNoteAs.shortcut_display(), "⇧⌘C");
        assert_eq!(NotesAction::CopyDeeplink.shortcut_display(), "⇧⌘D");
        assert_eq!(NotesAction::CreateQuicklink.shortcut_display(), "⇧⌘L");
        assert_eq!(NotesAction::Export.shortcut_display(), "⇧⌘E");
        assert_eq!(NotesAction::MoveListItemUp.shortcut_display(), "⌃⌘↑");
        assert_eq!(NotesAction::MoveListItemDown.shortcut_display(), "⌃⌘↓");
        assert_eq!(NotesAction::Format.shortcut_display(), "⇧⌘T");
    }

    #[test]
    fn test_notes_action_all() {
        let all = NotesAction::all();
        assert_eq!(all.len(), 11);
        assert!(all.contains(&NotesAction::NewNote));
        assert!(all.contains(&NotesAction::DuplicateNote));
        assert!(all.contains(&NotesAction::BrowseNotes));
        assert!(all.contains(&NotesAction::FindInNote));
        assert!(all.contains(&NotesAction::CopyNoteAs));
        assert!(all.contains(&NotesAction::CopyDeeplink));
        assert!(all.contains(&NotesAction::CreateQuicklink));
        assert!(all.contains(&NotesAction::Export));
        assert!(all.contains(&NotesAction::MoveListItemUp));
        assert!(all.contains(&NotesAction::MoveListItemDown));
        assert!(all.contains(&NotesAction::Format));
    }

    #[test]
    fn test_notes_action_ids() {
        assert_eq!(NotesAction::NewNote.id(), "new_note");
        assert_eq!(NotesAction::DuplicateNote.id(), "duplicate_note");
        assert_eq!(NotesAction::BrowseNotes.id(), "browse_notes");
        assert_eq!(NotesAction::FindInNote.id(), "find_in_note");
        assert_eq!(NotesAction::CopyNoteAs.id(), "copy_note_as");
        assert_eq!(NotesAction::CopyDeeplink.id(), "copy_deeplink");
        assert_eq!(NotesAction::CreateQuicklink.id(), "create_quicklink");
        assert_eq!(NotesAction::Export.id(), "export");
        assert_eq!(NotesAction::MoveListItemUp.id(), "move_list_item_up");
        assert_eq!(NotesAction::MoveListItemDown.id(), "move_list_item_down");
        assert_eq!(NotesAction::Format.id(), "format");
    }

    #[test]
    fn test_panel_constants() {
        // Verify panel matches main ActionsDialog dimensions
        assert_eq!(PANEL_WIDTH, 320.0);
        assert_eq!(PANEL_MAX_HEIGHT, 400.0); // Standardized to match main dialog
        assert_eq!(PANEL_CORNER_RADIUS, 12.0);
        assert_eq!(ACTION_ITEM_HEIGHT, 36.0); // Unified with main dialog ACTION_ITEM_HEIGHT
        assert_eq!(ACTION_ROW_INSET, 6.0);
        assert_eq!(SELECTION_RADIUS, 8.0);
        assert_eq!(NOTES_PANEL_PAGE_JUMP, 8);
    }

    #[test]
    fn test_build_theme_drop_shadows_uses_theme_shadow_color_and_alpha_scaling() {
        let shadows = NotesActionsPanel::build_theme_drop_shadows(0x123456, 0.4);
        assert_eq!(shadows.len(), 2);
        assert_eq!(
            shadows[0].color,
            crate::ui_foundation::hex_to_hsla_with_alpha(0x123456, 0.4)
        );
        assert_eq!(
            shadows[1].color,
            crate::ui_foundation::hex_to_hsla_with_alpha(0x123456, 0.2)
        );
    }

    #[test]
    fn test_find_selectable_forward_returns_first_enabled_from_start() {
        let selectable = [false, false, true, false, true];
        assert_eq!(find_selectable_forward(&selectable, 0), Some(2));
        assert_eq!(find_selectable_forward(&selectable, 3), Some(4));
        assert_eq!(find_selectable_forward(&selectable, 5), None);
    }

    #[test]
    fn test_find_selectable_backward_returns_last_enabled_before_start() {
        let selectable = [false, true, false, true, false];
        assert_eq!(find_selectable_backward(&selectable, 4), Some(3));
        assert_eq!(find_selectable_backward(&selectable, 2), Some(1));
        assert_eq!(find_selectable_backward(&[false, false], 1), None);
    }

    #[test]
    fn test_find_page_up_selects_nearest_enabled_when_target_disabled() {
        let selectable = [
            true, false, true, false, false, true, true, false, true, false,
        ];
        // selected = 9, target = 1, nearest selectable at-or-before target is 0
        assert_eq!(find_page_up_selectable(&selectable, 9), Some(0));
        // selected = 5, target = 0
        assert_eq!(find_page_up_selectable(&selectable, 5), Some(0));
    }

    #[test]
    fn test_find_page_down_selects_nearest_enabled_when_target_disabled() {
        let selectable = [
            true, false, true, false, true, false, true, false, false, true, false,
        ];
        // selected = 0, target = 8, nearest selectable at-or-after target is 9
        assert_eq!(find_page_down_selectable(&selectable, 0), Some(9));
        // selected = 9, target clamps to last index and falls back backward to 9
        assert_eq!(find_page_down_selectable(&selectable, 9), Some(9));
    }

    #[test]
    fn test_resolve_navigation_intent_matches_lower_and_camel_case_keys() {
        assert_eq!(
            resolve_navigation_intent("home"),
            Some(NotesNavigationIntent::Home)
        );
        assert_eq!(
            resolve_navigation_intent("Home"),
            Some(NotesNavigationIntent::Home)
        );
        assert_eq!(
            resolve_navigation_intent("end"),
            Some(NotesNavigationIntent::End)
        );
        assert_eq!(
            resolve_navigation_intent("End"),
            Some(NotesNavigationIntent::End)
        );
        assert_eq!(
            resolve_navigation_intent("pageup"),
            Some(NotesNavigationIntent::PageUp)
        );
        assert_eq!(
            resolve_navigation_intent("PageUp"),
            Some(NotesNavigationIntent::PageUp)
        );
        assert_eq!(
            resolve_navigation_intent("pagedown"),
            Some(NotesNavigationIntent::PageDown)
        );
        assert_eq!(
            resolve_navigation_intent("PageDown"),
            Some(NotesNavigationIntent::PageDown)
        );
        assert_eq!(resolve_navigation_intent("tab"), None);
    }

    #[test]
    fn test_score_notes_action_prefers_earlier_word_boundary_matches() {
        let early = NotesActionSearchCache {
            label_lower: "copy note as".to_string(),
            id_lower: "copy_note_as".to_string(),
            shortcut_lower: "cmdc".to_string(),
            description_lower: String::new(),
        };
        let late = NotesActionSearchCache {
            label_lower: "bulk copy tool".to_string(),
            id_lower: "bulk_copy_tool".to_string(),
            shortcut_lower: String::new(),
            description_lower: String::new(),
        };

        assert!(
            score_notes_action(&early, "copy") > score_notes_action(&late, "copy"),
            "expected earlier boundary match to rank higher"
        );
    }

    #[test]
    fn test_fuzzy_match_score_rewards_consecutive_characters() {
        let consecutive = fuzzy_match_score("duplicate note", "dn");
        let sparse = fuzzy_match_score("do not disturb", "dn");
        assert!(consecutive > sparse);
    }

    #[test]
    fn test_find_filtered_index_for_row_key_preserves_identity() {
        let rows = vec![
            NotesPanelRow::BuiltIn(NotesActionItem {
                action: NotesAction::NewNote,
                enabled: true,
            }),
            NotesPanelRow::BuiltIn(NotesActionItem {
                action: NotesAction::DuplicateNote,
                enabled: true,
            }),
            NotesPanelRow::Sdk(NotesSdkActionRow {
                protocol_index: 4,
                name: "Archive".to_string(),
                description: None,
                shortcut_keys: vec!["⌘".to_string(), "A".to_string()],
                enabled: true,
            }),
        ];

        let filtered_indices = vec![2, 1];
        assert_eq!(
            find_filtered_index_for_row_key(
                &filtered_indices,
                &rows,
                NotesPanelRowKey::BuiltIn(NotesAction::DuplicateNote),
            ),
            Some(1)
        );
        assert_eq!(
            find_filtered_index_for_row_key(&filtered_indices, &rows, NotesPanelRowKey::Sdk(4)),
            Some(0)
        );
    }

    #[test]
    fn test_build_sdk_rows_filters_hidden_actions_and_preserves_protocol_index() {
        let visible = ProtocolAction::new("Visible".to_string());
        let hidden = ProtocolAction {
            name: "Hidden".to_string(),
            visible: Some(false),
            ..ProtocolAction::new("Hidden".to_string())
        };
        let visible_after_hidden = ProtocolAction::with_value("Run".to_string(), "run".to_string());
        let rows = build_sdk_rows(&[visible, hidden, visible_after_hidden]);

        assert_eq!(rows.len(), 2);
        match &rows[0] {
            NotesPanelRow::Sdk(row) => assert_eq!(row.protocol_index, 0),
            _ => panic!("expected sdk row at index 0"),
        }
        match &rows[1] {
            NotesPanelRow::Sdk(row) => assert_eq!(row.protocol_index, 2),
            _ => panic!("expected sdk row at index 1"),
        }
    }

    #[test]
    fn test_row_invocation_emits_builtin_and_sdk_variants() {
        let built_in = NotesPanelRow::BuiltIn(NotesActionItem {
            action: NotesAction::BrowseNotes,
            enabled: true,
        });
        assert_eq!(
            row_invocation(&built_in),
            Some(NotesActionInvocation::BuiltIn(NotesAction::BrowseNotes))
        );

        let sdk = NotesPanelRow::Sdk(NotesSdkActionRow {
            protocol_index: 7,
            name: "Archive".to_string(),
            description: None,
            shortcut_keys: vec![],
            enabled: true,
        });
        assert_eq!(
            row_invocation(&sdk),
            Some(NotesActionInvocation::Sdk {
                action_name: "Archive".to_string(),
                protocol_index: 7,
            })
        );
    }
}

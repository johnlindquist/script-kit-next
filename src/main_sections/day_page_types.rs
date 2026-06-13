// Day Page view entity (included before AppView so the enum can hold Entity<DayPageView>).

use std::path::PathBuf;

use gpui::WeakEntity;

use crate::components::notes_editor::NotesEditor;
use script_kit_gpui::day_page::DayPageDocumentSession;

pub(crate) const DAY_PAGE_EDITOR_ID: &str = "day-page-editor";

/// Host for today's day page inside the main launcher window.
pub struct DayPageView {
    pub(crate) app: WeakEntity<ScriptListApp>,
    pub(crate) session: DayPageDocumentSession,
    pub(crate) notes_editor: Entity<NotesEditor>,
    pub(crate) editor_state: Entity<InputState>,
    pub(crate) editor_subscription: Subscription,
    pub(crate) focus_handle: FocusHandle,
    /// Resolved fragment paths aligned with parsed fragment card indices.
    pub(crate) fragment_open_targets: Vec<PathBuf>,
    pub(crate) spine_selected_index: usize,
    pub(crate) spine_hovered_index: Option<usize>,
    pub(crate) spine_cache_key: String,
    pub(crate) spine_cwd_revision: u64,
    pub(crate) spine_cwd_submit_anchor: bool,
    pub(crate) spine_dismissed_cache_key: Option<String>,
    pub(crate) spine_mention_aliases:
        std::collections::HashMap<String, crate::ai::message_parts::AiContextPart>,
    pub(crate) spine_grouped_cache: Vec<crate::list_item::GroupedListItem>,
    pub(crate) spine_flat_cache: Vec<crate::scripts::SearchResult>,
    pub(crate) spine_alias_cache:
        std::collections::HashMap<String, (String, crate::ai::message_parts::AiContextPart)>,
    /// Last debounced autosave write (Notes-parity SAVE_DEBOUNCE_MS throttle).
    pub(crate) last_autosave: Option<std::time::Instant>,
    /// True while a trailing autosave flush timer is pending.
    pub(crate) autosave_flush_scheduled: bool,
    /// Open past-day switcher (Cmd+P); None when closed.
    pub(crate) day_switcher: Option<DaySwitcherState>,
    /// Editor byte length at the last observed change. The `@context`
    /// main-menu swap only triggers on growth so deleting inside an existing
    /// mention never re-opens the search (day_page_round_trip.rs).
    pub(crate) last_editor_content_len: usize,
}

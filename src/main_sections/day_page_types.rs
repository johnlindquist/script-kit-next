// Day Page view entity (included before AppView so the enum can hold Entity<DayPageView>).

use std::path::PathBuf;

use gpui::WeakEntity;

use crate::components::notes_editor::NotesEditor;
use crate::components::notes_editor::spine::NotesEditorSpineRuntime;
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
    /// Resolved fragment paths aligned with parsed fragment reference indices.
    pub(crate) fragment_open_targets: Vec<PathBuf>,
    pub(crate) spine_runtime: NotesEditorSpineRuntime<crate::scripts::SearchResult>,
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

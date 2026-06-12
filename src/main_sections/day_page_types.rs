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
}

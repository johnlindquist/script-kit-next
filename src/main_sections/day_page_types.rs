// Day Page view entity (included before AppView so the enum can hold Entity<DayPageView>).

use std::path::PathBuf;

use gpui::WeakEntity;

use crate::components::notes_editor::NotesEditor;
use script_kit_gpui::day_page::DayPageDocumentSession;

pub(crate) const DAY_PAGE_EDITOR_ID: &str = "day-page-editor";

/// Minimal Day Page state for accepted `@` mention aliases. It intentionally
/// does not carry list rows, selection, hover, submit anchors, or dismissed-cache
/// state because Day must not own a local inline Spine UI or Agent handoff.
#[derive(Default)]
pub(crate) struct DayPageSpineHandoffState {
    pub(crate) mention_aliases:
        std::collections::HashMap<String, crate::ai::message_parts::AiContextPart>,
}

impl DayPageSpineHandoffState {
    pub(crate) fn reset(&mut self, _clear_cwd_anchor: bool, clear_mentions: bool) {
        if clear_mentions {
            self.mention_aliases.clear();
        }
    }

    pub(crate) fn register_mention_alias(
        &mut self,
        token: String,
        part: crate::ai::message_parts::AiContextPart,
    ) {
        self.mention_aliases.insert(token, part);
    }

    pub(crate) fn prune_mention_aliases_for_content(&mut self, content: &str) {
        crate::components::notes_editor::spine::prune_mention_aliases(
            &mut self.mention_aliases,
            content,
        );
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DayPageKitResourcePreviewState {
    pub(crate) title: String,
    pub(crate) uri: String,
    pub(crate) mime_type: String,
    pub(crate) text: String,
    pub(crate) truncated: bool,
}

impl From<crate::notes::deeplink_activation::KitResourcePreview>
    for DayPageKitResourcePreviewState
{
    fn from(preview: crate::notes::deeplink_activation::KitResourcePreview) -> Self {
        Self {
            title: preview.title,
            uri: preview.uri,
            mime_type: preview.mime_type,
            text: preview.text,
            truncated: preview.truncated,
        }
    }
}

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
    pub(crate) spine_handoff: DayPageSpineHandoffState,
    /// Last debounced autosave write (Notes-parity SAVE_DEBOUNCE_MS throttle).
    pub(crate) last_autosave: Option<std::time::Instant>,
    /// True while a trailing autosave flush timer is pending.
    pub(crate) autosave_flush_scheduled: bool,
    /// Open past-day switcher (Cmd+P); None when closed.
    pub(crate) day_switcher: Option<DaySwitcherState>,
    /// Shared Notes Cmd+P switcher component hosted by Day Page.
    pub(crate) note_switcher: crate::actions::CommandBar,
    /// Editor byte length at the last observed change. The `@context`
    /// main-menu swap only triggers on growth so deleting inside an existing
    /// mention never re-opens the search (day_page_round_trip.rs).
    pub(crate) last_editor_content_len: usize,
    /// Read-only preview opened from a `kit://` resource link in Day Page markdown.
    pub(crate) kit_resource_preview: Option<DayPageKitResourcePreviewState>,
}

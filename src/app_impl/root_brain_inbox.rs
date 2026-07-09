//! Root launcher "Brain Inbox" snapshot plumbing.
//!
//! The curator (src/brain/curator.rs) files small observations — commitments,
//! unanswered questions, drifting topics, stale pins — into the brain inbox.
//! This module keeps an app-state snapshot of the OPEN items so the grouped
//! empty-query view can pin a "Brain Inbox" section at the very top (see
//! `crate::scripts::prepend_root_brain_inbox_section`).
//!
//! Refresh model: cheap sqlite read, throttled to once per
//! [`ROOT_BRAIN_INBOX_TTL`]. Hooked where the main window becomes visible
//! (`show_main_window_helper`) and on filter-text changes, so the section is
//! current whenever the empty root query is shown. Resolving an item drops it
//! from the snapshot immediately — notification semantics: touching it clears
//! it.

use super::*;

/// Cap on inbox items loaded into the snapshot per refresh. The grouped view
/// renders at most the configured max (default 3, clamped to 5); loading a
/// few extra keeps the section populated when items resolve between reloads.
const ROOT_BRAIN_INBOX_LOAD_LIMIT: usize = 8;

/// How long a loaded snapshot stays fresh before the next hook re-reads it.
const ROOT_BRAIN_INBOX_TTL: std::time::Duration = std::time::Duration::from_secs(30);

impl ScriptListApp {
    /// Reload the open brain-inbox snapshot when it is older than
    /// [`ROOT_BRAIN_INBOX_TTL`] (or never loaded). On change: bump the inbox
    /// epoch, invalidate the passive frame + grouped cache, and notify.
    ///
    /// `allow_reorder` controls what a changed read may do to rows already on
    /// screen: the window-show hook passes `true` (fresh glance, newest
    /// first), mid-session hooks pass `false` (stable merge — see
    /// [`stable_merge_root_brain_inbox`]).
    pub(crate) fn refresh_root_brain_inbox_if_stale(
        &mut self,
        allow_reorder: bool,
        cx: &mut Context<Self>,
    ) {
        if !self
            .root_search
            .root_brain_inbox_refresh_if_stale(std::time::Instant::now(), ROOT_BRAIN_INBOX_TTL)
        {
            return;
        }

        // Errors degrade to "no section" — the launcher must never surface a
        // brain storage failure.
        let mut items =
            crate::brain::open_inbox_items(ROOT_BRAIN_INBOX_LOAD_LIMIT).unwrap_or_default();
        if !allow_reorder {
            items = crate::brain::stable_merge_open_inbox(
                self.root_search.root_brain_inbox_items(),
                items,
            );
        }
        if !self.root_search.install_root_brain_inbox_items(items) {
            return;
        }
        tracing::debug!(
            target: "script_kit::brain",
            open_items = self.root_search.root_brain_inbox_items().len(),
            "brain inbox snapshot refreshed"
        );
        self.invalidate_root_passive_and_grouped_cache();
        cx.notify();
    }

    /// Mark an inbox item resolved (best-effort) and drop it from the
    /// snapshot immediately so the pinned section shrinks/disappears without
    /// waiting for the next staleness reload.
    pub(crate) fn resolve_root_brain_inbox_item(&mut self, id: i64, cx: &mut Context<Self>) {
        if let Err(error) = crate::brain::resolve_inbox_item(id) {
            logging::log(
                "ERROR",
                &format!("Failed to resolve brain inbox item {id}: {error}"),
            );
        }
        if !self.root_search.remove_root_brain_inbox_item(id) {
            return;
        }
        self.invalidate_root_passive_and_grouped_cache();
        cx.notify();
    }
}

//! File Search's two borrowed uses of the main input: typing a new name (Rename…) and choosing
//! a folder (Move to… / Copy to…). Both stay in the main window under the same input; neither
//! opens a dialog of its own.
//!
//! Renaming keeps File Search's list and query exactly as they were and only borrows the input's
//! words. Choosing a folder browses File Search itself, the way Tab-to-pick-a-folder does, and
//! goes back to the search it started from.

use super::*;

use crate::file_search::file_ops::{rename_preview, rename_stem_len, RenamePreview};

impl ScriptListApp {
    /// Sets the main input's words without running a search for them.
    pub(crate) fn set_main_input_text_quietly(
        &mut self,
        text: &str,
        selection: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(text.to_string(), window, cx);
            state.set_selection_focused(selection.start, selection.end, window, cx);
        });
        self.suppress_filter_events = false;
    }

    /// Whether File Search is borrowing the input for a rename.
    pub(crate) fn file_search_renaming(&self) -> bool {
        self.file_search_rename.is_some()
            && matches!(self.current_view, AppView::FileSearchView { .. })
    }

    /// Whether File Search is choosing a folder for a move or copy.
    pub(crate) fn file_search_picking_place(&self) -> bool {
        self.file_search_place_pick.is_some()
            && matches!(self.current_view, AppView::FileSearchView { .. })
    }

    /// Starts typing a new name for `path` in the main input, with the name (not the
    /// extension) selected.
    pub(crate) fn begin_file_search_rename(
        &mut self,
        path: String,
        previous_display_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(self.current_view, AppView::FileSearchView { .. }) {
            return;
        }
        let item = std::path::Path::new(&path);
        let name = item
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let stem = rename_stem_len(&name, item.is_dir());
        tracing::info!(
            target: "script_kit::file_search",
            event = "rename_started",
            path = %path,
        );
        self.file_search_place_pick = None;
        self.file_search_rename = Some(crate::file_search::FileRenameEdit {
            path,
            previous_display_index,
            typed: name.clone(),
        });
        self.set_main_input_text_quietly(&name, 0..stem, window, cx);
        cx.notify();
    }

    /// What the rename row shows now.
    pub(crate) fn file_search_rename_preview(&self, _cx: &App) -> Option<RenamePreview> {
        let edit = self.file_search_rename.as_ref()?;
        Some(rename_preview(&edit.path, &edit.typed))
    }

    /// Keeps the typed name in step with the input.
    pub(crate) fn file_search_rename_typed(&mut self, cx: &mut Context<Self>) {
        let typed = self.gpui_input_state.read(cx).value().to_string();
        if let Some(edit) = self.file_search_rename.as_mut() {
            edit.typed = typed;
        }
        cx.notify();
    }

    /// ↵ while renaming: renames when the name is ready; otherwise says why not and keeps
    /// the words so they can be fixed.
    pub(crate) fn commit_file_search_rename(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The input is the authority for what was typed, even if no change event ran yet.
        let typed = self.gpui_input_state.read(cx).value().to_string();
        if let Some(edit) = self.file_search_rename.as_mut() {
            edit.typed = typed;
        }
        let Some(preview) = self.file_search_rename_preview(cx) else {
            return;
        };
        if !preview.ready {
            self.notice_failed(preview.title, cx);
            return;
        }
        let Some(edit) = self.file_search_rename.take() else {
            return;
        };
        let typed = edit.typed.clone();
        self.restore_file_search_query_text(window, cx);
        self.file_search_actions_path = Some(edit.path.clone());
        self.finish_file_search_rename_from_view(
            &edit.path,
            &typed,
            edit.previous_display_index,
            cx,
        );
    }

    /// Puts the search's own words back in the input.
    fn restore_file_search_query_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let query = match &self.current_view {
            AppView::FileSearchView { query, .. } => query.clone(),
            _ => return,
        };
        self.filter_text = query.clone();
        let end = query.len();
        self.set_main_input_text_quietly(&query, end..end, window, cx);
    }

    /// Starts choosing a folder for a move or copy, browsing from the item's own folder.
    pub(crate) fn begin_file_search_place_pick(
        &mut self,
        kind: crate::file_search::PlaceKind,
        path: String,
        previous_display_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (origin_query, presentation) = match &self.current_view {
            AppView::FileSearchView {
                query,
                presentation,
                ..
            } => (query.clone(), *presentation),
            _ => return,
        };
        let start = std::path::Path::new(&path)
            .parent()
            .map(|parent| {
                format!(
                    "{}/",
                    crate::file_search::shorten_path(&parent.to_string_lossy())
                        .trim_end_matches('/')
                )
            })
            .unwrap_or_else(|| "~/".to_string());
        tracing::info!(
            target: "script_kit::file_search",
            event = "place_pick_started",
            kind = ?kind,
            path = %path,
            start = %start,
        );
        self.file_search_rename = None;
        self.file_search_place_pick = Some(crate::file_search::FilePlacePick {
            kind,
            path,
            origin_query,
            previous_display_index,
        });
        self.filter_text = start.clone();
        self.open_file_search_view_preserving_current_results(start.clone(), presentation, cx);
        let end = start.len();
        self.set_main_input_text_quietly(&start, end..end, window, cx);
        cx.notify();
    }

    /// Puts the item being moved or copied into `folder` and goes back to the search the
    /// pick started from.
    pub(crate) fn commit_file_search_place_pick(
        &mut self,
        folder: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pick) = self.file_search_place_pick.take() else {
            return;
        };
        tracing::info!(
            target: "script_kit::file_search",
            event = "place_pick_chosen",
            kind = ?pick.kind,
            folder = %folder,
        );
        self.return_to_file_search_query(&pick.origin_query, window, cx);
        self.finish_file_search_place(
            pick.kind,
            &pick.path,
            &folder,
            crate::file_search::file_ops::OnConflict::Refuse,
            pick.previous_display_index,
            window,
            cx,
        );
    }

    fn return_to_file_search_query(
        &mut self,
        query: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let presentation = match &self.current_view {
            AppView::FileSearchView { presentation, .. } => *presentation,
            _ => FileSearchPresentation::Full,
        };
        self.filter_text = query.to_string();
        self.open_file_search_view_preserving_current_results(query.to_string(), presentation, cx);
        let end = query.len();
        self.set_main_input_text_quietly(query, end..end, window, cx);
    }

    /// Esc while renaming or choosing a folder: stop and put the search back as it was.
    /// Returns whether there was something to stop.
    pub(crate) fn cancel_file_search_borrowed_input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, AppView::FileSearchView { .. }) {
            return false;
        }
        if self.file_search_rename.take().is_some() {
            tracing::info!(target: "script_kit::file_search", event = "rename_cancelled");
            self.restore_file_search_query_text(window, cx);
            cx.notify();
            return true;
        }
        if let Some(pick) = self.file_search_place_pick.take() {
            tracing::info!(target: "script_kit::file_search", event = "place_pick_cancelled");
            self.return_to_file_search_query(&pick.origin_query, window, cx);
            cx.notify();
            return true;
        }
        false
    }

    /// ↵ in File Search while renaming or choosing a folder. `selected` is the highlighted row;
    /// `here` asks for the folder being shown instead (⌘↵). Returns whether the key was used.
    pub(crate) fn file_search_borrowed_enter(
        &mut self,
        selected: Option<&crate::file_search::FileResult>,
        here: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.file_search_renaming() {
            self.commit_file_search_rename(window, cx);
            return true;
        }
        if !self.file_search_picking_place() {
            return false;
        }
        let folder = if here {
            self.current_file_search_directory_abs()
        } else {
            selected
                .filter(|file| file.file_type == crate::file_search::FileType::Directory)
                .map(|file| file.path.clone())
        };
        match folder {
            Some(folder) => self.commit_file_search_place_pick(folder, window, cx),
            None => self.one_window_notice(
                crate::line_notice::INFO,
                "Choose a folder, or press \u{2318}\u{21b5} for the folder you're in".to_string(),
                cx,
            ),
        }
        true
    }

    /// File Search keys that must be seen before the input and the list.
    /// Searching: ⌘Z with an empty input takes back the latest file change; ⌘C with no words
    /// selected copies the highlighted file.
    /// Renaming: ↵ renames, Esc cancels, Tab does nothing (the input keeps every other key).
    /// Choosing a folder: ↵ / ⌘↵ choose, Esc cancels; Tab and arrows browse as usual.
    pub(crate) fn try_handle_file_search_borrowed_key(
        &mut self,
        key: &str,
        modifiers: &gpui::Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let renaming = self.file_search_renaming();
        let picking = self.file_search_picking_place();
        if !renaming
            && !picking
            && matches!(self.current_view, AppView::FileSearchView { .. })
            && !self.show_actions_popup
            && key.eq_ignore_ascii_case("z")
            && modifiers.platform
            && !modifiers.shift
            && !modifiers.alt
            && !modifiers.control
            && self.gpui_input_state.read(cx).value().is_empty()
        {
            // ⌘Z with an empty input takes back the latest file change; with words typed,
            // it stays the input's own undo.
            return self.file_search_undo_last(cx);
        }
        if !renaming
            && !picking
            && matches!(self.current_view, AppView::FileSearchView { .. })
            && !self.show_actions_popup
            && key.eq_ignore_ascii_case("c")
            && modifiers.platform
            && !modifiers.shift
            && !modifiers.alt
            && !modifiers.control
            && self.gpui_input_state.read(cx).selected_range().is_empty()
        {
            // ⌘C copies the highlighted file. The input's own Copy would otherwise take the key
            // and copy its empty selection; with words selected, ⌘C stays the input's.
            let selected = match &self.current_view {
                AppView::FileSearchView { selected_index, .. } => {
                    self.file_search_selection_binding(*selected_index).file
                }
                _ => None,
            };
            let Some(file) = selected else {
                return false;
            };
            self.file_search_actions_path = Some(file.path.clone());
            self.handle_action("copy_file".to_string(), window, cx);
            return true;
        }
        if !renaming && !picking {
            return false;
        }
        if self.show_actions_popup {
            return false;
        }
        let only_cmd =
            modifiers.platform && !modifiers.shift && !modifiers.alt && !modifiers.control;
        let plain = !modifiers.platform && !modifiers.shift && !modifiers.alt && !modifiers.control;

        if crate::ui_foundation::is_key_escape(key) && plain {
            return self.cancel_file_search_borrowed_input(window, cx);
        }
        if crate::ui_foundation::is_key_enter(key) && (plain || (picking && only_cmd)) {
            let selected = match &self.current_view {
                AppView::FileSearchView { selected_index, .. } => {
                    self.file_search_selection_binding(*selected_index).file
                }
                _ => None,
            };
            return self.file_search_borrowed_enter(selected.as_ref(), only_cmd, window, cx);
        }
        if renaming && key.eq_ignore_ascii_case("tab") {
            return true;
        }
        false
    }

    /// The footer words while the input is borrowed (at most three, like every footer).
    pub(crate) fn file_search_borrowed_hints(
        &self,
        selected: Option<&crate::file_search::FileResult>,
        cx: &App,
    ) -> Option<Vec<SharedString>> {
        if self.file_search_renaming() {
            let preview = self.file_search_rename_preview(cx)?;
            let first = if preview.ready {
                format!("\u{21b5} {}", preview.title)
            } else {
                preview.title
            };
            return Some(vec![first.into(), "Esc Cancel".into()]);
        }
        if !self.file_search_picking_place() {
            return None;
        }
        let pick = self.file_search_place_pick.as_ref()?;
        let verb = match pick.kind {
            crate::file_search::PlaceKind::Move => "Move",
            crate::file_search::PlaceKind::Copy => "Copy",
        };
        let first = selected
            .filter(|file| file.file_type == crate::file_search::FileType::Directory)
            .map(|file| format!("\u{21b5} {verb} into {} \u{b7} \u{21e5} Browse", file.name))
            .unwrap_or_else(|| "\u{21e5} Browse".to_string());
        Some(vec![
            first.into(),
            format!("\u{2318}\u{21b5} {verb} Here").into(),
            "Esc Cancel".into(),
        ])
    }
}

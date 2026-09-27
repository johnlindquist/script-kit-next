//! Script Kit input extensions.
//!
//! Script Kit's launcher, Notes editor, Actions search and prompts relied on
//! APIs its former gpui-component 0.5 fork added to `InputState`. They are
//! re-homed here on gpui-kit's shared `InputBaseState<M>` so every mode
//! (`InputState`, `TextareaState`, `EditorState`) gets them, with only small
//! hooks in `state.rs` (text-version bumps, selection-change emission) and
//! `editor/indent.rs` (tab navigation).

use std::cell::Cell;
use std::ops::Range;

use gpui::{Bounds, Context, HighlightStyle, Hsla, Pixels, Point, SharedString, UnderlineStyle, Window, px};

use super::{
    EditorMode, InputBaseState, InputModeKind,
    state::{Backspace, Delete, DeleteToNextWordEnd, DeleteToPreviousWordStart, Paste, Redo, Undo},
};
use crate::actions::{SelectLeft, SelectRight};
use super::state::{MoveLeft, MoveRight};

/// Script Kit input events, emitted alongside gpui-kit's `InputEvent` (kept
/// separate so the upstream enum — matched exhaustively by gpui-ai — is
/// unchanged). Subscribe with `cx.subscribe(&state, |_, _, e: &ScriptKitInputEvent, _| ..)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptKitInputEvent {
    /// The caret or selection moved (emitted once per change).
    SelectionChange,
    /// Tab/Shift-Tab pressed while `tab_navigation` is on; the key is
    /// propagated instead of indenting (template/snippet tabstops).
    PressTab { secondary: bool },
}

impl<M: InputModeKind> gpui::EventEmitter<ScriptKitInputEvent> for InputBaseState<M> {}

/// Fingerprint of everything a revision epoch covers.
#[derive(Clone, PartialEq, Eq, Default)]
struct RevisionFingerprint {
    text_version: u64,
    selection_version: u64,
    selection: (usize, usize, bool),
    marked: Option<(usize, usize)>,
}

/// Per-state extension data (a single field on `InputBaseState`).
#[derive(Default)]
pub(crate) struct ScriptKitInputExt {
    /// Bumped at every text mutation site.
    pub(crate) text_version: u64,
    /// Bumped whenever the selection effectively moves, so an A→B→A
    /// round trip still advances `revision()` (frozen-anchor ABA guard).
    pub(crate) selection_version: u64,
    /// Lazily advanced revision epoch and the fingerprint it last covered.
    revision: Cell<(u64, Option<RevisionFingerprint>)>,
    /// Last selection reported through `InputEvent::SelectionChange`.
    last_reported_selection: Option<(usize, usize, bool)>,
    /// Tab/Shift-Tab propagate (and emit `PressTab`) instead of indenting.
    pub(crate) tab_navigation: bool,
    /// Host-owned colored ranges painted over syntax highlighting.
    highlight_ranges: Vec<(Range<usize>, Hsla)>,
    /// Semantic role per highlight range (same index), e.g. `markdownLink`.
    highlight_range_roles: Vec<String>,
    /// Highlight range currently under the pointer, with its role.
    hovered_highlight_range: Option<(Range<usize>, String)>,
    /// Host-owned word marks: a colored bar under each range; the text keeps its color.
    mark_ranges: Vec<(Range<usize>, Hsla)>,
    /// Semantic role per mark range (same index), e.g. `subject`.
    mark_range_roles: Vec<String>,
    /// Capture-only devtools fidelity scope for shaped line measurements.
    pub(crate) fidelity_scope: Option<SharedString>,
}

impl<M: InputModeKind> InputBaseState<M> {
    fn revision_fingerprint(&self) -> RevisionFingerprint {
        let active = self.active_selection();
        RevisionFingerprint {
            text_version: self.sk.text_version,
            selection_version: self.sk.selection_version,
            selection: (active.start, active.end, active.reversed),
            marked: self.ime_marked_range.map(|range| (range.start, range.end)),
        }
    }

    /// Monotonic epoch of effective text, cursor, selection and IME changes.
    /// Reading, rendering and no-op edits do not advance it.
    pub fn revision(&self) -> u64 {
        let fingerprint = self.revision_fingerprint();
        let (epoch, previous) = self.sk.revision.take();
        let epoch = if previous.as_ref() == Some(&fingerprint) {
            epoch
        } else if previous.is_none() && epoch == 0 && fingerprint == RevisionFingerprint::default()
        {
            0
        } else {
            epoch.checked_add(1).expect("input revision exhausted")
        };
        self.sk.revision.set((epoch, Some(fingerprint)));
        epoch
    }

    pub(super) fn emit_selection_change_if_moved(&mut self, cx: &mut Context<Self>) {
        let active = self.active_selection();
        let current = (active.start, active.end, active.reversed);
        if self.sk.last_reported_selection != Some(current) {
            self.sk.last_reported_selection = Some(current);
            self.sk.selection_version = self.sk.selection_version.wrapping_add(1);
            cx.emit(ScriptKitInputEvent::SelectionChange);
        }
    }

    /// Runtime scroll metrics for automation/devtools receipts, in CSS-like
    /// positive scrollTop/scrollLeft units.
    pub fn automation_scroll_metrics(&self) -> serde_json::Value {
        let live_offset = self.scroll_handle.offset();
        let effective_offset = self.deferred_scroll_offset.unwrap_or(live_offset);
        let viewport = self.input_bounds.size;
        let content = self.scroll_size;
        let max_scroll_top = (content.height - viewport.height).max(px(0.)).as_f32();
        let max_scroll_left = (content.width - viewport.width).max(px(0.)).as_f32();
        let scroll_top = (-effective_offset.y.as_f32()).clamp(0.0, max_scroll_top);
        let scroll_left = (-effective_offset.x.as_f32()).clamp(0.0, max_scroll_left);
        serde_json::json!({
            "schemaVersion": 1,
            "source": "gpui_base.input.InputBaseState",
            "available": true,
            "offsetUnit": "logicalPx",
            "scrollTop": scroll_top,
            "scrollLeft": scroll_left,
            "liveScrollTop": (-live_offset.y.as_f32()).clamp(0.0, max_scroll_top),
            "liveScrollLeft": (-live_offset.x.as_f32()).clamp(0.0, max_scroll_left),
            "hasDeferredScrollOffset": self.deferred_scroll_offset.is_some(),
            "scrollHeight": content.height.as_f32(),
            "scrollWidth": content.width.as_f32(),
            "clientHeight": viewport.height.as_f32(),
            "clientWidth": viewport.width.as_f32(),
            "maxScrollTop": max_scroll_top,
            "maxScrollLeft": max_scroll_left,
            "canScrollY": max_scroll_top > 0.0,
            "canScrollX": max_scroll_left > 0.0,
        })
    }

    /// Scroll to the end of the content on the next layout (offsets clamp).
    pub fn scroll_to_bottom(&mut self, cx: &mut Context<Self>) {
        let x = self.scroll_handle.offset().x;
        self.deferred_scroll_offset = Some(Point::new(x, px(-f32::MAX / 4.)));
        cx.notify();
    }

    /// Alias kept for callers that ask for the post-layout bottom scroll.
    pub fn scroll_to_bottom_after_layout(&mut self, cx: &mut Context<Self>) {
        self.scroll_to_bottom(cx);
    }

    /// Replace the value, put the caret at `cursor`, and keep the viewport.
    pub fn set_value_preserving_scroll(
        &mut self,
        value: impl Into<SharedString>,
        cursor: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let offset = self.deferred_scroll_offset.unwrap_or(self.scroll_handle.offset());
        self.set_value(value, window, cx);
        let cursor = cursor.min(self.text.len());
        self.set_selected_range(cursor..cursor, cx);
        self.deferred_scroll_offset = Some(offset);
        cx.notify();
    }

    /// Replace the value keeping the (clamped) selection and the viewport.
    pub fn set_value_preserving_selection_and_scroll(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selection = self.selected_range();
        let reversed = self.active_selection().reversed;
        let offset = self.deferred_scroll_offset.unwrap_or(self.scroll_handle.offset());
        self.set_value(value, window, cx);
        let len = self.text.len();
        self.set_selected_range(selection.start.min(len)..selection.end.min(len), cx);
        self.active_selection_mut().reversed = reversed && !selection.is_empty();
        self.deferred_scroll_offset = Some(offset);
        cx.notify();
    }

    /// Number of visual (soft-wrapped) rows in the last layout.
    pub fn soft_wrapped_lines_len(&self) -> usize {
        self.display_map.wrap_row_count()
    }

    /// Builder: Tab/Shift-Tab propagate and emit `PressTab` instead of indenting.
    pub fn tab_navigation(mut self, enabled: bool) -> Self {
        self.sk.tab_navigation = enabled;
        self
    }

    /// Setter for [`Self::tab_navigation`].
    pub fn set_tab_navigation(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.sk.tab_navigation = enabled;
        cx.notify();
    }

    /// Replace the host highlight ranges (colors only, default roles).
    pub fn set_highlight_ranges(&mut self, ranges: Vec<(Range<usize>, Hsla)>) {
        self.sk.highlight_range_roles = vec![String::new(); ranges.len()];
        self.sk.highlight_ranges = ranges;
        self.sk.hovered_highlight_range = None;
    }

    /// Replace the host highlight ranges with a semantic role per range.
    pub fn set_highlight_ranges_with_roles(&mut self, ranges: Vec<(Range<usize>, Hsla, String)>) {
        let (ranges, roles): (Vec<_>, Vec<_>) = ranges
            .into_iter()
            .map(|(range, color, role)| ((range, color), role))
            .unzip();
        self.sk.highlight_ranges = ranges;
        self.sk.highlight_range_roles = roles;
        if let Some((hovered, _)) = self.sk.hovered_highlight_range.as_ref() {
            if !self.sk.highlight_ranges.iter().any(|(range, _)| range == hovered) {
                self.sk.hovered_highlight_range = None;
            }
        }
    }

    pub fn clear_highlight_ranges(&mut self) {
        self.set_highlight_ranges(Vec::new());
    }

    pub fn highlight_ranges(&self) -> &[(Range<usize>, Hsla)] {
        &self.sk.highlight_ranges
    }

    pub fn highlight_range_roles(&self) -> &[String] {
        &self.sk.highlight_range_roles
    }

    /// The host highlight range under the pointer, with its role.
    pub fn hovered_highlight_range(&self) -> Option<(Range<usize>, String)> {
        self.sk.hovered_highlight_range.clone()
    }

    /// Replace the word marks: each range gets a bar in its color under the text, and the
    /// text keeps its own color.
    pub fn set_mark_ranges_with_roles(&mut self, ranges: Vec<(Range<usize>, Hsla, String)>) {
        let (ranges, roles): (Vec<_>, Vec<_>) = ranges
            .into_iter()
            .map(|(range, color, role)| ((range, color), role))
            .unzip();
        self.sk.mark_ranges = ranges;
        self.sk.mark_range_roles = roles;
    }

    pub fn clear_mark_ranges(&mut self) {
        self.set_mark_ranges_with_roles(Vec::new());
    }

    pub fn mark_ranges(&self) -> &[(Range<usize>, Hsla)] {
        &self.sk.mark_ranges
    }

    pub fn mark_range_roles(&self) -> &[String] {
        &self.sk.mark_range_roles
    }

    /// The marks to paint: none for masked text, and only ranges that fit the text.
    pub(super) fn script_kit_paint_marks(&self) -> Vec<(Range<usize>, Hsla)> {
        if self.masked {
            return Vec::new();
        }
        let len = self.text.len();
        self.sk
            .mark_ranges
            .iter()
            .filter(|(range, _)| range.start < range.end && range.end <= len)
            .cloned()
            .collect()
    }

    pub(super) fn update_script_kit_hovered_range(
        &mut self,
        offset: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        let next = offset.and_then(|offset| {
            self.sk
                .highlight_ranges
                .iter()
                .enumerate()
                .find(|(_, (range, _))| range.start <= offset && offset < range.end)
                .map(|(index, (range, _))| {
                    let role = self
                        .sk
                        .highlight_range_roles
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    (range.clone(), role)
                })
        });
        if next != self.sk.hovered_highlight_range {
            self.sk.hovered_highlight_range = next;
            cx.notify();
        }
    }

    /// Paint styles for the host highlight ranges within `visible`
    /// (the hovered one also gets an underline, like a hover-definition).
    pub(super) fn script_kit_highlight_styles(
        &self,
        visible: &Range<usize>,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        let hovered = self.sk.hovered_highlight_range.as_ref().map(|(r, _)| r.clone());
        let mut styles: Vec<(Range<usize>, HighlightStyle)> = self
            .sk
            .highlight_ranges
            .iter()
            .filter(|(range, _)| range.start < visible.end && range.end > visible.start)
            .map(|(range, color)| {
                let mut style = HighlightStyle {
                    color: Some(*color),
                    ..Default::default()
                };
                if hovered.as_ref() == Some(range) {
                    style.underline = Some(UnderlineStyle {
                        thickness: px(1.),
                        color: Some(*color),
                        wavy: false,
                    });
                }
                (range.start.max(visible.start)..range.end.min(visible.end), style)
            })
            .collect();
        styles.sort_by_key(|(range, _)| range.start);
        styles
    }

    /// Bounds of a byte range in window coordinates (last layout).
    pub fn highlight_range_bounds(&self, range: &Range<usize>) -> Option<Bounds<Pixels>> {
        self.range_to_bounds(range)
    }

    /// Byte offset under a window-space point (last layout).
    pub fn offset_for_window_position(&self, position: Point<Pixels>) -> usize {
        self.index_for_mouse_position(position).0
    }

    pub fn edit_backspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.backspace(&Backspace, window, cx);
    }

    pub fn edit_delete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.delete(&Delete, window, cx);
    }

    pub fn edit_delete_previous_word(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.delete_previous_word(&DeleteToPreviousWordStart, window, cx);
    }

    pub fn edit_delete_next_word(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.delete_next_word(&DeleteToNextWordEnd, window, cx);
    }

    pub fn edit_paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.paste(&Paste, window, cx);
    }

    pub fn edit_move_left(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.left(&MoveLeft, window, cx);
    }

    pub fn edit_move_right(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.right(&MoveRight, window, cx);
    }

    pub fn edit_select_left(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_left(&SelectLeft, window, cx);
    }

    pub fn edit_select_right(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_right(&SelectRight, window, cx);
    }

    pub fn edit_select_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_all(window, cx);
    }

    pub fn edit_undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.undo(&Undo, window, cx);
    }

    pub fn edit_redo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.redo(&Redo, window, cx);
    }
}

impl InputBaseState<EditorMode> {
    /// Show `text` as ghost-text inline completion at the cursor (Tab accepts).
    pub fn set_inline_completion_text(
        &mut self,
        text: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let text = text.into();
        if text.is_empty() {
            self.clear_inline_completion(cx);
            return;
        }
        self.extras.inline_completion.item = Some(lsp_types::InlineCompletionItem {
            insert_text: text,
            filter_text: None,
            range: None,
            command: None,
            insert_text_format: None,
        });
        cx.notify();
    }

}

impl InputBaseState<EditorMode> {
    /// Minimum visible rows for the editor viewport.
    pub fn rows(mut self, rows: usize) -> Self {
        if let super::mode::LayoutMode::CodeEditor { rows: value, .. } = &mut self.mode {
            *value = rows;
        }
        self
    }
}

/// Single-line mode extras: host-driven ghost text for the launcher filter.
#[derive(Default)]
pub struct SingleLineExtras {
    pub(crate) inline_completion: Option<lsp_types::InlineCompletionItem>,
    pub(crate) visible_without_focus: bool,
}

impl super::InputExtras for SingleLineExtras {
    fn inline_completion_item(&self) -> Option<&lsp_types::InlineCompletionItem> {
        self.inline_completion.as_ref()
    }

    fn inline_completion_visible_without_focus(&self) -> bool {
        self.visible_without_focus
    }
}

impl InputBaseState<super::InputMode> {
    /// Keep ghost text painted while the input is unfocused.
    pub fn inline_completion_visible_without_focus(mut self, enabled: bool) -> Self {
        self.extras.visible_without_focus = enabled;
        self
    }

    /// Show `text` as ghost-text inline completion at the cursor.
    pub fn set_inline_completion_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        let text = text.into();
        if text.is_empty() {
            self.clear_inline_completion(cx);
            return;
        }
        self.extras.inline_completion = Some(lsp_types::InlineCompletionItem {
            insert_text: text,
            filter_text: None,
            range: None,
            command: None,
            insert_text_format: None,
        });
        cx.notify();
    }

    /// Whether ghost text is currently shown.
    pub fn has_inline_completion(&self) -> bool {
        self.extras.inline_completion.is_some()
    }

    /// Drop the ghost text.
    pub fn clear_inline_completion(&mut self, cx: &mut Context<Self>) {
        if self.extras.inline_completion.take().is_some() {
            cx.notify();
        }
    }

    /// Insert the ghost text at the cursor. Returns false when there was none.
    pub fn accept_inline_completion(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(item) = self.extras.inline_completion.take() else {
            return false;
        };
        let cursor = self.cursor();
        let range_utf16 = self.range_to_utf16(&(cursor..cursor));
        self.replace_text_in_range_silent(Some(range_utf16), &item.insert_text, window, cx);
        true
    }
}

impl<M: InputModeKind> InputBaseState<M> {
    /// Window-relative caret bounds from the most recent paint, if any.
    pub fn last_cursor_bounds(&self) -> Option<Bounds<Pixels>> {
        self.last_layout.as_ref().and_then(|layout| layout.cursor_bounds)
    }

    /// Select `start..end` (UTF-8 byte offsets, clamped), scroll it into view
    /// and focus the input — the fork's `set_selection` contract.
    pub fn set_selection_focused(
        &mut self,
        start: usize,
        end: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let len = self.text.len();
        let before = self.selected_range();
        self.set_selected_range(start.min(len)..end.min(len), cx);
        if self.selected_range() != before {
            self.sk.selection_version = self.sk.selection_version.wrapping_add(1);
        }
        self.focus(window, cx);
    }
}

impl<M: InputModeKind> InputBaseState<M> {
    /// Record shaped/wrapped line boxes under `scope` during a devtools
    /// fidelity capture. Raw text is never retained by the fidelity frame.
    pub fn fidelity_scope(mut self, scope: impl Into<SharedString>) -> Self {
        self.sk.fidelity_scope = Some(scope.into());
        self
    }

    pub(crate) fn sk_fidelity_scope(&self) -> Option<&SharedString> {
        self.sk.fidelity_scope.as_ref()
    }
}

/// Fidelity evidence for one painted soft-wrapped line (fork parity:
/// `textLineBox` metadata, exact-text hash, layout hash; never raw text).
pub(crate) fn fidelity_line_evidence(
    base_scope: &str,
    row: usize,
    wrapped_index: usize,
    wrapped_count: usize,
    shaped_line: &gpui::ShapedLine,
    line_height: Pixels,
    scale_factor: f32,
) -> (String, String, String, serde_json::Value) {
    let scope_id = format!("{base_scope}.line.{row}.{wrapped_index}");
    let text = shaped_line.text.as_str();
    let text_hash = Window::fidelity_text_hash(text);
    let font_fingerprint_input = shaped_line
        .runs
        .iter()
        .map(|run| format!("{:?}", run.font_id))
        .collect::<Vec<_>>()
        .join(":");
    let font_family_fingerprint = Window::fidelity_text_hash(&font_fingerprint_input);
    let layout_hash = Window::fidelity_text_hash(&format!(
        "{}:{}:{}:{}:{}:{}",
        f32::from(shaped_line.font_size),
        f32::from(shaped_line.ascent),
        f32::from(shaped_line.descent),
        f32::from(shaped_line.width),
        f32::from(line_height),
        scale_factor,
    ));
    let grapheme_count =
        unicode_segmentation::UnicodeSegmentation::graphemes(text, true).count();
    let metadata = serde_json::json!({
        "schemaVersion": 1,
        "measurementId": format!("text:{base_scope}:line:{row}:{wrapped_index}"),
        "semanticId": format!("input:{base_scope}"),
        "role": "textLineBox",
        "fontFamilyFingerprint": font_family_fingerprint,
        "fontSize": f32::from(shaped_line.font_size),
        "fontWeight": "mixedOrRendererOwned",
        "lineHeight": f32::from(line_height),
        "ascent": f32::from(shaped_line.ascent),
        "descent": f32::from(shaped_line.descent),
        "backingScaleFactor": scale_factor,
        "fontsReady": true,
        "wrappingPolicy": if wrapped_count > 1 { "softWrap" } else { "none" },
        "truncationPolicy": "fullDisplay",
        "contentKind": "userContent",
        "graphemeCount": grapheme_count,
        "lineCount": 1,
        "rawContentReturned": false,
    });
    (scope_id, text_hash, layout_hash, metadata)
}

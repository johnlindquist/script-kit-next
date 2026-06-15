use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, Hsla, ScrollHandle, Window};
use gpui_component::input::InputState;
use gpui_component::theme::ActiveTheme;
use std::ops::Range;

use crate::notes::markdown_highlighting::register_markdown_highlighter;

use super::types::{
    NotesEditorConfig, NotesEditorInputSizing, NotesEditorLayout, NotesEditorMarkdownConfig,
};

/// Shared markdown editor used by the Notes window and future Day Page surface.
///
/// Owns markdown editing, formatting entry points, code highlighting registration,
/// preview rendering, and editor focus. The host binds document content, wires
/// save/change callbacks, and supplies chrome-specific empty states.
pub struct NotesEditor {
    pub(crate) input_state: Entity<InputState>,
    pub(crate) preview_scroll_handle: ScrollHandle,
    pub(crate) layout: NotesEditorLayout,
    last_markdown_link_highlight_text: String,
    last_markdown_link_highlight_ranges: Vec<(Range<usize>, Hsla, String)>,
}

impl NotesEditor {
    pub fn new_markdown_pair<T>(
        window: &mut Window,
        cx: &mut Context<T>,
        config: NotesEditorMarkdownConfig,
    ) -> (Entity<InputState>, Entity<Self>)
    where
        T: 'static,
    {
        register_markdown_highlighter();

        let editor_config = config.editor;
        let placeholder = editor_config.placeholder.clone();
        let initial_content = editor_config.initial_content.clone();
        let sizing = config.sizing;
        let input_state = cx.new(|cx| {
            let state = InputState::new(window, cx)
                .code_editor("markdown")
                .code_editor_dynamic_bottom_margin(false)
                .line_number(false)
                .searchable(true)
                .placeholder(placeholder)
                .default_value(initial_content);
            match sizing {
                NotesEditorInputSizing::Rows(rows) => state.rows(rows),
                NotesEditorInputSizing::AutoGrow { min_rows, max_rows } => {
                    state.auto_grow(min_rows, max_rows)
                }
            }
        });
        let notes_editor = cx.new(|_| NotesEditor::new(input_state.clone(), editor_config));
        notes_editor.update(cx, |editor, cx| editor.sync_markdown_link_highlights(cx));
        (input_state, notes_editor)
    }

    pub fn new(input_state: Entity<InputState>, config: NotesEditorConfig) -> Self {
        register_markdown_highlighter();

        Self {
            input_state,
            preview_scroll_handle: ScrollHandle::new(),
            layout: config.layout,
            last_markdown_link_highlight_text: String::new(),
            last_markdown_link_highlight_ranges: Vec::new(),
        }
    }

    pub fn input_state(&self) -> Entity<InputState> {
        self.input_state.clone()
    }

    pub fn preview_scroll_handle(&self) -> &ScrollHandle {
        &self.preview_scroll_handle
    }

    pub fn layout(&self) -> NotesEditorLayout {
        self.layout
    }

    pub fn set_layout(&mut self, layout: NotesEditorLayout) {
        self.layout = layout;
    }

    pub fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    pub(crate) fn focus_with_cursor_at_end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            let cursor = state.value().len();
            state.set_selection(cursor, cursor, window, cx);
            // `set_selection`'s scroll_to is a no-op until the element has
            // painted (last_layout/last_bounds are None on the load/mount
            // frame), so the Day Page would land at the top on open/reopen.
            // This vendor flag is consumed during the next paint
            // (element.rs: scroll_to_bottom_after_layout) to force the scroll
            // offset to the bottom once layout commits.
            state.scroll_to_bottom_after_layout(cx);
            state.scroll_to_bottom(cx);
        });
    }

    pub(crate) fn scroll_to_bottom(&mut self, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            state.scroll_to_bottom_after_layout(cx);
            state.scroll_to_bottom(cx);
        });
    }

    pub fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input_state.read(cx).focus_handle(cx)
    }

    pub fn is_focused(&self, window: &Window, cx: &App) -> bool {
        self.focus_handle(cx).is_focused(window)
    }

    pub fn set_value(
        &mut self,
        value: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        self.input_state.update(cx, |state, cx| {
            state.set_value(value, window, cx);
        });
        self.sync_markdown_link_highlights(cx);
    }

    pub fn load_value_with_cursor_at_end(
        &mut self,
        value: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        let cursor = value.len();
        self.input_state.update(cx, |state, cx| {
            state.set_value(value, window, cx);
            state.set_selection(cursor, cursor, window, cx);
            state.scroll_to_bottom_after_layout(cx);
            state.scroll_to_bottom(cx);
        });
        self.sync_markdown_link_highlights(cx);
    }

    pub fn set_value_with_cursor_at_end(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load_value_with_cursor_at_end(text, window, cx);
    }

    pub fn set_selection(
        &mut self,
        start: usize,
        end: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.input_state.update(cx, |state, cx| {
            state.set_selection(start, end, window, cx);
        });
    }

    pub fn content(&self, cx: &App) -> String {
        self.input_state.read(cx).value().to_string()
    }

    pub fn selection(&self, cx: &App) -> std::ops::Range<usize> {
        self.input_state.read(cx).selection()
    }

    pub fn soft_wrapped_lines_len(&self, cx: &App) -> usize {
        self.input_state.read(cx).soft_wrapped_lines_len()
    }

    pub fn has_inline_completion(&self, cx: &App) -> bool {
        self.input_state.read(cx).has_inline_completion()
    }

    pub fn sync_inline_completion(&mut self, suffix: Option<String>, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| match suffix {
            Some(suffix) => state.set_inline_completion_text(suffix, cx),
            None => {
                if state.has_inline_completion() {
                    state.clear_inline_completion(cx);
                }
            }
        });
    }

    pub fn read_input<F, R>(&self, cx: &App, f: F) -> R
    where
        F: FnOnce(&InputState) -> R,
    {
        f(self.input_state.read(cx))
    }

    pub fn markdown_runtime_info(&self) -> crate::protocol::ElementEditorRuntimeInfo {
        crate::notes::markdown_highlighting::markdown_editor_runtime_info()
    }

    pub fn markdown_runtime_info_with_scroll(
        &self,
        cx: &App,
    ) -> crate::protocol::ElementEditorRuntimeInfo {
        let mut info = self.markdown_runtime_info();
        info.editor_scroll_metrics =
            Some(self.read_input(cx, |state| state.automation_scroll_metrics()));
        info.markdown_link_highlight_ranges = Some(self.markdown_link_highlight_runtime_info(cx));
        info
    }

    fn markdown_link_highlight_runtime_info(&self, cx: &App) -> serde_json::Value {
        self.read_input(cx, |state| {
            let text = state.value();
            let roles = state.highlight_range_roles();
            let ranges = state
                .highlight_ranges()
                .iter()
                .enumerate()
                .filter_map(|(index, (range, _color))| {
                    let text = text.get(range.clone())?;
                    Some(serde_json::json!({
                        "range": [range.start, range.end],
                        "text": text,
                        "role": roles
                            .get(index)
                            .cloned()
                            .unwrap_or_else(|| "markdownLink".to_string()),
                    }))
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "count": ranges.len(),
                "ranges": ranges,
            })
        })
    }

    pub fn sync_markdown_link_highlights(&mut self, cx: &mut Context<Self>) {
        let text = self.input_state.read(cx).value().to_string();
        if text == self.last_markdown_link_highlight_text {
            return;
        }

        let accent = cx.theme().accent;
        let ranges = markdown_link_highlight_ranges(&text, accent);
        if ranges != self.last_markdown_link_highlight_ranges {
            self.input_state.update(cx, |state, _cx| {
                state.set_highlight_ranges_with_roles(ranges.clone());
            });
            self.last_markdown_link_highlight_ranges = ranges;
        }
        self.last_markdown_link_highlight_text = text;
    }
}

fn markdown_link_highlight_ranges(text: &str, accent: Hsla) -> Vec<(Range<usize>, Hsla, String)> {
    let mut ranges = Vec::new();
    for line in markdown_non_code_lines(text) {
        collect_reference_definition_destination(text, line.clone(), accent, &mut ranges);
        collect_inline_markdown_links(text, line.clone(), accent, &mut ranges);
        collect_autolinks_and_bare_urls(text, line, accent, &mut ranges);
    }
    ranges.sort_by(|(a, _, _), (b, _, _)| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));
    ranges.dedup_by(|(a, _, ar), (b, _, br)| a == b && ar == br);
    ranges
}

fn markdown_non_code_lines(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut in_fence = false;
    let mut offset = 0;
    for segment in text.split_inclusive('\n') {
        let line_end = offset + segment.len();
        let line = segment.trim_end_matches('\n');
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            in_fence = !in_fence;
            offset = line_end;
            continue;
        }
        if !in_fence {
            ranges.push(offset..line_end);
        }
        offset = line_end;
    }
    if offset < text.len() && !in_fence {
        ranges.push(offset..text.len());
    }
    ranges
}

fn collect_inline_markdown_links(
    text: &str,
    line: Range<usize>,
    accent: Hsla,
    ranges: &mut Vec<(Range<usize>, Hsla, String)>,
) {
    let bytes = text.as_bytes();
    let mut index = line.start;
    while index < line.end {
        if bytes[index] != b'[' || index > 0 && bytes[index - 1] == b'!' {
            index += 1;
            continue;
        }
        let Some(label_close) = find_unescaped_byte(text, index + 1, line.end, b']') else {
            break;
        };
        let label = index + 1..label_close;
        match bytes.get(label_close + 1).copied() {
            Some(b'(') => {
                if let Some(dest_end) =
                    find_markdown_destination_end(text, label_close + 2, line.end)
                {
                    if label.start < label.end {
                        ranges.push((label, accent, "markdownLinkText".to_string()));
                    }
                    let dest = trim_ascii_range(text, label_close + 2..dest_end);
                    if dest.start < dest.end {
                        ranges.push((dest, accent, "markdownLinkUri".to_string()));
                    }
                    index = dest_end + 1;
                    continue;
                }
            }
            Some(b'[') => {
                if let Some(ref_close) = find_unescaped_byte(text, label_close + 2, line.end, b']')
                {
                    if label.start < label.end {
                        ranges.push((label, accent, "markdownLinkText".to_string()));
                    }
                    let reference = label_close + 2..ref_close;
                    if reference.start < reference.end {
                        ranges.push((reference, accent, "markdownLinkReference".to_string()));
                    }
                    index = ref_close + 1;
                    continue;
                }
            }
            _ => {}
        }
        index = label_close + 1;
    }
}

fn collect_reference_definition_destination(
    text: &str,
    line: Range<usize>,
    accent: Hsla,
    ranges: &mut Vec<(Range<usize>, Hsla, String)>,
) {
    let line_text = &text[line.clone()];
    let trimmed_start = line.start + line_text.len() - line_text.trim_start().len();
    if text.as_bytes().get(trimmed_start) != Some(&b'[') {
        return;
    }
    let Some(label_close) = find_unescaped_byte(text, trimmed_start + 1, line.end, b']') else {
        return;
    };
    if text.as_bytes().get(label_close + 1) != Some(&b':') {
        return;
    }
    let label = trimmed_start + 1..label_close;
    if label.start < label.end {
        ranges.push((label, accent, "markdownLinkReference".to_string()));
    }
    let dest = trim_ascii_range(text, label_close + 2..line.end);
    if dest.start < dest.end {
        ranges.push((dest, accent, "markdownLinkUri".to_string()));
    }
}

fn collect_autolinks_and_bare_urls(
    text: &str,
    line: Range<usize>,
    accent: Hsla,
    ranges: &mut Vec<(Range<usize>, Hsla, String)>,
) {
    let bytes = text.as_bytes();
    let mut index = line.start;
    while index < line.end {
        if bytes[index] == b'<' {
            if let Some(close) = find_unescaped_byte(text, index + 1, line.end, b'>') {
                let inner = index + 1..close;
                if text[inner.clone()].starts_with("http://")
                    || text[inner.clone()].starts_with("https://")
                    || text[inner.clone()].starts_with("scriptkit://")
                    || text[inner.clone()].starts_with("file:")
                {
                    ranges.push((inner, accent, "markdownLinkUri".to_string()));
                }
                index = close + 1;
                continue;
            }
        }
        if starts_url_at(text, index) {
            let mut end = index;
            while end < line.end && !text.as_bytes()[end].is_ascii_whitespace() {
                end += 1;
            }
            let url = trim_url_trailing_punctuation(text, index..end);
            if url.start < url.end
                && !ranges
                    .iter()
                    .any(|(range, _, _)| ranges_overlap(range, &url))
            {
                ranges.push((url, accent, "markdownLinkUri".to_string()));
            }
            index = end;
            continue;
        }
        index += 1;
    }
}

fn starts_url_at(text: &str, index: usize) -> bool {
    let bytes = &text.as_bytes()[index..];
    bytes.starts_with(b"http://")
        || bytes.starts_with(b"https://")
        || bytes.starts_with(b"scriptkit://")
        || bytes.starts_with(b"file:")
}

fn find_unescaped_byte(text: &str, start: usize, end: usize, needle: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = start;
    while index < end {
        if bytes[index] == needle && (index == 0 || bytes[index - 1] != b'\\') {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn find_markdown_destination_end(text: &str, start: usize, end: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut index = start;
    while index < end {
        match bytes[index] {
            b'\\' => index += 1,
            b'(' => depth += 1,
            b')' if depth == 0 => return Some(index),
            b')' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    None
}

fn trim_ascii_range(text: &str, range: Range<usize>) -> Range<usize> {
    let bytes = text.as_bytes();
    let mut start = range.start;
    let mut end = range.end;
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    start..end
}

fn trim_url_trailing_punctuation(text: &str, range: Range<usize>) -> Range<usize> {
    let bytes = text.as_bytes();
    let mut end = range.end;
    while end > range.start {
        match bytes[end - 1] {
            b'.' | b',' | b';' | b':' => end -= 1,
            _ => break,
        }
    }
    range.start..end
}

fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

#[cfg(test)]
mod tests {
    use super::markdown_link_highlight_ranges;
    use gpui::rgb;

    fn highlighted_texts(input: &str) -> Vec<String> {
        markdown_link_highlight_ranges(input, rgb(0xffcc00).into())
            .into_iter()
            .map(|(range, _, role)| format!("{role}:{}", &input[range]))
            .collect()
    }

    #[test]
    fn markdown_link_highlights_cover_inline_links_and_urls() {
        let input = "[Screenflow](scriptkit://spine/file/screenflow)\nhttps://example.com/path,\n";
        assert_eq!(
            highlighted_texts(input),
            vec![
                "markdownLinkText:Screenflow",
                "markdownLinkUri:scriptkit://spine/file/screenflow",
                "markdownLinkUri:https://example.com/path",
            ]
        );
    }

    #[test]
    fn markdown_link_highlights_cover_reference_links() {
        let input = "[Guide][guide]\n[guide]: https://scriptkit.com/guide\n";
        assert_eq!(
            highlighted_texts(input),
            vec![
                "markdownLinkText:Guide",
                "markdownLinkReference:guide",
                "markdownLinkReference:guide",
                "markdownLinkUri:https://scriptkit.com/guide",
            ]
        );
    }

    #[test]
    fn markdown_link_highlights_skip_fenced_code() {
        let input = "```md\n[Nope](https://example.com)\n```\n[Yep](https://scriptkit.com)\n";
        assert_eq!(
            highlighted_texts(input),
            vec![
                "markdownLinkText:Yep",
                "markdownLinkUri:https://scriptkit.com",
            ]
        );
    }

    #[test]
    fn markdown_link_highlights_do_not_panic_after_non_ascii_text() {
        let input = "Script Kit memory — the Brain [Guide](https://scriptkit.com/guide)";
        assert_eq!(
            highlighted_texts(input),
            vec![
                "markdownLinkText:Guide",
                "markdownLinkUri:https://scriptkit.com/guide",
            ]
        );
    }
}

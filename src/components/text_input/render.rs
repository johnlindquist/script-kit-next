use super::TextSelection;
use crate::{
    panel::{CURSOR_HEIGHT_LG, CURSOR_WIDTH},
    ui_foundation::ALPHA_SELECTION,
};
use gpui::{div, px, rgb, rgba, Div, Hsla, IntoElement, ParentElement, Styled};

/// A character range to render with a specific text color.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TextHighlightRange {
    /// Start character index (inclusive).
    pub start: usize,
    /// End character index (exclusive).
    pub end: usize,
    /// Override text color for this range.
    pub color: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextInputRenderIndicator<'a> {
    pub text: &'a str,
    pub color: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextInputRenderConfig<'a> {
    pub text: &'a str,
    pub cursor: usize,
    pub selection: Option<TextSelection>,
    pub window: Option<(usize, usize)>,
    pub cursor_visible: bool,
    pub cursor_width: f32,
    pub cursor_height: f32,
    pub cursor_margin_y: f32,
    pub cursor_gap_before: f32,
    pub cursor_gap_after: f32,
    pub cursor_color: u32,
    pub cursor_hidden_color: Option<Hsla>,
    pub text_color: u32,
    pub selection_color: u32,
    pub selection_alpha: u32,
    pub selection_text_color: u32,
    pub container_height: Option<f32>,
    pub overflow_x_hidden: bool,
    pub leading_indicator: Option<TextInputRenderIndicator<'a>>,
    pub trailing_indicator: Option<TextInputRenderIndicator<'a>>,
    pub transform: Option<fn(&str) -> String>,
    /// Character ranges to render with a specific text color (e.g. gold @mentions).
    /// Ranges are in terms of character indices in the full text.
    pub highlight_ranges: &'a [TextHighlightRange],
}

impl<'a> TextInputRenderConfig<'a> {
    pub(crate) fn default_for_prompt(text: &'a str) -> Self {
        Self {
            text,
            cursor: 0,
            selection: None,
            window: None,
            cursor_visible: true,
            cursor_width: CURSOR_WIDTH,
            cursor_height: CURSOR_HEIGHT_LG,
            cursor_margin_y: 0.0,
            cursor_gap_before: 0.0,
            cursor_gap_after: 0.0,
            cursor_color: 0xffffff,
            cursor_hidden_color: None,
            text_color: 0xffffff,
            selection_color: 0x0000ff,
            selection_alpha: u32::from(ALPHA_SELECTION),
            selection_text_color: 0xffffff,
            container_height: None,
            overflow_x_hidden: false,
            leading_indicator: None,
            trailing_indicator: None,
            transform: None,
            highlight_ranges: &[],
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ComputedTextInputSegments {
    before: String,
    selected: String,
    after: String,
    /// Character offset of `before` in the full text.
    before_char_offset: usize,
    /// Character offset of `after` in the full text.
    after_char_offset: usize,
    show_cursor: bool,
    show_leading_indicator: bool,
    show_trailing_indicator: bool,
}

pub(crate) fn render_text_input_cursor_selection(config: TextInputRenderConfig<'_>) -> Div {
    let segments = compute_text_input_segments(&config);
    let mut content = div()
        .flex()
        .flex_row()
        .items_center()
        .text_color(rgb(config.text_color));

    if let Some(height) = config.container_height {
        content = content.h(px(height));
    }
    if config.overflow_x_hidden {
        content = content.overflow_x_hidden();
    }
    if segments.show_leading_indicator {
        if let Some(indicator) = config.leading_indicator {
            content = content.child(
                div()
                    .text_color(rgb(indicator.color))
                    .child(indicator.text.to_string()),
            );
        }
    }
    let has_highlights = !config.highlight_ranges.is_empty();
    if !segments.before.is_empty() {
        if has_highlights {
            content = content.child(render_segment_with_highlights(
                &segments.before,
                segments.before_char_offset,
                config.highlight_ranges,
                config.text_color,
                config.transform,
            ));
        } else {
            content =
                content.child(div().child(format_segment(&segments.before, config.transform)));
        }
    }
    if segments.show_cursor {
        if config.cursor_gap_before > 0.0 {
            content = content.child(div().w(px(config.cursor_gap_before)));
        }
        content = content.child(render_cursor(&config));
        if config.cursor_gap_after > 0.0 {
            content = content.child(div().w(px(config.cursor_gap_after)));
        }
    } else if !segments.selected.is_empty() {
        content = content.child(
            div()
                .bg(rgba((config.selection_color << 8) | config.selection_alpha))
                .text_color(rgb(config.selection_text_color))
                .child(format_segment(&segments.selected, config.transform)),
        );
    }
    if !segments.after.is_empty() {
        if has_highlights {
            content = content.child(render_segment_with_highlights(
                &segments.after,
                segments.after_char_offset,
                config.highlight_ranges,
                config.text_color,
                config.transform,
            ));
        } else {
            content =
                content.child(div().child(format_segment(&segments.after, config.transform)));
        }
    }
    if segments.show_trailing_indicator {
        if let Some(indicator) = config.trailing_indicator {
            content = content.child(
                div()
                    .text_color(rgb(indicator.color))
                    .child(indicator.text.to_string()),
            );
        }
    }

    content
}

fn render_cursor(config: &TextInputRenderConfig<'_>) -> Div {
    let mut cursor = div().w(px(config.cursor_width)).h(px(config.cursor_height));
    if config.cursor_margin_y > 0.0 {
        cursor = cursor.my(px(config.cursor_margin_y));
    }
    if let Some(hidden_color) = config.cursor_hidden_color {
        cursor = cursor.bg(hidden_color);
    }
    if config.cursor_visible {
        cursor = cursor.bg(rgb(config.cursor_color));
    }
    cursor
}

fn format_segment(segment: &str, transform: Option<fn(&str) -> String>) -> String {
    match transform {
        Some(transform_fn) => transform_fn(segment),
        None => segment.to_string(),
    }
}

/// Render a text segment, splitting it into sub-spans where highlight ranges
/// overlap. Characters outside any highlight range use `default_color`.
fn render_segment_with_highlights(
    segment: &str,
    segment_char_offset: usize,
    highlights: &[TextHighlightRange],
    default_color: u32,
    transform: Option<fn(&str) -> String>,
) -> gpui::AnyElement {
    if segment.is_empty() {
        return div().into_any_element();
    }

    let seg_chars: Vec<char> = segment.chars().collect();
    let seg_len = seg_chars.len();

    // Build a color map for each character in this segment.
    let mut colors: Vec<u32> = vec![default_color; seg_len];
    for hl in highlights {
        if hl.end <= segment_char_offset || hl.start >= segment_char_offset + seg_len {
            continue;
        }
        let local_start = hl.start.saturating_sub(segment_char_offset);
        let local_end = (hl.end - segment_char_offset).min(seg_len);
        for c in &mut colors[local_start..local_end] {
            *c = hl.color;
        }
    }

    // Group consecutive characters with the same color into runs.
    let mut spans: Vec<(String, u32)> = Vec::new();
    let mut run_start = 0;
    while run_start < seg_len {
        let run_color = colors[run_start];
        let mut run_end = run_start + 1;
        while run_end < seg_len && colors[run_end] == run_color {
            run_end += 1;
        }
        let text: String = seg_chars[run_start..run_end].iter().collect();
        spans.push((text, run_color));
        run_start = run_end;
    }

    if spans.len() == 1 {
        let (text, color) = &spans[0];
        return div()
            .text_color(rgb(*color))
            .child(format_segment(text, transform))
            .into_any_element();
    }

    let mut container = div().flex().flex_row();
    for (text, color) in &spans {
        container = container.child(
            div()
                .text_color(rgb(*color))
                .child(format_segment(text, transform)),
        );
    }
    container.into_any_element()
}

fn compute_text_input_segments(config: &TextInputRenderConfig<'_>) -> ComputedTextInputSegments {
    let chars: Vec<char> = config.text.chars().collect();
    let text_len = chars.len();
    let (window_start, window_end) = clamped_window(config.window, text_len);
    let visible_chars = &chars[window_start..window_end];
    let local_cursor = config
        .cursor
        .min(window_end)
        .saturating_sub(window_start)
        .min(visible_chars.len());

    let local_selection = config.selection.and_then(|selection| {
        let (selection_start, selection_end) = selection.range();
        let selection_start = selection_start.min(text_len);
        let selection_end = selection_end.min(text_len);
        if selection_start >= selection_end {
            return None;
        }

        let visible_selection_start = selection_start.clamp(window_start, window_end);
        let visible_selection_end = selection_end.clamp(window_start, window_end);
        if visible_selection_start >= visible_selection_end {
            return None;
        }

        Some((
            visible_selection_start - window_start,
            visible_selection_end - window_start,
        ))
    });

    let (before, selected, after, show_cursor, before_char_offset, after_char_offset) =
        if let Some((selection_start, selection_end)) = local_selection {
            (
                visible_chars[..selection_start].iter().collect(),
                visible_chars[selection_start..selection_end]
                    .iter()
                    .collect(),
                visible_chars[selection_end..].iter().collect(),
                false,
                window_start,
                window_start + selection_end,
            )
        } else {
            (
                visible_chars[..local_cursor].iter().collect(),
                String::new(),
                visible_chars[local_cursor..].iter().collect(),
                true,
                window_start,
                window_start + local_cursor,
            )
        };

    ComputedTextInputSegments {
        before,
        selected,
        after,
        before_char_offset,
        after_char_offset,
        show_cursor,
        show_leading_indicator: window_start > 0,
        show_trailing_indicator: window_end < text_len,
    }
}

fn clamped_window(window: Option<(usize, usize)>, text_len: usize) -> (usize, usize) {
    match window {
        Some((start, end)) => {
            let mut start = start.min(text_len);
            let mut end = end.min(text_len);
            if end < start {
                std::mem::swap(&mut start, &mut end);
            }
            (start, end)
        }
        None => (0, text_len),
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_text_input_segments, TextInputRenderConfig};
    use crate::components::TextSelection;

    fn render_config_for(text: &str) -> TextInputRenderConfig<'_> {
        TextInputRenderConfig {
            ..TextInputRenderConfig::default_for_prompt(text)
        }
    }

    #[test]
    fn test_default_for_prompt_sets_canonical_cursor_height_and_selection_alpha() {
        let config = TextInputRenderConfig::default_for_prompt("hello");

        assert_eq!(config.cursor_height, crate::panel::CURSOR_HEIGHT_LG);
        assert_eq!(
            config.selection_alpha,
            u32::from(crate::ui_foundation::ALPHA_SELECTION)
        );
    }

    #[test]
    fn test_compute_text_input_segments_clamps_indices_when_selection_exceeds_text_bounds() {
        let mut config = render_config_for("hello");
        config.cursor = 99;
        config.selection = Some(TextSelection {
            anchor: 1,
            cursor: 99,
        });

        let segments = compute_text_input_segments(&config);

        assert_eq!(segments.before, "h");
        assert_eq!(segments.selected, "ello");
        assert_eq!(segments.after, "");
        assert!(!segments.show_cursor);
    }

    #[test]
    fn test_compute_text_input_segments_windows_visible_selection_when_range_is_truncated() {
        let mut config = render_config_for("abcdefghijklmnopqrstuvwxyz");
        config.window = Some((10, 18));
        config.selection = Some(TextSelection {
            anchor: 5,
            cursor: 20,
        });

        let segments = compute_text_input_segments(&config);

        assert_eq!(segments.before, "");
        assert_eq!(segments.selected, "klmnopqr");
        assert_eq!(segments.after, "");
        assert!(!segments.show_cursor);
        assert!(segments.show_leading_indicator);
        assert!(segments.show_trailing_indicator);
    }

    #[test]
    fn test_compute_text_input_segments_splits_before_and_after_when_cursor_has_no_selection() {
        let mut config = render_config_for("abcdef");
        config.cursor = 3;

        let segments = compute_text_input_segments(&config);

        assert_eq!(segments.before, "abc");
        assert_eq!(segments.selected, "");
        assert_eq!(segments.after, "def");
        assert!(segments.show_cursor);
    }

    #[test]
    fn test_compute_text_input_segments_swaps_window_bounds_when_range_is_reversed() {
        let mut config = render_config_for("abcdefghij");
        config.window = Some((8, 2));
        config.cursor = 5;

        let segments = compute_text_input_segments(&config);

        assert_eq!(segments.before, "cde");
        assert_eq!(segments.after, "fgh");
        assert!(segments.show_leading_indicator);
        assert!(segments.show_trailing_indicator);
    }
}

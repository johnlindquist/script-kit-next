//! Paint the insertion point over the very same shaped lines used to draw and hit-test text.
//! An inline bar between two text nodes changes wrapping, kerning and (on Return) even the row.
use gpui::{prelude::FluentBuilder as _, *};

use super::helpers::byte_idx_from_char_idx;
use super::{FormFieldColors, FormFieldMetrics};

#[derive(Default)]
pub(super) struct FieldViewport {
    pub scroll: ScrollHandle,
    shown_revision: Option<u64>,
}

pub(super) struct FieldHitMap {
    layout: TextLayout,
    displayed: SharedString,
    chars: usize,
}
impl FieldHitMap {
    pub fn at(&self, position: Point<Pixels>) -> usize {
        if self.chars == 0 {
            return 0;
        }
        // TextLayout::index_for_position hits a character (suitable for a link).
        // Insertion wants the *nearer boundary* of that glyph, including its right half.
        let p = position - self.layout.bounds().origin;
        if p.y < px(0.) {
            return 0;
        }
        let height = self.layout.line_height();
        let mut top = px(0.);
        let mut start = 0;
        for line in self.layout.line_layouts() {
            let bottom = top + line.size(height).height;
            if p.y < bottom {
                let local = point(p.x, p.y - top);
                let byte = start
                    + line
                        .closest_index_for_position(local, height)
                        .unwrap_or_else(|i| i);
                return self
                    .displayed
                    .char_indices()
                    .take_while(|(i, _)| *i < byte)
                    .count()
                    .min(self.chars);
            }
            start += line.len() + 1;
            top = bottom;
        }
        self.chars
    }
}

impl FieldViewport {
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        value: String,
        placeholder: &str,
        cursor: usize,
        selection: Option<(usize, usize)>,
        focused: bool,
        multiline: bool,
        revision: u64,
        colors: FormFieldColors,
    ) -> (Div, FieldHitMap) {
        let metrics = FormFieldMetrics::from_colors(colors);
        let empty = value.is_empty();
        let chars = value.chars().count();
        let cursor = byte_idx_from_char_idx(&value, cursor);
        let range = selection
            .filter(|(a, b)| focused && a != b)
            .map(|(a, b)| byte_idx_from_char_idx(&value, a)..byte_idx_from_char_idx(&value, b));
        let displayed: SharedString = if empty {
            placeholder.to_owned().into()
        } else {
            value.into()
        };
        let selection_color =
            rgba((colors.border_focused << 8) | u32::from(crate::ui_foundation::ALPHA_SELECTION));
        let breaks = range
            .as_ref()
            .map(|r| {
                displayed[r.clone()]
                    .match_indices('\n')
                    .map(|(i, _)| r.start + i)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let collapsed = range.is_none();
        let text = StyledText::new(displayed.clone()).with_highlights(range.map(|r| {
            (
                r,
                HighlightStyle {
                    background_color: Some(selection_color.into()),
                    ..Default::default()
                },
            )
        }));
        let layout = text.layout().clone();
        let hit = FieldHitMap {
            layout: layout.clone(),
            displayed,
            chars,
        };
        let scroll = self.scroll.clone();
        let reveal = focused && self.shown_revision != Some(revision);
        self.shown_revision = focused.then_some(revision);
        let line = px(metrics.input_line_height);
        let width = px(metrics.cursor_width_px);
        let content = div()
            .relative()
            .flex_none()
            .min_h(line)
            .pr(width)
            .text_size(px(metrics.input_font_size))
            .line_height(line)
            .text_color(if empty {
                colors.placeholder
            } else {
                colors.text
            })
            .when(multiline, |d| d.w_full())
            .when(!multiline, |d| d.whitespace_nowrap())
            .child(text)
            .when(focused, |d| {
                d.child(
                    canvas(
                        |_, _, _| (),
                        move |_, (), window, _| {
                            let Some(p) = layout.position_for_index(cursor) else {
                                return;
                            };
                            let h = (window.rem_size() * metrics.cursor_height_rems).min(line);
                            let caret = Bounds::new(
                                point(p.x, p.y + (layout.line_height() - h) * 0.5),
                                size(width, h),
                            );
                            if collapsed {
                                window.paint_quad(fill(caret, colors.cursor));
                            }
                            // A selected hard return has no glyph, but is still a real selected character.
                            for byte in breaks {
                                if let Some(p) = layout.position_for_index(byte) {
                                    window.paint_quad(fill(
                                        Bounds::new(p, size(width, line)),
                                        selection_color,
                                    ));
                                }
                            }
                            if reveal {
                                let bounds = scroll.bounds();
                                let old = scroll.offset();
                                let mut new = old;
                                if multiline {
                                    new.y += (bounds.top() - caret.top()).max(px(0.))
                                        - (caret.bottom() - bounds.bottom()).max(px(0.));
                                } else {
                                    new.x += (bounds.left() - caret.left()).max(px(0.))
                                        - (caret.right() - bounds.right()).max(px(0.));
                                }
                                if new != old {
                                    scroll.set_offset(new);
                                    window.refresh();
                                }
                            }
                        },
                    )
                    .absolute()
                    .inset_0(),
                )
            });
        (content, hit)
    }
}

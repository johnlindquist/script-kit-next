//! The line above the input, drawn from the reducer's [`SlotLine`].
//!
//! It is one height in every main-window state and never hidden: its words swap in place and
//! only its strength eases.

use std::cell::Cell;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gpui::{div, prelude::*, px, rgb, rgba, svg, Animation, AnimationExt as _, AnyElement};
use sk_one_window::{SlotKind, SlotLine, SlotStyle};

use crate::designs::icon_variations::{icon_name_from_str, IconName};
use crate::designs::MainMenuThemeDef;

pub(crate) const SLOT_LINE_ID: &str = "main-view-slot-line";

/// A change of strength the line eases through.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SlotDim {
    pub(crate) from: f32,
    pub(crate) to: f32,
    generation: u64,
}

/// Remembers the strength last drawn, so the next change eases from it.
#[derive(Debug, Default)]
#[allow(dead_code, reason = "the binary's ScriptListApp owns the reducer")]
pub(crate) struct SlotDimmer(Cell<Option<SlotDim>>);

#[allow(dead_code, reason = "the binary's ScriptListApp owns the reducer")]
impl SlotDimmer {
    pub(crate) fn toward(&self, opacity: f32) -> SlotDim {
        let next = match self.0.get() {
            None => SlotDim {
                from: opacity,
                to: opacity,
                generation: 0,
            },
            Some(dim) if dim.to == opacity => dim,
            Some(dim) => SlotDim {
                from: dim.to,
                to: opacity,
                generation: dim.generation + 1,
            },
        };
        self.0.set(Some(next));
        next
    }
}

/// What the host hands every surface that draws the main window's header.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SlotLineView {
    pub(crate) line: SlotLine,
    pub(crate) style: SlotStyle,
    pub(crate) dim: SlotDim,
}

impl SlotLineView {
    #[allow(dead_code, reason = "the binary's ScriptListApp owns the reducer")]
    pub(crate) fn of(window: &sk_one_window::OneWindow, dimmer: &SlotDimmer) -> Self {
        let now = now_ms();
        let style = window.slot_style(now);
        Self {
            line: window.slot_line(now),
            style,
            dim: dimmer.toward(style.opacity),
        }
    }

    pub(crate) fn render(&self, theme: &crate::theme::Theme, def: MainMenuThemeDef) -> AnyElement {
        let tokens = def.slot_line;
        let colors = &theme.colors;
        let line = &self.line;
        let (tint, label_color) = match line.kind {
            SlotKind::Staged => (
                Some(rgba((colors.ui.info << 8) | tokens.staged_tint_alpha)),
                colors.ui.info,
            ),
            SlotKind::Holding => (
                Some(rgba(
                    (colors.accent.selected << 8) | tokens.holding_tint_alpha,
                )),
                colors.accent.selected,
            ),
            SlotKind::Focused | SlotKind::Recent | SlotKind::Typing => (None, colors.text.muted),
        };
        let title_color = if line.dimmed() {
            colors.text.secondary
        } else {
            colors.text.primary
        };
        let edge = def.header_info_bar.context_edge_outset_x + def.shell.header_padding_x;

        let row = div()
            .id(SLOT_LINE_ID)
            .debug_selector(|| SLOT_LINE_ID.to_string())
            .relative()
            .w_full()
            .h(px(tokens.height_px))
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(tokens.gap_px))
            .pl(px(edge + def.search.text_inset_x))
            .pr(px(edge + def.search.text_inset_x * 0.5))
            .overflow_hidden()
            .whitespace_nowrap()
            .text_color(rgb(colors.text.muted))
            .when_some(tint, |row, tint| row.bg(tint))
            .child(
                div()
                    .flex_none()
                    .text_size(px(tokens.label_font_size))
                    .text_color(rgb(label_color))
                    .child(line.label.to_uppercase()),
            )
            .when(!line.icon.is_empty(), |row| {
                row.child(
                    svg()
                        .path(slot_icon(&line.icon).asset_path())
                        .flex_none()
                        .size(px(tokens.icon_size))
                        .text_color(rgb(title_color)),
                )
            })
            .child(
                div()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(px(tokens.title_font_size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(title_color))
                    .child(line.title.clone()),
            )
            .when(!line.sub.is_empty(), |row| {
                row.child(
                    div()
                        .flex_none()
                        .text_size(px(tokens.title_font_size))
                        .child(line.sub.clone()),
                )
            })
            .child(div().flex_1())
            .when(self.style.hints, |row| {
                row.children(
                    line.hints
                        .iter()
                        .map(|(key, label)| render_hint(theme, def, key, label)),
                )
            })
            .when_some(line.progress, |row, progress| {
                row.child(
                    div()
                        .absolute()
                        .left_0()
                        .bottom_0()
                        .h(px(tokens.progress_height_px))
                        .w(gpui::relative(progress.clamp(0.0, 1.0)))
                        .bg(rgb(label_color)),
                )
            });

        let SlotDim {
            from,
            to,
            generation,
        } = self.dim;
        if from == to {
            return row.opacity(to).into_any_element();
        }
        row.with_animation(
            gpui::ElementId::NamedInteger("slot-line-dim".into(), generation),
            Animation::new(Duration::from_millis(tokens.dim_ease_ms))
                .with_easing(crate::transitions::ease_out_quad),
            move |row, t| row.opacity(from + (to - from) * t),
        )
        .into_any_element()
    }
}

/// The reducer's clock: monotonic milliseconds since the app started.
#[allow(dead_code, reason = "the binary's ScriptListApp owns the reducer")]
pub(crate) fn now_ms() -> sk_one_window::Ms {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as sk_one_window::Ms
}

fn render_hint(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    key: &str,
    label: &str,
) -> AnyElement {
    let keys = crate::components::hint_strip::shortcut_tokens_from_hint(key);
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(def.footer.metrics.content_gap))
        .text_size(px(def.slot_line.hint_font_size))
        .child(
            crate::components::footer_chrome::render_footer_row_shortcut_keycaps_from_tokens(
                keys.iter().map(String::as_str),
                theme,
                (theme.colors.text.muted << 8) | 0xFF,
            ),
        )
        .when(!label.is_empty(), |hint| hint.child(label.to_string()))
        .into_any_element()
}

/// The app's icon for a name the one-window spec uses.
fn slot_icon(name: &str) -> IconName {
    match name {
        "window" => IconName::Sidebar,
        "builtin" => IconName::BoltFilled,
        "ask" => IconName::MessageCircle,
        "again" => IconName::Refresh,
        "append" => IconName::Plus,
        "move" => IconName::ArrowRight,
        "learned" => IconName::StarFilled,
        "selection" => IconName::Pencil,
        other => icon_name_from_str(other).unwrap_or(IconName::Check),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_strength_is_drawn_without_easing() {
        let dimmer = SlotDimmer::default();
        let dim = dimmer.toward(0.8);
        assert_eq!((dim.from, dim.to), (0.8, 0.8));
        assert_eq!(
            dimmer.toward(0.8),
            dim,
            "an unchanged strength keeps its ease"
        );
    }

    #[test]
    fn a_change_eases_from_the_last_strength_under_a_new_id() {
        let dimmer = SlotDimmer::default();
        let first = dimmer.toward(0.8);
        let typed = dimmer.toward(0.5);
        assert_eq!((typed.from, typed.to), (0.8, 0.5));
        assert_ne!(typed.generation, first.generation);
        let staged = dimmer.toward(1.0);
        assert_eq!((staged.from, staged.to), (0.5, 1.0));
    }

    #[test]
    fn every_spec_icon_resolves_to_an_app_icon() {
        for name in [
            "window", "message", "builtin", "check", "ask", "again", "append",
        ] {
            assert!(!slot_icon(name).asset_path().is_empty(), "{name}");
        }
    }
}

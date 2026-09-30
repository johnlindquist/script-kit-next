//! The line above the input, drawn from the reducer's [`SlotLine`]: what's waiting to go, news,
//! or the last thing that ran. What you act on is the subject chip inside the input.
//!
//! It is one height in every main-window state and never hidden: its words swap in place and
//! only its strength eases.

use std::cell::Cell;
use std::time::Duration;

use gpui::{div, prelude::*, px, rgb, rgba, svg, Animation, AnimationExt as _, AnyElement};
use sk_one_window::{SlotKind, SlotLine, SlotStyle};

use crate::designs::icon_variations::{icon_name_from_str, IconName};
use crate::designs::MainMenuThemeDef;

pub(crate) const SLOT_LINE_ID: &str = "main-view-slot-line";

/// The subject chip inside the main input.
pub(crate) const SUBJECT_CHIP_ID: &str = "main-input-subject-chip";
/// Each `@` chip's id is this and its place among them, from 0.
pub(crate) const CONTEXT_CHIP_ID_PREFIX: &str = "main-input-context-chip-";

/// A subject icon naming the front app by bundle id: `app:com.google.Chrome`.
pub(crate) const APP_ICON_PREFIX: &str = "app:";

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
    pub(crate) fn of(
        window: &sk_one_window::OneWindow,
        dimmer: &SlotDimmer,
        now: sk_one_window::Ms,
    ) -> Self {
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
        let (tint, accent) = match line.kind {
            SlotKind::Holding => (
                Some(rgba(
                    (colors.accent.selected << 8) | tokens.holding_tint_alpha,
                )),
                colors.accent.selected,
            ),
            _ if line.lit => (
                Some(rgba((colors.ui.info << 8) | tokens.staged_tint_alpha)),
                colors.ui.info,
            ),
            _ => (None, colors.text.muted),
        };
        let title_color = if line.lit || line.kind == SlotKind::Holding {
            colors.text.primary
        } else {
            colors.text.secondary
        };
        let edge = crate::components::main_view_chrome::main_view_edge_inset_x(def)
            - def.shell.header_padding_x;

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
            .px(px(edge))
            .overflow_hidden()
            .whitespace_nowrap()
            .text_color(rgb(colors.text.muted))
            .when_some(tint, |row, tint| row.bg(tint))
            .when(!line.icon.is_empty(), |row| {
                row.child(render_slot_icon(&line.icon, tokens.icon_size, title_color))
            })
            // "Last" before the last run, "Behind" before the window the words would act on.
            .when(
                !line.label.is_empty()
                    && (line.kind == SlotKind::Recent
                        || line.label == sk_one_window::contract::words::BEHIND),
                |row| {
                    row.child(
                        div()
                            .flex_none()
                            .text_size(px(tokens.title_font_size))
                            .child(line.label.clone()),
                    )
                },
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .when(!line.sub.is_empty(), |title| title.flex_shrink_0())
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(px(tokens.title_font_size))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(title_color))
                    .child(line.title.clone()),
            )
            .when(!line.sub.is_empty(), |row| {
                row.child(
                    div()
                        .min_w(px(0.0))
                        .overflow_hidden()
                        .text_ellipsis()
                        .text_size(px(tokens.title_font_size))
                        .child(format!("· {}", line.sub)),
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
                        .bg(rgb(accent)),
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
        .when(!label.is_empty(), |hint| hint.child(label.to_string()))
        .child(
            crate::components::footer_chrome::render_footer_row_shortcut_keycaps_from_tokens(
                keys.iter().map(String::as_str),
                theme,
                (theme.colors.text.muted << 8) | 0xFF,
            ),
        )
        .into_any_element()
}

/// The subject as a chip in the input: solid in the info color at its start once attached (Esc
/// lets go, so it carries no key); dashed at its end while only offered, with the key that
/// attaches it. `highlighted` when ← stands on it, so ⌫ deletes it.
#[allow(dead_code)] // The binary's main render draws it; the library only builds the line.
pub(crate) fn render_subject_chip(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    chip: &sk_one_window::SubjectChip,
    highlighted: bool,
) -> AnyElement {
    if chip.attached {
        return render_attached_chip(
            theme,
            def,
            gpui::ElementId::Name(SUBJECT_CHIP_ID.into()),
            SUBJECT_CHIP_ID.to_string(),
            &chip.icon,
            &chip.title,
            highlighted,
        );
    }
    let tokens = def.slot_line.chip;
    let colors = &theme.colors;
    div()
        .id(SUBJECT_CHIP_ID)
        .debug_selector(|| SUBJECT_CHIP_ID.to_string())
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(tokens.gap_px))
        .h(px(tokens.height_px))
        .relative()
        .top(px(tokens.input_drop_px))
        .pl(px(tokens.pad_left_px))
        .pr(px(tokens.pad_right_px))
        .rounded(px(tokens.radius_px))
        .border_1()
        .whitespace_nowrap()
        .text_size(px(tokens.font_size))
        .text_color(rgba((colors.text.primary << 8) | tokens.offered_text_alpha))
        .ml(px(tokens.offered_margin_px))
        .mr(px(def.search.text_inset_x))
        .border_dashed()
        .border_color(rgba((colors.text.primary << 8) | tokens.offered_edge_alpha))
        .font_weight(gpui::FontWeight(tokens.offered_weight))
        .child(render_slot_icon(
            &chip.icon,
            tokens.icon_size,
            colors.text.muted,
        ))
        .child(chip_title(tokens.max_title_px, &chip.title))
        .child(
            div()
                .text_color(rgb(colors.text.muted))
                .font_weight(gpui::FontWeight::NORMAL)
                .child(render_hint(theme, def, &chip.key, &chip.word)),
        )
        .into_any_element()
}

/// What `@` attached, as a chip after the subject's at the input's start, drawn like the
/// attached subject. `highlighted` when ← stands on it, so ⌫ deletes it.
#[allow(dead_code)] // The binary's main render draws it; the library only builds the line.
pub(crate) fn render_context_chip(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    index: usize,
    chip: &sk_one_window::ContextChip,
    highlighted: bool,
) -> AnyElement {
    let id = format!("{CONTEXT_CHIP_ID_PREFIX}{index}");
    render_attached_chip(
        theme,
        def,
        gpui::ElementId::Name(id.clone().into()),
        id,
        &chip.icon,
        &chip.title,
        highlighted,
    )
}

fn render_attached_chip(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    id: gpui::ElementId,
    selector: String,
    icon: &str,
    title: &str,
    highlighted: bool,
) -> AnyElement {
    let tokens = def.slot_line.chip;
    let colors = &theme.colors;
    let info = colors.ui.info;
    // Highlighted, the chip is filled like a selected token and its icon takes the text's color.
    let (fill, edge, icon_color) = if highlighted {
        (
            tokens.highlight_fill_alpha,
            tokens.highlight_edge_alpha,
            colors.text.primary,
        )
    } else {
        (tokens.fill_alpha, tokens.edge_alpha, info)
    };
    div()
        .id(id)
        .debug_selector(move || selector.clone())
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(tokens.gap_px))
        .h(px(tokens.height_px))
        .relative()
        .top(px(tokens.input_drop_px))
        .px(px(tokens.pad_left_px))
        .rounded(px(tokens.radius_px))
        .border_1()
        .whitespace_nowrap()
        .text_size(px(tokens.font_size))
        .text_color(rgba((colors.text.primary << 8) | 0xFF))
        .bg(rgba((info << 8) | fill))
        .border_color(rgba((info << 8) | edge))
        .font_weight(gpui::FontWeight(tokens.attached_weight))
        .child(render_slot_icon(icon, tokens.icon_size, icon_color))
        .child(chip_title(tokens.max_title_px, title))
        .into_any_element()
}

fn chip_title(max_px: f32, title: &str) -> gpui::Div {
    div()
        .min_w(px(0.0))
        .max_w(px(max_px))
        .overflow_hidden()
        .text_ellipsis()
        .child(title.to_string())
}

/// The subject's icon: the front app's own icon for `app:<bundle id>`, else the app's icon for a
/// name the one-window spec uses.
fn render_slot_icon(name: &str, size: f32, color: u32) -> AnyElement {
    if let Some(bundle) = name.strip_prefix(APP_ICON_PREFIX) {
        let drawn = size + 2.0;
        if let Some(image) = scaled_app_icon(bundle, (drawn * 2.0).ceil() as u32) {
            return gpui::img(move |_: &mut gpui::Window, _: &mut gpui::App| {
                Some(Ok(image.clone()))
            })
            .flex_none()
            .size(px(drawn))
            .into_any_element();
        }
        return svg()
            .path(IconName::Sidebar.asset_path())
            .flex_none()
            .size(px(size))
            .text_color(rgb(color))
            .into_any_element();
    }
    svg()
        .path(slot_icon(name))
        .flex_none()
        .size(px(size))
        .text_color(rgb(color))
        .into_any_element()
}

/// The app's icon at `side` pixels, cached by content. GPUI samples images without mipmaps, so a
/// 256–1024px icon drawn at ~30px skips most of its pixels and thin bright edges flicker in.
fn scaled_app_icon(bundle: &str, side: u32) -> Option<std::sync::Arc<gpui::RenderImage>> {
    type Scaled = std::collections::HashMap<([u8; 32], u32), std::sync::Arc<gpui::RenderImage>>;
    static SCALED: std::sync::LazyLock<std::sync::Mutex<Scaled>> =
        std::sync::LazyLock::new(Default::default);

    let icon = crate::app_launcher::cached_app_icon_for_bundle(bundle)?;
    let key = (*icon.content_digest(), side);
    if let Some(hit) = SCALED.lock().ok()?.get(&key) {
        return Some(hit.clone());
    }
    let image = icon.image();
    let size = image.size(0);
    let (width, height) = (size.width.0.max(0) as u32, size.height.0.max(0) as u32);
    let scaled = if width <= side && height <= side {
        image.clone()
    } else {
        std::sync::Arc::new(shrink_icon(image.as_bytes(0)?, width, height, side)?)
    };
    let mut cache = SCALED.lock().ok()?;
    if cache.len() >= 64 {
        cache.clear();
    }
    cache.insert(key, scaled.clone());
    Some(scaled)
}

/// Fits 8-bit straight-alpha pixels inside `side`×`side`, resampled with alpha premultiplied so
/// transparent pixels don't darken the edges.
fn shrink_icon(bytes: &[u8], width: u32, height: u32, side: u32) -> Option<gpui::RenderImage> {
    let mut source = image::RgbaImage::from_raw(width, height, bytes.to_vec())?;
    for pixel in source.pixels_mut() {
        let alpha = u16::from(pixel[3]);
        for channel in &mut pixel.0[..3] {
            *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
    let scale = side as f32 / width.max(height).max(1) as f32;
    let (out_w, out_h) = (
        ((width as f32 * scale).round() as u32).max(1),
        ((height as f32 * scale).round() as u32).max(1),
    );
    let mut shrunk =
        image::imageops::resize(&source, out_w, out_h, image::imageops::FilterType::Lanczos3);
    for pixel in shrunk.pixels_mut() {
        let alpha = u16::from(pixel[3]);
        if alpha == 0 {
            pixel.0 = [0; 4];
            continue;
        }
        for channel in &mut pixel.0[..3] {
            *channel = ((u16::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
        }
    }
    Some(gpui::RenderImage::new(smallvec::SmallVec::from_elem(
        image::Frame::new(shrunk),
        1,
    )))
}

/// The asset for a name the one-window spec uses: the app's own icon set first, then Lucide
/// ("history" for the Last line), then a check.
fn slot_icon(name: &str) -> gpui::SharedString {
    use gpui_component::IconNamed;
    let own = match name {
        "window" => Some(IconName::Sidebar),
        "builtin" => Some(IconName::BoltFilled),
        "ask" => Some(IconName::MessageCircle),
        "again" => Some(IconName::Refresh),
        "append" => Some(IconName::Plus),
        "move" => Some(IconName::ArrowRight),
        "learned" => Some(IconName::StarFilled),
        "selection" | "type" => Some(IconName::Pencil),
        other => icon_name_from_str(other),
    };
    match own {
        Some(icon) => icon.asset_path().into(),
        None => crate::icons::lucide_from_str(name)
            .map(IconNamed::path)
            .unwrap_or_else(|| IconName::Check.asset_path().into()),
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

    /// A big icon drawn small averages its pixels instead of sampling a few: thin stripes blend
    /// to gray, and transparent corners stay transparent with no dark fringe.
    #[test]
    fn a_shrunk_icon_averages_its_pixels_and_keeps_transparency() {
        let (width, height) = (256u32, 256u32);
        let mut bytes = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let inside = (32..224).contains(&x) && (32..224).contains(&y);
                let value = if x % 2 == 0 { 255 } else { 0 };
                bytes.extend_from_slice(&if inside {
                    [value, value, value, 255]
                } else {
                    [255, 255, 255, 0]
                });
            }
        }
        let image = shrink_icon(&bytes, width, height, 32).expect("shrinks");
        let size = image.size(0);
        assert_eq!((size.width.0, size.height.0), (32, 32));
        let out = image.as_bytes(0).expect("frame");
        let at = |x: usize, y: usize| &out[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4];
        assert_eq!(at(0, 0)[3], 0, "corners stay transparent");
        let center = at(16, 16);
        assert_eq!(center[3], 255);
        assert!(
            (100..=155).contains(&center[0]),
            "stripes blend to gray, got {center:?}"
        );
    }

    #[test]
    fn every_spec_icon_resolves_to_an_app_icon() {
        for name in [
            "window", "message", "builtin", "check", "ask", "again", "append", "type",
        ] {
            assert!(!slot_icon(name).is_empty(), "{name}");
        }
        assert_eq!(
            slot_icon(sk_one_window::contract::RECENT_ICON).as_ref(),
            "icons/history.svg",
            "the Last line draws the history glyph, not the fallback check"
        );
    }
}

#![allow(dead_code)]

use gpui::{
    div, prelude::*, px, rgba, svg, AnyElement, App, FontWeight, IntoElement, RenderOnce,
    SharedString, Styled, Window,
};

use crate::ui::chrome::{
    alpha_from_opacity, HINT_STRIP_HEIGHT, HINT_STRIP_PADDING_X, HINT_STRIP_PADDING_Y,
    HINT_TEXT_OPACITY,
};
use crate::ui_foundation::HexColorExt;

const HINT_STRIP_CONTENT_GAP: f32 = 8.0;

/// Size for keyboard glyph icons in the hint strip.
/// Slightly larger than text_xs (12px) for visual clarity at hint opacity.
const KEY_ICON_SIZE: f32 = 14.0;

/// Gap between a key icon and its label text within a single hint.
const KEY_ICON_LABEL_GAP: f32 = 3.0;

/// External (filesystem) paths for keyboard glyph SVGs.
/// GPUI requires `svg().external_path()` for file-based SVGs; `.path()` is for embedded assets.
const RETURN_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/return.svg");
const TAB_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/tab.svg");
const COMMAND_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/command.svg");
const SHIFT_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/shift.svg");
const ESCAPE_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/escape.svg");

const KEYCAP_PADDING_X: f32 = 6.0;
const KEYCAP_PADDING_Y: f32 = 1.0;
const KEYCAP_RADIUS: f32 = 5.0;
const KEYCAP_BG_OPACITY: f32 = 0.12;

#[derive(IntoElement)]
pub struct HintStrip {
    hints: Vec<SharedString>,
    leading: Option<AnyElement>,
}

impl HintStrip {
    pub fn new(hints: impl IntoHints) -> Self {
        Self {
            hints: hints.into_hints(),
            leading: None,
        }
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }
}

pub trait IntoHints {
    fn into_hints(self) -> Vec<SharedString>;
}

impl IntoHints for Vec<SharedString> {
    fn into_hints(self) -> Vec<SharedString> {
        self
    }
}

impl IntoHints for SharedString {
    fn into_hints(self) -> Vec<SharedString> {
        vec![self]
    }
}

impl IntoHints for &str {
    fn into_hints(self) -> Vec<SharedString> {
        vec![self.to_string().into()]
    }
}

impl IntoHints for String {
    fn into_hints(self) -> Vec<SharedString> {
        vec![self.into()]
    }
}

fn text_color_with_opacity(primary: u32, opacity: f32) -> u32 {
    // Theme text colors are stored as 0xAARRGGBB; strip the original alpha, shift RGB into
    // RRGGBB00, then inject the requested alpha byte for gpui::rgba.
    ((primary & 0x00FF_FFFF) << 8) | alpha_from_opacity(opacity)
}

/// A parsed hint: either an icon+label pair or plain text.
enum HintElement {
    /// One or more keyboard glyph icons or text keycaps followed by a text label.
    KeyHint {
        parts: Vec<KeyHintPart>,
        label: SharedString,
    },
    /// Plain text (no icon).
    Text(SharedString),
}

enum KeyHintPart {
    Icon(&'static str),
    Keycap(SharedString),
}

fn is_boundary_or_end(rest: &str) -> bool {
    rest.is_empty() || rest.chars().next().is_some_and(char::is_whitespace)
}

/// Parse a hint string and extract a leading keyboard glyph if present.
///
/// Recognized patterns (all map to SVG icons):
/// - `"↵ Run"`, `"⏎ Send"`, `"↩ Send"` → Return icon + label
/// - `"⌘K Actions"`, `"⌘⇧↵ Send"` → icon sequence + rest
/// - `"Tab AI"` → Tab icon + label
/// - `"Esc Back"` → Esc text keycap + rest
fn parse_hint(hint: &str) -> HintElement {
    let mut rest = hint;
    let mut parts = Vec::new();

    loop {
        if let Some(next) = rest.strip_prefix('⌘') {
            parts.push(KeyHintPart::Icon(COMMAND_ICON_PATH));
            rest = next;
            continue;
        }

        if let Some(next) = rest.strip_prefix('⇧') {
            parts.push(KeyHintPart::Icon(SHIFT_ICON_PATH));
            rest = next;
            continue;
        }

        if let Some(next) = rest.strip_prefix("Tab") {
            if is_boundary_or_end(next) {
                parts.push(KeyHintPart::Icon(TAB_ICON_PATH));
                rest = next;
                continue;
            }
        }

        if let Some(next) = rest.strip_prefix("Esc") {
            if is_boundary_or_end(next) {
                parts.push(KeyHintPart::Keycap("Esc".into()));
                rest = next;
                continue;
            }
        }

        if let Some(next) = rest.strip_prefix('↵') {
            parts.push(KeyHintPart::Icon(RETURN_ICON_PATH));
            rest = next;
            continue;
        }

        if let Some(next) = rest.strip_prefix('\u{23CE}') {
            parts.push(KeyHintPart::Icon(RETURN_ICON_PATH));
            rest = next;
            continue;
        }

        if let Some(next) = rest.strip_prefix('\u{21A9}') {
            parts.push(KeyHintPart::Icon(RETURN_ICON_PATH));
            rest = next;
            continue;
        }

        break;
    }

    if parts.is_empty() {
        return HintElement::Text(hint.to_string().into());
    }

    HintElement::KeyHint {
        parts,
        label: rest.trim_start().to_string().into(),
    }
}

/// Render a single hint element (icon+label or plain text) with a pre-computed RGBA color.
fn render_hint_element(element: HintElement, text_rgba: u32) -> AnyElement {
    render_hint_element_hsla(element, rgba(text_rgba).into())
}

/// Render a single hint element with an HSLA color.
fn render_hint_element_hsla(element: HintElement, color: gpui::Hsla) -> AnyElement {
    match element {
        HintElement::KeyHint { parts, label } => {
            let theme = crate::theme::get_cached_theme();
            let keycap_bg = theme.colors.text.primary.with_opacity(KEYCAP_BG_OPACITY);

            let mut keys_row = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(KEY_ICON_LABEL_GAP));

            for part in parts {
                keys_row = keys_row.child(match part {
                    KeyHintPart::Icon(icon_path) => svg()
                        .external_path(icon_path)
                        .size(px(KEY_ICON_SIZE))
                        .flex_shrink_0()
                        .text_color(color)
                        .into_any_element(),
                    KeyHintPart::Keycap(text) => div()
                        .px(px(KEYCAP_PADDING_X))
                        .py(px(KEYCAP_PADDING_Y))
                        .rounded(px(KEYCAP_RADIUS))
                        .bg(keycap_bg)
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(color)
                        .child(text)
                        .into_any_element(),
                });
            }

            let mut hint_row = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(KEY_ICON_LABEL_GAP))
                .child(keys_row);

            if !label.is_empty() {
                hint_row = hint_row.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(color)
                        .child(label),
                );
            }

            hint_row.into_any_element()
        }
        HintElement::Text(text) => div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(color)
            .child(text)
            .into_any_element(),
    }
}

impl RenderOnce for HintStrip {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let text_rgba = text_color_with_opacity(theme.colors.text.primary, HINT_TEXT_OPACITY);

        let mut row = div()
            .w_full()
            .h(px(HINT_STRIP_HEIGHT))
            .px(px(HINT_STRIP_PADDING_X))
            .py(px(HINT_STRIP_PADDING_Y))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(HINT_STRIP_CONTENT_GAP));

        if let Some(leading) = self.leading {
            row = row.child(leading);
        }

        // Build the right-aligned hints container with icon-aware rendering.
        let mut hints_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(HINT_STRIP_CONTENT_GAP));

        for hint in &self.hints {
            let element = parse_hint(hint.as_ref());
            hints_row = hints_row.child(render_hint_element(element, text_rgba));
        }

        row.child(div().flex_1()).child(hints_row)
    }
}

/// Render a list of hint strings as icon-aware elements in a flex row.
///
/// This is the shared entry point for any footer that needs keyboard glyph icons.
/// Callers supply the hints (e.g. `["↵ Run", "⌘K Actions", "Tab AI"]`) and the
/// pre-computed RGBA text color. Returns a right-aligned flex row `AnyElement`.
///
/// Use this instead of rendering hint strings as plain text — it replaces Unicode
/// keyboard glyphs (↵, ⌘, ⏎, ↩, Tab) with pixel-precise SVG icons.
pub fn render_hint_icons(hints: &[&str], text_rgba: u32) -> AnyElement {
    render_hint_icons_hsla(hints, rgba(text_rgba).into())
}

/// Like [`render_hint_icons`] but accepts an HSLA color directly.
///
/// Use this when the caller already has an `Hsla` (e.g. from `cx.theme()`).
pub fn render_hint_icons_hsla(hints: &[&str], color: gpui::Hsla) -> AnyElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(HINT_STRIP_CONTENT_GAP));

    for hint in hints {
        let element = parse_hint(hint);
        row = row.child(render_hint_element_hsla(element, color));
    }

    row.into_any_element()
}

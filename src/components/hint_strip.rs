#![allow(dead_code)]

use std::{
    collections::HashSet,
    rc::Rc,
    sync::{Mutex, OnceLock},
};

use gpui::{
    div, prelude::*, px, rgba, svg, AnyElement, App, ClickEvent, FontWeight, IntoElement,
    RenderOnce, SharedString, Styled, Window,
};

use crate::ui::chrome::{
    alpha_from_opacity, HINT_STRIP_HEIGHT, HINT_STRIP_PADDING_X, HINT_STRIP_PADDING_Y,
    HINT_TEXT_OPACITY,
};
use crate::ui_foundation::HexColorExt;

const HINT_STRIP_CONTENT_GAP: f32 = 8.0;

/// Padding inside each clickable hint button.
const HINT_BUTTON_PADDING_X: f32 = 4.0;
const HINT_BUTTON_PADDING_Y: f32 = 2.0;

/// Corner radius for hint button hover highlight.
const HINT_BUTTON_RADIUS: f32 = 4.0;

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

/// A click handler for a single hint button.
pub(crate) type HintClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct HintStrip {
    hints: Vec<SharedString>,
    leading: Option<AnyElement>,
    /// Optional per-hint click handlers. When set, hints become clickable buttons
    /// with ghost-bg hover feedback using the theme's hover token.
    on_clicks: Vec<Option<HintClickHandler>>,
}

impl HintStrip {
    pub fn new(hints: impl IntoHints) -> Self {
        let hints = hints.into_hints();
        let len = hints.len();
        Self {
            hints,
            leading: None,
            on_clicks: vec![None; len],
        }
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    /// Attach a click handler to the hint at `index`.
    /// When set, the hint renders as a clickable button with ghost-bg hover.
    pub fn on_hint_click(
        mut self,
        index: usize,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        if index < self.on_clicks.len() {
            self.on_clicks[index] = Some(Rc::new(handler));
        }
        self
    }

    /// Attach click handlers to all hints at once.
    /// Each entry maps to the hint at the same index. `None` entries remain non-interactive.
    pub fn on_hint_clicks(
        mut self,
        handlers: Vec<Option<impl Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    ) -> Self {
        for (i, handler) in handlers.into_iter().enumerate() {
            if i < self.on_clicks.len() {
                self.on_clicks[i] = handler.map(|h| Rc::new(h) as HintClickHandler);
            }
        }
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

// ─── Shared compact shortcut renderer ────────────────────────────────

const INLINE_SHORTCUT_GAP: f32 = 3.0;
const INLINE_SHORTCUT_ICON_SIZE: f32 = 12.0;
const INLINE_SHORTCUT_TEXT_SIZE: f32 = 11.0;
const INLINE_SHORTCUT_KEYCAP_PADDING_X: f32 = 4.0;
const INLINE_SHORTCUT_KEYCAP_PADDING_Y: f32 = 1.0;
const INLINE_SHORTCUT_KEYCAP_RADIUS: f32 = 4.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct InlineShortcutColors {
    pub glyph: gpui::Hsla,
    pub keycap_bg: gpui::Hsla,
    pub keycap_border: Option<gpui::Hsla>,
}

/// Shared whisper-chrome preset for compact inline shortcuts.
///
/// Produces ultra-low-opacity keycap backgrounds (0.08) with faint borders (0.18),
/// matching the whisper-chrome design language used in the footer hint strip.
/// All primary surfaces should use this instead of per-surface keycap opacity tuning.
#[inline]
pub(crate) fn whisper_inline_shortcut_colors(
    glyph: gpui::Hsla,
    chrome: gpui::Hsla,
    show_border: bool,
) -> InlineShortcutColors {
    let mut bg = chrome;
    bg.a = 0.08;
    let border = if show_border {
        let mut b = chrome;
        b.a = 0.18;
        Some(b)
    } else {
        None
    };
    InlineShortcutColors {
        glyph,
        keycap_bg: bg,
        keycap_border: border,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ShortcutChromeAudit {
    surface: &'static str,
    mode: &'static str,
}

fn seen_shortcut_chrome_audits() -> &'static Mutex<HashSet<ShortcutChromeAudit>> {
    static SEEN: OnceLock<Mutex<HashSet<ShortcutChromeAudit>>> = OnceLock::new();
    SEEN.get_or_init(|| Mutex::new(HashSet::new()))
}

pub(crate) fn emit_shortcut_chrome_audit(surface: &'static str, mode: &'static str) {
    let audit = ShortcutChromeAudit { surface, mode };
    let mut seen = seen_shortcut_chrome_audits()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if seen.insert(audit.clone()) {
        tracing::info!(surface = surface, mode = mode, "shortcut_chrome_audit");
    }
}

enum InlineShortcutToken {
    Icon(&'static str),
    Text(SharedString),
    Keycap(SharedString),
}

fn is_symbol_shortcut_char(ch: char) -> bool {
    matches!(
        ch,
        '⌘' | '⌃'
            | '⌥'
            | '⇧'
            | '↵'
            | '↩'
            | '⏎'
            | '⎋'
            | '⇥'
            | '⌫'
            | '␣'
            | '↑'
            | '↓'
            | '←'
            | '→'
            | '⇞'
            | '⇟'
            | '↖'
            | '↘'
    )
}

fn normalize_shortcut_part(part: &str) -> String {
    match part.to_lowercase().as_str() {
        "cmd" | "command" | "meta" | "super" => "⌘".to_string(),
        "ctrl" | "control" => "⌃".to_string(),
        "alt" | "option" | "opt" => "⌥".to_string(),
        "shift" => "⇧".to_string(),
        "enter" | "return" => "↵".to_string(),
        "escape" | "esc" => "⎋".to_string(),
        "tab" => "⇥".to_string(),
        "space" => "␣".to_string(),
        "backspace" | "delete" => "⌫".to_string(),
        "up" | "arrowup" => "↑".to_string(),
        "down" | "arrowdown" => "↓".to_string(),
        "left" | "arrowleft" => "←".to_string(),
        "right" | "arrowright" => "→".to_string(),
        "pageup" => "⇞".to_string(),
        "pagedown" => "⇟".to_string(),
        "home" => "↖".to_string(),
        "end" => "↘".to_string(),
        other if other.chars().all(is_symbol_shortcut_char) => other.to_string(),
        other => other.to_uppercase(),
    }
}

pub(crate) fn shortcut_tokens_from_hint(shortcut: &str) -> Vec<String> {
    let trimmed = shortcut.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    // "cmd+shift+k" style
    if trimmed.contains('+') || trimmed.chars().any(char::is_whitespace) {
        return trimmed
            .replace('+', " ")
            .split_whitespace()
            .map(normalize_shortcut_part)
            .collect();
    }

    // "⌃⌘↑" style — each char is a token
    if trimmed.chars().any(is_symbol_shortcut_char) {
        return trimmed
            .chars()
            .map(|ch| {
                if is_symbol_shortcut_char(ch) {
                    ch.to_string()
                } else {
                    ch.to_uppercase().to_string()
                }
            })
            .collect();
    }

    vec![normalize_shortcut_part(trimmed)]
}

fn inline_shortcut_token(token: &str) -> InlineShortcutToken {
    match token {
        "⌘" => InlineShortcutToken::Icon(COMMAND_ICON_PATH),
        "⇧" => InlineShortcutToken::Icon(SHIFT_ICON_PATH),
        "↵" | "↩" | "⏎" => InlineShortcutToken::Icon(RETURN_ICON_PATH),
        "⇥" => InlineShortcutToken::Icon(TAB_ICON_PATH),
        value if value.chars().count() > 1 => InlineShortcutToken::Keycap(value.to_string().into()),
        value => InlineShortcutToken::Text(value.to_uppercase().into()),
    }
}

pub(crate) fn render_inline_shortcut_keys<'a>(
    keys: impl IntoIterator<Item = &'a str>,
    colors: InlineShortcutColors,
) -> AnyElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(INLINE_SHORTCUT_GAP));
    let mut has_keys = false;

    for key in keys {
        has_keys = true;
        row = row.child(match inline_shortcut_token(key) {
            InlineShortcutToken::Icon(icon_path) => svg()
                .external_path(icon_path)
                .size(px(INLINE_SHORTCUT_ICON_SIZE))
                .flex_shrink_0()
                .text_color(colors.glyph)
                .into_any_element(),
            InlineShortcutToken::Text(text) => div()
                .text_size(px(INLINE_SHORTCUT_TEXT_SIZE))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors.glyph)
                .child(text)
                .into_any_element(),
            InlineShortcutToken::Keycap(text) => {
                let mut chip = div()
                    .px(px(INLINE_SHORTCUT_KEYCAP_PADDING_X))
                    .py(px(INLINE_SHORTCUT_KEYCAP_PADDING_Y))
                    .rounded(px(INLINE_SHORTCUT_KEYCAP_RADIUS))
                    .bg(colors.keycap_bg)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.glyph)
                    .child(text);
                if let Some(border) = colors.keycap_border {
                    chip = chip.border_1().border_color(border);
                }
                chip.into_any_element()
            }
        });
    }

    if has_keys {
        row.into_any_element()
    } else {
        div().into_any_element()
    }
}

#[cfg(test)]
mod inline_shortcut_tests {
    use super::shortcut_tokens_from_hint;

    #[test]
    fn shortcut_tokens_handle_raw_and_symbol_inputs() {
        assert_eq!(
            shortcut_tokens_from_hint("cmd+shift+k"),
            vec!["⌘", "⇧", "K"]
        );
        assert_eq!(shortcut_tokens_from_hint("⌃⌘↑"), vec!["⌃", "⌘", "↑"]);
        assert_eq!(shortcut_tokens_from_hint("cmd+pageup"), vec!["⌘", "⇞"]);
        assert_eq!(shortcut_tokens_from_hint("cmd+home"), vec!["⌘", "↖"]);
    }
}

// ─── End shared compact shortcut renderer ────────────────────────────

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
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let text_rgba = text_color_with_opacity(theme.colors.text.primary, HINT_TEXT_OPACITY);
        let hover_bg = rgba(chrome.hover_rgba);
        let active_bg = rgba(chrome.selection_rgba);

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

        for (i, (hint, on_click)) in self
            .hints
            .iter()
            .zip(self.on_clicks.into_iter())
            .enumerate()
        {
            let element = parse_hint(hint.as_ref());
            let hint_content = render_hint_element(element, text_rgba);

            if let Some(handler) = on_click {
                // Clickable hint button with ghost-bg hover from theme tokens.
                let button = div()
                    .id(SharedString::from(format!("hint-btn-{i}")))
                    .cursor_pointer()
                    .px(px(HINT_BUTTON_PADDING_X))
                    .py(px(HINT_BUTTON_PADDING_Y))
                    .rounded(px(HINT_BUTTON_RADIUS))
                    .hover(move |s| s.bg(hover_bg))
                    .active(move |s| s.bg(active_bg))
                    .on_click(move |event, window, cx| handler(event, window, cx))
                    .child(hint_content);
                hints_row = hints_row.child(button);
            } else {
                hints_row = hints_row.child(hint_content);
            }
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
/// Hints are rendered as clickable buttons with ghost-bg hover.
pub fn render_hint_icons(hints: &[&str], text_rgba: u32) -> AnyElement {
    render_hint_icons_hsla(hints, rgba(text_rgba).into())
}

/// Like [`render_hint_icons`] but accepts an HSLA color directly.
///
/// Use this when the caller already has an `Hsla` (e.g. from `cx.theme()`).
/// Hints are rendered as clickable buttons with ghost-bg hover.
pub fn render_hint_icons_hsla(hints: &[&str], color: gpui::Hsla) -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let chrome = crate::theme::AppChromeColors::from_theme(&theme);
    let hover_bg = rgba(chrome.hover_rgba);
    let active_bg = rgba(chrome.selection_rgba);

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(HINT_STRIP_CONTENT_GAP));

    for (i, hint) in hints.iter().enumerate() {
        let element = parse_hint(hint);
        let hint_content = render_hint_element_hsla(element, color);

        let button = div()
            .id(SharedString::from(format!("hint-icon-{i}")))
            .cursor_pointer()
            .px(px(HINT_BUTTON_PADDING_X))
            .py(px(HINT_BUTTON_PADDING_Y))
            .rounded(px(HINT_BUTTON_RADIUS))
            .hover(move |s| s.bg(hover_bg))
            .active(move |s| s.bg(active_bg))
            .child(hint_content);
        row = row.child(button);
    }

    row.into_any_element()
}

/// A hint label paired with an optional click handler for [`render_hint_icons_clickable`].
pub struct ClickableHint {
    pub label: &'static str,
    pub on_click: Option<HintClickHandler>,
}

impl ClickableHint {
    pub fn new(
        label: &'static str,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label,
            on_click: Some(Rc::new(on_click)),
        }
    }
}

/// Render clickable hint icons with per-hint click handlers.
///
/// Each [`ClickableHint`] renders as a clickable button with ghost-bg hover.
pub fn render_hint_icons_clickable(hints: Vec<ClickableHint>, text_rgba: u32) -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let chrome = crate::theme::AppChromeColors::from_theme(&theme);
    let hover_bg = rgba(chrome.hover_rgba);
    let active_bg = rgba(chrome.selection_rgba);
    let color: gpui::Hsla = rgba(text_rgba).into();

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(HINT_STRIP_CONTENT_GAP));

    for (i, hint) in hints.into_iter().enumerate() {
        let element = parse_hint(hint.label);
        let hint_content = render_hint_element_hsla(element, color);

        let mut button = div()
            .id(SharedString::from(format!("hint-click-{i}")))
            .cursor_pointer()
            .px(px(HINT_BUTTON_PADDING_X))
            .py(px(HINT_BUTTON_PADDING_Y))
            .rounded(px(HINT_BUTTON_RADIUS))
            .hover(move |s| s.bg(hover_bg))
            .active(move |s| s.bg(active_bg))
            .child(hint_content);

        if let Some(handler) = hint.on_click {
            button = button.on_click(move |event, window, cx| handler(event, window, cx));
        }

        row = row.child(button);
    }

    row.into_any_element()
}

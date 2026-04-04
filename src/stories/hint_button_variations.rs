//! Hint Button Variations — 9 clickable footer hint-strip treatments.
//!
//! Each variation renders the canonical three-key footer ("↵ Run", "⌘K Actions", "Tab AI")
//! as clickable buttons with different hover/active feedback styles. The base rendering
//! (SVG icons + text_xs + semibold + hint opacity) stays identical across all variants —
//! only the interactive feedback layer varies.

use gpui::*;

use crate::storybook::{story_container, story_section, Story, StorySurface, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

// ── SVG icon paths (same as hint_strip.rs) ──────────────────────────────────

const RETURN_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/return.svg");
const TAB_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/tab.svg");
const COMMAND_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/command.svg");

// ── Layout constants ────────────────────────────────────────────────────────

const KEY_ICON_SIZE: f32 = 14.0;
const KEY_ICON_LABEL_GAP: f32 = 3.0;
const HINT_STRIP_CONTENT_GAP: f32 = 8.0;
const KEYCAP_BG_OPACITY: f32 = 0.12;
const KEYCAP_PADDING_X: f32 = 6.0;
const KEYCAP_PADDING_Y: f32 = 1.0;
const KEYCAP_RADIUS: f32 = 5.0;

// ── Story ───────────────────────────────────────────────────────────────────

pub struct HintButtonVariationsStory;

impl Story for HintButtonVariationsStory {
    fn id(&self) -> &'static str {
        "hint-button-variations"
    }

    fn name(&self) -> &'static str {
        "Hint Button Variations (9)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Footer
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let c = Colors::from_theme(&theme);

        story_container()
            .child(
                story_section("Clickable Footer Hint Buttons — hover each to compare")
                    .child(variation_row(
                        "1. Current + cursor only",
                        mock_window(c, render_variation(c, 1)),
                    ))
                    .child(variation_row(
                        "2. Ghost bg hover",
                        mock_window(c, render_variation(c, 2)),
                    ))
                    .child(variation_row(
                        "3. Brighter text hover",
                        mock_window(c, render_variation(c, 3)),
                    ))
                    .child(variation_row(
                        "4. Ghost bg + brighter text",
                        mock_window(c, render_variation(c, 4)),
                    ))
                    .child(variation_row(
                        "5. Underline hover",
                        mock_window(c, render_variation(c, 5)),
                    ))
                    .child(variation_row(
                        "6. Keycap badge hover",
                        mock_window(c, render_variation(c, 6)),
                    ))
                    .child(variation_row(
                        "7. Opacity press feedback",
                        mock_window(c, render_variation(c, 7)),
                    ))
                    .child(variation_row(
                        "8. Pill bg hover",
                        mock_window(c, render_variation(c, 8)),
                    ))
                    .child(variation_row(
                        "9. Border hover",
                        mock_window(c, render_variation(c, 9)),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        // No variants — render() shows all 9 in a single scrollable view.
        // This avoids the single-variant-preview path and shows the full overview.
        vec![]
    }
}

// ── Colors ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Colors {
    background: u32,
    text_primary: u32,
    text_muted: u32,
    accent: u32,
    border: u32,
}

impl Colors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            text_primary: theme.colors.text.primary,
            text_muted: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
        }
    }

    fn hint_opacity_color(self) -> Hsla {
        rgba(text_rgba(self.text_primary, 0.45)).into()
    }

    fn bright_opacity_color(self) -> Hsla {
        rgba(text_rgba(self.text_primary, 0.85)).into()
    }

    fn keycap_bg(self) -> Rgba {
        self.text_primary.with_opacity(KEYCAP_BG_OPACITY).into()
    }
}

fn text_rgba(primary: u32, opacity: f32) -> u32 {
    ((primary & 0x00FF_FFFF) << 8) | ((opacity.clamp(0.0, 1.0) * 255.0).round() as u32)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn variation_row(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .mb_4()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

/// Wraps a footer in a mock window frame for compare mode.
fn mock_window(c: Colors, footer: impl IntoElement) -> AnyElement {
    div()
        .w(px(420.))
        .h(px(120.))
        .flex()
        .flex_col()
        .justify_end()
        .bg(c.background.to_rgb())
        .rounded(px(10.))
        .border_1()
        .border_color(rgba((c.border << 8) | 0x30))
        .overflow_hidden()
        .child(
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .text_xs()
                    .text_color(rgba(text_rgba(c.text_primary, 0.20)))
                    .child("(hover the buttons below)"),
            ),
        )
        .child(footer)
        .into_any()
}

// ── Hint data ───────────────────────────────────────────────────────────────

struct HintDef {
    icon_path: &'static str,
    label: &'static str,
}

fn hint_defs() -> [HintDef; 3] {
    [
        HintDef {
            icon_path: RETURN_ICON_PATH,
            label: "Run",
        },
        HintDef {
            icon_path: COMMAND_ICON_PATH,
            label: "K Actions",
        },
        HintDef {
            icon_path: TAB_ICON_PATH,
            label: "AI",
        },
    ]
}

// ── Base hint rendering (matches current hint_strip.rs exactly) ─────────────

fn render_base_hint(def: &HintDef, color: Hsla, _keycap_bg: Rgba) -> Div {
    let icon = svg()
        .external_path(def.icon_path)
        .size(px(KEY_ICON_SIZE))
        .flex_shrink_0()
        .text_color(color);

    // For "K Actions", render "K" as a keycap badge before the icon
    // Actually the command icon IS the ⌘, and "K" needs its own keycap
    // Let's match the original hint_strip rendering:
    // "⌘K Actions" → command icon + "K" text (no keycap) + "Actions" label
    // Actually looking at parse_hint: ⌘ → command icon, K is part of the label
    // So it's: [⌘ icon] [K Actions] label

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(KEY_ICON_LABEL_GAP))
        .child(icon)
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color)
                .child(def.label.to_string()),
        )
}

/// A hint with keycap-styled icon portion (for variation 6).
fn render_keycap_hint(
    def: &HintDef,
    color: Hsla,
    keycap_bg: Rgba,
    hover_keycap_bg: Rgba,
    id_suffix: &str,
) -> Stateful<Div> {
    let icon = svg()
        .external_path(def.icon_path)
        .size(px(KEY_ICON_SIZE))
        .flex_shrink_0()
        .text_color(color);

    div()
        .id(SharedString::from(format!("keycap-{id_suffix}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(KEY_ICON_LABEL_GAP))
        .cursor_pointer()
        .px(px(4.))
        .py(px(2.))
        .rounded(px(4.))
        .hover(move |s| s.bg(hover_keycap_bg))
        .child(
            div()
                .px(px(KEYCAP_PADDING_X))
                .py(px(KEYCAP_PADDING_Y))
                .rounded(px(KEYCAP_RADIUS))
                .bg(keycap_bg)
                .child(icon),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color)
                .child(def.label.to_string()),
        )
}

// ── Footer container ────────────────────────────────────────────────────────

fn footer_container(c: Colors) -> Div {
    div()
        .w_full()
        .h(px(30.))
        .px(px(14.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .border_t(px(1.))
        .border_color(rgba((c.border << 8) | 0x20))
}

// ── Variation renderers ─────────────────────────────────────────────────────

fn render_variation(c: Colors, num: usize) -> impl IntoElement {
    let defs = hint_defs();
    let hint_color = c.hint_opacity_color();
    let _bright_color = c.bright_opacity_color();
    let keycap_bg = c.keycap_bg();

    let ghost_bg = rgba(text_rgba(c.text_primary, 0.06));
    let pill_bg = rgba(text_rgba(c.text_primary, 0.10));
    let active_bg = rgba(text_rgba(c.text_primary, 0.14));
    let border_hover_color = rgba(text_rgba(c.text_primary, 0.12));
    let underline_color = rgba(text_rgba(c.text_primary, 0.30));
    let hover_keycap_bg = rgba(text_rgba(c.text_primary, 0.25));

    let mut footer = footer_container(c);
    footer = footer.child(div().flex_1());

    let mut hints_row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(HINT_STRIP_CONTENT_GAP));

    for (i, def) in defs.iter().enumerate() {
        let id = SharedString::from(format!("hint-v{num}-{i}"));

        let hint_element: AnyElement = match num {
            // 1. Current + cursor only
            1 => div()
                .id(id)
                .cursor_pointer()
                .child(render_base_hint(def, hint_color, keycap_bg))
                .into_any_element(),

            // 2. Ghost bg hover
            2 => div()
                .id(id)
                .cursor_pointer()
                .px(px(4.))
                .py(px(2.))
                .rounded(px(4.))
                .hover(move |s| s.bg(ghost_bg))
                .active(move |s| s.bg(active_bg))
                .child(render_base_hint(def, hint_color, keycap_bg))
                .into_any_element(),

            // 3. Brighter text hover — use group hover pattern
            // Since we can't dynamically change child text color on parent hover easily,
            // we'll use opacity on the whole group instead
            3 => div()
                .id(id)
                .cursor_pointer()
                .opacity(0.45)
                .hover(|s| s.opacity(0.85))
                .active(|s| s.opacity(0.70))
                .child(render_base_hint(
                    def,
                    rgba(text_rgba(c.text_primary, 1.0)).into(),
                    keycap_bg,
                ))
                .into_any_element(),

            // 4. Ghost bg + brighter text
            4 => div()
                .id(id)
                .cursor_pointer()
                .px(px(4.))
                .py(px(2.))
                .rounded(px(4.))
                .opacity(0.45)
                .hover(move |s| s.bg(ghost_bg).opacity(0.85))
                .active(move |s| s.bg(active_bg).opacity(0.70))
                .child(render_base_hint(
                    def,
                    rgba(text_rgba(c.text_primary, 1.0)).into(),
                    keycap_bg,
                ))
                .into_any_element(),

            // 5. Underline hover
            5 => div()
                .id(id)
                .cursor_pointer()
                .pb(px(1.))
                .border_b(px(1.))
                .border_color(gpui::transparent_black())
                .hover(move |s| s.border_color(underline_color))
                .child(render_base_hint(def, hint_color, keycap_bg))
                .into_any_element(),

            // 6. Keycap badge hover — icon portion gets stronger bg on hover
            6 => render_keycap_hint(
                def,
                hint_color,
                keycap_bg,
                hover_keycap_bg,
                &format!("v6-{i}"),
            )
            .into_any_element(),

            // 7. Opacity press feedback
            7 => div()
                .id(id)
                .cursor_pointer()
                .opacity(0.45)
                .hover(|s| s.opacity(0.65))
                .active(|s| s.opacity(0.35))
                .child(render_base_hint(
                    def,
                    rgba(text_rgba(c.text_primary, 1.0)).into(),
                    keycap_bg,
                ))
                .into_any_element(),

            // 8. Pill bg hover
            8 => div()
                .id(id)
                .cursor_pointer()
                .px(px(6.))
                .py(px(2.))
                .rounded(px(10.))
                .hover(move |s| s.bg(pill_bg))
                .active(move |s| s.bg(active_bg))
                .child(render_base_hint(def, hint_color, keycap_bg))
                .into_any_element(),

            // 9. Border hover
            9 => div()
                .id(id)
                .cursor_pointer()
                .px(px(5.))
                .py(px(2.))
                .rounded(px(6.))
                .border_1()
                .border_color(gpui::transparent_black())
                .hover(move |s| s.border_color(border_hover_color))
                .active(move |s| s.bg(ghost_bg).border_color(border_hover_color))
                .child(render_base_hint(def, hint_color, keycap_bg))
                .into_any_element(),

            _ => render_base_hint(def, hint_color, keycap_bg).into_any_element(),
        };

        hints_row = hints_row.child(hint_element);
    }

    footer.child(hints_row)
}

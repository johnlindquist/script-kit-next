//! Deterministic Quick Terminal preview fixtures for Storybook.
//!
//! These fixtures render Quick Terminal as app chrome (header, terminal body,
//! and apply-back/command chrome) using in-memory terminal rows. The runtime
//! PTY in `src/terminal/pty.rs` is intentionally not touched so the story is
//! safe to render in compare mode and headless verification.

use gpui::{div, prelude::*, px, rgba, AnyElement, FontWeight};

use crate::list_item::FONT_MONO;
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuickTerminalStateId {
    ColdEmpty,
    ActivePtyContent,
    ThemeVariant,
    ApplyBackReady,
}

impl QuickTerminalStateId {
    pub const ALL: [Self; 4] = [
        Self::ColdEmpty,
        Self::ActivePtyContent,
        Self::ThemeVariant,
        Self::ApplyBackReady,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ColdEmpty => "cold-empty",
            Self::ActivePtyContent => "active-pty-content",
            Self::ThemeVariant => "theme-variant",
            Self::ApplyBackReady => "apply-back-ready",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ColdEmpty => "Cold / Empty",
            Self::ActivePtyContent => "Active PTY Content",
            Self::ThemeVariant => "Theme Variant",
            Self::ApplyBackReady => "Apply-Back Ready",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::ColdEmpty => "Quick Terminal shell with no PTY output yet.",
            Self::ActivePtyContent => {
                "Deterministic terminal output fixture with prompt, command, and output rows."
            }
            Self::ThemeVariant => {
                "Theme-sensitive Quick Terminal chrome and terminal palette preview."
            }
            Self::ApplyBackReady => "Terminal output with apply-back affordance visible and ready.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "cold-empty" => Some(Self::ColdEmpty),
            "active-pty-content" => Some(Self::ActivePtyContent),
            "theme-variant" => Some(Self::ThemeVariant),
            "apply-back-ready" => Some(Self::ApplyBackReady),
            _ => None,
        }
    }
}

pub fn quick_terminal_state_story_variants() -> Vec<StoryVariant> {
    QuickTerminalStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "quickTerminal")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_quick_terminal_state_preview(stable_id: &str) -> AnyElement {
    let id =
        QuickTerminalStateId::from_stable_id(stable_id).unwrap_or(QuickTerminalStateId::ColdEmpty);
    render_quick_terminal_shell(id, fixture_for(id), false)
}

pub fn render_quick_terminal_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id =
        QuickTerminalStateId::from_stable_id(stable_id).unwrap_or(QuickTerminalStateId::ColdEmpty);
    render_quick_terminal_shell(id, fixture_for(id), true)
}

#[derive(Clone, Debug)]
struct QuickTerminalFixture {
    title: &'static str,
    cwd: &'static str,
    command: &'static str,
    rows: &'static [&'static str],
    apply_back_label: Option<&'static str>,
    theme_note: Option<&'static str>,
}

fn fixture_for(id: QuickTerminalStateId) -> QuickTerminalFixture {
    match id {
        QuickTerminalStateId::ColdEmpty => QuickTerminalFixture {
            title: "Quick Terminal",
            cwd: "~/dev/script-kit-gpui",
            command: "",
            rows: &[],
            apply_back_label: None,
            theme_note: None,
        },
        QuickTerminalStateId::ActivePtyContent => QuickTerminalFixture {
            title: "Quick Terminal — bun test",
            cwd: "~/dev/script-kit-gpui",
            command: "bun test smoke",
            rows: &[
                "$ bun test smoke",
                "bun test v1.1.30 (script-kit-gpui)",
                "smoke › launcher boots                   [pass] 12.4ms",
                "smoke › builtin browser opens            [pass]  8.2ms",
                "smoke › quick terminal renders chrome    [pass]  6.7ms",
                "",
                " 3 pass, 0 fail (0.0s)",
            ],
            apply_back_label: None,
            theme_note: None,
        },
        QuickTerminalStateId::ThemeVariant => QuickTerminalFixture {
            title: "Quick Terminal — themed",
            cwd: "~/dev/script-kit-gpui",
            command: "source checks",
            rows: &[
                "$ source checks",
                "lat: 142 sections checked",
                "lat: 0 broken doc links",
                "lat: 0 stale code refs",
                "lat: ok",
            ],
            apply_back_label: None,
            theme_note: Some("Theme-aware terminal palette"),
        },
        QuickTerminalStateId::ApplyBackReady => QuickTerminalFixture {
            title: "Quick Terminal — apply ready",
            cwd: "~/dev/script-kit-gpui",
            command: "cargo fmt -- --check",
            rows: &[
                "$ cargo fmt -- --check",
                "Diff in src/main.rs at line 312",
                "Diff in src/storybook/quick_terminal_states.rs at line 18",
                "",
                "2 files need formatting",
            ],
            apply_back_label: Some("Apply to Editor  ⏎"),
            theme_note: None,
        },
    }
}

fn render_quick_terminal_shell(
    id: QuickTerminalStateId,
    fixture: QuickTerminalFixture,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 520.0 } else { 820.0 };
    let height = if compact { 300.0 } else { 480.0 };
    let min_h = if compact { 320.0 } else { 520.0 };

    div()
        .w_full()
        .min_h(px(min_h))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .h(px(height))
                .rounded(px(10.0))
                .overflow_hidden()
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x66))
                .bg(theme.colors.background.main.to_rgb())
                .flex()
                .flex_col()
                .child(render_quick_terminal_header(id, &fixture, compact))
                .child(render_terminal_content_panel(&fixture, compact))
                .child(render_quick_terminal_command_bar(&fixture, compact)),
        )
        .into_any_element()
}

fn render_quick_terminal_header(
    _id: QuickTerminalStateId,
    fixture: &QuickTerminalFixture,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let pad_x = if compact { 12.0 } else { 18.0 };
    let pad_y = if compact { 8.0 } else { 10.0 };
    let title_size = if compact { 12.0 } else { 13.0 };

    div()
        .w_full()
        .px(px(pad_x))
        .py(px(pad_y))
        .border_b_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x50))
        .bg(rgba((theme.colors.background.main << 8) | 0xcc))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(5.0))
                .child(traffic_dot(0xff5f57))
                .child(traffic_dot(0xfebc2e))
                .child(traffic_dot(0x28c840)),
        )
        .child(
            div()
                .flex_1()
                .text_size(px(title_size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child(fixture.title),
        )
        .child(
            div()
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(fixture.cwd),
        )
        .into_any_element()
}

fn traffic_dot(color: u32) -> AnyElement {
    div()
        .w(px(10.0))
        .h(px(10.0))
        .rounded(px(5.0))
        .bg(rgba((color << 8) | 0xff))
        .into_any_element()
}

fn render_terminal_content_panel(fixture: &QuickTerminalFixture, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let pad = if compact { 12.0 } else { 16.0 };
    let line_size = if compact { 11.5 } else { 12.5 };
    let line_height = if compact { 16.0 } else { 18.0 };

    let body = if fixture.rows.is_empty() {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_size(px(line_size))
                    .font_family(FONT_MONO)
                    .text_color(theme.colors.text.muted.to_rgb())
                    .child("Press a key to start a shell session…"),
            )
            .into_any_element()
    } else {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .children(fixture.rows.iter().map(|row| {
                div()
                    .text_size(px(line_size))
                    .line_height(px(line_height))
                    .font_family(FONT_MONO)
                    .text_color(theme.colors.text.secondary.to_rgb())
                    .child(*row)
            }))
            .into_any_element()
    };

    div()
        .w_full()
        .flex_1()
        .min_h(px(0.0))
        .overflow_hidden()
        .px(px(pad))
        .py(px(pad))
        .bg(rgba((theme.colors.background.main << 8) | 0xee))
        .flex()
        .flex_col()
        .child(body)
        .when_some(fixture.theme_note, |container, note| {
            container.child(
                div()
                    .mt(px(8.0))
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.colors.accent.selected.to_rgb())
                    .child(note),
            )
        })
        .into_any_element()
}

fn render_quick_terminal_command_bar(fixture: &QuickTerminalFixture, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let pad_x = if compact { 12.0 } else { 18.0 };
    let pad_y = if compact { 8.0 } else { 10.0 };
    let cmd_size = if compact { 11.5 } else { 13.0 };

    let command_text = if fixture.command.is_empty() {
        "type a command…"
    } else {
        fixture.command
    };
    let command_color = if fixture.command.is_empty() {
        theme.colors.text.dimmed.to_rgb()
    } else {
        theme.colors.text.primary.to_rgb()
    };

    div()
        .w_full()
        .px(px(pad_x))
        .py(px(pad_y))
        .border_t_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x55))
        .bg(rgba((theme.colors.background.main << 8) | 0xdd))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .text_size(px(cmd_size))
                .font_family(FONT_MONO)
                .text_color(theme.colors.accent.selected.to_rgb())
                .child("›"),
        )
        .child(
            div()
                .flex_1()
                .text_size(px(cmd_size))
                .font_family(FONT_MONO)
                .text_color(command_color)
                .child(command_text),
        )
        .when_some(fixture.apply_back_label, |container, label| {
            container.child(
                div()
                    .px(px(9.0))
                    .py(px(3.0))
                    .rounded(px(5.0))
                    .border_1()
                    .border_color(rgba((theme.colors.accent.selected << 8) | 0x66))
                    .bg(rgba((theme.colors.accent.selected << 8) | 0x18))
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.colors.accent.selected.to_rgb())
                    .child(label),
            )
        })
        .child(
            div()
                .text_xs()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child("⌘. Stop"),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_terminal_variants_are_deterministic_presenter_fixtures() {
        let variants = quick_terminal_state_story_variants();
        assert_eq!(variants.len(), 4);
        for variant in variants {
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("presenterFixture")
            );
            assert_eq!(
                variant.props.get("surface").map(String::as_str),
                Some("quickTerminal")
            );
            assert!(!variant.props.contains_key("fixtureImagePresent"));
            assert!(!variant.props.contains_key("fixtureManifestPresent"));
        }
    }

    #[test]
    fn all_state_ids_round_trip_through_stable_id() {
        for id in QuickTerminalStateId::ALL {
            assert_eq!(QuickTerminalStateId::from_stable_id(id.as_str()), Some(id));
        }
    }
}

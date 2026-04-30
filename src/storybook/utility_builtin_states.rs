//! Presenter-backed one-off utility built-in states for Storybook.
//!
//! These surfaces do not all share the expanded browser shell, but they do
//! reuse the same row, button, divider, and footer primitives as the live app.

use gpui::{div, prelude::*, px, rgba, AnyElement, FontWeight, SharedString};

use crate::components::{Button, ButtonColors, ButtonVariant, SectionDivider};
use crate::list_item::{ListItem, ListItemColors};
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UtilityBuiltinStateId {
    EmojiPicker,
    AppLauncher,
    ProcessManager,
    Settings,
    ThemeChooser,
    DesignGallery,
}

impl UtilityBuiltinStateId {
    pub const ALL: [Self; 6] = [
        Self::EmojiPicker,
        Self::AppLauncher,
        Self::ProcessManager,
        Self::Settings,
        Self::ThemeChooser,
        Self::DesignGallery,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmojiPicker => "emoji-picker",
            Self::AppLauncher => "app-launcher",
            Self::ProcessManager => "process-manager",
            Self::Settings => "settings",
            Self::ThemeChooser => "theme-chooser",
            Self::DesignGallery => "design-gallery",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::EmojiPicker => "Emoji Picker",
            Self::AppLauncher => "App Launcher",
            Self::ProcessManager => "Process Manager",
            Self::Settings => "Settings",
            Self::ThemeChooser => "Theme Chooser",
            Self::DesignGallery => "Design Gallery",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::EmojiPicker => "Grid picker state with selected emoji and footer hints.",
            Self::AppLauncher => "Minimal app launcher list with selected application row.",
            Self::ProcessManager => "Process list with status metadata and stop actions.",
            Self::Settings => "Settings hub list with sectioned command rows.",
            Self::ThemeChooser => "Theme list and preview panel with color swatches.",
            Self::DesignGallery => "Design token gallery with grouped icon and separator rows.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "emoji-picker" => Some(Self::EmojiPicker),
            "app-launcher" => Some(Self::AppLauncher),
            "process-manager" => Some(Self::ProcessManager),
            "settings" => Some(Self::Settings),
            "theme-chooser" => Some(Self::ThemeChooser),
            "design-gallery" => Some(Self::DesignGallery),
            _ => None,
        }
    }
}

pub fn utility_builtin_state_story_variants() -> Vec<StoryVariant> {
    UtilityBuiltinStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "utilityBuiltIn")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_utility_builtin_state_preview(stable_id: &str) -> AnyElement {
    let id = UtilityBuiltinStateId::from_stable_id(stable_id)
        .unwrap_or(UtilityBuiltinStateId::EmojiPicker);
    render_utility_builtin_state(id, false)
}

pub fn render_utility_builtin_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = UtilityBuiltinStateId::from_stable_id(stable_id)
        .unwrap_or(UtilityBuiltinStateId::EmojiPicker);
    render_utility_builtin_state(id, true)
}

fn render_utility_builtin_state(id: UtilityBuiltinStateId, compact: bool) -> AnyElement {
    match id {
        UtilityBuiltinStateId::EmojiPicker => {
            render_shell(id, render_emoji_picker(compact), compact)
        }
        UtilityBuiltinStateId::AppLauncher => {
            render_shell(id, render_app_launcher(compact), compact)
        }
        UtilityBuiltinStateId::ProcessManager => {
            render_shell(id, render_process_manager(compact), compact)
        }
        UtilityBuiltinStateId::Settings => render_shell(id, render_settings(compact), compact),
        UtilityBuiltinStateId::ThemeChooser => {
            render_shell(id, render_theme_chooser(compact), compact)
        }
        UtilityBuiltinStateId::DesignGallery => {
            render_shell(id, render_design_gallery(compact), compact)
        }
    }
}

fn render_shell(id: UtilityBuiltinStateId, body: impl IntoElement, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 500.0 } else { 780.0 };
    let height = if compact { 330.0 } else { 500.0 };

    div()
        .w_full()
        .min_h(px(if compact { 350.0 } else { 540.0 }))
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
                .child(render_header(id, compact))
                .child(body)
                .child(crate::components::render_simple_hint_strip(
                    vec![
                        SharedString::from("enter Select"),
                        SharedString::from("cmd+k Actions"),
                        SharedString::from("esc Back"),
                    ],
                    None,
                )),
        )
        .into_any_element()
}

fn render_header(id: UtilityBuiltinStateId, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .px(px(if compact { 14.0 } else { 18.0 }))
        .py(px(if compact { 10.0 } else { 14.0 }))
        .flex()
        .flex_col()
        .gap(px(3.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child(id.name()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child(id.as_str()),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(id.description()),
        )
        .child(SectionDivider::new())
        .into_any_element()
}

fn render_emoji_picker(compact: bool) -> AnyElement {
    let emoji = [
        ("\u{1f600}", "Grinning"),
        ("\u{1f680}", "Rocket"),
        ("\u{2728}", "Sparkles"),
        ("\u{1f4a1}", "Idea"),
        ("\u{1f4cc}", "Pin"),
        ("\u{1f50d}", "Search"),
        ("\u{1f4dd}", "Memo"),
        ("\u{1f4e6}", "Package"),
        ("\u{2699}", "Settings"),
        ("\u{1f9ea}", "Test"),
        ("\u{1f4ca}", "Chart"),
        ("\u{1f517}", "Link"),
    ];
    let theme = get_cached_theme();

    div()
        .flex_1()
        .min_h(px(0.0))
        .p(px(if compact { 12.0 } else { 18.0 }))
        .grid()
        .grid_cols(4)
        .gap(px(if compact { 8.0 } else { 10.0 }))
        .children(emoji.iter().enumerate().map(|(ix, (glyph, label))| {
            div()
                .rounded(px(6.0))
                .border_1()
                .border_color(rgba(
                    (theme.colors.ui.border << 8) | if ix == 1 { 0xb0 } else { 0x44 },
                ))
                .bg(rgba(
                    (theme.colors.accent.selected_subtle << 8) | if ix == 1 { 0x88 } else { 0x22 },
                ))
                .p(px(if compact { 8.0 } else { 12.0 }))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(6.0))
                .child(
                    div()
                        .text_size(px(if compact { 22.0 } else { 30.0 }))
                        .child(*glyph),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child(*label),
                )
        }))
        .into_any_element()
}

fn render_app_launcher(compact: bool) -> AnyElement {
    let rows = [
        ("Arc", "com.thebrowser.Browser - running", "app"),
        ("Ghostty", "com.mitchellh.ghostty - running", "app"),
        ("Visual Studio Code", "com.microsoft.VSCode - recent", "app"),
        ("Finder", "com.apple.finder - system", "app"),
        ("Activity Monitor", "com.apple.ActivityMonitor", "app"),
    ];
    render_row_list(&rows, 1, compact)
}

fn render_process_manager(compact: bool) -> AnyElement {
    let rows = [
        (
            "cargo-watch",
            "Running - 18m - storybook build loop",
            "proc",
        ),
        ("script-kit-gpui", "Running - main window", "proc"),
        ("node", "Idle - SDK bridge", "proc"),
        ("agentic-testing", "Stopped - last receipt 10:42", "proc"),
    ];
    let theme = get_cached_theme();
    let button_colors = ButtonColors::from_theme(&theme);

    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .child(render_row_list(&rows, 0, compact))
        .child(
            div()
                .px(px(if compact { 12.0 } else { 18.0 }))
                .pb(px(if compact { 10.0 } else { 14.0 }))
                .flex()
                .gap(px(8.0))
                .child(Button::new("Stop Selected", button_colors).variant(ButtonVariant::Ghost))
                .child(Button::new("Stop All", button_colors).variant(ButtonVariant::Primary)),
        )
        .into_any_element()
}

fn render_settings(compact: bool) -> AnyElement {
    let rows = [
        ("Appearance", "Theme, vibrancy, and font controls", "set"),
        ("Shortcuts", "Record launcher and script shortcuts", "set"),
        ("Permissions", "Automation and accessibility status", "set"),
        ("Frecency", "Reset ranking and launch history", "set"),
        ("Developer", "Logs, diagnostics, and debug overlays", "set"),
    ];
    render_row_list(&rows, 2, compact)
}

fn render_theme_chooser(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let colors = ListItemColors::from_theme(&theme);
    let swatches = [0xffcc66, 0x66d9ef, 0xa6e22e, 0xf92672];

    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_row()
        .child(
            div()
                .w_1_2()
                .h_full()
                .py(px(6.0))
                .child(
                    ListItem::new("Default", colors)
                        .description("Current theme - dark vibrancy")
                        .selected(true)
                        .with_accent_bar(true),
                )
                .child(
                    ListItem::new("Solarized Light", colors)
                        .description("Light palette")
                        .with_accent_bar(true),
                )
                .child(
                    ListItem::new("High Contrast", colors)
                        .description("Accessibility preset")
                        .with_accent_bar(true),
                ),
        )
        .child(
            div()
                .w_1_2()
                .h_full()
                .p(px(if compact { 12.0 } else { 18.0 }))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child("Preview"),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .children(swatches.into_iter().map(|hex| {
                            div()
                                .w(px(38.0))
                                .h(px(38.0))
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(rgba((theme.colors.ui.border << 8) | 0x77))
                                .bg(gpui::rgb(hex))
                        })),
                )
                .child(SectionDivider::new())
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.colors.text.secondary.to_rgb())
                        .child("Theme chooser previews type, chrome, accent, and border tokens before adoption."),
                ),
        )
        .into_any_element()
}

fn render_design_gallery(compact: bool) -> AnyElement {
    let rows = [
        ("Separator: whisper", "1px low-opacity divider", "sep"),
        ("Icon: command", "Shortcut glyph icon", "ico"),
        ("Icon: external link", "Outbound affordance", "ico"),
        (
            "Chrome: selected row",
            "Accent bar and selected background",
            "row",
        ),
        ("Chrome: hover row", "Subtle hover overlay", "row"),
    ];
    render_row_list(&rows, 0, compact)
}

fn render_row_list(
    rows: &[(&'static str, &'static str, &'static str)],
    selected_index: usize,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let colors = ListItemColors::from_theme(&theme);

    div()
        .flex_1()
        .min_h(px(0.0))
        .px(px(if compact { 10.0 } else { 14.0 }))
        .py(px(if compact { 8.0 } else { 12.0 }))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .children(
            rows.iter()
                .enumerate()
                .map(|(ix, (title, description, badge))| {
                    ListItem::new(*title, colors)
                        .description(*description)
                        .tool_badge(*badge)
                        .selected(ix == selected_index)
                        .with_accent_bar(true)
                }),
        )
        .into_any_element()
}

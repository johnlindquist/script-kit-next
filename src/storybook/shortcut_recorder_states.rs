//! Shortcut recorder Storybook presenter states.
//!
//! These fixtures mirror the compact detached recorder chrome without
//! requiring keyboard input, so compare mode can show every important state.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::components::hint_strip::{render_inline_shortcut_keys, whisper_inline_shortcut_colors};
use crate::components::shortcut_recorder::{
    RecordedShortcut, ShortcutConflict, ShortcutRecorderColors,
};
use crate::theme::get_cached_theme;
use crate::ui_foundation::get_vibrancy_background;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutRecorderStateId {
    Empty,
    ModifiersHeld,
    Complete,
    Conflict,
}

impl ShortcutRecorderStateId {
    pub const ALL: [Self; 4] = [
        Self::Empty,
        Self::ModifiersHeld,
        Self::Complete,
        Self::Conflict,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::ModifiersHeld => "modifiers-held",
            Self::Complete => "complete",
            Self::Conflict => "conflict",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "Press Keys",
            Self::ModifiersHeld => "Modifiers Held",
            Self::Complete => "Complete Shortcut",
            Self::Conflict => "Conflict Warning",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Empty => "Initial recording state before any key input.",
            Self::ModifiersHeld => "Live preview while modifiers are held without a final key.",
            Self::Complete => "Valid shortcut ready to save.",
            Self::Conflict => "Complete shortcut blocked by an existing command conflict.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "modifiers-held" => Some(Self::ModifiersHeld),
            "complete" => Some(Self::Complete),
            "conflict" => Some(Self::Conflict),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShortcutRecorderStateSpec {
    pub id: ShortcutRecorderStateId,
    pub command_name: &'static str,
    pub keycaps: Vec<String>,
    pub shortcut: RecordedShortcut,
    pub conflict: Option<ShortcutConflict>,
}

impl ShortcutRecorderStateSpec {
    fn can_save(&self) -> bool {
        self.shortcut.is_complete() && self.conflict.is_none()
    }

    fn can_clear(&self) -> bool {
        !self.keycaps.is_empty() || !self.shortcut.is_empty()
    }
}

pub fn shortcut_recorder_state_specs() -> Vec<ShortcutRecorderStateSpec> {
    ShortcutRecorderStateId::ALL
        .iter()
        .copied()
        .map(shortcut_recorder_state_spec)
        .collect()
}

pub fn shortcut_recorder_state_spec(id: ShortcutRecorderStateId) -> ShortcutRecorderStateSpec {
    match id {
        ShortcutRecorderStateId::Empty => ShortcutRecorderStateSpec {
            id,
            command_name: "Open Actions",
            keycaps: Vec::new(),
            shortcut: RecordedShortcut::new(),
            conflict: None,
        },
        ShortcutRecorderStateId::ModifiersHeld => ShortcutRecorderStateSpec {
            id,
            command_name: "Open Actions",
            keycaps: vec!["⌘".into(), "⇧".into()],
            shortcut: RecordedShortcut {
                cmd: true,
                shift: true,
                ..RecordedShortcut::default()
            },
            conflict: None,
        },
        ShortcutRecorderStateId::Complete => {
            let shortcut = RecordedShortcut {
                cmd: true,
                shift: true,
                key: Some("K".into()),
                ..RecordedShortcut::default()
            };
            ShortcutRecorderStateSpec {
                id,
                command_name: "Open Actions",
                keycaps: shortcut.to_keycaps(),
                shortcut,
                conflict: None,
            }
        }
        ShortcutRecorderStateId::Conflict => {
            let shortcut = RecordedShortcut {
                cmd: true,
                key: Some("K".into()),
                ..RecordedShortcut::default()
            };
            ShortcutRecorderStateSpec {
                id,
                command_name: "Open Actions",
                keycaps: shortcut.to_keycaps(),
                shortcut,
                conflict: Some(ShortcutConflict {
                    command_name: "Search Scripts".into(),
                    shortcut: "cmd+k".into(),
                }),
            }
        }
    }
}

pub fn render_shortcut_recorder_state_preview(stable_id: &str) -> AnyElement {
    let id = ShortcutRecorderStateId::from_stable_id(stable_id)
        .unwrap_or(ShortcutRecorderStateId::Empty);
    render_shortcut_recorder_state(shortcut_recorder_state_spec(id), false)
}

pub fn render_shortcut_recorder_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = ShortcutRecorderStateId::from_stable_id(stable_id)
        .unwrap_or(ShortcutRecorderStateId::Empty);
    render_shortcut_recorder_state(shortcut_recorder_state_spec(id), true)
}

fn render_shortcut_recorder_state(spec: ShortcutRecorderStateSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let colors = ShortcutRecorderColors::from_theme(&theme);
    let button_colors = ButtonColors::from_theme(&theme);
    let width = if compact { 292.0 } else { 320.0 };

    div()
        .w_full()
        .min_h(px(if compact { 176.0 } else { 240.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .p(px(18.0))
                .when_some(get_vibrancy_background(&theme), |d, bg| d.bg(bg))
                .border_1()
                .border_color(rgba((colors.text_primary << 8) | 0x22))
                .rounded(px(8.0))
                .flex()
                .flex_col()
                .child(render_header(&spec, colors))
                .child(div().h(px(10.0)))
                .child(render_key_display(&spec, colors))
                .child(render_conflict_warning(&spec, colors))
                .child(render_button_row(&spec, button_colors, compact)),
        )
        .into_any_element()
}

fn render_header(spec: &ShortcutRecorderStateSpec, colors: ShortcutRecorderColors) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .w(px(2.0))
                .h(px(14.0))
                .rounded(px(1.0))
                .bg(rgb(colors.accent)),
        )
        .child(
            div()
                .min_w(px(0.0))
                .truncate()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(colors.text_primary))
                .child(spec.command_name),
        )
        .into_any_element()
}

fn render_key_display(
    spec: &ShortcutRecorderStateSpec,
    colors: ShortcutRecorderColors,
) -> AnyElement {
    let content = if spec.keycaps.is_empty() {
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgba((colors.text_primary << 8) | 0x66))
            .child("Press keys")
            .into_any_element()
    } else {
        render_inline_shortcut_keys(
            spec.keycaps.iter().map(String::as_str),
            whisper_inline_shortcut_colors(
                rgba((colors.text_primary << 8) | 0xCC).into(),
                rgba((colors.text_primary << 8) | 0xFF).into(),
                true,
            ),
        )
    };

    div()
        .w_full()
        .h(px(44.0))
        .px(px(12.0))
        .rounded(px(6.0))
        .bg(rgba((colors.text_primary << 8) | 0x0C))
        .border_1()
        .border_color(rgba((colors.text_primary << 8) | 0x18))
        .flex()
        .items_center()
        .justify_center()
        .child(content)
        .into_any_element()
}

fn render_conflict_warning(
    spec: &ShortcutRecorderStateSpec,
    colors: ShortcutRecorderColors,
) -> AnyElement {
    if let Some(conflict) = &spec.conflict {
        div()
            .w_full()
            .mt(px(8.0))
            .text_xs()
            .text_color(rgb(colors.warning))
            .text_center()
            .child(format!("Conflicts with \"{}\"", conflict.command_name))
            .into_any_element()
    } else {
        div().into_any_element()
    }
}

fn render_button_row(
    spec: &ShortcutRecorderStateSpec,
    button_colors: ButtonColors,
    compact: bool,
) -> AnyElement {
    let mut row = div()
        .w_full()
        .mt(px(12.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .gap(px(if compact { 5.0 } else { 6.0 }));

    if spec.can_clear() {
        row = row.child(Button::new("Clear", button_colors).variant(ButtonVariant::Ghost));
    }

    row.child(Button::new("Cancel", button_colors).variant(ButtonVariant::Ghost))
        .child(
            Button::new("Save", button_colors)
                .variant(ButtonVariant::Primary)
                .disabled(!spec.can_save()),
        )
        .into_any_element()
}

//! Theme color knob catalog for the dev style tool Theme Inspector tab.
//!
//! Each knob maps a stable string id (e.g. `theme.colors.text.primary`) to a
//! getter/setter over [`crate::theme::Theme`]'s `ColorScheme` so the runtime
//! override store can hold live color overrides and the export/devtools
//! surfaces can address them by id. The terminal/ANSI palette is intentionally
//! excluded; this catalog covers the app chrome colors only.

use crate::theme::hex_color::HexColor;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThemeColorKnobId(&'static str);

impl ThemeColorKnobId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColorKnobGroup {
    Background,
    Text,
    Accent,
    Ui,
}

impl ThemeColorKnobGroup {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Background => "Theme background colors",
            Self::Text => "Theme text colors",
            Self::Accent => "Theme accent colors",
            Self::Ui => "Theme UI colors",
        }
    }

    /// One-line anatomy hint tying the group to the surfaces it colors.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Background => "Window, title bar, search box, and log panel surfaces",
            Self::Text => "Primary through dimmed text plus text on accent fills",
            Self::Accent => "Selection highlight and subtle selection fills",
            Self::Ui => "Borders plus success / error / warning / info colors",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColorKnob {
    pub id: ThemeColorKnobId,
    pub label: &'static str,
    pub group: ThemeColorKnobGroup,
    pub get: fn(&Theme) -> HexColor,
    pub apply: fn(&mut Theme, HexColor),
}

macro_rules! theme_color_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$field:ident) => {
        pub const $id_const: ThemeColorKnobId = ThemeColorKnobId::new($id);
        fn $get_fn(theme: &Theme) -> HexColor {
            theme.colors.$section.$field
        }
        fn $apply_fn(theme: &mut Theme, value: HexColor) {
            theme.colors.$section.$field = value;
        }
    };
}

theme_color_knob!(
    THEME_BACKGROUND_MAIN_KNOB_ID,
    get_background_main,
    apply_background_main,
    "theme.colors.background.main",
    background.main
);
theme_color_knob!(
    THEME_BACKGROUND_TITLE_BAR_KNOB_ID,
    get_background_title_bar,
    apply_background_title_bar,
    "theme.colors.background.titleBar",
    background.title_bar
);
theme_color_knob!(
    THEME_BACKGROUND_SEARCH_BOX_KNOB_ID,
    get_background_search_box,
    apply_background_search_box,
    "theme.colors.background.searchBox",
    background.search_box
);
theme_color_knob!(
    THEME_BACKGROUND_LOG_PANEL_KNOB_ID,
    get_background_log_panel,
    apply_background_log_panel,
    "theme.colors.background.logPanel",
    background.log_panel
);
theme_color_knob!(
    THEME_TEXT_PRIMARY_KNOB_ID,
    get_text_primary,
    apply_text_primary,
    "theme.colors.text.primary",
    text.primary
);
theme_color_knob!(
    THEME_TEXT_SECONDARY_KNOB_ID,
    get_text_secondary,
    apply_text_secondary,
    "theme.colors.text.secondary",
    text.secondary
);
theme_color_knob!(
    THEME_TEXT_TERTIARY_KNOB_ID,
    get_text_tertiary,
    apply_text_tertiary,
    "theme.colors.text.tertiary",
    text.tertiary
);
theme_color_knob!(
    THEME_TEXT_MUTED_KNOB_ID,
    get_text_muted,
    apply_text_muted,
    "theme.colors.text.muted",
    text.muted
);
theme_color_knob!(
    THEME_TEXT_DIMMED_KNOB_ID,
    get_text_dimmed,
    apply_text_dimmed,
    "theme.colors.text.dimmed",
    text.dimmed
);
theme_color_knob!(
    THEME_TEXT_ON_ACCENT_KNOB_ID,
    get_text_on_accent,
    apply_text_on_accent,
    "theme.colors.text.onAccent",
    text.on_accent
);
theme_color_knob!(
    THEME_ACCENT_SELECTED_KNOB_ID,
    get_accent_selected,
    apply_accent_selected,
    "theme.colors.accent.selected",
    accent.selected
);
theme_color_knob!(
    THEME_ACCENT_SELECTED_SUBTLE_KNOB_ID,
    get_accent_selected_subtle,
    apply_accent_selected_subtle,
    "theme.colors.accent.selectedSubtle",
    accent.selected_subtle
);
theme_color_knob!(
    THEME_UI_BORDER_KNOB_ID,
    get_ui_border,
    apply_ui_border,
    "theme.colors.ui.border",
    ui.border
);
theme_color_knob!(
    THEME_UI_SUCCESS_KNOB_ID,
    get_ui_success,
    apply_ui_success,
    "theme.colors.ui.success",
    ui.success
);
theme_color_knob!(
    THEME_UI_ERROR_KNOB_ID,
    get_ui_error,
    apply_ui_error,
    "theme.colors.ui.error",
    ui.error
);
theme_color_knob!(
    THEME_UI_WARNING_KNOB_ID,
    get_ui_warning,
    apply_ui_warning,
    "theme.colors.ui.warning",
    ui.warning
);
theme_color_knob!(
    THEME_UI_INFO_KNOB_ID,
    get_ui_info,
    apply_ui_info,
    "theme.colors.ui.info",
    ui.info
);

pub const THEME_COLOR_KNOBS: &[ThemeColorKnob] = &[
    ThemeColorKnob {
        id: THEME_BACKGROUND_MAIN_KNOB_ID,
        label: "Background / Main",
        group: ThemeColorKnobGroup::Background,
        get: get_background_main,
        apply: apply_background_main,
    },
    ThemeColorKnob {
        id: THEME_BACKGROUND_TITLE_BAR_KNOB_ID,
        label: "Background / Title bar",
        group: ThemeColorKnobGroup::Background,
        get: get_background_title_bar,
        apply: apply_background_title_bar,
    },
    ThemeColorKnob {
        id: THEME_BACKGROUND_SEARCH_BOX_KNOB_ID,
        label: "Background / Search box",
        group: ThemeColorKnobGroup::Background,
        get: get_background_search_box,
        apply: apply_background_search_box,
    },
    ThemeColorKnob {
        id: THEME_BACKGROUND_LOG_PANEL_KNOB_ID,
        label: "Background / Log panel",
        group: ThemeColorKnobGroup::Background,
        get: get_background_log_panel,
        apply: apply_background_log_panel,
    },
    ThemeColorKnob {
        id: THEME_TEXT_PRIMARY_KNOB_ID,
        label: "Text / Primary",
        group: ThemeColorKnobGroup::Text,
        get: get_text_primary,
        apply: apply_text_primary,
    },
    ThemeColorKnob {
        id: THEME_TEXT_SECONDARY_KNOB_ID,
        label: "Text / Secondary",
        group: ThemeColorKnobGroup::Text,
        get: get_text_secondary,
        apply: apply_text_secondary,
    },
    ThemeColorKnob {
        id: THEME_TEXT_TERTIARY_KNOB_ID,
        label: "Text / Tertiary",
        group: ThemeColorKnobGroup::Text,
        get: get_text_tertiary,
        apply: apply_text_tertiary,
    },
    ThemeColorKnob {
        id: THEME_TEXT_MUTED_KNOB_ID,
        label: "Text / Muted",
        group: ThemeColorKnobGroup::Text,
        get: get_text_muted,
        apply: apply_text_muted,
    },
    ThemeColorKnob {
        id: THEME_TEXT_DIMMED_KNOB_ID,
        label: "Text / Dimmed",
        group: ThemeColorKnobGroup::Text,
        get: get_text_dimmed,
        apply: apply_text_dimmed,
    },
    ThemeColorKnob {
        id: THEME_TEXT_ON_ACCENT_KNOB_ID,
        label: "Text / On accent",
        group: ThemeColorKnobGroup::Text,
        get: get_text_on_accent,
        apply: apply_text_on_accent,
    },
    ThemeColorKnob {
        id: THEME_ACCENT_SELECTED_KNOB_ID,
        label: "Accent / Selected",
        group: ThemeColorKnobGroup::Accent,
        get: get_accent_selected,
        apply: apply_accent_selected,
    },
    ThemeColorKnob {
        id: THEME_ACCENT_SELECTED_SUBTLE_KNOB_ID,
        label: "Accent / Selected subtle",
        group: ThemeColorKnobGroup::Accent,
        get: get_accent_selected_subtle,
        apply: apply_accent_selected_subtle,
    },
    ThemeColorKnob {
        id: THEME_UI_BORDER_KNOB_ID,
        label: "UI / Border",
        group: ThemeColorKnobGroup::Ui,
        get: get_ui_border,
        apply: apply_ui_border,
    },
    ThemeColorKnob {
        id: THEME_UI_SUCCESS_KNOB_ID,
        label: "UI / Success",
        group: ThemeColorKnobGroup::Ui,
        get: get_ui_success,
        apply: apply_ui_success,
    },
    ThemeColorKnob {
        id: THEME_UI_ERROR_KNOB_ID,
        label: "UI / Error",
        group: ThemeColorKnobGroup::Ui,
        get: get_ui_error,
        apply: apply_ui_error,
    },
    ThemeColorKnob {
        id: THEME_UI_WARNING_KNOB_ID,
        label: "UI / Warning",
        group: ThemeColorKnobGroup::Ui,
        get: get_ui_warning,
        apply: apply_ui_warning,
    },
    ThemeColorKnob {
        id: THEME_UI_INFO_KNOB_ID,
        label: "UI / Info",
        group: ThemeColorKnobGroup::Ui,
        get: get_ui_info,
        apply: apply_ui_info,
    },
];

pub fn theme_color_knob_by_id(id: ThemeColorKnobId) -> Option<&'static ThemeColorKnob> {
    THEME_COLOR_KNOBS.iter().find(|knob| knob.id == id)
}

pub fn theme_color_knob_id_from_str(value: &str) -> Option<ThemeColorKnobId> {
    let normalized = value
        .strip_prefix("control:dev-style-tool-theme:")
        .or_else(|| value.strip_prefix("input:dev-style-tool-theme:"))
        .or_else(|| value.strip_prefix("button:dev-style-tool-theme-reset:"))
        .unwrap_or(value);
    THEME_COLOR_KNOBS
        .iter()
        .find(|knob| knob.id.as_str() == normalized)
        .map(|knob| knob.id)
}

/// Format a theme color as the canonical `#RRGGBB` hex string used by the
/// Theme Inspector inputs and the style export.
pub fn format_theme_color_hex(value: HexColor) -> String {
    format!("#{:06X}", value & 0xFF_FF_FF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_color_knob_ids_are_unique_and_resolvable() {
        for (index, knob) in THEME_COLOR_KNOBS.iter().enumerate() {
            assert!(
                THEME_COLOR_KNOBS
                    .iter()
                    .skip(index + 1)
                    .all(|other| other.id != knob.id),
                "duplicate theme color knob id: {}",
                knob.id.as_str()
            );
            assert_eq!(
                theme_color_knob_by_id(knob.id).map(|found| found.id),
                Some(knob.id)
            );
            assert_eq!(
                theme_color_knob_id_from_str(knob.id.as_str()),
                Some(knob.id)
            );
        }
    }

    #[test]
    fn theme_color_knob_id_from_str_strips_semantic_control_prefixes() {
        assert_eq!(
            theme_color_knob_id_from_str("input:dev-style-tool-theme:theme.colors.text.primary"),
            Some(THEME_TEXT_PRIMARY_KNOB_ID)
        );
        assert_eq!(
            theme_color_knob_id_from_str(
                "button:dev-style-tool-theme-reset:theme.colors.ui.border"
            ),
            Some(THEME_UI_BORDER_KNOB_ID)
        );
        assert_eq!(theme_color_knob_id_from_str("theme.colors.nope"), None);
    }

    #[test]
    fn theme_color_knob_get_apply_round_trips_on_default_theme() {
        let mut theme = Theme::default();
        for knob in THEME_COLOR_KNOBS {
            (knob.apply)(&mut theme, 0x123456);
            assert_eq!(
                (knob.get)(&theme),
                0x123456,
                "knob {} should round-trip",
                knob.id.as_str()
            );
        }
    }

    #[test]
    fn format_theme_color_hex_emits_canonical_rgb() {
        assert_eq!(format_theme_color_hex(0x000000), "#000000");
        assert_eq!(format_theme_color_hex(0xFBBF24), "#FBBF24");
        assert_eq!(format_theme_color_hex(0x00FF00), "#00FF00");
    }
}

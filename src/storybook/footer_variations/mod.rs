//! Footer variation registry backed by the real PromptFooter component.
//!
//! Provides a single source of truth for footer design variations that the
//! storybook compare mode renders and that agents can target for adoption.

use gpui::*;

use crate::components::prompt_footer::{PromptFooter, PromptFooterColors, PromptFooterConfig};
use crate::ui_foundation::HexColorExt;

use super::StoryVariant;

/// Identifies a footer design variation with a stable string ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FooterVariationId {
    RaycastExact,
    ScriptKitBranded,
    Minimal,
    StatusBar,
    Invisible,
}

impl FooterVariationId {
    pub const ALL: [Self; 5] = [
        Self::RaycastExact,
        Self::ScriptKitBranded,
        Self::Minimal,
        Self::StatusBar,
        Self::Invisible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RaycastExact => "raycast-exact",
            Self::ScriptKitBranded => "scriptkit-branded",
            Self::Minimal => "minimal",
            Self::StatusBar => "status-bar",
            Self::Invisible => "invisible",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::RaycastExact => "Raycast Exact",
            Self::ScriptKitBranded => "Script Kit Branded",
            Self::Minimal => "Minimal",
            Self::StatusBar => "Status Bar",
            Self::Invisible => "Invisible",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::RaycastExact => "Logo left, primary action and Actions shortcut on the right",
            Self::ScriptKitBranded => "Script Kit footer with helper text and info label",
            Self::Minimal => "Hint-strip treatment with no footer buttons",
            Self::StatusBar => "Left status text with right-aligned key hints",
            Self::Invisible => "No footer at all",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "raycast-exact" => Some(Self::RaycastExact),
            "scriptkit-branded" => Some(Self::ScriptKitBranded),
            "minimal" => Some(Self::Minimal),
            "status-bar" => Some(Self::StatusBar),
            "invisible" => Some(Self::Invisible),
            _ => None,
        }
    }
}

/// Declarative spec for a single footer variation, driving PromptFooter config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterVariationSpec {
    pub id: FooterVariationId,
    pub show_logo: bool,
    pub show_primary: bool,
    pub show_secondary: bool,
    pub show_info_label: bool,
    pub primary_label: &'static str,
    pub primary_shortcut: &'static str,
    pub secondary_label: &'static str,
    pub secondary_shortcut: &'static str,
    pub helper_text: Option<&'static str>,
    pub info_label: Option<&'static str>,
    pub left_slot_text: Option<&'static str>,
    pub right_slot_text: Option<&'static str>,
}

const FOOTER_VARIATIONS: [FooterVariationSpec; 5] = [
    FooterVariationSpec {
        id: FooterVariationId::RaycastExact,
        show_logo: true,
        show_primary: true,
        show_secondary: true,
        show_info_label: false,
        primary_label: "Open Application",
        primary_shortcut: "↵",
        secondary_label: "Actions",
        secondary_shortcut: "⌘K",
        helper_text: None,
        info_label: None,
        left_slot_text: None,
        right_slot_text: None,
    },
    FooterVariationSpec {
        id: FooterVariationId::ScriptKitBranded,
        show_logo: true,
        show_primary: true,
        show_secondary: true,
        show_info_label: true,
        primary_label: "Run Script",
        primary_shortcut: "↵",
        secondary_label: "Actions",
        secondary_shortcut: "⌘K",
        helper_text: Some("Tab AI"),
        info_label: Some("Built-in"),
        left_slot_text: None,
        right_slot_text: None,
    },
    FooterVariationSpec {
        id: FooterVariationId::Minimal,
        show_logo: false,
        show_primary: false,
        show_secondary: false,
        show_info_label: false,
        primary_label: "Run Script",
        primary_shortcut: "↵",
        secondary_label: "Actions",
        secondary_shortcut: "⌘K",
        helper_text: None,
        info_label: None,
        left_slot_text: None,
        right_slot_text: Some("↵ Open   ⌘K Actions   Tab AI"),
    },
    FooterVariationSpec {
        id: FooterVariationId::StatusBar,
        show_logo: true,
        show_primary: false,
        show_secondary: false,
        show_info_label: false,
        primary_label: "Run Script",
        primary_shortcut: "↵",
        secondary_label: "Actions",
        secondary_shortcut: "⌘K",
        helper_text: None,
        info_label: None,
        left_slot_text: Some("Ready"),
        right_slot_text: Some("⌘K Actions  •  Esc Close"),
    },
    FooterVariationSpec {
        id: FooterVariationId::Invisible,
        show_logo: false,
        show_primary: false,
        show_secondary: false,
        show_info_label: false,
        primary_label: "Run Script",
        primary_shortcut: "↵",
        secondary_label: "Actions",
        secondary_shortcut: "⌘K",
        helper_text: None,
        info_label: None,
        left_slot_text: None,
        right_slot_text: None,
    },
];

/// Returns the full set of footer variation specs (stable order).
pub fn footer_variation_specs() -> &'static [FooterVariationSpec] {
    &FOOTER_VARIATIONS
}

/// Converts every spec into a `StoryVariant` with semantic props.
pub fn footer_story_variants() -> Vec<StoryVariant> {
    footer_variation_specs()
        .iter()
        .map(|spec| {
            let mut variant = StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "footer")
                .with_prop("variantId", spec.id.as_str())
                .with_prop("showLogo", if spec.show_logo { "true" } else { "false" })
                .with_prop(
                    "showPrimary",
                    if spec.show_primary { "true" } else { "false" },
                )
                .with_prop(
                    "showSecondary",
                    if spec.show_secondary { "true" } else { "false" },
                );

            if let Some(info_label) = spec.info_label {
                variant = variant.with_prop("infoLabel", info_label);
            }
            if let Some(helper_text) = spec.helper_text {
                variant = variant.with_prop("helperText", helper_text);
            }
            if let Some(left_slot_text) = spec.left_slot_text {
                variant = variant.with_prop("leftSlot", left_slot_text);
            }
            if let Some(right_slot_text) = spec.right_slot_text {
                variant = variant.with_prop("rightSlot", right_slot_text);
            }

            variant
        })
        .collect()
}

/// Renders a real `PromptFooter`-backed preview for the given stable ID.
///
/// Non-invisible variations render inside a mock window shell so the footer
/// appears at realistic proportions. The invisible variation renders an
/// empty shell with no footer.
pub fn render_footer_story_preview(stable_id: &str) -> AnyElement {
    let spec = footer_variation_specs()
        .iter()
        .find(|s| s.id.as_str() == stable_id)
        .copied()
        .unwrap_or(FOOTER_VARIATIONS[0]);

    tracing::info!(
        footer_variation_id = spec.id.as_str(),
        footer_variation_name = spec.id.name(),
        "render_footer_story_preview"
    );

    let theme = crate::theme::get_cached_theme();

    let mut shell = div()
        .w(px(420.))
        .h(px(220.))
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(theme.colors.ui.border.to_rgb())
        .bg(theme.colors.background.main.to_rgb())
        .overflow_hidden()
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .p_4()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child("Sample results"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.colors.text.muted.to_rgb())
                        .child("Preview body stays fake; footer component is real."),
                )
                .child(
                    div()
                        .mt_2()
                        .rounded(px(8.))
                        .bg(theme.colors.background.title_bar.to_rgb())
                        .p_3()
                        .text_sm()
                        .text_color(theme.colors.text.secondary.to_rgb())
                        .child("Clipboard History"),
                ),
        );

    if spec.id != FooterVariationId::Invisible {
        shell = shell.child(render_footer_component(spec));
    }

    shell.into_any_element()
}

/// Build a `PromptFooterConfig` from a `FooterVariationSpec`.
///
/// This is the single conversion path used by both story previews and runtime
/// adoption. Keeping it here avoids a `crate::storybook` dependency in
/// `prompt_footer.rs` (which is compiled in both the lib and binary crates).
pub fn config_from_footer_variation_spec(spec: &FooterVariationSpec) -> PromptFooterConfig {
    let mut config = PromptFooterConfig::new()
        .primary_label(spec.primary_label)
        .primary_shortcut(spec.primary_shortcut)
        .secondary_label(spec.secondary_label)
        .secondary_shortcut(spec.secondary_shortcut)
        .show_logo(spec.show_logo)
        .show_primary(spec.show_primary)
        .show_secondary(spec.show_secondary)
        .show_info_label(spec.show_info_label);

    if let Some(helper_text) = spec.helper_text {
        config = config.helper_text(helper_text);
    }
    if let Some(info_label) = spec.info_label {
        config = config.info_label(info_label);
    }

    config
}

/// Structured result of resolving a footer selection, exposing fallback status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FooterSelectionResolution {
    pub requested_variant_id: Option<String>,
    pub resolved_variant_id: String,
    pub fallback_used: bool,
}

/// Resolve a persisted footer selection value into a concrete spec and metadata.
pub fn resolve_footer_selection_spec(
    selected: Option<&str>,
) -> (&'static FooterVariationSpec, FooterSelectionResolution) {
    let requested_variant_id = selected.map(str::to_owned);
    let resolved_from_input = selected.and_then(FooterVariationId::from_stable_id);
    let variation = resolved_from_input.unwrap_or(FooterVariationId::RaycastExact);

    let spec = footer_variation_specs()
        .iter()
        .find(|s| s.id == variation)
        .unwrap_or(&footer_variation_specs()[0]);

    (
        spec,
        FooterSelectionResolution {
            requested_variant_id,
            resolved_variant_id: spec.id.as_str().to_string(),
            fallback_used: selected.is_some() && resolved_from_input.is_none(),
        },
    )
}

/// Resolve a persisted footer selection value into a config and structured resolution.
///
/// Returns the `PromptFooterConfig` for the requested variant (or the default
/// `RaycastExact` when the ID is `None` or unrecognised) together with a
/// [`FooterSelectionResolution`] that records whether a fallback was used.
pub fn resolve_footer_selection(
    selected: Option<&str>,
) -> (PromptFooterConfig, FooterSelectionResolution) {
    let (spec, resolution) = resolve_footer_selection_spec(selected);
    (config_from_footer_variation_spec(spec), resolution)
}

/// Build a `PromptFooterConfig` from a persisted storybook footer selection value.
///
/// Looks up the given stable ID in the footer variation registry and maps
/// the matching spec into a config. Falls back to the default
/// `RaycastExact` spec when the ID is `None` or unrecognised.
pub fn config_from_storybook_footer_selection_value(selected: Option<&str>) -> PromptFooterConfig {
    resolve_footer_selection(selected).0
}

/// Build a `PromptFooterConfig` from the on-disk storybook footer selection.
///
/// Reads the persisted `design-explorer-selections.json` for the
/// `"footer-layout-variations"` story and resolves via
/// [`config_from_storybook_footer_selection_value`].
pub fn config_from_storybook_footer_selection() -> PromptFooterConfig {
    let selected = super::load_selected_story_variant("footer-layout-variations");

    config_from_storybook_footer_selection_value(selected.as_deref())
}

fn render_footer_component(spec: FooterVariationSpec) -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let colors = PromptFooterColors::from_theme(&theme);

    let config = config_from_footer_variation_spec(&spec);

    let mut footer = PromptFooter::new(config, colors);

    if let Some(left_slot_text) = spec.left_slot_text {
        footer = footer.left_slot(render_footer_slot_text(left_slot_text, true));
    }
    if let Some(right_slot_text) = spec.right_slot_text {
        footer = footer.right_slot(render_footer_slot_text(right_slot_text, false));
    }

    footer.into_any_element()
}

pub fn render_footer_slot_text(text: &'static str, is_left: bool) -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let color = if is_left {
        theme.colors.text.muted
    } else {
        theme.colors.text.dimmed
    };
    div()
        .text_xs()
        .font_weight(if is_left {
            FontWeight::MEDIUM
        } else {
            FontWeight::NORMAL
        })
        .text_color(color.to_rgb())
        .child(text)
        .into_any_element()
}

#[cfg(test)]
mod tests;

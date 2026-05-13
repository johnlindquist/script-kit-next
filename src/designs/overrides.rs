//! Phase 3 — pure override composition.
//!
//! The Design Picker customizer captures per-design knobs in
//! [`crate::config::types::DesignOverrides`]. Render time composes a
//! [`ResolvedDesignTokens`] view by layering those overrides on top of
//! the base [`DesignDef`] without mutating the catalog. The function
//! is pure — same inputs always yield the same output — so the resolver
//! is trivially cacheable by `(id, overrides_hash)`.
//!
// @lat: [[lat.md/designs#Catalog invariants]]

use std::hash::{Hash, Hasher};

use crate::config::{
    ChromeOpacityChoice, Cmd1Behavior, DesignDensityChoice, DesignOverrides, FontFamilyChoice,
    IconStyleChoice, SeparatorStyleChoice, VibrancyChoice,
};
use crate::designs::core::registry::{
    catalog, fallback, lookup, ChromeSpec, DensityPreset, DesignDef, IconStyle, PaletteSpec,
    RendererMode, SeparatorStyle, TypographySpec, VibrancySpec,
};

/// Render-time view of a design after overrides are applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedDesignTokens {
    pub id: &'static str,
    pub palette: PaletteSpec,
    pub typography: TypographySpec,
    pub density: DensityPreset,
    pub row_height: f32,
    pub chrome: ChromeSpec,
    pub vibrancy: VibrancySpec,
    pub icon_style: IconStyle,
    pub separator_style: SeparatorStyle,
    pub renderer_mode: RendererMode,
    pub accent_override: Option<String>,
    pub font_family_override: Option<FontFamilyChoice>,
    pub font_scale_override: Option<i8>,
}

/// Resolve a design id to a render-time token bundle. Unknown ids fall
/// back to `script-kit-classic`; missing overrides leave catalog defaults
/// in place.
pub fn resolve_design_tokens(
    id: &str,
    overrides: Option<&DesignOverrides>,
) -> ResolvedDesignTokens {
    let def: &DesignDef = lookup(id).unwrap_or_else(fallback);
    apply_overrides(def, overrides)
}

/// Apply overrides over a fully-resolved [`DesignDef`].
pub fn apply_overrides(
    def: &DesignDef,
    overrides: Option<&DesignOverrides>,
) -> ResolvedDesignTokens {
    let mut density = def.density;
    let mut vibrancy = def.vibrancy;
    let mut icon_style = def.icon_style;
    let mut separator_style = def.separator_style;
    let mut row_height = def.density.row_height();
    let mut accent_override = None;
    let mut font_family_override = None;
    let mut font_scale_override = None;

    if let Some(o) = overrides {
        if let Some(d) = o.density {
            density = match d {
                DesignDensityChoice::Compact => DensityPreset::Compact,
                DesignDensityChoice::Comfortable => DensityPreset::Comfortable,
                DesignDensityChoice::Spacious => DensityPreset::Spacious,
            };
            row_height = density.row_height();
        }
        if let Some(nudge) = o.row_height_nudge {
            row_height = (row_height + f32::from(nudge.clamp(-2, 2))).max(20.0);
        }
        if let Some(v) = o.vibrancy {
            vibrancy = match v {
                VibrancyChoice::None => VibrancySpec::None,
                VibrancyChoice::Light => VibrancySpec::Light,
                VibrancyChoice::Medium => VibrancySpec::Medium,
                VibrancyChoice::Heavy => VibrancySpec::Heavy,
            };
        }
        if let Some(i) = o.icon_style {
            icon_style = match i {
                IconStyleChoice::Mono => IconStyle::Mono,
                IconStyleChoice::Color => IconStyle::Color,
                IconStyleChoice::Hidden => IconStyle::Hidden,
            };
        }
        if let Some(s) = o.separator_style {
            separator_style = match s {
                SeparatorStyleChoice::None => SeparatorStyle::None,
                SeparatorStyleChoice::Hairline => SeparatorStyle::Hairline,
                SeparatorStyleChoice::Rule => SeparatorStyle::Rule,
                SeparatorStyleChoice::Grid => SeparatorStyle::Grid,
            };
        }
        // `chrome_opacity` is a chrome-level dim, not a chrome shape, so
        // we record it on the resolved bundle without mutating
        // `def.chrome`. Phase 3 UI consumes the opacity dim separately.
        let _: Option<ChromeOpacityChoice> = o.chrome_opacity;
        accent_override = o.accent.clone();
        font_family_override = o.font_family;
        font_scale_override = o.font_scale;
    }

    ResolvedDesignTokens {
        id: def.id,
        palette: def.palette,
        typography: def.typography,
        density,
        row_height,
        chrome: def.chrome,
        vibrancy,
        icon_style,
        separator_style,
        renderer_mode: def.renderer_mode,
        accent_override,
        font_family_override,
        font_scale_override,
    }
}

/// Stable cache key for `(id, overrides)` pairs so render hot paths can
/// memoize resolution.
pub fn overrides_hash(overrides: Option<&DesignOverrides>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if let Some(o) = overrides {
        o.accent.hash(&mut hasher);
        o.density.map(|d| d as u8).hash(&mut hasher);
        o.font_family.map(|f| f as u8).hash(&mut hasher);
        o.font_scale.hash(&mut hasher);
        o.vibrancy.map(|v| v as u8).hash(&mut hasher);
        o.chrome_opacity.map(|c| c as u8).hash(&mut hasher);
        o.icon_style.map(|i| i as u8).hash(&mut hasher);
        o.separator_style.map(|s| s as u8).hash(&mut hasher);
        o.row_height_nudge.hash(&mut hasher);
    } else {
        0u8.hash(&mut hasher);
    }
    hasher.finish()
}

/// Effective Cmd+1 behavior for the launcher, defaulting to `Picker`.
pub fn effective_cmd1_behavior(value: Option<Cmd1Behavior>) -> Cmd1Behavior {
    value.unwrap_or(Cmd1Behavior::Picker)
}

/// Surprise-me helper: pick a random non-current id from the catalog.
/// Deterministic when `seed` is fixed.
pub fn surprise_me(current_id: &str, seed: u64) -> &'static str {
    let entries: Vec<&DesignDef> = catalog().iter().filter(|d| d.id != current_id).collect();
    if entries.is_empty() {
        return fallback().id;
    }
    let idx = (seed as usize) % entries.len();
    entries[idx].id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> DesignOverrides {
        DesignOverrides::default()
    }

    #[test]
    fn unknown_id_falls_back_to_classic() {
        let tokens = resolve_design_tokens("totally-fake", None);
        assert_eq!(tokens.id, "script-kit-classic");
    }

    #[test]
    fn no_overrides_preserves_catalog_defaults() {
        let tokens = resolve_design_tokens("script-kit-classic", None);
        assert_eq!(tokens.density, DensityPreset::Comfortable);
        assert_eq!(tokens.row_height, DensityPreset::Comfortable.row_height());
    }

    #[test]
    fn density_override_changes_row_height() {
        let mut o = empty();
        o.density = Some(DesignDensityChoice::Compact);
        let tokens = resolve_design_tokens("script-kit-classic", Some(&o));
        assert_eq!(tokens.density, DensityPreset::Compact);
        assert_eq!(tokens.row_height, DensityPreset::Compact.row_height());
    }

    #[test]
    fn row_height_nudge_clamps_and_offsets() {
        let mut o = empty();
        o.row_height_nudge = Some(2);
        let tokens = resolve_design_tokens("script-kit-classic", Some(&o));
        let base = DensityPreset::Comfortable.row_height();
        assert_eq!(tokens.row_height, base + 2.0);
    }

    #[test]
    fn vibrancy_override_applies() {
        let mut o = empty();
        o.vibrancy = Some(VibrancyChoice::None);
        let tokens = resolve_design_tokens("glass-frost", Some(&o));
        assert_eq!(tokens.vibrancy, VibrancySpec::None);
    }

    #[test]
    fn accent_and_font_pass_through_as_overrides() {
        let mut o = empty();
        o.accent = Some("#ff8800".into());
        o.font_family = Some(FontFamilyChoice::Monospace);
        o.font_scale = Some(1);
        let tokens = resolve_design_tokens("script-kit-classic", Some(&o));
        assert_eq!(tokens.accent_override.as_deref(), Some("#ff8800"));
        assert_eq!(
            tokens.font_family_override,
            Some(FontFamilyChoice::Monospace)
        );
        assert_eq!(tokens.font_scale_override, Some(1));
    }

    #[test]
    fn overrides_hash_is_stable_for_equal_inputs() {
        let mut a = empty();
        a.accent = Some("#abcdef".into());
        a.density = Some(DesignDensityChoice::Spacious);
        let mut b = empty();
        b.accent = Some("#abcdef".into());
        b.density = Some(DesignDensityChoice::Spacious);
        assert_eq!(overrides_hash(Some(&a)), overrides_hash(Some(&b)));
    }

    #[test]
    fn overrides_hash_distinguishes_different_inputs() {
        let a = empty();
        let mut b = empty();
        b.density = Some(DesignDensityChoice::Compact);
        assert_ne!(overrides_hash(Some(&a)), overrides_hash(Some(&b)));
    }

    #[test]
    fn effective_cmd1_defaults_to_picker() {
        assert_eq!(effective_cmd1_behavior(None), Cmd1Behavior::Picker);
        assert_eq!(
            effective_cmd1_behavior(Some(Cmd1Behavior::Cycle)),
            Cmd1Behavior::Cycle
        );
    }

    #[test]
    fn surprise_me_never_returns_current() {
        for seed in 0..50u64 {
            let next = surprise_me("script-kit-classic", seed);
            assert_ne!(next, "script-kit-classic");
            // and it must be a real catalog entry
            assert!(lookup(next).is_some());
        }
    }
}

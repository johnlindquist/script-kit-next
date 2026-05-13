//! Curated design catalog.
//!
//! The catalog is the single source of truth for every design id, its
//! token signature, and the renderer mode it dispatches to. The legacy
//! [`DesignVariant`] enum stays in place as a migration shim; production
//! code should consume designs by id via [`lookup`] and [`fallback`].
//!
// @lat: [[lat.md/designs#Catalog invariants]]

use std::hash::{Hash, Hasher};

/// Stable kebab-case id for a curated design entry.
pub type DesignId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaletteSpec {
    ThemeNeutral,
    DesaturatedTheme,
    HighContrast,
    InkOnPaper,
    PhosphorGreen,
    PhosphorAmber,
    WarmCream,
    EditorialMono,
    BrutalistGrid,
    FrostedGlass,
    GlassCompact,
    NeonAccent,
    SynthwaveGradient,
    Material3Tonal,
    AppleSystem,
    Mocha,
    OceanDeep,
    PastelMist,
    PlayfulVibrant,
    MonoContrast,
    CommandCenter,
    GalleryVisual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypographySpec {
    SystemUi,
    SystemUiCompact,
    Tabular,
    SerifMeta,
    Monospace,
    EditorialDisplay,
    SfHig,
    BoldHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DensityPreset {
    UltraCompact,
    Compact,
    Comfortable,
    Spacious,
    Gallery,
}

impl DensityPreset {
    pub const fn row_height(self) -> f32 {
        match self {
            DensityPreset::UltraCompact => 26.0,
            DensityPreset::Compact => 32.0,
            DensityPreset::Comfortable => 40.0,
            DensityPreset::Spacious => 48.0,
            DensityPreset::Gallery => 52.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromeSpec {
    Whisper,
    Flat,
    SoftHairline,
    ThickRule,
    Elevated,
    GlowOnSelected,
    GridGutters,
    NoFills,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VibrancySpec {
    None,
    Light,
    Medium,
    Heavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconStyle {
    Mono,
    Color,
    Hidden,
    Prominent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeparatorStyle {
    None,
    Hairline,
    Rule,
    Grid,
    ThickRule,
}

/// Which low-level renderer this design dispatches to. The catalog must
/// stay narrow here: 25 designs share a handful of renderer modes, with
/// per-design palette/typography/density driving the visible difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RendererMode {
    Default,
    Minimal,
    RetroTerminal,
    Glass,
    Brutalist,
    NeonCyber,
    Paper,
    AppleHig,
    Material,
    Playful,
    Gallery,
}

/// A single curated design entry. Every field is intentional — see
/// `lat.md/designs.md#Catalog invariants` for the uniqueness rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DesignDef {
    pub id: DesignId,
    pub name: &'static str,
    pub description: &'static str,
    pub palette: PaletteSpec,
    pub typography: TypographySpec,
    pub density: DensityPreset,
    pub chrome: ChromeSpec,
    pub vibrancy: VibrancySpec,
    pub icon_style: IconStyle,
    pub separator_style: SeparatorStyle,
    pub renderer_mode: RendererMode,
}

impl DesignDef {
    /// Token signature used by the uniqueness invariant. Two catalog
    /// entries may not share the same signature — the registry test
    /// enforces this.
    pub fn signature(&self) -> DesignSignature {
        DesignSignature {
            palette: self.palette,
            typography: self.typography,
            density: self.density,
            chrome: self.chrome,
            vibrancy: self.vibrancy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DesignSignature {
    pub palette: PaletteSpec,
    pub typography: TypographySpec,
    pub density: DensityPreset,
    pub chrome: ChromeSpec,
    pub vibrancy: VibrancySpec,
}

/// Phase 1 catalog (10 curated designs). Phase 2 expands to 25.
pub static CATALOG: &[DesignDef] = &[
    DesignDef {
        id: "script-kit-classic",
        name: "Script Kit Classic",
        description: "The current sane default, cleaned up and tokenized.",
        palette: PaletteSpec::ThemeNeutral,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::Whisper,
        vibrancy: VibrancySpec::Medium,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "pro-dense",
        name: "Pro Dense",
        description: "Maximum information density without terminal cosplay.",
        palette: PaletteSpec::ThemeNeutral,
        typography: TypographySpec::SystemUiCompact,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::Flat,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "ambient-quiet",
        name: "Ambient Quiet",
        description: "Low-noise launcher that recedes into the desktop.",
        palette: PaletteSpec::DesaturatedTheme,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::SoftHairline,
        vibrancy: VibrancySpec::Medium,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "focus-zen",
        name: "Focus Zen",
        description: "Distraction-free with generous breathing room and strong selected state.",
        palette: PaletteSpec::ThemeNeutral,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Spacious,
        chrome: ChromeSpec::SoftHairline,
        vibrancy: VibrancySpec::Light,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::None,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "minimal-ink",
        name: "Minimal Ink",
        description: "Stripped chrome, ink-on-paper feel.",
        palette: PaletteSpec::InkOnPaper,
        typography: TypographySpec::SerifMeta,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::NoFills,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Minimal,
    },
    DesignDef {
        id: "retro-terminal",
        name: "Retro Terminal",
        description: "Phosphor green on black, monospace, terminal cosplay done well.",
        palette: PaletteSpec::PhosphorGreen,
        typography: TypographySpec::Monospace,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::Flat,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::None,
        renderer_mode: RendererMode::RetroTerminal,
    },
    DesignDef {
        id: "paper-print",
        name: "Paper Print",
        description: "Warm paper, printed-page feel.",
        palette: PaletteSpec::WarmCream,
        typography: TypographySpec::SerifMeta,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::SoftHairline,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::Rule,
        renderer_mode: RendererMode::Paper,
    },
    DesignDef {
        id: "glass-frost",
        name: "Glass Frost",
        description: "Frosted glass with theme-aware blur.",
        palette: PaletteSpec::FrostedGlass,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::Elevated,
        vibrancy: VibrancySpec::Heavy,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Glass,
    },
    DesignDef {
        id: "neon-cyber",
        name: "Neon Cyber",
        description: "Dark canvas with vibrant neon accents.",
        palette: PaletteSpec::NeonAccent,
        typography: TypographySpec::Monospace,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::GlowOnSelected,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::None,
        renderer_mode: RendererMode::NeonCyber,
    },
    DesignDef {
        id: "apple-hig",
        name: "Apple HIG",
        description: "Aligned with macOS HIG spacing and typography.",
        palette: PaletteSpec::AppleSystem,
        typography: TypographySpec::SfHig,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::Whisper,
        vibrancy: VibrancySpec::Light,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::AppleHig,
    },
    DesignDef {
        id: "high-density-list",
        name: "High Density List",
        description: "Narrow, table-like launcher for fast keyboard selection.",
        palette: PaletteSpec::ThemeNeutral,
        typography: TypographySpec::Tabular,
        density: DensityPreset::UltraCompact,
        chrome: ChromeSpec::Flat,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Hidden,
        separator_style: SeparatorStyle::Grid,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "accessibility-high-contrast",
        name: "Accessibility High Contrast",
        description: "Max-contrast palette and large hit targets for AX needs.",
        palette: PaletteSpec::HighContrast,
        typography: TypographySpec::BoldHigh,
        density: DensityPreset::Spacious,
        chrome: ChromeSpec::ThickRule,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::ThickRule,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "retro-amber",
        name: "Retro Amber",
        description: "Amber CRT cousin to retro-terminal with warmer phosphor.",
        palette: PaletteSpec::PhosphorAmber,
        typography: TypographySpec::Monospace,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::Flat,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::None,
        renderer_mode: RendererMode::RetroTerminal,
    },
    DesignDef {
        id: "editorial-brutalist",
        name: "Editorial Brutalist",
        description: "Bold raw typography, strong contrast, editorial spacing.",
        palette: PaletteSpec::EditorialMono,
        typography: TypographySpec::EditorialDisplay,
        density: DensityPreset::Spacious,
        chrome: ChromeSpec::ThickRule,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Hidden,
        separator_style: SeparatorStyle::ThickRule,
        renderer_mode: RendererMode::Brutalist,
    },
    DesignDef {
        id: "brutalist-grid",
        name: "Brutalist Grid",
        description: "Brutalism with strict grid gutters and visible structure.",
        palette: PaletteSpec::BrutalistGrid,
        typography: TypographySpec::Monospace,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::GridGutters,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::Grid,
        renderer_mode: RendererMode::Brutalist,
    },
    DesignDef {
        id: "liquid-glass-compact",
        name: "Liquid Glass Compact",
        description: "Glass density variant with small rows and subtle highlights.",
        palette: PaletteSpec::GlassCompact,
        typography: TypographySpec::SystemUiCompact,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::Elevated,
        vibrancy: VibrancySpec::Heavy,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Glass,
    },
    DesignDef {
        id: "synthwave",
        name: "Synthwave",
        description: "Magenta/cyan gradient siblings to neon-cyber, more saturated.",
        palette: PaletteSpec::SynthwaveGradient,
        typography: TypographySpec::Monospace,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::GlowOnSelected,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::None,
        renderer_mode: RendererMode::NeonCyber,
    },
    DesignDef {
        id: "material-you",
        name: "Material You",
        description: "Material 3 inspired with rounded fills and elevation.",
        palette: PaletteSpec::Material3Tonal,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::Elevated,
        vibrancy: VibrancySpec::Light,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Material,
    },
    DesignDef {
        id: "mocha-warm",
        name: "Mocha Warm",
        description: "Warm brown/cream palette, low contrast.",
        palette: PaletteSpec::Mocha,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::SoftHairline,
        vibrancy: VibrancySpec::Light,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "ocean-deep",
        name: "Ocean Deep",
        description: "Deep blue-green palette with subtle depth.",
        palette: PaletteSpec::OceanDeep,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::GlowOnSelected,
        vibrancy: VibrancySpec::Light,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "pastel-mist",
        name: "Pastel Mist",
        description: "Low-chroma pastel palette, light-mode default.",
        palette: PaletteSpec::PastelMist,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::SoftHairline,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "playful-pop",
        name: "Playful Pop",
        description: "Rounded corners, vibrant accents, friendly.",
        palette: PaletteSpec::PlayfulVibrant,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::Elevated,
        vibrancy: VibrancySpec::Medium,
        icon_style: IconStyle::Color,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Playful,
    },
    DesignDef {
        id: "mono-contrast",
        name: "Mono Contrast",
        description: "Monochrome with maximum contrast accents only on state.",
        palette: PaletteSpec::MonoContrast,
        typography: TypographySpec::BoldHigh,
        density: DensityPreset::Comfortable,
        chrome: ChromeSpec::NoFills,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Minimal,
    },
    DesignDef {
        id: "command-center",
        name: "Command Center",
        description: "DevTools-inspector-like, debug-friendly meta.",
        palette: PaletteSpec::CommandCenter,
        typography: TypographySpec::Monospace,
        density: DensityPreset::Compact,
        chrome: ChromeSpec::Flat,
        vibrancy: VibrancySpec::None,
        icon_style: IconStyle::Mono,
        separator_style: SeparatorStyle::Hairline,
        renderer_mode: RendererMode::Default,
    },
    DesignDef {
        id: "gallery-visual",
        name: "Gallery Visual",
        description: "Larger row + visible icon swatch per item.",
        palette: PaletteSpec::GalleryVisual,
        typography: TypographySpec::SystemUi,
        density: DensityPreset::Gallery,
        chrome: ChromeSpec::Elevated,
        vibrancy: VibrancySpec::Medium,
        icon_style: IconStyle::Prominent,
        separator_style: SeparatorStyle::None,
        renderer_mode: RendererMode::Gallery,
    },
];

/// Canonical fallback id. Loaders must use this when an id is unknown.
pub const FALLBACK_ID: DesignId = "script-kit-classic";

pub fn catalog() -> &'static [DesignDef] {
    CATALOG
}

pub fn lookup(id: &str) -> Option<&'static DesignDef> {
    CATALOG.iter().find(|d| d.id == id)
}

pub fn fallback() -> &'static DesignDef {
    lookup(FALLBACK_ID).expect("FALLBACK_ID must exist in CATALOG")
}

/// Resolve an id or return the fallback. Logs a warning when the input is
/// non-empty and unknown so loaders can surface migration issues.
pub fn resolve_or_fallback(id: Option<&str>) -> &'static DesignDef {
    match id {
        Some(s) if !s.is_empty() => lookup(s).unwrap_or_else(|| {
            crate::logging::log(
                "DESIGNS",
                &format!(
                    "design id `{}` not in catalog; falling back to `{}`",
                    s, FALLBACK_ID
                ),
            );
            fallback()
        }),
        _ => fallback(),
    }
}

/// Hash-stable signature value (used by tests for uniqueness).
pub fn signature_hash(sig: &DesignSignature) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    sig.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn fallback_is_in_catalog() {
        assert!(lookup(FALLBACK_ID).is_some(), "fallback must exist");
    }

    #[test]
    fn catalog_ids_are_unique() {
        let mut seen = HashSet::new();
        for d in CATALOG {
            assert!(seen.insert(d.id), "duplicate id in CATALOG: {}", d.id);
        }
    }

    #[test]
    fn catalog_signatures_are_unique_phase1() {
        let mut seen: HashSet<u64> = HashSet::new();
        for d in CATALOG {
            let h = signature_hash(&d.signature());
            assert!(
                seen.insert(h),
                "duplicate token signature for design `{}` — every catalog entry must change ≥2 dims",
                d.id
            );
        }
    }

    #[test]
    fn catalog_has_exactly_25_designs() {
        assert_eq!(
            CATALOG.len(),
            25,
            "catalog must contain exactly 25 curated designs"
        );
    }

    #[test]
    fn unknown_id_resolves_to_fallback() {
        let def = resolve_or_fallback(Some("does-not-exist"));
        assert_eq!(def.id, FALLBACK_ID);
    }

    #[test]
    fn none_id_resolves_to_fallback() {
        let def = resolve_or_fallback(None);
        assert_eq!(def.id, FALLBACK_ID);
    }
}

//! Accent variation explorer for the main menu.
//!
//! A live, runtime-cyclable knob for *how* the theme accent color is used in the
//! launcher — both the list **rows** and the native **footer** chrome. This is a
//! design-exploration tool (parallel to [`super::variant::DesignVariant`]), not
//! persisted theme state: cycle through the treatments with `alt+left` /
//! `alt+right` to pick a favorite.
//!
//! Every variation shares one baseline: the legacy left-edge selected-row accent
//! bar is gone. Each variation then applies the accent to one or more surfaces.
//! Variations come in two axes:
//!
//! - **Row axis** — how the *selected list row* uses the accent (see
//!   [`AccentVariation::row_kind`]). `ListItem::render` keys off the row kind.
//! - **Footer axis** — whether the native footer's label text, keycap borders,
//!   and/or top divider are tinted toward the accent (see
//!   [`AccentVariation::footer_text_accent`] and friends). The native footer
//!   reads the *global* current variation via [`current_accent_variation`]
//!   because the per-row variation is not threaded into the AppKit footer host.

use std::sync::atomic::{AtomicU8, Ordering};

/// One of thirteen ways to surface the theme accent in the main menu.
///
/// The first group keeps the user-preferred **Icon Tile** row treatment and
/// layers progressively more accent onto the **footer**. The remaining entries
/// are standalone row treatments. `FooterOnly` isolates the footer accent with
/// plain rows so the footer change can be judged on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum AccentVariation {
    /// Selected icon sits in a solid accent tile; footer untouched. (Preferred.)
    #[default]
    IconTile = 1,
    /// Icon-tile rows + footer label/hint text tinted to accent.
    IconTileFooterText = 2,
    /// Icon-tile rows + footer keycap/labelcap borders tinted to accent.
    IconTileFooterKeycaps = 3,
    /// Icon-tile rows + the footer's top divider line in accent.
    IconTileFooterDivider = 4,
    /// Icon-tile rows + full accent footer (text + keycaps + divider).
    IconTileFooterFull = 5,
    /// Plain rows, full accent footer — isolates the footer treatment.
    FooterOnly = 6,
    /// Icon-tile rows + footer **button backgrounds** softly tinted to accent at
    /// rest, more on hover/active. Borders + text stay neutral.
    FooterButtonsSoft = 14,
    /// Same as `FooterButtonsSoft` with a medium-strength accent fill.
    FooterButtonsMedium = 15,
    /// Same as `FooterButtonsSoft` with a bold accent fill.
    FooterButtonsBold = 16,
    /// Selected row filled with a strong accent background; text flips to the
    /// on-accent contrast color.
    SolidFill = 7,
    /// Selected row's title and icon rendered in full bright accent.
    AccentText = 8,
    /// Selected row outlined with a thick accent border ring (+ light fill).
    Ring = 9,
    /// A chunky filled accent block leads the selected row.
    LeftBlock = 10,
    /// Every row's icon is accent-colored (selected brightest).
    AllIcons = 11,
    /// Selected title in bright accent with a thick accent underline.
    AccentName = 12,
    /// Loud: accent rows (fill + icon + title + badges) AND full accent footer.
    Loud = 13,
}

/// Accent alpha bytes (over the theme accent hue) for a footer button's three
/// interaction states. Borders/text are unaffected; only the button background
/// layer is tinted. See [`AccentVariation::footer_button_fill`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterButtonFill {
    /// Resting background alpha (subtle).
    pub rest: u32,
    /// Hovered background alpha (stronger).
    pub hover: u32,
    /// Active / selected background alpha (strongest).
    pub active: u32,
}

impl AccentVariation {
    /// Total number of accent variations.
    pub const COUNT: usize = 16;

    /// All variations in cycle order.
    pub fn all() -> &'static [AccentVariation] {
        use AccentVariation::*;
        &[
            IconTile,
            IconTileFooterText,
            IconTileFooterKeycaps,
            IconTileFooterDivider,
            IconTileFooterFull,
            FooterOnly,
            FooterButtonsSoft,
            FooterButtonsMedium,
            FooterButtonsBold,
            SolidFill,
            AccentText,
            Ring,
            LeftBlock,
            AllIcons,
            AccentName,
            Loud,
        ]
    }

    /// Reconstruct a variation from its `#[repr(u8)]` discriminant; unknown
    /// values fall back to the default ([`AccentVariation::IconTile`]).
    pub fn from_u8(value: u8) -> AccentVariation {
        Self::all()
            .iter()
            .copied()
            .find(|v| *v as u8 == value)
            .unwrap_or_default()
    }

    /// Next variation in the cycle (wraps).
    pub fn next(self) -> AccentVariation {
        let all = Self::all();
        let idx = all.iter().position(|&v| v == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    /// Previous variation in the cycle (wraps).
    pub fn prev(self) -> AccentVariation {
        let all = Self::all();
        let idx = all.iter().position(|&v| v == self).unwrap_or(0);
        let prev = if idx == 0 { all.len() - 1 } else { idx - 1 };
        all[prev]
    }

    /// Zero-based position of this variation within [`AccentVariation::all`].
    pub fn index(self) -> usize {
        Self::all().iter().position(|&v| v == self).unwrap_or(0)
    }

    /// The **row** treatment this variation maps to. Footer-combo variations
    /// reuse the `IconTile` row look; `ListItem::render` matches on this so the
    /// row rendering stays a small closed set independent of footer flags.
    pub fn row_kind(self) -> AccentVariation {
        use AccentVariation::*;
        match self {
            IconTile
            | IconTileFooterText
            | IconTileFooterKeycaps
            | IconTileFooterDivider
            | IconTileFooterFull
            | FooterButtonsSoft
            | FooterButtonsMedium
            | FooterButtonsBold => IconTile,
            other => other,
        }
    }

    /// Accent fill applied to the footer **button backgrounds** (rest / hover /
    /// active), or `None` when this variation leaves the button backgrounds at
    /// their neutral theme defaults. Borders and label text are unaffected — the
    /// button borders stay "normal" per the design intent.
    pub fn footer_button_fill(self) -> Option<FooterButtonFill> {
        use AccentVariation::*;
        match self {
            FooterButtonsSoft => Some(FooterButtonFill {
                rest: 0x0F,
                hover: 0x24,
                active: 0x36,
            }),
            FooterButtonsMedium => Some(FooterButtonFill {
                rest: 0x1C,
                hover: 0x36,
                active: 0x52,
            }),
            FooterButtonsBold => Some(FooterButtonFill {
                rest: 0x2B,
                hover: 0x52,
                active: 0x73,
            }),
            _ => None,
        }
    }

    /// True when the footer's label / hint text should be tinted to the accent.
    pub fn footer_text_accent(self) -> bool {
        use AccentVariation::*;
        matches!(
            self,
            IconTileFooterText | IconTileFooterFull | FooterOnly | Loud
        )
    }

    /// True when the footer's keycap / labelcap borders should be accent.
    pub fn footer_keycap_accent(self) -> bool {
        use AccentVariation::*;
        matches!(
            self,
            IconTileFooterKeycaps | IconTileFooterFull | FooterOnly | Loud
        )
    }

    /// True when the footer's top divider line should be accent.
    pub fn footer_divider_accent(self) -> bool {
        use AccentVariation::*;
        matches!(
            self,
            IconTileFooterDivider | IconTileFooterFull | FooterOnly | Loud
        )
    }

    /// True when this variation touches the footer at all (used to decide
    /// whether the footer needs to participate in its refresh signature).
    pub fn touches_footer(self) -> bool {
        self.footer_text_accent()
            || self.footer_keycap_accent()
            || self.footer_divider_accent()
            || self.footer_button_fill().is_some()
    }

    /// Human-readable name shown in the placeholder identifier.
    pub fn name(self) -> &'static str {
        match self {
            AccentVariation::IconTile => "Icon Tile",
            AccentVariation::IconTileFooterText => "Icon Tile + Footer Text",
            AccentVariation::IconTileFooterKeycaps => "Icon Tile + Footer Keys",
            AccentVariation::IconTileFooterDivider => "Icon Tile + Footer Line",
            AccentVariation::IconTileFooterFull => "Icon Tile + Full Footer",
            AccentVariation::FooterOnly => "Footer Only",
            AccentVariation::FooterButtonsSoft => "Footer Buttons Soft",
            AccentVariation::FooterButtonsMedium => "Footer Buttons Medium",
            AccentVariation::FooterButtonsBold => "Footer Buttons Bold",
            AccentVariation::SolidFill => "Solid Fill",
            AccentVariation::AccentText => "Accent Text",
            AccentVariation::Ring => "Accent Ring",
            AccentVariation::LeftBlock => "Left Block",
            AccentVariation::AllIcons => "All Icons",
            AccentVariation::AccentName => "Accent Name",
            AccentVariation::Loud => "Loud",
        }
    }

    /// One-line description of the treatment.
    pub fn description(self) -> &'static str {
        match self {
            AccentVariation::IconTile => "Selected icon in a solid accent tile",
            AccentVariation::IconTileFooterText => "Icon tile rows + accent footer text",
            AccentVariation::IconTileFooterKeycaps => "Icon tile rows + accent footer keycaps",
            AccentVariation::IconTileFooterDivider => "Icon tile rows + accent footer divider",
            AccentVariation::IconTileFooterFull => "Icon tile rows + full accent footer",
            AccentVariation::FooterOnly => "Plain rows, full accent footer",
            AccentVariation::FooterButtonsSoft => "Soft accent fill on footer buttons",
            AccentVariation::FooterButtonsMedium => "Medium accent fill on footer buttons",
            AccentVariation::FooterButtonsBold => "Bold accent fill on footer buttons",
            AccentVariation::SolidFill => "Selected row filled with strong accent",
            AccentVariation::AccentText => "Selected title and icon in bright accent",
            AccentVariation::Ring => "Selected row outlined with an accent ring",
            AccentVariation::LeftBlock => "Chunky accent block leads the selected row",
            AccentVariation::AllIcons => "Every row icon is accent-colored",
            AccentVariation::AccentName => "Bright accent title with thick underline",
            AccentVariation::Loud => "Accent rows + full accent footer at once",
        }
    }

    /// Identifier string shown as the main-menu placeholder so the active
    /// variation is always identifiable while browsing (e.g.
    /// `"Accent 5/13 · Icon Tile + Full Footer   ·   alt+←/→ to cycle"`).
    pub fn placeholder(self) -> String {
        format!(
            "Accent {}/{} · {}   ·   alt+\u{2190}/\u{2192} to cycle",
            self.index() + 1,
            Self::COUNT,
            self.name()
        )
    }
}

/// Process-global "currently active" accent variation.
///
/// The list rows receive the variation explicitly (threaded into `ListItem`),
/// but the native AppKit footer host (`src/footer_popup.rs`) runs outside the
/// GPUI render tree and cannot read `ScriptListApp` state directly. It consults
/// this global instead. `cycle_accent_variation` keeps it in sync, and the
/// footer's refresh signature includes the discriminant so a cycle forces a
/// rebuild with the new colors.
static CURRENT_ACCENT_VARIATION: AtomicU8 = AtomicU8::new(AccentVariation::IconTile as u8);

/// Record the active accent variation for the native footer to read.
pub fn set_current_accent_variation(variation: AccentVariation) {
    CURRENT_ACCENT_VARIATION.store(variation as u8, Ordering::Relaxed);
}

/// The active accent variation as last set by [`set_current_accent_variation`].
pub fn current_accent_variation() -> AccentVariation {
    AccentVariation::from_u8(CURRENT_ACCENT_VARIATION.load(Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::{current_accent_variation, set_current_accent_variation, AccentVariation};

    #[test]
    fn accent_variation_has_exactly_sixteen_variants() {
        assert_eq!(AccentVariation::all().len(), AccentVariation::COUNT);
        assert_eq!(AccentVariation::COUNT, 16);
        // Discriminants must be unique (round-trip every variant through u8).
        for v in AccentVariation::all() {
            assert_eq!(AccentVariation::from_u8(*v as u8), *v);
        }
    }

    #[test]
    fn accent_variation_cycles_forward_and_backward() {
        assert_eq!(
            AccentVariation::Loud.next(),
            AccentVariation::IconTile,
            "next() must wrap from the last variation to the first"
        );
        assert_eq!(
            AccentVariation::IconTile.prev(),
            AccentVariation::Loud,
            "prev() must wrap from the first variation to the last"
        );
        // A full forward cycle returns to the start.
        let mut v = AccentVariation::default();
        for _ in 0..AccentVariation::COUNT {
            v = v.next();
        }
        assert_eq!(v, AccentVariation::default());
    }

    #[test]
    fn accent_variation_names_and_placeholders_are_non_empty() {
        for variation in AccentVariation::all() {
            assert!(!variation.name().trim().is_empty());
            assert!(variation.placeholder().contains(variation.name()));
            assert!(variation.placeholder().contains("/16"));
        }
    }

    #[test]
    fn footer_button_fill_intensities_increase_and_keep_borders_neutral() {
        use AccentVariation::*;
        let soft = FooterButtonsSoft.footer_button_fill().expect("soft fill");
        let medium = FooterButtonsMedium
            .footer_button_fill()
            .expect("medium fill");
        let bold = FooterButtonsBold.footer_button_fill().expect("bold fill");
        // Within a variation: rest < hover < active.
        for fill in [soft, medium, bold] {
            assert!(fill.rest < fill.hover && fill.hover < fill.active);
        }
        // Across variations: soft < medium < bold at every state.
        assert!(soft.rest < medium.rest && medium.rest < bold.rest);
        assert!(soft.active < medium.active && medium.active < bold.active);
        // Button-fill variations leave borders/text/divider neutral and keep
        // the preferred Icon Tile rows.
        for v in [FooterButtonsSoft, FooterButtonsMedium, FooterButtonsBold] {
            assert_eq!(v.row_kind(), IconTile);
            assert!(!v.footer_text_accent());
            assert!(!v.footer_keycap_accent());
            assert!(!v.footer_divider_accent());
            assert!(v.touches_footer(), "button-fill must mark the footer dirty");
        }
        // Non-button-fill variations return None.
        assert!(IconTile.footer_button_fill().is_none());
        assert!(FooterOnly.footer_button_fill().is_none());
    }

    #[test]
    fn default_is_icon_tile_and_first() {
        assert_eq!(AccentVariation::default(), AccentVariation::IconTile);
        assert_eq!(AccentVariation::default().index(), 0);
    }

    #[test]
    fn footer_combo_rows_map_to_icon_tile() {
        use AccentVariation::*;
        for v in [
            IconTile,
            IconTileFooterText,
            IconTileFooterKeycaps,
            IconTileFooterDivider,
            IconTileFooterFull,
        ] {
            assert_eq!(v.row_kind(), IconTile, "{v:?} should render icon-tile rows");
        }
        // FooterOnly must NOT inherit a special row look (plain rows).
        assert_eq!(FooterOnly.row_kind(), FooterOnly);
    }

    #[test]
    fn footer_flags_match_intent() {
        use AccentVariation::*;
        assert!(IconTileFooterText.footer_text_accent());
        assert!(!IconTileFooterText.footer_keycap_accent());
        assert!(!IconTileFooterText.footer_divider_accent());

        assert!(IconTileFooterKeycaps.footer_keycap_accent());
        assert!(IconTileFooterDivider.footer_divider_accent());

        for flag in [
            IconTileFooterFull.footer_text_accent(),
            IconTileFooterFull.footer_keycap_accent(),
            IconTileFooterFull.footer_divider_accent(),
            FooterOnly.footer_text_accent(),
            FooterOnly.footer_keycap_accent(),
            FooterOnly.footer_divider_accent(),
            Loud.footer_text_accent(),
            Loud.footer_keycap_accent(),
            Loud.footer_divider_accent(),
        ] {
            assert!(flag, "full/footer-only/loud must enable every footer axis");
        }

        // Pure row treatments leave the footer alone.
        for v in [
            IconTile, SolidFill, AccentText, Ring, LeftBlock, AllIcons, AccentName,
        ] {
            assert!(!v.touches_footer(), "{v:?} must not touch the footer");
        }
    }

    #[test]
    fn global_round_trips_through_u8() {
        for v in AccentVariation::all() {
            set_current_accent_variation(*v);
            assert_eq!(current_accent_variation(), *v);
        }
        // Restore default so the global doesn't leak between tests.
        set_current_accent_variation(AccentVariation::default());
    }
}

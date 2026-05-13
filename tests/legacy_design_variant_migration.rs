//! Contract test pinning the Phase 1 invariant that every legacy
//! `DesignVariant` maps to a stable kebab-case id and that unknown
//! ids fall back to the canonical Script Kit Classic design.
//!
//! See `lat.md/designs.md#Legacy migration`.

use script_kit_gpui::designs::legacy_migration::{
    legacy_variant_def, map_legacy_variant_to_id, resolve_possibly_legacy_id,
};
use script_kit_gpui::designs::registry::{lookup, resolve_or_fallback, FALLBACK_ID};
use script_kit_gpui::designs::DesignVariant;

#[test]
fn every_legacy_design_variant_maps_to_known_design_id() {
    for v in DesignVariant::all() {
        let id = map_legacy_variant_to_id(*v);
        assert!(
            !id.is_empty(),
            "legacy variant {:?} mapped to an empty id",
            v
        );
    }
}

#[test]
fn unknown_design_id_falls_back_to_classic() {
    let def = resolve_or_fallback(Some("not-a-real-design"));
    assert_eq!(def.id, FALLBACK_ID);
    assert_eq!(FALLBACK_ID, "script-kit-classic");
}

#[test]
fn empty_design_id_falls_back_to_classic() {
    assert_eq!(resolve_or_fallback(Some("")).id, FALLBACK_ID);
    assert_eq!(resolve_or_fallback(None).id, FALLBACK_ID);
}

#[test]
fn every_legacy_variant_maps_to_exact_spec_id() {
    // The mapping table is the full migration contract per
    // `.goals/design-variants-overhaul.md`.
    let table = [
        (DesignVariant::Default, "script-kit-classic"),
        (DesignVariant::Minimal, "minimal-ink"),
        (DesignVariant::RetroTerminal, "retro-terminal"),
        (DesignVariant::Glassmorphism, "glass-frost"),
        (DesignVariant::Brutalist, "editorial-brutalist"),
        (DesignVariant::NeonCyberpunk, "neon-cyber"),
        (DesignVariant::Paper, "paper-print"),
        (DesignVariant::AppleHIG, "apple-hig"),
        (DesignVariant::Material3, "material-you"),
        (DesignVariant::Compact, "pro-dense"),
        (DesignVariant::Playful, "playful-pop"),
    ];
    for (variant, expected_id) in table {
        let id = map_legacy_variant_to_id(variant);
        assert_eq!(
            id, expected_id,
            "legacy variant {:?} must map to `{}`",
            variant, expected_id
        );
        assert!(
            lookup(id).is_some(),
            "id `{}` must be a real CATALOG entry",
            id
        );
        let def = legacy_variant_def(variant);
        assert_eq!(def.id, expected_id);
    }
}

#[test]
fn resolve_possibly_legacy_id_round_trip() {
    assert_eq!(
        resolve_possibly_legacy_id("Default"),
        Some("script-kit-classic")
    );
    assert_eq!(
        resolve_possibly_legacy_id("RetroTerminal"),
        Some("retro-terminal")
    );
    assert_eq!(
        resolve_possibly_legacy_id("script-kit-classic"),
        Some("script-kit-classic")
    );
    assert_eq!(resolve_possibly_legacy_id("totally-fake"), None);
}

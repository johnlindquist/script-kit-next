//! Legacy `DesignVariant` ↔ stable design id mapping.
//!
//! Old configs and code paths reference the [`DesignVariant`] enum; the
//! catalog-driven world keys on stable kebab-case ids. This module is the
//! only place that should bridge the two. Once the production sweep
//! finishes (Phase 6), the enum gets `#[deprecated]` and lives solely
//! here for config rewrites.
//!

use crate::designs::core::registry::{fallback, lookup, DesignDef, FALLBACK_ID};
use crate::designs::DesignVariant;

/// Map a legacy [`DesignVariant`] to its stable catalog id.
pub fn map_legacy_variant_to_id(variant: DesignVariant) -> &'static str {
    match variant {
        DesignVariant::Default => "script-kit-classic",
        DesignVariant::Minimal => "minimal-ink",
        DesignVariant::RetroTerminal => "retro-terminal",
        DesignVariant::Glassmorphism => "glass-frost",
        DesignVariant::Brutalist => "editorial-brutalist",
        DesignVariant::NeonCyberpunk => "neon-cyber",
        DesignVariant::Paper => "paper-print",
        DesignVariant::AppleHIG => "apple-hig",
        DesignVariant::Material3 => "material-you",
        DesignVariant::Compact => "pro-dense",
        DesignVariant::Playful => "playful-pop",
    }
}

/// Resolve a possibly-legacy id string. If it is one of the known
/// kebab-case ids it returns that; if it is a string version of a legacy
/// enum variant (e.g. "Default", "RetroTerminal") it is migrated.
/// Unknown inputs return [`None`] so callers can apply their own
/// fallback policy.
pub fn resolve_possibly_legacy_id(input: &str) -> Option<&'static str> {
    if lookup(input).is_some() {
        return Some(catalog_id_for(input));
    }
    let legacy = match input {
        "Default" | "default" => Some(DesignVariant::Default),
        "Minimal" | "minimal" => Some(DesignVariant::Minimal),
        "RetroTerminal" | "retro_terminal" | "retro-terminal-legacy" => {
            Some(DesignVariant::RetroTerminal)
        }
        "Glassmorphism" | "glassmorphism" => Some(DesignVariant::Glassmorphism),
        "Brutalist" | "brutalist" => Some(DesignVariant::Brutalist),
        "NeonCyberpunk" | "neon_cyberpunk" => Some(DesignVariant::NeonCyberpunk),
        "Paper" | "paper" => Some(DesignVariant::Paper),
        "AppleHIG" | "apple_hig" => Some(DesignVariant::AppleHIG),
        "Material3" | "material_3" => Some(DesignVariant::Material3),
        "Compact" | "compact" => Some(DesignVariant::Compact),
        "Playful" | "playful" => Some(DesignVariant::Playful),
        _ => None,
    };
    legacy.map(map_legacy_variant_to_id)
}

fn catalog_id_for(id: &str) -> &'static str {
    lookup(id).map(|d| d.id).unwrap_or(FALLBACK_ID)
}

/// Resolve a legacy variant directly to a `DesignDef`. Phase 2 ids that
/// are not yet seeded in CATALOG return the canonical fallback.
pub fn legacy_variant_def(variant: DesignVariant) -> &'static DesignDef {
    let id = map_legacy_variant_to_id(variant);
    lookup(id).unwrap_or_else(|| {
        crate::logging::log(
            "DESIGNS",
            &format!(
                "legacy variant {:?} -> id `{}` not yet in catalog; using fallback `{}`",
                variant, id, FALLBACK_ID
            ),
        );
        fallback()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::designs::DesignVariant;

    #[test]
    fn every_legacy_variant_maps_to_a_non_empty_id() {
        for v in DesignVariant::all() {
            let id = map_legacy_variant_to_id(*v);
            assert!(!id.is_empty(), "variant {:?} mapped to empty id", v);
        }
    }

    #[test]
    fn every_legacy_variant_resolves_to_real_catalog_entry() {
        for v in DesignVariant::all() {
            let id = map_legacy_variant_to_id(*v);
            assert!(
                lookup(id).is_some(),
                "legacy variant {:?} -> id `{}` must be in CATALOG",
                v,
                id
            );
            let def = legacy_variant_def(*v);
            assert_eq!(def.id, id);
        }
    }

    #[test]
    fn resolve_possibly_legacy_id_recognizes_legacy_strings() {
        assert_eq!(
            resolve_possibly_legacy_id("Default"),
            Some("script-kit-classic")
        );
        assert_eq!(
            resolve_possibly_legacy_id("RetroTerminal"),
            Some("retro-terminal")
        );
    }

    #[test]
    fn resolve_possibly_legacy_id_passes_through_known_kebab_ids() {
        assert_eq!(
            resolve_possibly_legacy_id("script-kit-classic"),
            Some("script-kit-classic")
        );
    }

    #[test]
    fn resolve_possibly_legacy_id_returns_none_for_unknown() {
        assert_eq!(resolve_possibly_legacy_id("totally-fake"), None);
    }
}

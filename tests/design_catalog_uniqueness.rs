//! Catalog uniqueness invariant.
//!
//! Phase 1 ships 10 curated designs. Phase 2 expands to exactly 25.
//! At every phase no two designs may share the same token signature
//! (palette + typography + density + chrome + vibrancy).
//!
//! See `removed-docs invariants`.

use std::collections::HashSet;

use script_kit_gpui::designs::registry::{catalog, signature_hash};

#[test]
fn design_catalog_ids_are_unique() {
    let mut seen = HashSet::new();
    for d in catalog() {
        assert!(seen.insert(d.id), "duplicate id `{}`", d.id);
    }
}

#[test]
fn design_catalog_signatures_are_unique() {
    let mut seen: HashSet<u64> = HashSet::new();
    for d in catalog() {
        let h = signature_hash(&d.signature());
        assert!(
            seen.insert(h),
            "duplicate token signature for `{}` — every design must change ≥2 dims",
            d.id
        );
    }
}

#[test]
fn design_catalog_has_25_unique_non_duplicate_designs() {
    let count = catalog().len();
    assert_eq!(count, 25, "catalog must contain exactly 25 designs");

    let mut id_set = HashSet::new();
    let mut sig_set: HashSet<u64> = HashSet::new();
    for d in catalog() {
        assert!(id_set.insert(d.id), "duplicate id `{}`", d.id);
        assert!(
            sig_set.insert(signature_hash(&d.signature())),
            "duplicate token signature for `{}`",
            d.id
        );
    }
    assert_eq!(id_set.len(), 25);
    assert_eq!(sig_set.len(), 25);
}

#[test]
fn every_legacy_migration_target_id_is_in_catalog() {
    use script_kit_gpui::designs::legacy_migration::map_legacy_variant_to_id;
    use script_kit_gpui::designs::registry::lookup;
    use script_kit_gpui::designs::DesignVariant;
    for v in DesignVariant::all() {
        let id = map_legacy_variant_to_id(*v);
        assert!(
            lookup(id).is_some(),
            "legacy variant {:?} -> id `{}` must exist in CATALOG",
            v,
            id
        );
    }
}

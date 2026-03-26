use std::collections::HashSet;

use super::{footer_story_variants, footer_variation_specs, FooterVariationId};

#[test]
fn footer_variation_ids_are_unique() {
    let ids: HashSet<_> = footer_variation_specs()
        .iter()
        .map(|spec| spec.id.as_str())
        .collect();

    assert_eq!(ids.len(), footer_variation_specs().len());
}

#[test]
fn footer_story_variants_cover_every_spec() {
    let stable_ids: Vec<_> = footer_story_variants()
        .into_iter()
        .map(|variant| variant.stable_id())
        .collect();

    assert_eq!(
        stable_ids,
        vec![
            "raycast-exact".to_string(),
            "scriptkit-branded".to_string(),
            "minimal".to_string(),
            "status-bar".to_string(),
            "invisible".to_string(),
        ]
    );
}

#[test]
fn footer_variation_lookup_round_trips() {
    for variation in FooterVariationId::ALL {
        assert_eq!(
            FooterVariationId::from_stable_id(variation.as_str()),
            Some(variation)
        );
    }
}

#[test]
fn footer_seed_set_preserves_existing_saved_ids() {
    let ids: Vec<_> = footer_variation_specs()
        .iter()
        .map(|spec| spec.id.as_str())
        .collect();

    assert!(ids.contains(&"raycast-exact"));
    assert!(ids.contains(&"scriptkit-branded"));
    assert!(ids.contains(&"minimal"));
}

#[test]
fn footer_variation_id_names_are_nonempty() {
    for variation in FooterVariationId::ALL {
        assert!(!variation.name().is_empty(), "{:?} has empty name", variation);
        assert!(
            !variation.description().is_empty(),
            "{:?} has empty description",
            variation
        );
    }
}

#[test]
fn footer_story_variants_have_surface_prop() {
    for variant in footer_story_variants() {
        assert_eq!(
            variant.props.get("surface").map(String::as_str),
            Some("footer"),
            "variant {} missing surface=footer prop",
            variant.stable_id()
        );
    }
}

#[test]
fn unknown_stable_id_returns_none() {
    assert_eq!(FooterVariationId::from_stable_id("nonexistent"), None);
}

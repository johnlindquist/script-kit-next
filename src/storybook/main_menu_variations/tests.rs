use std::collections::HashSet;

use super::{main_menu_story_variants, MainMenuVariationId};

#[test]
fn variation_ids_are_unique() {
    let ids: HashSet<_> = MainMenuVariationId::ALL.iter().map(|id| id.as_str()).collect();
    assert_eq!(ids.len(), MainMenuVariationId::ALL.len());
}

#[test]
fn story_variants_cover_every_id() {
    let stable_ids: Vec<_> = main_menu_story_variants()
        .into_iter()
        .map(|variant| variant.stable_id())
        .collect();

    assert_eq!(
        stable_ids,
        vec![
            "raycast-classic".to_string(),
            "compact-dense".to_string(),
            "spotlight".to_string(),
            "two-line".to_string(),
            "minimal-flat".to_string(),
            "big-cards".to_string(),
            "terminal".to_string(),
            "neon".to_string(),
        ]
    );
}

#[test]
fn variation_lookup_round_trips() {
    for variation in MainMenuVariationId::ALL {
        assert_eq!(
            MainMenuVariationId::from_stable_id(variation.as_str()),
            Some(variation)
        );
    }
}

#[test]
fn story_variants_have_surface_prop() {
    for variant in main_menu_story_variants() {
        assert_eq!(
            variant.props.get("surface").map(String::as_str),
            Some("main-menu"),
            "variant {} missing surface=main-menu prop",
            variant.stable_id()
        );
    }
}

#[test]
fn unknown_stable_id_returns_none() {
    assert_eq!(MainMenuVariationId::from_stable_id("nonexistent"), None);
}

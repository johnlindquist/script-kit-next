use std::collections::HashSet;

use super::{main_menu_story_variants, MainMenuVariationId};

#[test]
fn variation_ids_are_unique() {
    let ids: HashSet<_> = MainMenuVariationId::ALL
        .iter()
        .map(|id| id.as_str())
        .collect();
    assert_eq!(ids.len(), MainMenuVariationId::ALL.len());
}

#[test]
fn story_variants_cover_required_variants() {
    let stable_ids: Vec<_> = main_menu_story_variants()
        .into_iter()
        .map(|variant| variant.stable_id())
        .collect();

    assert_eq!(
        stable_ids,
        vec![
            "populated-results".to_string(),
            "empty-results".to_string(),
            "selected-row".to_string(),
            "bottom-of-list-footer-safe-reveal".to_string(),
            "frontmost-app-paste".to_string(),
            "acp-ready-footer".to_string(),
            "acp-not-ready-footer".to_string(),
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
fn story_variants_have_surface_and_live_preview_props() {
    for variant in main_menu_story_variants() {
        assert_eq!(
            variant.props.get("surface").map(String::as_str),
            Some("mainMenu"),
            "variant {} missing surface=mainMenu prop",
            variant.stable_id()
        );
        assert_eq!(
            variant.props.get("representation").map(String::as_str),
            Some("liveSurface"),
            "variant {} missing representation=liveSurface prop",
            variant.stable_id()
        );
    }
}

#[test]
fn unknown_stable_id_returns_none() {
    assert_eq!(MainMenuVariationId::from_stable_id("nonexistent"), None);
}

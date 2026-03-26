use std::collections::HashSet;

use super::{
    adopted_input_variation_id, input_story_variants, input_variation_specs, InputVariationId,
};

#[test]
fn input_variation_ids_are_unique() {
    let ids: HashSet<_> = input_variation_specs()
        .iter()
        .map(|spec| spec.id.as_str())
        .collect();

    assert_eq!(ids.len(), input_variation_specs().len());
}

#[test]
fn input_story_variants_cover_every_spec() {
    let stable_ids: Vec<_> = input_story_variants()
        .into_iter()
        .map(|variant| variant.stable_id())
        .collect();

    assert_eq!(
        stable_ids,
        vec![
            "bare".to_string(),
            "underline".to_string(),
            "pill".to_string(),
            "search-icon".to_string(),
            "prompt-prefix".to_string(),
        ]
    );
}

#[test]
fn input_variation_lookup_round_trips() {
    for variation in InputVariationId::ALL {
        assert_eq!(
            InputVariationId::from_stable_id(variation.as_str()),
            Some(variation)
        );
    }
}

#[test]
fn input_story_variants_have_surface_prop() {
    for variant in input_story_variants() {
        assert_eq!(
            variant.props.get("surface").map(String::as_str),
            Some("input"),
            "variant {} missing surface=input prop",
            variant.stable_id()
        );
    }
}

#[test]
fn unknown_stable_id_returns_none() {
    assert_eq!(InputVariationId::from_stable_id("nonexistent"), None);
}

#[test]
fn adopted_input_variation_defaults_to_bare() {
    assert_eq!(adopted_input_variation_id(None), InputVariationId::Bare);
    assert_eq!(
        adopted_input_variation_id(Some("not-a-variant")),
        InputVariationId::Bare
    );
}

#[test]
fn adopted_input_variation_accepts_saved_ids() {
    assert_eq!(
        adopted_input_variation_id(Some("search-icon")),
        InputVariationId::SearchIcon
    );
    assert_eq!(
        adopted_input_variation_id(Some("prompt-prefix")),
        InputVariationId::PromptPrefix
    );
}

#[test]
fn adopted_input_variation_resolves_all_known_ids() {
    for variation in InputVariationId::ALL {
        assert_eq!(
            adopted_input_variation_id(Some(variation.as_str())),
            variation,
            "adopted_input_variation_id should resolve {}",
            variation.as_str()
        );
    }
}

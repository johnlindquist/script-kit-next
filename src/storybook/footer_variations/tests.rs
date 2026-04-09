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
        assert!(
            !variation.name().is_empty(),
            "{:?} has empty name",
            variation
        );
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

// --- resolve_footer_selection structured resolution tests ---

use super::resolve_footer_selection;

#[test]
fn resolve_footer_selection_reports_fallback_for_unknown_value() {
    let (_config, resolution) = resolve_footer_selection(Some("not-a-variant"));

    assert_eq!(
        resolution.resolved_variant_id,
        FooterVariationId::RaycastExact.as_str()
    );
    assert!(resolution.fallback_used);
    assert_eq!(
        resolution.requested_variant_id.as_deref(),
        Some("not-a-variant")
    );
}

#[test]
fn resolve_footer_selection_preserves_minimal_layout() {
    let (config, resolution) = resolve_footer_selection(Some("minimal"));

    assert_eq!(resolution.resolved_variant_id, "minimal");
    assert!(!resolution.fallback_used);
    assert!(!config.show_primary);
    assert!(!config.show_secondary);
}

#[test]
fn resolve_footer_selection_none_uses_default_without_fallback() {
    let (_config, resolution) = resolve_footer_selection(None);

    assert_eq!(
        resolution.resolved_variant_id,
        FooterVariationId::RaycastExact.as_str()
    );
    assert!(!resolution.fallback_used);
    assert!(resolution.requested_variant_id.is_none());
}

#[test]
fn resolve_footer_selection_resolution_serializes_to_json() {
    let (_config, resolution) = resolve_footer_selection(Some("minimal"));
    let json = serde_json::to_string(&resolution).expect("serialize resolution");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse resolution");

    assert_eq!(value["resolvedVariantId"], "minimal");
    assert_eq!(value["fallbackUsed"], false);
    assert_eq!(value["requestedVariantId"], "minimal");
}

// --- Storybook footer selection → PromptFooterConfig bridge tests ---

use super::config_from_storybook_footer_selection_value;

#[test]
fn minimal_storybook_footer_selection_disables_action_buttons() {
    let config = config_from_storybook_footer_selection_value(Some("minimal"));

    assert!(!config.show_logo);
    assert!(!config.show_primary);
    assert!(!config.show_secondary);
}

#[test]
fn invalid_storybook_footer_selection_falls_back_to_default() {
    let config = config_from_storybook_footer_selection_value(Some("not-a-variant"));

    assert!(config.show_logo);
    assert!(config.show_primary);
    assert!(config.show_secondary);
    assert_eq!(config.primary_label, "Open Application");
}

#[test]
fn none_storybook_footer_selection_falls_back_to_default() {
    let config = config_from_storybook_footer_selection_value(None);

    assert!(config.show_logo);
    assert!(config.show_primary);
    assert!(config.show_secondary);
    assert_eq!(config.primary_label, "Open Application");
}

#[test]
fn scriptkit_branded_storybook_footer_selection_enables_helper_and_info() {
    let config = config_from_storybook_footer_selection_value(Some("scriptkit-branded"));

    assert!(config.show_logo);
    assert!(config.show_primary);
    assert!(config.show_secondary);
    assert!(config.show_info_label);
    assert_eq!(config.primary_label, "Run Script");
    assert_eq!(config.helper_text.as_deref(), Some("⌘↵ AI"));
    assert_eq!(config.info_label.as_deref(), Some("Built-in"));
}

#[test]
fn invisible_storybook_footer_selection_hides_everything() {
    let config = config_from_storybook_footer_selection_value(Some("invisible"));

    assert!(!config.show_logo);
    assert!(!config.show_primary);
    assert!(!config.show_secondary);
    assert!(!config.show_info_label);
}

#[test]
fn status_bar_storybook_footer_selection_shows_logo_hides_buttons() {
    let config = config_from_storybook_footer_selection_value(Some("status-bar"));

    assert!(config.show_logo);
    assert!(!config.show_primary);
    assert!(!config.show_secondary);
}

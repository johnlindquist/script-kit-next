use std::collections::HashSet;

use super::{
    actions_dialog_story_variants, adopted_actions_dialog_style, resolve_actions_dialog_style,
    ActionsDialogStyle, ActionsDialogSurface, ActionsDialogVariationId, SPECS,
};
use crate::storybook::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, VariationId,
};

#[test]
fn action_dialog_variation_ids_are_unique() {
    let ids: HashSet<_> = SPECS.iter().map(|spec| spec.id.as_str()).collect();
    assert_eq!(ids.len(), SPECS.len());
}

#[test]
fn action_dialog_variation_lookup_round_trips() {
    for variation in ActionsDialogVariationId::ALL {
        assert_eq!(
            ActionsDialogVariationId::from_stable_id(variation.as_str()),
            Some(variation)
        );
    }
}

#[test]
fn action_dialog_story_variants_cover_every_spec() {
    let ids: Vec<_> = actions_dialog_story_variants()
        .into_iter()
        .map(|variant| variant.stable_id())
        .collect();

    assert_eq!(
        ids,
        vec![
            "current".to_string(),
            "whisper".to_string(),
            "ghost-pills".to_string(),
            "typewriter".to_string(),
            "single-column".to_string(),
            "inline-keys".to_string(),
            "search-focused".to_string(),
            "dot-accent".to_string(),
        ]
    );
}

#[test]
fn action_dialog_story_variants_mark_surface() {
    for variant in actions_dialog_story_variants() {
        assert_eq!(
            variant.props.get("surface").map(String::as_str),
            Some("actionsDialog"),
            "variant {} missing surface=actionsDialog prop",
            variant.stable_id()
        );
    }
}

#[test]
fn resolve_surface_live_defaults_to_current() {
    let (style, resolution) = resolve_surface_live::<ActionsDialogSurface>(None);

    assert_eq!(style, SPECS[0].style);
    assert_eq!(resolution.story_id, ActionsDialogSurface::STORY_ID);
    assert_eq!(resolution.requested_variant_id, None);
    assert_eq!(resolution.resolved_variant_id, "current");
    assert!(!resolution.fallback_used);
}

#[test]
fn resolve_surface_live_falls_back_for_unknown_variant() {
    let (style, resolution) = resolve_actions_dialog_style(Some("unknown"));

    assert_eq!(style, SPECS[0].style);
    assert_eq!(resolution.requested_variant_id, Some("unknown".to_string()));
    assert_eq!(resolution.resolved_variant_id, "current");
    assert!(resolution.fallback_used);
}

#[test]
fn resolve_surface_live_accepts_whisper() {
    let (style, resolution) = resolve_actions_dialog_style(Some("whisper"));

    assert_eq!(style, SPECS[1].style);
    assert_eq!(resolution.resolved_variant_id, "whisper");
    assert!(!resolution.fallback_used);
}

#[test]
fn adopted_surface_live_defaults_to_current_when_store_missing() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let path = temp.path().join("design-explorer-selections.json");
    let style = crate::storybook::selection::with_test_selection_store_path(path, || {
        adopted_surface_live::<ActionsDialogSurface>()
    });

    assert_eq!(style, SPECS[0].style);
}

#[test]
fn adopted_actions_dialog_style_reads_persisted_selection() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let path = temp.path().join("design-explorer-selections.json");
    let style = crate::storybook::selection::with_test_selection_store_path(path, || {
        let result =
            crate::storybook::save_selected_story_variant("actions-mini-variations", "whisper")
                .expect("persist selection");
        assert_eq!(result.variant_id, "whisper");
        adopted_actions_dialog_style()
    });

    assert_eq!(style, SPECS[1].style);
}

#[test]
fn all_specs_have_valid_opacity_ranges() {
    for spec in SPECS {
        assert!(
            (0.0..=1.0).contains(&spec.style.selection_opacity),
            "{} selection opacity out of range",
            spec.id.as_str()
        );
        assert!(
            (0.0..=1.0).contains(&spec.style.hover_opacity),
            "{} hover opacity out of range",
            spec.id.as_str()
        );
    }
}

#[test]
fn dot_accent_has_no_selection_fill() {
    let dot_accent = SPECS
        .iter()
        .find(|spec| spec.id == ActionsDialogVariationId::DotAccent)
        .expect("dot accent spec");
    assert_eq!(
        dot_accent.style,
        ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: false,
            show_icons: false,
            selection_opacity: 0.0,
            hover_opacity: 0.03,
            row_height: SPECS[0].style.row_height,
            row_radius: SPECS[0].style.row_radius,
            shortcut_visible: true,
            mono_font: false,
            prefix_marker: None,
        }
    );
}

use std::collections::HashSet;

use super::{
    adopted_mini_ai_chat_style, mini_ai_chat_story_variants, resolve_mini_ai_chat_style,
    MiniAiChatStyle, MiniAiChatSurface, MiniAiChatVariationId, SPECS,
};
use crate::storybook::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, VariationId,
};

#[test]
fn mini_ai_chat_variation_ids_are_unique() {
    let ids: HashSet<_> = SPECS.iter().map(|spec| spec.id.as_str()).collect();
    assert_eq!(ids.len(), SPECS.len());
}

#[test]
fn mini_ai_chat_variation_lookup_round_trips() {
    for variation in MiniAiChatVariationId::ALL {
        assert_eq!(
            MiniAiChatVariationId::from_stable_id(variation.as_str()),
            Some(variation)
        );
    }
}

#[test]
fn mini_ai_chat_story_variants_cover_every_spec() {
    let ids: Vec<_> = mini_ai_chat_story_variants()
        .into_iter()
        .map(|variant| variant.stable_id())
        .collect();

    assert_eq!(
        ids,
        vec![
            "current".to_string(),
            "minilastic".to_string(),
            "bubbles".to_string(),
            "terminal".to_string(),
            "flush".to_string(),
        ]
    );
}

#[test]
fn mini_ai_chat_story_variants_mark_surface() {
    for variant in mini_ai_chat_story_variants() {
        assert_eq!(
            variant.props.get("surface").map(String::as_str),
            Some("miniAiChat"),
            "variant {} missing surface=miniAiChat prop",
            variant.stable_id()
        );
    }
}

#[test]
fn resolve_surface_live_defaults_to_current() {
    let (style, resolution) = resolve_surface_live::<MiniAiChatSurface>(None);

    assert_eq!(style, SPECS[0].style);
    assert_eq!(resolution.story_id, MiniAiChatSurface::STORY_ID);
    assert_eq!(resolution.requested_variant_id, None);
    assert_eq!(resolution.resolved_variant_id, "current");
    assert!(!resolution.fallback_used);
}

#[test]
fn resolve_surface_live_falls_back_for_unknown_variant() {
    let (style, resolution) = resolve_mini_ai_chat_style(Some("unknown"));

    assert_eq!(style, SPECS[0].style);
    assert_eq!(resolution.requested_variant_id, Some("unknown".to_string()));
    assert_eq!(resolution.resolved_variant_id, "current");
    assert!(resolution.fallback_used);
}

#[test]
fn resolve_surface_live_accepts_minilastic() {
    let (style, resolution) = resolve_mini_ai_chat_style(Some("minilastic"));

    assert_eq!(style, SPECS[1].style);
    assert_eq!(resolution.resolved_variant_id, "minilastic");
    assert!(!resolution.fallback_used);
}

#[test]
fn adopted_surface_live_defaults_to_current_when_store_missing() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let path = temp.path().join("design-explorer-selections.json");
    let style = crate::storybook::selection::with_test_selection_store_path(path, || {
        adopted_surface_live::<MiniAiChatSurface>()
    });

    assert_eq!(style, SPECS[0].style);
}

#[test]
fn adopted_mini_ai_chat_style_reads_persisted_selection() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let path = temp.path().join("design-explorer-selections.json");
    let style = crate::storybook::selection::with_test_selection_store_path(path, || {
        let result =
            crate::storybook::save_selected_story_variant("mini-ai-chat-variations", "terminal")
                .expect("persist selection");
        assert_eq!(result.variant_id, "terminal");
        adopted_mini_ai_chat_style()
    });

    assert_eq!(style, SPECS[3].style);
}

#[test]
fn all_specs_have_valid_opacity_ranges() {
    for spec in SPECS {
        let s = spec.style;
        let id = spec.id.as_str();

        for (name, val) in [
            ("titlebar_border_opacity", s.titlebar_border_opacity),
            ("titlebar_title_opacity", s.titlebar_title_opacity),
            ("titlebar_action_opacity", s.titlebar_action_opacity),
            ("composer_bg_opacity", s.composer_bg_opacity),
            ("composer_hairline_opacity", s.composer_hairline_opacity),
            ("composer_hint_opacity", s.composer_hint_opacity),
            (
                "composer_active_icon_opacity",
                s.composer_active_icon_opacity,
            ),
            ("message_user_bg_opacity", s.message_user_bg_opacity),
            (
                "message_assistant_bg_opacity",
                s.message_assistant_bg_opacity,
            ),
            ("welcome_icon_opacity", s.welcome_icon_opacity),
            ("welcome_heading_opacity", s.welcome_heading_opacity),
            ("welcome_title_opacity", s.welcome_title_opacity),
            ("welcome_badge_bg_opacity", s.welcome_badge_bg_opacity),
            ("action_hint_reveal_opacity", s.action_hint_reveal_opacity),
        ] {
            assert!(
                (0.0..=1.0).contains(&val),
                "{id} {name} = {val} is out of range [0.0, 1.0]"
            );
        }
    }
}

#[test]
fn current_matches_production_constants() {
    let current = SPECS[0].style;
    assert_eq!(current.titlebar_height, 44.0);
    assert_eq!(current.titlebar_border_opacity, 0.06);
    assert_eq!(current.titlebar_title_opacity, 0.55);
    assert_eq!(current.titlebar_action_opacity, 0.45);
    assert!(current.show_titlebar_border);
    assert_eq!(current.composer_bg_opacity, 0.03);
    assert_eq!(current.composer_hairline_opacity, 0.03);
    assert_eq!(current.composer_hint_opacity, 0.38);
    assert_eq!(current.composer_active_icon_opacity, 0.50);
    assert_eq!(current.message_user_bg_opacity, 0.06);
    assert_eq!(current.message_assistant_bg_opacity, 0.03);
    assert_eq!(current.message_padding_x, 12.0);
    assert_eq!(current.message_padding_y, 2.0);
    assert_eq!(current.message_gap, 8.0);
    assert_eq!(current.suggestion_count, 2);
    assert!(!current.mono_font);
    assert_eq!(current.user_prefix, None);
}

#[test]
fn terminal_uses_mono_and_prefix() {
    let (style, resolution) = resolve_mini_ai_chat_style(Some("terminal"));
    assert_eq!(resolution.resolved_variant_id, "terminal");
    assert!(style.mono_font);
    assert_eq!(style.user_prefix, Some(">"));
    assert_eq!(style.assistant_prefix, Some("<"));
    assert!(!style.show_titlebar_border);
}

#[test]
fn bubbles_shows_role_labels_and_rounded() {
    let (style, resolution) = resolve_mini_ai_chat_style(Some("bubbles"));
    assert_eq!(resolution.resolved_variant_id, "bubbles");
    assert!(style.show_role_labels);
    assert_eq!(style.message_border_radius, 12.0);
    assert_eq!(style.message_gap, 12.0);
}

#[test]
fn flush_has_zero_backgrounds() {
    let (style, resolution) = resolve_mini_ai_chat_style(Some("flush"));
    assert_eq!(resolution.resolved_variant_id, "flush");
    assert_eq!(style.message_user_bg_opacity, 0.0);
    assert_eq!(style.message_assistant_bg_opacity, 0.0);
    assert_eq!(style.composer_bg_opacity, 0.0);
    assert_eq!(style.welcome_badge_bg_opacity, 0.0);
    assert!(!style.show_titlebar_border);
}

#[test]
fn minilastic_no_titlebar_border() {
    let (style, resolution) = resolve_mini_ai_chat_style(Some("minilastic"));
    assert_eq!(resolution.resolved_variant_id, "minilastic");
    assert!(!style.show_titlebar_border);
    assert!(!style.show_action_hints);
    assert_eq!(style.titlebar_height, 40.0);
}

use super::*;

fn choice(name: &str, value: &str, description: Option<&str>) -> Choice {
    let mut choice = Choice::new(name.to_string(), value.to_string());
    choice.description = description.map(str::to_string);
    choice
}

#[test]
fn metadata_parses_shortcut_type_and_last_run() {
    let choice = choice(
        "Deploy API",
        "/Users/me/.scriptkit/scripts/deploy.ts",
        Some("Shortcut: cmd+shift+d • script • Last run 2h ago"),
    );

    let metadata = ChoiceDisplayMetadata::from_choice(&choice);

    assert_eq!(metadata.shortcut.as_deref(), Some("⌘⇧D"));
    assert_eq!(metadata.item_type.as_deref(), Some("Script"));
    assert_eq!(metadata.last_run.as_deref(), Some("Last run 2h ago"));
    assert!(metadata.description.is_none());
}

#[test]
fn score_choice_matches_description_and_value() {
    let choice = choice(
        "Deploy",
        "/Users/me/.scriptkit/scripts/deploy-api.ts",
        Some("Publish service to production"),
    );

    let mut description_ctx = scripts::NucleoCtx::new("production");
    let mut value_ctx = scripts::NucleoCtx::new("deploy-api.ts");
    let indexed_choice = SelectChoiceIndex::from_choice(&choice, 0);

    assert!(
        score_choice_for_filter(&choice, &indexed_choice, "production", &mut description_ctx)
            .is_some()
    );
    assert!(
        score_choice_for_filter(&choice, &indexed_choice, "deploy-api.ts", &mut value_ctx)
            .is_some()
    );
}

#[test]
fn score_choice_prefers_name_over_description_only_matches() {
    let name_match = choice(
        "Open Logs",
        "/tmp/open-logs.ts",
        Some("Tail runtime output"),
    );
    let description_match = choice("Tail Runtime", "/tmp/tail-runtime.ts", Some("Open logs"));
    let query = "open logs";

    let mut name_ctx = scripts::NucleoCtx::new(query);
    let mut description_ctx = scripts::NucleoCtx::new(query);
    let indexed_name_match = SelectChoiceIndex::from_choice(&name_match, 0);
    let indexed_description_match = SelectChoiceIndex::from_choice(&description_match, 1);

    let name_score =
        score_choice_for_filter(&name_match, &indexed_name_match, query, &mut name_ctx).unwrap();
    let description_score = score_choice_for_filter(
        &description_match,
        &indexed_description_match,
        query,
        &mut description_ctx,
    )
    .unwrap();

    assert!(
        name_score > description_score,
        "expected name match score ({name_score}) to beat description-only score ({description_score})"
    );
}

#[test]
fn char_indices_to_byte_ranges_handles_utf8_boundaries() {
    let text = "a😀b";
    // Indices for 😀 and b
    let ranges = char_indices_to_byte_ranges(text, &[1, 2]);
    assert_eq!(ranges, vec![1..6]);
}

#[test]
fn test_select_prompt_accepts_space_in_filter_query() {
    assert!(should_append_to_filter(' '));
}

#[test]
fn test_select_prompt_submit_uses_focused_item_in_single_mode_when_none_toggled() {
    let selected_indices = Vec::new();
    let resolved = resolve_submission_indices(false, &selected_indices, Some(4));
    assert_eq!(resolved, vec![4]);
}

#[test]
fn test_select_prompt_submit_prefers_focused_item_in_single_mode_when_selection_exists() {
    let selected_indices = vec![2];
    let resolved = resolve_submission_indices(false, &selected_indices, Some(4));
    assert_eq!(resolved, vec![4]);
}

#[test]
fn test_select_prompt_submit_uses_explicit_selection_in_multiple_mode() {
    let selected_indices = vec![2, 7];
    let resolved = resolve_submission_indices(true, &selected_indices, Some(4));
    assert_eq!(resolved, vec![2, 7]);
}

#[test]
fn test_select_prompt_cmd_a_toggles_only_when_all_filtered_items_are_selected() {
    let mut selected_indices = std::collections::HashSet::from([1, 7]);
    let filtered_indices = vec![1, 2, 3];

    assert!(!are_all_filtered_selected(
        &selected_indices,
        &filtered_indices
    ));
    toggle_filtered_selection(&mut selected_indices, &filtered_indices);
    assert_eq!(
        selected_indices,
        std::collections::HashSet::from([1, 2, 3, 7])
    );

    assert!(are_all_filtered_selected(
        &selected_indices,
        &filtered_indices
    ));
    toggle_filtered_selection(&mut selected_indices, &filtered_indices);
    assert_eq!(selected_indices, std::collections::HashSet::from([7]));
}

#[test]
fn test_select_prompt_select_all_preserves_existing_off_filter_selection() {
    let mut selected_indices = std::collections::HashSet::from([9]);
    let filtered_indices = vec![1, 2, 3];

    toggle_filtered_selection(&mut selected_indices, &filtered_indices);

    assert_eq!(
        selected_indices,
        std::collections::HashSet::from([1, 2, 3, 9])
    );
}

#[test]
fn test_select_prompt_generates_stable_semantic_id_when_filter_order_changes() {
    let stable_id = fallback_select_semantic_id(17, "scripts/demo.ts");

    assert_eq!(
        stable_id,
        fallback_select_semantic_id(17, "scripts/demo.ts")
    );
    assert_ne!(stable_id, fallback_select_semantic_id(3, "scripts/demo.ts"));
}

#[test]
fn test_select_prompt_resolves_search_box_bg_by_design_variant() {
    let mut theme = theme::Theme::default();
    theme.colors.background.search_box = 0x112233;

    let design_colors = DesignColors {
        background_secondary: 0x445566,
        ..Default::default()
    };

    assert_eq!(
        resolve_search_box_bg_hex(&theme, DesignVariant::Default, &design_colors),
        0x112233
    );
    assert_eq!(
        resolve_search_box_bg_hex(&theme, DesignVariant::Minimal, &design_colors),
        0x445566
    );
}

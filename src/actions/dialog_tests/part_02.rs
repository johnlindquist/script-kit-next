
// ============================================================
// 8. Constructor/config/sdk helper behavior
// ============================================================

#[test]
fn initial_selection_index_skips_header_row() {
    let rows = vec![
        GroupedActionItem::SectionHeader("Actions".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(initial_selection_index(&rows), 1);
}

#[test]
fn config_change_requires_rebuild_when_section_style_changes() {
    let previous = ActionsDialogConfig {
        section_style: SectionStyle::Separators,
        ..ActionsDialogConfig::default()
    };
    let next = ActionsDialogConfig {
        section_style: SectionStyle::Headers,
        ..ActionsDialogConfig::default()
    };

    assert!(should_rebuild_grouped_items_for_config_change(
        &previous, &next
    ));
}

#[test]
fn config_change_does_not_require_rebuild_when_section_style_same() {
    let previous = ActionsDialogConfig {
        search_position: super::types::SearchPosition::Bottom,
        section_style: SectionStyle::Separators,
        ..ActionsDialogConfig::default()
    };
    let next = ActionsDialogConfig {
        search_position: super::types::SearchPosition::Top,
        section_style: SectionStyle::Separators,
        ..ActionsDialogConfig::default()
    };

    assert!(!should_rebuild_grouped_items_for_config_change(
        &previous, &next
    ));
}

#[test]
fn selected_protocol_action_uses_visible_index_mapping() {
    // Visible action #1 should map to original protocol action index #3.
    let visible_to_protocol = vec![0, 3];
    let selected_action_index = Some(1);

    assert_eq!(
        resolve_selected_protocol_action_index(selected_action_index, &visible_to_protocol),
        Some(3)
    );
}

#[test]
fn selected_protocol_action_mapping_returns_none_for_out_of_bounds() {
    let visible_to_protocol = vec![0];
    let selected_action_index = Some(2);

    assert_eq!(
        resolve_selected_protocol_action_index(selected_action_index, &visible_to_protocol),
        None
    );
}

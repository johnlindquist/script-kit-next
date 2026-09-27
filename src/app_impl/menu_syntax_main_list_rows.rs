//! Main-list rows for the menu-syntax trigger picker and `@` object selector.
//!
//! Both pickers draw their choices as ordinary main-list rows: one section
//! header, then one `SpineProjection` result per picker row, so selection,
//! Enter and the row renderer are the launcher's own.

use crate::list_item::GroupedListItem;
use crate::scripts::SearchResult;

pub fn build_menu_syntax_trigger_picker_main_list_results(
    snapshot: &crate::menu_syntax::TriggerPickerSnapshot,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    let section = snapshot.mode.main_list_section();
    let mut grouped_items = Vec::with_capacity(snapshot.rows.len() + 1);
    let mut flat_results = Vec::with_capacity(snapshot.rows.len());
    grouped_items.push(GroupedListItem::SectionHeader(
        section.0.to_string(),
        Some(section.1.to_string()),
    ));

    for row in &snapshot.rows {
        let flat_index = flat_results.len();
        flat_results.push(SearchResult::SpineProjection(
            crate::menu_syntax_trigger_picker::trigger_picker_row_to_main_list_row(row),
        ));
        grouped_items.push(GroupedListItem::Item(flat_index));
    }

    (grouped_items, flat_results)
}

pub fn build_menu_syntax_object_selector_main_list_results(
    snapshot: &crate::menu_syntax::ObjectSelectorSnapshot,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    let mut grouped_items = Vec::with_capacity(snapshot.rows.len() + 1);
    let mut flat_results = Vec::with_capacity(snapshot.rows.len());
    grouped_items.push(GroupedListItem::SectionHeader(
        "Objects".to_string(),
        Some("at-sign".to_string()),
    ));

    for row in &snapshot.rows {
        let flat_index = flat_results.len();
        flat_results.push(SearchResult::SpineProjection(
            crate::menu_syntax::object_selector_row_to_main_list_row(row),
        ));
        grouped_items.push(GroupedListItem::Item(flat_index));
    }

    (grouped_items, flat_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::{
        build_object_selector_snapshot, build_trigger_picker_snapshot, CaptureObjectKind,
        ObjectSelectorCandidate, ObjectSelectorContext, TriggerPickerContext,
    };

    /// Every picker row becomes one selectable main-list row under one header, in picker order.
    fn assert_rows_follow_one_header(
        grouped: &[GroupedListItem],
        flat: &[SearchResult],
        expected_ids: &[String],
        header: (&str, &str),
    ) {
        assert!(
            matches!(
                &grouped[0],
                GroupedListItem::SectionHeader(label, Some(icon))
                    if label == header.0 && icon == header.1
            ),
            "the picker's rows start with its section header"
        );
        assert_eq!(grouped.len(), expected_ids.len() + 1);
        assert_eq!(flat.len(), expected_ids.len());
        for (position, expected_id) in expected_ids.iter().enumerate() {
            let GroupedListItem::Item(flat_index) = grouped[position + 1] else {
                panic!("row {position} is not a selectable item");
            };
            assert_eq!(flat_index, position);
            let SearchResult::SpineProjection(row) = &flat[flat_index] else {
                panic!("row {position} is not a main-list projection row");
            };
            assert_eq!(&*row.id, expected_id.as_str());
        }
    }

    #[test]
    fn trigger_picker_rows_become_main_list_rows_under_their_section() {
        let snapshot = build_trigger_picker_snapshot(
            ";snippet Hello there! :",
            &TriggerPickerContext::default(),
        )
        .expect("capture field picker");
        assert!(snapshot.rows.len() > 1);
        let expected_ids: Vec<String> = snapshot
            .rows
            .iter()
            .map(|row| {
                crate::menu_syntax_trigger_picker::trigger_picker_row_to_main_list_row(row)
                    .id
                    .to_string()
            })
            .collect();

        let (grouped, flat) = build_menu_syntax_trigger_picker_main_list_results(&snapshot);

        assert_rows_follow_one_header(
            &grouped,
            &flat,
            &expected_ids,
            snapshot.mode.main_list_section(),
        );
    }

    #[test]
    fn object_selector_rows_become_main_list_rows_under_objects() {
        let candidate = |id: &str, label: &str| ObjectSelectorCandidate {
            kind: CaptureObjectKind::Snippet,
            id: id.to_string(),
            label: label.to_string(),
            subtitle: "Snippet".to_string(),
        };
        let ctx = ObjectSelectorContext {
            candidates: vec![
                candidate("fetch-json", "fetch-json"),
                candidate("fetch-text", "fetch-text"),
            ],
        };
        let snapshot = build_object_selector_snapshot(";snippet update @fetch", &[], &ctx)
            .expect("object selector");
        let expected_ids: Vec<String> = snapshot
            .rows
            .iter()
            .map(|row| format!("menu-syntax-object:{}", row.id))
            .collect();
        assert_eq!(expected_ids.len(), 2);

        let (grouped, flat) = build_menu_syntax_object_selector_main_list_results(&snapshot);

        assert_rows_follow_one_header(&grouped, &flat, &expected_ids, ("Objects", "at-sign"));
    }

    #[test]
    fn an_empty_picker_keeps_only_its_header() {
        let snapshot = crate::menu_syntax::ObjectSelectorSnapshot {
            kind: CaptureObjectKind::Todo,
            query: String::new(),
            active_range: (0, 0),
            rows: Vec::new(),
        };

        let (grouped, flat) = build_menu_syntax_object_selector_main_list_results(&snapshot);

        assert_rows_follow_one_header(&grouped, &flat, &[], ("Objects", "at-sign"));
    }
}

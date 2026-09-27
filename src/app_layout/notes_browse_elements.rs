//! What `getElements` lists for Notes Browse: the filter, the list, the search state, the
//! primary action, and one choice plus one metadata panel per visible row, in the order the
//! list draws them. The binary's element collector calls it; the lib carries this file through
//! `#[path]` so its tests run under `cargo test --lib`.

use crate::notes::search_model::NoteSearchHostState;
use crate::protocol;

fn push_limited(
    elements: &mut Vec<protocol::ElementInfo>,
    limit: usize,
    element: protocol::ElementInfo,
) {
    if elements.len() < limit {
        elements.push(element);
    }
}

/// Up to `limit` elements for `search` and the total the surface has.
pub(crate) fn notes_browse_elements(
    search: &NoteSearchHostState,
    input_focused: bool,
    limit: usize,
) -> (Vec<protocol::ElementInfo>, usize) {
    let rows = search.state.rows();
    let total_count = rows.len().saturating_mul(2) + 4;
    let mut elements = Vec::with_capacity(limit.min(total_count));

    push_limited(
        &mut elements,
        limit,
        protocol::ElementInfo::input(
            "notes-browse-filter",
            Some(search.query.as_str()),
            input_focused,
        ),
    );
    push_limited(
        &mut elements,
        limit,
        protocol::ElementInfo::list("notes", rows.len()),
    );
    push_limited(
        &mut elements,
        limit,
        protocol::ElementInfo {
            semantic_id: "notes-search-state".to_string(),
            element_type: protocol::ElementType::Panel,
            text: Some(match search.state.kind() {
                "failed" => "Notes couldn’t be loaded".to_string(),
                "loading" => "Loading notes".to_string(),
                "readyEmpty" => "No notes yet".to_string(),
                "noMatch" => "No matching notes".to_string(),
                _ => "Notes ready".to_string(),
            }),
            value: Some(search.state.generation().to_string()),
            content: None,
            selected: None,
            focused: None,
            index: None,
            role: Some("status".to_string()),
            kind: Some("noteSearchState".to_string()),
            source: Some("notes".to_string()),
            source_name: None,
            selectable: Some(false),
            status_kind: Some(search.state.kind().to_string()),
            action_disabled: None,
            style: None,
        },
    );
    push_limited(
        &mut elements,
        limit,
        protocol::ElementInfo {
            semantic_id: search.destination.semantic_action().to_string(),
            element_type: protocol::ElementType::Button,
            text: Some(search.destination.primary_verb().to_string()),
            value: search.selected_id.map(|id| id.stable_id()),
            content: None,
            selected: None,
            focused: None,
            index: None,
            role: Some("action".to_string()),
            kind: Some(search.destination.as_str().to_string()),
            source: Some("notes".to_string()),
            source_name: Some("Notes search".to_string()),
            selectable: Some(search.selected_row().is_some()),
            status_kind: None,
            action_disabled: search
                .selected_row()
                .is_none()
                .then(|| "Select a note first.".to_string()),
            style: None,
        },
    );

    for (index, row) in rows.iter().enumerate() {
        if elements.len() >= limit {
            break;
        }
        elements.push(
            protocol::ElementInfo {
                semantic_id: row.semantic_id(),
                element_type: protocol::ElementType::Choice,
                text: Some(row.title.clone()),
                value: Some(row.stable_id()),
                content: None,
                selected: Some(search.selected_id == Some(row.id)),
                focused: None,
                index: Some(index),
                role: Some("result".to_string()),
                kind: Some(row.kind.as_str().to_string()),
                source: Some("notes".to_string()),
                source_name: Some(search.destination.primary_verb().to_string()),
                selectable: Some(true),
                status_kind: None,
                action_disabled: None,
                style: None,
            }
            .redact_text(protocol::ElementContentKind::UserContent),
        );
        push_limited(
            &mut elements,
            limit,
            protocol::ElementInfo {
                semantic_id: format!("{}:metadata", row.semantic_id()),
                element_type: protocol::ElementType::Panel,
                text: Some(row.preview.clone()),
                value: Some(row.automation_metadata()),
                content: None,
                selected: None,
                focused: None,
                index: Some(index),
                role: Some("resultMetadata".to_string()),
                kind: Some(row.kind.as_str().to_string()),
                source: Some("notes".to_string()),
                source_name: None,
                selectable: Some(false),
                status_kind: None,
                action_disabled: None,
                style: None,
            }
            .redact_content(protocol::ElementContentKind::UserContent),
        );
    }

    (elements, total_count)
}

#[cfg(test)]
mod tests {
    use super::notes_browse_elements;
    use crate::notes::search_model::{
        NoteSearchDestination, NoteSearchDocumentId, NoteSearchDocumentKind, NoteSearchHostState,
        NoteSearchRow, NoteSearchSnapshot, NoteSearchState,
    };
    use crate::notes::NoteId;
    use crate::protocol::ElementType;

    fn row(id: &str, title: &str) -> NoteSearchRow {
        NoteSearchRow {
            id: NoteSearchDocumentId::Note(NoteId::parse(id).expect("note id")),
            title: title.to_string(),
            preview: format!("{title} body"),
            updated_at: "2026-07-01T09:00:00Z".parse().expect("timestamp"),
            char_count: title.len(),
            pinned: false,
            kind: NoteSearchDocumentKind::Note,
        }
    }

    fn search(rows: Vec<NoteSearchRow>, selected: usize) -> NoteSearchHostState {
        let selected_id = rows.get(selected).map(|row| row.id);
        NoteSearchHostState {
            query: "plan".to_string(),
            selected_id,
            scroll_anchor: selected_id,
            generation: 3,
            destination: NoteSearchDestination::OpenNote,
            state: NoteSearchState::Ready {
                generation: 3,
                snapshot: NoteSearchSnapshot {
                    total_count: rows.len(),
                    rows,
                },
            },
        }
    }

    fn three_rows() -> Vec<NoteSearchRow> {
        vec![
            row("00000000-0000-0000-0000-000000000001", "Plan"),
            row("00000000-0000-0000-0000-000000000002", "Planets"),
            row("00000000-0000-0000-0000-000000000003", "Planning"),
        ]
    }

    #[test]
    fn the_listed_choices_are_the_visible_rows_in_the_order_the_list_draws_them() {
        let search = search(three_rows(), 1);
        let (elements, total) = notes_browse_elements(&search, true, 100);

        let listed: Vec<(&str, Option<bool>)> = elements
            .iter()
            .filter(|element| element.element_type == ElementType::Choice)
            .map(|element| (element.semantic_id.as_str(), element.selected))
            .collect();
        let visible: Vec<String> = search
            .state
            .rows()
            .iter()
            .map(|r| r.semantic_id())
            .collect();
        assert_eq!(
            listed,
            [
                (visible[0].as_str(), Some(false)),
                (visible[1].as_str(), Some(true)),
                (visible[2].as_str(), Some(false)),
            ],
            "one choice per visible row, in order, with the list's selection"
        );
        let list = elements
            .iter()
            .find(|element| element.element_type == ElementType::List)
            .expect("the notes list element");
        assert_eq!(list.text.as_deref(), Some("3 items"));
        assert_eq!(total, elements.len());
    }

    #[test]
    fn row_titles_and_previews_stay_out_of_the_projection() {
        let search = search(three_rows(), 0);
        let (elements, _) = notes_browse_elements(&search, false, 100);
        for element in &elements {
            for words in ["Plan body", "Planets", "Planning"] {
                assert!(
                    !element.text.as_deref().unwrap_or("").contains(words)
                        && !element.value.as_deref().unwrap_or("").contains(words),
                    "{} leaks {words}",
                    element.semantic_id
                );
            }
        }
    }

    #[test]
    fn the_limit_cuts_the_rows_but_not_the_total() {
        let search = search(three_rows(), 0);
        let (elements, total) = notes_browse_elements(&search, false, 5);
        assert_eq!(elements.len(), 5);
        assert_eq!(total, 10);
        assert_eq!(
            elements
                .iter()
                .filter(|element| element.element_type == ElementType::Choice)
                .count(),
            1
        );
    }
}

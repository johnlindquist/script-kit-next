#[test]
fn root_unified_notes_config_is_real_and_scoped() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");
    let defaults = include_str!("../../src/config/defaults.rs");

    assert!(config_types.contains("pub struct UnifiedSearchNotesConfig"));
    assert!(config_types.contains("fn notes_section_options("));
    assert!(config_schema.contains("notes?: UnifiedSearchNotesConfig"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_NOTES_ENABLED: bool = true"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_NOTES_SEARCH_CONTENT: bool = true"));
}

#[test]
fn root_unified_notes_search_is_metadata_only_bounded_and_active_only() {
    let storage = include_str!("../../src/notes/storage.rs");
    let search_fn = storage
        .split("pub(crate) fn search_root_notes_meta(")
        .nth(1)
        .and_then(|rest| rest.split("/// Permanently delete a note").next())
        .expect("search_root_notes_meta should exist");

    assert!(storage.contains("pub(crate) struct RootNotesSectionOptions"));
    assert!(storage.contains("pub(crate) struct RootNoteSearchHit"));
    assert!(storage.contains("root_notes_query_is_eligible("));
    assert!(search_fn.contains("n.deleted_at IS NULL"));
    assert!(search_fn.contains("if hits.is_empty()"));
    assert!(search_fn.contains("search_root_notes_meta_like(&conn, query, true, limit)?"));
    assert!(search_fn.contains("title LIKE ?1 OR content LIKE ?1"));
    assert!(search_fn.contains("deleted_at IS NULL AND title LIKE"));
    assert!(search_fn.contains("LIMIT ?2"));
    assert!(search_fn.contains("LIMIT ?4"));
    assert!(search_fn.contains("length(n.content)"));
    assert!(search_fn.contains("length(content)"));
    assert!(
        !search_fn.contains("content: String"),
        "root notes hits must not carry full note body content"
    );
}

#[test]
fn root_unified_notes_uses_passive_grouping_contract() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("fn append_root_notes_section("));
    assert!(grouping.contains("append_root_passive_section(grouped, flat_results, \"Notes\", rows"));
    assert!(grouping.contains("root_notes_query_is_eligible("));
    assert!(
        grouping.find("append_root_notes_section(")
            < grouping.find("append_root_clipboard_history_section("),
        "Notes rows should be appended before Clipboard History"
    );
    assert!(
        grouping.contains("label.starts_with(\"Use \\\"\") && label.ends_with(\"\\\" with...\")"),
        "passive insertion should target the fallback section header, not the first fallback row"
    );
}

#[test]
fn root_unified_notes_result_is_stable_and_non_bindable() {
    let types = include_str!("../../src/scripts/types.rs");

    assert!(types.contains("pub struct NoteMatch"));
    assert!(types.contains("Note(NoteMatch)"));
    assert!(types.contains("\"note/{}\""));
    assert!(types.contains("SearchResult::Note(_) => None"));
    assert!(types.contains("SearchResult::Note(_) => \"Open Note\""));
}

#[test]
fn root_unified_notes_enter_uses_non_toggle_open_helper() {
    let selection = include_str!("../../src/app_impl/selection_fallback.rs");
    let window_ops = include_str!("../../src/notes/window/window_ops.rs");

    assert!(selection.contains("SearchResult::Note(note_match)"));
    assert!(selection.contains("execute_root_note_open("));
    assert!(selection.contains("crate::notes::open_note_in_notes_window(cx, note_id)"));
    assert!(window_ops.contains("pub fn open_note_in_notes_window("));
    assert!(window_ops.contains(
        "open_notes_window_with_close_behavior(cx, NotesCloseBehavior::LeaveLauncherHidden)"
    ));

    let root_exec = selection
        .split("pub(crate) fn execute_root_note_open(")
        .nth(1)
        .and_then(|rest| {
            rest.split("pub(crate) fn selected_root_directory_query_owned")
                .next()
        })
        .expect("execute_root_note_open should exist");
    assert!(
        !root_exec.contains("open_notes_window("),
        "root note Enter must not call the toggle-style open_notes_window helper"
    );
}

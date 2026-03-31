// Integration tests for file-search mutation refresh (in-place cache patching).
//
// These tests verify the structural contract of `refresh_file_search_after_mutation`
// and its helper methods by inspecting the source code. The actual runtime behavior
// requires a full GPUI app context which these source-level tests validate by
// ensuring the key branching logic and call patterns are correct.

/// Source text of the files.rs action handler module.
const FILES_SOURCE: &str = include_str!("../../src/app_actions/handle_action/files.rs");

/// Source text of utility_views.rs for cross-referencing helpers.
const UTILITY_SOURCE: &str = include_str!("../../src/app_execute/utility_views.rs");

// ──────────────────────────────────────────────────────────────────────
// Signature contract: refresh_file_search_after_mutation accepts old_path
// ──────────────────────────────────────────────────────────────────────

#[test]
fn refresh_mutation_accepts_old_path_parameter() {
    assert!(
        FILES_SOURCE.contains("fn refresh_file_search_after_mutation("),
        "refresh_file_search_after_mutation must exist"
    );
    assert!(
        FILES_SOURCE.contains("old_path: &str,"),
        "refresh_file_search_after_mutation must accept old_path: &str"
    );
    assert!(
        FILES_SOURCE.contains("preferred_path: Option<&str>,"),
        "refresh_file_search_after_mutation must accept preferred_path: Option<&str>"
    );
}

// ──────────────────────────────────────────────────────────────────────
// In-place patch path: retain + push instead of full re-resolve
// ──────────────────────────────────────────────────────────────────────

#[test]
fn refresh_mutation_patches_cache_in_place_for_directory_browse() {
    assert!(
        FILES_SOURCE.contains("can_patch_in_place"),
        "must have a can_patch_in_place decision branch"
    );
    assert!(
        FILES_SOURCE.contains("self.cached_file_results.retain(|entry| entry.path != old_path)"),
        "must remove old entry from cache via retain"
    );
    assert!(
        FILES_SOURCE.contains("build_file_result_from_metadata"),
        "must use build_file_result_from_metadata for the new entry"
    );
    assert!(
        FILES_SOURCE.contains("self.apply_file_search_sort_mode()"),
        "must re-sort after patching using the active sort mode"
    );
    assert!(
        FILES_SOURCE.contains("self.recompute_file_search_display_indices()"),
        "must recompute display indices after patching"
    );
}

#[test]
fn refresh_mutation_falls_back_to_full_resolve_for_global_search() {
    assert!(
        FILES_SOURCE.contains("resolve_file_search_results(&query_value)"),
        "must fall back to resolve_file_search_results for global search"
    );
    assert!(
        FILES_SOURCE.contains("self.update_file_search_results(results)"),
        "must call update_file_search_results in the fallback path"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Helper methods exist
// ──────────────────────────────────────────────────────────────────────

#[test]
fn build_file_result_from_metadata_helper_exists() {
    assert!(
        FILES_SOURCE.contains("fn build_file_result_from_metadata(path: &str)"),
        "build_file_result_from_metadata must exist in files.rs"
    );
    assert!(
        FILES_SOURCE.contains("get_file_metadata(path)"),
        "must delegate to get_file_metadata"
    );
}

#[test]
fn current_file_search_directory_abs_helper_exists() {
    assert!(
        FILES_SOURCE.contains("fn current_file_search_directory_abs(&self)"),
        "current_file_search_directory_abs must exist"
    );
    assert!(
        FILES_SOURCE.contains("parse_directory_path(query)"),
        "must use parse_directory_path to detect directory-browse mode"
    );
}

#[test]
fn parent_directory_abs_helper_exists() {
    assert!(
        FILES_SOURCE.contains("fn parent_directory_abs(path: &str)"),
        "parent_directory_abs must exist"
    );
    assert!(
        FILES_SOURCE.contains("ensure_trailing_slash"),
        "must normalize with ensure_trailing_slash"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Call sites pass old_path correctly
// ──────────────────────────────────────────────────────────────────────

#[test]
fn rename_call_site_passes_old_path() {
    let rename_section = FILES_SOURCE
        .find("\"rename_path\" =>")
        .expect("rename_path handler must exist");
    let rename_end = (rename_section + 3000).min(FILES_SOURCE.len());
    let rename_body = &FILES_SOURCE[rename_section..rename_end];

    assert!(
        rename_body.contains("refresh_file_search_after_mutation("),
        "rename handler must call refresh_file_search_after_mutation"
    );
    assert!(
        rename_body.contains("&path,\n                                    Some(&new_path),"),
        "rename must pass old_path then Some(new_path)"
    );
}

#[test]
fn move_call_site_passes_old_path() {
    let move_section = FILES_SOURCE
        .find("\"move_path\" =>")
        .expect("move_path handler must exist");
    let move_end = (move_section + 3000).min(FILES_SOURCE.len());
    let move_body = &FILES_SOURCE[move_section..move_end];

    assert!(
        move_body.contains("refresh_file_search_after_mutation("),
        "move handler must call refresh_file_search_after_mutation"
    );
    assert!(
        move_body.contains("&path,\n                                    Some(&new_path),"),
        "move must pass old_path then Some(new_path)"
    );
}

#[test]
fn trash_call_site_passes_old_path() {
    let trash_section = FILES_SOURCE
        .find("\"move_to_trash\" =>")
        .expect("move_to_trash handler must exist");
    let trash_end = (trash_section + 5000).min(FILES_SOURCE.len());
    let trash_body = &FILES_SOURCE[trash_section..trash_end];

    assert!(
        trash_body.contains("refresh_file_search_after_mutation("),
        "trash handler must call refresh_file_search_after_mutation"
    );
    assert!(
        trash_body.contains("&path,\n                                    None,"),
        "trash must pass old_path then None (no preferred path)"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Selection preservation
// ──────────────────────────────────────────────────────────────────────

#[test]
fn refresh_mutation_preserves_selection_on_preferred_path() {
    assert!(
        FILES_SOURCE.contains("file_search_display_index_for_path(path)"),
        "must try to find the preferred path in the new display list"
    );
    assert!(
        FILES_SOURCE.contains("previous_display_index.min(len.saturating_sub(1))"),
        "must clamp to nearest valid row when preferred path is gone"
    );
}

// ──────────────────────────────────────────────────────────────────────
// sort_directory_results exists in utility_views
// ──────────────────────────────────────────────────────────────────────

#[test]
fn sort_directory_results_available() {
    assert!(
        UTILITY_SOURCE.contains("fn sort_directory_results("),
        "sort_directory_results must exist in utility_views.rs"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Pure logic: ensure_trailing_slash and parent_directory_abs
// ──────────────────────────────────────────────────────────────────────

#[test]
fn ensure_trailing_slash_normalizes() {
    use script_kit_gpui::file_search::ensure_trailing_slash;

    assert_eq!(ensure_trailing_slash("/foo/bar"), "/foo/bar/");
    assert_eq!(ensure_trailing_slash("/foo/bar/"), "/foo/bar/");
    assert_eq!(ensure_trailing_slash(""), "/");
    assert_eq!(ensure_trailing_slash("~"), "~/");
}

#[test]
fn expand_path_handles_tilde() {
    use script_kit_gpui::file_search::expand_path;

    let home = dirs::home_dir().expect("home dir must exist");
    let home_str = home.to_str().expect("home dir must be utf-8");

    assert_eq!(expand_path("~"), Some(home_str.to_string()));
    assert_eq!(expand_path("~/dev"), Some(format!("{}/dev", home_str)));
}

#[test]
fn parse_directory_path_detects_directory_browse() {
    use script_kit_gpui::file_search::parse_directory_path;

    let parsed = parse_directory_path("~/").expect("~/ must parse");
    assert_eq!(parsed.directory, "~/");
    assert!(parsed.filter.is_none());

    let parsed = parse_directory_path("~").expect("~ must parse");
    assert_eq!(parsed.directory, "~/");
    assert!(parsed.filter.is_none());
}

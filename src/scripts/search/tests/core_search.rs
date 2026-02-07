use std::path::PathBuf;
use std::sync::Arc;

use super::super::*;

// ============================================
// Helper function tests
// ============================================

#[test]
fn test_is_word_boundary_match_start() {
    assert!(is_word_boundary_match("Hello World", 0));
}

#[test]
fn test_is_word_boundary_match_after_space() {
    // "W" in "Hello World" at position 6
    assert!(is_word_boundary_match("Hello World", 6));
}

#[test]
fn test_is_word_boundary_match_after_dash() {
    // "c" in "git-commit" at position 4
    assert!(is_word_boundary_match("git-commit", 4));
}

#[test]
fn test_is_word_boundary_match_camel_case() {
    // "C" in "gitCommit" at position 3
    assert!(is_word_boundary_match("gitCommit", 3));
}

#[test]
fn test_is_word_boundary_match_mid_word() {
    // "e" in "Hello" at position 1 - NOT a word boundary
    assert!(!is_word_boundary_match("Hello", 1));
}

#[test]
fn test_is_exact_name_match() {
    assert!(is_exact_name_match("Hello", "hello"));
    assert!(is_exact_name_match("AI Chat", "ai chat"));
    assert!(!is_exact_name_match("Hello World", "hello"));
    assert!(!is_exact_name_match("Hi", "hello"));
}

// ============================================
// Search scoring tests
// ============================================

pub(super) fn make_script(name: &str, desc: Option<&str>) -> Arc<Script> {
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        description: desc.map(|d| d.to_string()),
        ..Default::default()
    })
}

#[test]
fn test_exact_name_match_ranks_first() {
    let scripts = vec![
        make_script("Notes Helper", Some("Manages notes")),
        make_script("Notes", Some("Take quick notes")),
        make_script("Notebook Viewer", Some("View notebooks")),
    ];
    let results = fuzzy_search_scripts(&scripts, "Notes");
    assert!(!results.is_empty());
    // Exact match "Notes" should be first
    assert_eq!(results[0].script.name, "Notes");
}

#[test]
fn test_word_boundary_bonus() {
    let scripts = vec![
        make_script("Renewal Plan", Some("Renew subscriptions")),
        make_script("New Tab", Some("Open new tab")),
    ];
    let results = fuzzy_search_scripts(&scripts, "new");
    assert!(!results.is_empty());
    // "New Tab" should rank higher because "new" is at a word start
    assert_eq!(results[0].script.name, "New Tab");
}

#[test]
fn test_single_char_query_no_nucleo() {
    // With MIN_FUZZY_QUERY_LEN=2, single char queries should only use
    // substring matching, not nucleo fuzzy. This reduces false positives.
    let scripts = vec![
        make_script("X Tool", None),
        make_script("Backup Files", None),
    ];
    let results = fuzzy_search_scripts(&scripts, "x");
    // "X Tool" should match, but "Backup Files" should only match if
    // it actually contains "x" as substring in name/filename/path
    for r in &results {
        let name_lower = r.script.name.to_lowercase();
        let filename_lower = r.filename.to_lowercase();
        let path_lower = r.script.path.to_string_lossy().to_lowercase();
        assert!(
            name_lower.contains('x') || filename_lower.contains('x') || path_lower.contains('x'),
            "Single-char query should only match via substring, not fuzzy: {}",
            r.script.name
        );
    }
}

#[test]
fn test_compute_match_indices_for_result_handles_unicode_normalization_in_name() {
    let scripts = vec![make_script("Café Utility", Some("Unicode accent test"))];
    let mut matches = fuzzy_search_scripts(&scripts, "cafe");
    assert!(
        !matches.is_empty(),
        "Expected fuzzy search to match Unicode-normalized script name"
    );

    let result = SearchResult::Script(matches.remove(0));
    let indices = compute_match_indices_for_result(&result, "cafe");

    assert!(
        !indices.name_indices.is_empty(),
        "Unicode-normalized name match should produce highlight indices"
    );
}

#[test]
fn test_compute_match_indices_for_result_handles_unicode_normalization_in_description() {
    let scripts = vec![make_script(
        "Invoice Tool",
        Some("Résumé template generator"),
    )];
    let mut matches = fuzzy_search_scripts(&scripts, "resume");
    assert!(
        !matches.is_empty(),
        "Expected fuzzy search to match Unicode-normalized description"
    );

    let result = SearchResult::Script(matches.remove(0));
    let indices = compute_match_indices_for_result(&result, "resume");

    assert!(
        !indices.description_indices.is_empty(),
        "Unicode-normalized description match should produce highlight indices"
    );
}

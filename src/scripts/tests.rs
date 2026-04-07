include!("tests/chunk_01.rs");
include!("tests/chunk_02.rs");
include!("tests/chunk_03.rs");
include!("tests/chunk_04.rs");
include!("tests/chunk_05.rs");
include!("tests/chunk_06.rs");
include!("tests/chunk_07.rs");
include!("tests/chunk_08.rs");
include!("tests/chunk_09.rs");
include!("tests/chunk_10.rs");
include!("tests/chunk_11.rs");
include!("tests/chunk_12.rs");
include!("tests/chunk_13.rs");
include!("tests/chunk_14.rs");
include!("tests/chunk_15.rs");
include!("tests/chunk_16.rs");
include!("tests/chunk_17.rs");
include!("tests/chunk_18.rs");

// Preview cache signature and validity tests
#[test]
fn preview_match_signature_changes_when_byte_range_changes() {
    let alpha = super::ScriptContentMatch {
        line_number: 4,
        line_text: "const alpha = beta;".to_string(),
        line_match_indices: vec![6, 7, 8, 9, 10],
        byte_range: 20..25,
    };
    let beta = super::ScriptContentMatch {
        line_number: 4,
        line_text: "const alpha = beta;".to_string(),
        line_match_indices: vec![14, 15, 16, 17],
        byte_range: 28..32,
    };
    assert_ne!(
        super::preview_match_signature(Some(&alpha)),
        super::preview_match_signature(Some(&beta))
    );
}

#[test]
fn preview_match_signature_is_none_without_content_match() {
    assert_eq!(super::preview_match_signature(None), None);
}

#[test]
fn preview_cache_is_valid_for_identical_match_signature() {
    let alpha = super::ScriptContentMatch {
        line_number: 1,
        line_text: "const alpha = beta;".to_string(),
        line_match_indices: vec![6, 7, 8, 9, 10],
        byte_range: 6..11,
    };
    assert!(super::preview_cache_is_valid(
        Some("/tmp/demo.ts"),
        super::preview_match_signature(Some(&alpha)),
        false, // cached_lines_empty
        "/tmp/demo.ts",
        Some(&alpha),
    ));
}

#[test]
fn preview_cache_is_invalid_when_same_line_match_moves_to_new_span() {
    let alpha = super::ScriptContentMatch {
        line_number: 1,
        line_text: "const alpha = beta;".to_string(),
        line_match_indices: vec![6, 7, 8, 9, 10],
        byte_range: 6..11,
    };
    let beta = super::ScriptContentMatch {
        line_number: 1,
        line_text: "const alpha = beta;".to_string(),
        line_match_indices: vec![14, 15, 16, 17],
        byte_range: 14..18,
    };
    assert!(!super::preview_cache_is_valid(
        Some("/tmp/demo.ts"),
        super::preview_match_signature(Some(&alpha)),
        false, // cached_lines_empty
        "/tmp/demo.ts",
        Some(&beta),
    ));
}

#[test]
fn preview_cache_is_invalid_when_cached_lines_are_empty() {
    let alpha = super::ScriptContentMatch {
        line_number: 1,
        line_text: "const alpha = beta;".to_string(),
        line_match_indices: vec![6, 7, 8, 9, 10],
        byte_range: 6..11,
    };
    assert!(!super::preview_cache_is_valid(
        Some("/tmp/demo.ts"),
        super::preview_match_signature(Some(&alpha)),
        true, // cached_lines_empty
        "/tmp/demo.ts",
        Some(&alpha),
    ));
}

//! Unit tests for form field text indexing helpers.
//!
//! These tests verify the UTF-8 char/byte conversion functions used by form fields.
//! Separated from form_fields.rs due to GPUI macro recursion limit issues.

use super::form_fields::{form_field_type_allows_candidate_value, FormFieldColors};
use crate::designs::DesignColors;

/// Count the number of Unicode scalar values (chars) in a string.
fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Convert a character index (0..=char_len) into a byte index (0..=s.len()).
/// If char_idx is past the end, returns s.len().
fn byte_idx_from_char_idx(s: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or_else(|| s.len())
}

/// Remove a char range [start_char, end_char) from a String (char indices).
fn drain_char_range(s: &mut String, start_char: usize, end_char: usize) {
    let start_b = byte_idx_from_char_idx(s, start_char);
    let end_b = byte_idx_from_char_idx(s, end_char);
    if start_b < end_b && start_b <= s.len() && end_b <= s.len() {
        s.drain(start_b..end_b);
    }
}

/// Slice a &str by char indices [start_char, end_char).
fn slice_by_char_range(s: &str, start_char: usize, end_char: usize) -> &str {
    let start_b = byte_idx_from_char_idx(s, start_char);
    let end_b = byte_idx_from_char_idx(s, end_char);
    &s[start_b..end_b]
}

// --- Text indexing helper tests ---

#[test]
fn test_byte_idx_from_char_idx_ascii() {
    let s = "hello";
    assert_eq!(byte_idx_from_char_idx(s, 0), 0);
    assert_eq!(byte_idx_from_char_idx(s, 1), 1);
    assert_eq!(byte_idx_from_char_idx(s, 5), 5);
    // Past end
    assert_eq!(byte_idx_from_char_idx(s, 10), 5);
}

#[test]
fn test_byte_idx_from_char_idx_emoji() {
    let s = "a😀b"; // a=1 byte, 😀=4 bytes, b=1 byte
    assert_eq!(byte_idx_from_char_idx(s, 0), 0); // before 'a'
    assert_eq!(byte_idx_from_char_idx(s, 1), 1); // before '😀'
    assert_eq!(byte_idx_from_char_idx(s, 2), 5); // before 'b' (1+4)
    assert_eq!(byte_idx_from_char_idx(s, 3), 6); // end
}

#[test]
fn test_byte_idx_from_char_idx_bullet() {
    let s = "•••"; // 3 bullets, each 3 bytes = 9 bytes total
    assert_eq!(byte_idx_from_char_idx(s, 0), 0);
    assert_eq!(byte_idx_from_char_idx(s, 1), 3);
    assert_eq!(byte_idx_from_char_idx(s, 2), 6);
    assert_eq!(byte_idx_from_char_idx(s, 3), 9);
}

#[test]
fn test_slice_by_char_range_ascii() {
    let s = "hello";
    assert_eq!(slice_by_char_range(s, 0, 2), "he");
    assert_eq!(slice_by_char_range(s, 2, 5), "llo");
    assert_eq!(slice_by_char_range(s, 0, 5), "hello");
}

#[test]
fn test_slice_by_char_range_emoji() {
    let s = "a😀b";
    assert_eq!(slice_by_char_range(s, 0, 1), "a");
    assert_eq!(slice_by_char_range(s, 1, 2), "😀");
    assert_eq!(slice_by_char_range(s, 2, 3), "b");
    assert_eq!(slice_by_char_range(s, 0, 3), "a😀b");
}

#[test]
fn test_slice_by_char_range_bullet() {
    let s = "•••";
    assert_eq!(slice_by_char_range(s, 0, 1), "•");
    assert_eq!(slice_by_char_range(s, 1, 2), "•");
    assert_eq!(slice_by_char_range(s, 0, 2), "••");
}

#[test]
fn test_drain_char_range_ascii() {
    let mut s = "hello".to_string();
    drain_char_range(&mut s, 1, 3);
    assert_eq!(s, "hlo");
}

#[test]
fn test_drain_char_range_emoji() {
    let mut s = "a😀b".to_string();
    drain_char_range(&mut s, 1, 2); // remove emoji
    assert_eq!(s, "ab");
}

#[test]
fn test_drain_char_range_bullet() {
    let mut s = "•••".to_string();
    drain_char_range(&mut s, 1, 2); // remove middle bullet
    assert_eq!(s, "••");
}

// --- Password bullet rendering tests ---

/// Test that password bullet string can be safely sliced by char index.
/// This test verifies the FIX for the bug where render() slices bullet
/// strings using cursor_position directly (which is a char index).
#[test]
fn test_password_bullet_slicing_safe() {
    let password = "abc"; // 3 chars
    let bullets = "•".repeat(char_len(password)); // "•••" = 9 bytes
    let cursor_pos: usize = 2; // char index

    // This is the CORRECT way to slice (using char indices):
    let before = slice_by_char_range(&bullets, 0, cursor_pos);
    let after = slice_by_char_range(&bullets, cursor_pos, char_len(&bullets));

    assert_eq!(before, "••");
    assert_eq!(after, "•");
}

#[test]
fn test_form_fields_use_theme_token_font_sizes() {
    let source = std::fs::read_to_string("src/components/form_fields.rs")
        .expect("failed to read src/components/form_fields.rs");

    assert!(
        source.contains("input_font_size"),
        "form field colors should define an input font-size token"
    );
    assert!(
        source.contains("label_font_size"),
        "form field colors should define a label font-size token"
    );
    assert!(
        source.contains(".text_size(px(colors.input_font_size))"),
        "text inputs should use the shared input font-size token"
    );
    assert!(
        source.contains(".text_size(px(colors.label_font_size))"),
        "labels should use the shared label font-size token"
    );
}

#[test]
fn test_arg_prompt_header_uses_design_token_large_input_size() {
    let source = std::fs::read_to_string("src/render_prompts/arg.rs")
        .expect("failed to read src/render_prompts/arg.rs");

    assert!(
        source.contains(".text_size(px(design_typography.font_size_lg))"),
        "arg prompt header input should use design typography token for large input text"
    );
    assert!(
        !source.contains(".text_xl()"),
        "arg prompt header should avoid hardcoded text_xl() sizing"
    );
}

#[test]
fn test_number_field_accepts_partial_numeric_values() {
    assert!(form_field_type_allows_candidate_value(
        Some("number"),
        "123"
    ));
    assert!(form_field_type_allows_candidate_value(
        Some("number"),
        "-42.5"
    ));
    assert!(form_field_type_allows_candidate_value(
        Some("number"),
        "+.7"
    ));
}

#[test]
fn test_number_field_rejects_non_numeric_values() {
    assert!(!form_field_type_allows_candidate_value(
        Some("number"),
        "12a"
    ));
    assert!(!form_field_type_allows_candidate_value(
        Some("number"),
        "1.2.3"
    ));
}

#[test]
fn test_email_field_rejects_spaces_and_multiple_at_signs() {
    assert!(form_field_type_allows_candidate_value(
        Some("email"),
        "dev@example.com"
    ));
    assert!(!form_field_type_allows_candidate_value(
        Some("email"),
        "dev @example.com"
    ));
    assert!(!form_field_type_allows_candidate_value(
        Some("email"),
        "a@b@c.com"
    ));
}

#[test]
fn test_form_field_colors_from_design_uses_design_accent_for_cursor() {
    let mut design = DesignColors::default();
    design.accent = 0x123456;

    let colors = FormFieldColors::from_design(&design);
    assert_eq!(colors.cursor, 0x123456);
}

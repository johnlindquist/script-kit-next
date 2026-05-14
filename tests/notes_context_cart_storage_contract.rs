//! Source-level contract for note-cart dedupe and note-scoped consume.

const STORAGE_SOURCE: &str = include_str!("../src/notes/storage.rs");

fn function_body<'a>(source: &'a str, signature: &str, next_signature: &str) -> &'a str {
    let start = source.find(signature).expect("function should exist");
    let rest = &source[start..];
    let end = rest.find(next_signature).unwrap_or(rest.len());
    &rest[..end]
}

// @lat: [[tests/notes-acp#Notes context cart consume and dedupe#Storage lists cart items once per dedupe key]]
#[test]
fn list_note_cart_items_deduped_uses_note_cart_dedup_key() {
    let body = function_body(
        STORAGE_SOURCE,
        "pub fn list_note_cart_items_deduped(",
        "/// Delete a cart item by ID.",
    );
    assert!(body.contains("list_note_cart_items(note_id)?"));
    assert!(body.contains("std::collections::HashSet"));
    assert!(body.contains("seen.insert(item.dedup_key())"));
}

// @lat: [[tests/notes-acp#Notes context cart consume and dedupe#Storage delete is note scoped]]
#[test]
fn delete_note_cart_items_is_transactional_and_scoped_to_note_id() {
    let body = function_body(
        STORAGE_SOURCE,
        "pub fn delete_note_cart_items(",
        "/// Convert a database row",
    );
    assert!(body.contains(".transaction()"));
    assert!(
        body.contains("DELETE FROM note_cart_items WHERE note_id = ?1 AND id = ?2"),
        "batch delete must scope by both note_id and item id"
    );
    assert!(body.contains("tx.commit()"));
}

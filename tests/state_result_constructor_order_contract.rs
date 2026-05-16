//! Source-level contract for keeping `StateResult` construction in lockstep.
//!
//! `Message::state_result(...)` is a narrow constructor wrapper around the
//! `Message::StateResult { ... }` variant, but its positional parameter list
//! is fragile because several adjacent fields share the same Rust type.

const QUERY_OPS_VARIANTS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");
const QUERY_OPS_CONSTRUCTORS: &str =
    include_str!("../src/protocol/message/constructors/query_ops.rs");

const EXPECTED_STATE_RESULT_FIELDS: &[&str] = &[
    "request_id",
    "prompt_type",
    "prompt_id",
    "surface_contract",
    "active_popup_contract",
    "active_footer",
    "placeholder",
    "input_value",
    "choice_count",
    "visible_choice_count",
    "selected_index",
    "selected_value",
    "is_focused",
    "window_visible",
    "mini_ai",
    "filter_input_decorations",
    "menu_syntax_main_hint",
    "capture_history_picker",
    "main_window_preflight",
    "actions_dialog",
    "root_file_search",
    "main_list_scroll",
    "screenshot_identity",
    "drop_state",
    "path_state",
];

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end after {start_pat}: {end_pat}"));
    &source[start..start + end_rel]
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn declared_field_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with("///") || trimmed.starts_with("//") {
                return None;
            }
            if !trimmed.ends_with(',') && !trimmed.ends_with('<') && !trimmed.ends_with(':') {
                return None;
            }
            let (name, _) = trimmed.split_once(':')?;
            let name = name.trim();
            if name == "crate" {
                return None;
            }
            is_ident(name).then(|| name.to_string())
        })
        .collect()
}

fn forwarded_field_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let name = line.trim().strip_suffix(',')?;
            is_ident(name).then(|| name.to_string())
        })
        .filter(|name| EXPECTED_STATE_RESULT_FIELDS.contains(&name.as_str()))
        .collect()
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn state_result_constructor_signature_and_forwarding_match_variant_field_order() {
    let variant = source_between(
        QUERY_OPS_VARIANTS,
        "#[serde(rename = \"stateResult\")]",
        "\n    // ============================================================\n    // ELEMENT QUERY",
    );
    let constructor = source_between(
        QUERY_OPS_CONSTRUCTORS,
        "pub fn state_result(",
        "\n    // ============================================================\n    // Constructor methods for element query",
    );
    let signature = source_between(constructor, "pub fn state_result(", ") -> Self");
    let literal = source_between(constructor, "Message::StateResult {", "\n        }\n    }");
    let expected = EXPECTED_STATE_RESULT_FIELDS
        .iter()
        .map(|field| field.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        declared_field_names(variant),
        expected,
        "StateResult variant fields changed. Update this test deliberately so reviewers see \
         whether a new field belongs before or after the repeated-type slots: \
         prompt_id/placeholder/selected_value/screenshot_identity, \
         choice_count/visible_choice_count, and is_focused/window_visible."
    );
    assert_eq!(
        declared_field_names(signature),
        expected,
        "Message::state_result parameter order must exactly match the StateResult field order. \
         Positional callers are too easy to desynchronize otherwise."
    );
    assert_eq!(
        forwarded_field_names(literal),
        expected,
        "Message::state_result must forward every parameter into Message::StateResult in the \
         same order as the variant fields."
    );
}

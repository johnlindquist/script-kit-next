//! Integration tests for Run 12 head-aware menu syntax main hint.
//!
//! These run as a separate test binary that links the rlib produced by
//! `cargo build --lib`, bypassing the lib-test-target proc-macro
//! recursion that currently blocks `cargo test --lib`.

use script_kit_gpui::menu_syntax::{
    build_menu_syntax_main_hint, input_spans_for_input_with_targets,
    main_hint::active_head_is_source_filter, mode::MenuSyntaxMode, prefix_span_for_input,
    MenuSyntaxFragmentRole, MenuSyntaxMainHintContext, MenuSyntaxMainHintSnapshot,
};

fn empty_hint_for(raw: &str) -> MenuSyntaxMainHintSnapshot {
    let mode = MenuSyntaxMode::from_input(raw);
    build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
        raw_filter_text: raw,
        mode: &mode,
        popup_snapshot: None,
        popup_selected_row_id: None,
        scripts: &[],
        scriptlets: &[],
        advanced_query_results_empty: true,
        menu_syntax_ai_proposal: None,
    })
    .expect("empty hint")
}

#[test]
fn has_partial_sh_lists_only_shortcut() {
    let hint = empty_hint_for("has:sh");
    assert!(
        hint.examples.iter().all(|e| e == "has:shortcut"),
        "has:sh examples must be exactly [has:shortcut], got {:?}",
        hint.examples
    );
    assert_eq!(hint.active_head.as_deref(), Some("has:"));
    assert_eq!(hint.active_head_value_partial.as_deref(), Some("sh"));
}

#[test]
fn has_bare_head_lists_catalog_examples() {
    let hint = empty_hint_for("has:");
    assert!(!hint.examples.is_empty());
    for token in &hint.examples {
        assert!(token.starts_with("has:"));
    }
    assert!(hint.examples.iter().any(|e| e == "has:shortcut"));
}

#[test]
fn has_unknown_field_empty_copy_suggests_known_has_fields() {
    let hint = empty_hint_for("has:notAField");
    assert!(hint.title.contains("notAField"));
    let primary = hint.primary_hint.as_deref().unwrap_or_default();
    assert!(primary.contains("has:shortcut"));
    assert!(primary.contains("has:alias"));
    assert!(primary.contains("has:menuSyntax"));
    assert_eq!(
        hint.examples,
        vec![
            "has:shortcut".to_string(),
            "has:alias".to_string(),
            "has:menuSyntax".to_string(),
        ]
    );
}

#[test]
fn clipboard_source_zero_copy_names_clipboard_entries() {
    let hint = empty_hint_for("c:zzz");
    assert_eq!(hint.title, "No clipboard entries match `zzz`.");
    assert_eq!(
        hint.primary_hint.as_deref(),
        Some("Press Esc to clear the filter.")
    );
    assert_eq!(hint.active_head.as_deref(), Some("c:"));
    assert_eq!(hint.active_head_value_partial.as_deref(), Some("zzz"));
}

#[test]
fn type_scriptlet_zero_copy_removes_type_filter() {
    let hint = empty_hint_for(":type:scriptlet zzz");
    assert_eq!(hint.title, "No scriptlets match `zzz`.");
    assert_eq!(
        hint.primary_hint.as_deref(),
        Some("Remove `type:scriptlet` to widen.")
    );
}

#[test]
fn snapshot_serializes_active_head_camel_case() {
    let hint = empty_hint_for("has:sh");
    let json = serde_json::to_value(&hint).unwrap();
    assert_eq!(
        json.get("activeHead").and_then(|v| v.as_str()),
        Some("has:")
    );
    assert_eq!(
        json.get("activeHeadValuePartial").and_then(|v| v.as_str()),
        Some("sh"),
    );
}

#[test]
fn source_heads_are_identified_for_render_gating() {
    assert!(active_head_is_source_filter("c:sub"));
    assert!(active_head_is_source_filter("clipboard: sub"));
    assert!(active_head_is_source_filter("files:report"));
    assert!(!active_head_is_source_filter("has:shortcut"));
    assert!(!active_head_is_source_filter(":type:scriptlet zzz"));
}

#[test]
fn source_heads_are_highlighted_as_input_prefixes() {
    assert_eq!(prefix_span_for_input("f: project"), Some(0..2));
    assert_eq!(prefix_span_for_input("clipboard:sub"), Some(0..10));

    let spans = input_spans_for_input_with_targets("budget f: report -notes:done", &[]);
    let prefix_ranges: Vec<_> = spans
        .iter()
        .filter(|span| span.role == MenuSyntaxFragmentRole::Prefix)
        .map(|span| span.range.clone())
        .collect();

    assert!(prefix_ranges.contains(&(7..9)), "{prefix_ranges:?}");
    assert!(prefix_ranges.contains(&(18..24)), "{prefix_ranges:?}");
}

#[test]
fn has_context_never_serializes_tag_examples() {
    for raw in ["has:", "has:s", "has:sh", "has:shortcut"] {
        let hint = empty_hint_for(raw);
        let json = serde_json::to_string(&hint).unwrap();
        assert!(!json.contains(":#work"), "{raw} leaked :#work — {json}");
        assert!(
            !json.contains(":tag:work"),
            "{raw} leaked :tag:work — {json}"
        );
        assert!(
            !json.contains(":type:script deploy"),
            "{raw} leaked :type:script deploy — {json}"
        );
    }
}

use super::*;
use super::analysis::{
    build_hybrid_snippet_plan, contains_explicit_tabstops, max_explicit_tabstop_index,
    HybridSnippetPlanKind,
};
use crate::template_variables::{
    promote_unresolved_variables_to_tabstops, substitute_variables_with_receipt, VariableContext,
};

// ============================================================================
// Snippet parser tests (migrated from inline mod tests)
// ============================================================================

#[test]
fn test_parse_simple_tabstop() {
    let snippet = ParsedSnippet::parse("$1");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Tabstop { index, .. } => assert_eq!(*index, 1),
        _ => panic!("Expected Tabstop"),
    }
    assert_eq!(snippet.text, "");
}

#[test]
fn test_parse_tabstop_with_placeholder() {
    let snippet = ParsedSnippet::parse("${1:name}");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Tabstop {
            index, placeholder, ..
        } => {
            assert_eq!(*index, 1);
            assert_eq!(placeholder.as_deref(), Some("name"));
        }
        _ => panic!("Expected Tabstop"),
    }
    assert_eq!(snippet.text, "name");
}

#[test]
fn test_parse_tabstop_with_choices() {
    let snippet = ParsedSnippet::parse("${1|a,b,c|}");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Tabstop { index, choices, .. } => {
            assert_eq!(*index, 1);
            assert_eq!(
                choices.as_ref().unwrap(),
                &vec!["a".to_string(), "b".to_string(), "c".to_string()]
            );
        }
        _ => panic!("Expected Tabstop"),
    }
    assert_eq!(snippet.text, "a");
}

#[test]
fn test_parse_text_and_tabstop() {
    let snippet = ParsedSnippet::parse("Hello $1!");
    assert_eq!(snippet.parts.len(), 3);

    match &snippet.parts[0] {
        SnippetPart::Text(t) => assert_eq!(t, "Hello "),
        _ => panic!("Expected Text"),
    }
    match &snippet.parts[1] {
        SnippetPart::Tabstop { index, .. } => assert_eq!(*index, 1),
        _ => panic!("Expected Tabstop"),
    }
    match &snippet.parts[2] {
        SnippetPart::Text(t) => assert_eq!(t, "!"),
        _ => panic!("Expected Text"),
    }

    assert_eq!(snippet.text, "Hello !");
}

#[test]
fn test_parse_escaped_dollar() {
    let snippet = ParsedSnippet::parse("$$100");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Text(t) => assert_eq!(t, "$100"),
        _ => panic!("Expected Text"),
    }
    assert_eq!(snippet.text, "$100");
}

#[test]
fn test_parse_linked_tabstops() {
    let snippet = ParsedSnippet::parse("${1:foo} and ${1:bar}");

    assert_eq!(snippet.parts.len(), 3);

    let tabstop1 = &snippet.parts[0];
    let tabstop2 = &snippet.parts[2];

    match (tabstop1, tabstop2) {
        (
            SnippetPart::Tabstop {
                index: idx1,
                placeholder: p1,
                ..
            },
            SnippetPart::Tabstop {
                index: idx2,
                placeholder: p2,
                ..
            },
        ) => {
            assert_eq!(*idx1, 1);
            assert_eq!(*idx2, 1);
            assert_eq!(p1.as_deref(), Some("foo"));
            assert_eq!(p2.as_deref(), Some("bar"));
        }
        _ => panic!("Expected two Tabstops"),
    }

    assert_eq!(snippet.tabstops.len(), 1);
    assert_eq!(snippet.tabstops[0].index, 1);
    assert_eq!(snippet.tabstops[0].ranges.len(), 2);
    assert_eq!(snippet.tabstops[0].placeholder.as_deref(), Some("foo"));
}

#[test]
fn test_parse_final_cursor() {
    let snippet = ParsedSnippet::parse("$0");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Tabstop { index, .. } => assert_eq!(*index, 0),
        _ => panic!("Expected Tabstop"),
    }
}

#[test]
fn test_parse_empty_string() {
    let snippet = ParsedSnippet::parse("");
    assert_eq!(snippet.parts.len(), 0);
    assert_eq!(snippet.text, "");
    assert_eq!(snippet.tabstops.len(), 0);
}

#[test]
fn test_tabstop_order() {
    let snippet = ParsedSnippet::parse("$3 $1 $2 $0");
    let order = snippet.tabstop_order();
    assert_eq!(order, vec![1, 2, 3, 0]);
}

#[test]
fn test_get_tabstop() {
    let snippet = ParsedSnippet::parse("${1:hello} ${2:world}");

    let t1 = snippet.get_tabstop(1).unwrap();
    assert_eq!(t1.index, 1);
    assert_eq!(t1.placeholder.as_deref(), Some("hello"));

    let t2 = snippet.get_tabstop(2).unwrap();
    assert_eq!(t2.index, 2);
    assert_eq!(t2.placeholder.as_deref(), Some("world"));

    assert!(snippet.get_tabstop(3).is_none());
}

#[test]
fn test_tabstop_ranges() {
    let snippet = ParsedSnippet::parse("Hello ${1:world}!");
    let t1 = snippet.get_tabstop(1).unwrap();
    assert_eq!(t1.ranges, vec![(6, 11)]);
}

#[test]
fn test_multiple_tabstops_with_text() {
    let snippet = ParsedSnippet::parse("function ${1:name}(${2:args}) { $0 }");
    assert_eq!(snippet.text, "function name(args) {  }");
    let order = snippet.tabstop_order();
    assert_eq!(order, vec![1, 2, 0]);
}

#[test]
fn test_simple_braced_tabstop() {
    let snippet = ParsedSnippet::parse("${1}");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Tabstop {
            index, placeholder, ..
        } => {
            assert_eq!(*index, 1);
            assert!(placeholder.is_none());
        }
        _ => panic!("Expected Tabstop"),
    }
}

#[test]
fn test_lone_dollar_preserved() {
    let snippet = ParsedSnippet::parse("$x");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Text(t) => assert_eq!(t, "$x"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_dollar_at_end() {
    let snippet = ParsedSnippet::parse("test$");
    assert_eq!(snippet.parts.len(), 1);
    match &snippet.parts[0] {
        SnippetPart::Text(t) => assert_eq!(t, "test$"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_parse_placeholder_keeps_escaped_closing_brace_literal() {
    let snippet = ParsedSnippet::parse("${1:foo\\}}");
    assert_eq!(snippet.text, "foo}");
    let t1 = snippet.get_tabstop(1).unwrap();
    assert_eq!(t1.placeholder.as_deref(), Some("foo}"));
    assert_eq!(t1.ranges, vec![(0, 4)]);
}

#[test]
fn test_parse_placeholder_preserves_backslash_before_non_special_character() {
    let snippet = ParsedSnippet::parse("${1:C:\\tmp}");
    assert_eq!(snippet.text, "C:\\tmp");
    let t1 = snippet.get_tabstop(1).unwrap();
    assert_eq!(t1.placeholder.as_deref(), Some("C:\\tmp"));
    assert_eq!(t1.ranges, vec![(0, 6)]);
}

#[test]
fn test_update_tabstops_after_insert_first_tabstop() {
    let mut snippet = ParsedSnippet::parse("${1:hello} ${2:world}");
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 5)]);
    assert_eq!(snippet.tabstops[1].ranges, vec![(6, 11)]);
    snippet.update_tabstops_after_edit(0, 0, 0, 2);
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 7)]);
    assert_eq!(snippet.tabstops[1].ranges, vec![(8, 13)]);
}

#[test]
fn test_update_tabstops_after_delete_in_first_tabstop() {
    let mut snippet = ParsedSnippet::parse("${1:hello} ${2:world}");
    snippet.update_tabstops_after_edit(0, 0, 3, 0);
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 2)]);
    assert_eq!(snippet.tabstops[1].ranges, vec![(3, 8)]);
}

#[test]
fn test_update_tabstops_after_replace_in_first_tabstop() {
    let mut snippet = ParsedSnippet::parse("${1:hello} ${2:world}");
    snippet.update_tabstops_after_edit(0, 0, 5, 2);
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 2)]);
    assert_eq!(snippet.tabstops[1].ranges, vec![(3, 8)]);
}

#[test]
fn test_update_tabstops_no_change_before_edit() {
    let mut snippet = ParsedSnippet::parse("prefix ${1:hello}");
    assert_eq!(snippet.tabstops[0].ranges, vec![(7, 12)]);
    snippet.update_tabstops_after_edit(usize::MAX, 2, 0, 2);
    assert_eq!(snippet.tabstops[0].ranges, vec![(9, 14)]);
}

#[test]
fn test_update_tabstops_linked_tabstops() {
    let mut snippet = ParsedSnippet::parse("${1:foo} and ${1:bar}");
    assert_eq!(snippet.tabstops.len(), 1);
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 3), (8, 11)]);
    snippet.update_tabstops_after_edit(0, 0, 0, 2);
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 5), (10, 13)]);
}

#[test]
fn test_update_tabstops_clamps_range_when_delete_exceeds_current_tabstop() {
    let mut snippet = ParsedSnippet::parse("${1:abc}${2:def}");
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 3)]);
    assert_eq!(snippet.tabstops[1].ranges, vec![(3, 6)]);
    snippet.update_tabstops_after_edit(0, 0, 5, 0);
    assert_eq!(snippet.tabstops[0].ranges, vec![(0, 0)]);
    assert_eq!(snippet.tabstops[1].ranges, vec![(0, 1)]);
}

#[test]
fn test_choices_with_commas() {
    let snippet = ParsedSnippet::parse("${1|apple,banana,cherry|}");
    match &snippet.parts[0] {
        SnippetPart::Tabstop { choices, .. } => {
            let c = choices.as_ref().unwrap();
            assert_eq!(c.len(), 3);
            assert_eq!(c[0], "apple");
            assert_eq!(c[1], "banana");
            assert_eq!(c[2], "cherry");
        }
        _ => panic!("Expected Tabstop"),
    }
}

#[test]
fn test_parse_choices_preserves_backslash_before_non_special_character() {
    let snippet = ParsedSnippet::parse("${1|C:\\tmp,D:\\logs|}");
    match &snippet.parts[0] {
        SnippetPart::Tabstop { choices, .. } => {
            let c = choices.as_ref().unwrap();
            assert_eq!(c, &vec!["C:\\tmp".to_string(), "D:\\logs".to_string()]);
        }
        _ => panic!("Expected Tabstop"),
    }
}

#[test]
fn test_complex_template() {
    let template = r#"import { ${1:Component} } from '${2:react}';

export default function ${1:Component}() {
    return (
        <div>$0</div>
    );
}"#;
    let snippet = ParsedSnippet::parse(template);
    let t1 = snippet.get_tabstop(1).unwrap();
    assert_eq!(t1.ranges.len(), 2);
    assert_eq!(t1.placeholder.as_deref(), Some("Component"));
    let t2 = snippet.get_tabstop(2).unwrap();
    assert_eq!(t2.ranges.len(), 1);
    assert_eq!(t2.placeholder.as_deref(), Some("react"));
    assert_eq!(snippet.tabstop_order(), vec![1, 2, 0]);
}

#[test]
fn test_final_cursor_at_end_of_text() {
    let snippet = ParsedSnippet::parse("Hello ${1:name}!$0");
    assert_eq!(snippet.text, "Hello name!");
    assert_eq!(snippet.text.chars().count(), 11);
    let t0 = snippet.get_tabstop(0).unwrap();
    assert_eq!(t0.ranges, vec![(11, 11)]);
    assert_eq!(snippet.tabstop_order(), vec![1, 0]);
}

#[test]
fn test_final_cursor_empty_range() {
    let snippet = ParsedSnippet::parse("$0");
    let t0 = snippet.get_tabstop(0).unwrap();
    assert_eq!(t0.ranges, vec![(0, 0)]);
    assert!(t0.placeholder.is_none());
}

#[test]
fn test_final_cursor_with_placeholder() {
    let snippet = ParsedSnippet::parse("${1:hello} ${0:cursor here}");
    assert_eq!(snippet.text, "hello cursor here");
    let t0 = snippet.get_tabstop(0).unwrap();
    assert_eq!(t0.placeholder.as_deref(), Some("cursor here"));
    assert_eq!(t0.ranges, vec![(6, 17)]);
}

#[test]
fn test_multiple_tabstops_then_final_cursor() {
    let snippet = ParsedSnippet::parse("fn ${1:name}(${2:args}) { $0 }");
    assert_eq!(snippet.text, "fn name(args) {  }");
    assert_eq!(snippet.tabstop_order(), vec![1, 2, 0]);
    let t0 = snippet.get_tabstop(0).unwrap();
    assert_eq!(t0.ranges, vec![(16, 16)]);
}

#[test]
fn test_only_final_cursor() {
    let snippet = ParsedSnippet::parse("$0");
    assert_eq!(snippet.tabstops.len(), 1);
    assert_eq!(snippet.tabstop_order(), vec![0]);
}

#[test]
fn test_parse_merges_all_ranges_when_template_contains_multiple_final_cursors() {
    let snippet = ParsedSnippet::parse("$0 and $0");
    assert_eq!(snippet.text, " and ");
    assert_eq!(snippet.tabstop_order(), vec![0]);
    let t0 = snippet.get_tabstop(0).unwrap();
    assert_eq!(t0.ranges, vec![(0, 0), (5, 5)]);
}

#[test]
fn test_final_cursor_unicode() {
    let snippet = ParsedSnippet::parse("你好$0");
    assert_eq!(snippet.text, "你好");
    assert_eq!(snippet.text.chars().count(), 2);
    assert_eq!(snippet.text.len(), 6);
    let t0 = snippet.get_tabstop(0).unwrap();
    assert_eq!(t0.ranges, vec![(2, 2)]);
}

#[test]
fn test_tabstop_navigation_order_with_gaps() {
    let snippet = ParsedSnippet::parse("$5 $1 $3 $0");
    assert_eq!(snippet.tabstop_order(), vec![1, 3, 5, 0]);
}

#[test]
fn test_tabstop_navigation_order_reverse_in_template() {
    let snippet = ParsedSnippet::parse("$3 $2 $1 $0");
    assert_eq!(snippet.tabstop_order(), vec![1, 2, 3, 0]);
}

#[test]
fn test_tabstop_without_zero() {
    let snippet = ParsedSnippet::parse("${1:first} ${2:second}");
    let order = snippet.tabstop_order();
    assert_eq!(order, vec![1, 2]);
    assert_eq!(snippet.get_tabstop(0), None);
}

// ============================================================================
// Explicit tabstop detection (analysis module)
// ============================================================================

#[test]
fn explicit_tabstop_detection_ignores_named_variables() {
    assert!(!contains_explicit_tabstops("Hello ${name}"));
    assert!(!contains_explicit_tabstops("Hello {{name}}"));
}

#[test]
fn explicit_tabstop_detection_finds_numbered_forms() {
    assert!(contains_explicit_tabstops("Hello $1"));
    assert!(contains_explicit_tabstops("Hello ${1:name}"));
    assert!(contains_explicit_tabstops("Hello ${1|a,b|}"));
    assert!(contains_explicit_tabstops("Hello $0"));
}

#[test]
fn max_explicit_tabstop_index_ignores_zero_and_named() {
    assert_eq!(max_explicit_tabstop_index("Hello ${name} $0"), 0);
    assert_eq!(max_explicit_tabstop_index("Hello ${2:name} $7"), 7);
}

#[test]
fn escaped_dollar_is_not_a_tabstop() {
    assert!(!contains_explicit_tabstops("Price is $$5"));
}

// ============================================================================
// Resolution receipts
// ============================================================================

#[test]
fn receipt_tracks_resolved_and_unresolved_variables() {
    let mut ctx = VariableContext::new().with_builtins(false);
    ctx.set("date", "2026-03-26");

    let receipt = substitute_variables_with_receipt("Hi {{name}}, today is ${date}.", &ctx);

    assert_eq!(receipt.text, "Hi {{name}}, today is 2026-03-26.");
    assert_eq!(receipt.resolved_names, vec!["date".to_string()]);
    assert_eq!(receipt.unresolved_names, vec!["name".to_string()]);
}

#[test]
fn receipt_with_all_resolved() {
    let mut ctx = VariableContext::new().with_builtins(false);
    ctx.set("name", "Alice");

    let receipt = substitute_variables_with_receipt("Hello ${name}!", &ctx);

    assert_eq!(receipt.text, "Hello Alice!");
    assert_eq!(receipt.resolved_names, vec!["name".to_string()]);
    assert!(receipt.unresolved_names.is_empty());
}

#[test]
fn receipt_with_no_variables() {
    let ctx = VariableContext::new().with_builtins(false);
    let receipt = substitute_variables_with_receipt("Plain text", &ctx);

    assert_eq!(receipt.text, "Plain text");
    assert!(receipt.resolved_names.is_empty());
    assert!(receipt.unresolved_names.is_empty());
}

#[test]
fn receipt_skips_numeric_tabstop_names() {
    let ctx = VariableContext::new().with_builtins(false);
    let receipt = substitute_variables_with_receipt("Hello ${1:name}", &ctx);

    assert!(receipt.unresolved_names.is_empty());
    assert!(receipt.resolved_names.is_empty());
    assert_eq!(receipt.text, "Hello ${1:name}");
}

// ============================================================================
// Variable promotion
// ============================================================================

#[test]
fn promoted_unresolved_variables_start_after_existing_tabstops() {
    let result = promote_unresolved_variables_to_tabstops(
        "${1|Hi,Hello|} {{name}}",
        &[String::from("name")],
        2,
    );
    assert_eq!(result, "${1|Hi,Hello|} ${2:name}");
}

#[test]
fn repeated_names_share_same_promoted_index() {
    let result = promote_unresolved_variables_to_tabstops(
        "Dear {{name}}, signed {{name}}",
        &[String::from("name")],
        1,
    );
    assert_eq!(result, "Dear ${1:name}, signed ${1:name}");
}

#[test]
fn multiple_unresolved_get_sequential_indices() {
    let result = promote_unresolved_variables_to_tabstops(
        "{{first}} {{last}}",
        &[String::from("first"), String::from("last")],
        1,
    );
    assert_eq!(result, "${1:first} ${2:last}");
}

// ============================================================================
// Hybrid snippet planner
// ============================================================================

#[test]
fn hybrid_plan_promotes_unresolved_into_interactive_template() {
    let mut ctx = VariableContext::new().with_builtins(false);
    ctx.set("date", "2026-03-26");

    let plan = build_hybrid_snippet_plan("Hi {{name}}, today is ${date}.", &ctx);

    assert_eq!(plan.kind, HybridSnippetPlanKind::InteractiveTemplate);
    assert_eq!(plan.resolved_content, "Hi {{name}}, today is 2026-03-26.");
    assert_eq!(plan.template, "Hi ${1:name}, today is 2026-03-26.");
    assert_eq!(plan.unresolved_variables, vec!["name".to_string()]);
    assert!(!plan.has_explicit_tabstops);
}

#[test]
fn hybrid_plan_keeps_static_content_on_fast_path() {
    let mut ctx = VariableContext::new().with_builtins(false);
    ctx.set("date", "2026-03-26");

    let plan = build_hybrid_snippet_plan("Today is ${date}.", &ctx);

    assert_eq!(plan.kind, HybridSnippetPlanKind::ImmediatePaste);
    assert_eq!(plan.resolved_content, "Today is 2026-03-26.");
    assert_eq!(plan.template, "Today is 2026-03-26.");
    assert!(plan.unresolved_variables.is_empty());
}

#[test]
fn hybrid_plan_respects_explicit_tabstops_alongside_variables() {
    let mut ctx = VariableContext::new().with_builtins(false);
    ctx.set("date", "2026-03-26");

    let plan =
        build_hybrid_snippet_plan("${1|Hi,Hello|} {{name}}, today is ${date}.", &ctx);

    assert_eq!(plan.kind, HybridSnippetPlanKind::InteractiveTemplate);
    assert!(plan.has_explicit_tabstops);
    assert_eq!(plan.next_promoted_tabstop_index, 2);
    assert!(plan.template.contains("${2:name}"));
}

#[test]
fn hybrid_plan_with_only_explicit_tabstops() {
    let ctx = VariableContext::new().with_builtins(false);
    let plan = build_hybrid_snippet_plan("Hello ${1:world}!", &ctx);

    assert_eq!(plan.kind, HybridSnippetPlanKind::InteractiveTemplate);
    assert!(plan.has_explicit_tabstops);
    assert!(plan.unresolved_variables.is_empty());
    assert_eq!(plan.template, "Hello ${1:world}!");
}

#[test]
fn hybrid_plan_repeated_unresolved_name_shares_tabstop() {
    let ctx = VariableContext::new().with_builtins(false);
    let plan = build_hybrid_snippet_plan(
        "Dear {{name}},\n\nFollowing up about {{topic}}.\n\nThanks,\n{{name}}",
        &ctx,
    );

    assert_eq!(plan.kind, HybridSnippetPlanKind::InteractiveTemplate);
    let name_count = plan.template.matches("${1:name}").count();
    assert_eq!(name_count, 2, "template = {}", plan.template);
    assert!(plan.template.contains("${2:topic}"));
}

#[test]
fn hybrid_plan_needs_interaction_helper() {
    let ctx = VariableContext::new().with_builtins(false);

    let paste_plan = build_hybrid_snippet_plan("static text", &ctx);
    assert!(!paste_plan.needs_interaction());

    let interactive_plan = build_hybrid_snippet_plan("Hello {{name}}", &ctx);
    assert!(interactive_plan.needs_interaction());
}

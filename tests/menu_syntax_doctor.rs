//! Integration tests for `menu_syntax::doctor::validate`.
//!
//! Covers the four fixture categories named in the story spec
//! (`sdk-menu-syntax-doctor`): good (no issues), bad-slug (target slug
//! malformed), bad-field (unknown family / accepts token / missing
//! required field), and duplicate-command (two `command.v1` handlers
//! share a `head`). Each fixture asserts both the presence/absence of
//! errors AND the JSON path the diagnostic points at — so a future
//! refactor can't silently lose path precision.
//!
//! Receipt: `cargo test --test menu_syntax_doctor`.

use script_kit_gpui::menu_syntax::{doctor_validate, doctor_validate_at_path, DoctorSeverity};
use serde_json::json;

fn err_paths(value: &serde_json::Value) -> Vec<String> {
    doctor_validate(value)
        .errors()
        .map(|i| i.path.clone())
        .collect()
}

fn err_messages(value: &serde_json::Value) -> Vec<String> {
    doctor_validate(value)
        .errors()
        .map(|i| i.message.clone())
        .collect()
}

fn warning_messages(value: &serde_json::Value) -> Vec<String> {
    doctor_validate(value)
        .issues
        .into_iter()
        .filter(|i| i.severity == DoctorSeverity::Warning)
        .map(|i| i.message)
        .collect()
}

fn warning_paths(value: &serde_json::Value) -> Vec<String> {
    doctor_validate(value)
        .issues
        .into_iter()
        .filter(|i| i.severity == DoctorSeverity::Warning)
        .map(|i| i.path)
        .collect()
}

// ============================================================================
// GOOD fixtures — no errors expected.
// ============================================================================

#[test]
fn good_capture_handler_has_no_errors() {
    let v = json!([{
        "family": "capture.v1",
        "targets": ["link"],
        "accepts": ["tags", "url", "kv"],
        "label": "Save tagged link",
        "defaultHandler": true
    }]);
    let report = doctor_validate(&v);
    assert!(
        !report.has_errors(),
        "expected no errors, got: {:?}",
        report.issues
    );
}

#[test]
fn good_command_handler_has_no_errors() {
    let v = json!([{
        "family": "command.v1",
        "head": "deploy",
        "args": [{"name": "env", "required": true}],
        "flags": [{"name": "--dry-run", "alias": "-n"}]
    }]);
    let report = doctor_validate(&v);
    assert!(
        !report.has_errors(),
        "expected no errors, got: {:?}",
        report.issues
    );
}

#[test]
fn good_skill_handler_has_no_errors() {
    let v = json!([{
        "family": "skill.v1",
        "slug": "review",
        "contextRequirements": ["selection.file"]
    }]);
    let report = doctor_validate(&v);
    assert!(
        !report.has_errors(),
        "expected no errors, got: {:?}",
        report.issues
    );
}

#[test]
fn good_mixed_array_has_no_errors() {
    let v = json!([
        {"family": "capture.v1", "targets": ["todo"]},
        {"family": "command.v1", "head": "deploy"},
        {"family": "skill.v1", "slug": "review"}
    ]);
    let report = doctor_validate(&v);
    assert!(
        !report.has_errors(),
        "expected no errors, got: {:?}",
        report.issues
    );
}

// ============================================================================
// BAD-SLUG fixtures — target / head / slug shape problems.
// ============================================================================

#[test]
fn bad_capture_empty_targets_yields_error_at_targets_path() {
    let v = json!([{"family": "capture.v1", "targets": []}]);
    assert_eq!(err_paths(&v), vec!["$[0].targets"]);
    assert!(err_messages(&v)[0].contains("targets is empty"));
}

#[test]
fn bad_capture_target_with_whitespace_errors_at_indexed_path() {
    let v = json!([{"family": "capture.v1", "targets": ["bad slug"]}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].targets[0]"]);
    assert!(err_messages(&v)[0].contains("whitespace"));
}

#[test]
fn bad_command_head_with_leading_bang_errors() {
    let v = json!([{"family": "command.v1", "head": ">deploy"}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].head"]);
    let msg = &err_messages(&v)[0];
    assert!(msg.contains("should NOT include the leading"));
    assert!(
        msg.contains("\"deploy\""),
        "expected suggestion of bare slug, got: {msg}"
    );
}

#[test]
fn bad_skill_slug_with_leading_slash_errors() {
    let v = json!([{"family": "skill.v1", "slug": "/review"}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].slug"]);
    let msg = &err_messages(&v)[0];
    assert!(msg.contains("should NOT include the leading"));
    assert!(msg.contains("\"review\""));
}

#[test]
fn bad_capture_target_non_string_errors_with_type_label() {
    let v = json!([{"family": "capture.v1", "targets": [42]}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].targets[0]"]);
    assert!(err_messages(&v)[0].contains("must be a string"));
    assert!(err_messages(&v)[0].contains("number"));
}

// ============================================================================
// BAD-FIELD fixtures — unknown family / accepts token / missing required.
// ============================================================================

#[test]
fn bad_unknown_family_errors_at_family_path() {
    let v = json!([{"family": "capture.v2", "targets": ["todo"]}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].family"]);
    let msg = &err_messages(&v)[0];
    assert!(msg.contains("unknown family"));
    assert!(msg.contains("capture.v1"));
    assert!(msg.contains("command.v1"));
    assert!(msg.contains("skill.v1"));
}

#[test]
fn bad_unknown_accepts_token_errors_at_indexed_path() {
    let v = json!([{
        "family": "capture.v1",
        "targets": ["todo"],
        "accepts": ["tags", "nonsense", "kv"]
    }]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].accepts[1]"]);
    let msg = &err_messages(&v)[0];
    assert!(msg.contains("unknown accepts token `nonsense`"));
    assert!(msg.contains("tags"));
}

#[test]
fn unknown_capture_requirement_tokens_warn_at_indexed_paths() {
    let v = json!([{
        "family": "capture.v1",
        "targets": ["expense"],
        "required": ["body", "location"],
        "optional": ["date:start", "attachment"],
        "forbidden": ["url"]
    }]);

    let report = doctor_validate(&v);
    assert!(
        !report.has_errors(),
        "unknown requirement tokens should warn, not fail: {:?}",
        report.issues
    );
    let paths = warning_paths(&v);
    assert!(paths.contains(&"$[0].required[1]".to_string()));
    assert!(paths.contains(&"$[0].optional[1]".to_string()));
    let messages = warning_messages(&v);
    assert!(messages
        .iter()
        .any(|message| message.contains("unknown required token `location`")));
    assert!(messages
        .iter()
        .any(|message| message.contains("unknown optional token `attachment`")));
}

#[test]
fn malformed_capture_requirement_lists_error_at_paths() {
    let v = json!([{
        "family": "capture.v1",
        "targets": ["expense"],
        "required": "body",
        "optional": [42],
        "forbidden": ["url"]
    }]);

    let paths = err_paths(&v);
    assert!(paths.contains(&"$[0].required".to_string()));
    assert!(paths.contains(&"$[0].optional[0]".to_string()));
    let messages = err_messages(&v);
    assert!(messages
        .iter()
        .any(|message| message.contains("required must be an array")));
    assert!(messages
        .iter()
        .any(|message| message.contains("optional entry must be a string")));
}

#[test]
fn bad_capture_missing_targets_errors_at_targets_path() {
    let v = json!([{"family": "capture.v1"}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].targets"]);
    assert!(err_messages(&v)[0].contains("requires `targets`"));
}

#[test]
fn bad_command_missing_head_errors_at_head_path() {
    let v = json!([{"family": "command.v1"}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].head"]);
    assert!(err_messages(&v)[0].contains("requires `head`"));
}

#[test]
fn bad_skill_missing_slug_errors_at_slug_path() {
    let v = json!([{"family": "skill.v1"}]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].slug"]);
    assert!(err_messages(&v)[0].contains("requires `slug`"));
}

#[test]
fn bad_command_arg_missing_name_errors_at_args_index_path() {
    let v = json!([{
        "family": "command.v1",
        "head": "deploy",
        "args": [{}]
    }]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0].args[0].name"]);
    assert!(err_messages(&v)[0].contains("missing required `name`"));
}

#[test]
fn bad_top_level_string_errors_at_dollar() {
    let v = json!("not an object");
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$"]);
    assert!(err_messages(&v)[0].contains("must be an array"));
}

#[test]
fn validate_at_path_prefixes_array_item_diagnostics() {
    let v = json!([{"family": "capture.v1", "targets": "todo"}]);
    let report = doctor_validate_at_path(&v, "$.menuSyntax");
    let paths: Vec<_> = report.errors().map(|i| i.path.as_str()).collect();
    assert_eq!(paths, vec!["$.menuSyntax[0].targets"]);
}

#[test]
fn validate_at_path_prefixes_duplicate_command_indices() {
    let v = json!([
        {"family": "command.v1", "head": "deploy"},
        {"family": "command.v1", "head": "deploy"}
    ]);
    let report = doctor_validate_at_path(&v, "$.metadata.menuSyntax");
    let paths: Vec<_> = report.errors().map(|i| i.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["$.metadata.menuSyntax[0], $.metadata.menuSyntax[1]"]
    );
}

#[test]
fn malformed_command_args_and_flags_shapes_get_indexed_paths() {
    let v = json!([{
        "family": "command.v1",
        "head": "deploy",
        "args": "env",
        "flags": [42, {"name": 42}, {"name": ""}]
    }]);
    let paths = err_paths(&v);
    assert!(paths.contains(&"$[0].args".to_string()));
    assert!(paths.contains(&"$[0].flags[0]".to_string()));
    assert!(paths.contains(&"$[0].flags[1].name".to_string()));
    assert!(paths.contains(&"$[0].flags[2].name".to_string()));
}

// ============================================================================
// DUPLICATE-COMMAND fixtures — same command head registered twice.
// ============================================================================

#[test]
fn duplicate_command_head_errors_with_both_indices_in_path() {
    let v = json!([
        {"family": "command.v1", "head": "deploy"},
        {"family": "command.v1", "head": "deploy"}
    ]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0], $[1]"]);
    assert!(err_messages(&v)[0].contains("duplicate command.v1 head `deploy`"));
    assert!(err_messages(&v)[0].contains("registered 2 times"));
}

#[test]
fn duplicate_command_head_case_insensitive() {
    let v = json!([
        {"family": "command.v1", "head": "Deploy"},
        {"family": "command.v1", "head": "DEPLOY"},
        {"family": "command.v1", "head": "deploy"}
    ]);
    let paths = err_paths(&v);
    assert_eq!(paths, vec!["$[0], $[1], $[2]"]);
    assert!(err_messages(&v)[0].contains("registered 3 times"));
}

#[test]
fn distinct_command_heads_have_no_duplicate_error() {
    let v = json!([
        {"family": "command.v1", "head": "deploy"},
        {"family": "command.v1", "head": "rollback"}
    ]);
    let report = doctor_validate(&v);
    assert!(
        !report.has_errors(),
        "expected no errors, got: {:?}",
        report.issues
    );
}

// ============================================================================
// MIXED — multiple errors in one input.
// ============================================================================

#[test]
fn multiple_errors_collected_with_distinct_paths() {
    let v = json!([
        {"family": "unknownFamily"},
        {"family": "capture.v1", "targets": ["bad slug"]},
        {"family": "command.v1"}
    ]);
    let paths = err_paths(&v);
    assert!(paths.contains(&"$[0].family".to_string()));
    assert!(paths.contains(&"$[1].targets[0]".to_string()));
    assert!(paths.contains(&"$[2].head".to_string()));
}

#[test]
fn unknown_capture_target_is_warning_not_error() {
    // Custom slugs are allowed — the doctor only warns ("you sure this
    // is intentional?") so authors can declare new targets without
    // tripping the CLI exit code.
    let v = json!([{"family": "capture.v1", "targets": ["expense"]}]);
    let report = doctor_validate(&v);
    assert!(!report.has_errors());
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.severity == DoctorSeverity::Warning && i.message.contains("not a built-in")),
        "expected a warning about non-built-in target, got: {:?}",
        report.issues
    );
}

#[test]
fn core_capture_targets_do_not_warn_as_custom() {
    for target in ["todo", "cal", "note", "social", "link"] {
        let v = json!([{"family": "capture.v1", "targets": [target]}]);
        let warnings = warning_messages(&v);
        assert!(
            warnings
                .iter()
                .all(|message| !message.contains("not a built-in")),
            "`{target}` should not warn as custom, got: {warnings:?}"
        );
    }
}

#[test]
fn mcal_is_known_special_case_for_doctor() {
    let v = json!([{"family": "capture.v1", "targets": ["mcal"]}]);
    let report = doctor_validate(&v);

    assert!(!report.has_errors());
    assert!(
        report.issues.iter().all(|issue| {
            !(issue.severity == DoctorSeverity::Warning && issue.message.contains("not a built-in"))
        }),
        "mcal is parser/schema-known and should not warn as custom: {:?}",
        report.issues
    );
}

#[test]
fn shipped_dynamic_targets_warn_as_metadata_driven_custom_targets() {
    for target in ["gcal", "github", "expense", "snippet", "fixture"] {
        let v = json!([{"family": "capture.v1", "targets": [target]}]);
        let report = doctor_validate(&v);

        assert!(!report.has_errors());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.severity == DoctorSeverity::Warning
                    && i.message.contains("not a built-in")),
            "`{target}` should warn as metadata-driven custom target, got: {:?}",
            report.issues
        );
    }
}

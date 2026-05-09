//! Pin test for [[src/menu_syntax/actions.rs#current_actions]] ordering.
//!
//! Defended refactor: A contributor extracts the per-state action construction
//! (capture_actions/refine_actions/command_actions) into a `HashMap<ActionKind,
//! ActionMeta>` registry for "extensibility", iterating it to build the Vec.
//! HashMap iteration order is unspecified — this would silently shuffle the
//! action rows in the Cmd+K dialog. Since the dialog displays actions in Vec
//! order with the 1st row pre-selected and binds keyboard shortcuts by index,
//! shuffling breaks both the visual contract and any persisted user muscle
//! memory.
//!
//! These assertions encode the exact ordering and id sequence the
//! actions-dialog renders. If a refactor passes this test, the resulting Vec
//! is observably equivalent regardless of the construction strategy.
//!
//! Receipt: `cargo test --test menu_syntax_actions_ordering_pin`.

use script_kit_gpui::menu_syntax::capture_schema::builtin_schema;
use script_kit_gpui::menu_syntax::payload::{CaptureAlias, CaptureInvocation};
use script_kit_gpui::menu_syntax::{
    current_menu_syntax_actions as current_actions, parse_advanced_query, MenuSyntaxAction,
    MenuSyntaxActionKind, MenuSyntaxActionState,
};

fn capture_payload(target: &str, body: &str) -> CaptureInvocation {
    CaptureInvocation {
        target: target.to_string(),
        alias_form: CaptureAlias::CapturePrefix,
        body: body.to_string(),
        tags: vec![],
        priority: None,
        url: None,
        duration: None,
        kv: vec![],
        date_phrases: vec![],
        raw: format!("+{target} {body}"),
    }
}

fn ids(actions: &[MenuSyntaxAction]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

#[test]
fn capture_state_emits_five_actions_in_fixed_order() {
    let payload = capture_payload("todo", "Buy milk");
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_actions(&state);
    assert_eq!(
        ids(&actions),
        vec![
            "capture.cancel",
            "capture.save_and_copy_id",
            "capture.edit_payload_json",
            "capture.change_handler",
            "capture.open_browser",
        ],
        "capture action ordering must be stable; HashMap-based registry refactor would shuffle these"
    );
}

#[test]
fn capture_state_appends_default_time_when_cal_needs_date() {
    let payload = capture_payload("cal", "Design review"); // no date_phrases
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_actions(&state);
    assert_eq!(
        ids(&actions),
        vec![
            "capture.cancel",
            "capture.save_and_copy_id",
            "capture.edit_payload_json",
            "capture.change_handler",
            "capture.open_browser",
            "capture.default_time_today_9am",
        ],
        "default-time row must be APPENDED (last), not inserted mid-list"
    );
    // Pin the kind payload so a refactor that swaps "today 9am" for a Local
    // chrono::Now lookup (which would be non-deterministic for tests) is caught.
    let last = actions.last().unwrap();
    assert!(matches!(
        &last.kind,
        MenuSyntaxActionKind::DefaultTime { phrase } if phrase == "today 9am"
    ));
}

#[test]
fn capture_state_without_schema_omits_default_time_row() {
    let payload = capture_payload("github", "Open PR"); // unknown target
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "github",
        payload: &payload,
        schema: None,
    };
    let actions = current_actions(&state);
    assert_eq!(actions.len(), 5, "no schema → no default-time append");
    assert!(!ids(&actions).contains(&"capture.default_time_today_9am"));
}

#[test]
fn refine_state_emits_four_actions_in_fixed_order() {
    let query = parse_advanced_query(":foo bar");
    let state = MenuSyntaxActionState::RefineQuery { query: &query };
    let actions = current_actions(&state);
    assert_eq!(
        ids(&actions),
        vec![
            "refine.save_named_search",
            "refine.add_pinned",
            "refine.open_builder",
            "refine.copy_filter",
        ],
        "refine action ordering pinned against HashMap-based registry refactor"
    );
}

#[test]
fn command_state_with_empty_argv_emits_three_actions_no_run_with_last() {
    let argv: Vec<String> = vec![];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let actions = current_actions(&state);
    assert_eq!(
        ids(&actions),
        vec![
            "command.show_schema",
            "command.edit_argv",
            "command.edit_script",
        ],
        "empty argv must omit run_with_last_argv (it has no last argv to replay)"
    );
}

#[test]
fn command_state_with_argv_inserts_run_with_last_at_index_2() {
    let argv = vec!["--prod".to_string(), "--dry-run".to_string()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let actions = current_actions(&state);
    assert_eq!(
        ids(&actions),
        vec![
            "command.show_schema",
            "command.edit_argv",
            "command.run_with_last_argv",
            "command.edit_script",
        ],
        "run_with_last_argv MUST land at index 2 (between edit_argv and edit_script) — \
         a HashMap registry refactor would land it last (or first, or anywhere)"
    );
}

#[test]
fn capture_save_and_copy_id_disabled_when_body_empty() {
    let mut payload = capture_payload("todo", "");
    payload.body = "   ".to_string(); // whitespace-only also empty
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_actions(&state);
    let save = actions
        .iter()
        .find(|a| a.id == "capture.save_and_copy_id")
        .expect("save_and_copy_id row must exist regardless of enabled state");
    assert!(
        !save.enabled,
        "save_and_copy_id must be disabled when body is empty/whitespace — pin against \
         a refactor that filters disabled actions out of the Vec entirely"
    );
}

#[test]
fn current_actions_is_deterministic_across_repeated_calls() {
    // Pin against any future memoization or interior mutability that could
    // make repeated calls return different Vec orderings (e.g. caching with
    // unstable hashing).
    let payload = capture_payload("cal", "Design review");
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let r1 = current_actions(&state);
    let r2 = current_actions(&state);
    let r3 = current_actions(&state);
    assert_eq!(r1, r2);
    assert_eq!(r2, r3);
}

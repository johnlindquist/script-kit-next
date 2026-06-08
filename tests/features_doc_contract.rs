const FEATURES: &str = include_str!("../FEATURES.md");

fn feature_matrix() -> &'static str {
    FEATURES
        .split("\n## Feature Matrix\n")
        .nth(1)
        .expect("FEATURES.md must contain Feature Matrix")
        .split("\n## Deferred\n")
        .next()
        .expect("FEATURES.md must contain Deferred after Feature Matrix")
}

#[test]
fn features_doc_lists_active_qa_contracts_and_excludes_deferred_scope() {
    let matrix = feature_matrix();
    let active_count = matrix.matches("\n### ").count();

    assert_eq!(
        active_count, 49,
        "Kit Store is deferred, so the active matrix should contain 49 QA contracts"
    );
    assert!(
        !FEATURES.contains("Kit Store") && !FEATURES.contains("kit-store"),
        "Kit Store should be omitted from FEATURES.md until product scope is settled"
    );
}

#[test]
fn features_doc_applies_agent_chat_pi_backend_terminology() {
    let matrix_without_code = feature_matrix()
        .split('`')
        .enumerate()
        .filter_map(|(index, part)| (index % 2 == 0).then_some(part))
        .collect::<String>();

    assert!(
        matrix_without_code.contains("Agent Chat with Pi Backend"),
        "Agent Chat with Pi Backend should be the user-facing terminology"
    );
    assert!(
        !matrix_without_code.contains("AgentChat"),
        "Rust-style AgentChat identifiers should not appear in user-facing feature prose"
    );
}

#[test]
fn features_doc_locks_user_feedback_for_cwd_and_mini_mode() {
    assert!(
        FEATURES.contains("`>` must not select cwd. Tab is the cwd trigger."),
        "cwd trigger feedback must be explicit"
    );
    assert!(
        FEATURES.contains("Mini Mode is documented as WIP")
            && FEATURES.contains("experimental/WIP documentation"),
        "Mini Mode should be documented as experimental/WIP only"
    );
}

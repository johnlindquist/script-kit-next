const SCRIPTLETS: &str = include_str!("../src/app_actions/handle_action/scriptlets.rs");

#[test]
fn scriptlet_dynamic_failure_detail_is_named_state() {
    assert!(
        SCRIPTLETS.contains("enum ScriptletDynamicFailureDetail")
            && SCRIPTLETS.contains("Stderr(String)")
            && SCRIPTLETS.contains("Unknown"),
        "scriptlet dynamic execution should classify failure details with named state"
    );
    assert!(
        SCRIPTLETS.contains("fn from_stderr(stderr: String) -> Self")
            && SCRIPTLETS.contains("Self::Unknown")
            && SCRIPTLETS.contains("Self::Stderr(stderr)")
            && SCRIPTLETS.contains("fn message(self) -> String")
            && SCRIPTLETS.contains("\"Unknown error\".to_string()"),
        "scriptlet dynamic failure detail should own stderr fallback copy"
    );
}

#[test]
fn scriptlet_dynamic_execution_result_uses_failure_detail_state() {
    assert!(
        SCRIPTLETS.contains("ScriptletDynamicFailureDetail::from_stderr(result.stderr).message()"),
        "scriptlet dynamic execution failures should route stderr through the named detail state"
    );
    assert!(
        !SCRIPTLETS.contains("let message = if result.stderr.is_empty()"),
        "scriptlet dynamic execution must not regress to inline stderr-empty branching"
    );
    assert!(
        SCRIPTLETS.contains("Some(format!(\"Failed to execute action: {message}\"))"),
        "scriptlet dynamic error toast should preserve the execution failure message"
    );
}

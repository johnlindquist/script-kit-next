const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");

// How a process count becomes an outcome is unit-tested in
// `utility_builtin_actions::utility_builtin_action_tests`.
#[test]
fn utility_process_builtin_routes_noop_and_stop_paths_through_outcome() {
    assert!(
        BUILTIN_EXECUTION.contains("let outcome = action.outcome(process_count)")
            && BUILTIN_EXECUTION.contains("if !outcome.should_stop_processes()")
            && BUILTIN_EXECUTION.contains("action.success_hud(outcome.process_count())"),
        "Stop All Processes should route its notice and kill-path decisions through the named outcome"
    );
    assert!(
        !BUILTIN_EXECUTION.contains("if process_count == 0"),
        "Stop All Processes must not regress to direct active-count branching"
    );
}

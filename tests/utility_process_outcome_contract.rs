const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");

#[test]
fn utility_process_builtin_uses_named_outcome_state() {
    assert!(
        BUILTIN_EXECUTION.contains("enum UtilityProcessBuiltinOutcome")
            && BUILTIN_EXECUTION.contains("NoRunningProcesses")
            && BUILTIN_EXECUTION.contains("StopRequested { process_count: usize }"),
        "Stop All Processes should classify active-count state with a named outcome"
    );
    assert!(
        BUILTIN_EXECUTION
            .contains("fn outcome(self, process_count: usize) -> UtilityProcessBuiltinOutcome")
            && BUILTIN_EXECUTION.contains("0 => UtilityProcessBuiltinOutcome::NoRunningProcesses")
            && BUILTIN_EXECUTION.contains(
                "process_count => UtilityProcessBuiltinOutcome::StopRequested { process_count }"
            ),
        "Stop All Processes should map process counts to outcome state in one transition"
    );
}

#[test]
fn utility_process_builtin_routes_noop_and_stop_paths_through_outcome() {
    assert!(
        BUILTIN_EXECUTION.contains("fn should_stop_processes(self) -> bool")
            && BUILTIN_EXECUTION.contains("fn process_count(self) -> usize")
            && BUILTIN_EXECUTION.contains("let outcome = action.outcome(process_count)")
            && BUILTIN_EXECUTION.contains("if !outcome.should_stop_processes()")
            && BUILTIN_EXECUTION.contains("action.success_hud(outcome.process_count())"),
        "Stop All Processes should route HUD and kill-path decisions through the named outcome"
    );
    assert!(
        !BUILTIN_EXECUTION.contains("if process_count == 0"),
        "Stop All Processes must not regress to direct active-count branching"
    );
}

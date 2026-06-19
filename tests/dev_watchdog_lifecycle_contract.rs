//! Source-level contracts for dev crash watchdog classification.

const WATCHDOG_SH: &str = include_str!("../scripts/agentic/dev-crash-watchdog.sh");
const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");
const SUPERVISOR_PY: &str = include_str!("../scripts/agentic/session-supervisor.py");

#[test]
fn session_start_uses_supervisor_for_structured_app_exit_receipts() {
    for needle in [
        "session-supervisor.py",
        "--session-dir \"$sdir\"",
        "--generation \"$session_generation\"",
        "app-exit.json",
        "supervisor_pid",
    ] {
        assert!(
            SESSION_SH.contains(needle),
            "session.sh must preserve supervisor launch/receipt marker: {needle}"
        );
    }
}

#[test]
fn supervisor_writes_pid_lifecycle_and_app_exit_json() {
    for needle in [
        "app_process_exited",
        "app-exit.json",
        "lifecycle.ndjson",
        "\"cleanExit\": return_code == 0",
        "pid_path.write_text",
    ] {
        assert!(
            SUPERVISOR_PY.contains(needle),
            "session-supervisor.py must preserve structured exit marker: {needle}"
        );
    }
}

#[test]
fn watchdog_does_not_relaunch_clean_app_exits() {
    for needle in [
        "exit_receipt_for_pid",
        "exit_receipt_is_clean",
        "app exited cleanly",
        "not relaunching",
        "crash_count=0",
    ] {
        assert!(
            WATCHDOG_SH.contains(needle),
            "watchdog must preserve clean-exit classification marker: {needle}"
        );
    }
}

#[test]
fn watchdog_classifies_abnormal_exit_without_scraping_app_log() {
    assert!(
        WATCHDOG_SH.contains("APP EXITED ABNORMALLY"),
        "watchdog should classify nonzero process exits separately from native crashes"
    );
    assert!(
        WATCHDOG_SH.contains("APP DIED WITHOUT EXIT RECEIPT"),
        "watchdog should make missing supervisor receipts explicit"
    );
    assert!(
        !WATCHDOG_SH.contains("Quit menu item clicked"),
        "watchdog must not parse app.log clean-quit copy; use app-exit.json receipts keyed by pid"
    );
}

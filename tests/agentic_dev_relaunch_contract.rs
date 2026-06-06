//! Source-level contracts for `scripts/agentic/dev-relaunch.sh`.

const DEV_RELAUNCH_SH: &str = include_str!("../scripts/agentic/dev-relaunch.sh");

#[test]
fn dev_relaunch_preserves_session_start_json_on_failure() {
    let start_marker = "RESULT=\"$(bash \"${SESSION_SCRIPT}\" start \"${SESSION_NAME}\")\"";
    let status_marker = "START_STATUS=$?";
    let print_marker = "printf '%s\\n' \"${RESULT}\"";
    let exit_marker = "exit \"${START_STATUS}\"";

    let start_pos = DEV_RELAUNCH_SH
        .find(start_marker)
        .expect("dev-relaunch.sh must capture session.sh start stdout");
    let status_pos = DEV_RELAUNCH_SH
        .find(status_marker)
        .expect("dev-relaunch.sh must capture session.sh start exit status");
    let print_pos = DEV_RELAUNCH_SH
        .find(print_marker)
        .expect("dev-relaunch.sh must print captured session.sh start stdout");
    let exit_pos = DEV_RELAUNCH_SH
        .find(exit_marker)
        .expect("dev-relaunch.sh must exit with the original start status");

    assert!(
        DEV_RELAUNCH_SH.contains("set +e\nRESULT=\"$(bash \"${SESSION_SCRIPT}\" start \"${SESSION_NAME}\")\""),
        "dev-relaunch.sh must disable `set -e` around session.sh start so nonzero starts do not swallow JSON"
    );
    assert!(
        start_pos < status_pos && status_pos < print_pos && print_pos < exit_pos,
        "dev-relaunch.sh must capture status, print JSON, then return the original start status"
    );
}

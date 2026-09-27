//! Source-level contracts for `scripts/agentic/dev-relaunch.sh`.

const DEV_RELAUNCH_SH: &str = include_str!("../scripts/agentic/dev-relaunch.sh");

fn line_index(lines: &[&str], line: &str, message: &str) -> usize {
    lines.iter().position(|l| *l == line).expect(message)
}

#[test]
fn dev_relaunch_preserves_session_start_json_on_failure() {
    let lines: Vec<&str> = DEV_RELAUNCH_SH.lines().map(str::trim).collect();

    let start_pos = line_index(
        &lines,
        "RESULT=\"$(bash \"$SESSION_SCRIPT\" start \"$SESSION_NAME\")\"",
        "dev-relaunch.sh must capture session.sh start stdout",
    );
    let status_pos = line_index(
        &lines,
        "START_STATUS=$?",
        "dev-relaunch.sh must capture session.sh start exit status",
    );
    let print_pos = line_index(
        &lines,
        "printf '%s\\n' \"$RESULT\"",
        "dev-relaunch.sh must print captured session.sh start stdout",
    );
    let exit_pos = line_index(
        &lines,
        "exit \"$START_STATUS\"",
        "dev-relaunch.sh must exit with the original start status",
    );

    assert!(
        start_pos > 0 && lines[start_pos - 1] == "set +e",
        "dev-relaunch.sh must disable `set -e` around session.sh start so nonzero starts do not swallow JSON"
    );
    assert!(
        start_pos < status_pos && status_pos < print_pos && print_pos < exit_pos,
        "dev-relaunch.sh must capture status, print JSON, then return the original start status"
    );
}

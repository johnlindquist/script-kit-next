//! Integration test for the `menu-syntax-doctor` CLI binary (Pass 37).
//!
//! Exercises the four exit-code branches (good vs bad fixture, json vs
//! human output, stdin vs --path) plus the argv-error branch (exit 2).

use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_menu-syntax-doctor"))
}

fn good_capture_v1_json() -> &'static str {
    r#"{"family":"capture.v1","slug":"todo","targets":["todo"],"required":["body"]}"#
}

fn bad_unknown_family_json() -> &'static str {
    r#"{"family":"bogus.v1","slug":"todo","targets":["todo"]}"#
}

fn run_with_stdin(input: &str, args: &[&str]) -> (i32, String, String) {
    let mut child = bin()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn menu-syntax-doctor");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap_or_default(),
        String::from_utf8(out.stderr).unwrap_or_default(),
    )
}

#[test]
fn good_fixture_via_stdin_human_exits_zero() {
    let (code, stdout, _) = run_with_stdin(good_capture_v1_json(), &[]);
    assert_eq!(code, 0, "stdout was: {stdout}");
    assert!(
        stdout.contains("OK") || stdout.contains("0 error"),
        "human output should signal OK; got: {stdout}"
    );
}

#[test]
fn good_fixture_via_stdin_json_exits_zero_and_emits_valid_json() {
    let (code, stdout, _) = run_with_stdin(good_capture_v1_json(), &["--json"]);
    assert_eq!(code, 0);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be valid JSON");
    assert!(parsed.get("issues").is_some(), "missing `issues` field");
}

#[test]
fn bad_fixture_via_stdin_human_exits_one_and_lists_error() {
    let (code, stdout, _) = run_with_stdin(bad_unknown_family_json(), &[]);
    assert_eq!(code, 1, "stdout was: {stdout}");
    assert!(
        stdout.contains("error") || stdout.contains("Error"),
        "human output should mention error; got: {stdout}"
    );
}

#[test]
fn bad_fixture_via_stdin_json_exits_one_with_error_severity_in_payload() {
    let (code, stdout, _) = run_with_stdin(bad_unknown_family_json(), &["--json"]);
    assert_eq!(code, 1);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let issues = parsed["issues"].as_array().expect("issues array");
    assert!(
        issues.iter().any(|i| i["severity"] == "error"),
        "expected at least one error-severity issue, got: {parsed}"
    );
}

#[test]
fn good_fixture_via_path_flag_exits_zero() {
    use std::io::Write;
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("good.json");
    {
        let mut f = std::fs::File::create(&path).expect("create");
        f.write_all(good_capture_v1_json().as_bytes())
            .expect("write");
    }
    let out = bin()
        .args(["--path", path.to_str().unwrap()])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn unknown_arg_exits_two_with_stderr_message() {
    let out = bin().args(["--no-such-flag"]).output().expect("run");
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown argument"),
        "stderr should explain; got: {stderr}"
    );
}

#[test]
fn yaml_input_via_stdin_validates_too() {
    // YAML form of the same good fixture — JSON-first parse fails, YAML
    // fallback succeeds. Confirms the dual-parse path.
    let yaml = "family: capture.v1\nslug: todo\ntargets: [todo]\nrequired: [body]\n";
    let (code, stdout, _) = run_with_stdin(yaml, &[]);
    assert_eq!(code, 0, "stdout was: {stdout}");
}

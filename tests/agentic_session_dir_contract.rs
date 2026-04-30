//! Source-level contract tests for agentic session directory canonicalization.

const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");
const DEV_SH: &str = include_str!("../dev.sh");

#[test]
fn session_sh_canonicalizes_script_kit_session_dir() {
    assert!(
        SESSION_SH
            .contains("SESSION_DIR_RAW=\"${SCRIPT_KIT_SESSION_DIR:-/tmp/sk-agentic-sessions}\""),
        "session.sh must preserve the caller/default session-dir input before canonicalization"
    );
    assert!(
        SESSION_SH.contains("canonical_session_dir()"),
        "session.sh must keep canonicalization in a named helper"
    );
    assert!(
        SESSION_SH.contains("mkdir -p \"$dir\""),
        "session.sh canonicalization must create the session dir before cd"
    );
    assert!(
        SESSION_SH.contains("(cd \"$dir\" && pwd -P)"),
        "session.sh must resolve the physical path so /tmp and /private/tmp share one tree"
    );
    assert!(
        SESSION_SH.contains("SESSION_DIR=\"$(canonical_session_dir \"$SESSION_DIR_RAW\")\""),
        "session.sh must store the canonical path in SESSION_DIR before building per-session paths"
    );
}

#[test]
fn dev_sh_exports_canonical_script_kit_session_dir_before_cargo_watch() {
    let raw_pos = DEV_SH
        .find("SESSION_DIR_RAW=\"${SCRIPT_KIT_SESSION_DIR:-/tmp/sk-agentic-sessions}\"")
        .expect("dev.sh must preserve the raw/default session-dir input");
    let mkdir_pos = DEV_SH
        .find("mkdir -p \"$SESSION_DIR_RAW\"")
        .expect("dev.sh must create the session dir before canonicalization");
    let export_pos = DEV_SH
        .find("export SCRIPT_KIT_SESSION_DIR=\"$(cd \"$SESSION_DIR_RAW\" && pwd -P)\"")
        .expect("dev.sh must export a physical SCRIPT_KIT_SESSION_DIR");
    let cargo_watch_pos = DEV_SH
        .find("cargo watch")
        .expect("dev.sh must launch cargo watch");

    assert!(
        raw_pos < mkdir_pos && mkdir_pos < export_pos,
        "dev.sh must derive, create, then canonicalize SCRIPT_KIT_SESSION_DIR"
    );
    assert!(
        export_pos < cargo_watch_pos,
        "dev.sh must export the canonical session dir before cargo-watch launches dev-relaunch"
    );
}

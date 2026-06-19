//! Source-level contracts for `dev.sh` watcher ownership.

const DEV_SH: &str = include_str!("../dev.sh");

#[test]
fn dev_sh_uses_repo_scoped_single_watcher_lock() {
    for needle in [
        "SCRIPT_KIT_DEV_LOCK_DIR",
        "sk-dev-launcher-locks",
        "local lock_root=\"/tmp/sk-dev-launcher-locks\"",
        "dev_sh_acquire_lock()",
        "dev_sh_lock_key()",
        "printf '%s\\n' \"$$\" > \"$SCRIPT_KIT_DEV_LOCK_DIR/pid\"",
        "printf '%s\\n' \"${SCRIPT_KIT_DEV_SESSION_NAME:-dev-watch}\" > \"$SCRIPT_KIT_DEV_LOCK_DIR/session\"",
        "printf '%s\\n' \"$repo_root\" > \"$SCRIPT_KIT_DEV_LOCK_DIR/root\"",
    ] {
        assert!(
            DEV_SH.contains(needle),
            "dev.sh must keep the repo-scoped watcher lock marker: {needle}"
        );
    }
}

#[test]
fn dev_sh_refuses_second_watcher_unless_explicitly_allowed() {
    assert!(
        DEV_SH.contains("another ./dev.sh is already running for this repo"),
        "second dev.sh launch must fail loudly instead of silently adding another cargo-watch"
    );
    assert!(
        DEV_SH.contains("SCRIPT_KIT_DEV_ALLOW_MULTI=1"),
        "dev.sh must expose an explicit opt-in escape hatch for intentional multi-watch runs"
    );
    assert!(
        DEV_SH.contains("[ \"${SCRIPT_KIT_DEV_ALLOW_MULTI:-0}\" != \"1\" ]"),
        "dev.sh must acquire the lock unless SCRIPT_KIT_DEV_ALLOW_MULTI=1"
    );
}

#[test]
fn dev_sh_cleanup_removes_only_its_own_lock() {
    let cleanup_marker =
        "[ \"$(cat \"$SCRIPT_KIT_DEV_LOCK_DIR/pid\" 2>/dev/null || true)\" = \"$$\" ]";
    assert!(
        DEV_SH.contains(cleanup_marker),
        "dev.sh cleanup must compare the lock pid to $$ before removing it"
    );
    assert!(
        DEV_SH.contains("rm -rf \"$SCRIPT_KIT_DEV_LOCK_DIR\""),
        "dev.sh cleanup must remove its own lock directory"
    );
}

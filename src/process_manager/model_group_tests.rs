//! Black-box lifetime challenge for the actual model clients. The fake engine deliberately
//! ignores stdin EOF and owns a second process; it expires by itself even if a regression
//! leaves both orphaned. Only the direct sacrificial Rust host is ever signalled by the test.
use std::fs;
use std::io::{BufRead, BufReader, Read as _, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const OWNER: &str = "SCRIPT_KIT_MODEL_TEST_OWNER";
const FIXTURE: &str = "#!/bin/sh\nset -eu\ntrap '' HUP TERM\n/bin/sleep 6 &\nkid=$!\nprintf '%s %s\\n' \"$$\" \"$kid\" > \"$0.starting\"\n/bin/mv \"$0.starting\" \"$0.ready\"\nwait \"$kid\"\n";

fn wait_for(limit: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + limit;
    loop {
        if condition() {
            return true;
        }
        if Instant::now() >= end {
            return false;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
}
fn gone(pid: i32) -> bool {
    // Signal zero observes only this recorded process, never kills it.
    pid > 1
        && unsafe { libc::kill(pid, 0) } == -1
        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
}

#[test]
fn model_group_preserves_protocol_and_cleans_descendants_on_engine_exit() {
    use super::model_group::ModelGroup;
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().canonicalize().unwrap();
    let mut other = Command::new("/bin/cat");
    other
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    let (mut peer, _peer_group) = ModelGroup::spawn(other).unwrap();
    let mut cmd = Command::new("/bin/sh");
    cmd.env_clear().env("VALUE", "two words").current_dir(&dir)
        .args(["-c", r#"IFS= read -r line; printf '%s\n' "$line" "$VALUE" "$PWD" "$1"; /bin/sleep 6 </dev/null >/dev/null 2>&1 & printf '%s\n' "$!"; exit 17"#, "fixture", "one argument"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    let (mut child, _retained_lease) = ModelGroup::spawn(cmd).unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"request with spaces\n")
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(17));
    let output = String::from_utf8(result.stdout).unwrap();
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(
        &lines[..4],
        &[
            "request with spaces",
            "two words",
            dir.to_str().unwrap(),
            "one argument"
        ]
    );
    let kid = lines[4].parse().unwrap();
    assert!(
        wait_for(Duration::from_secs(2), || gone(kid)),
        "the engine exited but its tool remained under a live Kit"
    );
    // Killing one group must not interfere with another model's protocol in the same Kit.
    peer.stdin
        .take()
        .unwrap()
        .write_all(b"still here\n")
        .unwrap();
    let reply = peer.wait_with_output().unwrap();
    assert!(reply.status.success());
    assert_eq!(reply.stdout, b"still here\n");
}

#[test]
fn model_group_drop_does_not_wait_forever_for_a_wedged_helper() {
    let mut cmd = Command::new("/bin/sh");
    cmd.env_clear().args(["-c", r#"trap '' TERM HUP; /bin/sleep 6 </dev/null >/dev/null 2>&1 & printf '%s\n' "$!"; wait"#]).stdout(Stdio::piped()).stdin(Stdio::piped());
    let (mut engine, mut lease) = super::model_group::ModelGroup::spawn(cmd).unwrap();
    let pid = engine.id() as i32;
    let mut msg = String::new();
    BufReader::new(engine.stdout.take().unwrap())
        .read_line(&mut msg)
        .unwrap();
    let kid = msg.trim().parse().unwrap();
    drop(engine.stdin.take());
    let start = Instant::now();
    lease.finish_after_eof(engine);
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(wait_for(Duration::from_secs(2), || gone(pid) && gone(kid)));
}

pub(crate) fn check_parent_loss<T>(start: impl FnOnce(&Path) -> T) {
    let test = std::thread::current().name().unwrap().to_owned();
    if let Some(home) = std::env::var_os(OWNER) {
        let home = Path::new(&home);
        assert_eq!(Some(home.as_os_str().to_owned()), std::env::var_os("HOME"));
        assert_eq!(home.canonicalize().unwrap(), home);
        assert!(home
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kit-model-test-"));
        let helper = home.join("sidecar");
        assert_eq!(fs::read_to_string(&helper).unwrap(), FIXTURE);
        let _client = start(&helper);
        fs::write(home.join("host-ready"), b"ready").unwrap();
        // Parent owns the only write side and can release us even on a failed assertion.
        let _ = std::io::stdin().read(&mut [0_u8]);
        return;
    }
    let tmp = tempfile::Builder::new()
        .prefix("kit-model-test-")
        .tempdir()
        .unwrap();
    let home = tmp.path().canonicalize().unwrap();
    let helper = home.join("sidecar");
    fs::write(&helper, FIXTURE).unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
    let mut host = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--test-threads=1", "--nocapture"])
        .env_clear()
        .current_dir(&home)
        .env(OWNER, &home)
        .env("HOME", &home)
        .env("SK_PATH", home.join(".scriptkit"))
        .env("TMPDIR", &home)
        .env("PATH", "/usr/bin:/bin")
        .env("SCRIPT_KIT_NONINTERACTIVE", "1")
        .env("SCRIPT_KIT_GHOST_LLM_HELPER", &helper)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let ready = wait_for(Duration::from_secs(8), || {
        home.join("host-ready").exists() && home.join("sidecar.ready").exists()
    });
    let ids: Vec<i32> = fs::read_to_string(home.join("sidecar.ready"))
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    let group = ids
        .first()
        .map(|p| unsafe { libc::getpgid(*p) })
        .unwrap_or(-1);
    let same_group = ids.len() == 2
        && group > 1
        && group != unsafe { libc::getpgrp() }
        && unsafe { libc::getpgid(ids[1]) } == group;
    host.kill().unwrap();
    host.wait().unwrap();
    let promptly =
        ready && same_group && wait_for(Duration::from_secs(2), || ids.iter().copied().all(gone));
    // Red runs don't leave an orphan behind on the build machine, and never fall back to a name/PGID scan.
    let finished = wait_for(Duration::from_secs(7), || ids.iter().copied().all(gone));
    assert!(
        ready && same_group,
        "fixture did not launch an independently grouped engine and child"
    );
    assert!(finished, "bounded fixture processes did not exit");
    assert!(
        promptly,
        "model process or its tool survived the loss of its Kit parent"
    );
}

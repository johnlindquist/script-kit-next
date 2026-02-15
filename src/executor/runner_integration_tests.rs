use super::runner::spawn_script;
use crate::protocol::{serialize_message, Message};
use std::io::Write;

#[cfg(unix)]
#[test]
fn test_spawn_script_returns_running_session_and_pid() {
    let session =
        spawn_script("sh", &["-c", "cat"], "[test:runner_spawn]").expect("spawn should succeed");
    let mut split = session.split();

    assert!(
        split.process_handle.pid > 0,
        "spawned process should have a valid pid"
    );
    assert!(
        split.process_handle.is_alive(),
        "session should report running immediately"
    );

    split.process_handle.kill();
    let _ = split.child.wait();
}

#[cfg(unix)]
#[test]
fn test_script_session_kill_is_idempotent() {
    let session = spawn_script("sh", &["-c", "cat"], "[test:runner_kill_idempotent]")
        .expect("spawn should succeed");
    let mut split = session.split();

    split.process_handle.kill();
    split.process_handle.kill();

    let _ = split.child.wait();
}

#[cfg(unix)]
#[test]
fn test_wait_returns_exit_code_after_process_exit() {
    let session = spawn_script("sh", &["-c", "exit 7"], "[test:runner_wait_exit]")
        .expect("spawn should succeed");
    let mut split = session.split();

    let status = split
        .child
        .wait()
        .expect("wait should return an exit status");
    let code = status.code().unwrap_or(-1);
    assert_eq!(code, 7, "wait should return the script exit status");
}

#[cfg(unix)]
#[test]
fn test_send_receive_message_round_trip_when_channel_open() {
    let session = spawn_script(
        "sh",
        &["-c", "IFS= read -r line; printf '%s\\n' \"$line\""],
        "[test:runner_round_trip]",
    )
    .expect("spawn should succeed");
    let mut split = session.split();

    let message = serialize_message(&Message::beep()).expect("serialize should succeed");
    writeln!(split.stdin, "{}", message).expect("send_message should succeed");
    split.stdin.flush().expect("flush should succeed");

    let echoed = split
        .stdout_reader
        .next_message()
        .expect("receive_message should succeed")
        .expect("script should echo one message");

    assert!(
        matches!(echoed, Message::Beep {}),
        "expected echoed beep message"
    );

    let status = split
        .child
        .wait()
        .expect("wait should succeed after round-trip");
    let code = status.code().unwrap_or(-1);
    assert_eq!(code, 0, "echo script should exit successfully");
}

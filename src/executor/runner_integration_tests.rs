use super::runner::spawn_script;
use crate::protocol::Message;

#[cfg(unix)]
#[test]
fn test_spawn_script_returns_running_session_and_pid() {
    let mut session =
        spawn_script("sh", &["-c", "cat"], "[test:runner_spawn]").expect("spawn should succeed");

    assert!(session.pid() > 0, "spawned process should have a valid pid");
    assert!(
        session.is_running(),
        "session should report running immediately"
    );

    session.kill().expect("kill should succeed");
    let _ = session.wait();
}

#[cfg(unix)]
#[test]
fn test_script_session_kill_is_idempotent() {
    let mut session = spawn_script("sh", &["-c", "cat"], "[test:runner_kill_idempotent]")
        .expect("spawn should succeed");

    session.kill().expect("first kill should succeed");
    session.kill().expect("second kill should also succeed");

    let _ = session.wait();
}

#[cfg(unix)]
#[test]
fn test_wait_returns_exit_code_after_process_exit() {
    let mut session = spawn_script("sh", &["-c", "exit 7"], "[test:runner_wait_exit]")
        .expect("spawn should succeed");

    let code = session.wait().expect("wait should return an exit code");
    assert_eq!(code, 7, "wait should return the script exit status");
}

#[cfg(unix)]
#[test]
fn test_send_receive_message_round_trip_when_channel_open() {
    let mut session = spawn_script(
        "sh",
        &["-c", "IFS= read -r line; printf '%s\\n' \"$line\""],
        "[test:runner_round_trip]",
    )
    .expect("spawn should succeed");

    session
        .send_message(&Message::beep())
        .expect("send_message should succeed");

    let echoed = session
        .receive_message()
        .expect("receive_message should succeed")
        .expect("script should echo one message");

    assert!(
        matches!(echoed, Message::Beep {}),
        "expected echoed beep message"
    );

    let code = session
        .wait()
        .expect("wait should succeed after round-trip");
    assert_eq!(code, 0, "echo script should exit successfully");
}

//! Child process management: spawn the app binary, hold stdin open, clean shutdown.

#![allow(dead_code)]

use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use super::environment::TestEnvironment;
use super::log_observer::LogObserver;
use super::response_reader::{ResponseReader, StateSnapshot};

/// Manages the lifecycle of a `script-kit-gpui` child process.
pub struct AppProcess {
    child: Child,
    /// Stdin pipe — kept open for sending commands. `None` after close.
    stdin: Option<std::process::ChildStdin>,
    /// Stderr log observer running in background thread.
    pub logs: LogObserver,
    /// Stdout response reader for structured JSONL query responses.
    pub responses: ResponseReader,
    /// The test environment (temp dirs, env vars). Kept alive for the process lifetime.
    pub env: TestEnvironment,
    /// Monotonic counter for generating unique request IDs.
    request_counter: std::sync::atomic::AtomicU64,
}

impl AppProcess {
    /// Find the built binary. Looks for `target/debug/script-kit-gpui[.exe]`.
    fn find_binary() -> anyhow::Result<PathBuf> {
        // Walk up from the test binary's location to find the workspace root,
        // or use a well-known relative path from the repo root.
        let candidates = [
            // Running from the repo root (most common)
            PathBuf::from("target/debug/script-kit-gpui.exe"),
            PathBuf::from("target/debug/script-kit-gpui"),
            // Absolute path from the workspace root
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/script-kit-gpui.exe"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/script-kit-gpui"),
        ];

        for path in &candidates {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        anyhow::bail!(
            "Could not find script-kit-gpui binary. \
             Run `cargo build --bin script-kit-gpui` first.\n\
             Searched: {:?}",
            candidates
        );
    }

    /// Spawn the app with the given environment.
    ///
    /// Does NOT wait for readiness — call `wait_for_ready()` after.
    pub fn spawn(env: TestEnvironment) -> anyhow::Result<Self> {
        let binary = Self::find_binary()?;
        let env_map = env.env_map();

        eprintln!(
            "[harness] Spawning: {} (SK_PATH={})",
            binary.display(),
            env.sk_path.display()
        );

        let mut child = Command::new(&binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .envs(&env_map)
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn {}: {}", binary.display(), e))?;

        let stdin = child.stdin.take();
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to capture stderr"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to capture stdout"))?;

        let logs = LogObserver::new(stderr);
        let responses = ResponseReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            logs,
            responses,
            env,
            request_counter: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Wait until the app reports it's ready (or timeout).
    ///
    /// Watches stderr for the "Application ready" log line.
    pub fn wait_for_ready(&self, timeout: Duration) -> anyhow::Result<()> {
        eprintln!(
            "[harness] Waiting for app readiness (timeout: {:?})...",
            timeout
        );
        self.logs
            .wait_for_log("Application ready", timeout)
            .map_err(|e| anyhow::anyhow!("app did not become ready: {}", e))?;
        eprintln!("[harness] App is ready.");
        Ok(())
    }

    /// Send a raw JSONL command string to the app's stdin.
    ///
    /// Appends a newline if not present.
    pub fn send_raw(&mut self, json: &str) -> anyhow::Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("stdin already closed"))?;

        let line = if json.ends_with('\n') {
            json.to_string()
        } else {
            format!("{}\n", json)
        };

        stdin
            .write_all(line.as_bytes())
            .map_err(|e| anyhow::anyhow!("failed to write to stdin: {}", e))?;
        stdin
            .flush()
            .map_err(|e| anyhow::anyhow!("failed to flush stdin: {}", e))?;

        eprintln!("[harness] Sent: {}", json.trim());
        Ok(())
    }

    /// Send a `{"type":"show"}` command.
    pub fn send_show(&mut self) -> anyhow::Result<()> {
        self.send_raw(r#"{"type":"show"}"#)
    }

    /// Send a `{"type":"hide"}` command.
    pub fn send_hide(&mut self) -> anyhow::Result<()> {
        self.send_raw(r#"{"type":"hide"}"#)
    }

    /// Send a `{"type":"setFilter","text":"..."}` command.
    pub fn send_filter(&mut self, text: &str) -> anyhow::Result<()> {
        let cmd = serde_json::json!({
            "type": "setFilter",
            "text": text,
        });
        self.send_raw(&cmd.to_string())
    }

    /// Send a `{"type":"simulateKey","key":"...","modifiers":[...]}` command.
    pub fn send_key(&mut self, key: &str, modifiers: &[&str]) -> anyhow::Result<()> {
        let cmd = serde_json::json!({
            "type": "simulateKey",
            "key": key,
            "modifiers": modifiers,
        });
        self.send_raw(&cmd.to_string())
    }

    /// Send a `{"type":"run","path":"..."}` command.
    pub fn send_run(&mut self, path: &str) -> anyhow::Result<()> {
        let cmd = serde_json::json!({
            "type": "run",
            "path": path,
        });
        self.send_raw(&cmd.to_string())
    }

    /// Generate a unique request ID for query commands.
    pub fn next_request_id(&self) -> String {
        let n = self
            .request_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("test-{}", n)
    }

    /// Send `{"type":"getState"}` and wait for the response.
    ///
    /// Returns a `StateSnapshot` with all observable state fields.
    pub fn get_state(&mut self, timeout: Duration) -> anyhow::Result<StateSnapshot> {
        let rid = self.next_request_id();
        let cmd = serde_json::json!({
            "type": "getState",
            "requestId": rid,
        });
        self.send_raw(&cmd.to_string())?;

        let resp = self.responses.wait_for_response(&rid, timeout)?;
        StateSnapshot::from_response(resp)
            .ok_or_else(|| anyhow::anyhow!("unexpected response type for getState"))
    }

    /// Send `{"type":"getChatActionsState"}` and wait for the response.
    ///
    /// Returns a `ChatActionsSnapshot` with chat and actions dialog state.
    pub fn get_chat_actions_state(
        &mut self,
        timeout: Duration,
    ) -> anyhow::Result<super::response_reader::ChatActionsSnapshot> {
        let rid = self.next_request_id();
        let cmd = serde_json::json!({
            "type": "getChatActionsState",
            "requestId": rid,
        });
        self.send_raw(&cmd.to_string())?;

        let resp = self.responses.wait_for_response(&rid, timeout)?;
        super::response_reader::ChatActionsSnapshot::from_response(resp)
            .ok_or_else(|| anyhow::anyhow!("unexpected response type for getChatActionsState"))
    }

    /// Send a `{"type":"simulateAiKey","key":"...","modifiers":[...]}` command.
    ///
    /// Routes the key directly to the AI window's `handle_simulated_key`.
    pub fn send_ai_key(&mut self, key: &str, modifiers: &[&str]) -> anyhow::Result<()> {
        let rid = self.next_request_id();
        let cmd = serde_json::json!({
            "type": "simulateAiKey",
            "key": key,
            "modifiers": modifiers,
            "requestId": rid,
        });
        self.send_raw(&cmd.to_string())
    }

    /// Send `{"type":"openAiWithMockData"}` to open the separate AI Chat window.
    pub fn send_open_ai_with_mock_data(&mut self) -> anyhow::Result<()> {
        self.send_raw(r#"{"type":"openAiWithMockData"}"#)
    }

    /// Send `{"type":"showAiCommandBar"}` to open the AI window's command bar.
    pub fn send_show_ai_command_bar(&mut self) -> anyhow::Result<()> {
        self.send_raw(r#"{"type":"showAiCommandBar"}"#)
    }

    /// Send `{"type":"getAiCommandBarState"}` and wait for the response.
    ///
    /// Returns an `AiCommandBarSnapshot` with command-bar selection state.
    pub fn get_ai_command_bar_state(
        &mut self,
        timeout: Duration,
    ) -> anyhow::Result<super::response_reader::AiCommandBarSnapshot> {
        let rid = self.next_request_id();
        let cmd = serde_json::json!({
            "type": "getAiCommandBarState",
            "requestId": rid,
        });
        self.send_raw(&cmd.to_string())?;

        let resp = self.responses.wait_for_response(&rid, timeout)?;
        super::response_reader::AiCommandBarSnapshot::from_response(resp)
            .ok_or_else(|| anyhow::anyhow!("unexpected response type for getAiCommandBarState"))
    }

    /// Close the stdin pipe (signals the app that no more commands are coming).
    pub fn close_stdin(&mut self) {
        self.stdin.take();
        eprintln!("[harness] Stdin closed.");
    }

    /// Wait for the process to exit, with a timeout.
    ///
    /// Returns the exit status if the process exited, or kills it on timeout.
    pub fn wait_for_exit(&mut self, timeout: Duration) -> anyhow::Result<std::process::ExitStatus> {
        let start = std::time::Instant::now();

        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    eprintln!("[harness] Process exited: {:?}", status);
                    return Ok(status);
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        eprintln!("[harness] Timeout waiting for exit, killing process.");
                        let _ = self.child.kill();
                        let status = self.child.wait()?;
                        anyhow::bail!(
                            "process did not exit within {:?} (killed, status: {:?})",
                            timeout,
                            status
                        );
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => anyhow::bail!("error checking process status: {}", e),
            }
        }
    }

    /// Kill the process immediately.
    pub fn kill(&mut self) -> anyhow::Result<()> {
        eprintln!("[harness] Killing process.");
        self.stdin.take(); // Close stdin first
        self.child
            .kill()
            .map_err(|e| anyhow::anyhow!("kill failed: {}", e))
    }
}

impl Drop for AppProcess {
    fn drop(&mut self) {
        // Best-effort cleanup
        self.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

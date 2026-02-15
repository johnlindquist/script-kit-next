//! Script execution and process spawning
//!
//! This module handles the core script execution logic, including:
//! - Finding executables (bun, node, etc.)
//! - Spawning interactive script processes
//! - SDK path management
//! - File type detection

use crate::logging;
use crate::process_manager::PROCESS_MANAGER;
use crate::protocol::{serialize_message, JsonlReader, Message};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;
use tracing::{debug, error, info, instrument};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

// Unix-specific process control using libc for correctness and performance
#[cfg(unix)]
mod unix_process {
    use libc::{c_int, pid_t, ESRCH};

    /// Send a signal to a process group (negative PID targets the group)
    ///
    /// Returns Ok(()) if signal was sent successfully.
    /// Returns Err with errno description on failure.
    pub fn kill_process_group(pgid: u32, signal: c_int) -> Result<(), &'static str> {
        // Safety: kill() is a simple syscall with no memory safety concerns
        // Negative PID targets the process group
        let rc = unsafe { libc::kill(-(pgid as pid_t), signal) };
        if rc == 0 {
            Ok(())
        } else {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            match errno {
                libc::ESRCH => Err("No such process group"),
                libc::EPERM => Err("Permission denied"),
                libc::EINVAL => Err("Invalid signal"),
                _ => Err("Unknown error"),
            }
        }
    }

    /// Check if a process group is still alive
    ///
    /// Uses signal 0 which doesn't actually send a signal but checks if the
    /// process group exists. Returns true if any process in the group is alive.
    ///
    /// Note: EPERM (permission denied) also means the process exists but we
    /// don't have permission to signal it - we still count this as "alive".
    pub fn process_group_alive(pgid: u32) -> bool {
        // Safety: kill() with signal 0 is safe - it only checks existence
        let rc = unsafe { libc::kill(-(pgid as pid_t), 0) };
        if rc == 0 {
            true
        } else {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            // EPERM means process exists but we can't signal it - still alive
            // ESRCH means no such process - dead
            errno != ESRCH
        }
    }

    /// SIGTERM signal number
    pub const SIGTERM: c_int = libc::SIGTERM;
    /// SIGKILL signal number
    pub const SIGKILL: c_int = libc::SIGKILL;
}

/// Embedded SDK content (included at compile time)
const EMBEDDED_SDK: &str = include_str!("../../scripts/kit-sdk.ts");

const SAFE_SCRIPT_ENV_VARS: &[&str] = &[
    "PATH",
    "HOME",
    "TMPDIR",
    "USER",
    "LANG",
    "TERM",
    "SHELL",
    "XDG_RUNTIME_DIR",
];

/// OnceLock for single-flight SDK extraction
/// Ensures SDK is extracted exactly once, preventing race conditions
/// when multiple scripts start simultaneously
static SDK_EXTRACTED: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

/// Find an executable, checking common locations that GUI apps might miss
pub fn find_executable(name: &str) -> Option<PathBuf> {
    logging::log("EXEC", &format!("Looking for executable: {}", name));

    // Common paths where executables might be installed
    let common_paths = [
        // User-specific paths
        dirs::home_dir().map(|h| h.join(".bun/bin")),
        dirs::home_dir().map(|h| h.join("Library/pnpm")), // pnpm on macOS
        dirs::home_dir().map(|h| h.join(".nvm/current/bin")),
        dirs::home_dir().map(|h| h.join(".volta/bin")),
        dirs::home_dir().map(|h| h.join(".local/bin")),
        dirs::home_dir().map(|h| h.join("bin")),
        // Homebrew paths
        Some(PathBuf::from("/opt/homebrew/bin")),
        Some(PathBuf::from("/usr/local/bin")),
        // System paths
        Some(PathBuf::from("/usr/bin")),
        Some(PathBuf::from("/bin")),
    ];

    for path in common_paths.iter().flatten() {
        let exe_path = path.join(name);
        logging::log("EXEC", &format!("  Checking: {}", exe_path.display()));
        if exe_path.exists() {
            logging::log("EXEC", &format!("  FOUND: {}", exe_path.display()));
            return Some(exe_path);
        }
    }

    logging::log("EXEC", "  NOT FOUND in common paths, will try PATH");
    None
}

/// Get the SDK path - SDK extraction is now handled by setup::ensure_kit_setup() at startup
/// This function just returns the expected path since setup has already done the work
///
/// ## Race Condition Prevention
/// Uses OnceLock to ensure SDK is extracted exactly once, even when multiple scripts
/// start simultaneously. The fallback extraction uses atomic write (temp + rename)
/// to prevent partial reads.
fn ensure_sdk_extracted() -> Option<PathBuf> {
    SDK_EXTRACTED
        .get_or_init(|| {
            // Target path: ~/.scriptkit/sdk/kit-sdk.ts
            // This is extracted by setup::ensure_kit_setup() which runs at app startup
            let home = dirs::home_dir()?;
            let sdk_path = home.join(".scriptkit/sdk/kit-sdk.ts");

            if sdk_path.exists() {
                return Some(sdk_path);
            }

            // Fallback: write embedded SDK if somehow missing
            // This shouldn't happen in normal operation since setup runs first
            logging::log(
                "EXEC",
                "SDK not found at expected path, extracting embedded SDK",
            );

            let kit_sdk = home.join(".scriptkit/sdk");
            if !kit_sdk.exists() {
                std::fs::create_dir_all(&kit_sdk).ok()?;
            }

            // Atomic write: temp file then rename to prevent partial reads
            let temp_path = sdk_path.with_extension("tmp");
            std::fs::write(&temp_path, EMBEDDED_SDK).ok()?;
            std::fs::rename(&temp_path, &sdk_path).ok()?;

            logging::log(
                "EXEC",
                &format!("Extracted fallback SDK to {}", sdk_path.display()),
            );
            Some(sdk_path)
        })
        .clone()
}

/// Find the SDK path, checking standard locations
pub fn find_sdk_path() -> Option<PathBuf> {
    logging::log("EXEC", "Looking for SDK...");

    // 1. Check ~/.scriptkit/sdk/kit-sdk.ts (primary location)
    if let Some(home) = dirs::home_dir() {
        let kit_sdk = home.join(".scriptkit/sdk/kit-sdk.ts");
        logging::log(
            "EXEC",
            &format!("  Checking kit sdk: {}", kit_sdk.display()),
        );
        if kit_sdk.exists() {
            logging::log("EXEC", &format!("  FOUND SDK (kit): {}", kit_sdk.display()));
            return Some(kit_sdk);
        }
    }

    // 2. Extract embedded SDK to ~/.scriptkit/sdk/kit-sdk.ts (production)
    logging::log("EXEC", "  Trying to extract embedded SDK...");
    if let Some(sdk_path) = ensure_sdk_extracted() {
        logging::log(
            "EXEC",
            &format!("  FOUND SDK (embedded): {}", sdk_path.display()),
        );
        return Some(sdk_path);
    }

    // 3. Check relative to executable (for app bundle)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let sdk_path = exe_dir.join("kit-sdk.ts");
            logging::log(
                "EXEC",
                &format!("  Checking exe dir: {}", sdk_path.display()),
            );
            if sdk_path.exists() {
                logging::log(
                    "EXEC",
                    &format!("  FOUND SDK (exe dir): {}", sdk_path.display()),
                );
                return Some(sdk_path);
            }
        }
    }

    // 4. Development fallback - project scripts directory
    let dev_sdk = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/kit-sdk.ts");
    logging::log(
        "EXEC",
        &format!("  Checking dev path: {}", dev_sdk.display()),
    );
    if dev_sdk.exists() {
        logging::log("EXEC", &format!("  FOUND SDK (dev): {}", dev_sdk.display()));
        return Some(dev_sdk);
    }

    logging::log("EXEC", "  SDK NOT FOUND anywhere!");
    None
}

/// Wrapper that tracks process ID for cleanup
/// This stores the PID at spawn time so we can kill the process group even after
/// the Child is moved or consumed.
///
/// CRITICAL: The Drop impl kills the process group, so this MUST be kept alive
/// until the script is done executing!
#[derive(Debug)]
pub struct ProcessHandle {
    /// Process ID (used as PGID since we spawn with process_group(0))
    pub(crate) pid: u32,
    /// Path to the script being executed (for process tracking)
    /// Used during registration with PROCESS_MANAGER in new()
    #[allow(dead_code)]
    script_path: String,
    /// Whether the process has been explicitly killed
    pub(crate) killed: bool,
}

impl ProcessHandle {
    pub fn new(pid: u32, script_path: String) -> Self {
        logging::log(
            "EXEC",
            &format!(
                "ProcessHandle created for PID {} (script: {})",
                pid, script_path
            ),
        );

        // Register with global process manager for tracking
        PROCESS_MANAGER.register_process(pid, &script_path);

        Self {
            pid,
            script_path,
            killed: false,
        }
    }

    /// Kill the process group with graceful escalation (Unix) or just the process (other platforms)
    ///
    /// ## Escalation Protocol
    /// 1. Send SIGTERM to process group (graceful termination request)
    /// 2. Wait up to TERM_GRACE_MS for process group to exit
    /// 3. If still alive, send SIGKILL (forceful termination)
    ///
    /// ## Critical Fix
    /// Uses libc::kill() to check process GROUP liveness (not just leader PID).
    /// This prevents orphan child processes when the leader exits but children remain.
    ///
    /// This gives scripts a chance to clean up before being forcefully killed.
    pub fn kill(&mut self) {
        /// Grace period after SIGTERM before escalating to SIGKILL (milliseconds)
        const TERM_GRACE_MS: u64 = 250;
        /// How often to check if process has exited during grace period
        const POLL_INTERVAL_MS: u64 = 50;

        if self.killed {
            logging::log(
                "EXEC",
                &format!("Process {} already killed, skipping", self.pid),
            );
            return;
        }
        self.killed = true;

        #[cfg(unix)]
        {
            use unix_process::{kill_process_group, process_group_alive, SIGKILL, SIGTERM};

            // Since we spawned with process_group(0), the PGID equals the PID
            let pgid = self.pid;

            // Step 1: Send SIGTERM for graceful shutdown
            logging::log(
                "EXEC",
                &format!(
                    "Sending SIGTERM to process group PGID {} (graceful shutdown)",
                    pgid
                ),
            );

            match kill_process_group(pgid, SIGTERM) {
                Ok(()) => {
                    logging::log("EXEC", &format!("SIGTERM sent to PGID {}", pgid));
                }
                Err("No such process group") => {
                    logging::log("EXEC", &format!("Process group {} already exited", pgid));
                    return;
                }
                Err(e) => {
                    logging::log(
                        "EXEC",
                        &format!("Failed to send SIGTERM to PGID {}: {}", pgid, e),
                    );
                    // Continue to try SIGKILL anyway
                }
            }

            // Step 2: Wait for grace period, polling process GROUP (not just leader)
            let start = std::time::Instant::now();
            let grace_duration = std::time::Duration::from_millis(TERM_GRACE_MS);
            let poll_interval = std::time::Duration::from_millis(POLL_INTERVAL_MS);

            while start.elapsed() < grace_duration {
                // CRITICAL: Check if process GROUP is alive, not just the leader PID
                // This prevents orphan processes when the leader exits but children remain
                if !process_group_alive(pgid) {
                    logging::log(
                        "EXEC",
                        &format!("Process group {} terminated gracefully after SIGTERM", pgid),
                    );
                    return;
                }
                std::thread::sleep(poll_interval);
            }

            // Step 3: Process group didn't exit in time, escalate to SIGKILL
            logging::log(
                "EXEC",
                &format!(
                    "Process group {} did not exit after {}ms, escalating to SIGKILL",
                    pgid, TERM_GRACE_MS
                ),
            );

            match kill_process_group(pgid, SIGKILL) {
                Ok(()) => {
                    logging::log(
                        "EXEC",
                        &format!("Successfully killed process group {} with SIGKILL", pgid),
                    );
                }
                Err("No such process group") => {
                    logging::log(
                        "EXEC",
                        &format!("Process group {} exited just before SIGKILL", pgid),
                    );
                }
                Err(e) => {
                    logging::log("EXEC", &format!("SIGKILL failed for PGID {}: {}", pgid, e));
                }
            }
        }

        #[cfg(not(unix))]
        {
            logging::log(
                "EXEC",
                &format!("Non-Unix platform: process {} marked as killed", self.pid),
            );
            // On non-Unix platforms, we rely on the Child::kill() method being called separately
        }
    }

    /// Check if process group is still running (Unix only)
    ///
    /// Checks the entire process group, not just the leader PID.
    /// This ensures we properly detect when all child processes have exited.
    #[cfg(unix)]
    #[allow(dead_code)]
    pub fn is_alive(&self) -> bool {
        unix_process::process_group_alive(self.pid)
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        logging::log(
            "EXEC",
            &format!("ProcessHandle dropping for PID {}", self.pid),
        );

        // Unregister from global process manager BEFORE killing
        PROCESS_MANAGER.unregister_process(self.pid);

        self.kill();
    }
}

/// Session for bidirectional communication with a running script
pub struct ScriptSession {
    pub stdin: ChildStdin,
    pub(crate) stdout_reader: JsonlReader<BufReader<ChildStdout>>,
    /// Captured stderr for error reporting
    pub stderr: Option<ChildStderr>,
    pub(crate) child: Child,
    /// Process handle for cleanup - kills process group on drop
    pub(crate) process_handle: ProcessHandle,
}

/// Split session components for separate read/write threads
pub struct SplitSession {
    pub stdin: ChildStdin,
    pub stdout_reader: JsonlReader<BufReader<ChildStdout>>,
    /// Captured stderr for error reporting
    pub stderr: Option<ChildStderr>,
    pub child: Child,
    /// Process handle for cleanup - kills process group on drop
    /// IMPORTANT: This MUST be kept alive until the script completes!
    /// Dropping it will kill the process group via the Drop impl.
    pub process_handle: ProcessHandle,
}

impl ScriptSession {
    /// Split the session into separate read/write components
    /// This allows using separate threads for reading and writing
    pub fn split(self) -> SplitSession {
        logging::log(
            "EXEC",
            &format!(
                "Splitting ScriptSession for PID {}",
                self.process_handle.pid
            ),
        );
        SplitSession {
            stdin: self.stdin,
            stdout_reader: self.stdout_reader,
            stderr: self.stderr,
            child: self.child,
            process_handle: self.process_handle,
        }
    }
}

#[allow(dead_code)]
impl SplitSession {
    /// Check if the child process is still running
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    /// Kill the child process and its process group
    pub fn kill(&mut self) -> Result<(), String> {
        logging::log(
            "EXEC",
            &format!("SplitSession::kill() for PID {}", self.process_handle.pid),
        );
        self.process_handle.kill();
        // Also try the standard kill for good measure
        let _ = self.child.kill();
        Ok(())
    }

    /// Wait for the child process to terminate and get its exit code
    pub fn wait(&mut self) -> Result<i32, String> {
        let status = self
            .child
            .wait()
            .map_err(|e| format!("Failed to wait for script process: {}", e))?;
        let code = status.code().unwrap_or(-1);
        logging::log("EXEC", &format!("Script exited with code: {}", code));
        Ok(code)
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.process_handle.pid
    }
}

#[allow(dead_code)]
impl ScriptSession {
    /// Send a message to the running script
    pub fn send_message(&mut self, msg: &Message) -> Result<(), String> {
        let json =
            serialize_message(msg).map_err(|e| format!("Failed to serialize message: {}", e))?;
        // Use truncated logging - avoids full payload (screenshots, clipboard, etc.)
        logging::log_protocol_send(0, &json);
        writeln!(self.stdin, "{}", json)
            .map_err(|e| format!("Failed to write to script stdin: {}", e))?;
        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        Ok(())
    }

    /// Receive a message from the running script (blocking)
    pub fn receive_message(&mut self) -> Result<Option<Message>, String> {
        let result = self
            .stdout_reader
            .next_message()
            .map_err(|e| format!("Failed to read from script stdout: {}", e));
        if let Ok(Some(ref msg)) = result {
            // Use truncated logging - message Debug impl may contain large payloads
            // Extract type name from Debug output: "Variant { ... }" -> "Variant"
            let debug_str = format!("{:?}", msg);
            let msg_type = debug_str.split_whitespace().next().unwrap_or("Unknown");
            logging::log_protocol_recv(msg_type, std::mem::size_of_val(msg));
        }
        result
    }

    /// Check if the child process is still running
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    /// Wait for the child process to terminate and get its exit code
    pub fn wait(&mut self) -> Result<i32, String> {
        let status = self
            .child
            .wait()
            .map_err(|e| format!("Failed to wait for script process: {}", e))?;
        let code = status.code().unwrap_or(-1);
        logging::log("EXEC", &format!("Script exited with code: {}", code));
        Ok(code)
    }

    /// Kill the child process and its process group
    pub fn kill(&mut self) -> Result<(), String> {
        logging::log(
            "EXEC",
            &format!("ScriptSession::kill() for PID {}", self.process_handle.pid),
        );
        self.process_handle.kill();
        // Also try the standard kill for good measure
        let _ = self.child.kill();
        Ok(())
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.process_handle.pid
    }
}

#[derive(Debug)]
struct RuntimeAttempt {
    name: &'static str,
    label: String,
    cmd: String,
    args: Vec<String>,
}

fn run_fallback_chain<T>(
    attempts: &[RuntimeAttempt],
    start: &Instant,
    mut runner: impl FnMut(&str, &[&str], &str) -> Result<T, String>,
    script_path: &str,
) -> Option<T> {
    for attempt in attempts {
        if attempt.args.is_empty() {
            logging::log("EXEC", &format!("Trying: {}", attempt.cmd));
        } else {
            logging::log(
                "EXEC",
                &format!("Trying: {} {}", attempt.cmd, attempt.args.join(" ")),
            );
        }
        logging::bench_log(attempt.name);

        let args: Vec<&str> = attempt.args.iter().map(String::as_str).collect();
        match runner(&attempt.cmd, &args, script_path) {
            Ok(result) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                info!(
                    runtime = attempt.name,
                    duration_ms = duration_ms,
                    "Runtime fallback succeeded"
                );
                logging::bench_log(&format!("{} succeeded in {}ms", attempt.name, duration_ms));
                logging::log("EXEC", &format!("SUCCESS: {}", attempt.label));
                return Some(result);
            }
            Err(e) => {
                debug!(error = %e, runtime = attempt.name, "Spawn attempt failed");
                logging::log("EXEC", &format!("FAILED: {}: {}", attempt.label, e));
            }
        }
    }
    None
}

/// Execute a script with bidirectional JSONL communication
#[instrument(skip_all, fields(script_path = %path.display()))]
pub fn execute_script_interactive(path: &Path) -> Result<ScriptSession, String> {
    let start = Instant::now();
    logging::bench_log("execute_script_interactive_start");
    debug!(path = %path.display(), "Starting interactive script execution");
    logging::log(
        "EXEC",
        &format!("execute_script_interactive: {}", path.display()),
    );

    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid path encoding".to_string())?;

    // Find SDK for preloading
    let sdk_path = find_sdk_path();

    let bun_path = "bun".to_string();
    let node_path = "node".to_string();

    let mut attempts = Vec::new();
    if is_typescript(path) {
        if let Some(ref sdk) = sdk_path {
            let sdk_str = sdk.to_string_lossy().into_owned();
            attempts.push(RuntimeAttempt {
                name: "bun",
                label: "bun with preload".to_string(),
                cmd: bun_path.clone(),
                args: vec![
                    "run".to_string(),
                    "--preload".to_string(),
                    sdk_str,
                    path_str.to_string(),
                ],
            });
        }

        attempts.push(RuntimeAttempt {
            name: "bun",
            label: "bun without preload".to_string(),
            cmd: bun_path,
            args: vec!["run".to_string(), path_str.to_string()],
        });
    }

    if is_javascript(path) {
        attempts.push(RuntimeAttempt {
            name: "node",
            label: "node".to_string(),
            cmd: node_path,
            args: vec![path_str.to_string()],
        });
    }

    if let Some(session) = run_fallback_chain(&attempts, &start, spawn_script, path_str) {
        return Ok(session);
    }

    let err = format!(
        "Failed to execute script '{}' interactively. Make sure bun or node is installed.",
        path.display()
    );
    error!(
        duration_ms = start.elapsed().as_millis() as u64,
        path = %path.display(),
        "All script execution methods failed"
    );
    logging::log("EXEC", &format!("ALL METHODS FAILED: {}", err));
    Err(err)
}

/// Spawn a script as an interactive process with piped stdin/stdout
#[instrument(skip_all, fields(cmd = %cmd))]
pub fn spawn_script(cmd: &str, args: &[&str], script_path: &str) -> Result<ScriptSession, String> {
    // Try to find the executable in common locations
    let executable = find_executable(cmd)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| cmd.to_string());

    debug!(executable = %executable, args = ?args, "Spawning script process");
    logging::log("EXEC", &format!("spawn_script: {} {:?}", executable, args));

    let mut command = Command::new(&executable);
    command.env_clear();
    for key in SAFE_SCRIPT_ENV_VARS {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    for (key, value) in std::env::vars() {
        if key.starts_with("SCRIPT_KIT") {
            command.env(key, value);
        }
    }
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()); // Capture stderr for error handling

    // On Unix, spawn in a new process group so we can kill all children
    // process_group(0) means the child's PID becomes the PGID
    #[cfg(unix)]
    {
        command.process_group(0);
        logging::log("EXEC", "Using process group for child process");
    }

    let mut child = command.spawn().map_err(|e| {
        error!(error = %e, executable = %executable, "Process spawn failed");
        let err = format!("Failed to spawn '{}': {}", executable, e);
        logging::log("EXEC", &format!("SPAWN ERROR: {}", err));
        err
    })?;

    let pid = child.id();
    info!(pid = pid, pgid = pid, executable = %executable, "Process spawned");
    logging::log(
        "EXEC",
        &format!("Process spawned with PID: {} (PGID: {})", pid, pid),
    );

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to open script stdin".to_string())?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to open script stdout".to_string())?;

    // Capture stderr for error reporting
    let stderr = child.stderr.take();
    logging::log("EXEC", &format!("Stderr captured: {}", stderr.is_some()));

    let process_handle = ProcessHandle::new(pid, script_path.to_string());
    logging::log("EXEC", "ScriptSession created successfully");

    Ok(ScriptSession {
        stdin,
        stdout_reader: JsonlReader::new(BufReader::new(stdout)),
        stderr,
        child,
        process_handle,
    })
}

/// Check if the path points to a TypeScript file
pub fn is_typescript(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "ts")
        .unwrap_or(false)
}

/// Check if the path points to a JavaScript file
pub fn is_javascript(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "js")
        .unwrap_or(false)
}

#[cfg(test)]
mod runtime_fallback_tests {
    use super::{run_fallback_chain, RuntimeAttempt};
    use std::cell::RefCell;
    use std::time::Instant;

    #[test]
    fn test_run_fallback_chain_returns_first_success_when_previous_attempt_fails() {
        let start = Instant::now();
        let calls = RefCell::new(Vec::new());
        let attempts = vec![
            RuntimeAttempt {
                name: "kit",
                label: "kit".to_string(),
                cmd: "kit".to_string(),
                args: vec!["run".to_string(), "test.ts".to_string()],
            },
            RuntimeAttempt {
                name: "bun",
                label: "bun without preload".to_string(),
                cmd: "bun".to_string(),
                args: vec!["run".to_string(), "test.ts".to_string()],
            },
            RuntimeAttempt {
                name: "node",
                label: "node".to_string(),
                cmd: "node".to_string(),
                args: vec!["test.js".to_string()],
            },
        ];

        let result = run_fallback_chain(
            &attempts,
            &start,
            |cmd, _, _| {
                calls.borrow_mut().push(cmd.to_string());
                if cmd == "kit" {
                    Err("kit missing".to_string())
                } else {
                    Ok(cmd.to_string())
                }
            },
            "test.ts",
        );

        assert_eq!(result, Some("bun".to_string()));
        assert_eq!(
            calls.borrow().as_slice(),
            &["kit".to_string(), "bun".to_string()]
        );
    }

    #[test]
    fn test_run_fallback_chain_returns_none_when_all_attempts_fail() {
        let start = Instant::now();
        let attempts = vec![
            RuntimeAttempt {
                name: "kit",
                label: "kit".to_string(),
                cmd: "kit".to_string(),
                args: vec!["run".to_string(), "test.ts".to_string()],
            },
            RuntimeAttempt {
                name: "node",
                label: "node".to_string(),
                cmd: "node".to_string(),
                args: vec!["test.js".to_string()],
            },
        ];

        let result: Option<String> = run_fallback_chain(
            &attempts,
            &start,
            |_, _, _| Err("failed".to_string()),
            "test.ts",
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_run_fallback_chain_passes_script_path_and_args_to_runner() {
        let start = Instant::now();
        let attempts = vec![RuntimeAttempt {
            name: "bun",
            label: "bun with preload".to_string(),
            cmd: "bun".to_string(),
            args: vec![
                "run".to_string(),
                "--preload".to_string(),
                "sdk.ts".to_string(),
                "test.ts".to_string(),
            ],
        }];

        let captured = RefCell::new((String::new(), Vec::new(), String::new()));
        let result = run_fallback_chain(
            &attempts,
            &start,
            |cmd, args, script_path| {
                captured.borrow_mut().0 = cmd.to_string();
                captured.borrow_mut().1 = args.iter().map(|arg| arg.to_string()).collect();
                captured.borrow_mut().2 = script_path.to_string();
                Ok("ok".to_string())
            },
            "test.ts",
        );

        assert_eq!(result, Some("ok".to_string()));
        assert_eq!(captured.borrow().0, "bun".to_string());
        assert_eq!(
            captured.borrow().1,
            vec![
                "run".to_string(),
                "--preload".to_string(),
                "sdk.ts".to_string(),
                "test.ts".to_string()
            ]
        );
        assert_eq!(captured.borrow().2, "test.ts".to_string());
    }
}

#[cfg(test)]
#[cfg(unix)]
mod env_scrub_tests {
    use super::spawn_script;
    use std::env;
    use std::ffi::OsString;
    use std::io::Read;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        key: &'static str,
        original: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = env::var_os(key);
            env::set_var(key, value);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.original {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn test_spawn_script_forwards_script_kit_vars_when_env_is_scrubbed() {
        let _lock = env_lock().lock().expect("env lock should be available");
        let _script_kit = EnvVarGuard::set("SCRIPT_KIT_ENV_SCRUB_SPAWN", "forwarded");
        let _private = EnvVarGuard::set("RUNNER_ENV_SCRUB_SPAWN_PRIVATE", "blocked");

        let mut session = spawn_script(
            "sh",
            &[
                "-c",
                "printf '%s|%s' \"${SCRIPT_KIT_ENV_SCRUB_SPAWN:-missing}\" \"${RUNNER_ENV_SCRUB_SPAWN_PRIVATE:-missing}\" >&2",
            ],
            "[test:runner_env_scrub_spawn]",
        )
        .expect("spawn_script should succeed");

        let exit_code = session.wait().expect("wait should succeed");
        assert_eq!(exit_code, 0);

        let mut stderr = String::new();
        let mut stderr_reader = session.stderr.take().expect("stderr should be captured");
        stderr_reader
            .read_to_string(&mut stderr)
            .expect("stderr should be readable");

        assert_eq!(stderr, "forwarded|missing");
    }
}

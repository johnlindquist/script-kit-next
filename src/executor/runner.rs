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

/// Embedded SDK content (included at compile time)
const EMBEDDED_SDK: &str = include_str!("../../scripts/kit-sdk.ts");

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

// Note: tsconfig.json path mapping is now handled by setup::ensure_kit_setup()
// This function is kept for backward compatibility but is a no-op
#[allow(dead_code)]
fn ensure_tsconfig_paths(_tsconfig_path: &PathBuf) {
    // Setup module now handles this at startup
    // See setup::ensure_tsconfig_paths()
}

/// Get the SDK path - SDK extraction is now handled by setup::ensure_kit_setup() at startup
/// This function just returns the expected path since setup has already done the work
fn ensure_sdk_extracted() -> Option<PathBuf> {
    // Target path: ~/.sk/kit/sdk/kit-sdk.ts
    // This is extracted by setup::ensure_kit_setup() which runs at app startup
    let sdk_path = dirs::home_dir()?.join(".sk/kit/sdk/kit-sdk.ts");

    if sdk_path.exists() {
        Some(sdk_path)
    } else {
        // Fallback: write embedded SDK if somehow missing
        // This shouldn't happen in normal operation since setup runs first
        logging::log(
            "EXEC",
            "SDK not found at expected path, extracting embedded SDK",
        );
        let kit_sdk = dirs::home_dir()?.join(".sk/kit/sdk");
        if !kit_sdk.exists() {
            std::fs::create_dir_all(&kit_sdk).ok()?;
        }
        std::fs::write(&sdk_path, EMBEDDED_SDK).ok()?;
        logging::log(
            "EXEC",
            &format!("Extracted fallback SDK to {}", sdk_path.display()),
        );
        Some(sdk_path)
    }
}

/// Find the SDK path, checking standard locations
pub fn find_sdk_path() -> Option<PathBuf> {
    logging::log("EXEC", "Looking for SDK...");

    // 1. Check ~/.sk/kit/sdk/kit-sdk.ts (primary location)
    if let Some(home) = dirs::home_dir() {
        let kit_sdk = home.join(".sk/kit/sdk/kit-sdk.ts");
        logging::log(
            "EXEC",
            &format!("  Checking kit sdk: {}", kit_sdk.display()),
        );
        if kit_sdk.exists() {
            logging::log("EXEC", &format!("  FOUND SDK (kit): {}", kit_sdk.display()));
            return Some(kit_sdk);
        }
    }

    // 2. Extract embedded SDK to ~/.sk/kit/sdk/kit-sdk.ts (production)
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

    /// Kill the process group (Unix) or just the process (other platforms)
    pub fn kill(&mut self) {
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
            // Kill the entire process group using the kill command with negative PID
            // Since we spawned with process_group(0), the PGID equals the PID
            // Using negative PID tells kill to target the process group
            let negative_pgid = format!("-{}", self.pid);
            logging::log(
                "EXEC",
                &format!("Killing process group PGID {} with SIGKILL", self.pid),
            );

            match Command::new("kill").args(["-9", &negative_pgid]).output() {
                Ok(output) => {
                    if output.status.success() {
                        logging::log(
                            "EXEC",
                            &format!("Successfully killed process group {}", self.pid),
                        );
                    } else {
                        // Process might already be dead, which is fine
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("No such process") {
                            logging::log(
                                "EXEC",
                                &format!("Process group {} already exited", self.pid),
                            );
                        } else {
                            logging::log(
                                "EXEC",
                                &format!("kill command failed for PGID {}: {}", self.pid, stderr),
                            );
                        }
                    }
                }
                Err(e) => {
                    logging::log("EXEC", &format!("Failed to execute kill command: {}", e));
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

    /// Check if process is still running (Unix only)
    #[cfg(unix)]
    #[allow(dead_code)]
    pub fn is_alive(&self) -> bool {
        // Use kill -0 to check if process exists
        Command::new("kill")
            .args(["-0", &self.pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
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
        logging::log("EXEC", &format!("Sending to script: {}", json));
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
            logging::log("EXEC", &format!("Received from script: {:?}", msg));
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

/// Execute a script with bidirectional JSONL communication
#[instrument(skip_all, fields(script_path = %path.display()))]
pub fn execute_script_interactive(path: &Path) -> Result<ScriptSession, String> {
    let start = Instant::now();
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

    // Try bun with preload (preferred - supports TypeScript natively)
    if let Some(ref sdk) = sdk_path {
        let sdk_str = sdk.to_str().unwrap_or("");
        logging::log(
            "EXEC",
            &format!("Trying: bun run --preload {} {}", sdk_str, path_str),
        );
        match spawn_script("bun", &["run", "--preload", sdk_str, path_str], path_str) {
            Ok(session) => {
                info!(
                    duration_ms = start.elapsed().as_millis() as u64,
                    runtime = "bun",
                    preload = true,
                    "Script session started"
                );
                logging::log("EXEC", "SUCCESS: bun with preload");
                return Ok(session);
            }
            Err(e) => {
                debug!(error = %e, runtime = "bun", preload = true, "Spawn attempt failed");
                logging::log("EXEC", &format!("FAILED: bun with preload: {}", e));
            }
        }
    }

    // Try bun without preload as fallback
    if is_typescript(path) {
        logging::log("EXEC", &format!("Trying: bun run {}", path_str));
        match spawn_script("bun", &["run", path_str], path_str) {
            Ok(session) => {
                info!(
                    duration_ms = start.elapsed().as_millis() as u64,
                    runtime = "bun",
                    preload = false,
                    "Script session started"
                );
                logging::log("EXEC", "SUCCESS: bun without preload");
                return Ok(session);
            }
            Err(e) => {
                debug!(error = %e, runtime = "bun", preload = false, "Spawn attempt failed");
                logging::log("EXEC", &format!("FAILED: bun without preload: {}", e));
            }
        }
    }

    // Try node for JavaScript files
    if is_javascript(path) {
        logging::log("EXEC", &format!("Trying: node {}", path_str));
        match spawn_script("node", &[path_str], path_str) {
            Ok(session) => {
                info!(
                    duration_ms = start.elapsed().as_millis() as u64,
                    runtime = "node",
                    "Script session started"
                );
                logging::log("EXEC", "SUCCESS: node");
                return Ok(session);
            }
            Err(e) => {
                debug!(error = %e, runtime = "node", "Spawn attempt failed");
                logging::log("EXEC", &format!("FAILED: node: {}", e));
            }
        }
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

/// Execute a script and return its output (non-interactive, for backwards compatibility)
#[allow(dead_code)]
#[instrument(skip_all, fields(script_path = %path.display()))]
pub fn execute_script(path: &Path) -> Result<String, String> {
    let start = Instant::now();
    debug!(path = %path.display(), "Starting blocking script execution");
    logging::log(
        "EXEC",
        &format!("execute_script (blocking): {}", path.display()),
    );

    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid path encoding".to_string())?;

    // Find SDK for preloading globals
    let sdk_path = find_sdk_path();
    logging::log("EXEC", &format!("SDK path: {:?}", sdk_path));

    // Try kit CLI first (preferred for script-kit)
    logging::log("EXEC", &format!("Trying: kit run {}", path_str));
    match run_command("kit", &["run", path_str]) {
        Ok(output) => {
            info!(
                duration_ms = start.elapsed().as_millis() as u64,
                output_bytes = output.len(),
                runtime = "kit",
                "Script completed"
            );
            logging::log(
                "EXEC",
                &format!("SUCCESS: kit (output: {} bytes)", output.len()),
            );
            return Ok(output);
        }
        Err(e) => {
            debug!(error = %e, runtime = "kit", "Command failed");
            logging::log("EXEC", &format!("FAILED: kit: {}", e));
        }
    }

    // Try bun with preload for TypeScript files (injects arg, div, md globals)
    if is_typescript(path) {
        if let Some(ref sdk) = sdk_path {
            let sdk_str = sdk.to_str().unwrap_or("");
            logging::log(
                "EXEC",
                &format!("Trying: bun run --preload {} {}", sdk_str, path_str),
            );
            match run_command("bun", &["run", "--preload", sdk_str, path_str]) {
                Ok(output) => {
                    info!(
                        duration_ms = start.elapsed().as_millis() as u64,
                        output_bytes = output.len(),
                        runtime = "bun",
                        preload = true,
                        "Script completed"
                    );
                    logging::log(
                        "EXEC",
                        &format!("SUCCESS: bun with preload (output: {} bytes)", output.len()),
                    );
                    return Ok(output);
                }
                Err(e) => {
                    debug!(error = %e, runtime = "bun", preload = true, "Command failed");
                    logging::log("EXEC", &format!("FAILED: bun with preload: {}", e));
                }
            }
        }

        // Fallback: try bun without preload
        logging::log(
            "EXEC",
            &format!("Trying: bun run {} (no preload)", path_str),
        );
        match run_command("bun", &["run", path_str]) {
            Ok(output) => {
                info!(
                    duration_ms = start.elapsed().as_millis() as u64,
                    output_bytes = output.len(),
                    runtime = "bun",
                    preload = false,
                    "Script completed"
                );
                logging::log(
                    "EXEC",
                    &format!("SUCCESS: bun (output: {} bytes)", output.len()),
                );
                return Ok(output);
            }
            Err(e) => {
                debug!(error = %e, runtime = "bun", preload = false, "Command failed");
                logging::log("EXEC", &format!("FAILED: bun: {}", e));
            }
        }
    }

    // Try node for JavaScript files
    if is_javascript(path) {
        logging::log("EXEC", &format!("Trying: node {}", path_str));
        match run_command("node", &[path_str]) {
            Ok(output) => {
                info!(
                    duration_ms = start.elapsed().as_millis() as u64,
                    output_bytes = output.len(),
                    runtime = "node",
                    "Script completed"
                );
                logging::log(
                    "EXEC",
                    &format!("SUCCESS: node (output: {} bytes)", output.len()),
                );
                return Ok(output);
            }
            Err(e) => {
                debug!(error = %e, runtime = "node", "Command failed");
                logging::log("EXEC", &format!("FAILED: node: {}", e));
            }
        }
    }

    let err = format!(
        "Failed to execute script '{}'. Make sure kit, bun, or node is installed.",
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

/// Run a command and capture its output
#[allow(dead_code)]
pub fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    // Try to find the executable in common locations
    let executable = find_executable(cmd)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| cmd.to_string());

    logging::log("EXEC", &format!("run_command: {} {:?}", executable, args));

    let output = Command::new(&executable).args(args).output().map_err(|e| {
        let err = format!("Failed to run '{}': {}", executable, e);
        logging::log("EXEC", &format!("COMMAND ERROR: {}", err));
        err
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    logging::log(
        "EXEC",
        &format!(
            "Command status: {}, stdout: {} bytes, stderr: {} bytes",
            output.status,
            stdout.len(),
            stderr.len()
        ),
    );

    if output.status.success() {
        if stdout.is_empty() {
            Ok(stderr.into_owned())
        } else {
            Ok(stdout.into_owned())
        }
    } else {
        let err = if stderr.is_empty() {
            format!("Command '{}' failed with status: {}", cmd, output.status)
        } else {
            stderr.into_owned()
        };
        logging::log("EXEC", &format!("Command failed: {}", err));
        Err(err)
    }
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

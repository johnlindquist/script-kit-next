use crate::logging;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{LazyLock, RwLock};
use sysinfo::{Pid, System};
/// Global singleton process manager
pub static PROCESS_MANAGER: LazyLock<ProcessManager> = LazyLock::new(ProcessManager::new);
/// Information about a tracked child process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Path to the script being executed
    pub script_path: String,
    /// Timestamp when the process was started
    pub started_at: DateTime<Utc>,
}
/// Thread-safe process manager for tracking bun script processes
#[derive(Debug)]
pub struct ProcessManager {
    /// Map of PID -> ProcessInfo for active child processes
    active_processes: RwLock<HashMap<u32, ProcessInfo>>,
    /// Path to main app PID file
    main_pid_path: PathBuf,
    /// Path to active child PIDs JSON file
    active_pids_path: PathBuf,
}
impl ProcessManager {
    /// Create a new ProcessManager with default paths
    pub fn new() -> Self {
        let kit_dir = dirs::home_dir()
            .map(|h| h.join(".scriptkit"))
            .unwrap_or_else(|| {
                // Use system temp directory instead of hardcoded /tmp for better security
                std::env::temp_dir().join(".scriptkit")
            });

        Self {
            active_processes: RwLock::new(HashMap::new()),
            main_pid_path: kit_dir.join("script-kit.pid"),
            active_pids_path: kit_dir.join("active-bun-pids.json"),
        }
    }

    /// Write the main application PID to disk
    ///
    /// This should be called at startup. On subsequent calls, it will
    /// overwrite the existing PID file.
    ///
    /// # Errors
    ///
    /// Returns an error if the PID file cannot be written.
    pub fn write_main_pid(&self) -> std::io::Result<()> {
        let pid = std::process::id();
        logging::log(
            "PROC",
            &format!("Writing main PID {} to {:?}", pid, self.main_pid_path),
        );

        // Ensure parent directory exists
        if let Some(parent) = self.main_pid_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&self.main_pid_path)?;
        write!(file, "{}", pid)?;

        logging::log("PROC", &format!("Main PID {} written successfully", pid));
        Ok(())
    }

    /// Remove the main PID file
    ///
    /// This should be called on clean shutdown.
    pub fn remove_main_pid(&self) {
        if self.main_pid_path.exists() {
            if let Err(e) = fs::remove_file(&self.main_pid_path) {
                logging::log("PROC", &format!("Failed to remove main PID file: {}", e));
            } else {
                logging::log("PROC", "Main PID file removed");
            }
        }
    }

    /// Read the main PID from disk, if it exists
    pub fn read_main_pid(&self) -> Option<u32> {
        if !self.main_pid_path.exists() {
            return None;
        }

        let mut file = File::open(&self.main_pid_path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        contents.trim().parse().ok()
    }

    /// Check if the main PID is stale (process no longer running)
    ///
    /// Returns true if there's a PID file but the process is not running.
    pub fn is_main_pid_stale(&self) -> bool {
        if let Some(pid) = self.read_main_pid() {
            !self.is_process_running(pid)
        } else {
            false
        }
    }

    /// Register a new child process
    ///
    /// This adds the process to the in-memory map and persists to disk.
    pub fn register_process(&self, pid: u32, script_path: &str) {
        let info = ProcessInfo {
            pid,
            script_path: script_path.to_string(),
            started_at: Utc::now(),
        };

        logging::log(
            "PROC",
            &format!(
                "Registering process PID {} for script: {}",
                pid, script_path
            ),
        );

        // Add to in-memory map
        if let Ok(mut processes) = self.active_processes.write() {
            processes.insert(pid, info);
        }

        // Persist to disk
        if let Err(e) = self.persist_active_pids() {
            logging::log("PROC", &format!("Failed to persist active PIDs: {}", e));
        }
    }

    /// Unregister a child process
    ///
    /// This removes the process from tracking when it exits normally.
    pub fn unregister_process(&self, pid: u32) {
        logging::log("PROC", &format!("Unregistering process PID {}", pid));

        // Remove from in-memory map
        if let Ok(mut processes) = self.active_processes.write() {
            processes.remove(&pid);
        }

        // Persist to disk
        if let Err(e) = self.persist_active_pids() {
            logging::log("PROC", &format!("Failed to persist active PIDs: {}", e));
        }
    }

    /// Get all currently tracked active processes
    pub fn get_active_processes(&self) -> Vec<ProcessInfo> {
        if let Ok(processes) = self.active_processes.read() {
            processes.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all active processes sorted by start time (newest first).
    pub fn get_active_processes_sorted(&self) -> Vec<ProcessInfo> {
        let mut processes = self.get_active_processes();
        processes.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        processes
    }

    /// Build a human-readable process report that can be shown in the UI/clipboard.
    pub fn format_active_process_report(&self, max_entries: usize) -> String {
        let processes = self.get_active_processes_sorted();
        if processes.is_empty() {
            return "No active Script Kit processes.".to_string();
        }

        let total = processes.len();
        let limit = max_entries.max(1);
        let mut lines = Vec::with_capacity(limit + 2);
        lines.push(format!("Active Script Kit processes: {}", total));

        for process in processes.iter().take(limit) {
            lines.push(format!(
                "PID {} • {} • started {}",
                process.pid,
                process.script_path,
                process.started_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }

        if total > limit {
            lines.push(format!("... and {} more", total - limit));
        }

        lines.join("\n")
    }

    /// Get count of active processes
    pub fn active_count(&self) -> usize {
        if let Ok(processes) = self.active_processes.read() {
            processes.len()
        } else {
            0
        }
    }

    /// Kill all tracked child processes
    ///
    /// This sends SIGKILL to each tracked process group.
    /// Used during graceful shutdown.
    pub fn kill_all_processes(&self) {
        let processes: Vec<ProcessInfo> = if let Ok(procs) = self.active_processes.read() {
            procs.values().cloned().collect()
        } else {
            Vec::new()
        };

        if processes.is_empty() {
            logging::log("PROC", "No active processes to kill");
            return;
        }

        logging::log(
            "PROC",
            &format!("Killing {} active process(es)", processes.len()),
        );

        for info in &processes {
            self.kill_process(info.pid);
        }

        // Clear the in-memory map
        if let Ok(mut procs) = self.active_processes.write() {
            procs.clear();
        }

        // Remove the active PIDs file
        if self.active_pids_path.exists() {
            if let Err(e) = fs::remove_file(&self.active_pids_path) {
                logging::log("PROC", &format!("Failed to remove active PIDs file: {}", e));
            }
        }

        logging::log("PROC", "All processes killed and tracking cleared");
    }

    /// Kill a single process by PID
    ///
    /// Sends SIGKILL to the process group on Unix.
    pub fn kill_process(&self, pid: u32) {
        logging::log("PROC", &format!("Killing process PID {}", pid));

        #[cfg(unix)]
        {
            // Kill the entire process group
            let negative_pgid = format!("-{}", pid);
            match Command::new("kill").args(["-9", &negative_pgid]).output() {
                Ok(output) => {
                    if output.status.success() {
                        logging::log(
                            "PROC",
                            &format!("Successfully killed process group {}", pid),
                        );
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("No such process") {
                            logging::log("PROC", &format!("Process {} already exited", pid));
                        } else {
                            logging::log(
                                "PROC",
                                &format!("Failed to kill process {}: {}", pid, stderr),
                            );
                        }
                    }
                }
                Err(e) => {
                    logging::log("PROC", &format!("Failed to execute kill command: {}", e));
                }
            }
        }

        #[cfg(not(unix))]
        {
            logging::log(
                "PROC",
                &format!("Non-Unix platform: cannot kill process {}", pid),
            );
        }
    }

    /// Check if a process is currently running
    pub fn is_process_running(&self, pid: u32) -> bool {
        let mut system = System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        system.process(Pid::from_u32(pid)).is_some()
    }

    /// Detect and clean up orphaned processes from a previous crash
    ///
    /// This should be called at startup. It reads the persisted PID file,
    /// checks which processes are still running, kills them, and clears the file.
    ///
    /// Returns the number of orphans killed.
    pub fn cleanup_orphans(&self) -> usize {
        logging::log(
            "PROC",
            "Checking for orphaned processes from previous session",
        );

        let orphans = self.load_persisted_pids();
        if orphans.is_empty() {
            logging::log("PROC", "No orphaned processes found");
            return 0;
        }

        logging::log(
            "PROC",
            &format!("Found {} potentially orphaned process(es)", orphans.len()),
        );

        let mut killed_count = 0;

        for info in &orphans {
            if self.is_process_running(info.pid) {
                logging::log(
                    "PROC",
                    &format!(
                        "Killing orphaned process PID {} (script: {})",
                        info.pid, info.script_path
                    ),
                );
                self.kill_process(info.pid);
                killed_count += 1;
            } else {
                logging::log("PROC", &format!("Orphan PID {} already exited", info.pid));
            }
        }

        // Clear the persisted file
        if self.active_pids_path.exists() {
            if let Err(e) = fs::remove_file(&self.active_pids_path) {
                logging::log("PROC", &format!("Failed to remove orphan PIDs file: {}", e));
            }
        }

        if killed_count > 0 {
            logging::log(
                "PROC",
                &format!("Cleaned up {} orphaned process(es)", killed_count),
            );
        }

        killed_count
    }

    /// Persist the current active PIDs to disk
    fn persist_active_pids(&self) -> std::io::Result<()> {
        let processes: Vec<ProcessInfo> = if let Ok(procs) = self.active_processes.read() {
            procs.values().cloned().collect()
        } else {
            Vec::new()
        };

        // Ensure parent directory exists
        if let Some(parent) = self.active_pids_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&processes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.active_pids_path)?;

        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load persisted PIDs from disk
    fn load_persisted_pids(&self) -> Vec<ProcessInfo> {
        if !self.active_pids_path.exists() {
            return Vec::new();
        }

        let contents = match fs::read_to_string(&self.active_pids_path) {
            Ok(c) => c,
            Err(e) => {
                logging::log("PROC", &format!("Failed to read active PIDs file: {}", e));
                return Vec::new();
            }
        };

        match serde_json::from_str(&contents) {
            Ok(pids) => pids,
            Err(e) => {
                logging::log("PROC", &format!("Failed to parse active PIDs JSON: {}", e));
                Vec::new()
            }
        }
    }
}
impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

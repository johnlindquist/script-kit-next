# Process Lifecycle Management - Expert Bundle

## Overview

Script Kit manages multiple child processes (bun scripts) with proper tracking, cleanup, and crash recovery.

## Process Manager Architecture

### Global Singleton (src/process_manager.rs)

```rust
use std::sync::{LazyLock, RwLock};
use std::collections::HashMap;

/// Global singleton process manager
pub static PROCESS_MANAGER: LazyLock<ProcessManager> = LazyLock::new(ProcessManager::new);

pub struct ProcessManager {
    /// Map of PID -> ProcessInfo for active child processes
    active_processes: RwLock<HashMap<u32, ProcessInfo>>,
    /// Path to main app PID file (~/.scriptkit/script-kit.pid)
    main_pid_path: PathBuf,
    /// Path to active child PIDs JSON (~/.scriptkit/active-bun-pids.json)
    active_pids_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub script_path: String,
    pub started_at: DateTime<Utc>,
}
```

## Lifecycle Operations

### Main App PID Management

```rust
impl ProcessManager {
    /// Write the main application PID to disk (call at startup)
    pub fn write_main_pid(&self) -> std::io::Result<()> {
        let pid = std::process::id();
        logging::log("PROC", &format!("Writing main PID {}", pid));

        if let Some(parent) = self.main_pid_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&self.main_pid_path)?;
        write!(file, "{}", pid)?;
        Ok(())
    }

    /// Remove the main PID file (call on clean shutdown)
    pub fn remove_main_pid(&self) {
        if self.main_pid_path.exists() {
            if let Err(e) = fs::remove_file(&self.main_pid_path) {
                logging::log("PROC", &format!("Failed to remove main PID file: {}", e));
            }
        }
    }

    /// Check if the main PID is stale (process no longer running)
    pub fn is_main_pid_stale(&self) -> bool {
        if let Some(pid) = self.read_main_pid() {
            !self.is_process_running(pid)
        } else {
            false
        }
    }
}
```

### Child Process Registration

```rust
impl ProcessManager {
    /// Register a new child process
    pub fn register_process(&self, pid: u32, script_path: &str) {
        let info = ProcessInfo {
            pid,
            script_path: script_path.to_string(),
            started_at: Utc::now(),
        };

        logging::log("PROC", &format!(
            "Registering process PID {} for script: {}", 
            pid, script_path
        ));

        // Add to in-memory map
        if let Ok(mut processes) = self.active_processes.write() {
            processes.insert(pid, info);
        }

        // Persist to disk for crash recovery
        if let Err(e) = self.persist_active_pids() {
            logging::log("PROC", &format!("Failed to persist active PIDs: {}", e));
        }
    }

    /// Unregister a child process (call when it exits normally)
    pub fn unregister_process(&self, pid: u32) {
        logging::log("PROC", &format!("Unregistering process PID {}", pid));

        if let Ok(mut processes) = self.active_processes.write() {
            processes.remove(&pid);
        }

        if let Err(e) = self.persist_active_pids() {
            logging::log("PROC", &format!("Failed to persist active PIDs: {}", e));
        }
    }
}
```

### Process Killing

```rust
impl ProcessManager {
    /// Kill all tracked child processes (for graceful shutdown)
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

        logging::log("PROC", &format!(
            "Killing {} active process(es)", 
            processes.len()
        ));

        for info in &processes {
            self.kill_process(info.pid);
        }

        // Clear the in-memory map
        if let Ok(mut procs) = self.active_processes.write() {
            procs.clear();
        }

        // Remove the active PIDs file
        if self.active_pids_path.exists() {
            let _ = fs::remove_file(&self.active_pids_path);
        }
    }

    /// Kill a single process by PID (sends SIGKILL to process group)
    pub fn kill_process(&self, pid: u32) {
        logging::log("PROC", &format!("Killing process PID {}", pid));

        #[cfg(unix)]
        {
            // Kill the entire process group
            let negative_pgid = format!("-{}", pid);
            match Command::new("kill").args(["-9", &negative_pgid]).output() {
                Ok(output) => {
                    if output.status.success() {
                        logging::log("PROC", &format!(
                            "Successfully killed process group {}", pid
                        ));
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("No such process") {
                            logging::log("PROC", &format!("Process {} already exited", pid));
                        } else {
                            logging::log("PROC", &format!(
                                "Failed to kill process {}: {}", pid, stderr
                            ));
                        }
                    }
                }
                Err(e) => {
                    logging::log("PROC", &format!("Failed to execute kill: {}", e));
                }
            }
        }
    }
}
```

## Orphan Detection and Cleanup

```rust
impl ProcessManager {
    /// Detect and clean up orphaned processes from a previous crash
    /// Call this at startup before writing the new main PID
    pub fn cleanup_orphans(&self) -> usize {
        logging::log("PROC", "Checking for orphaned processes");

        let orphans = self.load_persisted_pids();
        if orphans.is_empty() {
            logging::log("PROC", "No orphaned processes found");
            return 0;
        }

        logging::log("PROC", &format!(
            "Found {} potentially orphaned process(es)", 
            orphans.len()
        ));

        let mut killed_count = 0;

        for info in &orphans {
            if self.is_process_running(info.pid) {
                logging::log("PROC", &format!(
                    "Killing orphaned process PID {} (script: {})",
                    info.pid, info.script_path
                ));
                self.kill_process(info.pid);
                killed_count += 1;
            } else {
                logging::log("PROC", &format!(
                    "Orphan PID {} already exited", 
                    info.pid
                ));
            }
        }

        // Clear the persisted file
        if self.active_pids_path.exists() {
            let _ = fs::remove_file(&self.active_pids_path);
        }

        killed_count
    }

    /// Check if a process is currently running
    pub fn is_process_running(&self, pid: u32) -> bool {
        let mut system = System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        system.process(Pid::from_u32(pid)).is_some()
    }
}
```

## Persistence Layer

```rust
impl ProcessManager {
    /// Persist the current active PIDs to disk
    fn persist_active_pids(&self) -> std::io::Result<()> {
        let processes: Vec<ProcessInfo> = if let Ok(procs) = self.active_processes.read() {
            procs.values().cloned().collect()
        } else {
            Vec::new()
        };

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

        match fs::read_to_string(&self.active_pids_path) {
            Ok(contents) => {
                serde_json::from_str(&contents).unwrap_or_default()
            }
            Err(_) => Vec::new()
        }
    }
}
```

## Startup Sequence

```rust
fn main() {
    // 1. Clean up orphans from previous crash
    let orphans_killed = PROCESS_MANAGER.cleanup_orphans();
    if orphans_killed > 0 {
        info!("Cleaned up {} orphaned processes", orphans_killed);
    }
    
    // 2. Write our PID
    if let Err(e) = PROCESS_MANAGER.write_main_pid() {
        error!("Failed to write main PID: {}", e);
    }
    
    // 3. Register shutdown handler
    let _ = ctrlc::set_handler(|| {
        PROCESS_MANAGER.kill_all_processes();
        PROCESS_MANAGER.remove_main_pid();
        std::process::exit(0);
    });
    
    // ... rest of app startup
}
```

## Script Execution Integration

```rust
pub fn spawn_script(path: &str) -> anyhow::Result<Child> {
    let mut cmd = Command::new("bun");
    cmd.arg("run")
       .arg("--preload")
       .arg(get_sdk_path())
       .arg(path);
    
    // Set process group for clean termination
    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            // Create new process group
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    
    let child = cmd.spawn()?;
    let pid = child.id();
    
    // Register with process manager
    PROCESS_MANAGER.register_process(pid, path);
    
    Ok(child)
}

pub async fn wait_for_script(mut child: Child, path: &str) -> anyhow::Result<ExitStatus> {
    let pid = child.id();
    let status = child.wait().await?;
    
    // Unregister when done
    PROCESS_MANAGER.unregister_process(pid);
    
    Ok(status)
}
```

## File Locations

```
~/.scriptkit/
├── script-kit.pid           # Main app PID
├── active-bun-pids.json     # Active child processes
└── logs/
    └── script-kit-gpui.jsonl  # Log file
```

### active-bun-pids.json Format

```json
[
  {
    "pid": 12345,
    "script_path": "/Users/john/.scriptkit/scripts/hello.ts",
    "started_at": "2024-01-15T10:30:00Z"
  },
  {
    "pid": 12346,
    "script_path": "/Users/john/.scriptkit/scripts/timer.ts",
    "started_at": "2024-01-15T10:31:00Z"
  }
]
```

## Best Practices

### 1. Always Register Before Spawn Completes

```rust
// Good - register immediately after spawn
let child = cmd.spawn()?;
PROCESS_MANAGER.register_process(child.id(), path);

// Bad - might miss if process exits quickly
let child = cmd.spawn()?;
// ... other code ...
PROCESS_MANAGER.register_process(child.id(), path);
```

### 2. Handle Race Conditions

```rust
// Use RwLock for thread-safe access
if let Ok(mut processes) = self.active_processes.write() {
    processes.insert(pid, info);
}
```

### 3. Graceful vs Force Kill

```rust
// Try graceful first
fn graceful_kill(&self, pid: u32) {
    #[cfg(unix)]
    {
        // SIGTERM first
        let _ = Command::new("kill")
            .args(["-15", &pid.to_string()])
            .output();
        
        // Wait briefly
        std::thread::sleep(Duration::from_millis(100));
        
        // Check if still running
        if self.is_process_running(pid) {
            // Force kill
            let _ = Command::new("kill")
                .args(["-9", &format!("-{}", pid)])
                .output();
        }
    }
}
```

### 4. Log All Lifecycle Events

```rust
logging::log("PROC", &format!("Registering process PID {}", pid));
logging::log("PROC", &format!("Unregistering process PID {}", pid));
logging::log("PROC", &format!("Killing process group PID {}", pid));
logging::log("PROC", &format!("Cleaned up {} orphans", count));
```

## Summary

1. **Global singleton** ProcessManager tracks all child processes
2. **Persist PIDs to disk** for crash recovery
3. **Clean up orphans on startup** before normal operation
4. **Kill process groups** not just individual processes
5. **Register immediately** after spawn, unregister on exit
6. **Use RwLock** for thread-safe access
7. **Log all lifecycle events** for debugging

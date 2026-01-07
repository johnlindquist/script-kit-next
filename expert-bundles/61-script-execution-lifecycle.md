# Script Execution Lifecycle - Expert Bundle

## Overview

Script Kit manages script execution through a multi-stage lifecycle: discovery, parsing, spawning, communication, and cleanup.

## Execution Stages

```
Discovery → Parsing → Spawning → Communication → Cleanup
    ↓          ↓          ↓            ↓           ↓
  Find       Parse     Start        JSONL       Cleanup
 scripts    metadata    bun        protocol     processes
```

## Script Discovery

### Reading Scripts (src/scripts.rs)

```rust
pub fn read_scripts() -> Vec<Arc<Script>> {
    let scripts_dir = dirs::home_dir()
        .unwrap()
        .join(".scriptkit/scripts");
    
    let mut scripts = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&scripts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Only .ts and .js files
            if !is_script_file(&path) {
                continue;
            }
            
            if let Some(script) = parse_script(&path) {
                scripts.push(Arc::new(script));
            }
        }
    }
    
    // Sort by name
    scripts.sort_by(|a, b| a.name.cmp(&b.name));
    scripts
}

fn is_script_file(path: &Path) -> bool {
    path.extension()
        .map(|e| e == "ts" || e == "js")
        .unwrap_or(false)
}
```

## Metadata Parsing

### Comment-Based Metadata

```rust
pub fn parse_script(path: &Path) -> Option<Script> {
    let contents = fs::read_to_string(path).ok()?;
    
    let mut script = Script {
        path: path.to_path_buf(),
        name: path.file_stem()?.to_string_lossy().to_string(),
        ..Default::default()
    };
    
    // Parse comment-based metadata
    for line in contents.lines().take(50) { // First 50 lines
        let line = line.trim();
        
        if let Some(value) = line.strip_prefix("// Name:") {
            script.name = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("// Description:") {
            script.description = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("// Shortcut:") {
            script.shortcut = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("// Author:") {
            script.author = Some(value.trim().to_string());
        }
    }
    
    Some(script)
}
```

### Export Metadata (Preferred)

```rust
pub fn parse_export_metadata(contents: &str) -> Option<ScriptMetadata> {
    // Look for: export const metadata = { ... }
    let re = Regex::new(r"export\s+const\s+metadata\s*=\s*(\{[\s\S]*?\})\s*;?").ok()?;
    
    if let Some(cap) = re.captures(contents) {
        // Parse as JSON5 (allows trailing commas, comments)
        let json_str = &cap[1];
        json5::from_str(json_str).ok()
    } else {
        None
    }
}

#[derive(Debug, Deserialize)]
pub struct ScriptMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub shortcut: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub background: bool,
}
```

## Script Spawning

### Execute Script (src/executor/runner.rs)

```rust
pub async fn spawn_script(path: &str) -> Result<ProcessHandle> {
    let sdk_path = find_sdk_path()?;
    let bun_path = find_executable("bun")?;
    
    let mut cmd = Command::new(&bun_path);
    cmd.arg("run")
       .arg("--preload")
       .arg(&sdk_path)
       .arg(path);
    
    // Set up environment
    cmd.env("SCRIPT_KIT_PATH", path);
    
    // Configure for bidirectional communication
    cmd.stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    // Create new process group (for clean termination)
    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    
    let child = cmd.spawn()?;
    let pid = child.id();
    
    // Register with process manager
    PROCESS_MANAGER.register_process(pid, path);
    
    logging::log("EXEC", &format!(
        "Spawned script {} with PID {}", path, pid
    ));
    
    Ok(ProcessHandle {
        child,
        path: path.to_string(),
    })
}
```

### SDK Preloading

```rust
pub fn find_sdk_path() -> Result<PathBuf> {
    let sdk_path = dirs::home_dir()
        .ok_or_else(|| anyhow!("No home directory"))?
        .join(".scriptkit/sdk/kit-sdk.ts");
    
    if !sdk_path.exists() {
        // Extract embedded SDK
        ensure_sdk_extracted()?;
    }
    
    Ok(sdk_path)
}

fn ensure_sdk_extracted() -> Result<()> {
    let sdk_content = include_str!("../scripts/kit-sdk.ts");
    let sdk_path = dirs::home_dir()
        .unwrap()
        .join(".scriptkit/sdk/kit-sdk.ts");
    
    if let Some(parent) = sdk_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&sdk_path, sdk_content)?;
    Ok(())
}
```

## Interactive Session

### Script Session

```rust
pub struct ScriptSession {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr_buffer: StderrBuffer,
    path: String,
}

impl ScriptSession {
    pub async fn new(path: &str) -> Result<Self> {
        let handle = spawn_script(path).await?;
        let mut child = handle.child;
        
        let stdin = BufWriter::new(child.stdin.take().unwrap());
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let stderr_buffer = StderrBuffer::new(child.stderr.take().unwrap());
        
        Ok(Self {
            child,
            stdin,
            stdout,
            stderr_buffer,
            path: path.to_string(),
        })
    }

    pub async fn send(&mut self, msg: &AppToScriptMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<ScriptToAppMessage>> {
        let mut line = String::new();
        match self.stdout.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF - script exited
            Ok(_) => {
                let msg = serde_json::from_str(&line)?;
                Ok(Some(msg))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn wait(mut self) -> Result<ExitStatus> {
        let status = self.child.wait().await?;
        PROCESS_MANAGER.unregister_process(self.child.id());
        Ok(status)
    }
}
```

## Communication Protocol

### App to Script Messages

```rust
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppToScriptMessage {
    // User input
    #[serde(rename = "input")]
    Input { value: String },
    
    // Form submission
    #[serde(rename = "submit")]
    Submit { value: serde_json::Value },
    
    // User cancelled
    #[serde(rename = "escape")]
    Escape,
    
    // Window events
    #[serde(rename = "blur")]
    Blur,
    
    #[serde(rename = "focus")]
    Focus,
    
    // Choice selection
    #[serde(rename = "select")]
    Select { index: usize, value: serde_json::Value },
}
```

### Script to App Messages

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ScriptToAppMessage {
    // UI updates
    #[serde(rename = "setPrompt")]
    SetPrompt {
        prompt: String,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        choices: Vec<Choice>,
    },
    
    #[serde(rename = "setChoices")]
    SetChoices { choices: Vec<Choice> },
    
    #[serde(rename = "setHint")]
    SetHint { hint: String },
    
    #[serde(rename = "setInput")]
    SetInput { value: String },
    
    // Window control
    #[serde(rename = "show")]
    Show,
    
    #[serde(rename = "hide")]
    Hide,
    
    #[serde(rename = "resize")]
    Resize { width: Option<f32>, height: Option<f32> },
    
    // Lifecycle
    #[serde(rename = "exit")]
    Exit { code: i32 },
    
    // Notifications
    #[serde(rename = "notify")]
    Notify { title: String, body: String },
}
```

## Message Handling

### Processing Messages

```rust
impl App {
    async fn run_script_session(&mut self, path: &str, cx: &mut Context<Self>) {
        let mut session = match ScriptSession::new(path).await {
            Ok(s) => s,
            Err(e) => {
                self.show_error(&e, cx);
                return;
            }
        };
        
        loop {
            match session.recv().await {
                Ok(Some(msg)) => {
                    self.handle_script_message(msg, cx);
                }
                Ok(None) => {
                    // Script exited normally
                    break;
                }
                Err(e) => {
                    // Check stderr for error details
                    let stderr = session.stderr_buffer.read();
                    self.handle_script_error(&e, &stderr, cx);
                    break;
                }
            }
        }
        
        let status = session.wait().await;
        self.handle_script_exit(status, cx);
    }

    fn handle_script_message(&mut self, msg: ScriptToAppMessage, cx: &mut Context<Self>) {
        match msg {
            ScriptToAppMessage::SetPrompt { prompt, placeholder, choices } => {
                self.set_prompt(&prompt, placeholder.as_deref());
                if !choices.is_empty() {
                    self.set_choices(choices);
                }
                cx.notify();
            }
            ScriptToAppMessage::SetChoices { choices } => {
                self.set_choices(choices);
                cx.notify();
            }
            ScriptToAppMessage::Resize { width, height } => {
                self.resize_window(width, height, cx);
            }
            ScriptToAppMessage::Exit { code } => {
                self.handle_exit(code, cx);
            }
            // ... other messages
        }
    }
}
```

## Cleanup

### Process Cleanup

```rust
impl App {
    fn cleanup_current_script(&mut self, cx: &mut Context<Self>) {
        if let Some(pid) = self.current_script_pid.take() {
            // Kill process and all children
            PROCESS_MANAGER.kill_process(pid);
            PROCESS_MANAGER.unregister_process(pid);
            
            logging::log("EXEC", &format!("Cleaned up script PID {}", pid));
        }
        
        // Reset UI state
        self.reset_prompt_state(cx);
    }

    fn handle_script_exit(&mut self, status: Result<ExitStatus>, cx: &mut Context<Self>) {
        match status {
            Ok(status) if status.success() => {
                logging::log("EXEC", "Script exited successfully");
            }
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                logging::log("EXEC", &format!("Script exited with code {}", code));
                
                if code != 0 {
                    // Show error to user
                    self.show_exit_error(code, cx);
                }
            }
            Err(e) => {
                logging::log("EXEC", &format!("Script wait failed: {}", e));
            }
        }
        
        self.current_script_pid = None;
        self.reset_prompt_state(cx);
    }
}
```

### Graceful Shutdown

```rust
fn graceful_shutdown() {
    logging::log("APP", "Starting graceful shutdown");
    
    // Kill all script processes
    PROCESS_MANAGER.kill_all_processes();
    
    // Remove main PID file
    PROCESS_MANAGER.remove_main_pid();
    
    // Flush logs
    logging::flush();
    
    logging::log("APP", "Shutdown complete");
}

// Register shutdown handler
fn setup_signal_handlers() {
    ctrlc::set_handler(|| {
        graceful_shutdown();
        std::process::exit(0);
    }).ok();
}
```

## Error Recovery

### Script Errors

```rust
pub fn extract_error_message(stderr: &str) -> Option<String> {
    // Look for common error patterns
    if let Some(line) = stderr.lines().find(|l| l.contains("Error:")) {
        return Some(line.to_string());
    }
    
    // Return first non-empty line
    stderr.lines().find(|l| !l.is_empty()).map(String::from)
}

pub fn generate_suggestions(error: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    if error.contains("Cannot find module") {
        suggestions.push("Run 'bun install' to install dependencies".to_string());
    }
    
    if error.contains("SyntaxError") {
        suggestions.push("Check for syntax errors in your script".to_string());
    }
    
    suggestions
}
```

## Best Practices

1. **Always register processes** - enables cleanup on crash
2. **Set process group** - allows killing all children
3. **Use buffered I/O** - improves performance
4. **Handle stderr separately** - capture error details
5. **Graceful shutdown** - clean up all processes
6. **Parse metadata early** - avoid re-parsing
7. **Extract SDK once** - don't embed in each script

## Lifecycle Summary

| Stage | Location | Key Operations |
|-------|----------|----------------|
| Discovery | `scripts.rs` | Find .ts/.js files |
| Parsing | `metadata_parser.rs` | Extract metadata |
| Spawning | `executor/runner.rs` | Start bun process |
| Communication | `executor/mod.rs` | JSONL protocol |
| Cleanup | `process_manager.rs` | Kill, unregister |

# IPC & Messaging Patterns - Expert Bundle

## Overview

Script Kit uses multiple IPC mechanisms: stdin/stdout JSONL protocol for script communication, async channels for internal messaging, and GCD dispatch for cross-thread coordination.

## Stdin JSONL Protocol

### Protocol Messages (src/protocol.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StdinCommand {
    /// Run a script
    #[serde(rename = "run")]
    Run { path: String },
    
    /// Show the window
    #[serde(rename = "show")]
    Show,
    
    /// Hide the window
    #[serde(rename = "hide")]
    Hide,
    
    /// Set filter text
    #[serde(rename = "setFilter")]
    SetFilter { text: String },
    
    /// Show debug grid overlay
    #[serde(rename = "showGrid")]
    ShowGrid {
        #[serde(default)]
        show_bounds: bool,
        #[serde(default)]
        show_box_model: bool,
        #[serde(default)]
        show_alignment_guides: bool,
        #[serde(default)]
        show_dimensions: bool,
    },
    
    /// Hide debug grid
    #[serde(rename = "hideGrid")]
    HideGrid,
    
    /// Open Notes window
    #[serde(rename = "openNotes")]
    OpenNotes,
    
    /// Open AI window
    #[serde(rename = "openAi")]
    OpenAi,
}
```

### Stdin Reader

```rust
pub fn spawn_stdin_reader(sender: Sender<StdinCommand>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        
        for line in reader.lines() {
            match line {
                Ok(line) if !line.trim().is_empty() => {
                    match serde_json::from_str::<StdinCommand>(&line) {
                        Ok(cmd) => {
                            logging::log("STDIN", &format!("Received: {:?}", cmd));
                            if sender.send_blocking(cmd).is_err() {
                                break; // Channel closed
                            }
                        }
                        Err(e) => {
                            logging::log("STDIN", &format!("Parse error: {}", e));
                        }
                    }
                }
                Ok(_) => {} // Empty line
                Err(e) => {
                    logging::log("STDIN", &format!("Read error: {}", e));
                    break;
                }
            }
        }
    });
}
```

## Script-to-App Communication

### Script Session

```rust
pub struct ScriptSession {
    child: Child,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
    stderr_buffer: StderrBuffer,
}

impl ScriptSession {
    /// Send a message to the script
    pub async fn send(&mut self, msg: &AppToScriptMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Receive a message from the script
    pub async fn recv(&mut self) -> Result<Option<ScriptToAppMessage>> {
        let mut line = String::new();
        match self.stdout_reader.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                let msg = serde_json::from_str(&line)?;
                Ok(Some(msg))
            }
            Err(e) => Err(e.into()),
        }
    }
}
```

### Message Types

```rust
/// App -> Script messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppToScriptMessage {
    #[serde(rename = "input")]
    Input { value: String },
    
    #[serde(rename = "submit")]
    Submit { value: serde_json::Value },
    
    #[serde(rename = "escape")]
    Escape,
    
    #[serde(rename = "blur")]
    Blur,
    
    #[serde(rename = "focus")]
    Focus,
}

/// Script -> App messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ScriptToAppMessage {
    #[serde(rename = "setPrompt")]
    SetPrompt {
        prompt: String,
        #[serde(default)]
        choices: Vec<Choice>,
    },
    
    #[serde(rename = "setHint")]
    SetHint { hint: String },
    
    #[serde(rename = "setChoices")]
    SetChoices { choices: Vec<Choice> },
    
    #[serde(rename = "exit")]
    Exit { code: i32 },
    
    #[serde(rename = "showNotification")]
    ShowNotification {
        title: String,
        body: String,
    },
}
```

## Async Channels

### Channel Patterns

```rust
use async_channel::{bounded, Sender, Receiver};

// Bounded channel for backpressure
static HOTKEY_CHANNEL: OnceLock<(Sender<()>, Receiver<()>)> = OnceLock::new();

pub fn hotkey_channel() -> &'static (Sender<()>, Receiver<()>) {
    HOTKEY_CHANNEL.get_or_init(|| bounded(10))
}

// Usage patterns:

// Non-blocking send (hotkey thread)
hotkey_channel().0.try_send(()).ok();

// Blocking send (when backpressure is OK)
hotkey_channel().0.send(()).await?;

// Non-blocking receive (UI polling)
while let Ok(()) = hotkey_channel().1.try_recv() {
    self.handle_hotkey(cx);
}

// Async receive (background task)
if let Ok(()) = hotkey_channel().1.recv().await {
    // Handle
}
```

### Typed Channels

```rust
// Script hotkey channel sends script path
static SCRIPT_HOTKEY_CHANNEL: OnceLock<(
    Sender<String>,
    Receiver<String>,
)> = OnceLock::new();

// UI update channel with rich events
#[derive(Debug, Clone)]
pub enum UiEvent {
    FilterChanged(String),
    SelectionChanged(usize),
    PromptChanged(PromptState),
    ThemeReloaded(Theme),
}

static UI_EVENT_CHANNEL: OnceLock<(
    Sender<UiEvent>,
    Receiver<UiEvent>,
)> = OnceLock::new();
```

## GCD Dispatch (macOS)

### Main Thread Dispatch

```rust
#[cfg(target_os = "macos")]
mod gcd {
    use std::ffi::c_void;

    #[link(name = "System", kind = "framework")]
    extern "C" {
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        #[link_name = "_dispatch_main_q"]
        static DISPATCH_MAIN_QUEUE: c_void;
    }

    /// Dispatch a closure to the main thread
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        let boxed: Box<dyn FnOnce() + Send> = Box::new(f);
        let raw = Box::into_raw(Box::new(boxed));

        extern "C" fn trampoline(context: *mut c_void) {
            unsafe {
                let boxed: Box<Box<dyn FnOnce() + Send>> = 
                    Box::from_raw(context as *mut _);
                // Catch panics to prevent UB
                let _ = std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| boxed())
                );
            }
        }

        unsafe {
            let main_queue = &DISPATCH_MAIN_QUEUE as *const c_void;
            dispatch_async_f(main_queue, raw as *mut c_void, trampoline);
        }
    }
}
```

### Usage Pattern

```rust
fn dispatch_notes_hotkey() {
    let handler = NOTES_HANDLER.lock().unwrap().clone();

    if let Some(handler) = handler {
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // Fallback to channel
        notes_hotkey_channel().0.try_send(()).ok();
        // Wake GPUI event loop
        gcd::dispatch_to_main(|| {});
    }
}
```

## GPUI Context Updates

### Spawning Async Tasks

```rust
impl App {
    fn spawn_script(&mut self, path: &str, cx: &mut Context<Self>) {
        let path = path.to_string();
        
        cx.spawn(|this, mut cx| async move {
            let mut session = execute_script_interactive(&path).await?;
            
            loop {
                match session.recv().await? {
                    Some(msg) => {
                        // Update UI from async context
                        let _ = this.update(&mut cx, |app, cx| {
                            app.handle_script_message(msg, cx);
                        });
                    }
                    None => break, // Script exited
                }
            }
            
            Ok::<_, anyhow::Error>(())
        }).detach();
    }
}
```

### Model Updates

```rust
cx.spawn(|this, mut cx| async move {
    let result = fetch_data().await?;
    
    // Update model
    let _ = this.update(&mut cx, |app, cx| {
        app.data = result;
        cx.notify(); // Trigger re-render
    });
    
    Ok::<_, anyhow::Error>(())
}).detach();
```

## Window Communication

### Multi-Window Messaging

```rust
pub struct WindowRegistry {
    windows: RwLock<HashMap<WindowId, WindowHandle>>,
}

impl WindowRegistry {
    pub fn broadcast(&self, event: &AppEvent) {
        if let Ok(windows) = self.windows.read() {
            for (_, handle) in windows.iter() {
                handle.update(|view, cx| {
                    view.handle_app_event(event, cx);
                });
            }
        }
    }

    pub fn send_to(&self, id: WindowId, event: AppEvent) -> Option<()> {
        let windows = self.windows.read().ok()?;
        let handle = windows.get(&id)?;
        handle.update(|view, cx| {
            view.handle_app_event(&event, cx);
        });
        Some(())
    }
}
```

## Stdin Command Handling

```rust
impl App {
    fn handle_stdin_command(&mut self, cmd: StdinCommand, cx: &mut Context<Self>) {
        match cmd {
            StdinCommand::Run { path } => {
                self.run_script(&path, cx);
            }
            StdinCommand::Show => {
                self.show_window(cx);
            }
            StdinCommand::Hide => {
                self.hide_window(cx);
            }
            StdinCommand::SetFilter { text } => {
                self.set_filter(&text, cx);
            }
            StdinCommand::OpenNotes => {
                crate::notes::window::open_notes(cx);
            }
            StdinCommand::OpenAi => {
                crate::ai::window::open_ai(cx);
            }
            StdinCommand::ShowGrid { show_bounds, .. } => {
                self.debug_grid.show(show_bounds);
                cx.notify();
            }
            StdinCommand::HideGrid => {
                self.debug_grid.hide();
                cx.notify();
            }
        }
    }
}
```

## Best Practices

### 1. Use Bounded Channels

```rust
// Good - bounded prevents memory growth
let (tx, rx) = async_channel::bounded(10);

// Bad - unbounded can grow forever
let (tx, rx) = async_channel::unbounded();
```

### 2. Non-Blocking in Hot Paths

```rust
// Good - non-blocking in hotkey thread
if sender.try_send(msg).is_err() {
    logging::log("WARN", "Channel full, dropping message");
}

// Bad - blocking in hot path
sender.send(msg).await; // Can block
```

### 3. Update UI from Main Thread

```rust
// Good - update through cx
cx.spawn(|this, mut cx| async move {
    let data = fetch().await?;
    this.update(&mut cx, |app, cx| {
        app.data = data;
        cx.notify();
    })?;
    Ok(())
}).detach();

// Bad - direct mutation (race condition)
self.data = fetch().await?; // Not thread-safe!
```

### 4. Handle Channel Closure

```rust
loop {
    match receiver.recv().await {
        Ok(msg) => handle(msg),
        Err(RecvError::Closed) => {
            logging::log("INFO", "Channel closed, exiting");
            break;
        }
    }
}
```

## Summary

| Mechanism | Use Case | Thread Safety |
|-----------|----------|---------------|
| Stdin JSONL | External control | Dedicated thread |
| Script stdout | Script communication | Async task |
| Async channels | Internal events | Send + Sync |
| GCD dispatch | macOS main thread | FFI safe |
| cx.spawn/update | GPUI async | Model updates |

# Error Handling Patterns - Expert Bundle

## Overview

Comprehensive error handling architecture for Script Kit, combining Rust's type safety with user-friendly recovery.

## Error Type Hierarchy

### Core Error Types (src/error.rs)

```rust
/// Error severity for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,     // Blue - informational
    Warning,  // Yellow - recoverable
    Error,    // Red - operation failed
    Critical, // Red + modal - requires user action
}

/// Domain-specific errors for Script Kit
#[derive(Error, Debug)]
pub enum ScriptKitError {
    #[error("Script execution failed: {message}")]
    ScriptExecution {
        message: String,
        script_path: Option<String>,
    },

    #[error("Failed to parse protocol message: {0}")]
    ProtocolParse(#[from] serde_json::Error),

    #[error("Theme loading failed for '{path}': {source}")]
    ThemeLoad {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Process spawn failed: {0}")]
    ProcessSpawn(String),

    #[error("File watch error: {0}")]
    FileWatch(String),

    #[error("Window operation failed: {0}")]
    Window(String),
}
```

### Severity Mapping

```rust
impl ScriptKitError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ScriptExecution { .. } => ErrorSeverity::Error,
            Self::ProtocolParse(_) => ErrorSeverity::Warning,
            Self::ThemeLoad { .. } => ErrorSeverity::Warning,
            Self::Config(_) => ErrorSeverity::Warning,
            Self::ProcessSpawn(_) => ErrorSeverity::Error,
            Self::FileWatch(_) => ErrorSeverity::Warning,
            Self::Window(_) => ErrorSeverity::Error,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::ScriptExecution { message, .. } => message.clone(),
            Self::ProtocolParse(e) => format!("Invalid message format: {}", e),
            Self::ThemeLoad { path, .. } => format!("Could not load theme from {}", path),
            Self::Config(msg) => format!("Configuration issue: {}", msg),
            Self::ProcessSpawn(msg) => format!("Could not start process: {}", msg),
            Self::FileWatch(msg) => format!("File watcher issue: {}", msg),
            Self::Window(msg) => msg.clone(),
        }
    }
}
```

## Error Extension Traits

### Silent Error Logging

```rust
/// Extension trait for silent error logging with caller location tracking.
pub trait ResultExt<T> {
    /// Log error with caller location and return None. Use for recoverable failures.
    fn log_err(self) -> Option<T>;
    /// Log as warning with caller location and return None. Use for expected failures.
    fn warn_on_err(self) -> Option<T>;
}

impl<T, E: std::fmt::Debug> ResultExt<T> for std::result::Result<T, E> {
    #[track_caller]
    fn log_err(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                let caller = std::panic::Location::caller();
                error!(
                    error = ?error,
                    file = caller.file(),
                    line = caller.line(),
                    "Operation failed"
                );
                None
            }
        }
    }

    #[track_caller]
    fn warn_on_err(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                let caller = std::panic::Location::caller();
                warn!(
                    error = ?error,
                    file = caller.file(),
                    line = caller.line(),
                    "Operation had warning"
                );
                None
            }
        }
    }
}
```

### Async Error Logging

```rust
/// Log an error from an async operation. Use for fire-and-forget patterns.
pub fn log_async_err<T, E: std::fmt::Debug>(
    result: std::result::Result<T, E>,
    operation: &str,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(err) => {
            error!(
                error = ?err,
                operation = operation,
                "Async operation failed"
            );
            None
        }
    }
}
```

## Debug Panic Macro

```rust
/// Panic in debug mode, log error in release mode.
///
/// Use for "impossible" states that should crash during development
/// but gracefully degrade in production.
#[macro_export]
macro_rules! debug_panic {
    ( $($fmt_arg:tt)* ) => {
        if cfg!(debug_assertions) {
            panic!( $($fmt_arg)* );
        } else {
            tracing::error!("IMPOSSIBLE STATE: {}", format_args!($($fmt_arg)*));
        }
    };
}
```

## Script Error Parsing (src/executor/errors.rs)

### Stack Trace Parsing

```rust
pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

pub fn parse_stack_trace(stderr: &str) -> Vec<StackFrame> {
    let mut frames = Vec::new();
    let re = Regex::new(r"at\s+(.+?)\s+\((.+):(\d+):(\d+)\)").unwrap();
    
    for cap in re.captures_iter(stderr) {
        frames.push(StackFrame {
            function: cap[1].to_string(),
            file: cap[2].to_string(),
            line: cap[3].parse().unwrap_or(0),
            column: cap[4].parse().unwrap_or(0),
        });
    }
    frames
}
```

### Error Suggestions

```rust
pub fn generate_suggestions(error_message: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    if error_message.contains("Cannot find module") {
        suggestions.push("Run `bun install` to install dependencies".to_string());
        suggestions.push("Check if the module path is correct".to_string());
    }
    
    if error_message.contains("SyntaxError") {
        suggestions.push("Check for missing brackets or quotes".to_string());
        suggestions.push("Verify TypeScript/JavaScript syntax".to_string());
    }
    
    if error_message.contains("Permission denied") {
        suggestions.push("Check file permissions".to_string());
        suggestions.push("Try running with appropriate access".to_string());
    }
    
    suggestions
}
```

## Crash Recovery

```rust
pub struct CrashInfo {
    pub signal: i32,
    pub signal_name: String,
    pub pid: u32,
    pub script_path: String,
}

pub fn signal_to_name(signal: i32) -> &'static str {
    match signal {
        1 => "SIGHUP",
        2 => "SIGINT",
        9 => "SIGKILL",
        11 => "SIGSEGV",
        15 => "SIGTERM",
        _ => "UNKNOWN",
    }
}

pub fn generate_crash_suggestions(crash: &CrashInfo) -> Vec<String> {
    match crash.signal {
        11 => vec![
            "Segmentation fault - check native module compatibility".to_string(),
            "Verify Bun version matches expected".to_string(),
        ],
        9 => vec![
            "Process was killed - possibly OOM".to_string(),
            "Check system memory usage".to_string(),
        ],
        _ => vec![format!("Process exited with signal {}", crash.signal_name)],
    }
}
```

## Error Display in UI

### Toast Notifications

```rust
impl App {
    pub fn show_error(&mut self, error: &ScriptKitError, cx: &mut Context<Self>) {
        let severity = error.severity();
        let message = error.user_message();
        
        let duration = match severity {
            ErrorSeverity::Info => Duration::from_secs(3),
            ErrorSeverity::Warning => Duration::from_secs(5),
            ErrorSeverity::Error => Duration::from_secs(10),
            ErrorSeverity::Critical => Duration::from_secs(30),
        };
        
        self.hud_manager.show_notification(
            &message,
            severity,
            duration,
            cx,
        );
    }
}
```

### Error Notification Component

```rust
pub struct ErrorNotification {
    error: ScriptKitError,
    suggestions: Vec<String>,
    dismiss_after: Option<Instant>,
}

impl Render for ErrorNotification {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let severity_color = match self.error.severity() {
            ErrorSeverity::Info => rgb(0x3B82F6),
            ErrorSeverity::Warning => rgb(0xF59E0B),
            ErrorSeverity::Error => rgb(0xEF4444),
            ErrorSeverity::Critical => rgb(0xDC2626),
        };
        
        div()
            .flex()
            .flex_col()
            .bg(severity_color.opacity(0.1))
            .border_l_4()
            .border_color(severity_color)
            .p_4()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.error.user_message())
            )
            .children(self.suggestions.iter().map(|s| {
                div()
                    .text_xs()
                    .text_color(rgb(0x6B7280))
                    .child(format!("• {}", s))
            }))
    }
}
```

## Best Practices

### 1. Context Chaining

```rust
// Always add context at each level
fn load_config(path: &Path) -> anyhow::Result<Config> {
    let contents = fs::read_to_string(path)
        .context(format!("Failed to read config from {:?}", path))?;
    
    let config: Config = serde_json::from_str(&contents)
        .context("Failed to parse config JSON")?;
    
    Ok(config)
}
```

### 2. Never Unwrap in Production Paths

```rust
// Bad
let value = some_result.unwrap();

// Good - with recovery
let value = match some_result {
    Ok(v) => v,
    Err(e) => {
        error!(error = ?e, "Operation failed");
        return default_value;
    }
};

// Good - with extension trait
let value = some_result.log_err().unwrap_or(default_value);
```

### 3. Typed Error Fields in Logs

```rust
// Bad - string interpolation hides structure
error!("Failed to execute script {} with error {}", path, err);

// Good - typed fields for querying
error!(
    script_path = %path,
    error = ?err,
    exit_code = code,
    "Script execution failed"
);
```

### 4. Graceful Degradation

```rust
fn load_theme(&mut self) -> Theme {
    match self.try_load_custom_theme() {
        Ok(theme) => theme,
        Err(e) => {
            warn!(error = ?e, "Custom theme failed, using default");
            Theme::default()
        }
    }
}
```

## Testing Error Handling

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_mapping() {
        let err = ScriptKitError::ScriptExecution {
            message: "test".to_string(),
            script_path: None,
        };
        assert_eq!(err.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_parse_stack_trace() {
        let stderr = r#"
            at someFunction (/path/to/file.ts:10:5)
            at anotherFunction (/path/to/other.ts:20:3)
        "#;
        let frames = parse_stack_trace(stderr);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].line, 10);
    }

    #[test]
    fn test_suggestion_generation() {
        let suggestions = generate_suggestions("Cannot find module 'foo'");
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("bun install"));
    }
}
```

## Summary

- Use `ScriptKitError` for domain errors with severity
- Use `ResultExt` traits for ergonomic error handling
- Add context at every level with `.context()`
- Use `debug_panic!` for impossible states
- Parse script errors to generate actionable suggestions
- Display errors with appropriate severity styling
- Never `unwrap()` in production code paths

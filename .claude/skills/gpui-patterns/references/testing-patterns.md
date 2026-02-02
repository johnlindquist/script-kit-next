# GPUI Testing Patterns

## Test File Organization

```
src/
  theme/
    mod.rs           # Implementation
    theme_tests.rs   # Tests (separate file)
```

Import in mod.rs:
```rust
#[cfg(test)]
mod theme_tests;
```

## Test Helpers (Reduce Boilerplate)

```rust
fn test_scriptlet(name: &str, tool: &str, code: &str) -> Scriptlet {
    Scriptlet { name: name.to_string(), tool: tool.to_string(), code: code.to_string(), ..Default::default() }
}

fn wrap_scripts(scripts: Vec<Script>) -> Vec<Arc<Script>> {
    scripts.into_iter().map(Arc::new).collect()
}
```

## System Tests (Feature-Gated)

Tests that need OS permissions (clipboard, accessibility, hotkeys):
```rust
#[cfg(feature = "system-tests")]
#[test]
fn test_clipboard_integration() { ... }
```

Run with: `cargo test --features system-tests`

## Platform-Specific Tests

```rust
#[cfg(target_os = "macos")]
#[test]
fn test_macos_specific() { ... }

#[cfg(unix)]
#[test]
fn test_unix_signals() { ... }
```

## Environment Variable Tests

```rust
#[test]
fn test_env_var() {
    // Save original
    let original = std::env::var("MY_VAR").ok();

    // Test
    std::env::set_var("MY_VAR", "value");
    assert!(check_var());

    // Restore (always!)
    match original {
        Some(v) => std::env::set_var("MY_VAR", v),
        None => std::env::remove_var("MY_VAR"),
    }
}
```

## Code Audit Tests

Enforce invariants about the codebase:
```rust
#[test]
fn test_no_direct_cx_hide() {
    let content = fs::read_to_string("src/app_execute.rs").unwrap_or_default();
    assert!(!content.contains("cx.hide()"), "Use close_and_reset_window() instead");
}
```

## Running Tests

```bash
cargo test                           # All tests
cargo test theme_tests               # Specific module
cargo test --features system-tests   # System tests
cargo test -- --nocapture            # Show output
cargo test test_default_config       # Single test
```

## Anti-Patterns

- **DON'T** use `cx.run()` in unit tests (needs running app)
- **DON'T** rely on global state between tests
- **DON'T** hardcode paths (`/Users/john/...`) - use temp dirs
- **DON'T** forget platform guards for OS-specific tests
- **DON'T** skip cleanup (env vars, temp files)

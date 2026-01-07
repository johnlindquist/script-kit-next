# Testing Strategies - Expert Bundle

## Overview

Comprehensive testing approach for Script Kit combining Rust unit tests, integration tests, and visual regression testing via the SDK.

## Test Directory Structure

```
tests/
├── smoke/           # E2E tests run via stdin JSON protocol
│   ├── test-arg.ts
│   ├── test-div.ts
│   ├── test-editor-height.ts
│   └── hello-world.ts
├── sdk/             # SDK unit tests run directly with bun
│   ├── test-scroll-perf.ts
│   └── test-arg.ts
src/
├── *_tests.rs       # Inline test modules
└── lib.rs           # #[cfg(test)] mod tests
```

## Rust Unit Testing

### Test Module Pattern

```rust
// src/filter_coalescer.rs
#[cfg(test)]
mod tests {
    use super::FilterCoalescer;

    #[test]
    fn coalescer_returns_latest_value_on_tick() {
        let mut coalescer = FilterCoalescer::new();

        assert!(coalescer.queue("a"));
        assert!(!coalescer.queue("ab")); // Returns false - already pending

        assert_eq!(coalescer.take_latest().as_deref(), Some("ab"));
    }

    #[test]
    fn coalescer_only_starts_one_task_per_batch() {
        let mut coalescer = FilterCoalescer::new();

        assert!(coalescer.queue("first"));  // true - starts batch
        assert!(!coalescer.queue("second")); // false - batch pending
        assert!(!coalescer.queue("third"));  // false - batch pending

        assert!(coalescer.take_latest().is_some());
        assert!(coalescer.queue("next")); // true - new batch
    }
}
```

### Test Helpers

```rust
// src/process_manager.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a ProcessManager with a temporary directory for testing
    fn create_test_manager() -> (ProcessManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProcessManager {
            active_processes: RwLock::new(HashMap::new()),
            main_pid_path: temp_dir.path().join("script-kit.pid"),
            active_pids_path: temp_dir.path().join("active-bun-pids.json"),
        };
        (manager, temp_dir)
    }

    #[test]
    fn test_register_and_unregister_process() {
        let (manager, _temp_dir) = create_test_manager();

        manager.register_process(12345, "/path/to/test.ts");

        let active = manager.get_active_processes();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].pid, 12345);

        manager.unregister_process(12345);
        assert!(manager.get_active_processes().is_empty());
    }
}
```

### Feature-Gated Tests

```rust
// Tests requiring system access
#[cfg(test)]
#[cfg(feature = "system-tests")]
mod system_tests {
    use super::*;

    #[test]
    fn test_clipboard_integration() {
        // This test modifies the system clipboard
        let clipboard = SystemClipboard::new();
        clipboard.write_text("test");
        assert_eq!(clipboard.read_text(), Some("test".to_string()));
    }
}
```

## Running Tests

```bash
# All unit tests
cargo test

# Specific test
cargo test test_register_and_unregister

# With output
cargo test -- --nocapture

# System tests (clipboard, accessibility)
cargo test --features system-tests

# Ignored/interactive tests
cargo test --features system-tests -- --ignored
```

## E2E Testing via Stdin Protocol

### Test Script Template

```typescript
// tests/smoke/test-arg.ts
import '../../scripts/kit-sdk';

function log(test: string, status: string, extra: any = {}) {
  console.log(JSON.stringify({ 
    test, 
    status, 
    timestamp: new Date().toISOString(), 
    ...extra 
  }));
}

const name = "arg-string-choices";
log(name, "running");
const start = Date.now();

try {
  const result = await arg("Pick a fruit", ["Apple", "Banana", "Cherry"]);
  log(name, "pass", { result, duration_ms: Date.now() - start });
} catch (e) {
  log(name, "fail", { error: String(e), duration_ms: Date.now() - start });
}

process.exit(0);
```

### Running E2E Tests

```bash
# Build first
cargo build

# Run via stdin JSON protocol
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-arg.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# With log filtering
echo '{"type":"run","path":"..."}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'pass|fail|error'
```

### Test Output Format

```json
{"test":"arg-string-choices","status":"running","timestamp":"2024-..."}
{"test":"arg-string-choices","status":"pass","result":"Apple","duration_ms":45}
```

Status values: `running`, `pass`, `fail`, `skip`

## Visual Regression Testing

### Screenshot Capture

```typescript
// tests/smoke/visual-test.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Render UI
await div(`
  <div class="p-4 bg-blue-500 text-white rounded-lg">
    Test Component
  </div>
`);

// Wait for render
await new Promise(r => setTimeout(r, 500));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `visual-test-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
```

### Layout Inspection

```typescript
// Get programmatic layout info
const layout = await getLayoutInfo();

console.log('Window:', layout.windowWidth, 'x', layout.windowHeight);
console.log('Prompt type:', layout.promptType);

for (const comp of layout.components) {
  console.log(`${comp.name}: ${comp.bounds.width}x${comp.bounds.height}`);
}
```

### Grid Overlay Testing

```bash
# Show grid overlay then run test
(echo '{"type":"showGrid","showBounds":true,"showDimensions":true}'; \
 echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-layout.ts"}') | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

## Database Testing

### Isolated Test Database

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_db() -> Result<(TempDir, Arc<Mutex<Connection>>)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.sqlite");

        let conn = Connection::open(&db_path)?;
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS menu_cache (
                bundle_id TEXT PRIMARY KEY,
                menu_json TEXT NOT NULL,
                last_scanned INTEGER NOT NULL
            );
        "#)?;

        Ok((temp_dir, Arc::new(Mutex::new(conn))))
    }

    #[test]
    fn test_cache_insert_and_retrieve() {
        let (_temp_dir, db) = setup_test_db().unwrap();
        
        // Test insert
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO menu_cache VALUES (?1, ?2, ?3)",
                params!["com.test.app", "{}", 12345],
            ).unwrap();
        }
        
        // Test retrieve
        {
            let conn = db.lock().unwrap();
            let result: Option<String> = conn
                .query_row(
                    "SELECT menu_json FROM menu_cache WHERE bundle_id = ?1",
                    params!["com.test.app"],
                    |row| row.get(0),
                )
                .optional()
                .unwrap();
            
            assert!(result.is_some());
        }
    }
}
```

## Hotkey Testing

### Channel Testing

```rust
#[test]
fn hotkey_channels_are_independent() {
    // Drain any pending messages
    while hotkey_channel().1.try_recv().is_ok() {}
    while script_hotkey_channel().1.try_recv().is_ok() {}

    // Send to main hotkey channel
    hotkey_channel().0.send_blocking(()).expect("send hotkey");
    
    // Script channel should be empty
    assert!(matches!(
        script_hotkey_channel().1.try_recv(),
        Err(TryRecvError::Empty)
    ));
    
    // Main channel should have message
    assert!(hotkey_channel().1.try_recv().is_ok());
}
```

### Route Testing

```rust
#[test]
fn test_add_script_route() {
    let mut routes = HotkeyRoutes::new();
    let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyT);
    let path = "/test/script.ts".to_string();
    
    routes.add_route(hotkey.id(), RegisteredHotkey {
        hotkey,
        action: HotkeyAction::Script(path.clone()),
        display: "cmd+shift+t".to_string(),
    });

    assert_eq!(routes.get_script_id(&path), Some(hotkey.id()));
    assert_eq!(
        routes.get_action(hotkey.id()),
        Some(HotkeyAction::Script(path))
    );
}
```

## Performance Testing

### Script Performance

```typescript
// tests/sdk/test-scroll-perf.ts
import '../../scripts/kit-sdk';

const samples: number[] = [];
const iterations = 100;

for (let i = 0; i < iterations; i++) {
  const start = performance.now();
  
  // Simulate scroll operation
  await arg("Pick", ["A", "B", "C", "D", "E"]);
  
  samples.push(performance.now() - start);
}

const sorted = samples.sort((a, b) => a - b);
const p50 = sorted[Math.floor(samples.length * 0.5)];
const p95 = sorted[Math.floor(samples.length * 0.95)];
const p99 = sorted[Math.floor(samples.length * 0.99)];

console.log(JSON.stringify({
  test: "scroll-perf",
  p50_ms: p50.toFixed(2),
  p95_ms: p95.toFixed(2),
  p99_ms: p99.toFixed(2),
}));

// Thresholds
if (p95 > 50) {
  console.error("FAIL: P95 latency exceeds 50ms");
  process.exit(1);
}
```

### Rust Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_filter_1000_scripts() {
        let scripts: Vec<Script> = (0..1000)
            .map(|i| Script { name: format!("script-{}", i), ..Default::default() })
            .collect();
        
        let start = Instant::now();
        let _filtered: Vec<_> = scripts
            .iter()
            .filter(|s| s.name.contains("99"))
            .collect();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 10, "Filter took too long: {:?}", duration);
    }
}
```

## Test Organization Tips

### 1. One Assertion per Test (When Practical)

```rust
#[test]
fn test_opacity_clamp_upper() {
    let clamped = Opacity::new(1.5);
    assert!((clamped.0 - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_opacity_clamp_lower() {
    let clamped = Opacity::new(-0.5);
    assert!((clamped.0 - 0.0).abs() < f32::EPSILON);
}
```

### 2. Test Edge Cases

```rust
#[test]
fn test_empty_filter() {
    let coalescer = FilterCoalescer::new();
    assert!(coalescer.take_latest().is_none());
}

#[test]
fn test_clear_filter() {
    let mut coalescer = FilterCoalescer::new();
    assert!(coalescer.queue("query"));
    assert!(!coalescer.queue("")); // Clear is a valid update
    assert_eq!(coalescer.take_latest().as_deref(), Some(""));
}
```

### 3. Use Descriptive Names

```rust
#[test]
fn coalescer_returns_latest_value_when_multiple_queued() { }

#[test]
fn process_manager_removes_pid_on_unregister() { }

#[test]
fn cache_expires_after_max_age() { }
```

## Pre-Commit Verification Gate

```bash
# ALWAYS run before commit
cargo check && cargo clippy --all-targets -- -D warnings && cargo test

# All three must pass
```

## Summary

1. **Unit tests** in `#[cfg(test)]` modules
2. **E2E tests** via stdin JSON protocol
3. **Visual tests** using `captureScreenshot()` + `getLayoutInfo()`
4. **Database tests** with `tempfile::TempDir`
5. **Feature-gated** system tests for clipboard/accessibility
6. **Performance tests** with timing assertions
7. **Always run verification gate** before commits

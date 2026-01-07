# Debugging & Troubleshooting - Expert Bundle

## Overview

Comprehensive debugging guide for Script Kit including log analysis, visual debugging, common issues, and resolution patterns.

## Log Analysis

### AI Compact Log Mode

```bash
# Enable compact logs (saves ~70% tokens)
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Format: SS.mmm|L|C|message
# Example: 13.150|i|P|Selected display origin=(0,0)
```

### Log Categories

| Code | Category | What to Look For |
|------|----------|------------------|
| P | POSITION | Window placement issues |
| A | APP | Lifecycle events |
| U | UI | Rendering problems |
| S | STDIN | Protocol parsing |
| H | HOTKEY | Shortcut failures |
| V | VISIBILITY | Show/hide issues |
| E | EXEC | Script execution |
| K | KEY | Keyboard events |
| F | FOCUS | Focus problems |
| T | THEME | Theme loading |
| Z | RESIZE | Window sizing |

### Filtering Logs

```bash
# Filter by category
grep '|E|' log.txt  # Execution only

# Filter by level
grep '|e|' log.txt  # Errors only
grep -E '\|e\||\|w\|' log.txt  # Errors and warnings

# Multiple categories
grep -E '\|E\||\|H\||\|K\|' log.txt
```

### JSONL Queries

```bash
# Recent errors
tail -100 ~/.scriptkit/logs/script-kit-gpui.jsonl | grep ERROR

# Slow operations
cat log.jsonl | jq 'select(.fields.duration_ms > 100)'

# Script execution timeline
grep '"EXEC"' log.jsonl | jq '{time: .timestamp, msg: .message}'
```

## Visual Debugging

### Grid Overlay

```bash
# Show bounds
echo '{"type":"showGrid","showBounds":true}' | ./target/debug/script-kit-gpui

# Full debug info
echo '{"type":"showGrid","showBounds":true,"showDimensions":true,"showBoxModel":true}' | ./target/debug/script-kit-gpui

# Hide
echo '{"type":"hideGrid"}' | ./target/debug/script-kit-gpui
```

### Screenshot Capture

```typescript
// In test script
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';

await div(`<div>UI to capture</div>`);
await new Promise(r => setTimeout(r, 500));

const shot = await captureScreenshot();
mkdirSync('.test-screenshots', { recursive: true });
writeFileSync('.test-screenshots/debug.png', Buffer.from(shot.data, 'base64'));
process.exit(0);
```

### Layout Inspection

```typescript
const layout = await getLayoutInfo();
console.log('Window:', layout.windowWidth, 'x', layout.windowHeight);
console.log('Prompt:', layout.promptType);

for (const comp of layout.components) {
    console.log(`${comp.name}: ${comp.bounds.width}x${comp.bounds.height} at (${comp.bounds.x}, ${comp.bounds.y})`);
}
```

## Common Issues

### 1. Window Not Appearing

**Symptoms:**
- App runs but no window
- Logs show no errors

**Check:**
```bash
# Check visibility logs
grep -E '\|V\|' log.txt

# Check window creation
grep -E 'window|display' log.txt
```

**Common Causes:**
- Window positioned off-screen
- Display ID mismatch
- Focus not set

**Fix:**
```rust
// Use centered positioning
Bounds::centered(None, size, cx)

// Activate after creation
cx.activate(true);
```

### 2. Hotkey Not Working

**Symptoms:**
- Global shortcut doesn't trigger
- Works in some apps, not others

**Check:**
```bash
grep '|H|' log.txt
```

**Common Causes:**
- Registration failed
- Conflict with another app
- Wrong key code format

**Fix:**
```rust
// Check for conflict
match manager.register(hotkey) {
    Err(HotkeyError::AlreadyRegistered(_)) => {
        logging::log("HOTKEY", "Conflict - try different shortcut");
    }
    // ...
}
```

### 3. Script Execution Fails

**Symptoms:**
- Script starts but produces no UI
- Error in stderr

**Check:**
```bash
# Check execution logs
grep '|E|' log.txt

# Check stderr capture
grep 'stderr' log.txt
```

**Common Causes:**
- SDK not found
- Bun not in PATH
- Syntax error in script

**Fix:**
```bash
# Verify bun
which bun

# Verify SDK
ls ~/.scriptkit/sdk/kit-sdk.ts

# Test script directly
bun run --preload ~/.scriptkit/sdk/kit-sdk.ts path/to/script.ts
```

### 4. UI Not Updating

**Symptoms:**
- Selection doesn't highlight
- State changes not reflected

**Check:**
```rust
// Missing cx.notify()!
fn set_filter(&mut self, text: String, cx: &mut Context<Self>) {
    self.filter_text = text;
    self.recompute_filtered();
    cx.notify(); // REQUIRED!
}
```

**Common Cause:**
- Missing `cx.notify()` after state change

### 5. Focus Issues

**Symptoms:**
- Keyboard events not received
- Focus ring not showing

**Check:**
```bash
grep '|F|' log.txt
```

**Common Causes:**
- Focus handle not tracked
- Not implementing Focusable
- Focus not set after window activation

**Fix:**
```rust
impl Focusable for MyView {
    fn focus_handle(&self, _cx: &Context<Self>) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// In render
div().track_focus(&self.focus_handle)
```

### 6. Arrow Keys Not Working

**Symptoms:**
- Up/Down arrows don't navigate

**Check:**
- Are you matching both variants?

**Fix:**
```rust
match key.as_str() {
    // Handle BOTH variants!
    "up" | "arrowup" => self.move_up(cx),
    "down" | "arrowdown" => self.move_down(cx),
    _ => {}
}
```

### 7. Slow Scrolling

**Symptoms:**
- Laggy list navigation
- Missed keystrokes

**Cause:**
- No key coalescing

**Fix:**
```rust
// Coalesce rapid key events
if self.scroll_coalescer.queue(direction) {
    cx.spawn(|this, mut cx| async move {
        Timer::after(Duration::from_millis(20)).await;
        this.update(&mut cx, |view, cx| view.flush_scroll(cx));
    }).detach();
}
```

### 8. Theme Not Applied

**Symptoms:**
- Hardcoded colors showing
- Theme changes not reflected

**Check:**
```rust
// Bad - hardcoded
div().bg(rgb(0x1E1E1E))

// Good - from theme
div().bg(rgb(theme.colors.background.main))
```

### 9. Process Orphans

**Symptoms:**
- Bun processes still running after app crash

**Check:**
```bash
# Find orphans
ps aux | grep bun

# Check PID file
cat ~/.scriptkit/active-bun-pids.json
```

**Fix:**
```bash
# Manual cleanup
kill $(cat ~/.scriptkit/active-bun-pids.json | jq -r '.[].pid')
rm ~/.scriptkit/active-bun-pids.json
```

## Debug Checklist

### Before Committing

```bash
# 1. Verification gate
cargo check && cargo clippy --all-targets -- -D warnings && cargo test

# 2. Manual test
echo '{"type":"run","path":"tests/smoke/hello-world.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### When UI Looks Wrong

1. Capture screenshot
2. Get layout info
3. Enable grid overlay
4. Check log categories U, Z, P

### When Scripts Fail

1. Test script directly with bun
2. Check stderr buffer
3. Verify SDK path
4. Check process registration

## Debug Panel

### Open Debug Panel

```
Cmd+L (in app)
```

### Filter by Tag

```
[UI] [EXEC] [KEY] [THEME] [FOCUS] [HOTKEY] [PANEL]
```

### Performance Tags

```
[KEY_PERF] [SCROLL_TIMING] [PERF_SLOW]
```

## Summary

| Issue | First Check |
|-------|-------------|
| No window | Logs: `|V|`, `|P|` |
| No hotkey | Logs: `|H|` |
| Script fails | Direct bun test |
| No update | `cx.notify()` calls |
| No focus | Focusable impl |
| Arrow keys | Both variants |
| Slow scroll | Coalescing |
| Wrong colors | Theme usage |
| Orphans | PID file |

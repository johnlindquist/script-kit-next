# How to Automate AI Agent Debugging Sessions

This document describes the **gold-standard workflow** for AI agents debugging and fixing issues in Script Kit GPUI. This workflow was derived from a highly successful debugging session that fixed the vibrancy loss issue.

---

## Table of Contents

1. [Philosophy](#philosophy)
2. [The Complete Workflow](#the-complete-workflow)
3. [Phase 1: Exploration](#phase-1-exploration)
4. [Phase 2: Implementation](#phase-2-implementation)
5. [Phase 3: Verification](#phase-3-verification)
6. [Logging Reference](#logging-reference)
7. [Visual Testing](#visual-testing)
8. [Real-World Example](#real-world-example)
9. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
10. [Success Checklist](#success-checklist)

---

## Philosophy

**The AI agent is responsible for the entire fix lifecycle.** Never ask the user to test. Never assume a fix works without verification. The agent must:

1. **Explore** - Understand before coding
2. **Fix** - Make targeted, minimal changes
3. **Build** - Ensure compilation succeeds
4. **Launch** - Actually run the application
5. **Verify** - Confirm the fix via logs/screenshots
6. **Test** - Run the full test suite

This is non-negotiable. Skipping any step leads to unreliable fixes.

---

## The Complete Workflow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AUTONOMOUS FIX-VERIFY WORKFLOW                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌──────────────┐                                                          │
│   │   EXPLORE    │ ◄── Use Task tool with explore agent                     │
│   │              │     Read relevant files                                  │
│   │              │     Identify root cause                                  │
│   └──────┬───────┘                                                          │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐                                                          │
│   │     FIX      │ ◄── Make minimal, targeted edits                         │
│   │              │     One logical change at a time                         │
│   └──────┬───────┘                                                          │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐                                                          │
│   │    BUILD     │ ◄── cargo check && cargo clippy --all-targets            │
│   │              │     -- -D warnings                                       │
│   └──────┬───────┘                                                          │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐                                                          │
│   │   LAUNCH     │ ◄── echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 \       │
│   │              │       ./target/debug/script-kit-gpui 2>&1                │
│   └──────┬───────┘                                                          │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐                                                          │
│   │  CHECK LOGS  │ ◄── grep -i "keyword" logs | tail -5                     │
│   │              │     Confirm the fix in log output                        │
│   └──────┬───────┘                                                          │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐                                                          │
│   │ VISUAL TEST  │ ◄── captureScreenshot() + READ the PNG                   │
│   │  (if UI)     │     Compare against expected state                       │
│   └──────┬───────┘                                                          │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐                                                          │
│   │  RUN TESTS   │ ◄── cargo test                                           │
│   │              │     All must pass                                        │
│   └──────────────┘                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Exploration

### Goal
Understand the problem completely before writing any code.

### Tools to Use

1. **Task tool with explore agent** - For broad codebase searches
   ```
   Task(subagent_type="explore", prompt="Find where NSVisualEffectView is configured...")
   ```

2. **Grep** - For specific pattern searches
   ```bash
   grep -r "setState" --include="*.rs" src/
   ```

3. **Read** - For reading specific files
   ```
   Read("/Users/.../src/platform.rs", offset=850, limit=100)
   ```

### What to Look For

- Where the relevant code lives
- How the feature currently works
- What triggers the problematic behavior
- Related code that might be affected

### Example Exploration Session

```
1. User reports: "Main window loses vibrancy when actions window opens"

2. Agent searches for vibrancy-related code:
   - Task(explore): "Find vibrancy, NSVisualEffectView, window focus handling"
   
3. Agent discovers:
   - platform.rs configures NSVisualEffectView with setState: 0
   - State 0 = "followsWindowActiveState" (dims when not key window)
   - Actions window opens with focus: true (becomes key window)
   
4. Root cause identified:
   - When actions takes focus, main window loses key status
   - NSVisualEffectView automatically dims due to state=0
```

---

## Phase 2: Implementation

### Goal
Make minimal, targeted changes that fix the root cause.

### Best Practices

1. **One logical change** - Don't fix multiple unrelated issues
2. **Preserve existing behavior** - Only change what's necessary
3. **Add comments** - Explain why the change was made
4. **Update documentation** - If behavior changes

### Example Fix

```rust
// Before: State 0 = followsWindowActiveState (dims when window loses key focus)
let _: () = msg_send![view, setState: 0isize];

// After: State 1 = active (always vibrant, doesn't dim)
// This prevents the main window from dimming when Actions popup opens
// NSVisualEffectState: 0=followsWindowActiveState, 1=active, 2=inactive
let _: () = msg_send![view, setState: 1isize];
```

---

## Phase 3: Verification

### Step 1: Build Check

```bash
cargo check && cargo clippy --all-targets -- -D warnings
```

Both must pass with no errors or warnings.

### Step 2: Launch the App

```bash
# Compact logs (saves tokens):
echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Full debug logs:
echo '{"type":"show"}' | RUST_LOG=debug ./target/debug/script-kit-gpui 2>&1

# Specific module:
echo '{"type":"show"}' | RUST_LOG=script_kit::platform=debug ./target/debug/script-kit-gpui 2>&1
```

### Step 3: Check Logs

```bash
# Live filtering during app run:
... | grep -iE 'vibrancy|visual|state'

# Check persisted logs:
grep -i "NSVisualEffectView config" ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -5

# Look for specific confirmation:
# "state 1 -> 1" means fix is applied (keeping active state)
# "state 1 -> 0" means old behavior (changing to followsWindowActiveState)
```

### Step 4: Visual Testing (if UI change)

```typescript
// tests/smoke/test-my-fix.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Set up the UI state to test
await div(`<div class="p-4 bg-blue-500">Test content</div>`);
await new Promise(r => setTimeout(r, 500));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `my-test-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);
process.exit(0);
```

Run it:
```bash
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-fix.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

**CRITICAL**: You must READ the PNG file to verify. Capturing is not enough.

### Step 5: Run Tests

```bash
cargo test
```

All tests must pass.

---

## Logging Reference

### Log Modes

| Environment Variable | Output | Use Case |
|---------------------|--------|----------|
| `SCRIPT_KIT_AI_LOG=1` | Compact format: `SS.mmm\|L\|C\|message` | Default for AI agents |
| `RUST_LOG=debug` | Full tracing output | Deep debugging |
| `RUST_LOG=module::path=debug` | Module-specific | Target investigation |

### Compact Log Format

Format: `SS.mmm|L|C|message`

- `SS.mmm` - Seconds.milliseconds since start
- `L` - Level: `i`=INFO, `w`=WARN, `e`=ERROR, `d`=DEBUG, `t`=TRACE
- `C` - Category code (see below)
- `message` - Log content

### Category Codes

| Code | Category | Description |
|------|----------|-------------|
| `P` | POSITION | Window positioning |
| `A` | APP | Application lifecycle |
| `U` | UI | User interface rendering |
| `S` | STDIN | Stdin protocol messages |
| `H` | HOTKEY | Global hotkey handling |
| `V` | VISIBILITY | Window show/hide |
| `E` | EXEC | Script execution |
| `K` | KEY | Keyboard events |
| `F` | FOCUS | Focus changes |
| `T` | THEME | Theme loading/changes |
| `C` | CACHE | Caching operations |
| `R` | PERF | Performance metrics |
| `W` | WINDOW_MGR | Window management |
| `X` | ERROR | Errors |
| `Z` | RESIZE | Window resize |

### Log File Location

```
~/.scriptkit/logs/script-kit-gpui.jsonl
```

---

## Visual Testing

### The captureScreenshot() SDK Function

This captures **only the app window**, not the full screen. This is intentional and required.

```typescript
const screenshot = await captureScreenshot();
// Returns: { data: string (base64), width: number, height: number }
```

### Complete Pattern

```typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

// 1. Set up UI state
await div(`<div class="...">Your test content</div>`);

// 2. Wait for render
await new Promise(r => setTimeout(r, 500));

// 3. Capture screenshot
const screenshot = await captureScreenshot();

// 4. Ensure directory exists
const dir = join(process.cwd(), 'test-screenshots');
if (!existsSync(dir)) {
  mkdirSync(dir, { recursive: true });
}

// 5. Save to file
const filename = `test-${Date.now()}.png`;
const filepath = join(dir, filename);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));

// 6. Log the path (so agent can find and read it)
console.error(`[SCREENSHOT] ${filepath}`);

// 7. Exit cleanly
process.exit(0);
```

### Blocked Tools

Do NOT use system screenshot tools:
- `screencapture` (macOS)
- `scrot`, `gnome-screenshot`, `flameshot`, `maim` (Linux)
- ImageMagick `import`

These capture the full screen, not just the app window.

---

## Real-World Example

### The Problem

User reported: "Main window loses vibrancy when actions window opens."

### Exploration Phase

1. **Used Task tool** with explore agent to search for:
   - Actions window creation
   - Vibrancy configuration
   - Window focus handling

2. **Discovered** in `src/platform.rs`:
   ```rust
   // NSVisualEffectView configured with state=0
   let _: () = msg_send![view, setState: 0isize];
   ```

3. **Understood the root cause**:
   - State 0 = `followsWindowActiveState`
   - When actions window opens with `focus: true`, it becomes the key window
   - Main window loses key status → NSVisualEffectView automatically dims

### Implementation Phase

Changed one line in `src/platform.rs`:

```rust
// Before:
let _: () = msg_send![view, setState: 0isize];

// After:
let _: () = msg_send![view, setState: 1isize];
```

Added explanatory comments.

### Verification Phase

1. **Build check**:
   ```bash
   cargo check && cargo clippy --all-targets -- -D warnings
   # Passed
   ```

2. **Launched app**:
   ```bash
   timeout 8 bash -c 'echo '\''{"type":"show"}'\'' | \
     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'
   ```

3. **Checked logs**:
   ```bash
   grep -i "NSVisualEffectView config" ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -3
   # Output showed: "state 1 -> 1" (fix confirmed!)
   ```

4. **Ran tests**:
   ```bash
   cargo test
   # All passed (14 passed, 50 ignored)
   ```

### Outcome

Fix verified without asking user to test. Logs confirmed the change took effect.

---

## Anti-Patterns to Avoid

### Verification Failures

| Anti-Pattern | Why It's Wrong |
|--------------|----------------|
| "The user should test this" | Agent is responsible for verification |
| "I made the change, it should work" | Must verify with logs/screenshots |
| "I can't test this automatically" | Use stdin protocol + logs |
| Capturing screenshot but not reading PNG | Must actually examine the image |

### Technical Failures

| Anti-Pattern | Correct Approach |
|--------------|------------------|
| Running scripts via CLI args | Use stdin JSON protocol |
| Skipping `cargo check` | Always build check first |
| Not using log environment variables | Always use `SCRIPT_KIT_AI_LOG=1` or `RUST_LOG=debug` |
| Guessing at fix without exploration | Understand root cause first |

### Process Failures

| Anti-Pattern | Correct Approach |
|--------------|------------------|
| Fixing multiple issues at once | One logical change per fix cycle |
| Not checking persisted logs | Log file survives app exit |
| Assuming tests will catch everything | Logs reveal runtime behavior |

---

## Success Checklist

Before declaring a fix complete, verify ALL of these:

### Exploration
- [ ] Used Task tool or grep to find relevant code
- [ ] Read the actual implementation
- [ ] Identified the root cause (not just symptoms)
- [ ] Understand why the bug occurs

### Implementation
- [ ] Made minimal, focused changes
- [ ] Added comments explaining the fix
- [ ] Did not introduce new issues

### Verification
- [ ] `cargo check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] Launched the app with `SCRIPT_KIT_AI_LOG=1` or `RUST_LOG=debug`
- [ ] Checked logs for confirmation of fix
- [ ] (If UI change) Captured AND READ screenshot
- [ ] `cargo test` passes

### Documentation
- [ ] Fix is self-explanatory from code/comments
- [ ] Any behavior changes are documented

---

## Quick Reference Commands

```bash
# Build check
cargo check && cargo clippy --all-targets -- -D warnings

# Run app with compact logs
echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Run app with full debug logs
echo '{"type":"show"}' | RUST_LOG=debug ./target/debug/script-kit-gpui 2>&1

# Run a test script
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-foo.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Check persisted logs
tail -50 ~/.scriptkit/logs/script-kit-gpui.jsonl | grep -i "keyword"

# Run test suite
cargo test

# Full verification gate (before commit)
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

---

## Conclusion

This workflow produces reliable fixes because:

1. **Exploration prevents guessing** - Understanding the root cause leads to targeted fixes
2. **Building catches errors early** - Compilation issues found before runtime
3. **Launching verifies behavior** - Actual app behavior, not assumed behavior
4. **Logs provide evidence** - Concrete proof the fix works
5. **Tests prevent regressions** - Ensure nothing else broke

Never skip steps. Never ask the user to verify. The agent owns the entire fix lifecycle.

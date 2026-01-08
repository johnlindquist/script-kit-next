# Task: Fix Missing Application Cursor Mode in Terminal

**Priority:** CRITICAL  
**Estimated Effort:** 45 minutes  
**Skill Reference:** `script-kit-terminal`

---

## Problem Description

The terminal emulator always sends normal mode arrow key escape sequences, but many terminal applications (vim, less, htop, fzf) switch to "application cursor mode" (DECCKM) which requires different escape sequences.

When in application cursor mode:
- Normal mode: `\x1b[A` (up), `\x1b[B` (down), `\x1b[C` (right), `\x1b[D` (left)
- Application mode: `\x1bOA` (up), `\x1bOB` (down), `\x1bOC` (right), `\x1bOD` (left)

Currently, arrow keys are broken in:
- `vim` / `nvim` (navigation in normal mode)
- `less` / `man` (scrolling)
- `htop` (navigation)
- `fzf` (selection)
- Any application using readline with cursor keys

---

## Affected Files

| File | Lines | Issue |
|------|-------|-------|
| `src/term_prompt.rs` | 777-780 | Always sends normal mode sequences |
| `src/terminal/alacritty.rs` | N/A | May need `is_application_cursor_mode()` method |

---

## Current Problematic Code

### Location: `src/term_prompt.rs:777-780`

```rust
// WRONG - always sends normal mode sequences
"up" | "arrowup" => Some(b"\x1b[A"),
"down" | "arrowdown" => Some(b"\x1b[B"),
"right" | "arrowright" => Some(b"\x1b[C"),
"left" | "arrowleft" => Some(b"\x1b[D"),
```

---

## Solution

### Step 1: Add Application Cursor Mode Check to TerminalHandle

First, check if `src/terminal/alacritty.rs` already has an `is_application_cursor_mode()` method. If not, add it:

```rust
// In src/terminal/alacritty.rs, add this method to TerminalHandle impl

/// Check if terminal is in application cursor mode (DECCKM)
pub fn is_application_cursor_mode(&self) -> bool {
    let state = self.state.lock();
    state.term.mode().contains(alacritty_terminal::term::TermMode::APP_CURSOR)
}
```

### Step 2: Update Arrow Key Handling in term_prompt.rs

Find the key handling section around line 777 in `src/term_prompt.rs`:

**Before:**
```rust
let escape_seq: Option<&[u8]> = match key_str.as_str() {
    "up" | "arrowup" => Some(b"\x1b[A"),
    "down" | "arrowdown" => Some(b"\x1b[B"),
    "right" | "arrowright" => Some(b"\x1b[C"),
    "left" | "arrowleft" => Some(b"\x1b[D"),
    // ... other keys
};
```

**After:**
```rust
// Check if terminal is in application cursor mode
let app_cursor = this.terminal.is_application_cursor_mode();

let escape_seq: Option<&[u8]> = match key_str.as_str() {
    "up" | "arrowup" => Some(if app_cursor { b"\x1bOA" } else { b"\x1b[A" }),
    "down" | "arrowdown" => Some(if app_cursor { b"\x1bOB" } else { b"\x1b[B" }),
    "right" | "arrowright" => Some(if app_cursor { b"\x1bOC" } else { b"\x1b[C" }),
    "left" | "arrowleft" => Some(if app_cursor { b"\x1bOD" } else { b"\x1b[D" }),
    // ... other keys remain unchanged
};
```

### Step 3: Handle Home/End Keys (Also Affected)

Home and End keys also have different sequences in application mode:

```rust
// Normal mode
"home" => Some(b"\x1b[H"),
"end" => Some(b"\x1b[F"),

// Application mode
"home" => Some(b"\x1bOH"),
"end" => Some(b"\x1bOF"),
```

Update these as well:

```rust
"home" => Some(if app_cursor { b"\x1bOH" } else { b"\x1b[H" }),
"end" => Some(if app_cursor { b"\x1bOF" } else { b"\x1b[F" }),
```

### Step 4: Consider Modifier Combinations (Optional Enhancement)

For a complete implementation, also handle Shift/Ctrl/Alt modifiers with arrow keys:

```rust
// Shift+Arrow (selection in editors)
// Ctrl+Arrow (word navigation)
// These use CSI sequences with modifiers: \x1b[1;{mod}A

// Modifier codes:
// 2 = Shift
// 3 = Alt
// 4 = Shift+Alt
// 5 = Ctrl
// 6 = Shift+Ctrl
// 7 = Alt+Ctrl
// 8 = Shift+Alt+Ctrl

// Example: Shift+Up = \x1b[1;2A
// Example: Ctrl+Up = \x1b[1;5A
```

This is optional but would improve editor support.

---

## Verification Steps

1. **Build Check:**
   ```bash
   cargo check
   ```

2. **Test:**
   ```bash
   cargo test
   ```

3. **Manual Testing - vim:**
   ```bash
   cargo build
   # Start Script Kit, open terminal prompt, run:
   vim test.txt
   # Press j, k, h, l - should work (vim's own bindings)
   # Press arrow keys - should NOW work for navigation
   # Press i to enter insert mode, arrow keys should work
   ```

4. **Manual Testing - less:**
   ```bash
   # In terminal prompt:
   less /etc/hosts
   # Arrow up/down should scroll
   # Page up/down should work
   # q to quit
   ```

5. **Manual Testing - htop:**
   ```bash
   # In terminal prompt:
   htop
   # Arrow keys should navigate processes
   # F10 or q to quit
   ```

6. **Manual Testing - fzf:**
   ```bash
   # In terminal prompt:
   ls | fzf
   # Arrow up/down should select items
   # Enter to select, Esc to cancel
   ```

---

## Success Criteria

- [ ] `is_application_cursor_mode()` method exists and works
- [ ] Arrow keys work in vim normal mode
- [ ] Arrow keys work in vim insert mode
- [ ] Arrow keys work in less/man
- [ ] Arrow keys work in htop
- [ ] Arrow keys work in fzf
- [ ] Home/End keys work in application mode
- [ ] `cargo check && cargo clippy && cargo test` passes

---

## Technical Background

### DECCKM (Application Cursor Keys Mode)

Applications send `\x1b[?1h` to enable and `\x1b[?1l` to disable application cursor mode. Alacritty's terminal emulator tracks this in `TermMode::APP_CURSOR`.

### Why This Matters

When vim starts, it sends escape sequences to:
1. Enter alternate screen buffer
2. Enable application cursor mode
3. Disable line wrapping (sometimes)

If we don't respect application cursor mode, vim receives `\x1b[A` when it expects `\x1bOA`, which it interprets as a different key sequence entirely.

---

## Related Files

- `src/terminal/alacritty.rs` - Terminal emulator wrapper
- `src/term_prompt.rs` - GPUI terminal component
- Alacritty source: `alacritty_terminal::term::TermMode`

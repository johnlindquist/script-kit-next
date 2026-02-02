# Research: Terminal Command Executor

## 1) TermPrompt owns a TerminalHandle
- `TermPrompt` is defined in `src/term_prompt.rs` and includes `pub terminal: TerminalHandle` as a field. (See `src/term_prompt.rs:67-71`.)
- The same file imports the handle from `crate::terminal` via `use crate::terminal::{..., TerminalHandle};` (See `src/term_prompt.rs:16-19`.)

## 2) No command bar module exists yet
- The terminal module currently only contains `alacritty.rs`, `pty.rs`, and `theme_adapter.rs` (directory listing: `src/terminal/`). There is no `src/terminal/command_bar.rs` file.
- `src/terminal/mod.rs` only declares `pub mod alacritty;`, `pub mod pty;`, and `pub mod theme_adapter;` (See `src/terminal/mod.rs:25-31`), confirming no command bar module is wired.

## 3) TerminalHandle exposes the required terminal actions
`TerminalHandle` in `src/terminal/alacritty.rs` provides the core operations needed by a command executor:
- `input(&mut self, bytes: &[u8])` for raw input/control sequences (See `src/terminal/alacritty.rs:592-608`).
- `scroll(&mut self, delta: i32)` (See `src/terminal/alacritty.rs:745-755`).
- `scroll_to_top(&mut self)` (See `src/terminal/alacritty.rs:775-781`).
- `scroll_to_bottom(&mut self)` (See `src/terminal/alacritty.rs:784-790`).
- `selection_to_string(&self)` (See `src/terminal/alacritty.rs:799-807`).
- `clear_selection(&mut self)` (See `src/terminal/alacritty.rs:809-813`).
- Additional helpers like `scroll_page_up/down()` and `display_offset()` are also present in the same block.

## 4) TermPrompt already uses TerminalHandle for input + selection
- `TermPrompt` uses `TerminalHandle` for scroll and selection operations (e.g., `scroll_to_top`, `scroll_to_bottom`, `selection_to_string`, `clear_selection`) in its key handlers (See `src/term_prompt.rs:680-720`).
- It sends control bytes via `terminal.input(&[0x03])` (Ctrl+C) and regular text input via `terminal.input(key_char.as_bytes())` (See `src/term_prompt.rs:723-779`).

## 5) Missing pieces for a command executor
- There is no `TerminalAction` enum or terminal command bar module; `src/terminal/command_bar.rs` must be created and referenced from `src/terminal/mod.rs`.
- `TermPrompt` does not currently expose an `execute_action` method (no `execute_action` symbols in `src/term_prompt.rs`), so a new method should be added to map `TerminalAction` -> `TerminalHandle` operations.

## 6) Clipboard operations rely on arboard
- The project already depends on `arboard = "3.6"` for clipboard access (See `Cargo.toml:62-65`).
- `TermPrompt` uses `arboard::Clipboard` for copy/paste handling (See `src/term_prompt.rs:699-759`).

## 7) Input method for control sequences
- The canonical method for sending control sequences to the terminal is `terminal.input(bytes)` on `TerminalHandle` (See `src/terminal/alacritty.rs:592-608`). This is already used in `TermPrompt` for Ctrl+C and paste operations (See `src/term_prompt.rs:723-779`).

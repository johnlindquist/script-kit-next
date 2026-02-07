//! Global hotkey registration and routing for launcher, notes, AI, and script shortcuts.
//! Key APIs include `HotkeyAction`, `update_hotkeys`, script hotkey registration helpers,
//! and dynamic shortcut registration/unregistration utilities.
//! This module depends on `config`, `scripts`, `shortcuts`, and `logging`, and is used by app startup/reload flows.

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
include!("part_004.rs");

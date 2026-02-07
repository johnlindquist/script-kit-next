#![allow(dead_code)]
//! File-watching services for config, theme, scripts, and app-level reload orchestration.
//! Public watchers include `ConfigWatcher`, `ThemeWatcher`, `ScriptWatcher`, and `AppWatcher`,
//! plus reload event enums consumed by the UI/application loop.
//! This module depends on `notify`, `config`, and `setup`, and feeds change events into runtime state updates.

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");
include!("part_004.rs");

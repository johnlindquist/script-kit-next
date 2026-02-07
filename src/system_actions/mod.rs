//! macOS System Actions Module
//!
//! Provides AppleScript-based system actions for macOS including:
//! - Power management (sleep, restart, shutdown, lock, log out)
//! - UI controls (dark mode, show desktop, mission control, launchpad)
//! - Media controls (volume)
//! - System utilities (empty trash, force quit, screen saver, do not disturb)
//! - System Preferences navigation
//! - Running application management (list GUI apps, force quit specific apps)
//!
//! All functions use `osascript` to execute AppleScript commands and return
//! `Result<(), String>` for consistent error handling.

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");

//! Window Manager Module for Script Kit GPUI
//!
//! # Problem
//! When GPUI creates windows and macOS creates tray icons, the app's windows array
//! contains multiple windows in unpredictable order. Using `objectAtIndex:0` to find
//! "our" window fails because:
//! - Tray icon popups appear as windows
//! - Menu bar items create windows
//! - System overlays create windows
//!
//! Debug logs showed:
//! - Window[0]: 34x24 - Tray icon popup
//! - Window[1]: 0x37 - Menu bar
//! - Window[2]: 0x24 - System window
//! - Window[3]: 750x501 - Our main window (the one we want!)
//!
//! # Solution
//! This module provides a thread-safe registry to track our windows by role.
//! After GPUI creates a window, we register it with its role (MainWindow, etc.)
//! and later retrieve it reliably, avoiding the index-based lookup problem.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    Window Manager Architecture                       │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │                                                                      │
//! │  ┌──────────────┐     ┌───────────────────────────────────────┐    │
//! │  │  main.rs     │     │     WindowManager (Global Singleton)  │    │
//! │  │              │     │                                        │    │
//! │  │ cx.open_window()   │  ┌─────────────────────────────────┐  │    │
//! │  │      │        │────▶│  │ OnceLock<Mutex<WindowManager>> │  │    │
//! │  │      ▼        │     │  │                                 │  │    │
//! │  │ register_     │     │  │ windows: HashMap<WindowRole,id> │  │    │
//! │  │   main_window │     │  │                                 │  │    │
//! │  │              │     │  │ • MainWindow -> id               │  │    │
//! │  └──────────────┘     │  │ • (future roles...)              │  │    │
//! │                       │  └─────────────────────────────────┘  │    │
//! │  ┌──────────────┐     │                                        │    │
//! │  │ window_      │     │  Public API:                          │    │
//! │  │ resize.rs    │────▶│  • register_window(role, id)          │    │
//! │  │              │     │  • get_window(role) -> Option<id>     │    │
//! │  │ get_main_    │◀────│  • get_main_window() -> Option<id>    │    │
//! │  │   window()   │     │  • find_main_window_by_size()         │    │
//! │  └──────────────┘     └───────────────────────────────────────┘    │
//! │                                                                      │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//!
//! # Thread Safety
//!
//! The module uses `OnceLock<Mutex<WindowManager>>` for thread-safe global access:
//! - `OnceLock` ensures one-time initialization (like lazy_static but in std)
//! - `Mutex` protects concurrent access to the HashMap
//! - All public functions handle locking internally
//!
//! # Platform Support
//!
//! This module is macOS-specific. On other platforms, all functions are no-ops
//! that return None or do nothing.

include!("part_000.rs");
include!("part_001.rs");

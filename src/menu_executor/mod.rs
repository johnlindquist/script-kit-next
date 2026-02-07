//! Menu Action Executor module using macOS Accessibility APIs
//!
//! This module provides functionality to execute menu bar actions on applications.
//! It navigates the AX hierarchy (App -> MenuBar -> MenuBarItem -> Menu -> MenuItem)
//! and performs the AXPress action on the target menu item.
//!
//! ## Architecture
//!
//! The execution flow:
//! 1. Verify the target app is frontmost (required for menu access)
//! 2. Navigate to the AXMenuBar of the application
//! 3. Find each menu item in the path by title
//! 4. Open intermediate menus (AXPress on MenuBarItems/MenuItems with submenus)
//! 5. Execute the final action (AXPress on the target MenuItem)
//!
//! ## Permissions
//!
//! Requires Accessibility permission in System Preferences > Privacy & Security > Accessibility
//!
//! ## Usage
//!
//! ```ignore
//! use script_kit_gpui::menu_executor::execute_menu_action;
//!
//! // Execute "File" -> "New Window" in Safari
//! execute_menu_action("com.apple.Safari", &["File", "New Window"])?;
//! ```

#![allow(non_upper_case_globals)]
#![allow(dead_code)]

include!("part_000.rs");
include!("part_001.rs");

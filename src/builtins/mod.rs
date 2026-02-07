//! Built-in Features Registry
//!
//! Provides a registry of built-in features that appear in the main search
//! alongside scripts. Features like Clipboard History and App Launcher are
//! configurable and can be enabled/disabled via config.
//!
//! ## Command Types
//!
//! The registry supports various command types organized by category:
//! - **System Actions**: Power management, UI controls, volume/brightness
//! - **Window Actions**: Window tiling and management for the frontmost window
//! - **Notes Commands**: Notes window operations
//! - **AI Commands**: AI chat window operations  
//! - **Script Commands**: Create new scripts and scriptlets
//! - **Permission Commands**: Accessibility permission management
//!

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
include!("part_003.rs");

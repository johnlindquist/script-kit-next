//! Script and Extension Creation Module
//!
//! This module provides functions to create new scripts and extensions
//! in the Script Kit environment, as well as opening files in the configured editor.
//!
//! # Usage
//!
//! ```rust,ignore
//! use script_kit_gpui::script_creation::{create_new_script, create_new_extension, open_in_editor};
//! use script_kit_gpui::config::Config;
//!
//! // Create a new script
//! let script_path = create_new_script("my-script")?;
//!
//! // Create a new extension
//! let extension_path = create_new_extension("my-extension")?;
//!
//! // Open in editor
//! let config = Config::default();
//! open_in_editor(&script_path, &config)?;
//! ```

include!("part_000.rs");
include!("part_001.rs");

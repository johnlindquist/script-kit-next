//! External command handling via stdin.
//!
//! This module provides the ability to control the Script Kit app via stdin JSONL commands.
//! This is primarily used for testing and automation.
//!
//! # Protocol
//!
//! Commands are sent as JSON objects, one per line (JSONL format):
//!
//! ```json
//! {"type": "run", "path": "/path/to/script.ts"}
//! {"type": "show"}
//! {"type": "hide"}
//! {"type": "setFilter", "text": "search term"}
//! {"type": "triggerBuiltin", "name": "clipboardHistory"}
//! {"type": "simulateKey", "key": "enter", "modifiers": ["cmd"]}
//! ```
//!
//! # Example Usage
//!
//! ```bash
//! # Run a script via stdin
//! echo '{"type": "run", "path": "/path/to/script.ts"}' | ./script-kit-gpui
//!
//! # Show/hide the window
//! echo '{"type": "show"}' | ./script-kit-gpui
//! echo '{"type": "hide"}' | ./script-kit-gpui
//!
//! # Filter the script list (for testing)
//! echo '{"type": "setFilter", "text": "hello"}' | ./script-kit-gpui
//! ```

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");

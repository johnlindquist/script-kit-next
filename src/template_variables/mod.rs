//! Centralized Template Variable Substitution Module
//!
//! Provides a consistent, well-tested system for variable substitution in templates
//! across the entire application. Used by:
//! - Text expansion/snippets (keyword_manager.rs)
//! - Template prompts (prompts/template.rs)
//! - Future template features
//!
//! # Variable Syntax
//!
//! Supports two interchangeable syntaxes:
//! - `${variable}` - Dollar-brace syntax (JavaScript/Shell style)
//! - `{{variable}}` - Double-brace syntax (Handlebars/Mustache style)
//!
//! # Built-in Variables
//!
//! | Variable | Description | Example Output |
//! |----------|-------------|----------------|
//! | `clipboard` | Current clipboard text | "copied text" |
//! | `date` | Current date (YYYY-MM-DD) | "2024-01-15" |
//! | `time` | Current time (HH:MM:SS) | "14:30:45" |
//! | `datetime` | Date and time | "2024-01-15 14:30:45" |
//! | `timestamp` | Unix timestamp (seconds) | "1705330245" |
//! | `date_short` | Short date (MM/DD/YYYY) | "01/15/2024" |
//! | `date_long` | Long date | "January 15, 2024" |
//! | `time_12h` | 12-hour time | "2:30 PM" |
//! | `day` | Day of week | "Monday" |
//! | `month` | Month name | "January" |
//! | `year` | Year | "2024" |
//!

include!("part_000.rs");
include!("part_001.rs");

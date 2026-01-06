//! Built-in fallback command definitions
//!
//! This module defines the default fallback commands that appear when no scripts
//! match the user's input. Fallbacks are Raycast-style actions like "Search Google",
//! "Open URL", "Calculate", etc.
//!
//! NOTE: Some items are currently unused as this is a new module being integrated.
#![allow(dead_code)]

use crate::scripts::input_detection::{is_file_path, is_math_expression, is_url, InputType};

/// Simple percent-encoding for URL query strings
/// Encodes characters that are not unreserved per RFC 3986
fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            // Unreserved characters: A-Z a-z 0-9 - . _ ~
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            // Space becomes + in query strings
            b' ' => encoded.push_str("%20"),
            // Everything else gets percent-encoded
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", byte));
            }
        }
    }
    encoded
}

/// Condition that determines when a fallback should be shown
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackCondition {
    /// Always show this fallback
    Always,
    /// Show when input is detected as a URL
    WhenUrl,
    /// Show when input is detected as a math expression
    WhenMath,
    /// Show when input is detected as a file path
    WhenFilePath,
    /// Show when input matches a specific type
    WhenInputType(InputType),
}

impl FallbackCondition {
    /// Check if this condition is met for the given input
    pub fn matches(&self, input: &str) -> bool {
        match self {
            FallbackCondition::Always => true,
            FallbackCondition::WhenUrl => is_url(input),
            FallbackCondition::WhenMath => is_math_expression(input),
            FallbackCondition::WhenFilePath => is_file_path(input),
            FallbackCondition::WhenInputType(expected) => {
                let detected = crate::scripts::input_detection::detect_input_type(input);
                &detected == expected
            }
        }
    }
}

/// Action type that determines how the fallback executes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackAction {
    /// Run input as a shell command in terminal
    RunInTerminal,
    /// Add input to Notes
    AddToNotes,
    /// Copy input to clipboard
    CopyToClipboard,
    /// Search using a URL template (input replaces {query})
    SearchUrl { template: String },
    /// Open input as URL directly
    OpenUrl,
    /// Calculate the math expression
    Calculate,
    /// Open file path in default application
    OpenFile,
    /// Search files using the input as query
    SearchFiles,
}

/// A built-in fallback command
#[derive(Debug, Clone)]
pub struct BuiltinFallback {
    /// Unique identifier for this fallback
    pub id: &'static str,
    /// Display name shown in the UI
    pub name: &'static str,
    /// Description shown below the name
    pub description: &'static str,
    /// Lucide icon name
    pub icon: &'static str,
    /// What happens when this fallback is executed
    pub action: FallbackAction,
    /// When to show this fallback
    pub condition: FallbackCondition,
    /// Whether this fallback is enabled by default
    pub enabled: bool,
    /// Sort priority (lower = higher in list)
    pub priority: u8,
}

impl BuiltinFallback {
    /// Create a new built-in fallback
    pub const fn new(
        id: &'static str,
        name: &'static str,
        description: &'static str,
        icon: &'static str,
        action: FallbackAction,
        condition: FallbackCondition,
        priority: u8,
    ) -> Self {
        Self {
            id,
            name,
            description,
            icon,
            action,
            condition,
            enabled: true,
            priority,
        }
    }

    /// Check if this fallback should be shown for the given input
    pub fn is_applicable(&self, input: &str) -> bool {
        self.enabled && self.condition.matches(input)
    }

    /// Execute this fallback with the given input
    ///
    /// Returns the result of the action or an error message
    pub fn execute(&self, input: &str) -> Result<FallbackResult, String> {
        match &self.action {
            FallbackAction::RunInTerminal => Ok(FallbackResult::RunTerminal {
                command: input.to_string(),
            }),

            FallbackAction::AddToNotes => Ok(FallbackResult::AddNote {
                content: input.to_string(),
            }),

            FallbackAction::CopyToClipboard => Ok(FallbackResult::Copy {
                text: input.to_string(),
            }),

            FallbackAction::SearchUrl { template } => {
                let encoded = percent_encode(input);
                let url = template.replace("{query}", &encoded);
                Ok(FallbackResult::OpenUrl { url })
            }

            FallbackAction::OpenUrl => {
                if is_url(input) {
                    Ok(FallbackResult::OpenUrl {
                        url: input.to_string(),
                    })
                } else {
                    Err("Input is not a valid URL".to_string())
                }
            }

            FallbackAction::Calculate => {
                if is_math_expression(input) {
                    // For now, return the expression - actual calculation done elsewhere
                    Ok(FallbackResult::Calculate {
                        expression: input.to_string(),
                    })
                } else {
                    Err("Input is not a valid math expression".to_string())
                }
            }

            FallbackAction::OpenFile => {
                if is_file_path(input) {
                    Ok(FallbackResult::OpenFile {
                        path: input.to_string(),
                    })
                } else {
                    Err("Input is not a valid file path".to_string())
                }
            }

            FallbackAction::SearchFiles => Ok(FallbackResult::SearchFiles {
                query: input.to_string(),
            }),
        }
    }

    /// Get the display subtitle with input preview
    ///
    /// Returns a subtitle like "Search Google for 'hello world'"
    pub fn get_subtitle(&self, input: &str) -> String {
        let truncated = if input.len() > 40 {
            format!("{}...", &input[..37])
        } else {
            input.to_string()
        };

        match &self.action {
            FallbackAction::RunInTerminal => format!("Run '{}'", truncated),
            FallbackAction::AddToNotes => format!("Add '{}'", truncated),
            FallbackAction::CopyToClipboard => format!("Copy '{}'", truncated),
            FallbackAction::SearchUrl { .. } => format!("Search for '{}'", truncated),
            FallbackAction::OpenUrl => format!("Open {}", truncated),
            FallbackAction::Calculate => format!("Calculate {}", truncated),
            FallbackAction::OpenFile => format!("Open {}", truncated),
            FallbackAction::SearchFiles => format!("Search files for '{}'", truncated),
        }
    }
}

/// Result of executing a fallback action
#[derive(Debug, Clone)]
pub enum FallbackResult {
    /// Run command in terminal
    RunTerminal { command: String },
    /// Add content to notes
    AddNote { content: String },
    /// Copy text to clipboard
    Copy { text: String },
    /// Open URL in browser
    OpenUrl { url: String },
    /// Calculate expression (result will be computed by caller)
    Calculate { expression: String },
    /// Open file in default application
    OpenFile { path: String },
    /// Search files with the given query
    SearchFiles { query: String },
}

/// Get all built-in fallback commands
///
/// Returns a vector of all default fallbacks in priority order
pub fn get_builtin_fallbacks() -> Vec<BuiltinFallback> {
    vec![
        // Search Files - high priority, always available
        BuiltinFallback {
            id: "search-files",
            name: "Search Files",
            description: "Search for files matching this query",
            icon: "folder-search",
            action: FallbackAction::SearchFiles,
            condition: FallbackCondition::Always,
            enabled: true,
            priority: 5,
        },
        // Conditional fallbacks first (more specific)
        BuiltinFallback {
            id: "open-url",
            name: "Open URL",
            description: "Open this URL in your default browser",
            icon: "external-link",
            action: FallbackAction::OpenUrl,
            condition: FallbackCondition::WhenUrl,
            enabled: true,
            priority: 10,
        },
        BuiltinFallback {
            id: "calculate",
            name: "Calculate",
            description: "Evaluate this mathematical expression",
            icon: "calculator",
            action: FallbackAction::Calculate,
            condition: FallbackCondition::WhenMath,
            enabled: true,
            priority: 11,
        },
        BuiltinFallback {
            id: "open-file",
            name: "Open File",
            description: "Open this file path in the default application",
            icon: "file",
            action: FallbackAction::OpenFile,
            condition: FallbackCondition::WhenFilePath,
            enabled: true,
            priority: 12,
        },
        // Always-available fallbacks
        BuiltinFallback {
            id: "run-in-terminal",
            name: "Run in Terminal",
            description: "Execute this command in a terminal",
            icon: "terminal",
            action: FallbackAction::RunInTerminal,
            condition: FallbackCondition::Always,
            enabled: true,
            priority: 20,
        },
        BuiltinFallback {
            id: "add-to-notes",
            name: "Add to Notes",
            description: "Save this text to your notes",
            icon: "sticky-note",
            action: FallbackAction::AddToNotes,
            condition: FallbackCondition::Always,
            enabled: true,
            priority: 21,
        },
        BuiltinFallback {
            id: "copy-to-clipboard",
            name: "Copy to Clipboard",
            description: "Copy this text to the clipboard",
            icon: "clipboard-copy",
            action: FallbackAction::CopyToClipboard,
            condition: FallbackCondition::Always,
            enabled: true,
            priority: 22,
        },
        BuiltinFallback {
            id: "search-google",
            name: "Search Google",
            description: "Search Google for this text",
            icon: "search",
            action: FallbackAction::SearchUrl {
                template: "https://www.google.com/search?q={query}".to_string(),
            },
            condition: FallbackCondition::Always,
            enabled: true,
            priority: 30,
        },
        BuiltinFallback {
            id: "search-duckduckgo",
            name: "Search DuckDuckGo",
            description: "Search DuckDuckGo for this text",
            icon: "search",
            action: FallbackAction::SearchUrl {
                template: "https://duckduckgo.com/?q={query}".to_string(),
            },
            condition: FallbackCondition::Always,
            enabled: true,
            priority: 31,
        },
    ]
}

/// Get fallbacks that are applicable for the given input
///
/// Filters the built-in fallbacks based on their conditions and the input type.
/// Results are sorted by priority (lower priority number = higher in list).
pub fn get_applicable_fallbacks(input: &str) -> Vec<BuiltinFallback> {
    let mut fallbacks: Vec<BuiltinFallback> = get_builtin_fallbacks()
        .into_iter()
        .filter(|f| f.is_applicable(input))
        .collect();

    // Sort by priority (lower = higher in list)
    fallbacks.sort_by_key(|f| f.priority);

    fallbacks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_fallbacks_count() {
        let fallbacks = get_builtin_fallbacks();
        assert_eq!(fallbacks.len(), 9, "Should have 9 built-in fallbacks");
    }

    #[test]
    fn test_fallback_condition_always() {
        let condition = FallbackCondition::Always;
        assert!(condition.matches("anything"));
        assert!(condition.matches(""));
        assert!(condition.matches("https://example.com"));
    }

    #[test]
    fn test_fallback_condition_when_url() {
        let condition = FallbackCondition::WhenUrl;
        assert!(condition.matches("https://example.com"));
        assert!(condition.matches("http://localhost:3000"));
        assert!(!condition.matches("not a url"));
        assert!(!condition.matches("example.com"));
    }

    #[test]
    fn test_fallback_condition_when_math() {
        let condition = FallbackCondition::WhenMath;
        assert!(condition.matches("2+2"));
        assert!(condition.matches("100 * 50"));
        assert!(condition.matches("(10 + 5) / 3"));
        assert!(!condition.matches("hello world"));
        assert!(!condition.matches("42")); // Just a number, not an expression
    }

    #[test]
    fn test_fallback_condition_when_file_path() {
        let condition = FallbackCondition::WhenFilePath;
        assert!(condition.matches("/path/to/file"));
        assert!(condition.matches("~/Documents/file.txt"));
        assert!(condition.matches("./relative/path"));
        assert!(!condition.matches("just some text"));
    }

    #[test]
    fn test_applicable_fallbacks_plain_text() {
        let fallbacks = get_applicable_fallbacks("hello world");

        // Should include all "Always" fallbacks
        let ids: Vec<&str> = fallbacks.iter().map(|f| f.id).collect();
        assert!(ids.contains(&"search-files"));
        assert!(ids.contains(&"run-in-terminal"));
        assert!(ids.contains(&"add-to-notes"));
        assert!(ids.contains(&"copy-to-clipboard"));
        assert!(ids.contains(&"search-google"));
        assert!(ids.contains(&"search-duckduckgo"));

        // Should NOT include conditional fallbacks
        assert!(!ids.contains(&"open-url"));
        assert!(!ids.contains(&"calculate"));
        assert!(!ids.contains(&"open-file"));
    }

    #[test]
    fn test_applicable_fallbacks_url() {
        let fallbacks = get_applicable_fallbacks("https://example.com");

        let ids: Vec<&str> = fallbacks.iter().map(|f| f.id).collect();

        // Should include open-url
        assert!(ids.contains(&"open-url"));

        // Should still include always fallbacks
        assert!(ids.contains(&"copy-to-clipboard"));
        assert!(ids.contains(&"search-google"));
    }

    #[test]
    fn test_applicable_fallbacks_math() {
        let fallbacks = get_applicable_fallbacks("2 + 2 * 3");

        let ids: Vec<&str> = fallbacks.iter().map(|f| f.id).collect();

        // Should include calculate
        assert!(ids.contains(&"calculate"));

        // Should still include always fallbacks
        assert!(ids.contains(&"copy-to-clipboard"));
    }

    #[test]
    fn test_applicable_fallbacks_file_path() {
        let fallbacks = get_applicable_fallbacks("/Users/john/Documents");

        let ids: Vec<&str> = fallbacks.iter().map(|f| f.id).collect();

        // Should include open-file
        assert!(ids.contains(&"open-file"));
    }

    #[test]
    fn test_fallback_priority_order() {
        let fallbacks = get_applicable_fallbacks("https://example.com/test");

        // open-url (priority 10) should come before run-in-terminal (priority 20)
        let url_pos = fallbacks.iter().position(|f| f.id == "open-url");
        let terminal_pos = fallbacks.iter().position(|f| f.id == "run-in-terminal");

        assert!(url_pos.is_some());
        assert!(terminal_pos.is_some());
        assert!(
            url_pos.unwrap() < terminal_pos.unwrap(),
            "open-url should come before run-in-terminal"
        );
    }

    #[test]
    fn test_execute_copy_to_clipboard() {
        let fallbacks = get_builtin_fallbacks();
        let copy = fallbacks
            .iter()
            .find(|f| f.id == "copy-to-clipboard")
            .unwrap();

        let result = copy.execute("test text").unwrap();
        match result {
            FallbackResult::Copy { text } => assert_eq!(text, "test text"),
            _ => panic!("Expected Copy result"),
        }
    }

    #[test]
    fn test_execute_search_google() {
        let fallbacks = get_builtin_fallbacks();
        let google = fallbacks.iter().find(|f| f.id == "search-google").unwrap();

        let result = google.execute("hello world").unwrap();
        match result {
            FallbackResult::OpenUrl { url } => {
                assert!(url.contains("google.com"));
                assert!(url.contains("hello%20world"));
            }
            _ => panic!("Expected OpenUrl result"),
        }
    }

    #[test]
    fn test_execute_open_url_valid() {
        let fallbacks = get_builtin_fallbacks();
        let open_url = fallbacks.iter().find(|f| f.id == "open-url").unwrap();

        let result = open_url.execute("https://example.com").unwrap();
        match result {
            FallbackResult::OpenUrl { url } => assert_eq!(url, "https://example.com"),
            _ => panic!("Expected OpenUrl result"),
        }
    }

    #[test]
    fn test_execute_open_url_invalid() {
        let fallbacks = get_builtin_fallbacks();
        let open_url = fallbacks.iter().find(|f| f.id == "open-url").unwrap();

        let result = open_url.execute("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_subtitle() {
        let fallbacks = get_builtin_fallbacks();
        let google = fallbacks.iter().find(|f| f.id == "search-google").unwrap();

        let subtitle = google.get_subtitle("hello");
        assert_eq!(subtitle, "Search for 'hello'");
    }

    #[test]
    fn test_get_subtitle_truncation() {
        let fallbacks = get_builtin_fallbacks();
        let copy = fallbacks
            .iter()
            .find(|f| f.id == "copy-to-clipboard")
            .unwrap();

        let long_input = "a".repeat(50);
        let subtitle = copy.get_subtitle(&long_input);
        assert!(subtitle.contains("..."));
        assert!(subtitle.len() < 60); // Should be truncated
    }

    #[test]
    fn test_search_files_fallback() {
        let fallbacks = get_builtin_fallbacks();
        let search_files = fallbacks.iter().find(|f| f.id == "search-files").unwrap();

        // Verify properties
        assert_eq!(search_files.name, "Search Files");
        assert_eq!(search_files.priority, 5);
        assert_eq!(search_files.condition, FallbackCondition::Always);
        assert!(search_files.enabled);

        // Test execute
        let result = search_files.execute("test query").unwrap();
        match result {
            FallbackResult::SearchFiles { query } => assert_eq!(query, "test query"),
            _ => panic!("Expected SearchFiles result"),
        }

        // Test subtitle
        let subtitle = search_files.get_subtitle("my file");
        assert_eq!(subtitle, "Search files for 'my file'");
    }

    #[test]
    fn test_search_files_priority() {
        let fallbacks = get_applicable_fallbacks("test");

        // search-files (priority 5) should come first among "Always" fallbacks
        let search_files_pos = fallbacks.iter().position(|f| f.id == "search-files");
        let run_terminal_pos = fallbacks.iter().position(|f| f.id == "run-in-terminal");

        assert!(search_files_pos.is_some());
        assert!(run_terminal_pos.is_some());
        assert!(
            search_files_pos.unwrap() < run_terminal_pos.unwrap(),
            "search-files should come before run-in-terminal"
        );
    }
}

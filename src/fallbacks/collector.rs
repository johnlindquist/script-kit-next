//! Fallback collector module
//!
//! Collects and filters all available fallbacks: built-in fallbacks and
//! user scripts with `fallback: true` in their typed metadata.
//!
//! NOTE: Some items are currently unused as this is a new module being integrated.
#![allow(dead_code)]

use std::sync::Arc;

use crate::fallbacks::builtins::{get_applicable_fallbacks, BuiltinFallback};
use crate::scripts::{FallbackConfig, Script};

/// Unified fallback item representing either a built-in fallback or a user script fallback
#[derive(Debug, Clone)]
pub enum FallbackItem {
    /// A built-in fallback command (Search Google, Copy to Clipboard, etc.)
    Builtin(BuiltinFallback),
    /// A user script with `fallback: true` in its metadata
    Script(FallbackConfig),
}

impl FallbackItem {
    /// Get the display name for this fallback
    pub fn name(&self) -> &str {
        match self {
            FallbackItem::Builtin(b) => b.name,
            FallbackItem::Script(s) => &s.script.name,
        }
    }

    /// Get the description for this fallback
    pub fn description(&self) -> &str {
        match self {
            FallbackItem::Builtin(b) => b.description,
            FallbackItem::Script(s) => s
                .script
                .description
                .as_deref()
                .unwrap_or("User script fallback"),
        }
    }

    /// Get the icon name for this fallback
    pub fn icon(&self) -> &str {
        match self {
            FallbackItem::Builtin(b) => b.icon,
            FallbackItem::Script(s) => s.script.icon.as_deref().unwrap_or("terminal"),
        }
    }

    /// Get the priority for sorting (lower = higher in list)
    pub fn priority(&self) -> u32 {
        match self {
            FallbackItem::Builtin(b) => b.priority as u32,
            // User script fallbacks have priority 50 (between conditional 10-12 and always 20-31)
            FallbackItem::Script(_) => 50,
        }
    }

    /// Get the display label for this fallback (with input substitution applied for script fallbacks)
    pub fn label(&self) -> &str {
        match self {
            FallbackItem::Builtin(b) => b.name,
            FallbackItem::Script(s) => &s.label,
        }
    }

    /// Check if this is a built-in fallback
    pub fn is_builtin(&self) -> bool {
        matches!(self, FallbackItem::Builtin(_))
    }

    /// Check if this is a script fallback
    pub fn is_script(&self) -> bool {
        matches!(self, FallbackItem::Script(_))
    }
}

/// Collect all applicable fallbacks for the given input
///
/// This function:
/// 1. Gets all applicable built-in fallbacks (filtered by input type)
/// 2. Gets all user scripts with `fallback: true` metadata
/// 3. Applies input substitution to script fallback labels
/// 4. Sorts by priority (lower = higher in list)
///
/// # Arguments
/// * `input` - The current user input text
/// * `scripts` - All available scripts to check for fallback metadata
///
/// # Returns
/// A sorted vector of `FallbackItem`s
pub fn collect_fallbacks(input: &str, scripts: &[Arc<Script>]) -> Vec<FallbackItem> {
    let mut fallbacks = Vec::new();

    // 1. Add applicable built-in fallbacks (already filtered by input type)
    for builtin in get_applicable_fallbacks(input) {
        fallbacks.push(FallbackItem::Builtin(builtin));
    }

    // 2. Add user scripts with fallback: true metadata
    for script in scripts {
        // Try to create a FallbackConfig from the script
        if let Some(config) = FallbackConfig::from_script(script.clone()) {
            // Apply input substitution to the label
            let config_with_input = config.with_input(input);
            fallbacks.push(FallbackItem::Script(config_with_input));
        }
    }

    // 3. Sort by priority (lower = higher in list)
    fallbacks.sort_by_key(|f| f.priority());

    fallbacks
}

/// Get only built-in fallbacks (no user scripts)
///
/// Useful when you want to show built-in fallbacks without user customizations.
pub fn collect_builtin_fallbacks(input: &str) -> Vec<FallbackItem> {
    get_applicable_fallbacks(input)
        .into_iter()
        .map(FallbackItem::Builtin)
        .collect()
}

/// Get only script fallbacks (no built-ins)
///
/// Useful when you want to process user script fallbacks separately.
pub fn collect_script_fallbacks(input: &str, scripts: &[Arc<Script>]) -> Vec<FallbackItem> {
    scripts
        .iter()
        .filter_map(|script| {
            FallbackConfig::from_script(script.clone())
                .map(|config| FallbackItem::Script(config.with_input(input)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_parser::TypedMetadata;
    use std::path::PathBuf;

    /// Helper to create a test script with fallback metadata
    fn make_fallback_script(name: &str, fallback_label: Option<&str>) -> Arc<Script> {
        Arc::new(Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/scripts/{}.ts", name)),
            extension: "ts".to_string(),
            description: Some(format!("{} description", name)),
            icon: Some("search".to_string()),
            alias: None,
            shortcut: None,
            typed_metadata: Some(TypedMetadata {
                fallback: true,
                fallback_label: fallback_label.map(|s| s.to_string()),
                ..Default::default()
            }),
            schema: None,
            kit_name: None,
        })
    }

    /// Helper to create a regular script (not a fallback)
    fn make_regular_script(name: &str) -> Arc<Script> {
        Arc::new(Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/scripts/{}.ts", name)),
            extension: "ts".to_string(),
            description: Some(format!("{} description", name)),
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: None,
            kit_name: None,
        })
    }

    #[test]
    fn test_collect_fallbacks_empty_scripts() {
        let fallbacks = collect_fallbacks("hello", &[]);

        // Should have only built-in fallbacks
        assert!(!fallbacks.is_empty());
        assert!(fallbacks.iter().all(|f| f.is_builtin()));
    }

    #[test]
    fn test_collect_fallbacks_with_script_fallbacks() {
        let scripts = vec![
            make_fallback_script("search-docs", Some("Search docs for {input}")),
            make_regular_script("regular-script"),
        ];

        let fallbacks = collect_fallbacks("test query", &scripts);

        // Should have built-ins plus the fallback script
        let script_fallbacks: Vec<_> = fallbacks.iter().filter(|f| f.is_script()).collect();
        assert_eq!(script_fallbacks.len(), 1);

        // Check that the label has {input} replaced
        let script_fb = script_fallbacks[0];
        assert_eq!(script_fb.label(), "Search docs for test query");
    }

    #[test]
    fn test_collect_fallbacks_excludes_regular_scripts() {
        let scripts = vec![
            make_regular_script("script1"),
            make_regular_script("script2"),
        ];

        let fallbacks = collect_fallbacks("hello", &scripts);

        // Should have only built-in fallbacks (no script fallbacks)
        assert!(fallbacks.iter().all(|f| f.is_builtin()));
    }

    #[test]
    fn test_collect_fallbacks_priority_sorting() {
        let scripts = vec![make_fallback_script(
            "custom-search",
            Some("Custom search for {input}"),
        )];

        let fallbacks = collect_fallbacks("test", &scripts);

        // Check that fallbacks are sorted by priority
        let priorities: Vec<u32> = fallbacks.iter().map(|f| f.priority()).collect();
        let mut sorted_priorities = priorities.clone();
        sorted_priorities.sort();
        assert_eq!(priorities, sorted_priorities);
    }

    #[test]
    fn test_fallback_item_name() {
        let builtin_fallbacks = collect_builtin_fallbacks("test");
        if let Some(first) = builtin_fallbacks.first() {
            assert!(!first.name().is_empty());
        }
    }

    #[test]
    fn test_fallback_item_description() {
        let builtin_fallbacks = collect_builtin_fallbacks("test");
        if let Some(first) = builtin_fallbacks.first() {
            assert!(!first.description().is_empty());
        }
    }

    #[test]
    fn test_fallback_item_icon() {
        let builtin_fallbacks = collect_builtin_fallbacks("test");
        if let Some(first) = builtin_fallbacks.first() {
            assert!(!first.icon().is_empty());
        }
    }

    #[test]
    fn test_collect_builtin_fallbacks_only() {
        let fallbacks = collect_builtin_fallbacks("hello");

        // All should be built-in
        assert!(fallbacks.iter().all(|f| f.is_builtin()));
        assert!(!fallbacks.is_empty());
    }

    #[test]
    fn test_collect_script_fallbacks_only() {
        let scripts = vec![
            make_fallback_script("search1", Some("Search 1 for {input}")),
            make_fallback_script("search2", Some("Search 2 for {input}")),
            make_regular_script("regular"),
        ];

        let fallbacks = collect_script_fallbacks("query", &scripts);

        // All should be script fallbacks
        assert!(fallbacks.iter().all(|f| f.is_script()));
        assert_eq!(fallbacks.len(), 2);
    }

    #[test]
    fn test_fallback_without_custom_label() {
        // When fallback_label is None, it should use "ScriptName {input}"
        let scripts = vec![make_fallback_script("My Search", None)];

        let fallbacks = collect_fallbacks("test input", &scripts);

        let script_fallback = fallbacks.iter().find(|f| f.is_script()).unwrap();
        // Default label format is "ScriptName {input}"
        assert_eq!(script_fallback.label(), "My Search test input");
    }

    #[test]
    fn test_url_input_includes_conditional_fallbacks() {
        let fallbacks = collect_builtin_fallbacks("https://example.com");

        // Should include "open-url" conditional fallback
        let has_open_url = fallbacks
            .iter()
            .any(|f| matches!(f, FallbackItem::Builtin(b) if b.id == "open-url"));
        assert!(has_open_url, "URL input should show 'open-url' fallback");
    }

    #[test]
    fn test_math_input_includes_calculate_fallback() {
        let fallbacks = collect_builtin_fallbacks("2 + 2 * 3");

        // Should include "calculate" conditional fallback
        let has_calculate = fallbacks
            .iter()
            .any(|f| matches!(f, FallbackItem::Builtin(b) if b.id == "calculate"));
        assert!(has_calculate, "Math input should show 'calculate' fallback");
    }

    #[test]
    fn test_file_path_input_includes_open_file_fallback() {
        let fallbacks = collect_builtin_fallbacks("/Users/test/Documents");

        // Should include "open-file" conditional fallback
        let has_open_file = fallbacks
            .iter()
            .any(|f| matches!(f, FallbackItem::Builtin(b) if b.id == "open-file"));
        assert!(
            has_open_file,
            "File path input should show 'open-file' fallback"
        );
    }
}

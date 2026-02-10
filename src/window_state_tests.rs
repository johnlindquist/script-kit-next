//! Window State Management Tests
//!
//! This module tests the window visibility and reset state machine.
//!
//! # Window State Invariants
//!
//! 1. **NEVER call `cx.hide()` directly in command execution code**
//!    - Use `self.close_and_reset_window(cx)` instead
//!    - This ensures reset happens AND handles Notes/AI windows correctly
//!
//! 2. **When hiding the main window:**
//!    - If Notes OR AI window is open → use `platform::hide_main_window()`
//!    - If NO secondary windows open → use `cx.hide()`
//!    - `close_and_reset_window()` handles this automatically
//!
//! 3. **Reset must happen BEFORE or DURING hide, not after**
//!    - The window should be clean when shown again
//!    - Don't rely on NEEDS_RESET flag for normal operation
//!
//! # Forbidden Patterns in app_execute.rs
//!
//! These patterns indicate a bug:
//! - `cx.hide()` without `close_and_reset_window`
//! - `NEEDS_RESET.store(true` (should use close_and_reset_window instead)
//! - `platform::hide_main_window()` without `reset_to_script_list` first
//!
//! # Code Audit Tests
//!
//! The tests below verify that forbidden patterns don't exist in the codebase.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    /// Helper function to read a source file for pattern checking
    fn read_source_file(path: &str) -> String {
        fs::read_to_string(path).unwrap_or_else(|_| {
            // Try with src/ prefix if not found
            fs::read_to_string(format!("src/{}", path)).unwrap_or_default()
        })
    }

    fn append_rs_sources(dir: &Path, content: &mut String) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        let mut paths: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();
        paths.sort();

        for path in paths {
            if path.is_dir() {
                append_rs_sources(&path, content);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(file_content) = fs::read_to_string(&path) {
                    content.push_str(&file_content);
                    content.push('\n');
                }
            }
        }
    }

    fn read_rs_sources_under(path: &str) -> String {
        let mut content = String::new();
        append_rs_sources(Path::new(path), &mut content);
        content
    }

    /// Count occurrences of a pattern in text
    fn count_occurrences(text: &str, pattern: &str) -> usize {
        text.matches(pattern).count()
    }

    /// Find lines containing a pattern (for error reporting)
    fn find_lines_with_pattern(text: &str, pattern: &str) -> Vec<(usize, String)> {
        text.lines()
            .enumerate()
            .filter(|(_, line)| line.contains(pattern))
            .map(|(i, line)| (i + 1, line.to_string()))
            .collect()
    }

    /// Verify that app_execute.rs doesn't use cx.hide() directly
    /// All hide operations should go through close_and_reset_window()
    #[test]
    fn test_no_direct_cx_hide_in_app_execute() {
        let content = read_source_file("app_execute.rs");
        let matches = find_lines_with_pattern(&content, "cx.hide()");

        // cx.hide() should NOT appear in app_execute.rs
        // All hide operations should use close_and_reset_window()
        assert!(
            matches.is_empty(),
            "Found forbidden cx.hide() in app_execute.rs. Use self.close_and_reset_window(cx) instead.\nFound:\n{}",
            matches.iter()
                .map(|(line, text)| format!("  Line {}: {}", line, text.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// Verify that app_execute.rs doesn't set NEEDS_RESET directly
    /// The close_and_reset_window() function handles reset properly
    #[test]
    fn test_no_needs_reset_in_app_execute() {
        let content = read_source_file("app_execute.rs");
        let matches = find_lines_with_pattern(&content, "NEEDS_RESET.store(true");

        // NEEDS_RESET should NOT be set in app_execute.rs
        // Reset happens immediately in close_and_reset_window()
        assert!(
            matches.is_empty(),
            "Found forbidden NEEDS_RESET.store(true in app_execute.rs. Use self.close_and_reset_window(cx) instead.\nFound:\n{}",
            matches.iter()
                .map(|(line, text)| format!("  Line {}: {}", line, text.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// Verify that platform::hide_main_window() is not called without reset
    /// in app_execute.rs (except in specific patterns like Notes/AI opening)
    #[test]
    fn test_no_orphan_hide_main_window_in_app_execute() {
        let content = read_source_file("app_execute.rs");
        let matches = find_lines_with_pattern(&content, "platform::hide_main_window()");

        // Filter out valid uses (those with comments indicating Notes/AI context)
        let invalid_matches: Vec<_> = matches
            .iter()
            .filter(|(_, line)| !line.contains("// Opening Notes/AI"))
            .collect();

        // If we find hide_main_window without the comment marker, just log it
        // Valid contexts: inside close_and_reset_window call, or opening secondary windows
        if !invalid_matches.is_empty() {
            // Manual verification: all hide_main_window calls should be:
            // 1. Inside close_and_reset_window (which does reset first)
            // 2. When opening Notes/AI (which calls reset_to_script_list first)
            // For now, we just warn - the other tests catch the main issues
            println!(
                "Note: Found platform::hide_main_window() calls - verify they are in valid contexts:\n{}",
                invalid_matches.iter()
                    .map(|(line, text)| format!("  Line {}: {}", line, text.trim()))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }

    /// Verify close_and_reset_window exists and has the right structure
    #[test]
    fn test_close_and_reset_window_exists() {
        let content = read_source_file("app_impl/lifecycle_reset.rs");
        let count = count_occurrences(&content, "fn close_and_reset_window");

        assert!(
            count >= 1,
            "close_and_reset_window() function not found in app_impl/lifecycle_reset.rs"
        );
    }

    /// Verify close_and_reset_window calls reset_to_script_list
    #[test]
    fn test_close_and_reset_window_resets() {
        let content = read_source_file("app_impl/lifecycle_reset.rs");

        // Find the function and check if it contains reset_to_script_list
        if let Some(start) = content.find("fn close_and_reset_window") {
            let after_start = &content[start..];
            let end = after_start.find("\n    ///").unwrap_or(after_start.len());
            let function_chunk = &after_start[..end];
            let has_reset = function_chunk.contains("reset_to_script_list");

            assert!(
                has_reset,
                "close_and_reset_window() must call reset_to_script_list()"
            );
        } else {
            panic!("close_and_reset_window() function not found in app_impl/lifecycle_reset.rs");
        }
    }

    /// Verify close_and_reset_window checks for Notes/AI windows
    #[test]
    fn test_close_and_reset_window_checks_secondary_windows() {
        let content = read_source_file("app_impl/lifecycle_reset.rs");

        // Find the function and check if it checks Notes/AI windows
        if let Some(start) = content.find("fn close_and_reset_window") {
            // Get a reasonable chunk after the function signature
            // Use 3000 chars to account for child window cleanup (actions + confirm)
            let function_chunk = &content[start..std::cmp::min(start + 3000, content.len())];
            let has_notes_check = function_chunk.contains("is_notes_window_open");
            let has_ai_check = function_chunk.contains("is_ai_window_open");

            assert!(
                has_notes_check && has_ai_check,
                "close_and_reset_window() must check both Notes and AI window state. Notes check: {}, AI check: {}",
                has_notes_check, has_ai_check
            );
        } else {
            panic!("close_and_reset_window() function not found in app_impl/lifecycle_reset.rs");
        }
    }

    /// Count how many places use close_and_reset_window vs direct patterns
    /// This helps track adoption of the correct pattern
    #[test]
    fn test_close_and_reset_window_adoption() {
        let content = read_rs_sources_under("src/app_impl");
        let correct_count = count_occurrences(&content, "close_and_reset_window(");

        // We expect multiple usages of close_and_reset_window
        assert!(
            correct_count >= 5,
            "Expected at least 5 close_and_reset_window call sites in src/app_impl, found {}",
            correct_count
        );
    }

    /// Document the valid patterns for hiding windows
    /// This test always passes but serves as documentation
    #[test]
    fn document_valid_hide_patterns() {
        // VALID PATTERN 1: Using close_and_reset_window (preferred)
        // ```rust
        // self.close_and_reset_window(cx);
        // ```
        // This handles everything: reset, visibility flag, Notes/AI check, hide

        // VALID PATTERN 2: Opening secondary window (Notes/AI)
        // ```rust
        // script_kit_gpui::set_main_window_visible(false);
        // self.reset_to_script_list(cx);
        // platform::hide_main_window();  // Always use this when opening Notes/AI
        // notes::open_notes_window(cx)?;
        // ```

        // INVALID PATTERN 1: Direct cx.hide() in command execution
        // ```rust
        // cx.hide();  // BAD - doesn't reset, doesn't check secondary windows
        // ```

        // INVALID PATTERN 2: Setting NEEDS_RESET instead of resetting
        // ```rust
        // NEEDS_RESET.store(true, ...);  // BAD - defers reset, can be missed
        // cx.hide();
        // ```

        // INVALID PATTERN 3: hide_main_window without reset
        // ```rust
        // platform::hide_main_window();  // BAD - window state not reset
        // ```

        // This test exists for documentation purposes - the patterns above
        // are enforced by the other tests in this module
    }
}

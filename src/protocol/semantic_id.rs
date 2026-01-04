//! Semantic ID generation for AI-driven UX targeting
//!
//! Provides functions to generate semantic IDs for UI elements that can be
//! used by AI agents to target specific elements in the interface.

/// Generate a semantic ID for an element.
///
/// Format: {type}:{index}:{value_slug}
///
/// # Arguments
/// * `element_type` - The element type (e.g., "choice", "button", "input")
/// * `index` - The numeric index of the element
/// * `value` - The value to convert to a slug
///
/// # Returns
/// A semantic ID string in the format: type:index:slug
pub fn generate_semantic_id(element_type: &str, index: usize, value: &str) -> String {
    let slug = value_to_slug(value);
    format!("{}:{}:{}", element_type, index, slug)
}

/// Generate a semantic ID for named elements (no index).
///
/// Format: {type}:{name}
///
/// # Arguments
/// * `element_type` - The element type (e.g., "input", "panel", "window")
/// * `name` - The name of the element
///
/// # Returns
/// A semantic ID string in the format: type:name
pub fn generate_semantic_id_named(element_type: &str, name: &str) -> String {
    let slug = value_to_slug(name);
    format!("{}:{}", element_type, slug)
}

/// Convert a value string to a URL-safe slug suitable for semantic IDs.
///
/// - Converts to lowercase
/// - Replaces spaces and underscores with hyphens
/// - **Restricts to ASCII alphanumeric only** (a-z, 0-9, hyphen)
/// - Non-ASCII characters (emoji, CJK, accented chars) become hyphens
/// - Collapses multiple hyphens to single
/// - Truncates to 20 **characters** (not bytes)
/// - Removes leading/trailing hyphens
///
/// # Why ASCII-only?
/// Semantic IDs are used in:
/// - CSS selectors (limited charset)
/// - Log files (needs to be safe across platforms)
/// - URL fragments (should be URL-safe)
/// - AI agent targeting (predictable format helps agents)
///
/// By restricting to ASCII, we ensure IDs are consistent and safe
/// across all contexts.
pub fn value_to_slug(value: &str) -> String {
    // Collapse multiple hyphens and trim, using char-based truncation
    let mut result = String::with_capacity(20);
    let mut prev_hyphen = false;
    let mut char_count = 0;

    for c in value.chars() {
        // Convert to lowercase and check if ASCII alphanumeric
        let lower = c.to_ascii_lowercase();
        let mapped = match lower {
            ' ' | '_' => '-',
            // ASCII alphanumeric only (a-z, 0-9)
            c if c.is_ascii_alphanumeric() => c,
            '-' => '-',
            // Non-ASCII (emoji, CJK, accented) -> hyphen
            _ => '-',
        };

        if mapped == '-' {
            if !prev_hyphen && !result.is_empty() {
                result.push('-');
                char_count += 1;
            }
            prev_hyphen = true;
        } else {
            result.push(mapped);
            char_count += 1;
            prev_hyphen = false;
        }

        // Truncate by character count, not bytes
        if char_count >= 20 {
            break;
        }
    }

    // Remove trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    // Ensure non-empty
    if result.is_empty() {
        result.push_str("item");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_slug_basic() {
        assert_eq!(value_to_slug("apple"), "apple");
        assert_eq!(value_to_slug("Apple"), "apple");
        assert_eq!(value_to_slug("APPLE"), "apple");
    }

    #[test]
    fn test_value_to_slug_spaces() {
        assert_eq!(value_to_slug("red apple"), "red-apple");
        assert_eq!(value_to_slug("red  apple"), "red-apple"); // multiple spaces
        assert_eq!(value_to_slug("  apple  "), "apple"); // leading/trailing spaces become hyphens then trimmed
    }

    #[test]
    fn test_value_to_slug_special_chars() {
        assert_eq!(value_to_slug("apple_pie"), "apple-pie");
        assert_eq!(value_to_slug("apple@pie!"), "apple-pie");
        assert_eq!(value_to_slug("hello-world"), "hello-world");
    }

    #[test]
    fn test_value_to_slug_truncation() {
        let long_value = "this is a very long value that exceeds twenty characters";
        let slug = value_to_slug(long_value);
        assert!(slug.len() <= 20);
        assert_eq!(slug, "this-is-a-very-long");
    }

    #[test]
    fn test_value_to_slug_empty() {
        assert_eq!(value_to_slug(""), "item");
        assert_eq!(value_to_slug("   "), "item");
        assert_eq!(value_to_slug("@#$%"), "item"); // all special chars
    }

    #[test]
    fn test_value_to_slug_non_ascii() {
        // Emoji become hyphens, then collapse
        assert_eq!(value_to_slug("🎉party🎉"), "party");

        // CJK characters become hyphens, then collapse
        assert_eq!(value_to_slug("文件"), "item"); // all non-ASCII -> empty -> "item"

        // Mixed ASCII and non-ASCII
        assert_eq!(value_to_slug("hello世界"), "hello");
        assert_eq!(value_to_slug("café"), "caf"); // 'é' is non-ASCII, becomes hyphen, then trimmed

        // Accented characters are replaced
        assert_eq!(value_to_slug("naïve"), "na-ve");
    }

    #[test]
    fn test_value_to_slug_truncation_by_chars_not_bytes() {
        // Even with multi-byte characters, truncation should be by character count
        // 20 characters max, but some chars are multi-byte
        let mixed = "a🎉b🎉c🎉d🎉e🎉f🎉g🎉h🎉i🎉j"; // 10 ASCII + 9 emoji
        let slug = value_to_slug(mixed);
        // Each emoji becomes a hyphen, so: a-b-c-d-e-f-g-h-i-j
        // That's 19 characters (10 letters + 9 hyphens), which fits in 20
        assert!(slug.chars().count() <= 20);
        // Result should be ASCII only
        assert!(slug.is_ascii());
    }

    #[test]
    fn test_generate_semantic_id() {
        assert_eq!(generate_semantic_id("choice", 0, "apple"), "choice:0:apple");
        assert_eq!(
            generate_semantic_id("choice", 5, "Red Apple"),
            "choice:5:red-apple"
        );
        assert_eq!(
            generate_semantic_id("button", 1, "Submit Form"),
            "button:1:submit-form"
        );
    }

    #[test]
    fn test_generate_semantic_id_named() {
        assert_eq!(
            generate_semantic_id_named("input", "filter"),
            "input:filter"
        );
        assert_eq!(
            generate_semantic_id_named("panel", "preview"),
            "panel:preview"
        );
        assert_eq!(
            generate_semantic_id_named("window", "Main Window"),
            "window:main-window"
        );
    }
}

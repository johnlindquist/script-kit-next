//! Shared utility functions for Script Kit GPUI
//!
//! This module is organized into submodules:
//! - `html`: HTML parsing and stripping
//! - `assets`: Asset path resolution
//! - `paths`: Path highlighting for search results
//! - `tailwind`: Tailwind CSS class mapping

mod applescript;
mod assets;
mod html;
mod paths;
mod tailwind;

// Re-export all public items for backwards compatibility
// Allow unused imports - these are public API exports for external use
pub use applescript::escape_applescript_string;
#[allow(unused_imports)]
pub use assets::{get_asset_path, get_logo_path};
#[allow(unused_imports)]
pub use html::{elements_to_text, parse_html, strip_html_tags, HtmlElement};
pub use paths::render_path_with_highlights;
pub use tailwind::{parse_color, TailwindStyles};

#[cfg(test)]
mod tests {
    // Integration tests that verify re-exports work
    use super::*;

    #[test]
    fn test_reexports_work() {
        // HTML
        assert_eq!(strip_html_tags("<p>test</p>"), "test");
        let elements = parse_html("<p>test</p>");
        assert!(!elements.is_empty());
        let text = elements_to_text(&elements);
        assert!(text.contains("test"));

        // Assets
        let path = get_asset_path("test.svg");
        assert!(path.contains("test.svg"));
        let logo = get_logo_path();
        assert!(logo.contains("logo.svg"));

        // Paths
        let highlights = render_path_with_highlights("path/to/file.txt", "file.txt", &[]);
        assert_eq!(highlights.len(), 1);

        // Tailwind
        let styles = TailwindStyles::parse("flex p-4");
        assert!(styles.flex);
        assert_eq!(styles.padding, Some(16.0));
        assert_eq!(parse_color("white"), Some(0xFFFFFF));
    }
}

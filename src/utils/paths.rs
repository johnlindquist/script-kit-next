//! Path highlighting utilities for search results

/// Render a path string with highlighted matched characters.
///
/// Takes the display path, the filename that was matched against, and the indices
/// of matched characters in the filename. Returns a vector of (text, is_highlighted)
/// tuples for rendering.
///
/// # Arguments
/// * `display_path` - The full path to display
/// * `filename` - The filename portion that was matched against
/// * `filename_indices` - Indices of matched characters in the filename
///
/// # Returns
/// A vector of (text, is_highlighted) tuples where highlighted segments
/// correspond to matched characters.
///
/// # Examples
///
/// ```
/// use script_kit_gpui::utils::render_path_with_highlights;
///
/// // No highlights - returns single unhighlighted segment
/// let result = render_path_with_highlights("path/to/file.txt", "file.txt", &[]);
/// assert_eq!(result, vec![("path/to/file.txt".to_string(), false)]);
///
/// // With highlights on 'f' and 'i' in filename
/// let result = render_path_with_highlights("path/to/file.txt", "file.txt", &[0, 1]);
/// // Path portion not highlighted, "fi" highlighted, "le.txt" not highlighted
/// assert!(result.len() >= 2);
/// ```
pub fn render_path_with_highlights(
    display_path: &str,
    filename: &str,
    filename_indices: &[usize],
) -> Vec<(String, bool)> {
    if filename_indices.is_empty() {
        return vec![(display_path.to_string(), false)];
    }

    // Find where the filename starts in the display path
    let filename_start = if let Some(pos) = display_path.rfind(filename) {
        pos
    } else if let Some(pos) = display_path.rfind('/') {
        pos + 1
    } else {
        0
    };

    let mut result = Vec::new();
    let chars: Vec<char> = display_path.chars().collect();
    let mut current_text = String::new();
    let mut current_highlighted = false;

    for (i, ch) in chars.iter().enumerate() {
        let is_in_filename = i >= filename_start;
        let filename_char_idx = if is_in_filename {
            i - filename_start
        } else {
            usize::MAX
        };
        let is_highlighted = is_in_filename && filename_indices.contains(&filename_char_idx);

        if is_highlighted != current_highlighted && !current_text.is_empty() {
            result.push((current_text.clone(), current_highlighted));
            current_text.clear();
        }

        current_text.push(*ch);
        current_highlighted = is_highlighted;
    }

    if !current_text.is_empty() {
        result.push((current_text, current_highlighted));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_path_no_highlights() {
        let result = render_path_with_highlights("path/to/file.txt", "file.txt", &[]);
        assert_eq!(result, vec![("path/to/file.txt".to_string(), false)]);
    }

    #[test]
    fn test_render_path_single_char_highlight() {
        // Highlight 'f' in filename (index 0)
        let result = render_path_with_highlights("path/to/file.txt", "file.txt", &[0]);

        // Should have: "path/to/" (not highlighted), "f" (highlighted), "ile.txt" (not highlighted)
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("path/to/".to_string(), false));
        assert_eq!(result[1], ("f".to_string(), true));
        assert_eq!(result[2], ("ile.txt".to_string(), false));
    }

    #[test]
    fn test_render_path_multiple_consecutive_highlights() {
        // Highlight "fil" in filename (indices 0, 1, 2)
        let result = render_path_with_highlights("path/to/file.txt", "file.txt", &[0, 1, 2]);

        // Should have: "path/to/" (not highlighted), "fil" (highlighted), "e.txt" (not highlighted)
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("path/to/".to_string(), false));
        assert_eq!(result[1], ("fil".to_string(), true));
        assert_eq!(result[2], ("e.txt".to_string(), false));
    }

    #[test]
    fn test_render_path_scattered_highlights() {
        // Highlight 'f' and 't' in "file.txt" (indices 0 and 5)
        let result = render_path_with_highlights("path/to/file.txt", "file.txt", &[0, 5]);

        // Should have: "path/to/" (no), "f" (yes), "ile." (no), "t" (yes), "xt" (no)
        assert_eq!(result.len(), 5);
        assert!(!result[0].1); // path/to/
        assert!(result[1].1); // f
        assert!(!result[2].1); // ile.
        assert!(result[3].1); // t
        assert!(!result[4].1); // xt
    }

    #[test]
    fn test_render_path_just_filename() {
        // When display_path IS the filename
        let result = render_path_with_highlights("file.txt", "file.txt", &[0, 1]);

        // "fi" (highlighted), "le.txt" (not highlighted)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("fi".to_string(), true));
        assert_eq!(result[1], ("le.txt".to_string(), false));
    }

    #[test]
    fn test_render_path_filename_not_found() {
        // When filename doesn't match (falls back to last '/' position)
        let result = render_path_with_highlights("path/to/other.txt", "file.txt", &[0]);

        // Should use filename_start at position after last '/'
        // Path is "path/to/other.txt", last '/' at position 7
        // So filename_start = 8, highlighting index 0 means 'o' in "other.txt"
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("path/to/".to_string(), false));
        assert_eq!(result[1], ("o".to_string(), true));
        assert_eq!(result[2], ("ther.txt".to_string(), false));
    }
}

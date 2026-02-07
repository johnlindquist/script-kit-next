/// Extract filename from a path for display
pub(crate) fn extract_filename(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

/// Extract display-friendly file path from scriptlet file_path
/// Converts "/path/to/file.md#slug" to "file.md#slug"
pub(crate) fn extract_scriptlet_display_path(file_path: &Option<String>) -> Option<String> {
    file_path.as_ref().map(|fp| {
        // Split on # to get path and anchor
        let parts: Vec<&str> = fp.splitn(2, '#').collect();
        let path_part = parts[0];
        let anchor = parts.get(1);

        // Extract just the filename from the path
        let filename = std::path::Path::new(path_part)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path_part);

        // Reconstruct with anchor if present
        match anchor {
            Some(a) => format!("{}#{}", filename, a),
            None => filename.to_string(),
        }
    })
}

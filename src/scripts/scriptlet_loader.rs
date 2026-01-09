//! Scriptlet loading and parsing
//!
//! This module provides functions for loading scriptlets from markdown files
//! in the ~/.scriptkit/kit/*/extensions/ directories.

use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, instrument, warn};

use glob::glob;

use crate::scriptlets as scriptlet_parser;
use crate::setup::get_kit_path;

use super::types::Scriptlet;

/// Extract metadata from HTML comments in scriptlet markdown
/// Looks for <!-- key: value --> patterns
pub(crate) fn extract_html_comment_metadata(
    text: &str,
) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut metadata = HashMap::new();

    // Find HTML comment blocks
    if let Some(start) = text.find("<!--") {
        if let Some(end) = text.find("-->") {
            if start < end {
                let comment_content = &text[start + 4..end];
                // Parse key: value pairs
                for line in comment_content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Handle format: "key: value"
                        if let Some(colon_pos) = trimmed.find(':') {
                            let key = trimmed[..colon_pos].trim().to_string();
                            let value = trimmed[colon_pos + 1..].trim().to_string();
                            metadata.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    metadata
}

/// Extract code block from markdown text
/// Looks for ```language ... ``` pattern and returns (language, code)
pub(crate) fn extract_code_block(text: &str) -> Option<(String, String)> {
    // Find first code fence
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        // Get the language specifier (rest of line)
        if let Some(newline_pos) = after_fence.find('\n') {
            let language = after_fence[..newline_pos].trim().to_string();
            let code_start = start + 3 + newline_pos + 1;

            // Find closing fence
            if let Some(end_pos) = text[code_start..].find("```") {
                let code = text[code_start..code_start + end_pos].trim().to_string();
                return Some((language, code));
            }
        }
    }
    None
}

/// Convert a name to a command slug (lowercase, spaces/special chars to hyphens)
pub(crate) fn slugify_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Parse a single scriptlet section from markdown
/// Input should be text from ## Name to the next ## or end of file
/// `source_path` is the path to the .md file containing the scriptlet
pub(crate) fn parse_scriptlet_section(
    section: &str,
    source_path: Option<&Path>,
) -> Option<Scriptlet> {
    let lines: Vec<&str> = section.lines().collect();
    if lines.is_empty() {
        return None;
    }

    // First line should be ## Name
    let first_line = lines[0];
    if !first_line.starts_with("##") {
        return None;
    }

    let name = first_line
        .strip_prefix("##")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if name.is_empty() {
        return None;
    }

    // Extract metadata from HTML comments
    let metadata = extract_html_comment_metadata(section);

    // Extract code block
    let (tool, code) = extract_code_block(section)?;

    // Generate command slug from name
    let command = slugify_name(&name);

    // Build file_path with anchor if source_path is provided
    let file_path = source_path.map(|p| format!("{}#{}", p.display(), command));

    Some(Scriptlet {
        name,
        description: metadata.get("description").cloned(),
        code,
        tool,
        shortcut: metadata.get("shortcut").cloned(),
        keyword: metadata.get("keyword").cloned(),
        group: None,
        file_path,
        command: Some(command),
        alias: metadata.get("alias").cloned(),
    })
}

/// Reads scriptlets from all *.md files in ~/.scriptkit/kit/*/extensions/
/// Returns a sorted list of Arc-wrapped Scriptlet structs parsed from markdown
/// Returns empty vec if directory doesn't exist or is inaccessible
///
/// H1 Optimization: Returns Arc<Scriptlet> to avoid expensive clones during filter operations.
#[instrument(level = "debug", skip_all)]
pub fn read_scriptlets() -> Vec<Arc<Scriptlet>> {
    let kit_path = get_kit_path();

    // Default to main kit (under kit/ subdirectory)
    let extensions_dir = kit_path.join("kit").join("main").join("extensions");

    // Check if directory exists
    if !extensions_dir.exists() {
        debug!(path = %extensions_dir.display(), "Extensions directory does not exist");
        return vec![];
    }

    let mut scriptlets = Vec::new();

    // Read all .md files in the scriptlets directory
    match fs::read_dir(&extensions_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();

                // Only process .md files
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }

                // Skip if not a file
                if !path.is_file() {
                    continue;
                }

                debug!(path = %path.display(), "Reading scriptlets file");

                match fs::read_to_string(&path) {
                    Ok(content) => {
                        // Split by ## headings
                        let mut current_section = String::new();
                        for line in content.lines() {
                            if line.starts_with("##") && !current_section.is_empty() {
                                // Parse previous section
                                if let Some(scriptlet) =
                                    parse_scriptlet_section(&current_section, Some(&path))
                                {
                                    scriptlets.push(Arc::new(scriptlet));
                                }
                                current_section = line.to_string();
                            } else {
                                current_section.push('\n');
                                current_section.push_str(line);
                            }
                        }

                        // Parse the last section
                        if !current_section.is_empty() {
                            if let Some(scriptlet) =
                                parse_scriptlet_section(&current_section, Some(&path))
                            {
                                scriptlets.push(Arc::new(scriptlet));
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            error = %e,
                            path = %path.display(),
                            "Failed to read scriptlets file"
                        );
                    }
                }
            }
        }
        Err(e) => {
            warn!(
                error = %e,
                path = %extensions_dir.display(),
                "Failed to read scriptlets directory"
            );
            return vec![];
        }
    }

    // Sort by name
    scriptlets.sort_by(|a, b| a.name.cmp(&b.name));

    debug!(
        count = scriptlets.len(),
        "Loaded scriptlets from all .md files"
    );
    scriptlets
}

/// Load scriptlets from markdown files using the comprehensive parser
///
/// Globs:
/// - ~/.scriptkit/kit/*/extensions/*.md (all kits)
///
/// Uses `crate::scriptlets::parse_markdown_as_scriptlets` for parsing.
/// Returns Arc-wrapped scriptlets sorted by group then by name.
///
/// H1 Optimization: Returns Arc<Scriptlet> to avoid expensive clones during filter operations.
#[instrument(level = "debug", skip_all)]
pub fn load_scriptlets() -> Vec<Arc<Scriptlet>> {
    let kit_path = get_kit_path();

    let mut scriptlets = Vec::new();

    // Glob pattern to search all kits (under kit/ subdirectory)
    let patterns = [kit_path.join("kit/*/extensions/*.md")];

    for pattern in patterns {
        let pattern_str = pattern.to_string_lossy().to_string();
        debug!(pattern = %pattern_str, "Globbing for scriptlet files");

        match glob(&pattern_str) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            debug!(path = %path.display(), "Parsing scriptlet file");

                            // Determine kit from path
                            let kit = extract_kit_from_path(&path, &kit_path);

                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    let path_str = path.to_string_lossy().to_string();
                                    let parsed = scriptlet_parser::parse_markdown_as_scriptlets(
                                        &content,
                                        Some(&path_str),
                                    );

                                    // Convert parsed scriptlets to our Scriptlet format
                                    for parsed_scriptlet in parsed {
                                        let file_path = build_scriptlet_file_path(
                                            &path,
                                            &parsed_scriptlet.command,
                                        );

                                        scriptlets.push(Arc::new(Scriptlet {
                                            name: parsed_scriptlet.name,
                                            description: parsed_scriptlet.metadata.description,
                                            code: parsed_scriptlet.scriptlet_content,
                                            tool: parsed_scriptlet.tool,
                                            shortcut: parsed_scriptlet.metadata.shortcut,
                                            keyword: parsed_scriptlet
                                                .typed_metadata
                                                .as_ref()
                                                .and_then(|t| t.keyword.clone())
                                                .or(parsed_scriptlet.metadata.keyword.clone()),
                                            group: if parsed_scriptlet.group.is_empty() {
                                                kit.clone()
                                            } else {
                                                Some(parsed_scriptlet.group)
                                            },
                                            file_path: Some(file_path),
                                            command: Some(parsed_scriptlet.command),
                                            alias: parsed_scriptlet.metadata.alias,
                                        }));
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        error = %e,
                                        path = %path.display(),
                                        "Failed to read scriptlet file"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to process glob entry");
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    pattern = %pattern_str,
                    "Failed to glob scriptlet files"
                );
            }
        }
    }

    // Sort by group first (None last), then by name
    scriptlets.sort_by(|a, b| match (&a.group, &b.group) {
        (Some(g1), Some(g2)) => match g1.cmp(g2) {
            Ordering::Equal => a.name.cmp(&b.name),
            other => other,
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.name.cmp(&b.name),
    });

    debug!(count = scriptlets.len(), "Loaded scriptlets via parser");
    scriptlets
}

/// Extract kit name from a kit path
/// e.g., ~/.scriptkit/kit/my-kit/extensions/file.md -> Some("my-kit")
pub(crate) fn extract_kit_from_path(path: &Path, kit_root: &Path) -> Option<String> {
    let kit_prefix = format!("{}/", kit_root.to_string_lossy());
    let path_str = path.to_string_lossy().to_string();

    if path_str.starts_with(&kit_prefix) {
        // Extract the kit name from the path
        // Path structure is: kit/<kit-name>/extensions/...
        let relative = &path_str[kit_prefix.len()..];
        let parts: Vec<&str> = relative.split('/').collect();

        // Skip "kit" directory and get the actual kit name
        if parts.len() >= 2 && parts[0] == "kit" {
            return Some(parts[1].to_string());
        } else if !parts.is_empty() {
            // Fallback for old structure
            return Some(parts[0].to_string());
        }
    }
    None
}

/// Build the file path with anchor for scriptlet execution
/// Format: /path/to/file.md#slug
pub(crate) fn build_scriptlet_file_path(md_path: &Path, command: &str) -> String {
    format!("{}#{}", md_path.display(), command)
}

/// Read scriptlets from a single markdown file
///
/// This function parses a single .md file and returns all scriptlets found in it.
/// Used for incremental updates when a scriptlet file changes.
///
/// H1 Optimization: Returns Arc<Scriptlet> to avoid expensive clones during filter operations.
///
/// # Arguments
/// * `path` - Path to the markdown file
///
/// # Returns
/// Vector of Arc-wrapped Scriptlet structs parsed from the file, or empty vec on error
#[instrument(level = "debug", skip_all, fields(path = %path.display()))]
pub fn read_scriptlets_from_file(path: &Path) -> Vec<Arc<Scriptlet>> {
    // Verify it's a markdown file
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
        debug!(path = %path.display(), "Not a markdown file, skipping");
        return vec![];
    }

    // Get kit path for kit extraction
    let kit_path = get_kit_path();

    // Read file content
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!(
                error = %e,
                path = %path.display(),
                "Failed to read scriptlet file"
            );
            return vec![];
        }
    };

    let path_str = path.to_string_lossy().to_string();
    let parsed = scriptlet_parser::parse_markdown_as_scriptlets(&content, Some(&path_str));

    // Determine kit from path
    let kit = extract_kit_from_path(path, &kit_path);

    // Convert parsed scriptlets to our Arc-wrapped Scriptlet format
    let scriptlets: Vec<Arc<Scriptlet>> = parsed
        .into_iter()
        .map(|parsed_scriptlet| {
            let file_path = build_scriptlet_file_path(path, &parsed_scriptlet.command);

            Arc::new(Scriptlet {
                name: parsed_scriptlet.name,
                description: parsed_scriptlet.metadata.description,
                code: parsed_scriptlet.scriptlet_content,
                tool: parsed_scriptlet.tool,
                shortcut: parsed_scriptlet.metadata.shortcut,
                keyword: parsed_scriptlet
                    .typed_metadata
                    .as_ref()
                    .and_then(|t| t.keyword.clone())
                    .or(parsed_scriptlet.metadata.keyword.clone()),
                group: if parsed_scriptlet.group.is_empty() {
                    kit.clone()
                } else {
                    Some(parsed_scriptlet.group)
                },
                file_path: Some(file_path),
                command: Some(parsed_scriptlet.command),
                alias: parsed_scriptlet.metadata.alias,
            })
        })
        .collect();

    debug!(
        count = scriptlets.len(),
        path = %path.display(),
        "Parsed scriptlets from file"
    );

    scriptlets
}

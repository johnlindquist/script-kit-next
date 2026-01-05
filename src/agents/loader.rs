//! Agent file loading
//!
//! This module handles loading agent files from the filesystem:
//! - Glob `~/.scriptkit/*/agents/*.md` directories
//! - Parse each file using the parser module
//! - Return Arc-wrapped agents for cheap cloning

// These functions are public API for future integration - allow them to be unused for now
#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;

use glob::glob;
use tracing::{debug, warn};

use crate::agents::parser::parse_agent;
use crate::agents::types::Agent;
use crate::setup::get_kit_path;

/// Load agents from all kits
///
/// Globs: `~/.scriptkit/*/agents/*.md`
///
/// Returns Arc-wrapped agents sorted by name.
///
/// # Example
///
/// ```no_run
/// use script_kit_gpui::agents::load_agents;
///
/// let agents = load_agents();
/// for agent in &agents {
///     println!("{}: {:?}", agent.name, agent.backend);
/// }
/// ```
pub fn load_agents() -> Vec<Arc<Agent>> {
    let kit_path = get_kit_path();
    load_agents_from_path(&kit_path)
}

/// Load agents from a specific kit root path
///
/// This is the internal implementation that can be tested with a custom path.
pub fn load_agents_from_path(kit_path: &Path) -> Vec<Arc<Agent>> {
    let mut agents = Vec::new();

    let pattern = kit_path.join("kit/*/agents/*.md");
    let pattern_str = pattern.to_string_lossy().to_string();

    debug!(pattern = %pattern_str, "Globbing for agent files");

    match glob(&pattern_str) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(path) => {
                        debug!(path = %path.display(), "Parsing agent file");

                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                if let Some(mut agent) = parse_agent(&path, &content) {
                                    // Extract kit from path
                                    agent.kit = extract_kit_from_path(&path, kit_path);
                                    agents.push(Arc::new(agent));
                                }
                            }
                            Err(e) => {
                                warn!(
                                    error = %e,
                                    path = %path.display(),
                                    "Failed to read agent file"
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
                "Failed to glob agent files"
            );
        }
    }

    // Sort by name
    agents.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    debug!(count = agents.len(), "Loaded agents");
    agents
}

/// Extract kit name from path
///
/// Given a path like `/Users/x/.scriptkit/kit/main/agents/task.claude.md`
/// and kit root `/Users/x/.scriptkit`, returns `Some("main")`.
fn extract_kit_from_path(path: &Path, kit_root: &Path) -> Option<String> {
    // Get the relative path from kit root
    let relative = path.strip_prefix(kit_root).ok()?;

    // Path structure is: kit/<kit-name>/agents/*.md
    // Skip first component ("kit") and get the second (kit name)
    let mut components = relative.components();
    let first = components.next()?.as_os_str().to_str()?;

    if first == "kit" {
        // New structure: kit/<kit-name>/...
        components.next()?.as_os_str().to_str().map(|s| s.to_string())
    } else {
        // Fallback for old structure (shouldn't happen but be safe)
        Some(first.to_string())
    }
}

/// Load a single agent from a file path
///
/// Useful for reloading a specific agent after file change.
pub fn load_agent_from_path(path: &Path) -> Option<Arc<Agent>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut agent = parse_agent(path, &content)?;

    // Try to extract kit from path (best effort)
    if let Some(kit_path) = find_kit_root(path) {
        agent.kit = extract_kit_from_path(path, &kit_path);
    }

    Some(Arc::new(agent))
}

/// Find the kit root path from an agent file path
///
/// Walks up the directory tree looking for a directory that contains
/// an "agents" subdirectory containing our file.
fn find_kit_root(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = path.parent()?; // Start at agents/ directory

    // If we're in an agents directory, go up one more
    if current.file_name().and_then(|n| n.to_str()) == Some("agents") {
        current = current.parent()?; // Now at kit name directory (e.g., "main")
        current = current.parent()?; // Now at kit root
    }

    Some(current.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create a test directory structure with agent files
    fn setup_test_agents(temp_dir: &TempDir) -> std::path::PathBuf {
        let kit_root = temp_dir.path().join("scriptkit");

        // Create kit/main/agents directory (new structure)
        let main_agents = kit_root.join("kit/main/agents");
        fs::create_dir_all(&main_agents).unwrap();

        // Create a Claude agent
        fs::write(
            main_agents.join("review.claude.md"),
            r#"---
_sk_name: "Review PR"
_sk_description: "Reviews pull requests"
model: sonnet
---
Please review the following:
"#,
        )
        .unwrap();

        // Create an interactive Gemini agent
        fs::write(
            main_agents.join("chat.i.gemini.md"),
            r#"---
_sk_name: "Chat"
model: gemini-2.0-flash
---
Interactive chat session
"#,
        )
        .unwrap();

        // Create kit/custom/agents directory (new structure)
        let custom_agents = kit_root.join("kit/custom/agents");
        fs::create_dir_all(&custom_agents).unwrap();

        // Create a Codex agent in custom kit
        fs::write(
            custom_agents.join("code.codex.md"),
            r#"---
_sk_name: "Generate Code"
---
Generate code for:
"#,
        )
        .unwrap();

        kit_root
    }

    #[test]
    fn test_load_agents_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = setup_test_agents(&temp_dir);

        let agents = load_agents_from_path(&kit_root);

        assert_eq!(agents.len(), 3);

        // Should be sorted by name (case-insensitive)
        let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["Chat", "Generate Code", "Review PR"]);
    }

    #[test]
    fn test_agent_kit_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = setup_test_agents(&temp_dir);

        let agents = load_agents_from_path(&kit_root);

        // Find the Review PR agent (should be in "main" kit)
        let review = agents.iter().find(|a| a.name == "Review PR").unwrap();
        assert_eq!(review.kit, Some("main".to_string()));

        // Find the Generate Code agent (should be in "custom" kit)
        let code = agents.iter().find(|a| a.name == "Generate Code").unwrap();
        assert_eq!(code.kit, Some("custom".to_string()));
    }

    #[test]
    fn test_agent_metadata_loaded() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = setup_test_agents(&temp_dir);

        let agents = load_agents_from_path(&kit_root);

        // Find the Review PR agent
        let review = agents.iter().find(|a| a.name == "Review PR").unwrap();
        assert_eq!(
            review.description,
            Some("Reviews pull requests".to_string())
        );
        assert_eq!(review.backend, crate::agents::AgentBackend::Claude);
        assert!(!review.interactive);
    }

    #[test]
    fn test_interactive_agent_detected() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = setup_test_agents(&temp_dir);

        let agents = load_agents_from_path(&kit_root);

        // Find the Chat agent (interactive)
        let chat = agents.iter().find(|a| a.name == "Chat").unwrap();
        assert!(chat.interactive);
        assert_eq!(chat.backend, crate::agents::AgentBackend::Gemini);
    }

    #[test]
    fn test_load_single_agent() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = setup_test_agents(&temp_dir);

        let path = kit_root.join("kit/main/agents/review.claude.md");
        let agent = load_agent_from_path(&path).unwrap();

        assert_eq!(agent.name, "Review PR");
        assert_eq!(agent.backend, crate::agents::AgentBackend::Claude);
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().join("scriptkit");

        // Create empty agents directory (new structure)
        fs::create_dir_all(kit_root.join("kit/main/agents")).unwrap();

        let agents = load_agents_from_path(&kit_root);
        assert!(agents.is_empty());
    }

    #[test]
    fn test_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().join("nonexistent");

        let agents = load_agents_from_path(&kit_root);
        assert!(agents.is_empty());
    }

    #[test]
    fn test_extract_kit_from_path() {
        let kit_root = Path::new("/Users/x/.scriptkit");
        let agent_path = Path::new("/Users/x/.scriptkit/kit/main/agents/task.claude.md");

        let kit = extract_kit_from_path(agent_path, kit_root);
        // Should extract the kit name ("main"), not the container ("kit")
        assert_eq!(kit, Some("main".to_string()));
    }

    #[test]
    fn test_extract_kit_from_nested_path() {
        let kit_root = Path::new("/Users/x/.scriptkit");
        let agent_path = Path::new("/Users/x/.scriptkit/kit/my-custom-kit/agents/task.claude.md");

        let kit = extract_kit_from_path(agent_path, kit_root);
        // Should extract the kit name ("my-custom-kit")
        assert_eq!(kit, Some("my-custom-kit".to_string()));
    }

    #[test]
    fn test_hidden_files_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().join("scriptkit");
        let main_agents = kit_root.join("kit/main/agents");
        fs::create_dir_all(&main_agents).unwrap();

        // Create a hidden file (should be skipped)
        fs::write(main_agents.join(".hidden.claude.md"), "---\n---\nHidden").unwrap();

        // Create a normal file
        fs::write(
            main_agents.join("visible.claude.md"),
            "---\n_sk_name: Visible\n---\nVisible",
        )
        .unwrap();

        let agents = load_agents_from_path(&kit_root);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "Visible");
    }

    #[test]
    fn test_non_md_files_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().join("scriptkit");
        let main_agents = kit_root.join("kit/main/agents");
        fs::create_dir_all(&main_agents).unwrap();

        // Create a .ts file (should be skipped by glob)
        fs::write(main_agents.join("script.ts"), "export default {}").unwrap();

        // Create a .md file
        fs::write(
            main_agents.join("task.claude.md"),
            "---\n_sk_name: Task\n---\nTask",
        )
        .unwrap();

        let agents = load_agents_from_path(&kit_root);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "Task");
    }
}

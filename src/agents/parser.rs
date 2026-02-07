//! Agent file parsing
//!
//! This module handles parsing mdflow agent files:
//! - YAML frontmatter extraction (preserved raw for mdflow)
//! - Script Kit metadata extraction (`_sk_*` keys)
//! - mdflow system key extraction (`_inputs`, `_interactive`, etc.)
//! - Shell inline and remote import detection
//!
//! # Design Principle
//!
//! We preserve the raw frontmatter and let mdflow interpret it.
//! Script Kit only extracts what it needs for UI/logic purposes.

// These functions are public API for future integration - allow them to be unused for now
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

use crate::agents::types::{
    Agent, AgentBackend, AgentFrontmatter, MdflowInput, MdflowInputType, MdflowInputs,
};
use tracing::warn;

/// Agent parsing errors for malformed file content.
#[derive(Debug, thiserror::Error)]
pub enum AgentParseError {
    #[error("frontmatter started with '---' but missing closing '---' delimiter")]
    MissingFrontmatterClosingDelimiter,
    #[error("frontmatter YAML root must be a mapping/object")]
    FrontmatterMustBeMapping,
    #[error("invalid frontmatter YAML: {0}")]
    InvalidFrontmatterYaml(#[from] serde_yaml::Error),
}

/// Parse YAML frontmatter from markdown content
///
/// Returns:
/// - `Ok(None)` when content has no frontmatter
/// - `Ok(Some(...))` when frontmatter is present and valid
/// - `Err(...)` when frontmatter delimiters/YAML are malformed
///
/// Frontmatter must:
/// - Start with `---` on the first line (after optional whitespace)
/// - End with `---` on its own line
///
/// # Example
///
/// ```yaml
/// ---
/// _sk_name: "Review PR"
/// model: sonnet
/// ---
/// ```
pub fn parse_frontmatter(content: &str) -> Result<Option<AgentFrontmatter>, AgentParseError> {
    let trimmed = content.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    let (opening_line, remaining_after_opening, opening_consumed) = split_first_line(trimmed);

    // Must start with a standalone --- delimiter line
    if opening_line.trim_end() != "---" {
        return Ok(None);
    }

    // Frontmatter must span multiple lines to include a closing delimiter
    if opening_consumed == trimmed.len() {
        return Err(AgentParseError::MissingFrontmatterClosingDelimiter);
    }

    // Find closing delimiter on its own line.
    let mut cursor = remaining_after_opening;
    let mut yaml_end_offset = 0usize;
    let mut found_closing = false;

    while !cursor.is_empty() {
        let (line, rest, consumed) = split_first_line(cursor);
        if line.trim_end() == "---" {
            found_closing = true;
            break;
        }
        yaml_end_offset += consumed;
        cursor = rest;
    }

    if !found_closing {
        return Err(AgentParseError::MissingFrontmatterClosingDelimiter);
    }

    let yaml_content = remaining_after_opening[..yaml_end_offset].trim();

    if yaml_content.is_empty() {
        return Ok(Some(AgentFrontmatter::default()));
    }

    // Parse as generic YAML and retain only string keys for frontmatter extraction.
    let parsed: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;
    let mapping = parsed
        .as_mapping()
        .ok_or(AgentParseError::FrontmatterMustBeMapping)?;
    let mut raw = HashMap::new();
    for (key, value) in mapping {
        if let Some(key_str) = key.as_str() {
            raw.insert(key_str.to_string(), value.clone());
        } else {
            warn!(
                key = ?key,
                "Skipping non-string frontmatter key while parsing agent"
            );
        }
    }

    Ok(Some(extract_frontmatter_fields(raw)))
}

/// Split the first line from a string, supporting `\n`, `\r\n`, and `\r`.
///
/// Returns: `(line_without_newline, rest_after_newline, consumed_bytes)`.
fn split_first_line(content: &str) -> (&str, &str, usize) {
    let bytes = content.as_bytes();
    for (idx, byte) in bytes.iter().enumerate() {
        match byte {
            b'\n' => {
                let line_end = if idx > 0 && bytes[idx - 1] == b'\r' {
                    idx - 1
                } else {
                    idx
                };
                return (&content[..line_end], &content[idx + 1..], idx + 1);
            }
            b'\r' => {
                if idx + 1 < bytes.len() && bytes[idx + 1] == b'\n' {
                    return (&content[..idx], &content[idx + 2..], idx + 2);
                }
                return (&content[..idx], &content[idx + 1..], idx + 1);
            }
            _ => {}
        }
    }

    (content, "", content.len())
}

/// Extract frontmatter fields from raw YAML
fn extract_frontmatter_fields(raw: HashMap<String, serde_yaml::Value>) -> AgentFrontmatter {
    let mut fm = AgentFrontmatter {
        raw: raw.clone(),
        ..Default::default()
    };

    for (key, value) in &raw {
        match key.as_str() {
            // Script Kit metadata
            "_sk_name" => {
                fm.sk_name = value.as_str().map(|s| s.to_string());
            }
            "_sk_description" => {
                fm.sk_description = value.as_str().map(|s| s.to_string());
            }
            "_sk_icon" => {
                fm.sk_icon = value.as_str().map(|s| s.to_string());
            }
            "_sk_alias" => {
                fm.sk_alias = value.as_str().map(|s| s.to_string());
            }
            "_sk_shortcut" => {
                fm.sk_shortcut = value.as_str().map(|s| s.to_string());
            }

            // mdflow system keys
            "_inputs" => {
                fm.inputs = parse_inputs(value);
            }
            "_interactive" | "_i" => {
                fm.interactive = value.as_bool().or(Some(true));
            }
            "_cwd" => {
                fm.cwd = value.as_str().map(|s| s.to_string());
            }
            "_command" | "_c" => {
                fm.command = value.as_str().map(|s| s.to_string());
            }
            "_env" => {
                fm.env = parse_env(value);
            }
            _ => {
                // Keep in raw for mdflow to interpret
            }
        }
    }

    fm
}

/// Parse `_inputs` from frontmatter
///
/// Supports two formats:
/// 1. Typed object format:
///    ```yaml
///    _inputs:
///      feature_name:
///        type: text
///        message: "Feature name?"
///    ```
/// 2. Legacy array format:
///    ```yaml
///    _inputs:
///      - feature_name
///      - confirm
///    ```
fn parse_inputs(value: &serde_yaml::Value) -> Option<MdflowInputs> {
    match value {
        serde_yaml::Value::Sequence(arr) => {
            // Legacy array format
            let names: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if names.is_empty() {
                None
            } else {
                Some(MdflowInputs::Legacy(names))
            }
        }
        serde_yaml::Value::Mapping(map) => {
            // Typed object format
            let mut inputs = HashMap::new();
            for (key, spec) in map {
                if let Some(key_str) = key.as_str() {
                    let input = parse_single_input(spec);
                    inputs.insert(key_str.to_string(), input);
                }
            }
            if inputs.is_empty() {
                None
            } else {
                Some(MdflowInputs::Typed(inputs))
            }
        }
        _ => None,
    }
}

/// Parse a single input specification
fn parse_single_input(spec: &serde_yaml::Value) -> MdflowInput {
    let mut input = MdflowInput::default();

    if let serde_yaml::Value::Mapping(map) = spec {
        for (key, value) in map {
            if let Some(key_str) = key.as_str() {
                match key_str {
                    "type" => {
                        if let Some(t) = value.as_str() {
                            input.input_type = MdflowInputType::parse(t);
                        }
                    }
                    "message" => {
                        input.message = value.as_str().map(|s| s.to_string());
                    }
                    "default" => {
                        input.default = value.as_str().map(|s| s.to_string());
                    }
                    "choices" => {
                        if let serde_yaml::Value::Sequence(arr) = value {
                            input.choices = Some(
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect(),
                            );
                        }
                    }
                    "required" => {
                        input.required = value.as_bool();
                    }
                    _ => {}
                }
            }
        }
    }

    input
}

/// Parse `_env` from frontmatter
fn parse_env(value: &serde_yaml::Value) -> Option<HashMap<String, String>> {
    if let serde_yaml::Value::Mapping(map) = value {
        let mut env = HashMap::new();
        for (key, val) in map {
            if let (Some(k), Some(v)) = (key.as_str(), val.as_str()) {
                env.insert(k.to_string(), v.to_string());
            }
        }
        if env.is_empty() {
            None
        } else {
            Some(env)
        }
    } else {
        None
    }
}

/// Detect shell inlines in markdown content
///
/// Shell inlines use these syntaxes in mdflow:
/// - `` !`command` `` - Shell inline execution
/// - `` @`command` `` - Command include (runs command, includes output)
pub fn has_shell_inlines(content: &str) -> bool {
    // Look for !` or @` pattern (backtick-enclosed command)
    content.contains("!`") || content.contains("@`")
}

/// Detect remote URL imports in markdown content
///
/// Remote imports use `@https://` or `@http://` syntax.
pub fn has_remote_imports(content: &str) -> bool {
    content.contains("@https://") || content.contains("@http://")
}

/// Extract display name from filename
///
/// Strips backend suffix and extensions, replaces separators with spaces.
/// Example: "my-task.claude.md" → "my task"
pub fn name_from_filename(filename: &str) -> String {
    filename
        .trim_end_matches(".md")
        // Remove backend suffixes
        .trim_end_matches(".claude")
        .trim_end_matches(".gemini")
        .trim_end_matches(".codex")
        .trim_end_matches(".copilot")
        // Remove interactive marker
        .trim_end_matches(".i")
        // Replace separators
        .replace(['-', '_'], " ")
}

/// Check if filename indicates interactive mode
pub fn is_interactive_filename(filename: &str) -> bool {
    filename.to_lowercase().contains(".i.")
}

/// Parse an agent from file path and content
///
/// This is the main entry point for parsing agent files.
pub fn parse_agent(path: &Path, content: &str) -> Option<Agent> {
    let filename = path.file_name()?.to_str()?;

    // Skip hidden files
    if filename.starts_with('.') {
        return None;
    }

    // Must be .md file
    if !filename.ends_with(".md") {
        return None;
    }

    // Parse backend from filename
    let mut backend = AgentBackend::from_filename(filename);

    // Check for interactive marker in filename
    let interactive_from_filename = is_interactive_filename(filename);

    // Parse frontmatter
    let frontmatter = match parse_frontmatter(content) {
        Ok(Some(frontmatter)) => frontmatter,
        Ok(None) => AgentFrontmatter::default(),
        Err(error) => {
            warn!(
                path = %path.display(),
                %error,
                "Skipping agent file due to malformed frontmatter"
            );
            return None;
        }
    };

    // Override backend if _command is specified
    if let Some(ref cmd) = frontmatter.command {
        backend = AgentBackend::Other(cmd.clone());
    }

    // Determine interactive mode (filename or frontmatter)
    let interactive = interactive_from_filename || frontmatter.interactive.unwrap_or(false);

    // Build name: prefer _sk_name, fall back to filename
    let name = frontmatter
        .sk_name
        .clone()
        .unwrap_or_else(|| name_from_filename(filename));

    Some(Agent {
        name,
        path: path.to_path_buf(),
        backend,
        interactive,
        description: frontmatter.sk_description.clone(),
        icon: frontmatter.sk_icon.clone(),
        shortcut: frontmatter.sk_shortcut.clone(),
        alias: frontmatter.sk_alias.clone(),
        frontmatter,
        kit: None, // Set by loader
        has_shell_inlines: has_shell_inlines(content),
        has_remote_imports: has_remote_imports(content),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_frontmatter_ok(content: &str) -> AgentFrontmatter {
        parse_frontmatter(content)
            .expect("frontmatter parse should not fail")
            .expect("frontmatter should exist")
    }

    // === Frontmatter parsing tests ===

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = r#"---
model: opus
---
Hello world"#;

        let fm = parse_frontmatter_ok(content);
        assert!(fm.raw.contains_key("model"));
        assert_eq!(fm.raw.get("model").and_then(|v| v.as_str()), Some("opus"));
    }

    #[test]
    fn test_parse_frontmatter_sk_metadata() {
        let content = r#"---
_sk_name: "Review PR"
_sk_description: "Reviews staged changes"
_sk_icon: "git-pull-request"
_sk_alias: "review"
_sk_shortcut: "cmd shift r"
model: sonnet
---
Prompt here"#;

        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.sk_name, Some("Review PR".to_string()));
        assert_eq!(
            fm.sk_description,
            Some("Reviews staged changes".to_string())
        );
        assert_eq!(fm.sk_icon, Some("git-pull-request".to_string()));
        assert_eq!(fm.sk_alias, Some("review".to_string()));
        assert_eq!(fm.sk_shortcut, Some("cmd shift r".to_string()));
        // Raw should also contain model
        assert!(fm.raw.contains_key("model"));
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "Just markdown content without frontmatter";
        assert!(matches!(parse_frontmatter(content), Ok(None)));
    }

    #[test]
    fn test_parse_agent_invalid_frontmatter_returns_none() {
        let path = Path::new("/path/to/bad-frontmatter.claude.md");
        let content = r#"---
_sk_name: "Broken
model: sonnet
---
Prompt"#;

        assert!(parse_agent(path, content).is_none());
    }

    #[test]
    fn test_parse_frontmatter_invalid_yaml_returns_error() {
        let content = r#"---
_sk_name: "Broken
model: sonnet
---
Prompt"#;

        assert!(matches!(
            parse_frontmatter(content),
            Err(AgentParseError::InvalidFrontmatterYaml(_))
        ));
    }

    #[test]
    fn test_parse_frontmatter_with_utf8_bom() {
        let content = "\u{feff}---\n_sk_name: \"Unicode Agent\"\n---\nPrompt";
        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.sk_name, Some("Unicode Agent".to_string()));
    }

    #[test]
    fn test_parse_frontmatter_requires_standalone_opening_delimiter() {
        let content = r#"---not-frontmatter
model: sonnet
---
Prompt"#;
        assert!(matches!(parse_frontmatter(content), Ok(None)));
    }

    #[test]
    fn test_parse_frontmatter_requires_standalone_closing_delimiter() {
        let content = r#"---
_sk_name: "Broken"
---not-a-delimiter
Prompt"#;
        assert!(matches!(
            parse_frontmatter(content),
            Err(AgentParseError::MissingFrontmatterClosingDelimiter)
        ));
    }

    #[test]
    fn test_parse_frontmatter_non_mapping_yaml_returns_error() {
        let content = r#"---
- one
- two
---
Prompt"#;
        assert!(matches!(
            parse_frontmatter(content),
            Err(AgentParseError::FrontmatterMustBeMapping)
        ));
    }

    #[test]
    fn test_parse_frontmatter_ignores_non_string_keys() {
        let content = r#"---
123: ignored
model: sonnet
---
Prompt"#;
        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.raw.get("model").and_then(|v| v.as_str()), Some("sonnet"));
        assert!(!fm.raw.contains_key("123"));
    }

    #[test]
    fn test_parse_frontmatter_crlf_with_trailing_delimiter_spaces() {
        let content = "---\r\n_sk_name: \"CRLF Agent\"\r\n---   \r\nPrompt";
        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.sk_name, Some("CRLF Agent".to_string()));
    }

    #[test]
    fn test_parse_frontmatter_empty() {
        let content = r#"---
---
Content"#;

        let fm = parse_frontmatter_ok(content);
        assert!(fm.raw.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_interactive() {
        let content = r#"---
_interactive: true
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.interactive, Some(true));
    }

    #[test]
    fn test_parse_frontmatter_interactive_short() {
        let content = r#"---
_i: true
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.interactive, Some(true));
    }

    #[test]
    fn test_parse_frontmatter_command() {
        let content = r#"---
_command: ollama
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.command, Some("ollama".to_string()));
    }

    #[test]
    fn test_parse_frontmatter_cwd() {
        let content = r#"---
_cwd: /tmp
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        assert_eq!(fm.cwd, Some("/tmp".to_string()));
    }

    #[test]
    fn test_parse_frontmatter_env() {
        let content = r#"---
_env:
  NODE_ENV: production
  DEBUG: "true"
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        let env = fm.env.unwrap();
        assert_eq!(env.get("NODE_ENV"), Some(&"production".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
    }

    // === _inputs parsing tests ===

    #[test]
    fn test_parse_inputs_legacy_array() {
        let content = r#"---
_inputs:
  - feature_name
  - confirm_deploy
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        match fm.inputs {
            Some(MdflowInputs::Legacy(names)) => {
                assert_eq!(names, vec!["feature_name", "confirm_deploy"]);
            }
            _ => panic!("Expected legacy array format"),
        }
    }

    #[test]
    fn test_parse_inputs_typed_object() {
        let content = r#"---
_inputs:
  feature_name:
    type: text
    message: "Feature name?"
    default: "New Feature"
  confirm_deploy:
    type: confirm
    message: "Deploy to production?"
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        match fm.inputs {
            Some(MdflowInputs::Typed(inputs)) => {
                assert!(inputs.contains_key("feature_name"));
                assert!(inputs.contains_key("confirm_deploy"));

                let feature = inputs.get("feature_name").unwrap();
                assert_eq!(feature.input_type, MdflowInputType::Text);
                assert_eq!(feature.message, Some("Feature name?".to_string()));
                assert_eq!(feature.default, Some("New Feature".to_string()));

                let confirm = inputs.get("confirm_deploy").unwrap();
                assert_eq!(confirm.input_type, MdflowInputType::Confirm);
            }
            _ => panic!("Expected typed object format"),
        }
    }

    #[test]
    fn test_parse_inputs_with_choices() {
        let content = r#"---
_inputs:
  environment:
    type: select
    message: "Select environment"
    choices:
      - dev
      - staging
      - prod
---
Prompt"#;

        let fm = parse_frontmatter_ok(content);
        match fm.inputs {
            Some(MdflowInputs::Typed(inputs)) => {
                let env = inputs.get("environment").unwrap();
                assert_eq!(env.input_type, MdflowInputType::Select);
                assert_eq!(
                    env.choices,
                    Some(vec![
                        "dev".to_string(),
                        "staging".to_string(),
                        "prod".to_string()
                    ])
                );
            }
            _ => panic!("Expected typed object format"),
        }
    }

    // === Shell inline detection tests ===

    #[test]
    fn test_has_shell_inlines_true() {
        let content = r#"
Build context:
!`git log -5`
!`cat README.md`
"#;
        assert!(has_shell_inlines(content));
    }

    #[test]
    fn test_has_shell_inlines_false() {
        let content = "Just regular markdown without shell commands";
        assert!(!has_shell_inlines(content));
    }

    // === Remote import detection tests ===

    #[test]
    fn test_has_remote_imports_https() {
        let content = "@https://example.com/context.md";
        assert!(has_remote_imports(content));
    }

    #[test]
    fn test_has_remote_imports_http() {
        let content = "@http://example.com/context.md";
        assert!(has_remote_imports(content));
    }

    #[test]
    fn test_has_remote_imports_false() {
        let content = "@./local/file.md";
        assert!(!has_remote_imports(content));
    }

    // === Name extraction tests ===

    #[test]
    fn test_name_from_filename_basic() {
        assert_eq!(name_from_filename("my-task.md"), "my task");
        assert_eq!(name_from_filename("my_task.md"), "my task");
    }

    #[test]
    fn test_name_from_filename_with_backend() {
        assert_eq!(name_from_filename("review.claude.md"), "review");
        assert_eq!(name_from_filename("task.gemini.md"), "task");
        assert_eq!(name_from_filename("analyze.codex.md"), "analyze");
        assert_eq!(name_from_filename("help.copilot.md"), "help");
    }

    #[test]
    fn test_name_from_filename_with_interactive() {
        assert_eq!(name_from_filename("task.i.claude.md"), "task");
        assert_eq!(name_from_filename("review.i.gemini.md"), "review");
    }

    // === Interactive filename detection tests ===

    #[test]
    fn test_is_interactive_filename_true() {
        assert!(is_interactive_filename("task.i.claude.md"));
        assert!(is_interactive_filename("review.I.gemini.md"));
    }

    #[test]
    fn test_is_interactive_filename_false() {
        assert!(!is_interactive_filename("task.claude.md"));
        assert!(!is_interactive_filename("interactive.md")); // "i" not as marker
    }

    // === Full agent parsing tests ===

    #[test]
    fn test_parse_agent_basic() {
        let path = Path::new("/path/to/review.claude.md");
        let content = r#"---
_sk_name: "Review Changes"
_sk_description: "Reviews staged git changes"
model: sonnet
---
Please review the following changes:
@`git diff --staged`
"#;

        let agent = parse_agent(path, content).unwrap();
        assert_eq!(agent.name, "Review Changes");
        assert_eq!(
            agent.description,
            Some("Reviews staged git changes".to_string())
        );
        assert_eq!(agent.backend, AgentBackend::Claude);
        assert!(!agent.interactive);
        assert!(agent.has_shell_inlines);
        assert!(!agent.has_remote_imports);
    }

    #[test]
    fn test_parse_agent_interactive() {
        let path = Path::new("/path/to/chat.i.claude.md");
        let content = r#"---
model: opus
---
Interactive chat session
"#;

        let agent = parse_agent(path, content).unwrap();
        assert!(agent.interactive);
        assert_eq!(agent.backend, AgentBackend::Claude);
    }

    #[test]
    fn test_parse_agent_name_from_filename() {
        let path = Path::new("/path/to/my-cool-task.gemini.md");
        let content = r#"---
model: gemini-2.0-flash
---
Do something cool
"#;

        let agent = parse_agent(path, content).unwrap();
        assert_eq!(agent.name, "my cool task");
        assert_eq!(agent.backend, AgentBackend::Gemini);
    }

    #[test]
    fn test_parse_agent_custom_command() {
        let path = Path::new("/path/to/ollama-task.md");
        let content = r#"---
_command: ollama
_sk_name: "Ollama Task"
---
Run with ollama
"#;

        let agent = parse_agent(path, content).unwrap();
        assert_eq!(agent.name, "Ollama Task");
        assert_eq!(agent.backend, AgentBackend::Other("ollama".to_string()));
    }

    #[test]
    fn test_parse_agent_skip_hidden() {
        let path = Path::new("/path/to/.hidden.claude.md");
        let content = "---\n---\nContent";
        assert!(parse_agent(path, content).is_none());
    }

    #[test]
    fn test_parse_agent_skip_non_md() {
        let path = Path::new("/path/to/script.ts");
        let content = "export default {}";
        assert!(parse_agent(path, content).is_none());
    }

    #[test]
    fn test_parse_agent_with_remote_imports() {
        let path = Path::new("/path/to/task.claude.md");
        let content = r#"---
model: sonnet
---
Use this context:
@https://raw.githubusercontent.com/user/repo/main/README.md
"#;

        let agent = parse_agent(path, content).unwrap();
        assert!(agent.has_remote_imports);
        assert!(!agent.has_shell_inlines);
    }

    #[test]
    fn test_parse_agent_preserves_raw_frontmatter() {
        let path = Path::new("/path/to/task.claude.md");
        let content = r#"---
model: opus
dangerously-skip-permissions: true
add-dir:
  - ./src
  - ./tests
custom-flag: value
---
Prompt
"#;

        let agent = parse_agent(path, content).unwrap();
        let raw = &agent.frontmatter.raw;

        // Verify all keys are preserved for mdflow
        assert!(raw.contains_key("model"));
        assert!(raw.contains_key("dangerously-skip-permissions"));
        assert!(raw.contains_key("add-dir"));
        assert!(raw.contains_key("custom-flag"));
    }
}

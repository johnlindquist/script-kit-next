use crate::config::Config;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, instrument, warn};
/// Scripts directory under ~/.scriptkit/kit/main/
const SCRIPTS_DIR: &str = "~/.scriptkit/kit/main/scripts";
/// Extensions directory under ~/.scriptkit/kit/main/
const EXTENSIONS_DIR: &str = "~/.scriptkit/kit/main/extensions";
/// Maximum filename size on most filesystems (bytes, not chars).
const MAX_FILENAME_BYTES: usize = 255;
/// Reserved filenames on Windows that are invalid even with an extension.
const WINDOWS_RESERVED_FILENAMES: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];
fn scripts_dir() -> PathBuf {
    PathBuf::from(shellexpand::tilde(SCRIPTS_DIR).as_ref())
}
fn extensions_dir() -> PathBuf {
    PathBuf::from(shellexpand::tilde(EXTENSIONS_DIR).as_ref())
}
fn validate_sanitized_name(
    original_name: &str,
    sanitized_name: &str,
    extension: &str,
    item_label: &str,
) -> Result<()> {
    if sanitized_name.is_empty() {
        anyhow::bail!(
            "{item_label} name cannot be empty after sanitization (original='{original_name}', sanitized='{sanitized_name}')"
        );
    }

    let normalized = sanitized_name.to_ascii_lowercase();
    if WINDOWS_RESERVED_FILENAMES.contains(&normalized.as_str()) {
        anyhow::bail!(
            "{item_label} name is reserved on Windows (original='{original_name}', sanitized='{sanitized_name}')"
        );
    }

    let filename = format!("{sanitized_name}.{extension}");
    if filename.len() > MAX_FILENAME_BYTES {
        anyhow::bail!(
            "{item_label} filename too long after sanitization (original='{original_name}', sanitized='{sanitized_name}', max_bytes={MAX_FILENAME_BYTES})"
        );
    }

    Ok(())
}
fn create_new_text_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
fn create_unique_templated_file<F>(
    original_name: &str,
    sanitized_name: &str,
    dir: &Path,
    extension: &str,
    item_label: &str,
    template_for_name: F,
) -> Result<(PathBuf, String)>
where
    F: Fn(&str) -> String,
{
    for suffix in 0usize.. {
        let candidate_name = if suffix == 0 {
            sanitized_name.to_string()
        } else {
            format!("{sanitized_name}-{suffix}")
        };
        validate_sanitized_name(original_name, &candidate_name, extension, item_label)?;

        let filename = format!("{candidate_name}.{extension}");
        let path = dir.join(&filename);
        let template = template_for_name(&candidate_name);

        match create_new_text_file(&path, &template) {
            Ok(()) => return Ok((path, candidate_name)),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                warn!(
                    candidate = %candidate_name,
                    path = %path.display(),
                    "script_creation_collision_retry"
                );
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!(
                        "Failed to write {item_label} file '{}' (sanitized='{}')",
                        path.display(),
                        candidate_name
                    )
                });
            }
        }
    }

    unreachable!("infinite suffix loop should always return or error")
}
/// Sanitize a script name for use as a filename.
///
/// - Converts to lowercase
/// - Replaces spaces and underscores with hyphens
/// - Removes special characters (keeps only alphanumeric and hyphens)
/// - Removes leading/trailing hyphens
/// - Collapses multiple consecutive hyphens into one
fn sanitize_name(name: &str) -> String {
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c == ' ' || c == '_' || c == '-' {
                '-'
            } else {
                // Skip other special characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect();

    // Collapse multiple consecutive hyphens and trim
    let mut result = String::new();
    let mut last_was_hyphen = false;

    for c in sanitized.chars() {
        if c == '-' {
            if !last_was_hyphen && !result.is_empty() {
                result.push(c);
                last_was_hyphen = true;
            }
        } else {
            result.push(c);
            last_was_hyphen = false;
        }
    }

    // Remove trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    result
}
/// Convert a sanitized filename to a human-readable title.
///
/// - Replaces hyphens with spaces
/// - Capitalizes first letter of each word
fn name_to_title(name: &str) -> String {
    name.split('-')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
/// Generate the script template using the global metadata format.
///
/// This is the preferred format per AGENTS.md - uses `export const metadata = {...}`
/// instead of comment-based metadata.
fn generate_script_template(name: &str) -> String {
    let title = name_to_title(name);
    format!(
        r#"import "@scriptkit/sdk";

export const metadata = {{
  name: "{title}",
  description: "",
}};

/*
Template Guide
=============
A) Input Prompt
   Ask for a value with `arg()` and continue with that input.

B) Selection List
   Return an array from `arg()` to let users pick one option.

C) Background Task
   Use async functions and status updates for longer work.
*/

// Quick starter: input prompt
const result = await arg("What should this script do?");
console.log(result);
"#
    )
}
/// Generate the extension template as markdown with embedded code.
///
/// Extensions are markdown files with code blocks that can be executed.
fn generate_extension_template(name: &str) -> String {
    let title = name_to_title(name);
    format!(
        r#"---
name: {title}
description: "What this extension bundle includes"
author: "Your Name"
icon: wrench
---

<!--
  YAML frontmatter:
  - name: display title in menus
  - description: short summary shown in UI
  - icon: choose a lucide icon name (for example: wrench, code, zap)

  Learn more about creating extensions: ~/.scriptkit/kit/GUIDE.md
-->

# {title}

Scriptlets in this bundle.

## Hello World

```bash
echo "Hello from {title}!"
```
"#
    )
}
/// Create a new script file in ~/.scriptkit/scripts/
///
/// # Arguments
///
/// * `name` - The name of the script (will be sanitized for filename)
///
/// # Returns
///
/// The path to the created script file.
///
/// # Errors
///
/// Returns an error if:
/// - The scripts directory cannot be created
/// - A valid filename cannot be derived from the provided name
/// - The file cannot be written
#[instrument(name = "create_new_script", skip_all, fields(name = %name))]
pub fn create_new_script(name: &str) -> Result<PathBuf> {
    create_new_script_in_dir(name, &scripts_dir())
}
fn create_new_script_in_dir(name: &str, scripts_dir: &Path) -> Result<PathBuf> {
    let sanitized_name = sanitize_name(name);
    validate_sanitized_name(name, &sanitized_name, "ts", "Script")?;

    // Ensure the scripts directory exists
    fs::create_dir_all(scripts_dir).with_context(|| {
        format!(
            "Failed to create scripts directory: {}",
            scripts_dir.display()
        )
    })?;

    let (script_path, created_name) = create_unique_templated_file(
        name,
        &sanitized_name,
        scripts_dir,
        "ts",
        "script",
        generate_script_template,
    )?;

    info!(
        path = %script_path.display(),
        name = %created_name,
        requested_name = %name,
        "Created new script"
    );

    Ok(script_path)
}
/// Create a new extension file in ~/.scriptkit/kit/main/extensions/
///
/// # Arguments
///
/// * `name` - The name of the extension (will be sanitized for filename)
///
/// # Returns
///
/// The path to the created extension file.
///
/// # Errors
///
/// Returns an error if:
/// - The extensions directory cannot be created
/// - A valid filename cannot be derived from the provided name
/// - The file cannot be written
#[instrument(name = "create_new_extension", skip_all, fields(name = %name))]
pub fn create_new_extension(name: &str) -> Result<PathBuf> {
    create_new_extension_in_dir(name, &extensions_dir())
}
fn create_new_extension_in_dir(name: &str, extensions_dir: &Path) -> Result<PathBuf> {
    let sanitized_name = sanitize_name(name);
    validate_sanitized_name(name, &sanitized_name, "md", "Extension")?;

    // Ensure the extensions directory exists
    fs::create_dir_all(extensions_dir).with_context(|| {
        format!(
            "Failed to create extensions directory: {}",
            extensions_dir.display()
        )
    })?;

    let (extension_path, created_name) = create_unique_templated_file(
        name,
        &sanitized_name,
        extensions_dir,
        "md",
        "extension",
        generate_extension_template,
    )?;

    info!(
        path = %extension_path.display(),
        name = %created_name,
        requested_name = %name,
        "Created new extension"
    );

    Ok(extension_path)
}
/// Open a file in the configured editor.
///
/// Uses the editor from config, falling back to $EDITOR env var,
/// then to "code" (VS Code) as the final default.
///
/// # Arguments
///
/// * `path` - The path to the file to open
/// * `config` - The application configuration
///
/// # Errors
///
/// Returns an error if the editor command fails to spawn.
fn parse_editor_command(editor: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = editor.chars().peekable();

    while let Some(ch) = chars.next() {
        match quote {
            Some(quote_char) => {
                if ch == quote_char {
                    quote = None;
                } else if ch == '\\' && quote_char == '"' {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    } else {
                        current.push(ch);
                    }
                } else {
                    current.push(ch);
                }
            }
            None => match ch {
                '\'' | '"' => quote = Some(ch),
                '\\' => {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    } else {
                        current.push(ch);
                    }
                }
                c if c.is_whitespace() => {
                    if !current.is_empty() {
                        parts.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(ch),
            },
        }
    }

    if quote.is_some() {
        anyhow::bail!("Unterminated quote in editor command: {editor}");
    }

    if !current.is_empty() {
        parts.push(current);
    }

    if parts.is_empty() {
        anyhow::bail!("Editor command is empty");
    }

    Ok(parts)
}
#[instrument(name = "open_in_editor", skip(config), fields(path = %path.display()))]
pub fn open_in_editor(path: &Path, config: &Config) -> Result<()> {
    let editor = config.get_editor();
    let command_parts = parse_editor_command(&editor)?;
    let executable = &command_parts[0];
    let args = &command_parts[1..];

    info!(editor = %editor, path = %path.display(), "Opening file in editor");

    let mut command = Command::new(executable);
    command.args(args);
    command.arg(path);

    let status = command.spawn().with_context(|| {
        format!(
            "Failed to spawn editor command for file {} (raw='{}', executable='{}', args={:?})",
            path.display(),
            editor,
            executable,
            args
        )
    })?;

    // We spawn and detach - don't wait for the editor to close
    // The child process handle is dropped, but the process continues
    drop(status);

    Ok(())
}

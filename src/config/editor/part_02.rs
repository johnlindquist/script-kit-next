use std::io::Write;

/// Validate TypeScript content by attempting compilation with bun.
///
/// Writes content to a temp .ts file and runs `bun build` on it.
/// Returns Ok(()) if valid, Err with details if invalid.
pub fn validate_typescript(content: &str, bun_path: Option<&str>) -> Result<(), String> {
    let bun = bun_path.unwrap_or("bun");

    let tmp = tempfile::Builder::new()
        .suffix(".ts")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    std::fs::write(tmp.path(), content).map_err(|e| format!("Failed to write temp file: {}", e))?;

    let output = std::process::Command::new(bun)
        .arg("build")
        .arg("--target=bun")
        .arg("--no-bundle")
        .arg(tmp.path())
        .arg("--outfile=/dev/null")
        .output()
        .map_err(|e| format!("Failed to run bun: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("TypeScript compilation failed: {}", stderr.trim()))
    }
}

/// Structural validation using tree-sitter AST.
///
/// Parses the content with tree-sitter-typescript and checks that:
/// 1. Content contains `export default`
/// 2. The AST has no ERROR nodes (no syntax errors)
/// 3. The export default object can be found
/// 4. Content ends with `satisfies Config;` or `as Config;`
pub fn validate_structure(content: &str) -> Result<(), String> {
    if !content.contains("export default") {
        return Err("Missing 'export default' declaration".into());
    }

    let tree =
        parse_typescript(content).map_err(|e| format!("Failed to parse TypeScript: {}", e))?;
    let root = tree.root_node();

    // Check for parse errors (syntax errors, corruption, etc.)
    if root.has_error() {
        let mut error_msg = String::new();
        collect_ast_errors(root, content, &mut error_msg);
        if !error_msg.is_empty() {
            return Err(format!("TypeScript parse errors: {}", error_msg));
        }
        return Err("TypeScript parse errors detected".into());
    }

    // Find export_statement and its object
    let export = find_export_statement(root).ok_or("No export statement found in AST")?;

    find_object_in_export(export, content)
        .ok_or_else(|| "Could not find config object in export statement".to_string())?;

    let trimmed = content.trim();
    if !trimmed.contains("satisfies Config;") && !trimmed.contains("as Config;") {
        return Err("Missing 'satisfies Config' or 'as Config' type assertion".into());
    }

    Ok(())
}

/// Generate a fresh config.ts with the given property included.
fn generate_fresh_config(property: &ConfigProperty) -> String {
    format!(
        r#"import type {{ Config }} from "@scriptkit/sdk";

export default {{
  hotkey: {{ modifiers: ["meta"], key: "Semicolon" }},
  {}: {},
}} satisfies Config;
"#,
        property.name, property.value
    )
}

fn atomic_write_with_secure_tempfile(
    config_path: &Path,
    content: &str,
) -> Result<(), ConfigWriteError> {
    let parent = config_path.parent().ok_or_else(|| {
        ConfigWriteError::IoError(format!(
            "config_atomic_write_missing_parent: target={}",
            config_path.display()
        ))
    })?;

    let mut temp_file = tempfile::Builder::new()
        .prefix(".config-write.")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|e| {
            ConfigWriteError::IoError(format!(
                "config_atomic_write_tempfile_create_failed: target={} error={}",
                config_path.display(),
                e
            ))
        })?;

    temp_file.write_all(content.as_bytes()).map_err(|e| {
        ConfigWriteError::IoError(format!(
            "config_atomic_write_tempfile_write_failed: target={} error={}",
            config_path.display(),
            e
        ))
    })?;

    temp_file.flush().map_err(|e| {
        ConfigWriteError::IoError(format!(
            "config_atomic_write_tempfile_flush_failed: target={} error={}",
            config_path.display(),
            e
        ))
    })?;

    temp_file.persist(config_path).map_err(|e| {
        ConfigWriteError::IoError(format!(
            "config_atomic_write_rename_failed: from={} to={} error={}",
            e.file.path().display(),
            config_path.display(),
            e.error
        ))
    })?;

    Ok(())
}

/// Safely modify config.ts: edit in memory, validate, backup, atomic-write.
///
/// This is the single entry point for all config file modifications.
/// Guarantees:
/// 1. Output is valid TypeScript (validated by bun, fallback to structural)
/// 2. A backup exists before overwriting
/// 3. The write is atomic (temp file + rename)
pub fn write_config_safely(
    config_path: &Path,
    property: &ConfigProperty,
    bun_path: Option<&str>,
) -> Result<WriteOutcome, ConfigWriteError> {
    // Step 1: Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigWriteError::IoError(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }
    }

    // Step 2: Read existing content
    let content = std::fs::read_to_string(config_path).unwrap_or_default();

    // Step 3: Produce new content
    let (new_content, was_empty) = if content.is_empty() {
        (generate_fresh_config(property), true)
    } else {
        match add_property(&content, property) {
            EditResult::Modified(new_content) => (new_content, false),
            EditResult::AlreadySet => return Ok(WriteOutcome::AlreadySet),
            EditResult::Failed(reason) => {
                return Err(ConfigWriteError::EditFailed(reason));
            }
        }
    };

    // Step 4: Validate the new content
    let validation_result = validate_typescript(&new_content, bun_path);
    match &validation_result {
        Ok(()) => { /* bun says it's valid */ }
        Err(bun_err) => {
            if bun_err.starts_with("Failed to run bun") {
                // bun unavailable - fall back to structural validation
                tracing::warn!("bun unavailable for validation, using structural check");
                validate_structure(&new_content).map_err(ConfigWriteError::ValidationFailed)?;
            } else {
                // bun ran but TypeScript is invalid - do NOT write
                return Err(ConfigWriteError::ValidationFailed(bun_err.clone()));
            }
        }
    }

    // Step 5: Backup existing file (if non-empty)
    if !content.is_empty() {
        let backup_path = config_path.with_extension("ts.bak");
        if let Err(e) = std::fs::copy(config_path, &backup_path) {
            tracing::warn!(
                error = %e,
                path = %backup_path.display(),
                "Failed to create config backup"
            );
        } else {
            tracing::info!(path = %backup_path.display(), "Config backup saved");
        }
    }

    // Step 6: Atomic write (randomized tempfile in same directory + rename)
    atomic_write_with_secure_tempfile(config_path, &new_content)?;

    if was_empty {
        Ok(WriteOutcome::Created)
    } else {
        Ok(WriteOutcome::Written)
    }
}

/// Enable Claude Code in config.ts using the safe write path.
pub fn enable_claude_code_safely(
    config_path: &Path,
    bun_path: Option<&str>,
) -> Result<WriteOutcome, ConfigWriteError> {
    let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
    write_config_safely(config_path, &property, bun_path)
}

/// Attempt to recover config.ts from its backup.
///
/// Returns Ok(true) if recovery succeeded, Ok(false) if no backup exists.
pub fn recover_from_backup(
    config_path: &Path,
    bun_path: Option<&str>,
) -> Result<bool, ConfigWriteError> {
    let backup_path = config_path.with_extension("ts.bak");

    if !backup_path.exists() {
        return Ok(false);
    }

    let backup_content = std::fs::read_to_string(&backup_path)
        .map_err(|e| ConfigWriteError::IoError(format!("Failed to read backup: {}", e)))?;

    // Validate backup before restoring
    let validation_result = validate_typescript(&backup_content, bun_path);
    match &validation_result {
        Ok(()) => {}
        Err(bun_err) => {
            if bun_err.starts_with("Failed to run bun") {
                validate_structure(&backup_content).map_err(|e| {
                    ConfigWriteError::ValidationFailed(format!("Backup is also invalid: {}", e))
                })?;
            } else {
                return Err(ConfigWriteError::ValidationFailed(format!(
                    "Backup is also invalid: {}",
                    bun_err
                )));
            }
        }
    }

    // Atomic write of backup content
    atomic_write_with_secure_tempfile(config_path, &backup_content)?;

    tracing::info!(
        path = %config_path.display(),
        backup = %backup_path.display(),
        "Config restored from backup"
    );

    Ok(true)
}

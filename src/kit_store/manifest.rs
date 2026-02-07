//! Kit manifest parsing from repository metadata files.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::KitManifest;

const KIT_MANIFEST_FILE: &str = "kit.json";
const PACKAGE_MANIFEST_FILE: &str = "package.json";

/// Parse kit metadata from `kit.json` or `package.json` in a repository root.
///
/// Lookup order:
/// 1. `kit.json`
/// 2. `package.json`
pub fn parse_kit_manifest(repo_path: &Path) -> Result<KitManifest> {
    let kit_manifest_path = repo_path.join(KIT_MANIFEST_FILE);
    if kit_manifest_path.exists() {
        return parse_kit_json_manifest(&kit_manifest_path);
    }

    let package_manifest_path = repo_path.join(PACKAGE_MANIFEST_FILE);
    if package_manifest_path.exists() {
        return parse_package_json_manifest(&package_manifest_path);
    }

    Err(anyhow!(
        "No kit manifest found for repository at {} (checked {} and {})",
        repo_path.display(),
        kit_manifest_path.display(),
        package_manifest_path.display()
    ))
}

fn parse_kit_json_manifest(path: &Path) -> Result<KitManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read kit manifest file: {}", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse kit manifest JSON: {}", path.display()))
}

fn parse_package_json_manifest(path: &Path) -> Result<KitManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read package manifest file: {}", path.display()))?;

    let value: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse package manifest JSON: {}", path.display()))?;

    let fallback_name = path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    let scripts = extract_string_list(&value, "scripts");
    let scriptlets = extract_string_list(&value, "scriptlets");

    Ok(KitManifest {
        name: extract_string_field(&value, "name").unwrap_or(fallback_name),
        description: extract_string_field(&value, "description").unwrap_or_default(),
        author: extract_author(&value),
        version: extract_string_field(&value, "version").unwrap_or_default(),
        repo_url: extract_repo_url(&value),
        scripts,
        scriptlets,
    })
}

fn extract_string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn extract_author(value: &Value) -> String {
    match value.get("author") {
        Some(Value::String(author)) => author.clone(),
        Some(Value::Object(author_map)) => author_map
            .get("name")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn extract_repo_url(value: &Value) -> String {
    if let Some(repo_url) = extract_string_field(value, "repo_url") {
        return repo_url;
    }

    if let Some(repo_url) = extract_string_field(value, "repoUrl") {
        return repo_url;
    }

    match value.get("repository") {
        Some(Value::String(repo)) => repo.to_string(),
        Some(Value::Object(repository_map)) => repository_map
            .get("url")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn extract_string_list(value: &Value, key: &str) -> Vec<String> {
    match value.get(key) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect(),
        Some(Value::Object(object_items)) => object_items.keys().cloned().collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_json(path: &Path, json: &str) {
        fs::write(path, json).expect("json should write");
    }

    #[test]
    fn test_parse_kit_manifest_prefers_kit_json_when_both_files_exist() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let repo_path = temp_dir.path();

        write_json(
            &repo_path.join("kit.json"),
            r#"{
                "name": "kit-json-name",
                "description": "kit description",
                "author": "Kit Author",
                "version": "1.0.0",
                "repo_url": "https://github.com/example/kit-json",
                "scripts": ["one.ts", "two.ts"],
                "scriptlets": ["alpha.md"]
            }"#,
        );

        write_json(
            &repo_path.join("package.json"),
            r#"{
                "name": "package-json-name"
            }"#,
        );

        let manifest = parse_kit_manifest(repo_path).expect("manifest should parse");
        assert_eq!(manifest.name, "kit-json-name");
        assert_eq!(manifest.repo_url, "https://github.com/example/kit-json");
        assert_eq!(manifest.scripts, vec!["one.ts", "two.ts"]);
        assert_eq!(manifest.scriptlets, vec!["alpha.md"]);
    }

    #[test]
    fn test_parse_kit_manifest_uses_package_json_when_kit_json_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let repo_path = temp_dir.path();

        write_json(
            &repo_path.join("package.json"),
            r#"{
                "name": "main-kit",
                "description": "Main scripts",
                "author": {"name": "Script Kit"},
                "version": "2.1.0",
                "repository": {"url": "https://github.com/example/main-kit"},
                "scripts": ["first.ts", "second.ts"],
                "scriptlets": ["note.md", "demo.md"]
            }"#,
        );

        let manifest = parse_kit_manifest(repo_path).expect("manifest should parse");
        assert_eq!(manifest.name, "main-kit");
        assert_eq!(manifest.description, "Main scripts");
        assert_eq!(manifest.author, "Script Kit");
        assert_eq!(manifest.version, "2.1.0");
        assert_eq!(manifest.repo_url, "https://github.com/example/main-kit");
        assert_eq!(manifest.scripts, vec!["first.ts", "second.ts"]);
        assert_eq!(manifest.scriptlets, vec!["note.md", "demo.md"]);
    }

    #[test]
    fn test_parse_kit_manifest_parses_repository_string_and_script_object_keys() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let repo_path = temp_dir.path();

        write_json(
            &repo_path.join("package.json"),
            r#"{
                "name": "tools",
                "repository": "https://github.com/example/tools",
                "scripts": {
                    "install": "bun install",
                    "build": "bun run build"
                }
            }"#,
        );

        let manifest = parse_kit_manifest(repo_path).expect("manifest should parse");
        assert_eq!(manifest.name, "tools");
        assert_eq!(manifest.repo_url, "https://github.com/example/tools");
        assert_eq!(manifest.scripts.len(), 2);
        assert!(manifest.scripts.contains(&"install".to_string()));
        assert!(manifest.scripts.contains(&"build".to_string()));
    }

    #[test]
    fn test_parse_kit_manifest_errors_when_no_manifest_files_exist() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let repo_path = temp_dir.path();

        let result = parse_kit_manifest(repo_path);
        assert!(result.is_err());
    }
}

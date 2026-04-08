//! Git operations for installing, updating, and removing kit repositories.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tracing::info;

/// Clone a kit repository into the canonical plugin root `<kit_path>/kit/<plugin-id>/`.
///
/// After cloning, writes a `plugin.json` manifest if one does not already exist in
/// the repository, so the plugin is immediately discoverable by `discover_plugins()`.
pub fn install_kit(repo_url: &str) -> Result<(String, PathBuf), String> {
    let repo_name = extract_repo_name(repo_url)?;
    let plugins_root = crate::plugins::plugins_container_dir();
    let clone_path = plugins_root.join(&repo_name);

    fs::create_dir_all(&plugins_root).map_err(|err| {
        format!(
            "Failed to create plugins directory at '{}': {}",
            plugins_root.display(),
            err
        )
    })?;

    if clone_path.exists() {
        return Err(format!(
            "Plugin '{}' is already installed at {}",
            repo_name,
            clone_path.display()
        ));
    }

    let output = build_git_clone_command(repo_url, &clone_path)
        .output()
        .map_err(|err| format!("Failed to run 'git clone': {}", err))?;

    if !output.status.success() {
        return Err(format!(
            "git clone failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let manifest = crate::plugins::read_plugin_manifest(&clone_path)
        .map_err(|err| format!("Failed to resolve plugin manifest after clone: {}", err))?;
    let plugin_id = sanitize_plugin_id(&manifest.id)?;
    let target_path = plugins_root.join(&plugin_id);

    if target_path != clone_path {
        if target_path.exists() {
            let _ = fs::remove_dir_all(&clone_path);
            return Err(format!(
                "Plugin '{}' is already installed at {}",
                plugin_id,
                target_path.display()
            ));
        }

        fs::rename(&clone_path, &target_path).map_err(|err| {
            format!(
                "Failed to move cloned plugin into canonical path '{}': {}",
                target_path.display(),
                err
            )
        })?;
    }

    // Write plugin.json if the repo doesn't already have one, so the plugin is
    // immediately visible to discover_plugins().
    write_plugin_json_if_missing(&target_path, &plugin_id, repo_url);

    info!(
        plugin_id = %plugin_id,
        install_path = %target_path.display(),
        "plugin_installed"
    );

    Ok((plugin_id, target_path))
}

/// Pull latest changes for an installed kit.
pub fn update_kit(kit_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(kit_path)
        .arg("pull")
        .arg("--ff-only")
        .output()
        .map_err(|err| {
            format!(
                "Failed to run 'git -C {} pull --ff-only': {}",
                kit_path, err
            )
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git pull failed for '{}': {}",
            kit_path,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

/// Remove an installed kit directory.
pub fn remove_kit(kit_path: &str) -> Result<(), String> {
    fs::remove_dir_all(kit_path)
        .map_err(|err| format!("Failed to remove kit directory '{}': {}", kit_path, err))
}

/// Read the current git HEAD hash from a repository.
pub fn git_head_hash(repo_path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .map_err(|err| format!("Failed to run git rev-parse: {}", err))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "git rev-parse failed with status {}{}",
            output.status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {}", stderr)
            }
        ));
    }
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() {
        return Err("git rev-parse returned empty hash".to_string());
    }
    Ok(hash)
}

/// Write a synthetic `plugin.json` if the plugin root does not already contain one.
fn write_plugin_json_if_missing(plugin_root: &Path, plugin_id: &str, repo_url: &str) {
    let manifest_path = plugin_root.join("plugin.json");
    if manifest_path.exists() {
        return;
    }

    let manifest = serde_json::json!({
        "id": plugin_id,
        "title": plugin_id,
        "repoUrl": repo_url,
    });

    if let Ok(content) = serde_json::to_string_pretty(&manifest) {
        let _ = fs::write(&manifest_path, content);
    }
}

fn build_git_clone_command(repo_url: &str, target_path: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("clone")
        // Guard against option injection when repo URL starts with '-'.
        .arg("--")
        .arg(repo_url)
        .arg(target_path);
    command
}

fn sanitize_plugin_id(plugin_id: &str) -> Result<String, String> {
    let trimmed = plugin_id.trim();
    if trimmed.is_empty() {
        return Err("Plugin manifest id cannot be empty".to_string());
    }

    if has_path_traversal_segment(trimmed) {
        return Err(format!(
            "Invalid plugin id '{}' from manifest: path traversal is not allowed",
            plugin_id
        ));
    }

    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(format!(
            "Invalid plugin id '{}' from manifest: path separators are not allowed",
            plugin_id
        ));
    }

    if is_windows_reserved_device_name(trimmed) {
        return Err(format!(
            "Invalid plugin id '{}' from manifest: reserved device names are not allowed",
            plugin_id
        ));
    }

    Ok(trimmed.to_string())
}

fn extract_repo_name(repo_url: &str) -> Result<String, String> {
    let trimmed = repo_url.trim();
    if trimmed.is_empty() {
        return Err("Repository URL cannot be empty".to_string());
    }

    let without_query = trimmed.split('?').next().unwrap_or(trimmed);
    let without_fragment = without_query.split('#').next().unwrap_or(without_query);
    let without_trailing_slash = without_fragment.trim_end_matches('/');
    let without_git_suffix = without_trailing_slash
        .strip_suffix(".git")
        .unwrap_or(without_trailing_slash);

    if has_path_traversal_segment(without_git_suffix) {
        return Err(format!(
            "Invalid repository URL '{}': path traversal is not allowed",
            repo_url
        ));
    }

    let candidate = without_git_suffix
        .rsplit(['/', ':'])
        .next()
        .unwrap_or_default()
        .trim();

    if candidate.is_empty() {
        return Err(format!(
            "Could not extract repository name from '{}'",
            repo_url
        ));
    }

    if candidate.contains("..") {
        return Err(format!(
            "Invalid repository name '{}' extracted from '{}': path traversal is not allowed",
            candidate, repo_url
        ));
    }

    if candidate.contains('/') || candidate.contains('\\') {
        return Err(format!(
            "Invalid repository name '{}' extracted from '{}': path separators are not allowed",
            candidate, repo_url
        ));
    }

    if is_windows_reserved_device_name(candidate) {
        return Err(format!(
            "Invalid repository name '{}' extracted from '{}': reserved device names are not allowed",
            candidate, repo_url
        ));
    }

    Ok(candidate.to_string())
}

fn has_path_traversal_segment(input: &str) -> bool {
    input.split(['/', '\\', ':']).any(|segment| segment == "..")
}

fn is_windows_reserved_device_name(name: &str) -> bool {
    let trimmed = name.trim_end_matches([' ', '.']);
    if trimmed.is_empty() {
        return false;
    }

    let base = trimmed.split('.').next().unwrap_or_default();
    matches!(
        base.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

#[cfg(test)]
mod tests {
    use super::{build_git_clone_command, extract_repo_name, install_kit};
    use crate::setup::SK_PATH_ENV;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::Mutex;

    static SK_PATH_LOCK: Mutex<()> = Mutex::new(());

    fn create_bare_repo_with_package_name(
        dir: &Path,
        repo_name: &str,
        package_name: &str,
    ) -> PathBuf {
        let repo_path = dir.join(format!("{}.git", repo_name));
        fs::create_dir_all(&repo_path).expect("create bare repo dir");

        let output = Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&repo_path)
            .output()
            .expect("git init --bare");
        assert!(output.status.success(), "git init --bare failed");

        let work_path = dir.join(format!("{}-work", repo_name));
        let clone_output = Command::new("git")
            .arg("clone")
            .arg(&repo_path)
            .arg(&work_path)
            .output()
            .expect("git clone for work tree");
        assert!(clone_output.status.success(), "git clone work tree failed");

        let package_json = serde_json::json!({
            "name": package_name,
            "description": format!("Test plugin {}", package_name),
            "version": "1.0.0"
        });
        fs::write(
            work_path.join("package.json"),
            serde_json::to_string_pretty(&package_json).expect("serialize"),
        )
        .expect("write package.json");

        fs::create_dir_all(work_path.join("scripts")).expect("create scripts dir");
        fs::write(
            work_path.join("scripts").join("hello.ts"),
            "// Test script\nconsole.log('hello')\n",
        )
        .expect("write hello.ts");

        for (key, val) in [("user.email", "test@test.com"), ("user.name", "Test")] {
            Command::new("git")
                .arg("-C")
                .arg(&work_path)
                .arg("config")
                .arg(key)
                .arg(val)
                .output()
                .expect("git config");
        }

        Command::new("git")
            .arg("-C")
            .arg(&work_path)
            .arg("add")
            .arg("-A")
            .output()
            .expect("git add");

        let commit_output = Command::new("git")
            .arg("-C")
            .arg(&work_path)
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .output()
            .expect("git commit");
        assert!(commit_output.status.success(), "git commit failed");

        let push_output = Command::new("git")
            .arg("-C")
            .arg(&work_path)
            .arg("push")
            .output()
            .expect("git push");
        assert!(push_output.status.success(), "git push failed");

        repo_path
    }

    #[test]
    fn test_extract_repo_name_from_https_url() {
        let name = extract_repo_name("https://github.com/user/my-kit").expect("should parse");
        assert_eq!(name, "my-kit");
    }

    #[test]
    fn test_extract_repo_name_from_ssh_url_with_git_suffix() {
        let name =
            extract_repo_name("git@github.com:user/my-kit.git").expect("should parse git ssh");
        assert_eq!(name, "my-kit");
    }

    #[test]
    fn test_extract_repo_name_trims_trailing_slash_and_query() {
        let name =
            extract_repo_name("https://github.com/user/my-kit/?tab=readme#section").expect("url");
        assert_eq!(name, "my-kit");
    }

    #[test]
    fn test_extract_repo_name_rejects_empty_url() {
        let err = extract_repo_name("   ").expect_err("empty url should fail");
        assert!(err.contains("cannot be empty"));
    }

    #[test]
    fn test_extract_repo_name_rejects_path_traversal_sequences() {
        let err =
            extract_repo_name("https://github.com/user/../evil-kit").expect_err("should reject");
        assert!(err.contains("path traversal"));
    }

    #[test]
    fn test_extract_repo_name_rejects_path_separators() {
        let err =
            extract_repo_name("https://github.com/user/kit\\evil").expect_err("should reject");
        assert!(err.contains("path separators"));
    }

    #[test]
    fn install_kit_uses_manifest_id_for_canonical_path() {
        let _lock = SK_PATH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let kit_root = temp_dir.path().join("sk-test");
        let repos_dir = temp_dir.path().join("repos");
        fs::create_dir_all(&repos_dir).expect("create repos dir");
        fs::create_dir_all(kit_root.join("kit")).expect("create kit dir");

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().expect("path str"));

        let bare_repo =
            create_bare_repo_with_package_name(&repos_dir, "repo-name", "manifest-plugin");
        let repo_url = bare_repo.to_str().expect("repo url str");

        let (plugin_id, install_path) = install_kit(repo_url).expect("install should succeed");

        assert_eq!(plugin_id, "manifest-plugin");
        assert_eq!(install_path, kit_root.join("kit").join("manifest-plugin"));
        assert!(install_path.exists(), "canonical plugin root should exist");
        assert!(
            !kit_root.join("kit").join("repo-name").exists(),
            "repo basename path should not remain after canonicalization"
        );

        std::env::remove_var(SK_PATH_ENV);
    }

    #[test]
    fn test_extract_repo_name_rejects_windows_reserved_device_names() {
        let err = extract_repo_name("https://github.com/user/CON.git").expect_err("should reject");
        assert!(err.contains("reserved device names"));
    }

    #[test]
    fn test_build_git_clone_command_inserts_double_dash_before_repo_url() {
        let command = build_git_clone_command("-unsafe-url", Path::new("/tmp/kit"));
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();

        assert_eq!(args[0], "clone");
        assert_eq!(args[1], "--");
        assert_eq!(args[2], "-unsafe-url");
        assert_eq!(args[3], "/tmp/kit");
    }
}

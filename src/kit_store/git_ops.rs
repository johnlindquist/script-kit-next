//! Git operations for installing, updating, and removing kit repositories.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Clone a kit repository into `~/.scriptkit/kits/<kit-name>`.
pub fn install_kit(repo_url: &str) -> Result<String, String> {
    let kit_name = extract_repo_name(repo_url)?;
    let kits_root = kits_root_path();
    let target_path = kits_root.join(&kit_name);

    fs::create_dir_all(&kits_root).map_err(|err| {
        format!(
            "Failed to create kits directory at '{}': {}",
            kits_root.display(),
            err
        )
    })?;

    let output = build_git_clone_command(repo_url, &target_path)
        .output()
        .map_err(|err| format!("Failed to run 'git clone': {}", err))?;

    if !output.status.success() {
        return Err(format!(
            "git clone failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(kit_name)
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

fn kits_root_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "~".to_string());
    PathBuf::from(home).join(".scriptkit").join("kits")
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
    use super::{build_git_clone_command, extract_repo_name};
    use std::path::Path;

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

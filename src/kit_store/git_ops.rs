//! Git operations for installing, updating, and removing kits.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{anyhow, Context, Result};

use crate::setup::get_kit_path;

use super::InstalledKit;

const KIT_STORE_GIT_OP_INSTALL: &str = "kit_store.git.install";
const KIT_STORE_GIT_OP_UPDATE: &str = "kit_store.git.update";
const KIT_STORE_GIT_OP_REMOVE: &str = "kit_store.git.remove";
const KIT_STORE_GIT_OP_CHECK_UPDATES: &str = "kit_store.git.check_updates";

/// Clone a kit repository into `~/.scriptkit/kits/<kit-name>`.
pub fn install_kit(repo_url: &str) -> Result<InstalledKit> {
    install_kit_at_root(repo_url, &get_kit_path())
}

/// Pull latest changes for an installed kit.
pub fn update_kit(name: &str) -> Result<()> {
    update_kit_at_root(name, &get_kit_path())
}

/// Remove an installed kit directory.
pub fn remove_kit(name: &str) -> Result<()> {
    remove_kit_at_root(name, &get_kit_path())
}

/// Check whether an installed kit has updates available from origin.
pub fn check_for_updates(name: &str) -> Result<bool> {
    check_for_updates_at_root(name, &get_kit_path())
}

fn install_kit_at_root(repo_url: &str, scriptkit_root: &Path) -> Result<InstalledKit> {
    let name = derive_kit_name(repo_url)?;
    let kits_root = kits_root(scriptkit_root);
    fs::create_dir_all(&kits_root).with_context(|| {
        format!(
            "Failed to prepare kits directory for {} at {}",
            KIT_STORE_GIT_OP_INSTALL,
            kits_root.display()
        )
    })?;

    let install_path = kits_root.join(&name);
    if install_path.exists() {
        return Err(anyhow!(
            "Cannot install kit for {} because destination already exists: {}",
            KIT_STORE_GIT_OP_INSTALL,
            install_path.display()
        ));
    }

    let mut clone_cmd = Command::new("git");
    clone_cmd.arg("clone").arg(repo_url).arg(&install_path);
    run_git_command(&mut clone_cmd, KIT_STORE_GIT_OP_INSTALL)?;

    let git_hash = current_git_hash(&install_path)?;
    let installed_at = chrono::Utc::now().to_rfc3339();

    Ok(InstalledKit {
        name,
        path: install_path,
        repo_url: repo_url.to_string(),
        git_hash,
        installed_at,
    })
}

fn update_kit_at_root(name: &str, scriptkit_root: &Path) -> Result<()> {
    let name = validated_kit_name(name, KIT_STORE_GIT_OP_UPDATE)?;
    let kit_path = installed_kit_path(name, scriptkit_root);
    ensure_kit_exists(name, &kit_path, KIT_STORE_GIT_OP_UPDATE)?;

    let mut pull_cmd = Command::new("git");
    pull_cmd.arg("-C").arg(&kit_path).arg("pull");
    run_git_command(&mut pull_cmd, KIT_STORE_GIT_OP_UPDATE)?;

    Ok(())
}

fn remove_kit_at_root(name: &str, scriptkit_root: &Path) -> Result<()> {
    let name = validated_kit_name(name, KIT_STORE_GIT_OP_REMOVE)?;
    let kit_path = installed_kit_path(name, scriptkit_root);
    if !kit_path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&kit_path).with_context(|| {
        format!(
            "Failed to remove kit directory for {}: {}",
            KIT_STORE_GIT_OP_REMOVE,
            kit_path.display()
        )
    })?;

    Ok(())
}

fn check_for_updates_at_root(name: &str, scriptkit_root: &Path) -> Result<bool> {
    let name = validated_kit_name(name, KIT_STORE_GIT_OP_CHECK_UPDATES)?;
    let kit_path = installed_kit_path(name, scriptkit_root);
    ensure_kit_exists(name, &kit_path, KIT_STORE_GIT_OP_CHECK_UPDATES)?;

    let mut fetch_cmd = Command::new("git");
    fetch_cmd
        .arg("-C")
        .arg(&kit_path)
        .arg("fetch")
        .arg("--dry-run");
    let output = run_git_command(&mut fetch_cmd, KIT_STORE_GIT_OP_CHECK_UPDATES)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(fetch_dry_run_reports_updates(&stdout, &stderr))
}

fn run_git_command(command: &mut Command, operation: &str) -> Result<Output> {
    let output = command.output().with_context(|| {
        format!(
            "Failed to spawn git command for operation {}. command={:?}",
            operation, command
        )
    })?;

    if output.status.success() {
        return Ok(output);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Err(anyhow!(
        "Git command failed for operation {} with status {}. stdout='{}' stderr='{}'",
        operation,
        output.status,
        stdout.trim(),
        stderr.trim()
    ))
}

fn current_git_hash(repo_path: &Path) -> Result<String> {
    let mut rev_parse_cmd = Command::new("git");
    rev_parse_cmd
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("HEAD");
    let output = run_git_command(&mut rev_parse_cmd, KIT_STORE_GIT_OP_INSTALL)?;
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if hash.is_empty() {
        return Err(anyhow!(
            "Git hash resolution produced empty output for repository at {}",
            repo_path.display()
        ));
    }

    Ok(hash)
}

fn kits_root(scriptkit_root: &Path) -> PathBuf {
    scriptkit_root.join("kits")
}

fn installed_kit_path(name: &str, scriptkit_root: &Path) -> PathBuf {
    kits_root(scriptkit_root).join(name)
}

fn ensure_kit_exists(name: &str, kit_path: &Path, operation: &str) -> Result<()> {
    if kit_path.exists() {
        return Ok(());
    }

    Err(anyhow!(
        "Kit '{}' not found for {} at {}",
        name,
        operation,
        kit_path.display()
    ))
}

fn validated_kit_name<'a>(name: &'a str, operation: &str) -> Result<&'a str> {
    let trimmed = name.trim();
    if is_safe_kit_name_segment(trimmed) {
        return Ok(trimmed);
    }

    Err(anyhow!(
        "Invalid kit name '{}' for {}. Kit names must be a single path segment",
        name,
        operation
    ))
}

fn derive_kit_name(repo_url: &str) -> Result<String> {
    let trimmed = repo_url.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Cannot derive kit name from empty repository URL"));
    }

    let without_query = trimmed.split('?').next().unwrap_or(trimmed);
    let without_fragment = without_query.split('#').next().unwrap_or(without_query);

    let normalized = match without_fragment.rsplit_once(':') {
        Some((_, remainder)) if without_fragment.contains('@') && remainder.contains('/') => {
            remainder
        }
        _ => without_fragment,
    };

    let normalized = normalized.trim_end_matches('/');
    let normalized = normalized.strip_suffix(".git").unwrap_or(normalized);
    let name = normalized.rsplit('/').next().unwrap_or_default().trim();

    if name.is_empty() {
        return Err(anyhow!(
            "Cannot derive kit name from repository URL '{}'",
            repo_url
        ));
    }
    if !is_safe_kit_name_segment(name) {
        return Err(anyhow!(
            "Derived kit name '{}' is unsafe for repository URL '{}'",
            name,
            repo_url
        ));
    }

    Ok(name.to_string())
}

fn is_safe_kit_name_segment(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\\')
}

fn fetch_dry_run_reports_updates(stdout: &str, stderr: &str) -> bool {
    stdout.lines().chain(stderr.lines()).any(|line| {
        let line = line.trim();
        if line.is_empty() {
            return false;
        }
        if line.starts_with("From ") {
            return false;
        }
        if line.contains("[up to date]") {
            return false;
        }

        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_git(args: &[&str], cwd: Option<&Path>) -> Output {
        let mut cmd = Command::new("git");
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd.output().expect("git command should execute");
        if !output.status.success() {
            panic!(
                "git {:?} failed with status {}. stdout='{}' stderr='{}'",
                args,
                output.status,
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }

        output
    }

    fn configure_git_user(repo_path: &Path) {
        run_git(&["config", "user.name", "Kit Store Test"], Some(repo_path));
        run_git(
            &["config", "user.email", "kit-store-test@example.com"],
            Some(repo_path),
        );
    }

    #[test]
    fn test_derive_kit_name_returns_repo_slug_for_common_url_formats() {
        let https_name =
            derive_kit_name("https://github.com/johnlindquist/my-kit.git").expect("name");
        assert_eq!(https_name, "my-kit");

        let ssh_name = derive_kit_name("git@github.com:johnlindquist/my-kit.git").expect("name");
        assert_eq!(ssh_name, "my-kit");

        let shorthand_name = derive_kit_name("johnlindquist/my-kit").expect("name");
        assert_eq!(shorthand_name, "my-kit");
    }

    #[test]
    fn test_derive_kit_name_errors_when_url_is_empty() {
        let result = derive_kit_name("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_kit_name_rejects_path_traversal_segments() {
        let dot_dot = derive_kit_name("https://github.com/example/..");
        assert!(dot_dot.is_err());

        let dot = derive_kit_name("https://github.com/example/.");
        assert!(dot.is_err());
    }

    #[test]
    fn test_fetch_dry_run_reports_updates_when_output_contains_ref_changes() {
        let has_updates = fetch_dry_run_reports_updates(
            "",
            "From /tmp/my-kit\n   abcdef0..1234567  main       -> origin/main",
        );
        assert!(has_updates);

        let no_updates =
            fetch_dry_run_reports_updates("", "From /tmp/my-kit\n = [up to date]      main");
        assert!(!no_updates);
    }

    #[test]
    fn test_operations_reject_path_traversal_name_inputs() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let scriptkit_root = temp_dir.path().join("scriptkit-root");

        assert!(update_kit_at_root("../escape", &scriptkit_root).is_err());
        assert!(remove_kit_at_root("../escape", &scriptkit_root).is_err());
        assert!(check_for_updates_at_root("../escape", &scriptkit_root).is_err());
    }

    #[test]
    fn test_install_update_remove_and_check_for_updates_when_remote_changes() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let scriptkit_root = temp_dir.path().join("scriptkit-root");

        let seed_repo = temp_dir.path().join("seed");
        fs::create_dir_all(&seed_repo).expect("seed repo dir should create");
        run_git(&["init"], Some(&seed_repo));
        configure_git_user(&seed_repo);

        let readme_path = seed_repo.join("README.md");
        fs::write(&readme_path, "initial\n").expect("initial file should write");
        run_git(&["add", "README.md"], Some(&seed_repo));
        run_git(&["commit", "-m", "initial"], Some(&seed_repo));

        let remote_repo = temp_dir.path().join("my-kit.git");
        let remote_repo_str = remote_repo.to_string_lossy().to_string();
        run_git(&["init", "--bare", remote_repo_str.as_str()], None);
        run_git(
            &["remote", "add", "origin", remote_repo_str.as_str()],
            Some(&seed_repo),
        );
        run_git(&["push", "-u", "origin", "HEAD"], Some(&seed_repo));

        let installed = install_kit_at_root(remote_repo_str.as_str(), &scriptkit_root)
            .expect("install should succeed");
        assert_eq!(installed.name, "my-kit");
        assert!(installed.path.exists());
        assert!(!installed.git_hash.is_empty());
        assert_eq!(installed.repo_url, remote_repo_str);

        let has_updates = check_for_updates_at_root("my-kit", &scriptkit_root)
            .expect("check updates should succeed");
        assert!(!has_updates);

        let publisher_repo = temp_dir.path().join("publisher");
        let publisher_repo_str = publisher_repo.to_string_lossy().to_string();
        run_git(
            &[
                "clone",
                remote_repo_str.as_str(),
                publisher_repo_str.as_str(),
            ],
            None,
        );
        configure_git_user(&publisher_repo);

        fs::write(publisher_repo.join("README.md"), "updated\n")
            .expect("updated file should write");
        run_git(&["add", "README.md"], Some(&publisher_repo));
        run_git(&["commit", "-m", "update"], Some(&publisher_repo));
        run_git(&["push", "origin", "HEAD"], Some(&publisher_repo));

        let has_updates = check_for_updates_at_root("my-kit", &scriptkit_root)
            .expect("check updates after push should succeed");
        assert!(has_updates);

        update_kit_at_root("my-kit", &scriptkit_root).expect("pull should succeed");
        let has_updates_after_pull = check_for_updates_at_root("my-kit", &scriptkit_root)
            .expect("check updates after pull should succeed");
        assert!(!has_updates_after_pull);

        let updated_hash = current_git_hash(&installed.path).expect("hash should resolve");
        assert_ne!(updated_hash, installed.git_hash);

        remove_kit_at_root("my-kit", &scriptkit_root).expect("remove should succeed");
        assert!(!installed.path.exists());
    }
}

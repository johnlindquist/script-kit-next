//! Integration tests verifying that kit store install, update, and removal
//! operate on the canonical plugin root (`<kit_path>/kit/<plugin-id>/`) and
//! that freshly installed plugins are immediately discoverable.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use script_kit_gpui::kit_store::git_ops;
use script_kit_gpui::kit_store::storage;
use script_kit_gpui::kit_store::InstalledKit;
use script_kit_gpui::plugins::discover_plugins_in;
use script_kit_gpui::setup::SK_PATH_ENV;

/// Shared lock for SK_PATH env var mutation.
/// Integration tests run in the same process, so env var changes are global.
static SK_PATH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Create a bare git repo that can be cloned locally.
fn create_bare_repo(dir: &std::path::Path, name: &str) -> PathBuf {
    let repo_path = dir.join(format!("{}.git", name));
    fs::create_dir_all(&repo_path).expect("create bare repo dir");

    let output = Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg(&repo_path)
        .output()
        .expect("git init --bare");
    assert!(output.status.success(), "git init --bare failed");

    // Create a temporary clone, commit something, and push back to bare repo.
    let work_path = dir.join(format!("{}-work", name));
    let clone_output = Command::new("git")
        .arg("clone")
        .arg(&repo_path)
        .arg(&work_path)
        .output()
        .expect("git clone for work tree");
    assert!(clone_output.status.success(), "git clone work tree failed");

    // Write a package.json and a script so the plugin has discoverable content.
    let package_json = serde_json::json!({
        "name": name,
        "description": format!("Test plugin {}", name),
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

    // Configure git user for the commit.
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

/// Create a bare git repo whose package.json name differs from the repo basename.
fn create_bare_repo_with_package_name(
    dir: &std::path::Path,
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

/// Run a test with SK_PATH pointing to a fresh temp workspace.
fn with_temp_workspace<F: FnOnce(&std::path::Path, &std::path::Path)>(f: F) {
    let _lock = SK_PATH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let kit_root = temp_dir.path().join("sk-test");
    let repos_dir = temp_dir.path().join("repos");
    fs::create_dir_all(&repos_dir).expect("create repos dir");

    // Create the plugin container directory.
    fs::create_dir_all(kit_root.join("kit")).expect("create kit dir");

    std::env::set_var(SK_PATH_ENV, kit_root.to_str().expect("path str"));
    f(&kit_root, &repos_dir);
    std::env::remove_var(SK_PATH_ENV);
}

// ── Install ────────────────────────────────────────────────────────────────

#[test]
fn install_clones_into_plugin_root() {
    with_temp_workspace(|kit_root, repos_dir| {
        let bare_repo = create_bare_repo(repos_dir, "test-plugin");
        let repo_url = bare_repo.to_str().expect("repo url str");

        let (plugin_id, install_path) =
            git_ops::install_kit(repo_url).expect("install should succeed");

        assert_eq!(plugin_id, "test-plugin");
        assert_eq!(install_path, kit_root.join("kit").join("test-plugin"));
        assert!(install_path.exists(), "install path should exist on disk");
    });
}

#[test]
fn install_writes_plugin_json_when_repo_lacks_one() {
    with_temp_workspace(|_kit_root, repos_dir| {
        let bare_repo = create_bare_repo(repos_dir, "no-manifest");
        let repo_url = bare_repo.to_str().expect("repo url str");

        let (_plugin_id, install_path) =
            git_ops::install_kit(repo_url).expect("install should succeed");

        let plugin_json_path = install_path.join("plugin.json");
        assert!(
            plugin_json_path.exists(),
            "plugin.json should be synthesized after install"
        );

        let content = fs::read_to_string(&plugin_json_path).expect("read plugin.json");
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("plugin.json should be valid JSON");
        assert_eq!(parsed["id"], "no-manifest");
    });
}

#[test]
fn install_rejects_duplicate_plugin_id() {
    with_temp_workspace(|_kit_root, repos_dir| {
        let bare_repo = create_bare_repo(repos_dir, "dup-plugin");
        let repo_url = bare_repo.to_str().expect("repo url str");

        git_ops::install_kit(repo_url).expect("first install should succeed");

        let err = git_ops::install_kit(repo_url).expect_err("second install should fail");
        assert!(
            err.contains("already installed"),
            "error should mention already installed: {}",
            err
        );
    });
}

#[test]
fn install_uses_manifest_id_as_canonical_plugin_root() {
    with_temp_workspace(|kit_root, repos_dir| {
        let bare_repo =
            create_bare_repo_with_package_name(repos_dir, "repo-name", "manifest-plugin");
        let repo_url = bare_repo.to_str().expect("repo url str");

        let (plugin_id, install_path) =
            git_ops::install_kit(repo_url).expect("install should succeed");

        assert_eq!(plugin_id, "manifest-plugin");
        assert_eq!(install_path, kit_root.join("kit").join("manifest-plugin"));
        assert!(install_path.exists(), "canonical plugin root should exist");
        assert!(
            !kit_root.join("kit").join("repo-name").exists(),
            "repo basename path should not remain after canonicalization"
        );
    });
}

// ── Discovery ──────────────────────────────────────────────────────────────

#[test]
fn installed_plugin_discoverable_without_extra_step() {
    with_temp_workspace(|kit_root, repos_dir| {
        let bare_repo = create_bare_repo(repos_dir, "discoverable");
        let repo_url = bare_repo.to_str().expect("repo url str");

        git_ops::install_kit(repo_url).expect("install should succeed");

        let plugins_container = kit_root.join("kit");
        let index =
            discover_plugins_in(&plugins_container).expect("discover_plugins should succeed");

        let found = index.plugins.iter().find(|p| p.id == "discoverable");
        assert!(
            found.is_some(),
            "installed plugin should be found by discover_plugins_in"
        );

        let plugin = found.expect("plugin");
        assert_eq!(plugin.root, plugins_container.join("discoverable"));
    });
}

// ── Registry Persistence ───────────────────────────────────────────────────

#[test]
fn registry_path_points_at_plugin_root() {
    with_temp_workspace(|kit_root, repos_dir| {
        let bare_repo = create_bare_repo(repos_dir, "reg-test");
        let repo_url = bare_repo.to_str().expect("repo url str");

        let (plugin_id, install_path) =
            git_ops::install_kit(repo_url).expect("install should succeed");

        let git_hash = git_ops::git_head_hash(&install_path).expect("git hash");

        let installed = InstalledKit {
            name: plugin_id.clone(),
            path: install_path.clone(),
            repo_url: repo_url.to_string(),
            git_hash,
            installed_at: "2026-04-08T00:00:00Z".to_string(),
        };

        storage::save_kit_registry(&[installed]).expect("save registry");

        let loaded = storage::list_installed_kits().expect("list installed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "reg-test");
        assert_eq!(loaded[0].path, kit_root.join("kit").join("reg-test"));
    });
}

// ── Update ─────────────────────────────────────────────────────────────────

#[test]
fn update_operates_on_plugin_root() {
    with_temp_workspace(|_kit_root, repos_dir| {
        let bare_repo = create_bare_repo(repos_dir, "updatable");
        let repo_url = bare_repo.to_str().expect("repo url str");

        let (_plugin_id, install_path) =
            git_ops::install_kit(repo_url).expect("install should succeed");

        let hash_before = git_ops::git_head_hash(&install_path).expect("hash before");

        // git pull --ff-only on an up-to-date repo should succeed (no-op).
        git_ops::update_kit(install_path.to_str().expect("path str"))
            .expect("update should succeed");

        let hash_after = git_ops::git_head_hash(&install_path).expect("hash after");
        assert_eq!(
            hash_before, hash_after,
            "hash should not change on no-op update"
        );
    });
}

// ── Removal ────────────────────────────────────────────────────────────────

#[test]
fn removal_deletes_plugin_root_and_cleans_registry() {
    with_temp_workspace(|kit_root, repos_dir| {
        let bare_repo = create_bare_repo(repos_dir, "removable");
        let repo_url = bare_repo.to_str().expect("repo url str");

        let (plugin_id, install_path) =
            git_ops::install_kit(repo_url).expect("install should succeed");

        let git_hash = git_ops::git_head_hash(&install_path).expect("git hash");

        let installed = InstalledKit {
            name: plugin_id.clone(),
            path: install_path.clone(),
            repo_url: repo_url.to_string(),
            git_hash,
            installed_at: "2026-04-08T00:00:00Z".to_string(),
        };
        storage::save_kit_registry(&[installed]).expect("save registry");

        // Remove the plugin.
        git_ops::remove_kit(install_path.to_str().expect("path str"))
            .expect("remove should succeed");
        storage::remove_kit(&plugin_id).expect("remove from registry");

        assert!(
            !install_path.exists(),
            "plugin directory should be removed from disk"
        );

        let loaded = storage::list_installed_kits().expect("list installed after removal");
        assert!(loaded.is_empty(), "registry should be empty after removal");

        // Plugin should no longer be discoverable.
        let plugins_container = kit_root.join("kit");
        let index =
            discover_plugins_in(&plugins_container).expect("discover_plugins should succeed");
        assert!(
            index.plugins.iter().all(|p| p.id != "removable"),
            "removed plugin should not be discoverable"
        );
    });
}

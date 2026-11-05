#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[cfg(unix)]
    #[test]
    fn setup_cli_leaves_a_conflicting_directory_private() {
        use std::os::unix::fs::PermissionsExt;

        with_sk_path(|root| {
            let target = root.join("bin/scriptkit");
            fs::create_dir_all(&target).unwrap();
            fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
            fs::write(target.join("keep"), b"unrelated contents").unwrap();

            let result = ensure_kit_setup();
            assert!(result.warnings.iter().any(|w| w.contains("bin/scriptkit")));
            assert_eq!(
                fs::metadata(&target).unwrap().permissions().mode() & 0o7777,
                0o700
            );
            assert_eq!(
                fs::read(target.join("keep")).unwrap(),
                b"unrelated contents"
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn setup_cli_repairs_only_its_own_command_and_keeps_a_ready_copy() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let scratch = TempDir::new().unwrap();
        let other = scratch.path().join("private copy");
        let target = scratch.path().join("scriptkit");
        fs::write(&other, b"command").unwrap();
        fs::set_permissions(&other, fs::Permissions::from_mode(0o600)).unwrap();
        fs::hard_link(&other, &target).unwrap();

        let mut warnings = Vec::new();
        write_executable_string_if_changed(&target, "command", &mut warnings, "bin/scriptkit");
        assert!(warnings.is_empty(), "{warnings:?}");
        let installed = fs::metadata(&target).unwrap();
        assert_eq!(installed.mode() & 0o7777, 0o755);
        assert_eq!(fs::read(&target).unwrap(), b"command");
        let original = fs::metadata(&other).unwrap();
        assert_eq!(original.mode() & 0o7777, 0o600);
        assert_ne!(original.ino(), installed.ino());

        write_executable_string_if_changed(&target, "command", &mut warnings, "bin/scriptkit");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(fs::metadata(&target).unwrap().ino(), installed.ino());
    }

    /// Run a test body while holding the shared SK_PATH lock.
    /// Automatically sets SK_PATH to `kit_root` and restores it on exit.
    pub(super) fn with_sk_path<F: FnOnce(&std::path::Path)>(f: F) {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().to_path_buf();
        struct RestoreSkPath(Option<std::ffi::OsString>);
        impl Drop for RestoreSkPath {
            fn drop(&mut self) {
                if let Some(previous) = &self.0 {
                    std::env::set_var(SK_PATH_ENV, previous);
                } else {
                    std::env::remove_var(SK_PATH_ENV);
                }
            }
        }
        let restore = RestoreSkPath(std::env::var_os(SK_PATH_ENV));
        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        f(&kit_root);

        drop(restore);
        drop(lock);
    }

    // The callback opens the real per-home startup log files, not a global
    // logger, process registry or GUI in the test runner's own home.
    fn disk_diagnostics(home: &std::path::Path) {
        use std::io::Write;
        let (all, latest) = crate::logging::session_files(&home.join(".scriptkit/logs"), 0);
        all.unwrap().write_all(b"new session\n").unwrap();
        latest.unwrap().write_all(b"new session\n").unwrap();
    }

    #[test]
    fn normal_workspace_startup_seeds_only_the_launch_that_creates_the_root() {
        with_sk_path(|home| {
            let kit = home.join(".scriptkit");
            std::env::set_var(SK_PATH_ENV, &kit);
            let launch = || setup_before_diagnostics(&kit, Some(home), || disk_diagnostics(home)).unwrap();
            let first = launch();
            assert!(first.is_fresh_install);
            assert!(first.warnings.is_empty(), "{:?}", first.warnings);
            let starter = kit.join("plugins/main/scripts/hello-world.ts");
            assert!(fs::read_to_string(&starter).unwrap().contains("Hello World"));
            fs::remove_file(&starter).unwrap();
            fs::write(kit.join("config.ts"), "export default { editorFontSize: 27 };\n").unwrap();
            let second = launch();
            assert!(!second.is_fresh_install);
            assert!(!starter.exists());
            assert_eq!(fs::read_to_string(kit.join("config.ts")).unwrap(), "export default { editorFontSize: 27 };\n");
        });
    }

    #[test]
    fn normal_workspace_startup_imports_scripts_and_keeps_the_v1_session_log() {
        with_sk_path(|home| {
            let kit = home.join(".scriptkit");
            std::env::set_var(SK_PATH_ENV, &kit);
            let legacy = home.join(".kenv");
            fs::create_dir_all(legacy.join("scripts")).unwrap();
            fs::create_dir(legacy.join("logs")).unwrap();
            fs::write(legacy.join("scripts/keep.ts"), "// my script").unwrap();
            fs::write(legacy.join("config.ts"), "export default { editorFontSize: 29 };\n").unwrap();
            fs::write(legacy.join("logs/latest-session.jsonl"), "v1 history\n").unwrap();
            let result = setup_before_diagnostics(&kit, Some(home), || disk_diagnostics(home)).unwrap();
            assert!(!result.is_fresh_install);
            assert!(result.warnings.is_empty(), "{:?}", result.warnings);
            assert_eq!(fs::read_to_string(kit.join("plugins/main/scripts/keep.ts")).unwrap(), "// my script");
            assert_eq!(fs::read_to_string(kit.join("config.ts")).unwrap(), "export default { editorFontSize: 29 };\n");
            assert_eq!(fs::read_to_string(kit.join("legacy-logs/latest-session.jsonl")).unwrap(), "v1 history\n");
            assert_eq!(fs::read_to_string(kit.join("logs/latest-session.jsonl")).unwrap(), "new session\n");
            #[cfg(unix)]
            assert_eq!(fs::read_link(legacy).unwrap(), kit);
        });
    }

    #[test]
    fn normal_workspace_startup_never_imports_into_an_existing_or_alternate_kit() {
        for alternate in [false, true] {
            with_sk_path(|home| {
                let default = home.join(".scriptkit");
                let kit = if alternate { home.join("another kit") } else { default.clone() };
                std::env::set_var(SK_PATH_ENV, &kit);
                let legacy = home.join(".kenv/scripts/old.ts");
                fs::create_dir_all(legacy.parent().unwrap()).unwrap();
                fs::write(&legacy, "// original").unwrap();
                if !alternate { fs::create_dir(&kit).unwrap(); }
                let result = setup_before_diagnostics(&kit, Some(home), || disk_diagnostics(home)).unwrap();
                assert_eq!(result.is_fresh_install, alternate);
                assert_eq!(fs::read_to_string(legacy).unwrap(), "// original");
                assert!(!kit.join("plugins/main/scripts/old.ts").exists());
                if alternate { assert!(!default.join("plugins").exists()); }
            });
        }
    }

    #[cfg(unix)]
    #[test]
    fn normal_workspace_startup_keeps_a_linked_legacy_tree_for_manual_import() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        with_sk_path(|home| {
            let kit = home.join(".scriptkit");
            std::env::set_var(SK_PATH_ENV, &kit);
            let legacy = home.join(".kenv");
            fs::create_dir_all(legacy.join("logs")).unwrap();
            let note = home.join("private.md");
            fs::write(&note, "keep this\n").unwrap();
            fs::set_permissions(&note, fs::Permissions::from_mode(0o640)).unwrap();
            let link = legacy.join("logs/latest-session.jsonl");
            fs::hard_link(&note, &link).unwrap();
            let result = setup_before_diagnostics(&kit, Some(home), || disk_diagnostics(home)).unwrap();
            assert!(result.is_fresh_install);
            assert!(result.warnings.iter().any(|w| w.contains("Legacy workspace left intact")));
            assert!(!legacy.is_symlink());
            assert_eq!(fs::metadata(link).unwrap().ino(), fs::metadata(&note).unwrap().ino());
            assert_eq!(fs::read_to_string(&note).unwrap(), "keep this\n");
            assert_eq!(fs::metadata(note).unwrap().mode() & 0o7777, 0o640);
        });
    }

    #[cfg(unix)]
    #[test]
    fn normal_workspace_startup_refuses_a_displaced_root_before_seeding() {
        with_sk_path(|home| {
            let kit = home.join(".scriptkit");
            std::env::set_var(SK_PATH_ENV, &kit);
            let result = setup_before_diagnostics(&kit, Some(home), || {
                fs::rename(&kit, home.join("displaced")).unwrap();
                fs::create_dir(&kit).unwrap();
                fs::write(kit.join("keep"), "rival").unwrap();
            });
            assert!(result.is_err());
            assert!(!kit.join("plugins").exists());
            assert_eq!(fs::read_to_string(kit.join("keep")).unwrap(), "rival");
        });
    }

    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    #[test]
    fn normal_workspace_startup_legacy_transfer_never_replaces_a_competing_entry() {
        for directory in [false, true] {
            let temp = TempDir::new().unwrap();
            let a = temp.path().join("v1"); let b = temp.path().join("new kit");
            fs::create_dir(&a).unwrap(); fs::create_dir(&b).unwrap();
            let (src,dest) = (a.join("item"), b.join("item"));
            if directory { fs::create_dir(&src).unwrap(); } else { fs::write(&src, b"original").unwrap(); }
            let expected = fs::symlink_metadata(&src).unwrap();
            let from = legacy_move::Directory::open(&a).unwrap();
            let to = legacy_move::Directory::open(&b).unwrap();
            if directory { fs::create_dir(&dest).unwrap(); } else { fs::write(&dest, b"rival").unwrap(); }
            assert!(from.move_entry("item".as_ref(), &expected, &to, "item".as_ref()).is_err());
            assert!(src.exists() && dest.exists());
            if !directory { assert_eq!(fs::read(src).unwrap(), b"original"); assert_eq!(fs::read(dest).unwrap(), b"rival"); }
        }
    }

    /// Test that plugin directories live under plugins/
    /// Expected structure: ~/.scriptkit/plugins/main/scripts, ~/.scriptkit/plugins/main/scriptlets
    #[test]
    fn test_plugin_directory_uses_plugins_subdirectory() {
        with_sk_path(|kit_root| {
            let result = ensure_kit_setup();

            let kit_main_scripts = kit_root.join("plugins").join("main").join("scripts");
            let kit_main_extensions = kit_root.join("plugins").join("main").join("scriptlets");

            assert!(
                kit_main_scripts.exists(),
                "Expected plugins/main/scripts to exist at {:?}",
                kit_main_scripts
            );
            assert!(
                kit_main_extensions.exists(),
                "Expected plugins/main/scriptlets to exist at {:?}",
                kit_main_extensions
            );

            let old_main_scripts = kit_root.join("main").join("scripts");
            assert!(
                !old_main_scripts.exists(),
                "Old structure main/scripts should NOT exist at {:?}",
                old_main_scripts
            );

            assert!(!result.warnings.iter().any(|w| w.contains("Failed")));
        });
    }

    /// Test that sample files are created in plugins/main/scripts
    #[test]
    fn test_sample_files_in_plugins_subdirectory() {
        with_sk_path(|kit_root| {
            let result = ensure_kit_setup();

            if result.is_fresh_install {
                let hello_script = kit_root
                    .join("plugins")
                    .join("main")
                    .join("scripts")
                    .join("hello-world.ts");
                assert!(
                    hello_script.exists(),
                    "Expected hello-world.ts at {:?}",
                    hello_script
                );
            }
        });
    }

    #[test]
    fn test_fresh_install_seeds_canonical_menu_syntax_handlers() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().join("scriptkit-fresh-menu-syntax");
        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        let result = ensure_kit_setup();
        assert!(result.is_fresh_install);

        let main_scripts = kit_root.join("plugins").join("main").join("scripts");
        for filename in [
            "capture-todo-inbox.ts",
            "create-calendar-event.ts",
            "create-mac-calendar-event.ts",
            "add-google-calendar-event.ts",
            "create-reminder.ts",
            "snooze-task.ts",
            "defer-task.ts",
            "append-daily-note.ts",
            "draft-social-post.ts",
            "save-tagged-link.ts",
        ] {
            let path = main_scripts.join(filename);
            assert!(path.exists(), "expected seeded handler at {:?}", path);
            let content = fs::read_to_string(&path).unwrap();
            assert!(
                content.contains("menuSyntax"),
                "{filename} should declare menuSyntax metadata"
            );
            assert!(
                content.contains(r#"family: "capture.v1""#),
                "{filename} should declare a capture.v1 handler"
            );
        }

        let hello_path = main_scripts.join("hello-world.ts");
        let hello_source = fs::read_to_string(&hello_path)
            .expect("the actual fresh-install Hello World script must be readable");
        let parsed = crate::metadata_parser::extract_typed_metadata(&hello_source);
        assert!(
            parsed.errors.is_empty(),
            "the seeded starter must have parseable host compatibility metadata: {:?}",
            parsed.errors
        );
        let metadata = parsed
            .metadata
            .expect("the seeded starter must expose typed metadata");
        assert_eq!(
            metadata.extra.get("sdkCapabilities"),
            Some(&serde_json::json!(["arg", "div"]))
        );
        assert_eq!(
            metadata.extra.get("executionTopology"),
            Some(&serde_json::json!("typescript-script"))
        );
        let hello_script = crate::scripts::Script {
            name: "Hello World".to_owned(),
            path: hello_path,
            extension: "ts".to_owned(),
            plugin_id: "main".to_owned(),
            typed_metadata: Some(metadata),
            ..crate::scripts::Script::default()
        };
        assert!(
            crate::scripts::validate_declared_sdk_capabilities(&hello_script).is_empty(),
            "the first-run starter must satisfy its actual interactive SDK contract"
        );

        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    #[test]
    fn alternate_kit_migration_does_not_touch_the_legacy_home() {
        let home = tempfile::tempdir().unwrap();
        let legacy = home.path().join(".kenv");
        let script = legacy.join("scripts/demo.ts");
        fs::create_dir_all(script.parent().unwrap()).unwrap();
        fs::write(&script, "fixture").unwrap();
        assert!(!migrate_from_kenv_in(
            home.path(),
            &home.path().join("other-kit")
        ));
        assert!(script.is_file());
        assert!(!home.path().join(".scriptkit").exists());
    }

    #[test]
    fn default_kit_migration_keeps_unmapped_files_and_links_only_an_empty_legacy_root() {
        for has_extra in [true, false] {
            let home = tempfile::tempdir().unwrap();
            let legacy = home.path().join(".kenv");
            let kit = home.path().join(".scriptkit");
            fs::create_dir_all(legacy.join("scripts")).unwrap();
            fs::write(legacy.join("scripts/demo.ts"), "fixture").unwrap();
            if has_extra {
                fs::write(legacy.join("keep.txt"), "mine").unwrap();
                fs::create_dir_all(legacy.join("kenvs/team")).unwrap();
                fs::create_dir(legacy.join(".git")).unwrap();
                assert!(!migrate_from_kenv_in(home.path(), &kit));
                assert!(!kit.exists());
                assert_eq!(fs::read_to_string(legacy.join("keep.txt")).unwrap(), "mine");
                assert!(legacy.join("scripts/demo.ts").is_file());
                assert!(legacy.join("kenvs/team").is_dir());
                assert!(legacy.join(".git").is_dir());
            } else {
                assert!(migrate_from_kenv_in(home.path(), &kit));
                assert_eq!(
                    fs::read_to_string(kit.join("plugins/main/scripts/demo.ts")).unwrap(),
                    "fixture"
                );
                #[cfg(unix)]
                assert!(legacy.is_symlink());
            }
        }
    }

    #[test]
    fn legacy_migration_refuses_an_indirect_script_before_moving_anything() {
        #[cfg(unix)]
        {
            let home = tempfile::tempdir().unwrap();
            let legacy = home.path().join(".kenv");
            let kit = home.path().join(".scriptkit");
            fs::create_dir_all(legacy.join("scripts")).unwrap();
            fs::write(legacy.join("config.ts"), "fixture").unwrap();
            let target = home.path().join("outside.ts");
            fs::write(&target, "untouched").unwrap();
            std::os::unix::fs::symlink(&target, legacy.join("scripts/indirect.ts")).unwrap();
            assert!(!migrate_from_kenv_in(home.path(), &kit));
            assert!(!kit.exists());
            assert!(legacy.join("config.ts").is_file());
            assert!(legacy.join("scripts/indirect.ts").is_symlink());
            assert_eq!(fs::read_to_string(target).unwrap(), "untouched");
        }
    }

    #[test]
    fn test_get_kit_path_default() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var(SK_PATH_ENV);
        let path = get_kit_path();
        assert!(path.to_string_lossy().contains(".scriptkit"));
        drop(lock);
    }

    #[test]
    fn test_get_kit_path_with_override() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var(SK_PATH_ENV, "/custom/path");
        let path = get_kit_path();
        assert_eq!(path, PathBuf::from("/custom/path"));
        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    #[test]
    fn test_get_kit_path_with_tilde() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var(SK_PATH_ENV, "~/.config/kit");
        let path = get_kit_path();
        assert!(!path.to_string_lossy().contains("~"));
        assert!(path.to_string_lossy().contains(".config/kit"));
        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    #[test]
    fn test_get_kit_path_with_env_var_expansion() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let env_var = "SCRIPT_KIT_TEST_SK_PATH_ROOT";
        std::env::set_var(env_var, "/tmp/script-kit-env-root");
        std::env::set_var(SK_PATH_ENV, format!("${env_var}/kit"));

        let path = get_kit_path();
        assert_eq!(path, PathBuf::from("/tmp/script-kit-env-root/kit"));

        std::env::remove_var(SK_PATH_ENV);
        std::env::remove_var(env_var);
        drop(lock);
    }

    /// Comprehensive setup verification test
    /// Verifies the complete directory structure matches documentation:
    /// ```
    /// ~/.scriptkit/
    /// ├── kit/
    /// │   ├── main/
    /// │   │   ├── scripts/
    /// │   │   ├── scriptlets/
    /// │   │   └── agents/
    /// │   ├── config.ts
    /// │   ├── theme.json
    /// │   ├── package.json
    /// │   ├── tsconfig.json
    /// │   ├── AGENTS.md
    /// │   └── CLAUDE.md
    /// ├── sdk/
    /// │   └── kit-sdk.ts
    /// ├── db/
    /// ├── logs/
    /// ├── cache/
    /// └── GUIDE.md
    /// ```
    #[test]
    fn test_complete_setup_structure() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = TempDir::new().unwrap();
        // Use a subdirectory that definitely doesn't exist for fresh install detection
        let kit_root = temp_dir.path().join("scriptkit-test");

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        let result = ensure_kit_setup();
        // Don't assert is_fresh_install - just verify the structure is correct
        assert!(
            result.warnings.is_empty() || !result.warnings.iter().any(|w| w.contains("Failed"))
        );

        // Verify plugins/ subdirectory structure
        let plugins_dir = kit_root.join("plugins");
        assert!(plugins_dir.exists(), "plugins/ directory should exist");

        // Verify main kit directories
        let main_dir = plugins_dir.join("main");
        assert!(
            main_dir.join("scripts").exists(),
            "plugins/main/scripts/ should exist"
        );
        assert!(
            main_dir.join("scriptlets").exists(),
            "plugins/main/scriptlets/ should exist"
        );
        assert!(
            main_dir.join("agents").exists(),
            "plugins/main/agents/ should exist"
        );

        // Verify user config files at workspace root
        assert!(
            kit_root.join("config.ts").exists(),
            "config.ts should exist"
        );
        assert!(
            kit_root.join("theme.json").exists(),
            "theme.json should exist"
        );
        assert!(
            kit_root.join("package.json").exists(),
            "package.json should exist"
        );
        assert!(
            kit_root.join("tsconfig.json").exists(),
            "tsconfig.json should exist"
        );
        assert!(
            kit_root.join("AGENTS.md").exists(),
            "AGENTS.md should exist"
        );
        assert!(
            kit_root.join("CLAUDE.md").exists(),
            "CLAUDE.md should exist"
        );

        // Verify SDK directory
        assert!(
            kit_root.join("sdk").join("kit-sdk.ts").exists(),
            "sdk/kit-sdk.ts should exist"
        );
        assert!(
            kit_root.join("bin").join("scriptkit").exists(),
            "bin/scriptkit should exist"
        );

        // Verify other directories
        assert!(kit_root.join("db").exists(), "db/ directory should exist");
        assert!(
            kit_root.join("logs").exists(),
            "logs/ directory should exist"
        );
        assert!(
            kit_root.join("cache").exists(),
            "cache/ directory should exist"
        );

        // Verify GUIDE.md at root
        assert!(
            kit_root.join("GUIDE.md").exists(),
            "GUIDE.md should exist at root"
        );

        // Verify sample script on fresh install
        let hello_script = main_dir.join("scripts").join("hello-world.ts");
        assert!(
            hello_script.exists(),
            "hello-world.ts sample script should exist"
        );

        // Verify config.ts content
        let config_content = fs::read_to_string(kit_root.join("config.ts")).unwrap();
        assert!(
            config_content.contains("@scriptkit/sdk"),
            "config.ts should import @scriptkit/sdk"
        );
        assert!(
            config_content.contains("hotkey"),
            "config.ts should have hotkey config"
        );

        // Verify package.json has correct name and type
        let package_content = fs::read_to_string(kit_root.join("package.json")).unwrap();
        assert!(
            package_content.contains("@scriptkit/kit"),
            "package.json should have @scriptkit/kit name"
        );
        assert!(
            package_content.contains("\"type\": \"module\""),
            "package.json should enable ESM"
        );

        // Root-level AGENTS.md has the actual SDK reference
        let agents_content = fs::read_to_string(kit_root.join("AGENTS.md")).unwrap();
        assert!(
            agents_content.contains("Script Kit"),
            "Root AGENTS.md should mention Script Kit"
        );
        assert!(
            agents_content.contains("~/.scriptkit/config.ts"),
            "Root AGENTS.md should have correct config path"
        );

        // Root-level CLAUDE.md has the actual agent instructions
        let claude_content = fs::read_to_string(kit_root.join("CLAUDE.md")).unwrap();
        assert!(
            claude_content.contains("Script Kit"),
            "Root CLAUDE.md should mention Script Kit"
        );
        assert!(
            claude_content.contains("@scriptkit/sdk"),
            "Root CLAUDE.md should reference the SDK"
        );

        // Verify CleanShot X built-in extension
        let cleanshot_dir = plugins_dir.join("cleanshot").join("scriptlets");
        assert!(
            cleanshot_dir.exists(),
            "plugins/cleanshot/scriptlets/ should exist"
        );
        let cleanshot_extension = cleanshot_dir.join("main.md");
        assert!(
            cleanshot_extension.exists(),
            "plugins/cleanshot/scriptlets/main.md should exist"
        );
        let cleanshot_content = fs::read_to_string(&cleanshot_extension).unwrap();
        assert!(
            cleanshot_content.contains("CleanShot X"),
            "CleanShot extension should have CleanShot X title"
        );
        assert!(
            cleanshot_content.contains("cleanshot://capture-area"),
            "CleanShot extension should have Capture Area command"
        );
        assert!(
            cleanshot_content.contains("cleanshot://record-screen"),
            "CleanShot extension should have Record Screen command"
        );

        // Verify 1Password built-in extension
        let onepassword_dir = plugins_dir.join("1password").join("scriptlets");
        assert!(
            onepassword_dir.exists(),
            "plugins/1password/scriptlets/ should exist"
        );
        let onepassword_extension = onepassword_dir.join("main.md");
        assert!(
            onepassword_extension.exists(),
            "plugins/1password/scriptlets/main.md should exist"
        );
        let onepassword_content = fs::read_to_string(&onepassword_extension).unwrap();
        assert!(
            onepassword_content.contains("1Password"),
            "1Password extension should have 1Password title"
        );
        assert!(
            onepassword_content.contains("op item list"),
            "1Password extension should have item list command"
        );
        assert!(
            onepassword_content.contains("op whoami"),
            "1Password extension should have whoami command"
        );

        // Verify Quick Links built-in extension
        let quicklinks_dir = plugins_dir.join("quicklinks").join("scriptlets");
        assert!(
            quicklinks_dir.exists(),
            "plugins/quicklinks/scriptlets/ should exist"
        );
        let quicklinks_extension = quicklinks_dir.join("main.md");
        assert!(
            quicklinks_extension.exists(),
            "plugins/quicklinks/scriptlets/main.md should exist"
        );
        let quicklinks_content = fs::read_to_string(&quicklinks_extension).unwrap();
        assert!(
            quicklinks_content.contains("Quick Links"),
            "Quick Links extension should have Quick Links title"
        );
        assert!(
            quicklinks_content.contains("https://github.com"),
            "Quick Links extension should have GitHub link"
        );
        assert!(
            quicklinks_content.contains("https://www.google.com"),
            "Quick Links extension should have Google link"
        );

        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    /// Test that paths in AGENTS.md match actual setup paths
    #[test]
    fn test_agents_md_paths_match_setup() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().to_path_buf();

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());
        let _ = ensure_kit_setup();

        // Root-level AGENTS.md is the canonical location now
        let agents_content = fs::read_to_string(kit_root.join("AGENTS.md")).unwrap();

        // Verify documented paths actually exist
        let documented_paths = [
            ("plugins/main/scripts", "~/.scriptkit/plugins/main/scripts/"),
            (
                "plugins/main/scriptlets",
                "~/.scriptkit/plugins/main/scriptlets/",
            ),
            ("config.ts", "~/.scriptkit/config.ts"),
            ("theme.json", "~/.scriptkit/theme.json"),
            ("sdk/kit-sdk.ts", "~/.scriptkit/sdk/"),
            ("bin/scriptkit", "~/.scriptkit/bin/scriptkit"),
        ];

        for (relative_path, doc_path) in documented_paths {
            assert!(
                agents_content.contains(doc_path),
                "AGENTS.md should document path: {}",
                doc_path
            );

            let actual_path = kit_root.join(relative_path);
            // For directories, check they exist; for files, check the parent exists
            if relative_path.contains('.') {
                assert!(
                    actual_path.exists(),
                    "Documented path {} should exist as file: {:?}",
                    doc_path,
                    actual_path
                );
            } else {
                assert!(
                    actual_path.exists(),
                    "Documented path {} should exist as directory: {:?}",
                    doc_path,
                    actual_path
                );
            }
        }

        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }
}

#[cfg(test)]
mod tab_ai_agent_doc_contract_tests {
    const ROOT_CLAUDE: &str = include_str!("../../kit-init/ROOT_CLAUDE.md");
    const ROOT_AGENTS: &str = include_str!("../../kit-init/ROOT_AGENTS.md");
    const AI_MOD_SOURCE: &str = include_str!("../ai/mod.rs");
    const TAB_CONTEXT_SOURCE: &str = include_str!("../ai/tab_context.rs");

    fn assert_tab_ai_doc_contract(source: &str, label: &str) {
        for needle in [
            "Quick Terminal with Flat Context Injection",
            "AppView::QuickTerminalView",
            "TermPrompt",
            "TabAiHarnessSubmissionMode",
            "PasteOnly",
            "Submit",
            "claudeCode",
            "CaptureContextOptions::tab_ai_submit()",
            "Cmd+W",
            "Escape",
            "Agent Chat",
            "open_tab_ai_agent_chat_with_entry_intent",
        ] {
            assert!(source.contains(needle), "{label} must contain `{needle}`");
        }

        assert!(
            source.contains("PTY") || source.contains("pty"),
            "{label} must describe the landed PTY-backed path"
        );

        // The universal AI entry migrated from plain Tab to Cmd+Enter
        // (commit b6c5752bb "Deprecate Tab ACP agent chat surface").
        assert!(
            source.contains(
                "Command+Enter in `AppView::ScriptList` routes through the Agent Chat entry path"
            ),
            "{label} must describe Cmd+Enter as the Agent Chat entry path"
        );

        assert!(
            !source.contains(
                "Plain `Tab` in `AppView::ScriptList` routes through the Agent Chat entry path"
            ),
            "{label} must not describe plain Tab as the Agent Chat entry path"
        );
        assert!(
            !source.contains("Plain `Tab` opens the harness terminal"),
            "{label} must not describe plain Tab as opening the harness terminal"
        );
        assert!(
            !source.contains("`Shift+Tab` in `AppView::ScriptList` with non-empty filter text"),
            "{label} must not describe Shift+Tab in ScriptList as the default quick-submit path"
        );
    }

    fn assert_tab_ai_schema_detail_contract(source: &str, label: &str) {
        for needle in [
            "pub const TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION: u32 = 2;",
            "pub struct TabAiExecutionRecord",
            "PanelOnlyElements",
            "CollectorFallback",
            "NoSemanticElements",
            "MissingFocusTarget",
            "InputNotExtractable",
            "InputNotApplicable",
        ] {
            assert!(source.contains(needle), "{label} must contain `{needle}`");
        }

        for stale in [
            "TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION: u32 = 1",
            "record + status + output + duration",
            "Persisted memory: intent, script, target bundle_id, outcome",
            "PanelOnlyWarning",
            "MissingInput",
        ] {
            assert!(
                !source.contains(stale),
                "{label} contains stale Tab AI schema detail: {stale}"
            );
        }
    }

    #[test]
    fn root_claude_doc_matches_landed_tab_ai_contract() {
        assert_tab_ai_doc_contract(ROOT_CLAUDE, "kit-init/ROOT_CLAUDE.md");
    }

    #[test]
    fn root_agents_doc_matches_landed_tab_ai_contract() {
        assert_tab_ai_doc_contract(ROOT_AGENTS, "kit-init/ROOT_AGENTS.md");
    }

    #[test]
    fn tab_ai_schema_detail_matches_current_tab_ai_types() {
        assert_tab_ai_schema_detail_contract(TAB_CONTEXT_SOURCE, "src/ai/tab_context.rs");
    }

    #[test]
    fn ai_mod_docs_reflect_agent_chat_primary_path() {
        for needle in [
            "//! AI surfaces and shared contracts.",
            "//! - The chat is a view under the main window's input (`crate::one_window`)",
            "`ScriptListApp::one_window_open_chat(ChatEntry, cx)`",
        ] {
            assert!(
                AI_MOD_SOURCE.contains(needle),
                "src/ai/mod.rs docs must contain `{needle}`"
            );
        }

        assert!(
            !AI_MOD_SOURCE.contains("//! - Submission shape: flat text-native"),
            "src/ai/mod.rs must not describe QuickTerminalView as the primary AI surface"
        );
    }
}

#[cfg(test)]
mod asset_destination_tests {
    use std::path::PathBuf;

    /// Resolve the relative destination path for an embedded kit-init asset.
    ///
    /// Skills map to the workspace root (`skills/…`), the config template
    /// maps to `config.ts`, and everything else passes through unchanged.
    fn embedded_asset_destination_relative(asset: &str) -> PathBuf {
        if let Some(_rest) = asset.strip_prefix("skills/") {
            // skills/ already carries the correct relative prefix
            return PathBuf::from(asset);
        }
        match asset {
            "config-template.ts" => PathBuf::from("config.ts"),
            other => PathBuf::from(other),
        }
    }

    #[test]
    fn skills_install_to_workspace_root_skills_directory() {
        assert_eq!(
            embedded_asset_destination_relative("skills/update-config/SKILL.md"),
            PathBuf::from("skills/update-config/SKILL.md")
        );
    }

    #[test]
    fn skills_readme_installs_to_workspace_root() {
        assert_eq!(
            embedded_asset_destination_relative("skills/README.md"),
            PathBuf::from("skills/README.md")
        );
    }

    #[test]
    fn config_template_installs_under_kit_directory() {
        assert_eq!(
            embedded_asset_destination_relative("config-template.ts"),
            PathBuf::from("config.ts")
        );
    }

    #[test]
    fn passthrough_asset_unchanged() {
        assert_eq!(
            embedded_asset_destination_relative("GUIDE.md"),
            PathBuf::from("GUIDE.md")
        );
    }

    /// Verify that `ensure_kit_setup` writes bundled skills under the Script Kit plugin.
    #[test]
    fn setup_creates_skills_under_scriptkit_plugin() {
        super::tests::with_sk_path(|kit_root| {
            let result = super::ensure_kit_setup();
            assert_eq!(
                result.kit_path, kit_root,
                "setup must stay inside the fixture root"
            );
            let skills = kit_root.join("plugins/scriptkit/skills");
            assert_eq!(
                std::fs::read_to_string(skills.join("README.md"))
                    .expect("installed skills library"),
                super::EMBEDDED_SKILLS_README
            );
            assert_eq!(
                std::fs::read_to_string(skills.join("update-config/SKILL.md"))
                    .expect("installed update-config skill"),
                super::EMBEDDED_SKILL_UPDATE_CONFIG
            );
            assert!(
                !kit_root.join("kit/skills").exists(),
                "skills must not be nested under kit/"
            );
        });
    }
}

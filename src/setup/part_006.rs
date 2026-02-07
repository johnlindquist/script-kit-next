#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Test that kit directory structure uses kit/ subdirectory
    /// Expected structure: ~/.scriptkit/kit/main/scripts, ~/.scriptkit/kit/main/extensions
    #[test]
    fn test_kit_directory_uses_kit_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().to_path_buf();

        // Set SK_PATH to our temp directory
        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        // Run setup
        let result = ensure_kit_setup();

        // Verify the kit/ subdirectory structure exists
        let kit_main_scripts = kit_root.join("kit").join("main").join("scripts");
        let kit_main_extensions = kit_root.join("kit").join("main").join("extensions");

        assert!(
            kit_main_scripts.exists(),
            "Expected kit/main/scripts to exist at {:?}",
            kit_main_scripts
        );
        assert!(
            kit_main_extensions.exists(),
            "Expected kit/main/extensions to exist at {:?}",
            kit_main_extensions
        );

        // The old structure should NOT exist
        let old_main_scripts = kit_root.join("main").join("scripts");
        assert!(
            !old_main_scripts.exists(),
            "Old structure main/scripts should NOT exist at {:?}",
            old_main_scripts
        );

        // Cleanup
        std::env::remove_var(SK_PATH_ENV);
        assert!(!result.warnings.iter().any(|w| w.contains("Failed")));
    }

    /// Test that sample files are created in kit/main/scripts
    #[test]
    fn test_sample_files_in_kit_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().to_path_buf();

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        let result = ensure_kit_setup();

        // On fresh install, sample hello-world.ts should be in kit/main/scripts
        if result.is_fresh_install {
            let hello_script = kit_root
                .join("kit")
                .join("main")
                .join("scripts")
                .join("hello-world.ts");
            assert!(
                hello_script.exists(),
                "Expected hello-world.ts at {:?}",
                hello_script
            );
        }

        std::env::remove_var(SK_PATH_ENV);
    }

    #[test]
    fn test_bun_is_discoverable() {
        // This test just verifies the function doesn't panic
        let _ = bun_is_discoverable();
    }

    #[test]
    fn test_bun_exe_name() {
        let name = bun_exe_name();
        #[cfg(windows)]
        assert_eq!(name, "bun.exe");
        #[cfg(not(windows))]
        assert_eq!(name, "bun");
    }

    #[test]
    fn test_get_kit_path_default() {
        // Without SK_PATH set, should return ~/.scriptkit
        std::env::remove_var(SK_PATH_ENV);
        let path = get_kit_path();
        assert!(path.to_string_lossy().contains(".scriptkit"));
    }

    #[test]
    fn test_get_kit_path_with_override() {
        // With SK_PATH set, should return the override
        std::env::set_var(SK_PATH_ENV, "/custom/path");
        let path = get_kit_path();
        assert_eq!(path, PathBuf::from("/custom/path"));
        std::env::remove_var(SK_PATH_ENV);
    }

    #[test]
    fn test_get_kit_path_with_tilde() {
        // SK_PATH with tilde should expand
        std::env::set_var(SK_PATH_ENV, "~/.config/kit");
        let path = get_kit_path();
        assert!(!path.to_string_lossy().contains("~"));
        assert!(path.to_string_lossy().contains(".config/kit"));
        std::env::remove_var(SK_PATH_ENV);
    }

    #[test]
    fn test_get_kit_path_with_env_var_expansion() {
        let env_var = "SCRIPT_KIT_TEST_SK_PATH_ROOT";
        std::env::set_var(env_var, "/tmp/script-kit-env-root");
        std::env::set_var(SK_PATH_ENV, format!("${env_var}/kit"));

        let path = get_kit_path();
        assert_eq!(path, PathBuf::from("/tmp/script-kit-env-root/kit"));

        std::env::remove_var(SK_PATH_ENV);
        std::env::remove_var(env_var);
    }

    /// Comprehensive setup verification test
    /// Verifies the complete directory structure matches documentation:
    /// ```
    /// ~/.scriptkit/
    /// ├── kit/
    /// │   ├── main/
    /// │   │   ├── scripts/
    /// │   │   ├── extensions/
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
        let temp_dir = TempDir::new().unwrap();
        // Use a subdirectory that definitely doesn't exist for fresh install detection
        let kit_root = temp_dir.path().join("scriptkit-test");

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        let result = ensure_kit_setup();
        // Don't assert is_fresh_install - just verify the structure is correct
        assert!(
            result.warnings.is_empty() || !result.warnings.iter().any(|w| w.contains("Failed"))
        );

        // Verify kit/ subdirectory structure
        let kit_dir = kit_root.join("kit");
        assert!(kit_dir.exists(), "kit/ directory should exist");

        // Verify main kit directories
        let main_dir = kit_dir.join("main");
        assert!(
            main_dir.join("scripts").exists(),
            "kit/main/scripts/ should exist"
        );
        assert!(
            main_dir.join("extensions").exists(),
            "kit/main/extensions/ should exist"
        );
        assert!(
            main_dir.join("agents").exists(),
            "kit/main/agents/ should exist"
        );

        // Verify user config files in kit/
        assert!(
            kit_dir.join("config.ts").exists(),
            "kit/config.ts should exist"
        );
        assert!(
            kit_dir.join("theme.json").exists(),
            "kit/theme.json should exist"
        );
        assert!(
            kit_dir.join("package.json").exists(),
            "kit/package.json should exist"
        );
        assert!(
            kit_dir.join("tsconfig.json").exists(),
            "kit/tsconfig.json should exist"
        );
        assert!(
            kit_dir.join("AGENTS.md").exists(),
            "kit/AGENTS.md should exist"
        );
        assert!(
            kit_dir.join("CLAUDE.md").exists(),
            "kit/CLAUDE.md should exist"
        );

        // Verify SDK directory
        assert!(
            kit_root.join("sdk").join("kit-sdk.ts").exists(),
            "sdk/kit-sdk.ts should exist"
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
        let config_content = fs::read_to_string(kit_dir.join("config.ts")).unwrap();
        assert!(
            config_content.contains("@scriptkit/sdk"),
            "config.ts should import @scriptkit/sdk"
        );
        assert!(
            config_content.contains("hotkey"),
            "config.ts should have hotkey config"
        );

        // Verify package.json has correct name and type
        let package_content = fs::read_to_string(kit_dir.join("package.json")).unwrap();
        assert!(
            package_content.contains("@scriptkit/kit"),
            "package.json should have @scriptkit/kit name"
        );
        assert!(
            package_content.contains("\"type\": \"module\""),
            "package.json should enable ESM"
        );

        // Verify AGENTS.md content
        let agents_content = fs::read_to_string(kit_dir.join("AGENTS.md")).unwrap();
        assert!(
            agents_content.contains("Script Kit"),
            "AGENTS.md should mention Script Kit"
        );
        assert!(
            agents_content.contains("~/.scriptkit/kit/config.ts"),
            "AGENTS.md should have correct config path"
        );

        // Verify CLAUDE.md content
        let claude_content = fs::read_to_string(kit_dir.join("CLAUDE.md")).unwrap();
        assert!(
            claude_content.contains("Script Kit GPUI"),
            "CLAUDE.md should mention Script Kit GPUI"
        );
        assert!(
            claude_content.contains("NOT the original Script Kit"),
            "CLAUDE.md should warn about v1 vs v2"
        );

        // Verify CleanShot X built-in extension
        let cleanshot_dir = kit_dir.join("cleanshot").join("extensions");
        assert!(
            cleanshot_dir.exists(),
            "kit/cleanshot/extensions/ should exist"
        );
        let cleanshot_extension = cleanshot_dir.join("main.md");
        assert!(
            cleanshot_extension.exists(),
            "kit/cleanshot/extensions/main.md should exist"
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
        let onepassword_dir = kit_dir.join("1password").join("extensions");
        assert!(
            onepassword_dir.exists(),
            "kit/1password/extensions/ should exist"
        );
        let onepassword_extension = onepassword_dir.join("main.md");
        assert!(
            onepassword_extension.exists(),
            "kit/1password/extensions/main.md should exist"
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
        let quicklinks_dir = kit_dir.join("quicklinks").join("extensions");
        assert!(
            quicklinks_dir.exists(),
            "kit/quicklinks/extensions/ should exist"
        );
        let quicklinks_extension = quicklinks_dir.join("main.md");
        assert!(
            quicklinks_extension.exists(),
            "kit/quicklinks/extensions/main.md should exist"
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
    }

    /// Test that paths in AGENTS.md match actual setup paths
    #[test]
    fn test_agents_md_paths_match_setup() {
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().to_path_buf();

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());
        let _ = ensure_kit_setup();

        let agents_content = fs::read_to_string(kit_root.join("kit").join("AGENTS.md")).unwrap();

        // Verify documented paths actually exist
        let documented_paths = [
            ("kit/main/scripts", "~/.scriptkit/kit/main/scripts/"),
            ("kit/main/extensions", "~/.scriptkit/kit/main/extensions/"),
            ("kit/config.ts", "~/.scriptkit/kit/config.ts"),
            ("kit/theme.json", "~/.scriptkit/kit/theme.json"),
            ("sdk/kit-sdk.ts", "~/.scriptkit/sdk/"),
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
    }
}

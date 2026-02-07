    // =========================================================================
    // Enhanced code preview tests (multi-line)
    // =========================================================================

    #[test]
    fn test_code_preview_short_first_line_appends_second() {
        let sl = make_test_scriptlet("Deploy", "cd ~/projects\nnpm run build", "bash");
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        assert!(
            preview.contains("\u{2192}"),
            "Short first line should append second line with arrow: {}",
            preview
        );
        assert!(preview.contains("cd ~/projects"));
        assert!(preview.contains("npm run build"));
    }

    #[test]
    fn test_code_preview_long_first_line_no_append() {
        let sl = make_test_scriptlet(
            "Long",
            "const result = fetchData()\nconsole.log(result)",
            "ts",
        );
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        // First line is > 20 chars, should NOT append second line
        assert!(
            !preview.contains("\u{2192}"),
            "Long first line should not append second: {}",
            preview
        );
    }

    #[test]
    fn test_code_preview_short_first_only_line() {
        let sl = make_test_scriptlet("Short", "ls -la", "bash");
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        // Only one line, can't append second
        assert_eq!(preview, "ls -la");
    }

    #[test]
    fn test_code_preview_multi_line_truncates_combined() {
        let sl = make_test_scriptlet(
            "Deploy",
            "cd ~/projects\nexport NODE_ENV=production && npm run build && npm run deploy --target staging",
            "bash",
        );
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        // Combined is long, should truncate
        assert!(preview.contains("\u{2192}"));
        assert!(
            preview.chars().count() <= 63,
            "Combined preview should be truncated, got {} chars: {}",
            preview.chars().count(),
            preview
        );
    }

    // =========================================================================
    // Extension default icon tests
    // =========================================================================

    #[test]
    fn test_extension_default_icon_shell() {
        assert_eq!(extension_default_icon("sh"), "Terminal");
        assert_eq!(extension_default_icon("bash"), "Terminal");
        assert_eq!(extension_default_icon("zsh"), "Terminal");
    }

    #[test]
    fn test_extension_default_icon_applescript() {
        assert_eq!(extension_default_icon("applescript"), "Terminal");
        assert_eq!(extension_default_icon("scpt"), "Terminal");
    }

    #[test]
    fn test_extension_default_icon_default_code() {
        assert_eq!(extension_default_icon("ts"), "Code");
        assert_eq!(extension_default_icon("js"), "Code");
        assert_eq!(extension_default_icon("py"), "Code");
        assert_eq!(extension_default_icon("rb"), "Code");
    }

    // =========================================================================
    // Extension language label tests
    // =========================================================================

    #[test]
    fn test_extension_language_label_typescript() {
        assert_eq!(extension_language_label("ts"), Some("TypeScript"));
        assert_eq!(extension_language_label("tsx"), Some("TypeScript"));
    }

    #[test]
    fn test_extension_language_label_javascript() {
        assert_eq!(extension_language_label("js"), Some("JavaScript"));
        assert_eq!(extension_language_label("mjs"), Some("JavaScript"));
    }

    #[test]
    fn test_extension_language_label_shell() {
        assert_eq!(extension_language_label("sh"), Some("Shell script"));
        assert_eq!(extension_language_label("bash"), Some("Shell script"));
        assert_eq!(extension_language_label("zsh"), Some("Zsh script"));
    }

    #[test]
    fn test_extension_language_label_python() {
        assert_eq!(extension_language_label("py"), Some("Python script"));
    }

    #[test]
    fn test_extension_language_label_unknown() {
        assert_eq!(extension_language_label("xyz"), None);
        assert_eq!(extension_language_label(""), None);
    }

    // =========================================================================
    // Auto-description with language label fallback tests
    // =========================================================================

    #[test]
    fn test_auto_description_language_label_fallback() {
        // Script with same-name filename and no metadata -> should get language label
        let script = crate::scripts::Script {
            name: "my-script".to_string(),
            path: std::path::PathBuf::from("/test/my-script.ts"),
            extension: "ts".to_string(),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        // Filename "my-script.ts" differs from name "my-script", so filename wins
        assert_eq!(desc, Some("my-script.ts".to_string()));
    }

    #[test]
    fn test_auto_description_language_label_when_filename_matches() {
        // Script where filename equals name -> language label should appear
        // This happens when the name IS the filename (without extension somehow)
        let script = crate::scripts::Script {
            name: "my-script.ts".to_string(),
            path: std::path::PathBuf::from("/test/my-script.ts"),
            extension: "ts".to_string(),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        // Filename "my-script.ts" == name "my-script.ts", so language label fallback
        assert_eq!(desc, Some("TypeScript".to_string()));
    }

    #[test]
    fn test_auto_description_shell_script_language_label() {
        let script = crate::scripts::Script {
            name: "backup.sh".to_string(),
            path: std::path::PathBuf::from("/test/backup.sh"),
            extension: "sh".to_string(),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        assert_eq!(desc, Some("Shell script".to_string()));
    }

    #[test]
    fn test_auto_description_explicit_description_unchanged() {
        let script = crate::scripts::Script {
            name: "test".to_string(),
            path: std::path::PathBuf::from("/test/test.ts"),
            extension: "ts".to_string(),
            description: Some("My custom description".to_string()),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        assert_eq!(desc, Some("My custom description".to_string()));
    }

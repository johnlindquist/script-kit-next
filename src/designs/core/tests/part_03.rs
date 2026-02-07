    // =========================================================================
    // Code preview tests
    // =========================================================================

    fn make_test_scriptlet(name: &str, code: &str, tool: &str) -> crate::scripts::Scriptlet {
        crate::scripts::Scriptlet {
            name: name.to_string(),
            description: None,
            code: code.to_string(),
            tool: tool.to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            file_path: None,
            command: None,
            alias: None,
        }
    }

    #[test]
    fn test_code_preview_shows_first_line() {
        let sl = make_test_scriptlet("Hello", "echo hello world", "bash");
        assert_eq!(
            code_preview_for_scriptlet(&sl),
            Some("echo hello world".to_string())
        );
    }

    #[test]
    fn test_code_preview_skips_comments() {
        let sl = make_test_scriptlet(
            "Script",
            "#!/bin/bash\n# This is a comment\n// Another comment\nls -la",
            "bash",
        );
        assert_eq!(code_preview_for_scriptlet(&sl), Some("ls -la".to_string()));
    }

    #[test]
    fn test_code_preview_empty_code() {
        let sl = make_test_scriptlet("Empty", "", "bash");
        assert_eq!(code_preview_for_scriptlet(&sl), None);
    }

    #[test]
    fn test_code_preview_only_comments() {
        let sl = make_test_scriptlet("Comments", "# comment\n// another\n/* block */", "bash");
        assert_eq!(code_preview_for_scriptlet(&sl), None);
    }

    #[test]
    fn test_code_preview_truncates_long_lines() {
        let long_code =
            "const result = await fetchDataFromRemoteServerWithComplexAuthenticationAndRetryLogic(url, options)";
        let sl = make_test_scriptlet("Long", long_code, "ts");
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        assert!(preview.ends_with("..."));
        assert!(preview.chars().count() <= 60);
    }

    #[test]
    fn test_code_preview_paste_shows_content() {
        let sl = make_test_scriptlet("Sig", "Best regards,\nJohn", "paste");
        // Short first line (< 20 chars) appends second line with arrow
        assert_eq!(
            code_preview_for_scriptlet(&sl),
            Some("Best regards, → John".to_string())
        );
    }

    #[test]
    fn test_code_preview_open_shows_url() {
        let sl = make_test_scriptlet("GitHub", "https://github.com", "open");
        assert_eq!(
            code_preview_for_scriptlet(&sl),
            Some("https://github.com".to_string())
        );
    }

    // =========================================================================
    // Match reason detection tests
    // =========================================================================

    #[test]
    fn test_match_reason_name_match_returns_none() {
        let s = make_test_script("Notes");
        // Query matches name → no reason indicator needed
        assert_eq!(detect_match_reason_for_script(&s, "notes"), None);
    }

    #[test]
    fn test_match_reason_short_query_returns_none() {
        let s = make_test_script("Notes");
        // Single char query → skip
        assert_eq!(detect_match_reason_for_script(&s, "n"), None);
    }

    #[test]
    fn test_match_reason_tag_match() {
        let mut s = make_test_script("Daily Backup");
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["productivity".to_string()],
            ..Default::default()
        });
        assert_eq!(
            detect_match_reason_for_script(&s, "productivity"),
            Some("tag: productivity".to_string())
        );
    }

    #[test]
    fn test_match_reason_author_match() {
        let mut s = make_test_script("My Tool");
        s.typed_metadata = Some(TypedMetadata {
            author: Some("John Lindquist".to_string()),
            ..Default::default()
        });
        assert_eq!(
            detect_match_reason_for_script(&s, "john"),
            Some("by John Lindquist".to_string())
        );
    }

    #[test]
    fn test_match_reason_shortcut_match() {
        let mut s = make_test_script("Quick Notes");
        s.shortcut = Some("opt n".to_string());
        assert_eq!(
            detect_match_reason_for_script(&s, "opt n"),
            Some("shortcut".to_string())
        );
    }

    #[test]
    fn test_match_reason_kit_match() {
        let mut s = make_test_script("Capture");
        s.kit_name = Some("cleanshot".to_string());
        assert_eq!(
            detect_match_reason_for_script(&s, "cleanshot"),
            Some("kit: cleanshot".to_string())
        );
    }

    #[test]
    fn test_match_reason_main_kit_not_shown() {
        let mut s = make_test_script("Capture");
        s.kit_name = Some("main".to_string());
        assert_eq!(detect_match_reason_for_script(&s, "main"), None);
    }

    #[test]
    fn test_scriptlet_match_reason_keyword() {
        let mut sl = make_test_scriptlet("Signature", "Best regards", "paste");
        sl.keyword = Some("!sig".to_string());
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "!sig"),
            Some("keyword: !sig".to_string())
        );
    }

    #[test]
    fn test_scriptlet_match_reason_code_match() {
        let sl = make_test_scriptlet("Open URL", "https://github.com", "open");
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "github"),
            Some("code match".to_string())
        );
    }

    #[test]
    fn test_scriptlet_match_reason_name_match_returns_none() {
        let sl = make_test_scriptlet("Open GitHub", "https://github.com", "open");
        // Query matches name → no reason indicator
        assert_eq!(detect_match_reason_for_scriptlet(&sl, "github"), None);
    }

    #[test]
    fn test_scriptlet_match_reason_group() {
        let mut sl = make_test_scriptlet("Hello", "echo hello", "bash");
        sl.group = Some("Development".to_string());
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "development"),
            Some("group: Development".to_string())
        );
    }

    // =========================================================================
    // Excerpt helper tests
    // =========================================================================

    #[test]
    fn test_excerpt_short_text_no_truncation() {
        let result = excerpt_around_match("short text", "short", 40);
        assert_eq!(result, "short text");
    }

    #[test]
    fn test_excerpt_long_text_shows_ellipsis() {
        let text = "This is a very long description that talks about managing clipboard history and other features";
        let result = excerpt_around_match(text, "clipboard", 30);
        assert!(
            result.contains("clipboard"),
            "Excerpt should contain the matched term"
        );
        assert!(
            result.contains("..."),
            "Long text should be truncated with ellipsis"
        );
    }

    #[test]
    fn test_excerpt_match_at_start() {
        let text = "clipboard manager that helps you organize your copy history across all apps";
        let result = excerpt_around_match(text, "clipboard", 30);
        // Match is at the start, so excerpt starts from beginning
        assert!(result.starts_with("clipboard"));
    }

    #[test]
    fn test_excerpt_match_at_end() {
        let text = "A tool that helps you organize and manage your clipboard";
        let result = excerpt_around_match(text, "clipboard", 30);
        assert!(result.contains("clipboard"));
    }

    // =========================================================================
    // Script match reason: description excerpt tests
    // =========================================================================

    #[test]
    fn test_match_reason_description_excerpt() {
        let mut s = make_test_script("My Tool");
        s.description = Some("Manages clipboard history across all your devices".to_string());
        let reason = detect_match_reason_for_script(&s, "clipboard");
        assert!(
            reason.is_some(),
            "Description match should produce a reason"
        );
        let reason = reason.unwrap();
        assert!(
            reason.starts_with("desc: "),
            "Should start with 'desc: ', got: {}",
            reason
        );
        assert!(
            reason.contains("clipboard"),
            "Excerpt should contain the match term"
        );
    }

    #[test]
    fn test_match_reason_description_not_shown_when_name_matches() {
        let mut s = make_test_script("Clipboard Manager");
        s.description = Some("Manages clipboard history".to_string());
        // Name matches "clipboard" so no reason needed
        assert_eq!(detect_match_reason_for_script(&s, "clipboard"), None);
    }

    // =========================================================================
    // Script match reason: alias tests
    // =========================================================================

    #[test]
    fn test_match_reason_alias_match() {
        let mut s = make_test_script("Git Commit Helper");
        s.alias = Some("gc".to_string());
        let reason = detect_match_reason_for_script(&s, "gc");
        assert_eq!(reason, Some("alias: /gc".to_string()));
    }

    #[test]
    fn test_match_reason_alias_not_shown_when_name_matches() {
        let mut s = make_test_script("GC Cleaner");
        s.alias = Some("gc".to_string());
        // Name contains "GC" so no reason needed
        assert_eq!(detect_match_reason_for_script(&s, "gc"), None);
    }

    // =========================================================================
    // Script match reason: path match tests
    // =========================================================================

    #[test]
    fn test_match_reason_path_match() {
        let mut s = make_test_script("My Tool");
        s.path = std::path::PathBuf::from("/Users/john/.kenv/scripts/secret-helper.ts");
        let reason = detect_match_reason_for_script(&s, "secret-helper");
        assert_eq!(reason, Some("path match".to_string()));
    }

    // =========================================================================
    // Scriptlet match reason: alias tests
    // =========================================================================

    #[test]
    fn test_scriptlet_match_reason_alias() {
        let mut sl = make_test_scriptlet("Quick Paste", "Best regards", "paste");
        sl.alias = Some("qp".to_string());
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "qp"),
            Some("alias: /qp".to_string())
        );
    }

    // =========================================================================
    // Scriptlet match reason: tool type tests
    // =========================================================================

    #[test]
    fn test_scriptlet_match_reason_tool_type() {
        let sl = make_test_scriptlet("Run Server", "npm start", "bash");
        let reason = detect_match_reason_for_scriptlet(&sl, "bash");
        assert!(reason.is_some(), "Tool type match should produce a reason");
        let reason = reason.unwrap();
        assert!(
            reason.starts_with("tool: "),
            "Should start with 'tool: ', got: {}",
            reason
        );
    }

    #[test]
    fn test_scriptlet_match_reason_tool_not_shown_when_name_matches() {
        let sl = make_test_scriptlet("Bash Helper", "echo hi", "bash");
        // Name matches "bash" so no reason needed
        assert_eq!(detect_match_reason_for_scriptlet(&sl, "bash"), None);
    }

    // =========================================================================
    // Scriptlet match reason: description excerpt tests
    // =========================================================================

    #[test]
    fn test_scriptlet_match_reason_description_excerpt() {
        let mut sl = make_test_scriptlet("Quick Action", "echo done", "bash");
        sl.description = Some("Automates the deployment pipeline for staging".to_string());
        let reason = detect_match_reason_for_scriptlet(&sl, "deployment");
        assert!(reason.is_some());
        let reason = reason.unwrap();
        assert!(reason.starts_with("desc: "));
        assert!(reason.contains("deployment"));
    }


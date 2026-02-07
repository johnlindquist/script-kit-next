    #[test]
    fn test_command_slugify() {
        let cmd = Command::new(
            "My Cool Script!".to_string(),
            "ts".to_string(),
            "".to_string(),
        );
        assert_eq!(cmd.command, "my-cool-script");
    }
    #[test]
    fn test_command_extract_inputs() {
        let cmd = Command::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo {{name}} and {{value}}".to_string(),
        );
        assert_eq!(cmd.inputs, vec!["name", "value"]);
    }
    #[test]
    fn test_command_extract_inputs_no_duplicates() {
        let cmd = Command::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo {{name}} {{name}} {{name}}".to_string(),
        );
        assert_eq!(cmd.inputs, vec!["name"]);
    }
    #[test]
    fn test_command_extract_inputs_skip_conditionals() {
        let cmd = Command::new(
            "Test".to_string(),
            "bash".to_string(),
            "{{#if flag}}content{{/if}} {{else}} {{name}}".to_string(),
        );
        // Should only extract "name", not "#if", "/if", "else"
        assert_eq!(cmd.inputs, vec!["name"]);
    }
    #[test]
    fn test_command_is_shell() {
        let bash_cmd = Command::new("Test".to_string(), "bash".to_string(), "".to_string());
        assert!(bash_cmd.is_shell());

        let zsh_cmd = Command::new("Test".to_string(), "zsh".to_string(), "".to_string());
        assert!(zsh_cmd.is_shell());

        let ts_cmd = Command::new("Test".to_string(), "ts".to_string(), "".to_string());
        assert!(!ts_cmd.is_shell());
    }
    #[test]
    fn test_command_is_valid_tool() {
        let bash_cmd = Command::new("Test".to_string(), "bash".to_string(), "".to_string());
        assert!(bash_cmd.is_valid_tool());

        let ts_cmd = Command::new("Test".to_string(), "ts".to_string(), "".to_string());
        assert!(ts_cmd.is_valid_tool());

        let invalid_cmd = Command::new(
            "Test".to_string(),
            "invalid_tool".to_string(),
            "".to_string(),
        );
        assert!(!invalid_cmd.is_valid_tool());
    }
    #[test]
    fn test_command_default() {
        let cmd = Command::default();
        assert_eq!(cmd.tool, "ts");
        assert!(cmd.name.is_empty());
        assert!(cmd.content.is_empty());
    }
    // ========================================
    // CommandValidationError Tests
    // ========================================

    #[test]
    fn test_command_validation_error_display() {
        let err = CommandValidationError::new(
            "/path/to/file.md",
            Some("My Command".to_string()),
            Some(42),
            "No code block found",
        );
        let display = format!("{}", err);
        assert!(display.contains("/path/to/file.md"));
        assert!(display.contains(":42"));
        assert!(display.contains("[My Command]"));
        assert!(display.contains("No code block found"));
    }
    #[test]
    fn test_command_validation_error_display_minimal() {
        let err = CommandValidationError::new("/path/to/file.md", None, None, "Parse error");
        let display = format!("{}", err);
        assert_eq!(display, "/path/to/file.md: Parse error");
    }
    // ========================================
    // ExtensionParseResult Tests
    // ========================================

    #[test]
    fn test_extension_parse_result_default() {
        let result = ExtensionParseResult::default();
        assert!(result.commands.is_empty());
        assert!(result.errors.is_empty());
        assert!(result.manifest.is_none());
    }

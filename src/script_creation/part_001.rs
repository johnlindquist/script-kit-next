#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_sanitize_name_basic() {
        assert_eq!(sanitize_name("hello"), "hello");
        assert_eq!(sanitize_name("Hello World"), "hello-world");
        assert_eq!(sanitize_name("my_script_name"), "my-script-name");
    }

    #[test]
    fn test_sanitize_name_special_chars() {
        assert_eq!(sanitize_name("hello@world!"), "helloworld");
        assert_eq!(sanitize_name("test#$%script"), "testscript");
        assert_eq!(sanitize_name("foo & bar"), "foo-bar");
    }

    #[test]
    fn test_sanitize_name_multiple_hyphens() {
        assert_eq!(sanitize_name("hello---world"), "hello-world");
        assert_eq!(sanitize_name("a - b - c"), "a-b-c");
        assert_eq!(sanitize_name("  spaces  "), "spaces");
    }

    #[test]
    fn test_sanitize_name_leading_trailing() {
        assert_eq!(sanitize_name("-hello-"), "hello");
        assert_eq!(sanitize_name("---test---"), "test");
        assert_eq!(sanitize_name(" - hello - "), "hello");
    }

    #[test]
    fn test_sanitize_name_empty() {
        assert_eq!(sanitize_name(""), "");
        assert_eq!(sanitize_name("   "), "");
        assert_eq!(sanitize_name("@#$%"), "");
    }

    #[test]
    fn test_name_to_title_basic() {
        assert_eq!(name_to_title("hello"), "Hello");
        assert_eq!(name_to_title("hello-world"), "Hello World");
        assert_eq!(name_to_title("my-awesome-script"), "My Awesome Script");
    }

    #[test]
    fn test_name_to_title_edge_cases() {
        assert_eq!(name_to_title(""), "");
        assert_eq!(name_to_title("a"), "A");
        assert_eq!(name_to_title("a-b-c"), "A B C");
    }

    #[test]
    fn test_generate_script_template() {
        let template = generate_script_template("my-script");
        assert!(template.contains("import \"@scriptkit/sdk\";"));
        assert!(template.contains("export const metadata = {"));
        assert!(template.contains("name: \"My Script\""));
        assert!(template.contains("description: \"\""));
        assert!(template.contains("await arg("));
        assert!(template.contains("Template Guide"));
        assert!(template.contains("A) Input Prompt"));
        assert!(template.contains("B) Selection List"));
        assert!(template.contains("C) Background Task"));
    }

    #[test]
    fn test_generate_extension_template() {
        let template = generate_extension_template("my-extension");
        assert!(template.starts_with("---"));
        assert!(template.contains("name: My Extension"));
        assert!(template.contains("description: \"What this extension bundle includes\""));
        assert!(template.contains("icon: wrench"));
        assert!(template.contains("YAML frontmatter"));
        assert!(template.contains("# My Extension"));
        assert!(template.contains("Scriptlets in this bundle"));
        assert!(template.contains("```bash"));
        assert!(template.contains("~/.scriptkit/kit/GUIDE.md"));
    }

    #[test]
    fn test_create_new_script_empty_name() {
        let result = create_new_script("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("empty after sanitization"));
    }

    #[test]
    fn test_create_new_script_special_chars_only() {
        let result = create_new_script("@#$%^&*");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("empty after sanitization"));
    }

    #[test]
    fn test_create_new_extension_empty_name() {
        let result = create_new_extension("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("empty after sanitization"));
    }

    // Integration tests that actually create files
    // These use tempdir to avoid polluting the real scripts directory

    #[test]
    fn test_create_script_integration() {
        let temp_dir = tempdir().unwrap();
        let scripts_dir = temp_dir.path().join("scripts");
        let script_path = create_new_script_in_dir("test-script", &scripts_dir).unwrap();

        // Verify the file was created
        assert!(script_path.exists());
        assert_eq!(script_path.file_name().unwrap(), "test-script.ts");

        // Verify the content
        let content = fs::read_to_string(&script_path).unwrap();
        assert!(content.contains("export const metadata"));
        assert!(content.contains("Test Script"));
    }

    #[test]
    fn test_create_extension_integration() {
        let temp_dir = tempdir().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        let extension_path =
            create_new_extension_in_dir("test-extension", &extensions_dir).unwrap();

        // Verify the file was created
        assert!(extension_path.exists());
        assert_eq!(extension_path.file_name().unwrap(), "test-extension.md");

        // Verify the content is markdown with a code block
        let content = fs::read_to_string(&extension_path).unwrap();
        assert!(content.contains("# Test Extension"));
        assert!(content.contains("```bash"));
    }

    #[test]
    fn test_create_new_script_in_dir_generates_unique_name_when_base_exists() {
        let temp_dir = tempdir().unwrap();
        let scripts_dir = temp_dir.path().join("scripts");

        let first = create_new_script_in_dir("untitled", &scripts_dir).unwrap();
        let second = create_new_script_in_dir("untitled", &scripts_dir).unwrap();

        assert_eq!(first.file_name().unwrap(), "untitled.ts");
        assert_eq!(second.file_name().unwrap(), "untitled-1.ts");
        assert!(first.exists());
        assert!(second.exists());
    }

    #[test]
    fn test_create_new_extension_in_dir_generates_unique_name_when_base_exists() {
        let temp_dir = tempdir().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");

        let first = create_new_extension_in_dir("my-extension", &extensions_dir).unwrap();
        let second = create_new_extension_in_dir("my-extension", &extensions_dir).unwrap();

        assert_eq!(first.file_name().unwrap(), "my-extension.md");
        assert_eq!(second.file_name().unwrap(), "my-extension-1.md");
        assert!(first.exists());
        assert!(second.exists());
    }

    #[test]
    fn test_validate_sanitized_name_rejects_windows_reserved_name() {
        let err = validate_sanitized_name("CON", "con", "ts", "Script").unwrap_err();
        assert!(err.to_string().contains("reserved"));
    }

    #[test]
    fn test_create_new_script_in_dir_rejects_windows_reserved_name_after_sanitization() {
        let temp_dir = tempdir().unwrap();
        let scripts_dir = temp_dir.path().join("scripts");

        let err = create_new_script_in_dir("CON!!!", &scripts_dir).unwrap_err();
        assert!(err.to_string().contains("reserved on Windows"));
    }

    #[test]
    fn test_validate_sanitized_name_rejects_overlong_filename() {
        let long_name = "a".repeat(253);
        let err = validate_sanitized_name(&long_name, &long_name, "ts", "Script").unwrap_err();
        assert!(err.to_string().contains("too long"));
    }

    #[test]
    fn test_parse_editor_command_splits_flags_and_quotes() {
        let parts = parse_editor_command(r#"code --reuse-window --goto "src/main.rs:10""#).unwrap();
        assert_eq!(
            parts,
            vec![
                "code".to_string(),
                "--reuse-window".to_string(),
                "--goto".to_string(),
                "src/main.rs:10".to_string(),
            ]
        );
    }

    #[test]
    fn test_create_new_text_file_does_not_overwrite_existing_file() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("atomic-test.txt");

        create_new_text_file(&path, "first").unwrap();
        let err = create_new_text_file(&path, "second").unwrap_err();

        assert_eq!(err.kind(), ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(path).unwrap(), "first");
    }

    #[test]
    fn test_config_get_editor() {
        // Test that Config::get_editor works as expected
        let config = Config::default();

        // Save and clear EDITOR env var for predictable test
        let original_editor = env::var("EDITOR").ok();
        env::remove_var("EDITOR");

        // With no config editor and no EDITOR env, should return "code"
        let default_config = Config {
            hotkey: config.hotkey.clone(),
            bun_path: None,
            editor: None,
            padding: None,
            editor_font_size: None,
            terminal_font_size: None,
            ui_scale: None,
            built_ins: None,
            process_limits: None,
            clipboard_history_max_text_length: None,
            suggested: None,
            notes_hotkey: None,
            ai_hotkey: None,
            logs_hotkey: None,
            ai_hotkey_enabled: None,
            logs_hotkey_enabled: None,
            watcher: None,
            layout: None,
            commands: None,
            claude_code: None,
        };
        assert_eq!(default_config.get_editor(), "code");

        // With config editor set, should use that
        let custom_config = Config {
            hotkey: config.hotkey.clone(),
            bun_path: None,
            editor: Some("vim".to_string()),
            padding: None,
            editor_font_size: None,
            terminal_font_size: None,
            ui_scale: None,
            built_ins: None,
            process_limits: None,
            clipboard_history_max_text_length: None,
            suggested: None,
            notes_hotkey: None,
            ai_hotkey: None,
            logs_hotkey: None,
            ai_hotkey_enabled: None,
            logs_hotkey_enabled: None,
            watcher: None,
            layout: None,
            commands: None,
            claude_code: None,
        };
        assert_eq!(custom_config.get_editor(), "vim");

        // Restore original EDITOR
        if let Some(val) = original_editor {
            env::set_var("EDITOR", val);
        }
    }
}

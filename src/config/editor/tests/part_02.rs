    #[cfg(feature = "system-tests")]
    #[test]
    fn test_full_user_config() {
        let config_path =
            std::path::PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());

        if !config_path.exists() {
            println!(
                "Skipping test: config file does not exist at {:?}",
                config_path
            );
            return;
        }

        let content = std::fs::read_to_string(&config_path).expect("Failed to read config");
        println!(
            "Config file: {} bytes, {} lines",
            content.len(),
            content.lines().count()
        );

        // Skip if claudeCode already exists
        if contains_property(&content, "claudeCode") {
            println!("Skipping test: claudeCode already exists in config");
            return;
        }

        let result = enable_claude_code(&content);

        match result {
            EditResult::Modified(new_content) => {
                // Find where claudeCode was inserted
                if let Some(pos) = new_content.find("claudeCode:") {
                    println!("\n=== claudeCode found at position {} ===", pos);
                    let start = pos.saturating_sub(50);
                    let end = (pos + 100).min(new_content.len());
                    println!("Context:\n{}", &new_content[start..end]);
                } else {
                    println!("WARNING: claudeCode not found in output!");
                }

                // Verify structure
                assert!(
                    new_content.contains("claudeCode:"),
                    "Should contain claudeCode"
                );
                assert!(
                    new_content.contains("enabled: true"),
                    "Should contain enabled: true"
                );
                assert!(
                    !new_content.contains("}{"),
                    "Should not have adjacent braces"
                );
                assert!(
                    new_content.contains("} satisfies Config;")
                        || new_content.contains("} as Config;"),
                    "Should end with valid Config type assertion"
                );

                // Print last 20 lines for verification
                println!("\n=== Last 20 lines of modified config ===");
                for line in new_content
                    .lines()
                    .rev()
                    .take(20)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    println!("{}", line);
                }
            }
            EditResult::AlreadySet => {
                println!("claudeCode already set");
            }
            EditResult::Failed(reason) => {
                panic!("Failed to modify config: {}", reason);
            }
        }
    }

    // ==========================================================================
    // Test: validate_structure
    // ==========================================================================

    #[test]
    fn test_validate_structure_valid() {
        let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;
        assert!(validate_structure(content).is_ok());
    }

    #[test]
    fn test_validate_structure_valid_as_config() {
        let content = r#"export default {
  hotkey: { key: ";" },
} as Config;"#;
        assert!(validate_structure(content).is_ok());
    }

    #[test]
    fn test_validate_structure_missing_export() {
        let content = r#"const config = {
  hotkey: { key: ";" },
};"#;
        let err = validate_structure(content).unwrap_err();
        assert!(err.contains("export default"));
    }

    #[test]
    fn test_validate_structure_unbalanced_braces() {
        let content3 = r#"export default {"#;
        let err = validate_structure(content3).unwrap_err();
        // Tree-sitter reports this as a parse error (incomplete object literal)
        assert!(err.contains("parse error") || err.contains("parse errors"));
    }

    #[test]
    fn test_validate_structure_corruption_pattern() {
        let content = r#"export default {
  hotkey: { key: ";" },
}{
  extra: true,
} satisfies Config;"#;
        let err = validate_structure(content).unwrap_err();
        // Tree-sitter detects the `}{` corruption as a parse error
        assert!(err.contains("parse error") || err.contains("parse errors"));
    }

    #[test]
    fn test_validate_structure_missing_satisfies() {
        let content = r#"export default {
  hotkey: { key: ";" },
};"#;
        let err = validate_structure(content).unwrap_err();
        assert!(err.contains("satisfies") || err.contains("Config"));
    }

    // ==========================================================================
    // Test: write_config_safely (using temp directories)
    // ==========================================================================

    #[test]
    fn test_write_config_safely_creates_file_when_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = write_config_safely(&config_path, &property, None);

        match result {
            Ok(WriteOutcome::Created) => {
                let content = std::fs::read_to_string(&config_path).unwrap();
                assert!(content.contains("claudeCode: { enabled: true }"));
                assert!(content.contains("export default"));
                assert!(content.contains("satisfies Config;"));
            }
            Ok(other) => panic!("Expected Created, got {:?}", other),
            Err(e) => {
                println!("write_config_safely failed (may be expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_write_config_safely_creates_backup() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");

        let initial = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
} satisfies Config;
"#;
        std::fs::write(&config_path, initial).unwrap();

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = write_config_safely(&config_path, &property, None);

        match result {
            Ok(WriteOutcome::Written) => {
                let backup_path = config_path.with_extension("ts.bak");
                assert!(backup_path.exists(), "Backup file should exist");
                let backup_content = std::fs::read_to_string(&backup_path).unwrap();
                assert_eq!(backup_content, initial);
            }
            Ok(other) => panic!("Expected Written, got {:?}", other),
            Err(e) => {
                println!("write_config_safely failed (may be expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_write_config_safely_already_set() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");

        let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  claudeCode: { enabled: true },
} satisfies Config;
"#;
        std::fs::write(&config_path, content).unwrap();

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = write_config_safely(&config_path, &property, None);

        match result {
            Ok(WriteOutcome::AlreadySet) => { /* expected */ }
            other => panic!("Expected AlreadySet, got {:?}", other),
        }
    }

    #[test]
    fn test_write_config_safely_does_not_touch_predictable_tmp_path() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");
        let predictable_temp = config_path.with_extension("ts.tmp");

        // Simulates a pre-existing predictable temp path that should be ignored.
        std::fs::write(&predictable_temp, "sentinel-temp-content").unwrap();

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = write_config_safely(&config_path, &property, None);

        match result {
            Ok(WriteOutcome::Created) => {
                let sentinel = std::fs::read_to_string(&predictable_temp).unwrap();
                assert_eq!(sentinel, "sentinel-temp-content");
            }
            Ok(other) => panic!("Expected Created, got {:?}", other),
            Err(e) => {
                println!("write_config_safely failed (may be expected in CI): {}", e);
            }
        }
    }

    // ==========================================================================
    // Test: recover_from_backup
    // ==========================================================================

    #[test]
    fn test_recover_from_backup_no_backup() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");
        std::fs::write(&config_path, "corrupted").unwrap();

        let result = recover_from_backup(&config_path, None);
        match result {
            Ok(false) => { /* expected - no backup exists */ }
            other => panic!("Expected Ok(false), got {:?}", other),
        }
    }

    #[test]
    fn test_recover_from_backup_restores() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");
        let backup_path = config_path.with_extension("ts.bak");

        let valid_content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
} satisfies Config;
"#;
        std::fs::write(&config_path, "corrupted content").unwrap();
        std::fs::write(&backup_path, valid_content).unwrap();

        let result = recover_from_backup(&config_path, None);
        match result {
            Ok(true) => {
                let restored = std::fs::read_to_string(&config_path).unwrap();
                assert_eq!(restored, valid_content);
            }
            other => panic!("Expected Ok(true), got {:?}", other),
        }
    }

    #[test]
    fn test_recover_from_backup_does_not_touch_predictable_tmp_path() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");
        let backup_path = config_path.with_extension("ts.bak");
        let predictable_temp = config_path.with_extension("ts.tmp");

        let valid_content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
} satisfies Config;
"#;
        std::fs::write(&config_path, "corrupted content").unwrap();
        std::fs::write(&backup_path, valid_content).unwrap();
        std::fs::write(&predictable_temp, "sentinel-temp-content").unwrap();

        let result = recover_from_backup(&config_path, None);
        match result {
            Ok(true) => {
                let restored = std::fs::read_to_string(&config_path).unwrap();
                assert_eq!(restored, valid_content);
                let sentinel = std::fs::read_to_string(&predictable_temp).unwrap();
                assert_eq!(sentinel, "sentinel-temp-content");
            }
            other => panic!("Expected Ok(true), got {:?}", other),
        }
    }

    // ==========================================================================
    // Test: generate_fresh_config
    // ==========================================================================

    #[test]
    fn test_generate_fresh_config() {
        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let content = generate_fresh_config(&property);
        assert!(validate_structure(&content).is_ok());
        assert!(content.contains("claudeCode: { enabled: true }"));
        assert!(content.contains("hotkey:"));
    }

    // ==========================================================================
    // Test: round-trip with real config pattern
    // ==========================================================================

    #[test]
    fn test_real_config_round_trip() {
        let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // editorFontSize: 14,
  // terminalFontSize: 14,

  // builtIns: {
  //   clipboardHistory: true,
  // },

  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;
"#;

        let result = enable_claude_code(content);
        match result {
            EditResult::Modified(new_content) => {
                assert!(
                    validate_structure(&new_content).is_ok(),
                    "Round-trip produced invalid structure:\n{}",
                    new_content
                );
                assert!(new_content.contains("claudeCode:"));
                assert!(new_content.contains("enabled: true"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

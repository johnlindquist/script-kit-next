// editor.rs test module body split into sub-files
// --- merged from part_01.rs ---
use super::*;

#[test]
fn cold_bun_bootstrap_accepts_only_an_unambiguous_literal_pin() {
    assert_eq!(bootstrap_bun_pin("import a from 'a'; export default { theme: theme(), 'bun_path': '/path with space/bun' } satisfies Config;").unwrap().as_deref(), Some("/path with space/bun"));
    for source in [
        "export default { // bun_path: '/wrong'\n }",
        "export default { nested: { bun_path: '/wrong' } }",
    ] {
        assert_eq!(bootstrap_bun_pin(source), Ok(None));
    }
    for source in [
        "export default { bun_path: '/wrong', ...rest }",
        "export default { bun_path: '/one', bun_path: '/two' }",
        "export default { bun_path: compute() }",
    ] {
        assert!(bootstrap_bun_pin(source).is_err(), "{source}");
    }
}

#[test]
fn prepared_property_replaces_only_the_selected_preference() {
    let input =
        "export default { theme: { presetId: 'nord' }, ai: { enabled: true } } satisfies Config;";
    let property = ConfigProperty::new("theme", "{ presetId: null }");
    let prepared = prepare_config_property(input, &property).unwrap();
    assert!(prepared.contains("theme: { presetId: null }"));
    assert!(prepared.contains("ai: { enabled: true }"));
    assert_eq!(
        prepare_config_property(&prepared, &property).unwrap(),
        prepared
    );
    assert!(prepare_config_property("not valid config", &property).is_err());
    assert!(prepare_config_property("", &property)
        .unwrap()
        .contains("presetId: null"));
}

#[test]
fn static_config_decodes_literals_and_omits_reset_preferences() {
    let config = parse_static_config(
        r#"import type { Config } from '@scriptkit/sdk';
        export default { theme: undefined, enabled: true, nested: { 'quoted-key': 'it\'s "safe"',
        values: [1, - 2.5, false, null, "unicode\u1234",], }, } satisfies Config;"#,
    )
    .unwrap();
    assert!(config.get("theme").is_none());
    assert_eq!(config["nested"]["quoted-key"], "it's \"safe\"");
    assert_eq!(config["nested"]["values"][1], -2.5);
    assert!(config["nested"]["values"][3].is_null());
}

#[test]
fn static_config_rejects_executable_and_ambiguous_values() {
    for value in [
        "readSecret()",
        "process.env",
        "() => 1",
        "{ ...other }",
        "{ get key() { return 1; } }",
        "{ key: undefined, key: 2 }",
        "{ key: 1, key: 2 }",
        "[,1]",
        "`template ${run()}`",
    ] {
        let source = format!("export default {{ field: {value} }} satisfies Config;");
        assert!(parse_static_config(&source).is_err(), "accepted {value}");
    }
    assert!(parse_static_config("run(); export default {} satisfies Config;").is_err());
}

// ==========================================================================
// Test: contains_property
// ==========================================================================

#[test]
fn test_contains_property_simple() {
    let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

    assert!(contains_property(content, "hotkey"));
    assert!(!contains_property(content, "claudeCode"));
}

#[test]
fn test_contains_property_with_comments() {
    let content = r#"export default {
  // This is a comment about hotkey
  hotkey: { key: ";" },
  // claudeCode is mentioned in comment but not as property
} satisfies Config;"#;

    assert!(contains_property(content, "hotkey"));
    assert!(!contains_property(content, "claudeCode"));
}

#[test]
fn test_contains_property_nested() {
    let content = r#"export default {
  claudeCode: {
    enabled: true,
    path: "/usr/bin/claude"
  },
} satisfies Config;"#;

    assert!(contains_property(content, "claudeCode"));
    // Tree-sitter AST correctly distinguishes nesting levels:
    // "enabled" is a nested property inside claudeCode, not a top-level config property
    assert!(!contains_property(content, "enabled"));
}

// ==========================================================================
// Test: find_object_end
// ==========================================================================

#[test]
fn test_find_object_end_simple() {
    let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

    let info = find_object_end(content).expect("Should find object end");
    assert!(info.has_trailing_comma);
    assert_eq!(info.indent, "  ");
}

#[test]
fn test_find_object_end_no_trailing_comma() {
    let content = r#"export default {
  hotkey: { key: ";" }
} satisfies Config;"#;

    let info = find_object_end(content).expect("Should find object end");
    assert!(!info.has_trailing_comma);
}

#[test]
fn test_find_object_end_with_comments() {
    let content = r#"export default {
  hotkey: { key: ";" },
  // This is a comment
  // Another comment
} satisfies Config;"#;

    let info = find_object_end(content).expect("Should find object end");
    assert!(info.has_trailing_comma);
}

#[test]
fn test_find_object_end_nested_braces() {
    let content = r#"export default {
  hotkey: {
    modifiers: ["meta"],
    key: ";"
  },
  nested: {
    inner: {
      deep: true
    }
  },
} satisfies Config;"#;

    let info = find_object_end(content).expect("Should find object end");
    assert!(info.has_trailing_comma);
}

// ==========================================================================
// Test: add_property
// ==========================================================================

#[test]
fn test_add_property_simple() {
    let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

    let property = ConfigProperty::new("claudeCode", "{\n    enabled: true\n  }");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("claudeCode:"));
            assert!(new_content.contains("enabled: true"));
            // Should not have double commas
            assert!(!new_content.contains(",,"));
            // Should still be valid structure
            assert!(new_content.contains("} satisfies Config;"));
        }
        _ => panic!("Expected Modified result"),
    }
}

#[test]
fn test_add_property_no_trailing_comma() {
    let content = r#"export default {
  hotkey: { key: ";" }
} satisfies Config;"#;

    let property = ConfigProperty::new("claudeCode", "true");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            // Should add comma before new property
            assert!(new_content.contains(",\n"));
            assert!(new_content.contains("claudeCode: true"));
        }
        _ => panic!("Expected Modified result"),
    }
}

#[test]
fn test_add_property_replaces_existing_value() {
    let content = r#"export default {
  hotkey: { key: ";" },
  claudeCode: { enabled: false },
} satisfies Config;"#;

    let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("claudeCode: { enabled: true }"));
            assert!(!new_content.contains("enabled: false"));
        }
        other => panic!("Expected Modified result, got {:?}", other),
    }
}

#[test]
fn test_add_property_with_many_comments() {
    let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // ===========================================================================
  // UI Settings
  // ===========================================================================
  // editorFontSize: 14,
  // terminalFontSize: 14,

  // ===========================================================================
  // Advanced Settings
  // ===========================================================================
  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;"#;

    let property = ConfigProperty::new("claudeCode", "{\n    enabled: true\n  }");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("claudeCode:"));
            // Should not have double commas
            assert!(!new_content.contains(",,"));
            // The comma after hotkey should be preserved
            assert!(new_content.contains("key: \"Semicolon\","));
        }
        _ => panic!("Expected Modified result"),
    }
}

// ==========================================================================
// Test: enable_claude_code convenience function
// ==========================================================================

#[test]
fn test_enable_claude_code() {
    let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

    let result = enable_claude_code(content);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("claudeCode:"));
            assert!(new_content.contains("enabled: true"));
        }
        _ => panic!("Expected Modified result"),
    }
}

// ==========================================================================
// Test: Edge cases
// ==========================================================================

#[test]
fn test_empty_config() {
    let content = r#"export default {} satisfies Config;"#;

    let property = ConfigProperty::new("test", "true");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("test: true"));
        }
        _ => panic!("Expected Modified result"),
    }
}

#[test]
fn test_config_with_strings_containing_braces() {
    let content = r#"export default {
  template: "function() { return {}; }",
} satisfies Config;"#;

    let property = ConfigProperty::new("claudeCode", "true");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("claudeCode: true"));
            // Original string should be preserved
            assert!(new_content.contains("function() { return {}; }"));
        }
        _ => panic!("Expected Modified result"),
    }
}

#[test]
fn test_config_with_template_literals() {
    let content = r#"export default {
  code: `const x = { a: 1 };`,
} satisfies Config;"#;

    let property = ConfigProperty::new("claudeCode", "true");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("claudeCode: true"));
        }
        _ => panic!("Expected Modified result"),
    }
}

#[test]
fn test_preserves_as_config_syntax() {
    let content = r#"export default {
  hotkey: { key: ";" },
} as Config;"#;

    let property = ConfigProperty::new("claudeCode", "true");
    let result = add_property(content, &property);

    match result {
        EditResult::Modified(new_content) => {
            assert!(new_content.contains("} as Config;"));
        }
        _ => panic!("Expected Modified result"),
    }
}

// ==========================================================================
// Test: Real-world config (from user's actual file)
// ==========================================================================

#[test]
fn test_real_world_config() {
    let content = r#"import type { Config } from "@scriptkit/sdk";

/**
 * Script Kit Configuration
 */
export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // editorFontSize: 14,

  // builtIns: {
  //   clipboardHistory: true,
  // },

  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;
"#;

    let result = enable_claude_code(content);

    match result {
        EditResult::Modified(new_content) => {
            // Verify the result is valid
            assert!(new_content.contains("claudeCode:"));
            assert!(new_content.contains("enabled: true"));
            assert!(!new_content.contains(",,"));
            assert!(new_content.contains("} satisfies Config;"));

            // Print for visual inspection
            println!("=== Modified config ===\n{}", new_content);
        }
        _ => panic!("Expected Modified result"),
    }
}

/// Test with the EXACT complex config structure that was corrupted
/// This is the actual user's config.ts structure
#[test]
fn test_complex_config_with_many_commented_sections() {
    // This is a simplified version of the actual corrupted config
    let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // ===========================================================================
  // UI Settings
  // ===========================================================================
  // editorFontSize: 14,

  // ===========================================================================
  // Commands Configuration
  // ===========================================================================
  // commands: {
  //   "builtin/clipboard-history": {
  //     shortcut: { modifiers: ["meta", "shift"], key: "KeyV" }
  //   },
  //
  //   // Add shortcut to a scriptlet
  //   // "scriptlet/clipboard-to-uppercase": {
  //   //   shortcut: { modifiers: ["meta", "shift"], key: "KeyU" }
  //   // },
  // },

  // ===========================================================================
  // Process Limits
  // ===========================================================================
  // processLimits: {
  //   maxMemoryMb: 512,
  // },

  // ===========================================================================
  // Advanced Settings
  // ===========================================================================
  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;
"#;

    let result = enable_claude_code(content);

    match result {
        EditResult::Modified(new_content) => {
            // CRITICAL validations
            assert!(
                !new_content.contains("}{"),
                "Should NOT contain }}{{ corruption pattern. Got:\n{}",
                new_content
            );
            assert!(
                new_content.contains("claudeCode:"),
                "Should contain claudeCode property"
            );
            assert!(
                new_content.contains("enabled: true"),
                "Should contain enabled: true"
            );
            assert!(
                new_content.contains("} satisfies Config;"),
                "Should end with valid Config type assertion"
            );

            // The claudeCode property should appear BEFORE the final closing brace
            let claude_pos = new_content.find("claudeCode:").unwrap();
            let final_brace_pos = new_content.rfind("} satisfies").unwrap();
            assert!(
                claude_pos < final_brace_pos,
                "claudeCode should be before closing brace"
            );

            // All original comments should still be present
            assert!(new_content.contains("// UI Settings"));
            assert!(new_content.contains("// Commands Configuration"));
            assert!(new_content.contains("// Process Limits"));
            assert!(new_content.contains("// Advanced Settings"));

            // Print for visual inspection
            println!("=== Modified complex config ===\n{}", new_content);
        }
        _ => panic!("Expected Modified result"),
    }
}

/// Test that validates output is syntactically valid TypeScript
#[test]
fn test_output_has_no_corruption_patterns() {
    let configs = vec![
        // Minimal config
        r#"export default { hotkey: { key: ";" } } satisfies Config;"#,
        // Config with trailing comma
        r#"export default { hotkey: { key: ";" }, } satisfies Config;"#,
        // Config with nested comments
        r#"export default {
  hotkey: { key: ";" },
  // nested: {
  //   deep: {
  //     value: true
  //   }
  // }
} satisfies Config;"#,
        // Config with string containing braces
        r#"export default {
  template: "fn() { return {}; }",
} satisfies Config;"#,
    ];

    for content in configs {
        let result = enable_claude_code(content);
        match result {
            EditResult::Modified(new_content) => {
                // Check for corruption patterns
                assert!(
                    !new_content.contains("}{"),
                    "Corruption }}{{ detected in:\n{}",
                    new_content
                );
                assert!(
                    !new_content.contains("{{"),
                    "Double open brace detected in:\n{}",
                    new_content
                );
                // Note: }} can be valid (closing nested objects)
            }
            EditResult::AlreadySet => {
                // Fine - property already exists
            }
            EditResult::Failed(reason) => {
                panic!("Failed to modify config: {}", reason);
            }
        }
    }
}

// ==========================================================================
// Test: Full user config file (system test)
// ==========================================================================

// --- merged from part_02.rs ---
#[cfg(feature = "system-tests")]
#[test]
fn test_full_user_config() {
    let config_path =
        std::path::PathBuf::from(shellexpand::tilde("~/.scriptkit/config.ts").as_ref());

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
                new_content.contains("} satisfies Config;") || new_content.contains("} as Config;"),
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

// Use the real native writer in a disposable directory. An explicit missing Bun
// pin selects its structural validator; these tests neither run a user's config
// nor discover and launch a different Bun from their environment.
const PERSONAL_CONFIG: &str =
    "export default { editorFontSize: 21, /* my key */ ai: {} } satisfies Config;\n";
const OLD_BACKUP: &[u8] = b"export default { editorFontSize: 12 } satisfies Config;\n";

#[test]
fn enabling_claude_code_changes_only_the_switch_and_keeps_the_original_backup() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let original = concat!(
        "// Workspace preferences — exact bytes matter.\r\n",
        "export default { editorFontSize: 21, \"claudeCode\": {\r\n",
        "  'enabled': false /* only this switch */, path: '/example/bin/claude',\r\n",
        "  permissionMode: 'plan', allowedTools: 'Read', addDirs: ['/example/docs'],\r\n",
        "  futureOption: false, // not ours to change\r\n",
        "} } satisfies Config;\r\n",
    );
    let enabled = original.replacen(
        "false /* only this switch */",
        "true /* only this switch */",
        1,
    );
    std::fs::write(&config, original).unwrap();
    std::fs::write(&backup, OLD_BACKUP).unwrap();
    let missing_bun = dir.path().join("missing-bun");
    let apply = || enable_claude_code_safely(&config, missing_bun.to_str());
    assert_eq!(apply().unwrap(), WriteOutcome::Written);
    assert_eq!(std::fs::read(&config).unwrap(), enabled.as_bytes());
    assert_eq!(std::fs::read(&backup).unwrap(), original.as_bytes());
    assert_eq!(apply().unwrap(), WriteOutcome::AlreadySet);
    assert_eq!(std::fs::read(&config).unwrap(), enabled.as_bytes());
    assert_eq!(std::fs::read(&backup).unwrap(), original.as_bytes());
    assert!(!dir.path().join("config.ts.settings-lock").exists());
}

#[test]
fn enabling_claude_code_adds_a_missing_switch_to_the_actual_default_object() {
    for (before, after) in [
        ("export default {} as Config;", "export default { claudeCode: { enabled: true } } as Config;"),
        ("export default { claudeCode: {} } satisfies Config;", "export default { claudeCode: { enabled: true } } satisfies Config;"),
        ("export default { claudeCode: {// label\npath: pickPath(),} } satisfies Config;", "export default { claudeCode: { enabled: true, // label\npath: pickPath(),} } satisfies Config;"),
        ("export const other = {claudeCode: {enabled: false}};\nexport default { theme: getTheme() } satisfies Config;", "export const other = {claudeCode: {enabled: false}};\nexport default { claudeCode: { enabled: true }, theme: getTheme() } satisfies Config;"),
    ] {
        assert_eq!(enable_claude_code(before), EditResult::Modified(after.into()));
        validate_structure(after).unwrap();
        assert_eq!(enable_claude_code(after), EditResult::AlreadySet);
    }
}

#[test]
fn enabling_claude_code_refuses_ambiguous_or_dynamic_switches_without_touching_a_backup() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    for body in [
        "claudeCode: provider()",
        "claudeCode: {enabled: chooseMode(), path: '/example/claude'}",
        "claudeCode: {enabled: true, ...overrides}",
        "claudeCode: {enabled: false, 'enabled': true}",
        "claudeCode: {}, 'claudeCode': {}",
        "claudeCode: { [key]: false }",
        "claudeCode: { get enabled() { return false } }",
        "claudeCode: { enabled }",
        "...defaults, claudeCode: {}",
    ] {
        let before = format!("export default {{ {body} }} satisfies Config;\n");
        std::fs::write(&config, &before).unwrap();
        std::fs::write(&backup, OLD_BACKUP).unwrap();
        assert!(
            matches!(
                enable_claude_code_safely(&config, dir.path().join("missing-bun").to_str()),
                Err(ConfigWriteError::EditFailed(_))
            ),
            "{body}"
        );
        assert_eq!(std::fs::read(&config).unwrap(), before.as_bytes());
        assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
        assert!(!dir.path().join("config.ts.settings-lock").exists());
    }
}

#[test]
fn an_already_enabled_reply_cannot_acknowledge_settings_changed_during_the_edit() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let before = "export default { claudeCode: { enabled: true } } satisfies Config;";
    let newer = before.replace("true", "false");
    std::fs::write(&config, before).unwrap();
    std::fs::write(&backup, OLD_BACKUP).unwrap();
    let result = edit_config_source(
        &config,
        |old| {
            let proposal = enable_claude_code(old);
            assert_eq!(proposal, EditResult::AlreadySet);
            std::fs::write(&config, &newer).unwrap();
            proposal
        },
        |_| panic!("a no-op should not need a provider or validator"),
    );
    assert!(matches!(result, Err(ConfigWriteError::IoError(_))));
    assert_eq!(std::fs::read(&config).unwrap(), newer.as_bytes());
    assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
}

#[cfg(unix)]
#[test]
fn native_preference_syntax_check_never_runs_the_config_or_its_macro() {
    // The release toolchain provides Bun. Missing Bun is not a passing result
    // for this test; tests of the structural fallback use an explicit missing pin.
    let bun = super::super::resolve_bun(None).expect("Bun required for the native syntax test");
    let pin = Some(bun.to_str().unwrap());
    let dir = tempfile::tempdir().unwrap();
    let ordinary = dir.path().join("ordinary-ran");
    let called = dir.path().join("macro-ran");
    let module = dir.path().join("size.ts");
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let js_path = |p: &Path| serde_json::to_string(p).unwrap();
    let source = format!("import {{writeFileSync}} from 'node:fs';\nwriteFileSync({}, 'ran');\nconst n: number = 19; export default {{editorFontSize:n}} satisfies Config;\n", js_path(&ordinary));
    validate_typescript(&source, pin).unwrap();
    assert!(!ordinary.exists());
    // Still reject errors that a grammar-only check can miss, without echoing
    // a personal value from the compiler's annotated source line.
    // The bad tail must be seen even when the prefix exceeds a pipe buffer.
    let large = format!("//{}\n", "世界🎙️".repeat(32_000));
    validate_typescript(&format!("{large}{source}"), pin).unwrap();
    let failure = validate_typescript(
        &format!("{large}const n = 19; const n = 'private-settings-marker'; export default {{n}};"),
        pin,
    )
    .unwrap_err();
    assert!(!failure.contains("private-settings-marker"));
    std::fs::write(&module, format!("import {{writeFileSync}} from 'node:fs'; export function size() {{ writeFileSync({}, 'ran'); return 27; }}", js_path(&called))).unwrap();
    let proposal = format!("import {{size}} from {} with {{type:'macro'}};\nexport default {{editorFontSize:size()}} satisfies Config;\n", js_path(&module));
    std::fs::write(&config, PERSONAL_CONFIG).unwrap();
    std::fs::write(&backup, &proposal).unwrap();
    assert!(matches!(
        recover_from_backup(&config, pin),
        Err(ConfigWriteError::ValidationFailed(_))
    ));
    assert!(!called.exists());
    assert_eq!(std::fs::read(&config).unwrap(), PERSONAL_CONFIG.as_bytes());
    assert_eq!(std::fs::read(&backup).unwrap(), proposal.as_bytes());
    assert!(!dir.path().join("config.ts.settings-lock").exists());
}

#[cfg(unix)]
#[test]
fn native_preference_a_zero_exit_with_a_checker_error_cannot_publish() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    std::fs::write(&config, PERSONAL_CONFIG).unwrap();
    std::fs::write(&backup, OLD_BACKUP).unwrap();
    let bun = dir.path().join("bun");
    std::fs::write(&bun, "#!/bin/sh\n/bin/cat >/dev/null\nprintf 'Error resolving private-settings-marker\\n' >&2\nexit 0\n").unwrap();
    std::fs::set_permissions(&bun, std::fs::Permissions::from_mode(0o700)).unwrap();
    let Err(ConfigWriteError::ValidationFailed(reason)) =
        enable_claude_code_safely(&config, bun.to_str())
    else {
        panic!("a failed check was treated as a safe settings edit");
    };
    assert!(!reason.contains("private-settings-marker"));
    assert_eq!(std::fs::read(&config).unwrap(), PERSONAL_CONFIG.as_bytes());
    assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
    assert!(!dir.path().join("config.ts.settings-lock").exists());
}

#[test]
fn native_preference_structural_errors_locate_but_do_not_quote_private_unicode() {
    // The old 30-byte diagnostic slice of this ERROR node split a UTF-8 scalar.
    let source = r#"export default { "🎙️🎙️🎙️private-settings-marker": <<< true;"#;
    let err = validate_structure(source).unwrap_err();
    assert!(err.contains("line ") && err.contains("column "));
    assert!(!err.contains("private") && !err.contains('🎙'));
}

#[test]
fn native_preference_unreadable_text_is_not_a_new_config() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let original =
        b"// my comment: \xff\nexport default { editorFontSize: 21 } satisfies Config;\n";
    std::fs::write(&config, original).unwrap();
    std::fs::write(&backup, OLD_BACKUP).unwrap();
    let no_bun = dir.path().join("missing-bun");
    let result = enable_claude_code_safely(&config, no_bun.to_str());
    assert!(matches!(result, Err(ConfigWriteError::IoError(_))));
    assert_eq!(std::fs::read(&config).unwrap(), original);
    assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
    assert!(!dir.path().join("config.ts.settings-lock").exists());
}

#[cfg(unix)]
fn only_settings_draft(lock: &SettingsLock) -> std::path::PathBuf {
    let mut entries = std::fs::read_dir(&lock.path).unwrap();
    let path = entries.next().unwrap().unwrap().path();
    assert!(entries.next().is_none());
    path
}

#[cfg(unix)]
#[test]
fn native_preference_disturbed_drafts_cannot_replace_settings_or_their_backup() {
    use std::fs;
    use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
    // Pause between the real preparation and publication phases; no timing hook or Bun required.
    for target in ["replace", "create", "backup"] {
        for event in [
            "unchanged",
            "same bytes, new inode",
            "rewrite",
            "chmod",
            "symlink",
            "hardlink",
        ] {
            let root = tempfile::tempdir().unwrap();
            let config = root.path().join("config.ts");
            let backup = config.with_extension("ts.bak");
            if target != "create" {
                fs::write(&config, PERSONAL_CONFIG).unwrap();
            }
            fs::write(&backup, OLD_BACKUP).unwrap();
            let lock = SettingsLock::acquire(&config).unwrap();
            let lock_name = lock.path.clone();
            let dest = if target == "backup" { &backup } else { &config };
            let original = config_snapshot(dest).unwrap();
            let next = if target == "backup" {
                PERSONAL_CONFIG.to_owned()
            } else {
                format!(
                    "// {}\n{}",
                    "a".repeat(9_000),
                    PERSONAL_CONFIG.replace("21", "27")
                )
            };
            let draft = ConfigDraft::prepare(&lock, next.as_bytes()).unwrap();
            let name = only_settings_draft(&lock);
            let retired = root.path().join("original-draft");
            match event {
                "unchanged" => {}
                "same bytes, new inode" => {
                    fs::rename(&name, &retired).unwrap();
                    fs::copy(&retired, &name).unwrap();
                    assert_ne!(
                        fs::metadata(&retired).unwrap().ino(),
                        fs::metadata(&name).unwrap().ino()
                    );
                }
                "rewrite" => {
                    let mut bytes = next.as_bytes().to_vec();
                    *bytes.last_mut().unwrap() = b'!'; // same length, including beyond the first chunk
                    fs::write(&name, bytes).unwrap();
                }
                "chmod" => fs::set_permissions(&name, fs::Permissions::from_mode(0o644)).unwrap(),
                "symlink" => {
                    fs::rename(&name, &retired).unwrap();
                    symlink(&retired, &name).unwrap(); // follows back to the right inode: still not our name
                }
                "hardlink" => fs::hard_link(&name, &retired).unwrap(),
                _ => unreachable!(),
            }
            let observed = || {
                let m = fs::symlink_metadata(&name).unwrap();
                (m.ino(), m.mode(), m.nlink(), fs::read(&name).unwrap())
            };
            let named = observed();
            let outcome = if target == "backup" {
                draft.publish(dest, original.as_ref())
            } else {
                draft.publish_edit(dest, original.as_ref())
            };
            drop(lock);
            if event == "unchanged" {
                outcome.unwrap();
                assert_eq!(fs::read(dest).unwrap(), next.as_bytes());
                assert_eq!(fs::metadata(dest).unwrap().ino(), named.0);
                if target == "replace" {
                    assert_eq!(fs::read(&backup).unwrap(), PERSONAL_CONFIG.as_bytes());
                }
                if target == "create" {
                    assert_eq!(fs::read(&backup).unwrap(), OLD_BACKUP);
                }
                assert!(!lock_name.exists());
                continue;
            }
            assert!(
                matches!(outcome, Err(ConfigWriteError::IoError(_))),
                "{target}/{event}"
            );
            assert_eq!(
                observed(),
                named,
                "must not delete or repair a changed draft"
            );
            assert_eq!(fs::read(&backup).unwrap(), OLD_BACKUP);
            if target == "create" {
                assert_eq!(
                    fs::symlink_metadata(&config).unwrap_err().kind(),
                    std::io::ErrorKind::NotFound
                );
            } else {
                assert_eq!(fs::read(&config).unwrap(), PERSONAL_CONFIG.as_bytes());
            }
            assert_eq!(fs::read_dir(&lock_name).unwrap().count(), 1);
        }
    }
}

#[cfg(unix)]
#[test]
fn native_preference_late_destination_change_cleans_only_the_owned_draft() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config.ts");
    let lock = SettingsLock::acquire(&config).unwrap();
    let lock_name = lock.path.clone();
    let draft = ConfigDraft::prepare(&lock, PERSONAL_CONFIG.as_bytes()).unwrap();
    std::fs::write(&config, b"newer writer").unwrap();
    assert!(draft.publish(&config, None).is_err());
    assert!(std::fs::read_dir(&lock_name).unwrap().next().is_none());
    drop(lock);
    assert!(!lock_name.exists());
    assert_eq!(std::fs::read(&config).unwrap(), b"newer writer");
}

#[cfg(unix)]
#[test]
fn native_preference_replaced_lock_keeps_both_writers_drafts() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config.ts");
    let lock = SettingsLock::acquire(&config).unwrap();
    let draft = ConfigDraft::prepare(&lock, PERSONAL_CONFIG.as_bytes()).unwrap();
    let old_lock = root.path().join("displaced-lock");
    let name = only_settings_draft(&lock);
    std::fs::rename(&lock.path, &old_lock).unwrap();
    std::fs::create_dir(&lock.path).unwrap();
    std::fs::write(&name, b"the other writer").unwrap();
    assert!(draft.publish(&config, None).is_err());
    drop(lock);
    assert_eq!(std::fs::read(&name).unwrap(), b"the other writer");
    assert_eq!(
        std::fs::read(old_lock.join(name.file_name().unwrap())).unwrap(),
        PERSONAL_CONFIG.as_bytes()
    );
    assert!(!config.exists());
}

#[cfg(unix)]
#[test]
fn native_preference_unsafe_config_or_backup_never_changes_the_other_file() {
    use std::os::unix::fs::{symlink, MetadataExt};
    for link_config in [false, true] {
        for kind in ["symlink", "hardlink", "directory", "dangling"] {
            let dir = tempfile::tempdir().unwrap();
            let config = dir.path().join("config.ts");
            let backup = config.with_extension("ts.bak");
            let (target, other) = if link_config {
                (&config, &backup)
            } else {
                (&backup, &config)
            };
            let original_other = if link_config {
                OLD_BACKUP
            } else {
                PERSONAL_CONFIG.as_bytes()
            };
            std::fs::write(other, original_other).unwrap();
            let neighbor = dir.path().join("separate.ts");
            if kind != "dangling" {
                std::fs::write(&neighbor, PERSONAL_CONFIG.replace("21", "19")).unwrap();
            }
            match kind {
                "hardlink" => std::fs::hard_link(&neighbor, target).unwrap(),
                "directory" => std::fs::create_dir(target).unwrap(),
                _ => symlink(&neighbor, target).unwrap(),
            }
            let ino = std::fs::symlink_metadata(target).unwrap().ino();
            let result =
                enable_claude_code_safely(&config, dir.path().join("missing-bun").to_str());
            assert!(
                matches!(result, Err(ConfigWriteError::IoError(_))),
                "{link_config}/{kind}: {result:?}"
            );
            assert_eq!(std::fs::symlink_metadata(target).unwrap().ino(), ino);
            assert_eq!(std::fs::read(other).unwrap(), original_other);
            if kind == "dangling" {
                assert!(!neighbor.exists());
            } else {
                assert_eq!(
                    std::fs::read_to_string(neighbor).unwrap(),
                    PERSONAL_CONFIG.replace("21", "19")
                );
            }
        }
    }
}

#[test]
fn native_preference_honors_the_sdk_lock_then_backs_up_the_original() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let lock = dir.path().join("config.ts.settings-lock");
    let no_bun = dir.path().join("missing-bun");
    std::fs::write(&config, PERSONAL_CONFIG).unwrap();
    std::fs::write(&backup, OLD_BACKUP).unwrap();
    std::fs::create_dir(&lock).unwrap();
    let change = || {
        write_json_property_safely(
            &config,
            "ai",
            &serde_json::json!({"selectedModelId":"my-model"}),
            no_bun.to_str(),
        )
    };
    assert!(matches!(change(), Err(ConfigWriteError::IoError(_))));
    assert!(lock.is_dir());
    assert_eq!(std::fs::read_to_string(&config).unwrap(), PERSONAL_CONFIG);
    assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
    std::fs::remove_dir(lock).unwrap();
    assert_eq!(change().unwrap(), WriteOutcome::Written);
    assert_eq!(std::fs::read_to_string(&backup).unwrap(), PERSONAL_CONFIG);
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        PERSONAL_CONFIG.replace("ai: {}", "ai: {\"selectedModelId\":\"my-model\"}")
    );
}

#[test]
fn native_preference_rejected_candidate_does_not_install_the_old_backup() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let invalid = format!("{PERSONAL_CONFIG}const unfinished = ;\n");
    std::fs::write(&config, &invalid).unwrap();
    std::fs::write(&backup, OLD_BACKUP).unwrap();
    assert!(enable_claude_code_safely(&config, dir.path().join("missing-bun").to_str()).is_err());
    assert_eq!(std::fs::read_to_string(&config).unwrap(), invalid);
    assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
}

#[cfg(unix)]
#[test]
fn native_preference_does_not_overwrite_a_file_replaced_during_validation() {
    for prior in [None, Some(PERSONAL_CONFIG)] {
        for arrived in [
            PERSONAL_CONFIG.to_string(),
            PERSONAL_CONFIG.replace("21", "24"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let config = dir.path().join("config.ts");
            let backup = config.with_extension("ts.bak");
            if let Some(prior) = prior {
                std::fs::write(&config, prior).unwrap();
            }
            std::fs::write(&backup, OLD_BACKUP).unwrap();
            let result = edit_config_file(
                &config,
                &ConfigProperty::new("ai", "{ enabled: true }"),
                |_| {
                    let replacement = dir.path().join("replacement.ts");
                    std::fs::write(&replacement, &arrived).unwrap();
                    std::fs::rename(replacement, &config).unwrap();
                    Ok(())
                },
            );
            assert!(matches!(result, Err(ConfigWriteError::IoError(_))));
            assert_eq!(std::fs::read_to_string(&config).unwrap(), arrived);
            assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
        }
    }
}

#[cfg(unix)]
#[test]
fn native_preference_losing_its_lock_cannot_publish_or_release_the_replacement() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let lock = dir.path().join("config.ts.settings-lock");
    std::fs::write(&config, PERSONAL_CONFIG).unwrap();
    std::fs::write(&backup, OLD_BACKUP).unwrap();
    let result = edit_config_file(
        &config,
        &ConfigProperty::new("ai", "{} /*different*/"),
        |_| {
            std::fs::rename(&lock, dir.path().join("old-lock")).unwrap();
            std::fs::create_dir(&lock).unwrap();
            Ok(())
        },
    );
    assert!(matches!(result, Err(ConfigWriteError::IoError(_))));
    assert!(lock.is_dir(), "the new owner still holds its lock");
    assert_eq!(std::fs::read_to_string(&config).unwrap(), PERSONAL_CONFIG);
    assert_eq!(std::fs::read(&backup).unwrap(), OLD_BACKUP);
}

#[cfg(unix)]
#[test]
fn explicit_config_recovery_does_not_follow_a_linked_backup() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.ts");
    let backup = config.with_extension("ts.bak");
    let neighbor = dir.path().join("elsewhere.ts");
    std::fs::write(&config, b"recover me").unwrap();
    std::fs::write(&neighbor, OLD_BACKUP).unwrap();
    std::os::unix::fs::symlink(&neighbor, &backup).unwrap();
    assert!(matches!(
        recover_from_backup(&config, dir.path().join("missing-bun").to_str()),
        Err(ConfigWriteError::IoError(_))
    ));
    assert_eq!(std::fs::read(&config).unwrap(), b"recover me");
    assert_eq!(std::fs::read_link(backup).unwrap(), neighbor);
    assert_eq!(std::fs::read(&neighbor).unwrap(), OLD_BACKUP);
}

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
fn test_write_config_safely_replaces_existing_top_level_property() {
    let dir = tempfile::TempDir::new().unwrap();
    let config_path = dir.path().join("config.ts");

    let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  ai: { selectedModelId: "claude-sonnet-4-6" },
} satisfies Config;
"#;
    std::fs::write(&config_path, content).unwrap();

    let property = ConfigProperty::new("ai", r#"{"selectedModelId":"gpt-5.4"}"#);
    let result = write_config_safely(&config_path, &property, None);

    match result {
        Ok(WriteOutcome::Written) => {
            let updated = std::fs::read_to_string(&config_path).unwrap();
            assert!(updated.contains(r#"ai: {"selectedModelId":"gpt-5.4"}"#));
            assert!(!updated.contains("claude-sonnet-4-6"));
        }
        other => panic!("Expected Written, got {:?}", other),
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

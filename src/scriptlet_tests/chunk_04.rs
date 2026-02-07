// ========================================
// Interpreter Tool Tests
// ========================================

#[test]
fn test_interpreter_tools_constant() {
    // Verify all expected interpreters are in the list
    assert!(INTERPRETER_TOOLS.contains(&"python"));
    assert!(INTERPRETER_TOOLS.contains(&"ruby"));
    assert!(INTERPRETER_TOOLS.contains(&"perl"));
    assert!(INTERPRETER_TOOLS.contains(&"php"));
    assert!(INTERPRETER_TOOLS.contains(&"node"));

    // Verify count
    assert_eq!(INTERPRETER_TOOLS.len(), 5);
}

#[test]
fn test_is_interpreter_tool() {
    // Positive cases
    assert!(is_interpreter_tool("python"));
    assert!(is_interpreter_tool("ruby"));
    assert!(is_interpreter_tool("perl"));
    assert!(is_interpreter_tool("php"));
    assert!(is_interpreter_tool("node"));

    // Negative cases - shell tools
    assert!(!is_interpreter_tool("bash"));
    assert!(!is_interpreter_tool("sh"));
    assert!(!is_interpreter_tool("zsh"));

    // Negative cases - other tools
    assert!(!is_interpreter_tool("ts"));
    assert!(!is_interpreter_tool("kit"));
    assert!(!is_interpreter_tool("open"));
    assert!(!is_interpreter_tool("paste"));
    assert!(!is_interpreter_tool("unknown"));
}

#[test]
fn test_get_interpreter_command() {
    // Python uses python3
    assert_eq!(get_interpreter_command("python"), "python3");

    // Others use their direct name
    assert_eq!(get_interpreter_command("ruby"), "ruby");
    assert_eq!(get_interpreter_command("perl"), "perl");
    assert_eq!(get_interpreter_command("php"), "php");
    assert_eq!(get_interpreter_command("node"), "node");

    // Unknown returns as-is
    assert_eq!(get_interpreter_command("unknown"), "unknown");
}

#[test]
fn test_get_interpreter_extension() {
    assert_eq!(get_interpreter_extension("python"), "py");
    assert_eq!(get_interpreter_extension("ruby"), "rb");
    assert_eq!(get_interpreter_extension("perl"), "pl");
    assert_eq!(get_interpreter_extension("php"), "php");
    assert_eq!(get_interpreter_extension("node"), "js");

    // Unknown returns txt
    assert_eq!(get_interpreter_extension("unknown"), "txt");
}

#[test]
fn test_validate_interpreter_tool_valid() {
    assert!(validate_interpreter_tool("python").is_ok());
    assert!(validate_interpreter_tool("ruby").is_ok());
    assert!(validate_interpreter_tool("perl").is_ok());
    assert!(validate_interpreter_tool("php").is_ok());
    assert!(validate_interpreter_tool("node").is_ok());
}

#[test]
fn test_validate_interpreter_tool_non_interpreter() {
    // bash is valid but not an interpreter tool
    let result = validate_interpreter_tool("bash");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not an interpreter tool"));
}

#[test]
fn test_validate_interpreter_tool_unknown() {
    let result = validate_interpreter_tool("unknown_tool");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not a recognized tool type"));
}

#[test]
fn test_interpreter_not_found_message_python() {
    let msg = interpreter_not_found_message("python3");

    // Should contain the tool name
    assert!(msg.contains("Python"));
    assert!(msg.contains("interpreter not found"));

    // Should have installation instructions
    #[cfg(target_os = "macos")]
    {
        assert!(msg.contains("brew install python"));
    }
    #[cfg(target_os = "linux")]
    {
        assert!(msg.contains("apt install python3") || msg.contains("dnf install python3"));
    }
    #[cfg(target_os = "windows")]
    {
        assert!(msg.contains("choco install python"));
    }

    // Should mention restart
    assert!(msg.contains("restart Script Kit"));
}

#[test]
fn test_interpreter_not_found_message_ruby() {
    let msg = interpreter_not_found_message("ruby");

    assert!(msg.contains("Ruby"));
    assert!(msg.contains("interpreter not found"));

    #[cfg(target_os = "macos")]
    {
        assert!(msg.contains("brew install ruby"));
    }
}

#[test]
fn test_interpreter_not_found_message_node() {
    let msg = interpreter_not_found_message("node");

    assert!(msg.contains("Node.js"));
    assert!(msg.contains("interpreter not found"));

    #[cfg(target_os = "macos")]
    {
        assert!(msg.contains("brew install node"));
    }
}

#[test]
fn test_interpreter_not_found_message_perl() {
    let msg = interpreter_not_found_message("perl");

    assert!(msg.contains("Perl"));
    assert!(msg.contains("interpreter not found"));
}

#[test]
fn test_interpreter_not_found_message_php() {
    let msg = interpreter_not_found_message("php");

    assert!(msg.contains("PHP"));
    assert!(msg.contains("interpreter not found"));
}

#[test]
fn test_interpreter_tools_are_valid_tools() {
    // All interpreter tools should also be in VALID_TOOLS
    for tool in INTERPRETER_TOOLS {
        assert!(
            VALID_TOOLS.contains(tool),
            "Interpreter tool '{}' should be in VALID_TOOLS",
            tool
        );
    }
}

#[test]
fn test_interpreter_tools_disjoint_from_shell_tools() {
    // Interpreter tools should not overlap with shell tools
    for tool in INTERPRETER_TOOLS {
        assert!(
            !SHELL_TOOLS.contains(tool),
            "Interpreter tool '{}' should not be in SHELL_TOOLS",
            tool
        );
    }
}

#[test]
fn test_scriptlet_with_interpreter_tool() {
    // Test creating a scriptlet with an interpreter tool
    let scriptlet = Scriptlet::new(
        "Python Script".to_string(),
        "python".to_string(),
        "print('Hello, World!')".to_string(),
    );

    assert_eq!(scriptlet.tool, "python");
    assert!(is_interpreter_tool(&scriptlet.tool));
    assert!(scriptlet.is_valid_tool());
    assert!(!scriptlet.is_shell());
}

#[test]
fn test_parse_markdown_with_interpreter_tools() {
    let markdown = r#"# Scripts

## Python Hello

```python
print("Hello from Python")
```

## Ruby Greeting

```ruby
puts "Hello from Ruby"
```

## Node Script

```node
console.log("Hello from Node");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);

    assert_eq!(scriptlets.len(), 3);

    // Python
    assert_eq!(scriptlets[0].tool, "python");
    assert!(is_interpreter_tool(&scriptlets[0].tool));
    assert!(scriptlets[0].scriptlet_content.contains("print"));

    // Ruby
    assert_eq!(scriptlets[1].tool, "ruby");
    assert!(is_interpreter_tool(&scriptlets[1].tool));
    assert!(scriptlets[1].scriptlet_content.contains("puts"));

    // Node
    assert_eq!(scriptlets[2].tool, "node");
    assert!(is_interpreter_tool(&scriptlets[2].tool));
    assert!(scriptlets[2].scriptlet_content.contains("console.log"));
}

#[test]
fn test_interpreter_extension_matches_tool_extension() {
    // get_interpreter_extension should match the tool_extension for interpreter tools
    // This ensures consistency between the two functions
    assert_eq!(get_interpreter_extension("python"), "py");
    assert_eq!(get_interpreter_extension("ruby"), "rb");
    assert_eq!(get_interpreter_extension("perl"), "pl");
    assert_eq!(get_interpreter_extension("php"), "php");
    assert_eq!(get_interpreter_extension("node"), "js");
}

// ========================================
// Cron and Schedule Metadata Tests
// ========================================

#[test]
fn test_parse_metadata_cron_basic() {
    let metadata = parse_html_comment_metadata("<!-- cron: */5 * * * * -->");
    assert_eq!(metadata.cron, Some("*/5 * * * *".to_string()));
}

#[test]
fn test_parse_metadata_cron_hourly() {
    let metadata = parse_html_comment_metadata("<!-- cron: 0 * * * * -->");
    assert_eq!(metadata.cron, Some("0 * * * *".to_string()));
}

#[test]
fn test_parse_metadata_cron_daily() {
    let metadata = parse_html_comment_metadata("<!-- cron: 0 9 * * * -->");
    assert_eq!(metadata.cron, Some("0 9 * * *".to_string()));
}

#[test]
fn test_parse_metadata_cron_weekly() {
    let metadata = parse_html_comment_metadata("<!-- cron: 0 9 * * 1 -->");
    assert_eq!(metadata.cron, Some("0 9 * * 1".to_string()));
}

#[test]
fn test_parse_metadata_schedule_natural_language() {
    let metadata = parse_html_comment_metadata("<!-- schedule: every hour -->");
    assert_eq!(metadata.schedule, Some("every hour".to_string()));
}

#[test]
fn test_parse_metadata_schedule_every_tuesday() {
    let metadata = parse_html_comment_metadata("<!-- schedule: every tuesday at 2pm -->");
    assert_eq!(metadata.schedule, Some("every tuesday at 2pm".to_string()));
}

#[test]
fn test_parse_metadata_schedule_every_day() {
    let metadata = parse_html_comment_metadata("<!-- schedule: every day at 9am -->");
    assert_eq!(metadata.schedule, Some("every day at 9am".to_string()));
}

#[test]
fn test_parse_metadata_cron_with_other_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\ncron: 0 */6 * * *\ndescription: Runs every 6 hours\nbackground: true\n-->",
    );
    assert_eq!(metadata.cron, Some("0 */6 * * *".to_string()));
    assert_eq!(metadata.description, Some("Runs every 6 hours".to_string()));
    assert_eq!(metadata.background, Some(true));
}

#[test]
fn test_parse_metadata_schedule_with_other_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\nschedule: every weekday at 9am\ndescription: Morning task\nbackground: true\n-->",
    );
    assert_eq!(metadata.schedule, Some("every weekday at 9am".to_string()));
    assert_eq!(metadata.description, Some("Morning task".to_string()));
    assert_eq!(metadata.background, Some(true));
}

#[test]
fn test_parse_metadata_cron_and_schedule_together() {
    // Both can exist, though typically only one would be used
    let metadata =
        parse_html_comment_metadata("<!--\ncron: 0 9 * * *\nschedule: every day at 9am\n-->");
    assert_eq!(metadata.cron, Some("0 9 * * *".to_string()));
    assert_eq!(metadata.schedule, Some("every day at 9am".to_string()));
}

#[test]
fn test_parse_metadata_cron_empty_value() {
    // Empty cron value should not be stored
    let metadata = parse_html_comment_metadata("<!-- cron: -->");
    assert_eq!(metadata.cron, None);
}

#[test]
fn test_parse_metadata_schedule_empty_value() {
    // Empty schedule value should not be stored
    let metadata = parse_html_comment_metadata("<!-- schedule: -->");
    assert_eq!(metadata.schedule, None);
}

#[test]
fn test_parse_markdown_scriptlet_with_cron() {
    let markdown = r#"## Hourly Backup

<!-- cron: 0 * * * * -->

```bash
backup.sh
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Hourly Backup");
    assert_eq!(scriptlets[0].metadata.cron, Some("0 * * * *".to_string()));
    assert_eq!(scriptlets[0].tool, "bash");
}

#[test]
fn test_parse_markdown_scriptlet_with_schedule() {
    let markdown = r#"## Weekly Report

<!-- schedule: every monday at 8am -->

```bash
generate-report.sh
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Weekly Report");
    assert_eq!(
        scriptlets[0].metadata.schedule,
        Some("every monday at 8am".to_string())
    );
    assert_eq!(scriptlets[0].tool, "bash");
}

#[test]
fn test_parse_markdown_multiple_scriptlets_with_cron_and_schedule() {
    let markdown = r#"# Scheduled Tasks

## Every 5 Minutes Check

<!-- cron: */5 * * * * -->

```bash
check-status.sh
```

## Daily Cleanup

<!-- schedule: every day at midnight -->

```bash
cleanup.sh
```

## No Schedule

```bash
manual-task.sh
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 3);

    assert_eq!(scriptlets[0].metadata.cron, Some("*/5 * * * *".to_string()));
    assert_eq!(scriptlets[0].metadata.schedule, None);

    assert_eq!(scriptlets[1].metadata.cron, None);
    assert_eq!(
        scriptlets[1].metadata.schedule,
        Some("every day at midnight".to_string())
    );

    assert_eq!(scriptlets[2].metadata.cron, None);
    assert_eq!(scriptlets[2].metadata.schedule, None);
}

#[test]
fn test_cron_metadata_serialization() {
    let metadata = ScriptletMetadata {
        cron: Some("0 9 * * 1-5".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"cron\":\"0 9 * * 1-5\""));

    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.cron, Some("0 9 * * 1-5".to_string()));
}

#[test]
fn test_schedule_metadata_serialization() {
    let metadata = ScriptletMetadata {
        schedule: Some("every friday at 5pm".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"schedule\":\"every friday at 5pm\""));

    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.schedule,
        Some("every friday at 5pm".to_string())
    );
}

#[test]
fn test_cron_metadata_deserialization_missing() {
    // When cron is not present in JSON, it should be None
    let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
    let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.cron, None);
}

#[test]
fn test_cron_complex_expression() {
    // Test parsing complex cron expressions with ranges and lists
    let metadata = parse_html_comment_metadata("<!-- cron: 0 9,12,18 * * 1-5 -->");
    assert_eq!(metadata.cron, Some("0 9,12,18 * * 1-5".to_string()));
}

#[test]
fn test_cron_six_field_expression() {
    // Some cron parsers support seconds as the first field
    let metadata = parse_html_comment_metadata("<!-- cron: 0 30 9 * * * -->");
    assert_eq!(metadata.cron, Some("0 30 9 * * *".to_string()));
}


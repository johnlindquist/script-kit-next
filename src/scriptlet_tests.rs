use super::*;

// ========================================
// Type and Constant Tests
// ========================================

#[test]
fn test_valid_tools_contains_common_tools() {
    assert!(VALID_TOOLS.contains(&"bash"));
    assert!(VALID_TOOLS.contains(&"python"));
    assert!(VALID_TOOLS.contains(&"ts"));
    assert!(VALID_TOOLS.contains(&"js"));
    assert!(VALID_TOOLS.contains(&"kit"));
    assert!(VALID_TOOLS.contains(&"paste"));
    assert!(VALID_TOOLS.contains(&"template"));
}

#[test]
fn test_shell_tools_contains_shells() {
    assert!(SHELL_TOOLS.contains(&"bash"));
    assert!(SHELL_TOOLS.contains(&"zsh"));
    assert!(SHELL_TOOLS.contains(&"sh"));
    assert!(SHELL_TOOLS.contains(&"fish"));
    assert!(SHELL_TOOLS.contains(&"powershell"));
}

#[test]
fn test_scriptlet_new_basic() {
    let scriptlet = Scriptlet::new(
        "My Test Script".to_string(),
        "bash".to_string(),
        "echo hello".to_string(),
    );

    assert_eq!(scriptlet.name, "My Test Script");
    assert_eq!(scriptlet.command, "my-test-script");
    assert_eq!(scriptlet.tool, "bash");
    assert_eq!(scriptlet.scriptlet_content, "echo hello");
    assert!(scriptlet.inputs.is_empty());
}

#[test]
fn test_scriptlet_new_with_inputs() {
    let scriptlet = Scriptlet::new(
        "Test".to_string(),
        "ts".to_string(),
        "const name = '{{name}}'; const age = {{age}};".to_string(),
    );

    assert_eq!(scriptlet.inputs.len(), 2);
    assert!(scriptlet.inputs.contains(&"name".to_string()));
    assert!(scriptlet.inputs.contains(&"age".to_string()));
}

#[test]
fn test_scriptlet_is_shell() {
    let bash = Scriptlet::new("test".to_string(), "bash".to_string(), "echo".to_string());
    let ts = Scriptlet::new(
        "test".to_string(),
        "ts".to_string(),
        "console.log()".to_string(),
    );

    assert!(bash.is_shell());
    assert!(!ts.is_shell());
}

#[test]
fn test_scriptlet_is_valid_tool() {
    let valid = Scriptlet::new("test".to_string(), "bash".to_string(), "echo".to_string());
    let invalid = Scriptlet::new(
        "test".to_string(),
        "invalid_tool".to_string(),
        "echo".to_string(),
    );

    assert!(valid.is_valid_tool());
    assert!(!invalid.is_valid_tool());
}

// ========================================
// Slugify Tests
// ========================================

#[test]
fn test_slugify_basic() {
    assert_eq!(slugify("Hello World"), "hello-world");
    assert_eq!(slugify("My Script"), "my-script");
}

#[test]
fn test_slugify_special_chars() {
    assert_eq!(slugify("Hello, World!"), "hello-world");
    assert_eq!(slugify("test@123"), "test-123");
}

#[test]
fn test_slugify_multiple_spaces() {
    assert_eq!(slugify("Hello   World"), "hello-world");
    assert_eq!(slugify("  Leading Trailing  "), "leading-trailing");
}

// ========================================
// Extract Named Inputs Tests
// ========================================

#[test]
fn test_extract_named_inputs_basic() {
    let inputs = extract_named_inputs("Hello {{name}}!");
    assert_eq!(inputs, vec!["name"]);
}

#[test]
fn test_extract_named_inputs_multiple() {
    let inputs = extract_named_inputs("{{first}} and {{second}}");
    assert_eq!(inputs.len(), 2);
    assert!(inputs.contains(&"first".to_string()));
    assert!(inputs.contains(&"second".to_string()));
}

#[test]
fn test_extract_named_inputs_no_duplicates() {
    let inputs = extract_named_inputs("{{name}} is {{name}}");
    assert_eq!(inputs, vec!["name"]);
}

#[test]
fn test_extract_named_inputs_ignores_conditionals() {
    let inputs = extract_named_inputs("{{#if flag}}{{name}}{{/if}}");
    assert_eq!(inputs, vec!["name"]);
    assert!(!inputs.contains(&"#if flag".to_string()));
    assert!(!inputs.contains(&"/if".to_string()));
}

#[test]
fn test_extract_named_inputs_empty() {
    let inputs = extract_named_inputs("No placeholders here");
    assert!(inputs.is_empty());
}

// ========================================
// Metadata Parsing Tests
// ========================================

#[test]
fn test_parse_metadata_basic() {
    let metadata = parse_html_comment_metadata("<!-- shortcut: cmd k -->");
    assert_eq!(metadata.shortcut, Some("cmd k".to_string()));
}

#[test]
fn test_parse_metadata_multiple_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\nshortcut: cmd k\ndescription: My script\ntrigger: test\n-->",
    );
    assert_eq!(metadata.shortcut, Some("cmd k".to_string()));
    assert_eq!(metadata.description, Some("My script".to_string()));
    assert_eq!(metadata.trigger, Some("test".to_string()));
}

#[test]
fn test_parse_metadata_background_bool() {
    let metadata = parse_html_comment_metadata("<!-- background: true -->");
    assert_eq!(metadata.background, Some(true));

    let metadata = parse_html_comment_metadata("<!-- background: false -->");
    assert_eq!(metadata.background, Some(false));
}

#[test]
fn test_parse_metadata_extra_fields() {
    let metadata = parse_html_comment_metadata("<!-- custom_field: value -->");
    assert_eq!(
        metadata.extra.get("custom_field"),
        Some(&"value".to_string())
    );
}

#[test]
fn test_parse_metadata_empty() {
    let metadata = parse_html_comment_metadata("No comments here");
    assert!(metadata.shortcut.is_none());
    assert!(metadata.description.is_none());
}

#[test]
fn test_parse_metadata_colons_in_value() {
    let metadata =
        parse_html_comment_metadata("<!-- description: Visit https://example.com for info -->");
    assert_eq!(
        metadata.description,
        Some("Visit https://example.com for info".to_string())
    );
}

// ========================================
// Expand Metadata Tests
// ========================================

#[test]
fn test_parse_metadata_keyword_basic() {
    let metadata = parse_html_comment_metadata("<!-- keyword: :sig -->");
    assert_eq!(metadata.keyword, Some(":sig".to_string()));
}

#[test]
fn test_parse_metadata_keyword_with_punctuation() {
    let metadata = parse_html_comment_metadata("<!-- keyword: !email -->");
    assert_eq!(metadata.keyword, Some("!email".to_string()));
}

#[test]
fn test_parse_metadata_keyword_with_double_suffix() {
    // Common pattern: keyword followed by double char like "ddate,,"
    let metadata = parse_html_comment_metadata("<!-- keyword: ddate,, -->");
    assert_eq!(metadata.keyword, Some("ddate,,".to_string()));
}

#[test]
fn test_parse_metadata_keyword_with_other_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\nkeyword: :addr\nshortcut: cmd e\ndescription: Insert address\n-->",
    );
    assert_eq!(metadata.keyword, Some(":addr".to_string()));
    assert_eq!(metadata.shortcut, Some("cmd e".to_string()));
    assert_eq!(metadata.description, Some("Insert address".to_string()));
}

#[test]
fn test_parse_metadata_keyword_empty_value() {
    // Empty expand value should not be stored
    let metadata = parse_html_comment_metadata("<!-- keyword: -->");
    assert_eq!(metadata.keyword, None);
}

#[test]
fn test_parse_markdown_scriptlet_with_keyword() {
    let markdown = r#"## Email Signature

<!-- keyword: :sig -->

```type
Best regards,
John Doe
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Email Signature");
    assert_eq!(scriptlets[0].metadata.keyword, Some(":sig".to_string()));
    assert_eq!(scriptlets[0].tool, "type");
}

#[test]
fn test_parse_markdown_multiple_scriptlets_with_keyword() {
    let markdown = r#"# Snippets

## Date Insert

<!-- keyword: :date -->

```type
{{date}}
```

## Email Template

<!-- keyword: !email -->

```type
Hello {{name}},
```

## No Expand

```type
Plain text
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 3);

    assert_eq!(scriptlets[0].metadata.keyword, Some(":date".to_string()));
    assert_eq!(scriptlets[1].metadata.keyword, Some("!email".to_string()));
    assert_eq!(scriptlets[2].metadata.keyword, None);
}

#[test]
fn test_keyword_metadata_serialization() {
    let metadata = ScriptletMetadata {
        keyword: Some(":test".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"keyword\":\":test\""));

    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.keyword, Some(":test".to_string()));
}

#[test]
fn test_keyword_metadata_deserialization_missing() {
    // When expand is not present in JSON, it should be None
    let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
    let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.keyword, None);
}

// ========================================
// Alias Metadata Tests
// ========================================

#[test]
fn test_parse_metadata_alias_basic() {
    let metadata = parse_html_comment_metadata("<!-- alias: goog -->");
    assert_eq!(metadata.alias, Some("goog".to_string()));
}

#[test]
fn test_parse_metadata_alias_with_punctuation() {
    let metadata = parse_html_comment_metadata("<!-- alias: g! -->");
    assert_eq!(metadata.alias, Some("g!".to_string()));
}

#[test]
fn test_parse_metadata_alias_with_numbers() {
    let metadata = parse_html_comment_metadata("<!-- alias: cmd123 -->");
    assert_eq!(metadata.alias, Some("cmd123".to_string()));
}

#[test]
fn test_parse_metadata_alias_with_other_fields() {
    let metadata = parse_html_comment_metadata(
        "<!--\nalias: search\nshortcut: cmd s\ndescription: Search the web\n-->",
    );
    assert_eq!(metadata.alias, Some("search".to_string()));
    assert_eq!(metadata.shortcut, Some("cmd s".to_string()));
    assert_eq!(metadata.description, Some("Search the web".to_string()));
}

#[test]
fn test_parse_metadata_alias_empty_value() {
    // Empty alias value should not be stored
    let metadata = parse_html_comment_metadata("<!-- alias: -->");
    assert_eq!(metadata.alias, None);
}

#[test]
fn test_parse_markdown_scriptlet_with_alias() {
    let markdown = r#"## Google Search

<!-- alias: goog -->

```bash
open "https://www.google.com/search?q=$1"
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Google Search");
    assert_eq!(scriptlets[0].metadata.alias, Some("goog".to_string()));
    assert_eq!(scriptlets[0].tool, "bash");
}

#[test]
fn test_parse_markdown_multiple_scriptlets_with_alias() {
    let markdown = r#"# Launchers

## Google Search

<!-- alias: goog -->

```open
https://google.com
```

## GitHub

<!-- alias: gh -->

```open
https://github.com
```

## No Alias

```open
https://example.com
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 3);

    assert_eq!(scriptlets[0].metadata.alias, Some("goog".to_string()));
    assert_eq!(scriptlets[1].metadata.alias, Some("gh".to_string()));
    assert_eq!(scriptlets[2].metadata.alias, None);
}

#[test]
fn test_alias_metadata_serialization() {
    let metadata = ScriptletMetadata {
        alias: Some("test".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"alias\":\"test\""));

    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.alias, Some("test".to_string()));
}

#[test]
fn test_alias_metadata_deserialization_missing() {
    // When alias is not present in JSON, it should be None
    let json = r#"{"trigger":null,"shortcut":null,"schedule":null,"background":null,"watch":null,"system":null,"description":null}"#;
    let metadata: ScriptletMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.alias, None);
}

#[test]
fn test_alias_and_keyword_together() {
    // Both alias and expand can coexist on the same scriptlet
    let metadata = parse_html_comment_metadata("<!--\nalias: goog\nkeyword: :google\n-->");
    assert_eq!(metadata.alias, Some("goog".to_string()));
    assert_eq!(metadata.keyword, Some(":google".to_string()));
}

// ========================================
// Code Block Extraction Tests
// ========================================

#[test]
fn test_extract_code_block_basic_backticks() {
    let result = extract_code_block_nested("```ts\nconst x = 1;\n```");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "ts");
    assert_eq!(code, "const x = 1;");
}

#[test]
fn test_extract_code_block_basic_tildes() {
    let result = extract_code_block_nested("~~~bash\necho hello\n~~~");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "bash");
    assert_eq!(code, "echo hello");
}

#[test]
fn test_extract_code_block_nested_backticks_in_tildes() {
    let content = "~~~md\nHere's code:\n```ts\nconst x = 1;\n```\nDone!\n~~~";
    let result = extract_code_block_nested(content);
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "md");
    assert!(code.contains("```ts"));
    assert!(code.contains("const x = 1;"));
}

#[test]
fn test_extract_code_block_no_language() {
    let result = extract_code_block_nested("```\ncode here\n```");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "");
    assert_eq!(code, "code here");
}

#[test]
fn test_extract_code_block_none_without_fence() {
    let result = extract_code_block_nested("No code fence here");
    assert!(result.is_none());
}

#[test]
fn test_extract_code_block_multiline() {
    let result = extract_code_block_nested("```python\ndef foo():\n    return 42\n```");
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "python");
    assert!(code.contains("def foo():"));
    assert!(code.contains("return 42"));
}

// ========================================
// Markdown Parsing Tests
// ========================================

#[test]
fn test_parse_markdown_basic_scriptlet() {
    let markdown = r#"## Test Script

```ts
console.log("hello");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].name, "Test Script");
    assert_eq!(scriptlets[0].tool, "ts");
    assert!(scriptlets[0].scriptlet_content.contains("console.log"));
}

#[test]
fn test_parse_markdown_with_group() {
    let markdown = r#"# My Group

## Script One

```bash
echo one
```

## Script Two

```bash
echo two
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 2);
    assert_eq!(scriptlets[0].group, "My Group");
    assert_eq!(scriptlets[1].group, "My Group");
}

#[test]
fn test_parse_markdown_with_metadata() {
    let markdown = r#"## Shortcut Script

<!-- shortcut: cmd k -->

```ts
console.log("triggered");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert_eq!(scriptlets[0].metadata.shortcut, Some("cmd k".to_string()));
}

#[test]
fn test_parse_markdown_with_global_prepend() {
    let markdown = r#"# Shell Scripts

```bash
#!/bin/bash
set -e
```

## Script A

```bash
echo "A"
```

## Script B

```bash
echo "B"
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 2);

    // Both should have the prepended content
    assert!(scriptlets[0].scriptlet_content.contains("#!/bin/bash"));
    assert!(scriptlets[0].scriptlet_content.contains("set -e"));
    assert!(scriptlets[0].scriptlet_content.contains("echo \"A\""));

    assert!(scriptlets[1].scriptlet_content.contains("#!/bin/bash"));
    assert!(scriptlets[1].scriptlet_content.contains("echo \"B\""));
}

#[test]
fn test_parse_markdown_default_tool() {
    let markdown = r#"## No Language

```
just code
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    // Empty tool should default to "ts"
    assert_eq!(scriptlets[0].tool, "ts");
}

#[test]
fn test_parse_markdown_extracts_inputs() {
    let markdown = r#"## Template

```ts
console.log("Hello {{name}}! You are {{age}} years old.");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);
    assert!(scriptlets[0].inputs.contains(&"name".to_string()));
    assert!(scriptlets[0].inputs.contains(&"age".to_string()));
}

#[test]
fn test_parse_markdown_source_path() {
    let markdown = "## Test\n\n```bash\necho\n```";
    let scriptlets = parse_markdown_as_scriptlets(markdown, Some("/path/to/file.md"));
    assert_eq!(
        scriptlets[0].source_path,
        Some("/path/to/file.md".to_string())
    );
}

#[test]
fn test_parse_markdown_empty() {
    let scriptlets = parse_markdown_as_scriptlets("", None);
    assert!(scriptlets.is_empty());
}

#[test]
fn test_parse_markdown_no_code_block() {
    let markdown = "## Title\n\nJust text, no code.";
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert!(scriptlets.is_empty());
}

// ========================================
// Variable Substitution Tests
// ========================================

#[test]
fn test_format_scriptlet_named_inputs() {
    let mut inputs = HashMap::new();
    inputs.insert("name".to_string(), "Alice".to_string());
    inputs.insert("greeting".to_string(), "Hello".to_string());

    let result = format_scriptlet("{{greeting}}, {{name}}!", &inputs, &[], false);

    assert_eq!(result, "Hello, Alice!");
}

#[test]
fn test_format_scriptlet_positional_unix() {
    let result = format_scriptlet(
        "echo $1 and $2",
        &HashMap::new(),
        &["first".to_string(), "second".to_string()],
        false,
    );

    assert_eq!(result, "echo first and second");
}

#[test]
fn test_format_scriptlet_positional_windows() {
    let result = format_scriptlet(
        "echo %1 and %2",
        &HashMap::new(),
        &["first".to_string(), "second".to_string()],
        true,
    );

    assert_eq!(result, "echo first and second");
}

#[test]
fn test_format_scriptlet_all_args_unix() {
    let result = format_scriptlet(
        "echo $@",
        &HashMap::new(),
        &["one".to_string(), "two".to_string(), "three".to_string()],
        false,
    );

    assert_eq!(result, r#"echo "one" "two" "three""#);
}

#[test]
fn test_format_scriptlet_all_args_windows() {
    let result = format_scriptlet(
        "echo %*",
        &HashMap::new(),
        &["one".to_string(), "two".to_string()],
        true,
    );

    assert_eq!(result, r#"echo "one" "two""#);
}

#[test]
fn test_format_scriptlet_combined() {
    let mut inputs = HashMap::new();
    inputs.insert("prefix".to_string(), "Result:".to_string());

    let result = format_scriptlet(
        "{{prefix}} $1 and $2",
        &inputs,
        &["A".to_string(), "B".to_string()],
        false,
    );

    assert_eq!(result, "Result: A and B");
}

#[test]
fn test_format_scriptlet_escape_quotes() {
    let result = format_scriptlet(
        "echo $@",
        &HashMap::new(),
        &["has\"quote".to_string()],
        false,
    );

    assert_eq!(result, r#"echo "has\"quote""#);
}

// ========================================
// Conditional Processing Tests
// ========================================

#[test]
fn test_process_conditionals_if_true() {
    let mut flags = HashMap::new();
    flags.insert("show".to_string(), true);

    let result = process_conditionals("{{#if show}}visible{{/if}}", &flags);
    assert_eq!(result, "visible");
}

#[test]
fn test_process_conditionals_if_false() {
    let mut flags = HashMap::new();
    flags.insert("show".to_string(), false);

    let result = process_conditionals("{{#if show}}visible{{/if}}", &flags);
    assert_eq!(result, "");
}

#[test]
fn test_process_conditionals_if_missing_flag() {
    let flags = HashMap::new();

    let result = process_conditionals("{{#if undefined}}visible{{/if}}", &flags);
    assert_eq!(result, "");
}

#[test]
fn test_process_conditionals_if_else_true() {
    let mut flags = HashMap::new();
    flags.insert("flag".to_string(), true);

    let result = process_conditionals("{{#if flag}}yes{{else}}no{{/if}}", &flags);
    assert_eq!(result, "yes");
}

#[test]
fn test_process_conditionals_if_else_false() {
    let mut flags = HashMap::new();
    flags.insert("flag".to_string(), false);

    let result = process_conditionals("{{#if flag}}yes{{else}}no{{/if}}", &flags);
    assert_eq!(result, "no");
}

#[test]
fn test_process_conditionals_else_if() {
    let mut flags = HashMap::new();
    flags.insert("a".to_string(), false);
    flags.insert("b".to_string(), true);

    let result = process_conditionals("{{#if a}}A{{else if b}}B{{else}}C{{/if}}", &flags);
    assert_eq!(result, "B");
}

#[test]
fn test_process_conditionals_nested() {
    let mut flags = HashMap::new();
    flags.insert("outer".to_string(), true);
    flags.insert("inner".to_string(), true);

    let result = process_conditionals("{{#if outer}}[{{#if inner}}nested{{/if}}]{{/if}}", &flags);
    assert_eq!(result, "[nested]");
}

#[test]
fn test_process_conditionals_preserves_other_content() {
    let mut flags = HashMap::new();
    flags.insert("show".to_string(), true);

    let result = process_conditionals("Before {{#if show}}middle{{/if}} after", &flags);
    assert_eq!(result, "Before middle after");
}

#[test]
fn test_process_conditionals_with_variables() {
    let mut flags = HashMap::new();
    flags.insert("useTitle".to_string(), true);

    let result = process_conditionals("{{#if useTitle}}Hello {{name}}{{/if}}", &flags);
    assert_eq!(result, "Hello {{name}}");
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_full_scriptlet_workflow() {
    let markdown = r#"# Tools

## Greeter

<!-- 
description: Greets a person
shortcut: cmd g
-->

```ts
const name = "{{name}}";
{{#if formal}}console.log(`Dear ${name}`);{{else}}console.log(`Hey ${name}!`);{{/if}}
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, Some("/test.md"));
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "Greeter");
    assert_eq!(scriptlet.group, "Tools");
    assert_eq!(
        scriptlet.metadata.description,
        Some("Greets a person".to_string())
    );
    assert_eq!(scriptlet.metadata.shortcut, Some("cmd g".to_string()));
    assert!(scriptlet.inputs.contains(&"name".to_string()));

    // Test variable substitution
    let mut inputs = HashMap::new();
    inputs.insert("name".to_string(), "Alice".to_string());

    let mut flags = HashMap::new();
    flags.insert("formal".to_string(), true);

    let content = process_conditionals(&scriptlet.scriptlet_content, &flags);
    let result = format_scriptlet(&content, &inputs, &[], false);

    assert!(result.contains("Alice"));
    assert!(result.contains("Dear"));
    assert!(!result.contains("Hey"));
}

#[test]
fn test_complex_markdown_parsing() {
    let markdown = r#"# Productivity

## Open URL

<!-- shortcut: cmd u -->

```open
https://example.com
```

## Type Date

<!-- keyword: ddate,, -->

```type
{{#if iso}}{{date}}{{else}}{{formattedDate}}{{/if}}
```

# Development

```bash
# Common setup
export PATH="$HOME/bin:$PATH"
```

## Run Tests

```bash
npm test $@
```

## Build

```bash
npm run build $1
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);

    // Should have 4 scriptlets: Open URL, Type Date, Run Tests, Build
    assert_eq!(scriptlets.len(), 4);

    // First two belong to "Productivity" group
    assert_eq!(scriptlets[0].group, "Productivity");
    assert_eq!(scriptlets[0].name, "Open URL");
    assert_eq!(scriptlets[0].tool, "open");

    assert_eq!(scriptlets[1].group, "Productivity");
    assert_eq!(scriptlets[1].name, "Type Date");
    assert_eq!(scriptlets[1].metadata.keyword, Some("ddate,,".to_string()));

    // Last two belong to "Development" group and have the common setup prepended
    assert_eq!(scriptlets[2].group, "Development");
    assert_eq!(scriptlets[2].name, "Run Tests");
    assert!(scriptlets[2].scriptlet_content.contains("export PATH"));
    assert!(scriptlets[2].scriptlet_content.contains("npm test"));

    assert_eq!(scriptlets[3].group, "Development");
    assert_eq!(scriptlets[3].name, "Build");
    assert!(scriptlets[3].scriptlet_content.contains("export PATH"));
}

#[test]
fn test_scriptlet_metadata_serialization() {
    let metadata = ScriptletMetadata {
        shortcut: Some("cmd k".to_string()),
        description: Some("Test".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ScriptletMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.shortcut, deserialized.shortcut);
    assert_eq!(metadata.description, deserialized.description);
}

#[test]
fn test_scriptlet_serialization() {
    let scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo hello".to_string(),
    );

    let json = serde_json::to_string(&scriptlet).unwrap();
    let deserialized: Scriptlet = serde_json::from_str(&json).unwrap();

    assert_eq!(scriptlet.name, deserialized.name);
    assert_eq!(scriptlet.tool, deserialized.tool);
    assert_eq!(scriptlet.scriptlet_content, deserialized.scriptlet_content);
}

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

// ========================================
// Codefence Metadata Integration Tests
// ========================================

#[test]
fn test_scriptlet_with_codefence_metadata() {
    // Test that scriptlets can be parsed from markdown with codefence metadata blocks
    let markdown = r#"## Quick Todo

```metadata
{ "name": "Quick Todo", "description": "Add a todo item", "shortcut": "cmd t" }
```

```ts
const item = await arg("Todo item");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "Quick Todo");
    assert_eq!(scriptlet.tool, "ts");

    // Typed metadata should be populated
    assert!(scriptlet.typed_metadata.is_some());
    let typed = scriptlet.typed_metadata.as_ref().unwrap();
    assert_eq!(typed.name, Some("Quick Todo".to_string()));
    assert_eq!(typed.description, Some("Add a todo item".to_string()));
    assert_eq!(typed.shortcut, Some("cmd t".to_string()));
}

#[test]
fn test_scriptlet_with_codefence_schema() {
    // Test that scriptlets can parse schema blocks
    let markdown = r#"## Input Script

```schema
{
    "input": {
        "title": { "type": "string", "required": true }
    },
    "output": {
        "result": { "type": "string" }
    }
}
```

```ts
const { title } = await input();
output({ result: title.toUpperCase() });
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert!(scriptlet.schema.is_some());

    let schema = scriptlet.schema.as_ref().unwrap();
    assert_eq!(schema.input.len(), 1);
    assert!(schema.input.contains_key("title"));
    assert_eq!(schema.output.len(), 1);
    assert!(schema.output.contains_key("result"));
}

#[test]
fn test_scriptlet_falls_back_to_html_comments() {
    // When no codefence metadata exists, should fall back to HTML comments
    let markdown = r#"## Legacy Script

<!-- shortcut: cmd l -->
<!-- description: A legacy script using HTML comments -->

```bash
echo "Hello from legacy"
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    // HTML comment metadata should still work
    assert_eq!(scriptlet.metadata.shortcut, Some("cmd l".to_string()));
    assert_eq!(
        scriptlet.metadata.description,
        Some("A legacy script using HTML comments".to_string())
    );

    // Typed metadata should be None since no codefence metadata block
    assert!(scriptlet.typed_metadata.is_none());
    assert!(scriptlet.schema.is_none());
}

#[test]
fn test_scriptlet_struct_has_typed_fields() {
    // Verify the Scriptlet struct has the new fields
    let scriptlet = Scriptlet::new(
        "Test".to_string(),
        "ts".to_string(),
        "console.log('test')".to_string(),
    );

    // New fields should exist and default to None
    assert!(scriptlet.typed_metadata.is_none());
    assert!(scriptlet.schema.is_none());
}

#[test]
fn test_mixed_codefence_and_html_prefers_codefence() {
    // When both codefence metadata and HTML comments exist,
    // codefence should take precedence for typed_metadata
    let markdown = r#"## Mixed Script

<!-- shortcut: cmd x -->
<!-- description: HTML description -->

```metadata
{ "name": "Codefence Name", "description": "Codefence description", "shortcut": "cmd y" }
```

```ts
console.log("mixed");
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];

    // Codefence metadata should populate typed_metadata
    assert!(scriptlet.typed_metadata.is_some());
    let typed = scriptlet.typed_metadata.as_ref().unwrap();
    assert_eq!(typed.name, Some("Codefence Name".to_string()));
    assert_eq!(typed.description, Some("Codefence description".to_string()));
    assert_eq!(typed.shortcut, Some("cmd y".to_string()));

    // HTML comments should still populate legacy metadata struct
    // (for backward compatibility)
    assert_eq!(scriptlet.metadata.shortcut, Some("cmd x".to_string()));
    assert_eq!(
        scriptlet.metadata.description,
        Some("HTML description".to_string())
    );
}

#[test]
fn test_codefence_metadata_and_schema_together() {
    // Test scriptlet with both metadata and schema codefence blocks
    let markdown = r#"## Full Featured

```metadata
{ "name": "Full Featured", "description": "Has both metadata and schema" }
```

```schema
{
    "input": {
        "name": { "type": "string", "required": true }
    }
}
```

```ts
const { name } = await input();
console.log(name);
```
"#;
    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];

    // Both should be populated
    assert!(scriptlet.typed_metadata.is_some());
    assert!(scriptlet.schema.is_some());

    let typed = scriptlet.typed_metadata.as_ref().unwrap();
    assert_eq!(typed.name, Some("Full Featured".to_string()));

    let schema = scriptlet.schema.as_ref().unwrap();
    assert!(schema.input.contains_key("name"));
}

// ========================================
// Per-Scriptlet Validation Tests
// ========================================

#[test]
fn test_validation_file_with_middle_malformed_loads_two() {
    // File with 3 scriptlets, middle one is malformed (no code block)
    let markdown = r#"## First Script

```bash
echo "first"
```

## Second Script (Malformed)

This scriptlet has no code block at all!
Just text here.

## Third Script

```bash
echo "third"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/file.md"));

    // Should load 2 scriptlets (first and third)
    assert_eq!(result.scriptlets.len(), 2);
    assert_eq!(result.scriptlets[0].name, "First Script");
    assert_eq!(result.scriptlets[1].name, "Third Script");

    // Should have 1 error for the malformed middle scriptlet
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0]
        .scriptlet_name
        .as_ref()
        .unwrap()
        .contains("Second Script"));
    assert!(result.errors[0]
        .error_message
        .contains("No code block found"));
}

#[test]
fn test_validation_all_valid_scriptlets_loads_all() {
    let markdown = r#"## Script A

```bash
echo "A"
```

## Script B

```python
print("B")
```

## Script C

```ts
console.log("C");
```
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/valid.md"));

    // All 3 should load
    assert_eq!(result.scriptlets.len(), 3);
    assert_eq!(result.scriptlets[0].name, "Script A");
    assert_eq!(result.scriptlets[1].name, "Script B");
    assert_eq!(result.scriptlets[2].name, "Script C");

    // No errors
    assert!(result.errors.is_empty());
}

#[test]
fn test_validation_all_invalid_scriptlets_loads_none() {
    let markdown = r#"## Bad One

No code block here.

## Bad Two

Also no code block.

## Bad Three

Still no code!
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/invalid.md"));

    // No scriptlets should load
    assert!(result.scriptlets.is_empty());

    // Should have 3 errors
    assert_eq!(result.errors.len(), 3);
}

#[test]
fn test_validation_error_includes_line_number() {
    let markdown = r#"## Good Script

```bash
echo "good"
```

## Bad Script On Line 8

No code block here.
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test/file.md"));

    assert_eq!(result.errors.len(), 1);
    let error = &result.errors[0];

    // Line number should be present and greater than 1 (since bad script is not first)
    assert!(error.line_number.is_some());
    assert!(error.line_number.unwrap() > 1);
}

#[test]
fn test_validation_error_includes_file_path() {
    let markdown = r#"## Bad Script

No code block.
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/path/to/my/scripts.md"));

    assert_eq!(result.errors.len(), 1);
    let error = &result.errors[0];

    assert_eq!(error.file_path.to_string_lossy(), "/path/to/my/scripts.md");
}

#[test]
fn test_validation_error_includes_reason() {
    let markdown = r#"## Broken Script

Just text, no code fence.
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test.md"));

    assert_eq!(result.errors.len(), 1);
    let error = &result.errors[0];

    // Error message should explain the problem
    assert!(!error.error_message.is_empty());
    assert!(error.error_message.contains("code block"));
}

#[test]
fn test_validation_empty_h2_name_generates_error() {
    let markdown = r#"## 

```bash
echo "orphan code"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, Some("/test.md"));

    // Empty name should generate an error
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].error_message.contains("Empty"));
}

#[test]
fn test_validation_parses_frontmatter() {
    let markdown = r#"---
name: My Bundle
icon: Star
author: Test Author
---

## Script One

```bash
echo "one"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, None);

    // Frontmatter should be parsed
    assert!(result.frontmatter.is_some());
    let fm = result.frontmatter.unwrap();
    assert_eq!(fm.name, Some("My Bundle".to_string()));
    assert_eq!(fm.icon, Some("Star".to_string()));
    assert_eq!(fm.author, Some("Test Author".to_string()));

    // Script should still load
    assert_eq!(result.scriptlets.len(), 1);
}

#[test]
fn test_validation_backward_compatibility_with_existing_parser() {
    // Same input should produce same scriptlets from both parsers
    let markdown = r#"# My Group

## Script A

```bash
echo "A"
```

## Script B

<!-- shortcut: cmd b -->

```ts
console.log("B");
```
"#;

    let old_result = parse_markdown_as_scriptlets(markdown, Some("/test.md"));
    let new_result = parse_scriptlets_with_validation(markdown, Some("/test.md"));

    // Same number of scriptlets
    assert_eq!(old_result.len(), new_result.scriptlets.len());

    // Same names
    assert_eq!(old_result[0].name, new_result.scriptlets[0].name);
    assert_eq!(old_result[1].name, new_result.scriptlets[1].name);

    // Same groups
    assert_eq!(old_result[0].group, new_result.scriptlets[0].group);
    assert_eq!(old_result[1].group, new_result.scriptlets[1].group);

    // Same metadata
    assert_eq!(
        old_result[1].metadata.shortcut,
        new_result.scriptlets[1].metadata.shortcut
    );
}

#[test]
fn test_validation_error_display() {
    let error = ScriptletValidationError::new(
        "/path/to/file.md",
        Some("My Script".to_string()),
        Some(42),
        "Something went wrong",
    );

    let display = format!("{}", error);

    // Should contain file path
    assert!(display.contains("/path/to/file.md"));
    // Should contain line number
    assert!(display.contains(":42"));
    // Should contain script name
    assert!(display.contains("[My Script]"));
    // Should contain error message
    assert!(display.contains("Something went wrong"));
}

#[test]
fn test_validation_error_display_without_optional_fields() {
    let error = ScriptletValidationError::new("/file.md", None, None, "Error message");

    let display = format!("{}", error);

    // Should still work without optional fields
    assert!(display.contains("/file.md"));
    assert!(display.contains("Error message"));
    // Should NOT contain line number prefix or script name brackets
    assert!(!display.contains("["));
    assert!(!display.contains("]"));
}

#[test]
fn test_scriptlet_parse_result_default() {
    let result = ScriptletParseResult::default();

    assert!(result.scriptlets.is_empty());
    assert!(result.errors.is_empty());
    assert!(result.frontmatter.is_none());
}

#[test]
fn test_validation_mixed_valid_invalid_preserves_order() {
    let markdown = r#"## First (Valid)

```bash
echo "1"
```

## Second (Invalid)

No code.

## Third (Valid)

```bash
echo "3"
```

## Fourth (Invalid)

Also no code.

## Fifth (Valid)

```bash
echo "5"
```
"#;

    let result = parse_scriptlets_with_validation(markdown, None);

    // Valid scriptlets should preserve order
    assert_eq!(result.scriptlets.len(), 3);
    assert_eq!(result.scriptlets[0].name, "First (Valid)");
    assert_eq!(result.scriptlets[1].name, "Third (Valid)");
    assert_eq!(result.scriptlets[2].name, "Fifth (Valid)");

    // Errors should also be in order
    assert_eq!(result.errors.len(), 2);
    assert!(result.errors[0]
        .scriptlet_name
        .as_ref()
        .unwrap()
        .contains("Second"));
    assert!(result.errors[1]
        .scriptlet_name
        .as_ref()
        .unwrap()
        .contains("Fourth"));
}

// ========================================
// Bundle Frontmatter Tests
// ========================================

#[test]
fn test_parse_bundle_frontmatter_basic() {
    let content = r#"---
name: Test Bundle
description: A test bundle
---

## Script
```bash
echo test
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_some());

    let fm = fm.unwrap();
    assert_eq!(fm.name, Some("Test Bundle".to_string()));
    assert_eq!(fm.description, Some("A test bundle".to_string()));
}

#[test]
fn test_parse_bundle_frontmatter_with_icon() {
    let content = r#"---
icon: Star
---

## Script
```bash
echo
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_some());
    assert_eq!(fm.unwrap().icon, Some("Star".to_string()));
}

#[test]
fn test_parse_bundle_frontmatter_no_frontmatter() {
    let content = r#"## Script Without Frontmatter

```bash
echo test
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_none());
}

#[test]
fn test_parse_bundle_frontmatter_unclosed() {
    // Frontmatter without closing ---
    let content = r#"---
name: Unclosed
author: Test

## Script
```bash
echo
```
"#;

    let fm = parse_bundle_frontmatter(content);
    assert!(fm.is_none()); // Should fail to parse
}

// ========================================
// Icon Resolution Tests
// ========================================

#[test]
fn test_tool_type_to_icon_shells() {
    assert_eq!(tool_type_to_icon("bash"), "terminal");
    assert_eq!(tool_type_to_icon("zsh"), "terminal");
    assert_eq!(tool_type_to_icon("sh"), "terminal");
    assert_eq!(tool_type_to_icon("fish"), "terminal");
}

#[test]
fn test_tool_type_to_icon_languages() {
    assert_eq!(tool_type_to_icon("python"), "snake");
    assert_eq!(tool_type_to_icon("ruby"), "gem");
    assert_eq!(tool_type_to_icon("ts"), "file-code");
    assert_eq!(tool_type_to_icon("js"), "file-code");
}

#[test]
fn test_tool_type_to_icon_actions() {
    assert_eq!(tool_type_to_icon("open"), "external-link");
    assert_eq!(tool_type_to_icon("paste"), "clipboard");
    assert_eq!(tool_type_to_icon("type"), "keyboard");
    assert_eq!(tool_type_to_icon("edit"), "edit");
}

#[test]
fn test_tool_type_to_icon_unknown() {
    assert_eq!(tool_type_to_icon("unknown_tool"), "file");
}

#[test]
fn test_resolve_scriptlet_icon_metadata_priority() {
    let mut metadata = ScriptletMetadata::default();
    metadata
        .extra
        .insert("icon".to_string(), "custom-icon".to_string());

    let fm = BundleFrontmatter {
        icon: Some("bundle-icon".to_string()),
        ..Default::default()
    };

    // Metadata icon should take priority
    let icon = resolve_scriptlet_icon(&metadata, Some(&fm), "bash");
    assert_eq!(icon, "custom-icon");
}

#[test]
fn test_resolve_scriptlet_icon_frontmatter_fallback() {
    let metadata = ScriptletMetadata::default(); // No icon in metadata

    let fm = BundleFrontmatter {
        icon: Some("bundle-icon".to_string()),
        ..Default::default()
    };

    // Frontmatter should be used when no metadata icon
    let icon = resolve_scriptlet_icon(&metadata, Some(&fm), "bash");
    assert_eq!(icon, "bundle-icon");
}

#[test]
fn test_resolve_scriptlet_icon_tool_fallback() {
    let metadata = ScriptletMetadata::default();

    // No frontmatter icon either
    let icon = resolve_scriptlet_icon(&metadata, None, "python");
    assert_eq!(icon, "snake");
}

// ========================================
// H3 Scriptlet Action Tests
// ========================================

#[test]
fn test_scriptlet_action_struct() {
    let action = ScriptletAction {
        name: "Copy to Clipboard".to_string(),
        command: "copy-to-clipboard".to_string(),
        tool: "bash".to_string(),
        code: "echo '{{text}}' | pbcopy".to_string(),
        inputs: vec!["text".to_string()],
        shortcut: Some("cmd+c".to_string()),
        description: Some("Copy text to clipboard".to_string()),
    };

    assert_eq!(action.name, "Copy to Clipboard");
    assert_eq!(action.action_id(), "scriptlet_action:copy-to-clipboard");
}

#[test]
fn test_parse_h3_action_basic() {
    let content = r#"
```bash
echo "action code"
```
"#;

    let action = parse_h3_action("My Action", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert_eq!(action.name, "My Action");
    assert_eq!(action.command, "my-action");
    assert_eq!(action.tool, "bash");
    assert_eq!(action.code, "echo \"action code\"");
}

#[test]
fn test_parse_h3_action_with_metadata() {
    let content = r#"
<!-- shortcut: cmd+c -->
<!-- description: Copies to clipboard -->

```bash
echo "copy"
```
"#;

    let action = parse_h3_action("Copy Action", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert_eq!(action.shortcut, Some("cmd+c".to_string()));
    assert_eq!(action.description, Some("Copies to clipboard".to_string()));
}

#[test]
fn test_parse_h3_action_extracts_inputs() {
    let content = r#"
```bash
echo "Hello {{name}}, your age is {{age}}"
```
"#;

    let action = parse_h3_action("Greeting", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert!(action.inputs.contains(&"name".to_string()));
    assert!(action.inputs.contains(&"age".to_string()));
}

#[test]
fn test_parse_h3_action_no_code_block() {
    let content = "Just text, no code fence.";
    let action = parse_h3_action("Bad Action", content);
    assert!(action.is_none());
}

#[test]
fn test_parse_h3_action_open_tool() {
    let content = r#"
```open
https://github.com/{{repo}}
```
"#;

    let action = parse_h3_action("Open GitHub", content);
    assert!(action.is_some());

    let action = action.unwrap();
    assert_eq!(action.tool, "open");
    assert!(action.code.contains("https://github.com"));
}

#[test]
fn test_parse_h3_action_invalid_tool() {
    let content = r#"
```invalidtool
some code
```
"#;

    // Invalid tool should return None
    let action = parse_h3_action("Bad Tool Action", content);
    assert!(action.is_none());
}

#[test]
fn test_extract_h3_actions_basic() {
    let section = r#"## My Scriptlet

```bash
echo "main code"
```

### Copy to Clipboard

```bash
echo "copy"
```

### Open Browser

```open
https://example.com
```
"#;

    let actions = extract_h3_actions(section);
    assert_eq!(actions.len(), 2);

    assert_eq!(actions[0].name, "Copy to Clipboard");
    assert_eq!(actions[0].tool, "bash");

    assert_eq!(actions[1].name, "Open Browser");
    assert_eq!(actions[1].tool, "open");
}

#[test]
fn test_extract_h3_actions_with_metadata() {
    let section = r#"## Scriptlet

```bash
main code
```

### Action One
<!-- shortcut: cmd+1 -->
```bash
action one code
```

### Action Two
<!-- shortcut: cmd+2 -->
<!-- description: Second action -->
```bash
action two code
```
"#;

    let actions = extract_h3_actions(section);
    assert_eq!(actions.len(), 2);

    assert_eq!(actions[0].shortcut, Some("cmd+1".to_string()));
    assert_eq!(actions[1].shortcut, Some("cmd+2".to_string()));
    assert_eq!(actions[1].description, Some("Second action".to_string()));
}

#[test]
fn test_extract_h3_actions_none_before_main_code() {
    // H3s before the main code block should not be captured
    let section = r#"## Scriptlet

### This Should Be Ignored
```bash
ignored
```

```bash
main code - this is the FIRST valid tool codefence
```

### This Should Be Captured
```bash
captured
```
"#;

    let actions = extract_h3_actions(section);
    // Only the H3 AFTER the main code should be captured
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "This Should Be Captured");
}

#[test]
fn test_extract_h3_actions_empty_section() {
    let section = r#"## Scriptlet

```bash
main code
```
"#;

    let actions = extract_h3_actions(section);
    assert!(actions.is_empty());
}

#[test]
fn test_extract_h3_actions_h3_without_code() {
    let section = r#"## Scriptlet

```bash
main
```

### Bad H3 Without Code

Just text, no code fence.

### Good H3

```bash
good code
```
"#;

    let actions = extract_h3_actions(section);
    // Only the good H3 should be captured
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "Good H3");
}

#[test]
fn test_parse_markdown_includes_h3_actions() {
    let markdown = r#"## GitHub Tools

<!-- shortcut: cmd+g -->

```open
https://github.com/{{repo}}
```

### Copy SSH URL
<!-- shortcut: cmd+shift+c -->
```bash
echo "git@github.com:{{repo}}.git" | pbcopy
```

### View README
```open
https://github.com/{{repo}}/blob/main/README.md
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 1);

    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "GitHub Tools");
    assert_eq!(scriptlet.actions.len(), 2);

    // First action
    assert_eq!(scriptlet.actions[0].name, "Copy SSH URL");
    assert_eq!(scriptlet.actions[0].tool, "bash");
    assert_eq!(
        scriptlet.actions[0].shortcut,
        Some("cmd+shift+c".to_string())
    );

    // Second action
    assert_eq!(scriptlet.actions[1].name, "View README");
    assert_eq!(scriptlet.actions[1].tool, "open");
    assert!(scriptlet.actions[1].shortcut.is_none());
}

#[test]
fn test_parse_markdown_multiple_scriptlets_with_actions() {
    let markdown = r#"# Tools

## URL Opener

```open
https://{{url}}
```

### Open in Safari
```bash
open -a Safari "https://{{url}}"
```

## File Manager

```bash
ls -la {{path}}
```

### Open in Finder
```bash
open {{path}}
```

### Copy Path
```bash
echo "{{path}}" | pbcopy
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    assert_eq!(scriptlets.len(), 2);

    // First scriptlet
    assert_eq!(scriptlets[0].name, "URL Opener");
    assert_eq!(scriptlets[0].actions.len(), 1);
    assert_eq!(scriptlets[0].actions[0].name, "Open in Safari");

    // Second scriptlet
    assert_eq!(scriptlets[1].name, "File Manager");
    assert_eq!(scriptlets[1].actions.len(), 2);
    assert_eq!(scriptlets[1].actions[0].name, "Open in Finder");
    assert_eq!(scriptlets[1].actions[1].name, "Copy Path");
}

#[test]
fn test_scriptlet_action_serialization() {
    let action = ScriptletAction {
        name: "Test Action".to_string(),
        command: "test-action".to_string(),
        tool: "bash".to_string(),
        code: "echo test".to_string(),
        inputs: vec!["var".to_string()],
        shortcut: Some("cmd+t".to_string()),
        description: Some("A test action".to_string()),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: ScriptletAction = serde_json::from_str(&json).unwrap();

    assert_eq!(action.name, deserialized.name);
    assert_eq!(action.tool, deserialized.tool);
    assert_eq!(action.shortcut, deserialized.shortcut);
}

#[test]
fn test_scriptlet_with_actions_serialization() {
    let markdown = r#"## Test Scriptlet

```bash
echo main
```

### Action One
```bash
echo one
```
"#;

    let scriptlets = parse_markdown_as_scriptlets(markdown, None);
    let scriptlet = &scriptlets[0];

    let json = serde_json::to_string(scriptlet).unwrap();
    let deserialized: Scriptlet = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.actions.len(), 1);
    assert_eq!(deserialized.actions[0].name, "Action One");
}

#[test]
fn test_validation_parser_includes_actions() {
    let markdown = r#"## Test

```bash
main
```

### Sub Action
```bash
sub
```
"#;

    let result = parse_scriptlets_with_validation(markdown, None);
    assert_eq!(result.scriptlets.len(), 1);
    assert_eq!(result.scriptlets[0].actions.len(), 1);
}

#[test]
fn test_scriptlet_new_has_empty_actions() {
    let scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());

    assert!(scriptlet.actions.is_empty());
}

// ========================================
// Shared Actions (.actions.md) Tests
// ========================================

#[test]
fn test_parse_actions_file_basic() {
    let content = r#"# URL Actions

### Copy URL
```bash
echo "{{url}}" | pbcopy
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "Copy URL");
    assert_eq!(actions[0].command, "copy-url");
    assert_eq!(actions[0].tool, "bash");
    assert!(actions[0].code.contains("pbcopy"));
}

#[test]
fn test_parse_actions_file_multiple_actions() {
    let content = r#"# URL Actions

### Copy URL
```bash
echo "{{url}}" | pbcopy
```

### Open URL
```open
{{url}}
```

# Text Actions

### Make Uppercase
```bash
echo "{{text}}" | tr '[:lower:]' '[:upper:]'
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].name, "Copy URL");
    assert_eq!(actions[1].name, "Open URL");
    assert_eq!(actions[2].name, "Make Uppercase");
}

#[test]
fn test_parse_actions_file_with_metadata() {
    let content = r#"### Copy URL
<!-- shortcut: cmd+c -->
<!-- description: Copy the URL to clipboard -->
```bash
echo "{{url}}" | pbcopy
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].shortcut, Some("cmd+c".to_string()));
    assert_eq!(
        actions[0].description,
        Some("Copy the URL to clipboard".to_string())
    );
}

#[test]
fn test_parse_actions_file_ignores_h2() {
    let content = r#"## This is an H2 (should be ignored)
```bash
echo "ignored"
```

### This is an H3 (should be parsed)
```bash
echo "action"
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "This is an H3 (should be parsed)");
}

#[test]
fn test_parse_actions_file_extracts_inputs() {
    let content = r#"### Send Email
```bash
echo "To: {{email}}, Subject: {{subject}}"
```
"#;

    let actions = parse_actions_file(content);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].inputs.len(), 2);
    assert!(actions[0].inputs.contains(&"email".to_string()));
    assert!(actions[0].inputs.contains(&"subject".to_string()));
}

#[test]
fn test_parse_actions_file_empty() {
    let content = "# Just a header with no actions\n\nSome text here.";
    let actions = parse_actions_file(content);
    assert!(actions.is_empty());
}

#[test]
fn test_parse_actions_file_skips_invalid_tool() {
    let content = r#"### Invalid Tool Action
```invalidtool
some code
```
"#;

    let actions = parse_actions_file(content);
    assert!(actions.is_empty());
}

#[test]
fn test_get_actions_file_path() {
    use std::path::Path;

    let md_path = Path::new("/path/to/main.md");
    let actions_path = get_actions_file_path(md_path);
    assert_eq!(actions_path.to_string_lossy(), "/path/to/main.actions.md");

    let md_path2 = Path::new("/extensions/foo.bar.md");
    let actions_path2 = get_actions_file_path(md_path2);
    assert_eq!(
        actions_path2.to_string_lossy(),
        "/extensions/foo.bar.actions.md"
    );
}

#[test]
fn test_merge_shared_actions_basic() {
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo test".to_string(),
    );

    let shared_actions = vec![
        ScriptletAction {
            name: "Copy".to_string(),
            command: "copy".to_string(),
            tool: "bash".to_string(),
            code: "pbcopy".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Open".to_string(),
            command: "open".to_string(),
            tool: "open".to_string(),
            code: "{{url}}".to_string(),
            inputs: vec!["url".to_string()],
            shortcut: None,
            description: None,
        },
    ];

    merge_shared_actions(&mut scriptlet, &shared_actions);
    assert_eq!(scriptlet.actions.len(), 2);
}

#[test]
fn test_merge_shared_actions_inline_takes_precedence() {
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo test".to_string(),
    );

    // Add an inline action
    scriptlet.actions.push(ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "inline copy code".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+c".to_string()),
        description: None,
    });

    // Shared action with same command
    let shared_actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "shared copy code".to_string(),
        inputs: vec![],
        shortcut: None,
        description: Some("Shared description".to_string()),
    }];

    merge_shared_actions(&mut scriptlet, &shared_actions);

    // Should still have only 1 action (inline takes precedence)
    assert_eq!(scriptlet.actions.len(), 1);
    // The code should be from the inline action
    assert_eq!(scriptlet.actions[0].code, "inline copy code");
    // Shortcut should be from inline action
    assert_eq!(scriptlet.actions[0].shortcut, Some("cmd+c".to_string()));
}

#[test]
fn test_merge_shared_actions_mixed() {
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo test".to_string(),
    );

    // Add an inline action
    scriptlet.actions.push(ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "inline".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    });

    // Shared actions: one conflicts, two are new
    let shared_actions = vec![
        ScriptletAction {
            name: "Copy".to_string(), // conflicts
            command: "copy".to_string(),
            tool: "bash".to_string(),
            code: "shared".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Open".to_string(), // new
            command: "open".to_string(),
            tool: "open".to_string(),
            code: "open".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Delete".to_string(), // new
            command: "delete".to_string(),
            tool: "bash".to_string(),
            code: "rm".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];

    merge_shared_actions(&mut scriptlet, &shared_actions);

    // Should have 3 actions: 1 inline + 2 new from shared
    assert_eq!(scriptlet.actions.len(), 3);

    // First is inline copy (unchanged)
    assert_eq!(scriptlet.actions[0].command, "copy");
    assert_eq!(scriptlet.actions[0].code, "inline");

    // Then the two new shared actions
    assert_eq!(scriptlet.actions[1].command, "open");
    assert_eq!(scriptlet.actions[2].command, "delete");
}

// ========================================
// Integration Tests (filesystem-based)
// ========================================

#[test]
fn test_shared_actions_loaded_from_companion_file() {
    use std::fs;
    use tempfile::TempDir;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create main.md with a scriptlet
    let main_md_path = temp_path.join("main.md");
    let main_md_content = r#"## Open Google

```open
https://www.google.com
```
"#;
    fs::write(&main_md_path, main_md_content).unwrap();

    // Create main.actions.md with shared actions
    let actions_md_path = temp_path.join("main.actions.md");
    let actions_md_content = r#"### Copy URL
<!-- shortcut: cmd+c -->
```bash
echo "{{content}}" | pbcopy
```

### Open in Safari
```bash
open -a Safari "{{content}}"
```
"#;
    fs::write(&actions_md_path, actions_md_content).unwrap();

    // Parse the main.md file
    let content = fs::read_to_string(&main_md_path).unwrap();
    let scriptlets = parse_markdown_as_scriptlets(&content, Some(main_md_path.to_str().unwrap()));

    // Should have 1 scriptlet
    assert_eq!(scriptlets.len(), 1);

    // The scriptlet should have the shared actions merged
    let scriptlet = &scriptlets[0];
    assert_eq!(scriptlet.name, "Open Google");
    assert_eq!(scriptlet.actions.len(), 2);

    // Check the actions are correct
    assert_eq!(scriptlet.actions[0].name, "Copy URL");
    assert_eq!(scriptlet.actions[0].shortcut, Some("cmd+c".to_string()));
    assert!(scriptlet.actions[0].code.contains("pbcopy"));

    assert_eq!(scriptlet.actions[1].name, "Open in Safari");
    assert!(scriptlet.actions[1].code.contains("Safari"));
}

#[test]
fn test_shared_actions_not_loaded_for_actions_file() {
    use std::fs;
    use tempfile::TempDir;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create main.actions.md (the actions file itself)
    let actions_md_path = temp_path.join("main.actions.md");
    let actions_md_content = r#"### Copy URL
```bash
echo "test" | pbcopy
```
"#;
    fs::write(&actions_md_path, actions_md_content).unwrap();

    // Parsing the .actions.md file should NOT try to load main.actions.actions.md
    // (it would be a recursive loop)
    let content = fs::read_to_string(&actions_md_path).unwrap();
    let scriptlets =
        parse_markdown_as_scriptlets(&content, Some(actions_md_path.to_str().unwrap()));

    // Should have 0 scriptlets (actions files only have H3, not H2)
    assert_eq!(scriptlets.len(), 0);
}

#[test]
fn test_inline_actions_take_precedence_over_shared() {
    use std::fs;
    use tempfile::TempDir;

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create main.md with a scriptlet that has an inline action
    let main_md_path = temp_path.join("main.md");
    let main_md_content = r#"## My Scriptlet

```bash
echo "hello"
```

### Copy
<!-- shortcut: cmd+shift+c -->
```bash
inline copy code
```
"#;
    fs::write(&main_md_path, main_md_content).unwrap();

    // Create main.actions.md with a shared action of the same name
    let actions_md_path = temp_path.join("main.actions.md");
    let actions_md_content = r#"### Copy
<!-- shortcut: cmd+c -->
```bash
shared copy code
```

### Delete
```bash
rm something
```
"#;
    fs::write(&actions_md_path, actions_md_content).unwrap();

    // Parse the main.md file
    let content = fs::read_to_string(&main_md_path).unwrap();
    let scriptlets = parse_markdown_as_scriptlets(&content, Some(main_md_path.to_str().unwrap()));

    assert_eq!(scriptlets.len(), 1);
    let scriptlet = &scriptlets[0];

    // Should have 2 actions: inline Copy + shared Delete
    // The shared Copy should be skipped because inline takes precedence
    assert_eq!(scriptlet.actions.len(), 2);

    // First action is inline Copy
    assert_eq!(scriptlet.actions[0].name, "Copy");
    assert_eq!(scriptlet.actions[0].code, "inline copy code");
    assert_eq!(
        scriptlet.actions[0].shortcut,
        Some("cmd+shift+c".to_string())
    ); // inline shortcut

    // Second action is shared Delete
    assert_eq!(scriptlet.actions[1].name, "Delete");
}

    use super::*;
    use crate::schema_parser::FieldType;
    // ========================================
    // Core Test Cases (from requirements)
    // ========================================

    #[test]
    fn test_parse_metadata_codefence() {
        let content = r#"
# Quick Todo

```metadata
{ "name": "Quick Todo", "description": "Add a todo item" }
```

```ts
const item = await arg("Todo item");
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.metadata.is_some());

        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.name, Some("Quick Todo".to_string()));
        assert_eq!(metadata.description, Some("Add a todo item".to_string()));
    }
    #[test]
    fn test_parse_schema_codefence() {
        let content = r#"
```schema
{
    "input": {
        "item": { "type": "string", "required": true }
    }
}
```

```ts
const { item } = await input();
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.schema.is_some());

        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 1);
        assert!(schema.input.contains_key("item"));

        let item_field = schema.input.get("item").unwrap();
        assert_eq!(item_field.field_type, FieldType::String);
        assert!(item_field.required);
    }
    #[test]
    fn test_parse_both_metadata_and_schema() {
        let content = r#"
```metadata
{ "name": "Quick Todo", "description": "Add a todo item", "icon": "CheckSquare" }
```

```schema
{
    "input": {
        "item": { "type": "string", "required": true, "description": "The todo item text" }
    },
    "output": {
        "id": { "type": "string" }
    }
}
```

```ts
const { item } = await input();
const id = await addTodo(item);
output({ id });
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

        // Check metadata
        assert!(result.metadata.is_some());
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.name, Some("Quick Todo".to_string()));
        assert_eq!(metadata.description, Some("Add a todo item".to_string()));
        assert_eq!(metadata.icon, Some("CheckSquare".to_string()));

        // Check schema
        assert!(result.schema.is_some());
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 1);
        assert_eq!(schema.output.len(), 1);

        let item_field = schema.input.get("item").unwrap();
        assert_eq!(
            item_field.description,
            Some("The todo item text".to_string())
        );

        assert!(schema.output.contains_key("id"));
    }
    #[test]
    fn test_no_codefence_returns_none() {
        let content = r#"
# Just a Regular Markdown

Some text here with no code fences at all.

- List item 1
- List item 2
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_none());
        assert!(result.schema.is_none());
        assert!(result.code.is_none());
        assert!(result.errors.is_empty());
    }
    #[test]
    fn test_malformed_content_returns_error() {
        // Content that's neither valid JSON nor valid simple key: value format
        let content = r#"
```metadata
this is just random text with no valid format
no key value pairs here either
```

```ts
console.log("test");
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("not valid JSON or simple key: value format"));
    }
    #[test]
    fn test_simple_format_keyword() {
        // Simple key: value format (not JSON)
        let content = r#"
```metadata
keyword: !testing
```

```paste
success!
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        assert!(result.errors.is_empty());
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.keyword, Some("!testing".to_string()));
    }
    #[test]
    fn test_simple_format_multiple_fields() {
        let content = r#"
```metadata
name: My Script
keyword: :sig
description: A helpful signature expander
shortcut: cmd shift s
```

```paste
Best regards,
John
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        assert!(result.errors.is_empty());
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.name, Some("My Script".to_string()));
        assert_eq!(metadata.keyword, Some(":sig".to_string()));
        assert_eq!(
            metadata.description,
            Some("A helpful signature expander".to_string())
        );
        assert_eq!(metadata.shortcut, Some("cmd shift s".to_string()));
    }
    #[test]
    fn test_simple_format_expand_alias() {
        // "expand" and "snippet" should also work as aliases for "keyword"
        let content1 = "```metadata\nexpand: !test\n```";
        let result1 = parse_codefence_metadata(content1);
        assert_eq!(result1.metadata.unwrap().keyword, Some("!test".to_string()));

        let content2 = "```metadata\nsnippet: :sig\n```";
        let result2 = parse_codefence_metadata(content2);
        assert_eq!(result2.metadata.unwrap().keyword, Some(":sig".to_string()));
    }
    #[test]
    fn test_simple_format_with_comments() {
        let content = r#"
```metadata
// This is a comment
keyword: !testing
// Another comment
name: Test Script
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.keyword, Some("!testing".to_string()));
        assert_eq!(metadata.name, Some("Test Script".to_string()));
    }
    #[test]
    fn test_code_block_extracted_correctly() {
        let content = r#"
```metadata
{ "name": "Test Script" }
```

```ts
const result = await arg("Pick one");
console.log(result);
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.code.is_some());
        let code = result.code.unwrap();
        assert_eq!(code.language, "ts");
        assert!(code.content.contains("const result = await arg"));
        assert!(code.content.contains("console.log(result)"));
    }
    // ========================================
    // Additional Test Cases
    // ========================================

    #[test]
    fn test_multiple_code_blocks_first_wins() {
        let content = r#"
```ts
first code block
```

```ts
second code block
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.code.is_some());
        let code = result.code.unwrap();
        assert_eq!(code.content, "first code block");
    }
    #[test]
    fn test_tilde_fences_supported() {
        let content = r#"
~~~metadata
{ "name": "Tilde Test" }
~~~

~~~ts
console.log("tilde fences");
~~~
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        assert_eq!(
            result.metadata.unwrap().name,
            Some("Tilde Test".to_string())
        );

        assert!(result.code.is_some());
        assert_eq!(result.code.unwrap().language, "ts");
    }
    #[test]
    fn test_mixed_fence_types() {
        let content = r#"
```metadata
{ "name": "Mixed Fences" }
```

~~~ts
const x = 1;
~~~
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        assert!(result.code.is_some());
        assert_eq!(result.code.unwrap().language, "ts");
    }
    #[test]
    fn test_metadata_with_all_fields() {
        let content = r#"
```metadata
{
    "name": "Full Script",
    "description": "A complete script",
    "author": "Test Author",
    "enter": "Execute",
    "alias": "fs",
    "icon": "Star",
    "shortcut": "cmd shift f",
    "tags": ["productivity", "utility"],
    "hidden": true,
    "placeholder": "Type something...",
    "cron": "0 9 * * *",
    "watch": ["*.ts", "*.js"],
    "background": true,
    "system": false
}
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.metadata.is_some());

        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Full Script".to_string()));
        assert_eq!(meta.description, Some("A complete script".to_string()));
        assert_eq!(meta.author, Some("Test Author".to_string()));
        assert_eq!(meta.enter, Some("Execute".to_string()));
        assert_eq!(meta.alias, Some("fs".to_string()));
        assert_eq!(meta.icon, Some("Star".to_string()));
        assert_eq!(meta.shortcut, Some("cmd shift f".to_string()));
        assert_eq!(meta.tags, vec!["productivity", "utility"]);
        assert!(meta.hidden);
        assert_eq!(meta.placeholder, Some("Type something...".to_string()));
        assert_eq!(meta.cron, Some("0 9 * * *".to_string()));
        assert_eq!(meta.watch, vec!["*.ts", "*.js"]);
        assert!(meta.background);
        assert!(!meta.system);
    }
    #[test]
    fn test_schema_with_all_field_types() {
        let content = r#"
```schema
{
    "input": {
        "name": { "type": "string" },
        "count": { "type": "number" },
        "enabled": { "type": "boolean" },
        "items": { "type": "array", "items": "string" },
        "config": { "type": "object" },
        "anything": { "type": "any" }
    }
}
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert!(result.schema.is_some());

        let schema = result.schema.unwrap();
        assert_eq!(
            schema.input.get("name").unwrap().field_type,
            FieldType::String
        );
        assert_eq!(
            schema.input.get("count").unwrap().field_type,
            FieldType::Number
        );
        assert_eq!(
            schema.input.get("enabled").unwrap().field_type,
            FieldType::Boolean
        );
        assert_eq!(
            schema.input.get("items").unwrap().field_type,
            FieldType::Array
        );
        assert_eq!(
            schema.input.get("config").unwrap().field_type,
            FieldType::Object
        );
        assert_eq!(
            schema.input.get("anything").unwrap().field_type,
            FieldType::Any
        );
    }
    #[test]
    fn test_malformed_schema_json_returns_error() {
        let content = r#"
```schema
{ "input": { not valid json } }
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.schema.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("Failed to parse schema JSON"));
    }
    #[test]
    fn test_different_code_languages() {
        let languages = vec!["bash", "python", "ruby", "js", "kit", "template"];

        for lang in languages {
            let content = format!(
                r#"
```{}
code content
```
"#,
                lang
            );
            let result = parse_codefence_metadata(&content);

            assert!(result.code.is_some(), "Failed for language: {}", lang);
            assert_eq!(result.code.unwrap().language, lang);
        }
    }

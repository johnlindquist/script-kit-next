    #[test]
    fn test_empty_codefence_blocks() {
        let content = r#"
```metadata
```

```ts
```
"#;
        let result = parse_codefence_metadata(content);

        // Empty metadata should fail to parse as JSON
        assert!(!result.errors.is_empty() || result.metadata.is_none());

        // Empty code block should still be captured
        assert!(result.code.is_some());
        assert_eq!(result.code.unwrap().content, "");
    }
    #[test]
    fn test_whitespace_handling() {
        let content = r#"
```metadata
  { "name": "Whitespace Test" }  
```

```ts
  const x = 1;  
  const y = 2;  
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        assert_eq!(
            result.metadata.unwrap().name,
            Some("Whitespace Test".to_string())
        );

        assert!(result.code.is_some());
        let code = result.code.unwrap();
        // Content should preserve internal whitespace but trim outer
        assert!(code.content.contains("const x = 1"));
        assert!(code.content.contains("const y = 2"));
    }
    #[test]
    fn test_nested_code_in_markdown() {
        // Simulate a scriptlet that contains a markdown code example
        let content = r#"
```metadata
{ "name": "Code Example Generator" }
```

```ts
const example = `
\`\`\`js
console.log("hello");
\`\`\`
`;
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        assert!(result.code.is_some());
        // The code block should contain the nested fence example
        assert!(result.code.unwrap().content.contains("console.log"));
    }
    #[test]
    fn test_order_independence() {
        // Schema before metadata should still work
        let content = r#"
```schema
{ "input": { "x": { "type": "string" } } }
```

```metadata
{ "name": "Order Test" }
```

```ts
code here
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_some());
        assert!(result.schema.is_some());
        assert!(result.code.is_some());

        assert_eq!(
            result.metadata.unwrap().name,
            Some("Order Test".to_string())
        );
        assert!(result.schema.unwrap().input.contains_key("x"));
    }
    #[test]
    fn test_code_block_without_metadata_or_schema() {
        let content = r#"
## Simple Script

```bash
echo "Hello World"
```
"#;
        let result = parse_codefence_metadata(content);

        assert!(result.metadata.is_none());
        assert!(result.schema.is_none());
        assert!(result.code.is_some());

        let code = result.code.unwrap();
        assert_eq!(code.language, "bash");
        assert_eq!(code.content, "echo \"Hello World\"");
    }
    #[test]
    fn test_multiple_errors_collected() {
        let content = r#"
```metadata
{ invalid json 1 }
```

```schema
{ invalid json 2 }
```
"#;
        let result = parse_codefence_metadata(content);

        assert_eq!(result.errors.len(), 2);
        assert!(result.errors[0].contains("metadata"));
        assert!(result.errors[1].contains("schema"));
    }

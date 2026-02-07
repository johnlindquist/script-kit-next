use super::*;

#[test]
fn test_extract_code_block_skips_metadata() {
    // This is the user's actual format - metadata block followed by paste block
    let text = r#"## Greet

```metadata
keyword: !testing
```

```paste
success!
```
"#;
    let result = extract_code_block(text);
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "paste");
    assert_eq!(code, "success!");
}

#[test]
fn test_extract_code_block_skips_schema() {
    let text = r#"## Test

```schema
{"input": {"name": "string"}}
```

```ts
console.log("hello");
```
"#;
    let result = extract_code_block(text);
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "ts");
    assert_eq!(code, "console.log(\"hello\");");
}

#[test]
fn test_extract_code_block_no_metadata() {
    // When there's no metadata block, should still work
    let text = r#"## Test

```paste
hello world
```
"#;
    let result = extract_code_block(text);
    assert!(result.is_some());
    let (tool, code) = result.unwrap();
    assert_eq!(tool, "paste");
    assert_eq!(code, "hello world");
}

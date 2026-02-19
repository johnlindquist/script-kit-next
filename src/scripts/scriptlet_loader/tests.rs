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

#[test]
fn test_read_scriptlets_keeps_first_scriptlet_when_file_starts_with_heading() {
    use crate::setup::SK_PATH_ENV;
    use std::fs;
    use tempfile::TempDir;

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    let temp_dir = TempDir::new().expect("create temp dir");
    let extensions_dir = temp_dir.path().join("kit").join("main").join("extensions");
    fs::create_dir_all(&extensions_dir).expect("create extensions dir");

    let scriptlet_file = extensions_dir.join("scriptlets.md");
    fs::write(
        &scriptlet_file,
        r#"## First Scriptlet
```paste
one
```

## Second Scriptlet
```paste
two
```
"#,
    )
    .expect("write scriptlet file");

    let _guard = EnvVarGuard::set(SK_PATH_ENV, &temp_dir.path().to_string_lossy());
    let scriptlets = super::loading::read_scriptlets();

    let names: Vec<String> = scriptlets.iter().map(|s| s.name.clone()).collect();
    assert_eq!(names, vec!["First Scriptlet", "Second Scriptlet"]);
}

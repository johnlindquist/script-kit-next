//! Integration tests for the AI context part model and deterministic resolvers.
//!
//! Validates that `AiContextPart` variants serialize correctly and that
//! resolution produces deterministic, well-structured prompt blocks.

use script_kit_gpui::ai::message_parts::{
    resolve_context_part_to_prompt_block, resolve_context_parts_to_prompt_prefix, AiContextPart,
};
use std::sync::Arc;

// ---------- Serde contract tests ----------

#[test]
fn context_part_resource_uri_serde_tagged() {
    let part = AiContextPart::ResourceUri {
        uri: "kit://context?profile=minimal".to_string(),
        label: "Current Context".to_string(),
    };

    let json = serde_json::to_value(&part).expect("serialize");
    assert_eq!(json["kind"], "resourceUri");
    assert_eq!(json["uri"], "kit://context?profile=minimal");
    assert_eq!(json["label"], "Current Context");

    let round_trip: AiContextPart = serde_json::from_value(json).expect("deserialize");
    assert_eq!(part, round_trip);
}

#[test]
fn context_part_file_path_serde_tagged() {
    let part = AiContextPart::FilePath {
        path: "/tmp/example.rs".to_string(),
        label: "example.rs".to_string(),
    };

    let json = serde_json::to_value(&part).expect("serialize");
    assert_eq!(json["kind"], "filePath");
    assert_eq!(json["path"], "/tmp/example.rs");
    assert_eq!(json["label"], "example.rs");

    let round_trip: AiContextPart = serde_json::from_value(json).expect("deserialize");
    assert_eq!(part, round_trip);
}

// ---------- ResourceUri resolution ----------

#[test]
fn context_part_resolution_resource_uri_returns_deterministic_block() {
    let scripts: Vec<Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let part = AiContextPart::ResourceUri {
        uri: "kit://context?profile=minimal".to_string(),
        label: "Current Context".to_string(),
    };

    let block =
        resolve_context_part_to_prompt_block(&part, &scripts, &scriptlets).expect("should resolve");

    // Must contain source URI and MIME type in the opening tag
    assert!(
        block.contains("source=\"kit://context?profile=minimal\""),
        "block should contain source URI"
    );
    assert!(
        block.contains("mimeType=\"application/json\""),
        "block should contain MIME type"
    );

    // Must be wrapped in <context> tags
    assert!(block.starts_with("<context "), "block should start with <context");
    assert!(block.ends_with("</context>"), "block should end with </context>");

    // Content should be valid JSON
    let inner = block
        .split('\n')
        .skip(1) // skip opening tag line
        .take_while(|line| !line.contains("</context>"))
        .collect::<Vec<_>>()
        .join("\n");
    let value: serde_json::Value =
        serde_json::from_str(&inner).expect("inner content should be valid JSON");
    assert!(
        value.get("schemaVersion").is_some(),
        "context JSON should have schemaVersion"
    );
}

// ---------- FilePath resolution ----------

#[test]
fn context_part_resolution_readable_file_path() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("sample.txt");
    std::fs::write(&file_path, "fn main() {}").expect("write");

    let part = AiContextPart::FilePath {
        path: file_path.to_string_lossy().to_string(),
        label: "sample.txt".to_string(),
    };

    let block = resolve_context_part_to_prompt_block(&part, &[], &[]).expect("should resolve");

    assert!(block.contains("<attachment path=\""));
    assert!(block.contains("fn main() {}"));
    assert!(block.contains("</attachment>"));
    assert!(
        !block.contains("unreadable"),
        "readable file should not have unreadable marker"
    );
}

#[test]
fn context_part_resolution_unreadable_file_path_returns_metadata_fallback() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("secret.bin");
    std::fs::write(&file_path, vec![0xFFu8; 128]).expect("write");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o000))
            .expect("set permissions");
    }

    let part = AiContextPart::FilePath {
        path: file_path.to_string_lossy().to_string(),
        label: "secret.bin".to_string(),
    };

    #[cfg(unix)]
    {
        let block = resolve_context_part_to_prompt_block(&part, &[], &[]).expect("should not panic");
        assert!(
            block.contains("unreadable=\"true\""),
            "unreadable file should have unreadable marker"
        );
        assert!(
            block.contains("bytes=\"128\""),
            "unreadable file should report size"
        );
        // Must be self-closing tag (no body)
        assert!(
            block.contains("/>"),
            "unreadable attachment should be self-closing"
        );
    }

    // Restore permissions for cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644));
    }
}

#[test]
fn context_part_resolution_nonexistent_file_returns_error_not_panic() {
    let part = AiContextPart::FilePath {
        path: "/nonexistent/absolutely/does/not/exist.txt".to_string(),
        label: "ghost.txt".to_string(),
    };

    let result = resolve_context_part_to_prompt_block(&part, &[], &[]);
    assert!(result.is_err(), "nonexistent file should return Err");
}

// ---------- Multi-part resolution ----------

#[test]
fn context_part_resolution_multiple_parts_concatenated() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_a = dir.path().join("a.rs");
    let file_b = dir.path().join("b.rs");
    std::fs::write(&file_a, "struct A;").expect("write a");
    std::fs::write(&file_b, "struct B;").expect("write b");

    let parts = vec![
        AiContextPart::FilePath {
            path: file_a.to_string_lossy().to_string(),
            label: "a.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: file_b.to_string_lossy().to_string(),
            label: "b.rs".to_string(),
        },
    ];

    let prefix = resolve_context_parts_to_prompt_prefix(&parts, &[], &[]).expect("should resolve");

    assert!(prefix.contains("struct A;"));
    assert!(prefix.contains("struct B;"));
    assert!(
        prefix.contains("</attachment>\n\n<attachment"),
        "blocks should be separated by double newline"
    );
}

#[test]
fn context_part_resolution_empty_parts_returns_empty() {
    let prefix = resolve_context_parts_to_prompt_prefix(&[], &[], &[]).expect("should resolve");
    assert!(prefix.is_empty());
}

// ---------- Mixed resource + file resolution ----------

#[test]
fn context_part_resolution_mixed_resource_and_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("code.rs");
    std::fs::write(&file_path, "let x = 42;").expect("write");

    let parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Context".to_string(),
        },
        AiContextPart::FilePath {
            path: file_path.to_string_lossy().to_string(),
            label: "code.rs".to_string(),
        },
    ];

    let prefix = resolve_context_parts_to_prompt_prefix(&parts, &[], &[]).expect("should resolve");

    assert!(
        prefix.contains("<context source="),
        "should contain resource block"
    );
    assert!(
        prefix.contains("<attachment path="),
        "should contain file block"
    );
    assert!(
        prefix.contains("let x = 42;"),
        "should contain file content"
    );
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A typed context part that can be attached to an AI composer message.
///
/// Each variant represents a different source of context that will be
/// resolved into a prompt block at submit time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum AiContextPart {
    /// An MCP resource URI (e.g. `kit://context?profile=minimal`)
    ResourceUri { uri: String, label: String },
    /// A local file path attachment
    FilePath { path: String, label: String },
}

impl AiContextPart {
    pub fn label(&self) -> &str {
        match self {
            Self::ResourceUri { label, .. } | Self::FilePath { label, .. } => label,
        }
    }
}

/// Resolve a single context part into a prompt block string.
///
/// - `ResourceUri` resolves via `mcp_resources::read_resource`.
/// - `FilePath` reads the file; falls back to metadata-only if unreadable.
pub fn resolve_context_part_to_prompt_block(
    part: &AiContextPart,
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> Result<String> {
    match part {
        AiContextPart::ResourceUri { uri, .. } => {
            let content = crate::mcp_resources::read_resource(uri, scripts, scriptlets, None)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("Failed to read MCP resource: {uri}"))?;

            tracing::info!(
                kind = "resource_uri",
                uri = %content.uri,
                mime_type = %content.mime_type,
                "Resolved resource URI context part"
            );

            Ok(format!(
                "<context source=\"{}\" mimeType=\"{}\">\n{}\n</context>",
                content.uri, content.mime_type, content.text
            ))
        }
        AiContextPart::FilePath { path, .. } => match std::fs::read_to_string(path) {
            Ok(text) => {
                tracing::info!(
                    kind = "file_path_readable",
                    path = %path,
                    bytes = text.len(),
                    "Resolved file path context part"
                );
                Ok(format!(
                    "<attachment path=\"{}\">\n{}\n</attachment>",
                    path, text
                ))
            }
            Err(_) => {
                let metadata = std::fs::metadata(path)
                    .with_context(|| format!("Failed to stat attachment: {path}"))?;

                tracing::info!(
                    kind = "file_path_unreadable",
                    path = %path,
                    bytes = metadata.len(),
                    "Resolved unreadable file path context part (metadata-only fallback)"
                );

                Ok(format!(
                    "<attachment path=\"{}\" unreadable=\"true\" bytes=\"{}\" />",
                    path,
                    metadata.len()
                ))
            }
        },
    }
}

/// Resolve multiple context parts into a single prompt prefix string.
pub fn resolve_context_parts_to_prompt_prefix(
    parts: &[AiContextPart],
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> Result<String> {
    let mut blocks = Vec::new();

    for part in parts {
        blocks.push(resolve_context_part_to_prompt_block(part, scripts, scriptlets)?);
    }

    Ok(blocks.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_roundtrip_resource_uri() {
        let part = AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        };
        let json = serde_json::to_string(&part).expect("serialize");
        let deserialized: AiContextPart = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(part, deserialized);
        assert!(json.contains("\"kind\":\"resourceUri\""));
    }

    #[test]
    fn test_serde_roundtrip_file_path() {
        let part = AiContextPart::FilePath {
            path: "/tmp/test.rs".to_string(),
            label: "test.rs".to_string(),
        };
        let json = serde_json::to_string(&part).expect("serialize");
        let deserialized: AiContextPart = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(part, deserialized);
        assert!(json.contains("\"kind\":\"filePath\""));
    }

    #[test]
    fn test_label_accessor() {
        let uri_part = AiContextPart::ResourceUri {
            uri: "kit://context".to_string(),
            label: "Context".to_string(),
        };
        assert_eq!(uri_part.label(), "Context");

        let file_part = AiContextPart::FilePath {
            path: "/tmp/foo.rs".to_string(),
            label: "foo.rs".to_string(),
        };
        assert_eq!(file_part.label(), "foo.rs");
    }

    #[test]
    fn test_resolve_readable_file_path() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("hello.txt");
        std::fs::write(&file_path, "Hello, world!").expect("write temp file");

        let part = AiContextPart::FilePath {
            path: file_path.to_string_lossy().to_string(),
            label: "hello.txt".to_string(),
        };

        let block =
            resolve_context_part_to_prompt_block(&part, &[], &[]).expect("resolve should succeed");

        assert!(block.contains("<attachment path=\""));
        assert!(block.contains("Hello, world!"));
        assert!(block.contains("</attachment>"));
        assert!(!block.contains("unreadable"));
    }

    #[test]
    fn test_resolve_unreadable_file_path_does_not_panic() {
        // Create a file, make it exist but unreadable by removing read permissions
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("binary.dat");
        std::fs::write(&file_path, vec![0u8; 64]).expect("write temp file");

        // On Unix, remove read permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o000))
                .expect("set permissions");
        }

        let part = AiContextPart::FilePath {
            path: file_path.to_string_lossy().to_string(),
            label: "binary.dat".to_string(),
        };

        // On unix, this should produce an unreadable fallback (metadata-only)
        #[cfg(unix)]
        {
            let block = resolve_context_part_to_prompt_block(&part, &[], &[])
                .expect("resolve should not panic");
            assert!(block.contains("unreadable=\"true\""));
            assert!(block.contains("bytes=\"64\""));
        }

        // On non-unix, file is readable, so just verify no panic
        #[cfg(not(unix))]
        {
            let _ = resolve_context_part_to_prompt_block(&part, &[], &[]);
        }

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644));
        }
    }

    #[test]
    fn test_resolve_nonexistent_file_returns_error() {
        let part = AiContextPart::FilePath {
            path: "/nonexistent/path/that/does/not/exist.txt".to_string(),
            label: "ghost.txt".to_string(),
        };

        let result = resolve_context_part_to_prompt_block(&part, &[], &[]);
        assert!(result.is_err(), "nonexistent file should error");
    }

    #[test]
    fn test_resolve_multiple_parts() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file1 = dir.path().join("a.txt");
        let file2 = dir.path().join("b.txt");
        std::fs::write(&file1, "content A").expect("write");
        std::fs::write(&file2, "content B").expect("write");

        let parts = vec![
            AiContextPart::FilePath {
                path: file1.to_string_lossy().to_string(),
                label: "a.txt".to_string(),
            },
            AiContextPart::FilePath {
                path: file2.to_string_lossy().to_string(),
                label: "b.txt".to_string(),
            },
        ];

        let prefix =
            resolve_context_parts_to_prompt_prefix(&parts, &[], &[]).expect("resolve prefix");
        assert!(prefix.contains("content A"));
        assert!(prefix.contains("content B"));
        // Two blocks separated by double newline
        assert!(prefix.contains("</attachment>\n\n<attachment"));
    }

    #[test]
    fn test_resolve_empty_parts_returns_empty_string() {
        let prefix =
            resolve_context_parts_to_prompt_prefix(&[], &[], &[]).expect("resolve empty");
        assert!(prefix.is_empty());
    }
}

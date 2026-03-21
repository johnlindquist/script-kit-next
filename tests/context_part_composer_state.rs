//! Integration tests for the composer-state SSOT contract.
//!
//! Validates that `pending_context_parts` is the single source of truth for
//! attachment state: rendering derives from `file_path_parts()`, removal
//! targets exactly one matching `FilePath` entry, and clearing only affects
//! `FilePath` entries while leaving `ResourceUri` parts intact.

use script_kit_gpui::ai::message_parts::{file_path_parts, AiContextPart};

// ---------- file_path_parts helper ----------

#[test]
fn file_path_parts_extracts_only_file_paths() {
    let parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context".to_string(),
            label: "Context".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/a.rs".to_string(),
            label: "a.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/b.rs".to_string(),
            label: "b.rs".to_string(),
        },
    ];

    let paths = file_path_parts(&parts);
    assert_eq!(paths, vec!["/tmp/a.rs", "/tmp/b.rs"]);
}

#[test]
fn file_path_parts_returns_empty_for_no_file_paths() {
    let parts = vec![AiContextPart::ResourceUri {
        uri: "kit://context".to_string(),
        label: "Context".to_string(),
    }];

    let paths = file_path_parts(&parts);
    assert!(paths.is_empty());
}

#[test]
fn file_path_parts_preserves_order() {
    let parts = vec![
        AiContextPart::FilePath {
            path: "/z.rs".to_string(),
            label: "z.rs".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://mid".to_string(),
            label: "Mid".to_string(),
        },
        AiContextPart::FilePath {
            path: "/a.rs".to_string(),
            label: "a.rs".to_string(),
        },
    ];

    let paths = file_path_parts(&parts);
    assert_eq!(paths, vec!["/z.rs", "/a.rs"]);
}

// ---------- Removal contract ----------

/// Mirrors the removal logic from `AiApp::remove_attachment` without GPUI context.
fn remove_attachment_from_parts(parts: &mut Vec<AiContextPart>, file_index: usize) -> bool {
    let abs_index = parts
        .iter()
        .enumerate()
        .filter(|(_, part)| matches!(part, AiContextPart::FilePath { .. }))
        .nth(file_index)
        .map(|(i, _)| i);

    if let Some(idx) = abs_index {
        parts.remove(idx);
        true
    } else {
        false
    }
}

#[test]
fn removing_attachment_removes_exactly_one_matching_file_path() {
    let mut parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context".to_string(),
            label: "Context".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/a.rs".to_string(),
            label: "a.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/b.rs".to_string(),
            label: "b.rs".to_string(),
        },
    ];

    // Remove the first file-path attachment (a.rs)
    assert!(remove_attachment_from_parts(&mut parts, 0));

    // Should have removed a.rs but kept the resource and b.rs
    assert_eq!(parts.len(), 2);
    assert_eq!(file_path_parts(&parts), vec!["/tmp/b.rs"]);

    // ResourceUri should be untouched
    assert!(matches!(
        &parts[0],
        AiContextPart::ResourceUri { uri, .. } if uri == "kit://context"
    ));
}

#[test]
fn removing_second_attachment_preserves_first() {
    let mut parts = vec![
        AiContextPart::FilePath {
            path: "/tmp/first.rs".to_string(),
            label: "first.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/second.rs".to_string(),
            label: "second.rs".to_string(),
        },
    ];

    // Remove the second file-path attachment
    assert!(remove_attachment_from_parts(&mut parts, 1));

    assert_eq!(parts.len(), 1);
    assert_eq!(file_path_parts(&parts), vec!["/tmp/first.rs"]);
}

#[test]
fn removing_out_of_bounds_index_is_noop() {
    let mut parts = vec![AiContextPart::FilePath {
        path: "/tmp/only.rs".to_string(),
        label: "only.rs".to_string(),
    }];

    assert!(!remove_attachment_from_parts(&mut parts, 5));
    assert_eq!(parts.len(), 1);
}

// ---------- Clear contract ----------

/// Mirrors the clearing logic from `AiApp::clear_attachments`.
fn clear_file_path_parts(parts: &mut Vec<AiContextPart>) -> usize {
    let before = file_path_parts(parts).len();
    parts.retain(|part| !matches!(part, AiContextPart::FilePath { .. }));
    before
}

#[test]
fn clearing_attachments_removes_all_file_path_context_parts() {
    let mut parts = vec![
        AiContextPart::FilePath {
            path: "/tmp/a.rs".to_string(),
            label: "a.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/b.rs".to_string(),
            label: "b.rs".to_string(),
        },
    ];

    let cleared = clear_file_path_parts(&mut parts);
    assert_eq!(cleared, 2);
    assert!(parts.is_empty());
    assert!(file_path_parts(&parts).is_empty());
}

#[test]
fn clearing_attachments_leaves_resource_uri_parts_intact() {
    let mut parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context".to_string(),
            label: "Context".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/file.rs".to_string(),
            label: "file.rs".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://browser".to_string(),
            label: "Browser".to_string(),
        },
    ];

    clear_file_path_parts(&mut parts);

    assert_eq!(parts.len(), 2);
    assert!(file_path_parts(&parts).is_empty());
    assert!(matches!(
        &parts[0],
        AiContextPart::ResourceUri { uri, .. } if uri == "kit://context"
    ));
    assert!(matches!(
        &parts[1],
        AiContextPart::ResourceUri { uri, .. } if uri == "kit://browser"
    ));
}

// ---------- Non-file removal does not corrupt file-path state ----------

#[test]
fn removing_non_file_context_part_does_not_corrupt_file_path_rendering() {
    let mut parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context".to_string(),
            label: "Context".to_string(),
        },
        AiContextPart::FilePath {
            path: "/tmp/keep.rs".to_string(),
            label: "keep.rs".to_string(),
        },
    ];

    // Remove the ResourceUri at overall index 0
    parts.remove(0);

    // File-path rendering should still work correctly
    assert_eq!(file_path_parts(&parts), vec!["/tmp/keep.rs"]);
    assert_eq!(parts.len(), 1);
}

// ---------- Dedup contract ----------

/// Mirrors the dedup logic from `AiApp::add_attachment`.
fn add_attachment(parts: &mut Vec<AiContextPart>, path: &str) -> bool {
    let already_present = parts.iter().any(|part| {
        matches!(part, AiContextPart::FilePath { path: p, .. } if p == path)
    });
    if already_present {
        return false;
    }
    let label = std::path::Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    parts.push(AiContextPart::FilePath {
        path: path.to_string(),
        label,
    });
    true
}

#[test]
fn add_attachment_deduplicates_by_path() {
    let mut parts = Vec::new();
    assert!(add_attachment(&mut parts, "/tmp/file.rs"));
    assert!(!add_attachment(&mut parts, "/tmp/file.rs"));
    assert_eq!(parts.len(), 1);
}

#[test]
fn add_attachment_does_not_confuse_file_path_with_resource_uri() {
    let mut parts = vec![AiContextPart::ResourceUri {
        uri: "/tmp/file.rs".to_string(),
        label: "file.rs".to_string(),
    }];

    // Same string as a FilePath should still be added (different kind)
    assert!(add_attachment(&mut parts, "/tmp/file.rs"));
    assert_eq!(parts.len(), 2);
}

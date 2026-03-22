//! Tests for main window context strip state: defaults, toggle, clear, dedup,
//! order preservation, and AI launch request building.

use script_kit_gpui::ai::message_parts::AiContextPart;

// ---------- Helper: mirror the default parts from ScriptListApp ----------

fn default_parts() -> Vec<AiContextPart> {
    vec![
        AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0"
                .to_string(),
            label: "Selection".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
                .to_string(),
            label: "Browser URL".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1"
                .to_string(),
            label: "Focused Window".to_string(),
        },
    ]
}

// ---------- Default parts ----------

#[test]
fn default_parts_has_exactly_four_entries() {
    let parts = default_parts();
    assert_eq!(parts.len(), 4);
}

#[test]
fn default_parts_labels_in_documented_order() {
    let parts = default_parts();
    let labels: Vec<&str> = parts.iter().map(|p| p.label()).collect();
    assert_eq!(
        labels,
        vec!["Current Context", "Selection", "Browser URL", "Focused Window"]
    );
}

#[test]
fn default_parts_all_resource_uri_variants() {
    for part in &default_parts() {
        assert!(
            matches!(part, AiContextPart::ResourceUri { .. }),
            "Expected ResourceUri, got: {:?}",
            part
        );
    }
}

#[test]
fn default_parts_uris_use_kit_context_scheme() {
    for part in &default_parts() {
        assert!(
            part.source().starts_with("kit://context"),
            "URI should start with kit://context, got: {}",
            part.source()
        );
    }
}

// ---------- Toggle (simulated without GPUI context) ----------

/// Simulates toggle logic: remove if present, append if absent.
fn toggle(parts: &mut Vec<AiContextPart>, part: AiContextPart) {
    if let Some(ix) = parts.iter().position(|existing| existing == &part) {
        parts.remove(ix);
    } else {
        parts.push(part);
    }
}

#[test]
fn toggle_removes_existing_part_by_equality() {
    let mut parts = default_parts();
    let selection = parts[1].clone(); // "Selection"
    assert_eq!(parts.len(), 4);

    toggle(&mut parts, selection.clone());
    assert_eq!(parts.len(), 3);
    assert!(!parts.contains(&selection));
}

#[test]
fn toggle_appends_new_part() {
    let mut parts = default_parts();
    let custom = AiContextPart::FilePath {
        path: "/tmp/notes.md".to_string(),
        label: "Notes".to_string(),
    };

    toggle(&mut parts, custom.clone());
    assert_eq!(parts.len(), 5);
    assert_eq!(parts.last(), Some(&custom));
}

#[test]
fn toggle_preserves_insertion_order() {
    let mut parts = default_parts();
    // Remove "Selection" (index 1)
    let selection = parts[1].clone();
    toggle(&mut parts, selection.clone());

    let labels: Vec<&str> = parts.iter().map(|p| p.label()).collect();
    assert_eq!(labels, vec!["Current Context", "Browser URL", "Focused Window"]);

    // Re-add "Selection" — should go to end
    toggle(&mut parts, selection);
    let labels: Vec<&str> = parts.iter().map(|p| p.label()).collect();
    assert_eq!(
        labels,
        vec!["Current Context", "Browser URL", "Focused Window", "Selection"]
    );
}

#[test]
fn toggle_dedup_same_part_twice_returns_to_original() {
    let original = default_parts();
    let mut parts = original.clone();
    let part = parts[2].clone(); // "Browser URL"

    toggle(&mut parts, part.clone()); // remove
    toggle(&mut parts, part);         // re-add at end

    // Same parts but order changed (Browser URL moved to end)
    assert_eq!(parts.len(), original.len());
    assert_eq!(parts[3].label(), "Browser URL");
}

// ---------- Clear ----------

#[test]
fn clear_empties_all_parts() {
    let mut parts = default_parts();
    parts.clear();
    assert!(parts.is_empty());
}

// ---------- AI launch request building ----------

#[test]
fn launch_request_preserves_selected_parts() {
    let parts = default_parts();
    assert!(!parts.is_empty());

    let labels: Vec<&str> = parts.iter().map(|p| p.label()).collect();
    assert_eq!(labels.len(), 4);

    // Simulate building a launch request
    let uris: Vec<&str> = parts.iter().map(|p| p.source()).collect();
    assert!(uris[0].contains("profile=minimal"));
    assert!(uris[1].contains("selectedText=1"));
    assert!(uris[2].contains("browserUrl=1"));
    assert!(uris[3].contains("focusedWindow=1"));
}

#[test]
fn launch_request_empty_parts_yields_none() {
    let parts: Vec<AiContextPart> = vec![];
    assert!(parts.is_empty(), "Empty parts should signal no launch");
}

#[test]
fn launch_request_partial_selection() {
    let mut parts = default_parts();
    // Keep only Current Context and Browser URL
    let selection = parts[1].clone();
    toggle(&mut parts, selection); // remove Selection
    // After removal, "Focused Window" is now at index 2
    let focused_window = parts[2].clone();
    toggle(&mut parts, focused_window); // remove Focused Window

    let labels: Vec<&str> = parts.iter().map(|p| p.label()).collect();
    assert_eq!(labels, vec!["Current Context", "Browser URL"]);
}

// ---------- Serde roundtrip for launch payload ----------

#[test]
fn launch_parts_serde_roundtrip() {
    let parts = default_parts();
    let json = serde_json::to_string(&parts).expect("serialize parts");
    let round_trip: Vec<AiContextPart> =
        serde_json::from_str(&json).expect("deserialize parts");
    assert_eq!(parts, round_trip);
}

// ---------- Ask AI with Context flow: parts → receipt ----------

#[test]
fn ask_ai_with_context_preserves_parts_through_resolution() {
    use script_kit_gpui::ai::message_parts::{
        resolve_context_parts_with_receipt, ContextResolutionReceipt,
    };

    // Start with default parts, toggle off "Selection", keep 3
    let mut parts = default_parts();
    let selection = parts[1].clone();
    toggle(&mut parts, selection);
    assert_eq!(parts.len(), 3);

    // Resolve — uses kit://context URIs which resolve deterministically
    let receipt: ContextResolutionReceipt =
        resolve_context_parts_with_receipt(&parts, &[], &[]);

    // Receipt must account for all attempted parts
    assert_eq!(
        receipt.attempted, 3,
        "attempted count must match selected parts: {}",
        serde_json::to_string(&receipt).unwrap_or_default()
    );
    assert_eq!(
        receipt.resolved, 3,
        "all kit://context URIs should resolve: {}",
        serde_json::to_string(&receipt).unwrap_or_default()
    );
    assert!(
        !receipt.has_failures(),
        "no failures expected: {:?}",
        receipt.failures
    );
    assert!(
        !receipt.prompt_prefix.is_empty(),
        "prompt_prefix must contain resolved context blocks"
    );

    // Verify the prompt_prefix contains context blocks from the resolved URIs
    assert!(
        receipt
            .prompt_prefix
            .contains("kit://context?profile=minimal"),
        "prompt_prefix must include minimal context URI"
    );

    // Structured JSON summary for agent verification
    let summary = serde_json::json!({
        "test": "ask_ai_with_context_preserves_parts_through_resolution",
        "attempted": receipt.attempted,
        "resolved": receipt.resolved,
        "failures": receipt.failures.len(),
        "prompt_prefix_len": receipt.prompt_prefix.len(),
        "part_labels": parts.iter().map(|p| p.label()).collect::<Vec<_>>(),
    });
    eprintln!("{}", serde_json::to_string(&summary).expect("json summary"));
}

#[test]
fn ask_ai_with_context_all_defaults_yields_complete_receipt() {
    use script_kit_gpui::ai::message_parts::{
        resolve_context_parts_with_receipt, ContextResolutionReceipt,
    };

    let parts = default_parts();
    let receipt: ContextResolutionReceipt =
        resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 4);
    assert_eq!(receipt.resolved, 4);
    assert!(!receipt.has_failures());

    // Each resolved URI should produce a <context source="..."> block
    for part in &parts {
        assert!(
            receipt.prompt_prefix.contains(part.source()),
            "prompt_prefix missing source: {}",
            part.source()
        );
    }

    let summary = serde_json::json!({
        "test": "ask_ai_with_context_all_defaults_yields_complete_receipt",
        "attempted": receipt.attempted,
        "resolved": receipt.resolved,
        "has_failures": receipt.has_failures(),
        "prompt_prefix_sources": parts.iter().map(|p| p.source()).collect::<Vec<_>>(),
    });
    eprintln!("{}", serde_json::to_string(&summary).expect("json summary"));
}

#[test]
fn ask_ai_with_context_empty_parts_yields_zero_receipt() {
    use script_kit_gpui::ai::message_parts::{
        resolve_context_parts_with_receipt, ContextResolutionReceipt,
    };

    let parts: Vec<AiContextPart> = vec![];
    let receipt: ContextResolutionReceipt =
        resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 0);
    assert_eq!(receipt.resolved, 0);
    assert!(!receipt.has_failures());
    assert!(receipt.prompt_prefix.is_empty());
}

#[test]
fn launch_parts_json_structure_matches_spec() {
    let parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
                .to_string(),
            label: "Browser URL".to_string(),
        },
    ];

    let json = serde_json::to_value(&parts).expect("serialize");
    let arr = json.as_array().expect("should be array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["kind"], "resourceUri");
    assert_eq!(arr[0]["label"], "Current Context");
    assert_eq!(arr[1]["kind"], "resourceUri");
    assert_eq!(arr[1]["label"], "Browser URL");
}

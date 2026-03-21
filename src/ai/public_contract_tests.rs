//! Black-box smoke test for the publicly exposed AI context API surface.
//!
//! This module lives outside the private `context_contract` implementation
//! and imports exclusively through `crate::ai::*` to prove that the
//! `pub` / `pub use` surface is sufficient for external verification.

use crate::ai::{context_attachment_specs, AiContextPart, ContextAttachmentKind};

/// Validates that every built-in attachment kind round-trips through
/// action ID, slash command, mention, URI, and label — using only
/// the public API surface exposed from `crate::ai`.
#[test]
fn public_context_attachment_contract_round_trips() {
    let specs = context_attachment_specs();
    assert!(!specs.is_empty(), "spec table must not be empty");

    let mut matrix: Vec<serde_json::Value> = Vec::new();

    for spec in specs {
        // kind.spec() must return the same spec
        let via_kind = spec.kind.spec();
        assert_eq!(
            via_kind.action_id, spec.action_id,
            "spec() action_id drift for {:?}",
            spec.kind
        );
        assert_eq!(
            via_kind.uri, spec.uri,
            "spec() URI drift for {:?}",
            spec.kind
        );
        assert_eq!(
            via_kind.label, spec.label,
            "spec() label drift for {:?}",
            spec.kind
        );

        // kind.part() must produce a ResourceUri with matching uri/label
        match spec.kind.part() {
            AiContextPart::ResourceUri { uri, label } => {
                assert_eq!(uri, spec.uri, "part() URI mismatch for {:?}", spec.kind);
                assert_eq!(
                    label, spec.label,
                    "part() label mismatch for {:?}",
                    spec.kind
                );
            }
            other => panic!("expected ResourceUri for {:?}, got {other:?}", spec.kind),
        }

        // from_action_id round-trip (both prefixed and bare)
        assert_eq!(
            ContextAttachmentKind::from_action_id(spec.action_id),
            Some(spec.kind),
            "from_action_id failed for {:?}",
            spec.kind
        );
        if let Some(bare) = spec.action_id.strip_prefix("chat:") {
            assert_eq!(
                ContextAttachmentKind::from_action_id(bare),
                Some(spec.kind),
                "from_action_id (bare) failed for {:?}",
                spec.kind
            );
        }

        // from_slash_command round-trip
        if let Some(slash) = spec.slash_command {
            assert_eq!(
                ContextAttachmentKind::from_slash_command(slash),
                Some(spec.kind),
                "from_slash_command failed for {:?}",
                spec.kind
            );
        }

        // from_mention_line round-trip
        if let Some(mention) = spec.mention {
            assert_eq!(
                ContextAttachmentKind::from_mention_line(mention),
                Some(spec.kind),
                "from_mention_line failed for {:?}",
                spec.kind
            );
        }

        // Build JSON row for the attachment matrix
        matrix.push(serde_json::json!({
            "kind": format!("{:?}", spec.kind),
            "action_id": spec.action_id,
            "action_title": spec.action_title,
            "slash_command": spec.slash_command,
            "mention": spec.mention,
            "uri": spec.uri,
            "label": spec.label,
        }));

        tracing::info!(
            kind = ?spec.kind,
            action_id = spec.action_id,
            uri = spec.uri,
            label = spec.label,
            "public_contract_spec_verified"
        );
    }

    // Print the full attachment matrix as JSON for agent diffing
    let matrix_json =
        serde_json::to_string_pretty(&matrix).expect("attachment matrix must serialize");
    println!("--- PUBLIC_ATTACHMENT_MATRIX_JSON ---");
    println!("{matrix_json}");
    println!("--- END_PUBLIC_ATTACHMENT_MATRIX_JSON ---");

    tracing::info!(
        total_specs = specs.len(),
        "public_context_attachment_contract_complete"
    );
}

use script_kit_gpui::ai::message_parts::AiContextPart;
use script_kit_gpui::test_support::acp_portal as portal;

// @lat: [[tests/acp-portal-contract#Clipboard history portal#Host-aware refusal leaves ACP idle]]
#[test]
fn clipboard_history_host_aware_refusal_leaves_acp_idle() {
    assert_eq!(
        portal::open_refusal(true, false),
        Some("missing_host_callback")
    );
    assert_eq!(
        portal::open_refusal(false, true),
        Some("unsupported_by_host")
    );
    assert_eq!(
        portal::state_transition("idle", "stage"),
        Some("staged".into())
    );
}

// @lat: [[tests/acp-portal-contract#Clipboard history portal#Round-trip accepts kit id URIs and preserves the inline token]]
#[test]
fn clipboard_history_round_trip_accepts_id_uri_and_preserves_inline_token() {
    let part = AiContextPart::ResourceUri {
        uri: "kit://clipboard-history?id=clip-123".to_string(),
        label: "Clipboard: copied text".to_string(),
    };

    assert_eq!(
        portal::clipboard_part_token("clip-123"),
        Some("@clipboard".into())
    );
    assert_eq!(
        portal::part_target(&part),
        Some(("clipboard_history".into(), "copied text".into()))
    );
    assert_eq!(
        portal::inline_target("@clipboard"),
        Some(("clipboard_history".into(), String::new()))
    );
    assert_eq!(
        portal::picker_query("clipboard_history", "copied text"),
        Some("copied text".into())
    );
}

// @lat: [[tests/acp-portal-contract#Clipboard history portal#Attach replaces exact range and terminal states clear to idle]]
#[test]
fn clipboard_history_attach_replaces_exact_range_and_terminal_states_clear() {
    assert_eq!(
        portal::state_transition("staged", "activate"),
        Some("active".into())
    );
    assert_eq!(
        portal::state_transition("active", "accept"),
        Some("accepted".into())
    );
    assert_eq!(
        portal::clear_terminal_state("accepted"),
        Some("idle".into())
    );
    assert_eq!(
        portal::state_transition("active", "cancel"),
        Some("cancelled".into())
    );
    assert_eq!(
        portal::clear_terminal_state("cancelled"),
        Some("idle".into())
    );
    assert_eq!(
        portal::state_transition("active", "orphan"),
        Some("orphaned".into())
    );
    assert_eq!(
        portal::clear_terminal_state("orphaned"),
        Some("idle".into())
    );

    let (next_text, next_cursor, exact_match) =
        portal::replacement("summarize @clipboard", 10..20, 10, "@clipboard ");
    assert!(exact_match);
    assert_eq!(next_text, "summarize @clipboard ");
    assert_eq!(next_cursor, next_text.chars().count());
}

// @lat: [[tests/acp-portal-contract#Dictation history portal#Host-aware refusal leaves ACP idle]]
#[test]
fn dictation_history_host_aware_refusal_leaves_acp_idle() {
    assert_eq!(
        portal::open_refusal(true, false),
        Some("missing_host_callback")
    );
    assert_eq!(
        portal::open_refusal(false, true),
        Some("unsupported_by_host")
    );
    assert_eq!(
        portal::state_transition("idle", "stage"),
        Some("staged".into())
    );
}

// @lat: [[tests/acp-portal-contract#Dictation history portal#Production URI construction pairs with inline token]]
#[test]
fn dictation_history_production_uri_pairs_with_inline_token() {
    let id = "test-id-7f3a";
    let part = portal::production_dictation_part(id, "sample dictation preview");

    let AiContextPart::ResourceUri { uri, label } = &part else {
        panic!(
            "dictation_history_part_for_entry must return ResourceUri, got {:?}",
            part
        );
    };

    assert!(
        uri.starts_with("kit://dictation-history?id="),
        "production URI must carry the dictation-history scheme prefix, got {uri:?}"
    );
    assert_eq!(
        uri.strip_prefix("kit://dictation-history?id=").unwrap(),
        id,
        "URI tail must be the entry id verbatim"
    );
    assert!(
        label.starts_with("Dictation: "),
        "label should describe the part as a dictation entry, got {label:?}"
    );

    let token = portal::part_inline_token(&part)
        .expect("production dictation part must serialize to an inline token");
    assert_eq!(
        token,
        format!("@dictation:{id}"),
        "inline token must pair id-for-id with the URI constructed by the production helper"
    );
}

// @lat: [[tests/acp-portal-contract#Dictation history portal#Round-trip preserves history id but opens unfiltered]]
#[test]
fn dictation_history_round_trip_preserves_history_id_but_opens_unfiltered() {
    assert_eq!(
        portal::dictation_part_token("abc123"),
        Some("@dictation:abc123".into())
    );
    assert_eq!(
        portal::inline_target("@dictation:abc123"),
        Some(("dictation_history".into(), "abc123".into()))
    );
    assert_eq!(
        portal::picker_query("dictation_history", "abc123"),
        Some(String::new())
    );
    assert_eq!(
        portal::picker_query("browser_history", "github"),
        Some("github".into())
    );
}

// @lat: [[tests/acp-portal-contract#Dictation history portal#Attach replaces exact range and terminal states clear to idle]]
#[test]
fn dictation_history_attach_replaces_exact_range_and_terminal_states_clear() {
    assert_eq!(
        portal::state_transition("staged", "activate"),
        Some("active".into())
    );
    assert_eq!(
        portal::state_transition("active", "accept"),
        Some("accepted".into())
    );
    assert_eq!(
        portal::clear_terminal_state("accepted"),
        Some("idle".into())
    );
    assert_eq!(
        portal::state_transition("active", "cancel"),
        Some("cancelled".into())
    );
    assert_eq!(
        portal::clear_terminal_state("cancelled"),
        Some("idle".into())
    );
    assert_eq!(
        portal::state_transition("active", "orphan"),
        Some("orphaned".into())
    );
    assert_eq!(
        portal::clear_terminal_state("orphaned"),
        Some("idle".into())
    );

    let (next_text, next_cursor, exact_match) = portal::replacement(
        "summarize @dictation:abc123",
        10..27,
        10,
        "@dictation:abc123 ",
    );
    assert!(exact_match);
    assert_eq!(next_text, "summarize @dictation:abc123 ");
    assert_eq!(next_cursor, next_text.chars().count());
}

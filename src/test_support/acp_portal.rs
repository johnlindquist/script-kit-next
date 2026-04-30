use std::ops::Range;

use crate::ai::acp::portal_contract::{
    apply_portal_replacement, clear_terminal_portal_state, decide_portal_open,
    exact_replacement_target_for_range, next_portal_state, picker_portal_query,
    portal_target_from_inline_token, portal_target_from_part, AcpPortalOpenRefusal,
    AcpPortalSessionEvent, AcpPortalSessionState,
};
use crate::ai::message_parts::AiContextPart;
use crate::ai::window::context_picker::types::PortalKind;

fn parse_kind(kind: &str) -> Option<PortalKind> {
    match kind {
        "file_search" => Some(PortalKind::FileSearch),
        "browser_history" => Some(PortalKind::BrowserHistory),
        "clipboard_history" => Some(PortalKind::ClipboardHistory),
        "dictation_history" => Some(PortalKind::DictationHistory),
        "script_search" => Some(PortalKind::ScriptSearch),
        "scriptlet_search" => Some(PortalKind::ScriptletSearch),
        "skill_search" => Some(PortalKind::SkillSearch),
        "notes_browse" => Some(PortalKind::NotesBrowse),
        "acp_history" => Some(PortalKind::AcpHistory),
        _ => None,
    }
}

fn format_kind(kind: PortalKind) -> String {
    match kind {
        PortalKind::FileSearch => "file_search",
        PortalKind::BrowserHistory => "browser_history",
        PortalKind::ClipboardHistory => "clipboard_history",
        PortalKind::DictationHistory => "dictation_history",
        PortalKind::ScriptSearch => "script_search",
        PortalKind::ScriptletSearch => "scriptlet_search",
        PortalKind::SkillSearch => "skill_search",
        PortalKind::NotesBrowse => "notes_browse",
        PortalKind::AcpHistory => "acp_history",
    }
    .to_string()
}

#[doc(hidden)]
pub fn picker_query(kind: &str, session_query: &str) -> Option<String> {
    Some(picker_portal_query(parse_kind(kind)?, session_query))
}

#[doc(hidden)]
pub fn inline_target(token: &str) -> Option<(String, String)> {
    portal_target_from_inline_token(token).map(|(kind, query)| (format_kind(kind), query))
}

#[doc(hidden)]
pub fn part_target(part: &AiContextPart) -> Option<(String, String)> {
    portal_target_from_part(part).map(|(kind, query)| (format_kind(kind), query))
}

#[doc(hidden)]
pub fn dictation_part_token(id: &str) -> Option<String> {
    let part = AiContextPart::ResourceUri {
        uri: format!("kit://dictation-history?id={id}"),
        label: format!("Dictation: {id}"),
    };
    crate::ai::context_mentions::part_to_inline_token(&part)
}

#[doc(hidden)]
pub fn production_dictation_part(id: &str, preview: &str) -> AiContextPart {
    let entry = crate::dictation::DictationHistoryEntry {
        id: id.to_string(),
        timestamp: "2026-04-16T00:00:00Z".to_string(),
        transcript: preview.to_string(),
        preview: preview.to_string(),
        target: String::new(),
        audio_duration_ms: 0,
    };
    crate::ai::context_mentions::dictation_history_part_for_entry(&entry)
}

#[doc(hidden)]
pub fn part_inline_token(part: &AiContextPart) -> Option<String> {
    crate::ai::context_mentions::part_to_inline_token(part)
}

#[doc(hidden)]
pub fn clipboard_part_token(id: &str) -> Option<String> {
    let _ = id;
    Some("@clipboard".to_string())
}

#[doc(hidden)]
pub fn replacement(
    current_text: &str,
    char_range: Range<usize>,
    fallback_cursor: usize,
    replacement_text: &str,
) -> (String, usize, bool) {
    let target = exact_replacement_target_for_range(current_text, char_range, fallback_cursor);
    apply_portal_replacement(current_text, &target, replacement_text)
}

#[doc(hidden)]
pub fn open_refusal(allowed: bool, has_host_callback: bool) -> Option<&'static str> {
    match decide_portal_open(allowed, has_host_callback) {
        Ok(()) => None,
        Err(AcpPortalOpenRefusal::UnsupportedByHost) => Some("unsupported_by_host"),
        Err(AcpPortalOpenRefusal::MissingHostCallback) => Some("missing_host_callback"),
    }
}

fn parse_state(value: &str) -> Option<AcpPortalSessionState> {
    match value {
        "idle" => Some(AcpPortalSessionState::Idle),
        "staged" => Some(AcpPortalSessionState::Staged),
        "active" => Some(AcpPortalSessionState::Active),
        "accepted" => Some(AcpPortalSessionState::Accepted),
        "cancelled" => Some(AcpPortalSessionState::Cancelled),
        "orphaned" => Some(AcpPortalSessionState::Orphaned),
        _ => None,
    }
}

fn format_state(value: AcpPortalSessionState) -> String {
    match value {
        AcpPortalSessionState::Idle => "idle",
        AcpPortalSessionState::Staged => "staged",
        AcpPortalSessionState::Active => "active",
        AcpPortalSessionState::Accepted => "accepted",
        AcpPortalSessionState::Cancelled => "cancelled",
        AcpPortalSessionState::Orphaned => "orphaned",
    }
    .to_string()
}

#[doc(hidden)]
pub fn state_transition(state: &str, event: &str) -> Option<String> {
    let state = parse_state(state)?;
    let event = match event {
        "stage" => AcpPortalSessionEvent::Stage,
        "activate" => AcpPortalSessionEvent::Activate,
        "accept" => AcpPortalSessionEvent::Accept,
        "cancel" => AcpPortalSessionEvent::Cancel,
        "orphan" => AcpPortalSessionEvent::Orphan,
        _ => return None,
    };
    next_portal_state(state, event).map(format_state)
}

#[doc(hidden)]
pub fn clear_terminal_state(state: &str) -> Option<String> {
    parse_state(state)
        .map(clear_terminal_portal_state)
        .map(format_state)
}

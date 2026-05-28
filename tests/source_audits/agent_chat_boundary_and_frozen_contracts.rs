//! Source audit for the Agent Chat / ACP rename boundary.
//!
//! The project renamed the *feature* to "Agent Chat" while keeping "ACP" as the
//! name of frozen compatibility contracts (action IDs, route IDs, serialized
//! surface IDs, `getAcpState`, telemetry labels). This audit proves two things
//! at once:
//!
//! 1. The canonical `agent_chat::ui` boundary exists and is wired into the
//!    launcher view state, so new code has a stable `AgentChat*` import surface.
//! 2. The frozen external contracts are still present verbatim. If a future
//!    rename pass deletes or edits one of these strings, this audit fails and
//!    forces the change to be deliberate (and paired with a contract migration).

use std::fs;
use std::path::{Path, PathBuf};

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn collect_rs_files(root: impl AsRef<Path>, files: &mut Vec<PathBuf>) {
    let root = root.as_ref();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

#[test]
fn agent_chat_ui_boundary_exists() {
    let mod_rs = read("src/ai/agent_chat/mod.rs");
    assert!(
        mod_rs.contains("pub(crate) mod ui;"),
        "agent_chat must expose the canonical `ui` boundary module"
    );
    assert!(
        mod_rs.contains("pub(crate) mod content;"),
        "agent_chat must expose the `content` boundary module"
    );

    let ui = read("src/ai/agent_chat/ui/mod.rs");
    for alias in [
        "AcpChatView as AgentChatView",
        "AcpThread as AgentChatThread",
        "AcpEvent as AgentChatEvent",
        "AcpChatSession as AgentChatSession",
        "AcpInlineSetupState as AgentChatInlineSetupState",
        "AcpRetryRequest as AgentChatRetryRequest",
        "AcpPermissionBroker as AgentChatPermissionBroker",
    ] {
        assert!(
            ui.contains(alias),
            "agent_chat::ui must re-export `{alias}` as part of the canonical boundary"
        );
    }
}

#[test]
fn app_view_state_uses_agent_chat_ui_boundary() {
    let app_view = read("src/main_sections/app_view_state.rs");
    assert!(
        app_view.contains("crate::ai::agent_chat::ui::AgentChatView"),
        "AcpChatView variant entity must flow through the agent_chat::ui boundary"
    );
}

#[test]
fn agent_client_protocol_is_imported_only_through_content_boundary() {
    // The external `agent_client_protocol` crate is a type-only content-block
    // dependency. All of `src/` must reach it through the single
    // `agent_chat::content` choke point so the dependency stays visible and
    // swappable. The only file allowed to name the crate is content.rs.
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_rs_files(manifest.join("src"), &mut files);

    let allowed = manifest.join("src/ai/agent_chat/content.rs");
    let mut offenders = Vec::new();
    for file in files {
        if file == allowed {
            continue;
        }
        let contents = fs::read_to_string(&file).unwrap_or_default();
        if contents.contains("agent_client_protocol") {
            offenders.push(file.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "agent_client_protocol must only be referenced in \
         src/ai/agent_chat/content.rs; offenders: {offenders:?}"
    );
}

/// The chat-runtime/UI types that MUST flow through the `agent_chat::ui`
/// facade once an outer consumer references them by an `acp::` path.
const RUNTIME_TYPES: &[&str] = &[
    "AcpChatView",
    "AcpThread",
    "AcpThreadInit",
    "AcpThreadMessage",
    "AcpThreadStatus",
    "AcpEvent",
    "AcpEventRx",
    "AcpChatSession",
    "AcpInlineSetupState",
    "AcpSetupAction",
    "AcpRetryRequest",
    "AcpHistoryResumeRequest",
    "AcpPermissionBroker",
    "AcpLaunchBlocker",
    "AcpLaunchRequirements",
    "AcpLaunchResolution",
    "AcpToolCallState",
];

/// Returns the first runtime type that `content` reaches via an `acp::` path
/// (optionally through a single submodule segment, e.g. `acp::view::AcpThread`).
/// The frozen `AppView::AcpChatView` enum variant is NOT matched because it has
/// no `acp::` path prefix.
fn first_acp_runtime_reference(content: &str) -> Option<String> {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    for (idx, _) in content.match_indices("acp::") {
        let rest = &content[idx + "acp::".len()..];
        // Optionally skip one lowercase submodule segment like `view::`.
        let after_submodule = match rest.find("::") {
            Some(pos)
                if pos > 0
                    && rest[..pos]
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c == '_') =>
            {
                &rest[pos + 2..]
            }
            _ => rest,
        };
        for ty in RUNTIME_TYPES {
            if let Some(tail) = after_submodule.strip_prefix(ty) {
                if !tail.chars().next().is_some_and(is_ident) {
                    return Some((*ty).to_string());
                }
            }
        }
    }
    None
}

#[test]
fn outer_consumers_reach_runtime_types_through_agent_chat_ui() {
    // Outer feature roots must import the chat-runtime/UI types via the
    // `agent_chat::ui` facade, not directly from `crate::ai::acp`. Non-runtime
    // ACP compatibility surfaces (catalog/config/portal/history/surface_state/
    // actions-dialog/context builders) are allowed to stay on `crate::ai::acp`.
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let outer_roots = [
        "src/app_impl",
        "src/notes",
        "src/render_builtins",
        "src/actions",
        "src/app_actions",
        "src/prompt_handler",
        "src/test_support",
    ];

    let mut offenders = Vec::new();
    for root in outer_roots {
        let mut files = Vec::new();
        collect_rs_files(manifest.join(root), &mut files);
        for file in files {
            let contents = fs::read_to_string(&file).unwrap_or_default();
            if let Some(ty) = first_acp_runtime_reference(&contents) {
                offenders.push(format!("{} reaches acp::{ty}", file.display()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "outer consumers must import chat-runtime types from \
         crate::ai::agent_chat::ui, not crate::ai::acp; offenders: {offenders:#?}"
    );
}

#[test]
fn agent_chat_ui_facade_stays_runtime_only() {
    // The facade is a runtime/UI boundary; it must NOT absorb non-runtime ACP
    // compatibility surfaces. Those get their own future boundaries if needed.
    let ui = read("src/ai/agent_chat/ui/mod.rs");
    for forbidden in [
        "AcpAgentCatalogEntry",
        "AcpAgentConfig",
        "AcpAgentRuntimeState",
        "AcpActionsDialogContext",
        "AcpPortalLaunchContract",
        "AcpHistoryEntry",
        "portal_contract",
        "catalog::",
        "config::",
        "history::",
    ] {
        assert!(
            !ui.contains(forbidden),
            "agent_chat::ui must stay runtime-only; it must not re-export `{forbidden}`"
        );
    }
}

#[test]
fn frozen_serialized_surface_ids_are_unchanged() {
    let app_view = read("src/main_sections/app_view_state.rs");
    // Serialized view-type ids feed launcher surface contracts and automation;
    // these MUST stay stable even though the feature is now "Agent Chat".
    assert!(
        app_view.contains("Some(\"acp_chat\")"),
        "frozen serialized surface id `acp_chat` must remain"
    );
    assert!(
        app_view.contains("SurfaceKind::AcpChat"),
        "frozen SurfaceKind::AcpChat variant must remain"
    );
    assert!(
        app_view.contains("AppView::AcpChatView"),
        "frozen AppView::AcpChatView variant must remain"
    );
}

#[test]
fn frozen_action_and_route_ids_are_unchanged() {
    let script_context = read("src/actions/builders/script_context.rs");
    for id in ["acp:root", "acp:change_model", "acp_switch_model:"] {
        assert!(
            script_context.contains(id),
            "frozen action/route id `{id}` must remain in script_context.rs"
        );
    }
}

#[test]
fn frozen_get_acp_state_protocol_contract_is_unchanged() {
    let acp_state = read("src/protocol/types/acp_state.rs");
    assert!(
        acp_state.contains("ACP_STATE_SCHEMA_VERSION"),
        "frozen `ACP_STATE_SCHEMA_VERSION` automation contract must remain"
    );
    assert!(
        acp_state.contains("AcpStateSnapshot"),
        "frozen `AcpStateSnapshot` automation type must remain"
    );
}

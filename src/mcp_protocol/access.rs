//! The external bearer controls both discovery and dispatch. A read can disclose private data.
use super::bridge;
use crate::{mcp_control, mcp_kit_tools};

pub(super) fn tool(name: &str, scopes: &[String]) -> bool {
    if name.starts_with("kit/handoff_") {
        return bridge::handoff::permitted(name, scopes);
    }
    if let Some(scope) = bridge::scope(name) {
        return scopes.iter().any(|s| s == scope);
    }
    if name.starts_with("desktop/filename_") {
        return scopes.iter().any(|s| s == "desktop:fill");
    }
    if let Some(scope) = mcp_control::build_default_mutation_registry().required_scope(name) {
        return mcp_control::scope_allows(scopes, scope);
    }
    if mcp_kit_tools::requires_ui_control_scope(name) {
        return mcp_control::scope_allows(scopes, "ui:control");
    }
    if matches!(name, "kit/state" | "kit/agent_access") {
        return true;
    }
    if name.starts_with("computer/") {
        let scope = if matches!(
            name,
            "computer/see" | "computer/capture_native_window" | "computer/capture_render_window"
        ) {
            "computer:capture"
        } else {
            "computer:observe"
        };
        return mcp_control::scope_allows(scopes, scope)
            || (name == "computer/list_native_windows"
                && scopes.iter().any(|s| s == "desktop:fill"));
    }
    false
}
pub(super) fn resource(uri: &str, scopes: &[String]) -> bool {
    let base = uri.split('?').next().unwrap_or(uri);
    if matches!(
        base,
        "kit://state"
            | "kit://context/schema"
            | "kit://sdk-reference"
            | "kit://trigger-builtins"
            | "kit://stdin-commands"
            | "kit://computer-use/readiness"
            | "kit://transactions/schema"
    ) {
        return true;
    }
    // Unknown families are not implicitly readable. A domain grant covers that family only.
    if base == "kit://context"
        && uri.split_once('?').is_some_and(|(_, q)| {
            q.split('&').any(|p| {
                matches!(p.split('=').next(), Some("screenshot" | "panelScreenshot"))
                    && !matches!(p.split_once('=').map(|(_, v)| v), Some("0" | "false"))
            })
        })
        && !mcp_control::scope_allows(scopes, "computer:capture")
    {
        return false;
    }
    let scope = match base {
        "kit://context" => "context:read",
        "kit://clipboard-history" => "clipboard:read",
        "kit://dictation" | "kit://dictation-history" => "dictation:read",
        "kit://calendar" => "calendar:read",
        "kit://audit" => "audit:read",
        "kit://notes" => "notes:read",
        b if b.starts_with("kit://notes/") => "notes:read",
        "kit://brain" => "brain:read",
        b if b.starts_with("kit://brain/") => "brain:read",
        "kit://scripts"
        | "kit://scriptlets"
        | "scripts://"
        | "scriptlets://"
        | "kit://command-doctor"
        | "kit://failed-scripts"
        | "kit://script-templates" => "scripts:read",
        _ => {
            return scopes
                .iter()
                .any(|s| matches!(s.as_str(), "mcp:*" | "*" | "dev:*"))
        }
    };
    mcp_control::scope_allows(scopes, scope)
}

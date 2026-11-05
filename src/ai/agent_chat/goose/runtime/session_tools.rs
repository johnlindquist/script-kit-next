//! Goose 1.52 adds its configured default Developer extension *before* the recipe.
//! Reconcile that session, not the user's engine settings. A session is never
//! adopted, warmed, forked into, or prompted on an ambiguous tool inventory.
use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::{Connection, SessionFailure, CONFIG_TIMEOUT};
use crate::ai::agent_chat::goose::recipe::GOOSE_DEVELOPER_EXTENSION as DEVELOPER;

fn incompatible(detail: &'static str) -> SessionFailure {
    Box::new(crate::ai::reliability::protocol_failure_with_detail(
        sk_protocol::ai_reliability::ProtocolComponent::Goose,
        crate::ai::reliability::ProtocolFailureFacts::MalformedResponse,
        detail,
    ))
}

struct Plan<'a> {
    remove_ambient_developer: bool,
    allowed_developer_tools: BTreeSet<&'a str>,
}

fn plan<'a>(recipe: &'a Value, response: &Value) -> Result<Plan<'a>, SessionFailure> {
    let invalid_recipe = || incompatible("invalid bound extension policy");
    let invalid_reply = || incompatible("invalid session extension inventory");
    let mut declared = BTreeSet::new();
    let mut allowed_developer_tools = BTreeSet::new();
    for extension in recipe["extensions"].as_array().ok_or_else(invalid_recipe)? {
        let name = extension["name"]
            .as_str()
            .filter(|n| !n.is_empty())
            .ok_or_else(invalid_recipe)?;
        if !declared.insert(name) {
            return Err(invalid_recipe());
        }
        if name == DEVELOPER {
            // An empty filter enables *all* Developer tools in Goose, never a grant from Kit.
            for tool in extension["available_tools"]
                .as_array()
                .ok_or_else(invalid_recipe)?
            {
                let tool = tool
                    .as_str()
                    .filter(|n| !n.is_empty())
                    .ok_or_else(invalid_recipe)?;
                if !allowed_developer_tools.insert(tool) {
                    return Err(invalid_recipe());
                }
            }
            if allowed_developer_tools.is_empty() {
                return Err(invalid_recipe());
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut remove_ambient_developer = false;
    for entry in response["_meta"]["extensionResults"]
        .as_array()
        .ok_or_else(invalid_reply)?
    {
        let name = entry["name"]
            .as_str()
            .filter(|n| !n.is_empty())
            .ok_or_else(invalid_reply)?;
        let loaded = entry["success"].as_bool().ok_or_else(invalid_reply)?;
        if !seen.insert(name) {
            return Err(invalid_reply());
        }
        if !declared.contains(name) {
            if name != DEVELOPER {
                return Err(incompatible("engine reported an undeclared extension"));
            }
            remove_ambient_developer = loaded;
        }
    }
    // Known extensions may report a load failure (e.g. an offline MCP server),
    // but a missing load record is not evidence about that extension.
    if !declared.is_subset(&seen) {
        return Err(invalid_reply());
    }
    Ok(Plan {
        remove_ambient_developer,
        allowed_developer_tools,
    })
}

pub(super) async fn verify(
    connection: &mut Connection,
    spec: &super::GooseLaunchSpec,
    response: &Value,
    session_id: &str,
) -> Result<(), SessionFailure> {
    let plan = plan(&spec.recipe, response)?;
    if plan.remove_ambient_developer {
        let method = "_goose/unstable/session/extensions/remove";
        connection
            .call(
                method,
                json!({"sessionId":session_id,"extensionKey":DEVELOPER}),
                CONFIG_TIMEOUT,
            )
            .await
            .map_err(|e| Box::new(e.failure(method)))?;
    }
    // Read the actual tools on this exact session. A successful removal reply,
    // an extension name, and the recipe echoed by the engine are not readback.
    read_exact_tools(
        connection,
        session_id,
        DEVELOPER,
        &plan.allowed_developer_tools,
    )
    .await?;
    if let Some(access) = spec.launcher_author {
        use sk_personal::api::Endpoint;
        let mut required = BTreeSet::from(["scriptkit__launcher_read"]);
        if access == Endpoint::Apply {
            required.insert("scriptkit__launcher_apply");
        }
        read_exact_tools(
            connection,
            session_id,
            super::super::recipe::SCRIPT_KIT_TOOLS_EXTENSION,
            &required,
        )
        .await?;
    }
    Ok(())
}

async fn read_exact_tools(
    connection: &mut Connection,
    session_id: &str,
    owner: &str,
    required: &BTreeSet<&str>,
) -> Result<(), SessionFailure> {
    let method = "_goose/unstable/tools/list";
    let inventory = connection
        .call(
            method,
            json!({"sessionId":session_id,"extensionName":owner}),
            CONFIG_TIMEOUT,
        )
        .await
        .map_err(|e| Box::new(e.failure(method)))?;
    let invalid = || incompatible("engine tools differ from the bound session grant");
    let tools = inventory["tools"].as_array().ok_or_else(invalid)?;
    let mut actual = BTreeSet::new();
    for tool in tools {
        if !actual.insert(tool["name"].as_str().ok_or_else(invalid)?) {
            return Err(invalid());
        }
    }
    if &actual != required {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests;

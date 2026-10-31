use crate::{
    desktop_fill::{agent_api, FilenameApi, Status},
    mcp_kit_tools::ToolDefinition,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
pub(super) const PROPOSE: &str = "desktop/filename_propose";
pub(super) const APPLY: &str = "desktop/filename_apply";
pub(super) const STATUS: &str = "desktop/filename_status";
const SCOPE: &str = "desktop:fill";
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Target {
    app: String,
    pid: u32,
    native_window_id: u32,
    filename: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Handle {
    proposal_id: Uuid,
}
pub(super) fn definitions() -> Vec<ToolDefinition> {
    let handle = json!({"type":"object","additionalProperties":false,"required":["proposalId"],"properties":{"proposalId":{"type":"string","format":"uuid"}}});
    [(PROPOSE,"Inspect one open native Save sheet. Supply its app name, PID and native window number from computer/list_native_windows; returns a 60-second, caller-bound preview. No write. Requires explicit desktop:fill.",json!({"type":"object","additionalProperties":false,"required":["app","pid","nativeWindowId","filename"],"properties":{"app":{"type":"string"},"pid":{"type":"integer","minimum":1},"nativeWindowId":{"type":"integer","minimum":1},"filename":{"type":"string","minLength":1,"maxLength":255}}})),(APPLY,"Apply the reviewed filename once to its original field. Never clicks Save. Uncertain means inspect the result, not retry. Requires explicit desktop:fill.",handle.clone()),(STATUS,"Read the state of your filename proposal. No desktop access. Requires explicit desktop:fill.",handle)].into_iter().map(|(name,d,input_schema)|ToolDefinition{name:name.into(),description:d.into(),input_schema}).collect()
}
pub(super) async fn call(
    name: &str,
    args: Value,
    principal: Option<String>,
    scopes: &[String],
) -> Value {
    call_with(agent_api(), name, args, principal, scopes).await
}
async fn call_with(
    api: &'static dyn FilenameApi,
    name: &str,
    args: Value,
    principal: Option<String>,
    scopes: &[String],
) -> Value {
    let result = if let Some(owner) = principal.filter(|_| scopes.iter().any(|s| s == SCOPE)) {
        let name = name.to_owned();
        smol::unblock(move || match name.as_str() {
            PROPOSE => {
                let t: Target = serde_json::from_value(args).map_err(|_| "invalid_arguments")?;
                let p = api.propose(&owner, &t.app, t.pid, t.native_window_id, None, &t.filename)?;
                Ok(json!({"ok":true,"state":"proposed","proposal":p}))
            }
            APPLY | STATUS => {
                let h: Handle = serde_json::from_value(args).map_err(|_| "invalid_arguments")?;
                let state = if name == APPLY {api.apply(&owner, h.proposal_id)} else {api.status(&owner, h.proposal_id)}?;
                Ok(json!({"ok": !matches!(state, Status::Refused{..} | Status::Uncertain{..}),"proposalId":h.proposal_id,"result":state}))
            }
            _ => Err("unknown_tool"),
        }).await
    } else {
        Err("scope_denied")
    };
    let body = result
        .unwrap_or_else(|code| json!({"ok":false,"error":{"code":code,"requiredScope":SCOPE}}));
    json!({"isError":body["ok"]!=true,"content":[{"type":"text","text":body.to_string()}]})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wildcard_and_read_only_contexts_never_confer_desktop_authority() {
        for scopes in [
            vec!["mcp:*".into()],
            vec!["computer:act".into()],
            vec!["mcp:read".into()],
        ] {
            let out = smol::block_on(call(PROPOSE, json!({}), Some("mcp:a".into()), &scopes));
            assert!(out.to_string().contains("scope_denied"));
        }
        assert!(
            smol::block_on(call(PROPOSE, json!({}), None, &[SCOPE.into()]))
                .to_string()
                .contains("scope_denied")
        );
    }
    #[test]
    fn scoped_agent_proposes_applies_and_reads_the_same_receipt() {
        use crate::desktop_fill::owned as fixture;
        let invoke = |owner: &str, name: &str, args: Value| {
            let envelope = smol::block_on(call_with(
                fixture::api(),
                name,
                args,
                Some(owner.into()),
                &[SCOPE.into()],
            ));
            serde_json::from_str::<Value>(envelope["content"][0]["text"].as_str().unwrap()).unwrap()
        };
        let p = invoke(
            "agent-a",
            PROPOSE,
            json!({"app":fixture::APP,"pid":fixture::PID,"nativeWindowId":fixture::WINDOW,"filename":"Agent outline"}),
        );
        assert_eq!(p["state"], "proposed");
        let h = json!({"proposalId":p["proposal"]["proposalId"]});
        assert_eq!(
            invoke("agent-b", APPLY, h.clone())["error"]["code"],
            "unknown_or_expired_proposal"
        );
        assert_eq!(
            invoke("agent-a", STATUS, h.clone())["result"]["state"],
            "proposed"
        );
        assert_eq!(
            invoke("agent-a", APPLY, h.clone())["result"]["state"],
            "verified"
        );
        assert_eq!(invoke("agent-a", STATUS, h)["result"]["state"], "verified");
    }
    #[test]
    fn no_command_or_value_can_be_smuggled_into_apply() {
        let out = smol::block_on(call(
            APPLY,
            json!({"proposalId":Uuid::new_v4(),"argv":["click","Save"]}),
            Some("mcp:a".into()),
            &[SCOPE.into()],
        ));
        assert!(out.to_string().contains("invalid_arguments"));
    }
}

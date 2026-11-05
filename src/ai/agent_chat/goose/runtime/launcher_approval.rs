//! Review a dedicated writer's exact launcher change, not an engine-generated tool title.
use super::*;
use crate::ai::agent_chat::ui::{
    AgentChatApprovalOption, AgentChatApprovalPreview, AgentChatApprovalPreviewKind,
    AgentChatApprovalRequest,
};
use crate::mcp_agent_tools::LauncherReviewTurn;
use sk_personal::api::Endpoint;
use std::sync::Weak;

pub(super) fn is_write(policy: AgentChatToolPolicy, params: &Value) -> bool {
    policy == AgentChatToolPolicy::LauncherAuthor(Endpoint::Apply)
        && params
            .get("toolCall")
            .and_then(super::super::events::tool_name)
            .as_deref()
            == Some(Endpoint::Apply.profile_tool())
}

pub(super) fn once(params: &Value, kind: &str) -> Option<String> {
    let options = params["options"].as_array()?;
    let mut wanted = options.iter().filter(|o| o["kind"].as_str() == Some(kind));
    let id = wanted.next()?["optionId"].as_str()?;
    if id.is_empty()
        || wanted.next().is_some()
        || options
            .iter()
            .filter(|o| o["optionId"].as_str() == Some(id))
            .count()
            != 1
    {
        return None;
    }
    Some(id.to_owned())
}
fn question(
    root: &std::path::Path,
    params: &Value,
    tx: async_channel::Sender<Option<String>>,
) -> Option<(AgentChatApprovalRequest, String, String)> {
    let save = once(params, "allow_once")?;
    let decline = once(params, "reject_once")?;
    let call = &params["toolCall"];
    let id = call["toolCallId"].as_str().filter(|id| !id.is_empty())?;
    let review = sk_personal::api::review_apply(
        root,
        call.get("rawInput")?.clone(),
        sk_personal::api::safe_mode(),
    )
    .ok()?;
    let mut req = approval_request(params, tx);
    req.title = review.title.into();
    req.preview = Some(
        AgentChatApprovalPreview::new(review.title, id)
            .with_kind(AgentChatApprovalPreviewKind::LauncherSave)
            .with_summary(Some(review.detail)),
    );
    req.options = vec![
        AgentChatApprovalOption {
            option_id: save.clone(),
            name: "Save once".into(),
            kind: "AllowOnce".into(),
        },
        AgentChatApprovalOption {
            option_id: decline.clone(),
            name: "Don’t save".into(),
            kind: "RejectOnce".into(),
        },
    ];
    Some((req, save, decline))
}

pub(super) async fn ask(
    id: Value,
    params: Value,
    event_tx: AgentChatEventTx,
    out_tx: mpsc::UnboundedSender<Value>,
    turn: Weak<LauncherReviewTurn>,
) {
    let choice = decision(&crate::setup::get_kit_path(), &params, event_tx, turn).await;
    let _ = out_tx.send(response(&id, selected_or_cancelled(choice)));
}

async fn decision(
    root: &std::path::Path,
    params: &Value,
    event_tx: AgentChatEventTx,
    turn: Weak<LauncherReviewTurn>,
) -> Option<String> {
    let (tx, rx) = async_channel::bounded(1);
    let (request, save, decline) = question(root, params, tx)?;
    let id = request.id;
    if !turn.upgrade()?.present(id) {
        return None;
    }
    let choice = wait_for_permission(request, rx, event_tx).await;
    let approved = choice.as_deref() == Some(save.as_str());
    if !turn
        .upgrade()?
        .resolve(id, approved.then(|| params["toolCall"]["rawInput"].clone()))
    {
        return None;
    }
    choice.filter(|id| id == &save || id == &decline)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refusing_the_displayed_save_sends_no_write_to_the_store() {
        use crate::mcp_agent_tools::{
            handle_rpc, register_bound_policy, start_launcher_review, unregister_policy,
            AgentRpcReply, AgentToolPolicy,
        };
        let tmp = tempfile::tempdir().unwrap();
        let store = sk_personal::Store::open(tmp.path()).unwrap();
        let before = store.inspect().unwrap();
        let uid = uuid::Uuid::new_v4();
        let args = json!({"op":"apply","request":{"workspace":before.workspace,"expectedRevision":before.revision,"requestId":uid,"change":{"op":"edit","edit":{"target":"launcher.header.pinned-label.weight","value":"semiboldWhileDictating"}}}});
        let params = json!({"toolCall":{"toolCallId":"two","rawInput":args},"options":[{"kind":"allow_once","optionId":"yes"},{"kind":"reject_once","optionId":"no"}]});
        let policy = register_bound_policy(
            AgentToolPolicy {
                allowed_tools: vec!["launcher_read".into(), "launcher_apply".into()],
                cwd: tmp.path().into(),
                read_roots: None,
                write_roots: None,
                web_search_budget: None,
            },
            true,
        );
        let turn = start_launcher_review(&policy).unwrap();
        let (tx, rx) = async_channel::bounded(1);
        let weak = Arc::downgrade(&turn);
        let dir = tmp.path().to_owned();
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
            .block_on(async {
                let job = tokio::spawn(async move { decision(&dir, &params, tx, weak).await });
                let AgentChatEvent::ApprovalRequested(req) =
                    tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
                        .await
                        .unwrap()
                        .unwrap()
                else {
                    panic!("expected the change before it can be saved")
                };
                assert!(req
                    .preview
                    .as_ref()
                    .unwrap()
                    .summary
                    .as_ref()
                    .unwrap()
                    .contains("only while dictating"));
                let no = req
                    .options
                    .iter()
                    .find(|choice| choice.is_reject())
                    .unwrap()
                    .option_id
                    .clone();
                req.reply_tx.send(Some(no)).await.unwrap();
                assert_eq!(job.await.unwrap().as_deref(), Some("no"));
            });
        // Even a sidecar that ignores the rejection and calls MCP cannot save these bytes.
        let AgentRpcReply::Json(reply)=handle_rpc(Some(&policy),&json!({"id":2,"method":"tools/call","params":{"name":"launcher_apply","arguments":args}}).to_string(),None) else {panic!("missing refusal")};
        assert!(reply.contains("launcher_review_required"));
        assert_eq!(store.inspect().unwrap(), before);
        assert_eq!(store.status(uid).unwrap()["status"], "unknown");
        assert!(!tmp.path().join("db").exists());
        unregister_policy(&policy);
    }
    #[test]
    fn an_engine_title_cannot_hide_the_proposed_condition_or_offer_standing_consent() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = sk_personal::Store::open(tmp.path())
            .unwrap()
            .inspect()
            .unwrap()
            .workspace;
        let mut p = json!({"toolCall":{"toolCallId":"one","title":"Only change this while recording","rawInput":{"op":"apply","request":{"requestId":uuid::Uuid::new_v4(),"workspace":workspace,"expectedRevision":"stock","change":{"op":"edit","edit":{"target":"launcher.header.pinned-label.weight","value":"semibold"}}}}},"options":[{"kind":"allow_always","optionId":"forever"},{"kind":"allow_once","optionId":"once"},{"kind":"reject_once","optionId":"no"}]});
        let build = |p: &Value| question(tmp.path(), p, async_channel::bounded(1).0);
        let (q, yes, no) = build(&p).unwrap();
        assert_eq!((yes.as_str(), no.as_str()), ("once", "no"));
        assert_eq!(q.options.len(), 2);
        assert_ne!(q.title, "Only change this while recording");
        assert!(q
            .preview
            .unwrap()
            .summary
            .unwrap()
            .contains("even when not dictating"));
        p["toolCall"]["rawInput"]["request"]["change"]["edit"]["value"] =
            json!("semiboldWhileDictating");
        assert!(build(&p)
            .unwrap()
            .0
            .preview
            .unwrap()
            .summary
            .unwrap()
            .contains("only while dictating"));
        p["options"][0]["optionId"] = json!("once");
        assert!(build(&p).is_none());
        assert_eq!(permission_outcome(&p, false)["outcome"]["optionId"], "no");
        let only_lasting = json!({"options":[{"kind":"allow_always","optionId":"forever"}]});
        assert_eq!(
            permission_outcome(&only_lasting, true)["outcome"]["outcome"],
            "cancelled"
        );
        assert!(!tmp.path().join("db").exists());
    }
}

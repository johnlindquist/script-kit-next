//! Both authenticated transports use the same narrowly typed owner. No GUI operation,
//! generic mutation authority or user-supplied path is accepted here.
use crate::mcp_kit_tools::McpKitRuntimeBridge;
use serde_json::{json, Value};
use sk_personal::{api::Endpoint, Author, Error};

pub(super) fn permitted(endpoint: Endpoint, scopes: &[String], principal: Option<&str>) -> bool {
    principal.is_some_and(|p| !p.is_empty()) && scopes.iter().any(|s| s == endpoint.scope())
}
pub(super) fn definitions(scopes: &[String], principal: Option<&str>) -> Vec<Value> {
    [Endpoint::Read,Endpoint::Apply].into_iter().filter(|e|permitted(*e,scopes,principal)).map(|e|json!({"name":e.mcp_tool(),"description":e.description(),"inputSchema":e.schema(),"annotations":{"readOnlyHint":e==Endpoint::Read,"openWorldHint":false}})).collect()
}
fn live(state: Option<crate::mcp_resources::AppStateResource>) -> Value {
    match state.and_then(|s| s.launcher) {
        Some(adoption) => {
            json!({"scope":"runningWindow","available":true,"adoption":adoption})
        }
        None => json!({"scope":"runningWindow","available":false}),
    }
}
pub(crate) fn call(
    endpoint: Endpoint,
    args: Value,
    author: Author,
    runtime: Option<&(dyn McpKitRuntimeBridge + Send + Sync)>,
) -> Result<Value, &'static str> {
    if endpoint == Endpoint::Read && args == json!({"op":"live"}) {
        return Ok(json!({"ok":true,"data":live(runtime.and_then(|r| r.state()))}));
    }
    let root = crate::setup::get_kit_path();
    if let Some(policy) = crate::runtime_policy::owned_evaluation() {
        policy
            .require_owned_path(&root)
            .map_err(|_| Error::WrongWorkspace.code())?;
    }
    let stored = endpoint
        .stored(&root, args, sk_personal::api::safe_mode(), author)
        .map_err(Error::code)?;
    Ok(json!({"ok":true,"data":stored}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_named_credential_only_discovers_its_exact_operations() {
        let r = vec!["launcher:read".into()];
        let w = vec!["launcher:apply".into()];
        assert_eq!(definitions(&r, Some("one"))[0]["name"], "kit/launcher_read");
        assert_eq!(
            definitions(&w, Some("two"))[0]["name"],
            "kit/launcher_apply"
        );
        assert!(!permitted(Endpoint::Apply, &r, Some("one")));
        assert!(definitions(&w, None).is_empty());
        assert!(definitions(
            &[
                "*".into(),
                "app".into(),
                "mcp:*".into(),
                "notes:quiet".into()
            ],
            Some("one")
        )
        .is_empty());
        let not_running = call(
            Endpoint::Read,
            json!({"op":"live"}),
            Author::ConnectedAgent,
            None,
        )
        .unwrap();
        assert_eq!(
            not_running,
            json!({"ok":true,"data":{"scope":"runningWindow","available":false}})
        );
    }
    #[test]
    fn a_launcher_read_never_returns_the_windows_private_words() {
        let state = crate::mcp_resources::AppStateResource {
            filter_text: Some("private search query".into()),
            launcher: Some(sk_personal::api::Adoption {
                workspace: Some("specific workspace".into()),
                applied_revision: Some("stock".into()),
                effective: Default::default(),
                active_view_ids: Vec::new(),
                error: None,
                pending_write: false,
            }),
            ..Default::default()
        };
        assert!(serde_json::to_value(&state)
            .unwrap()
            .get("launcher")
            .is_none());
        let exposed = live(Some(state));
        assert_eq!(exposed["adoption"]["appliedRevision"], "stock");
        assert!(!exposed.to_string().contains("private search query"));
    }
    #[test]
    fn an_authored_list_is_reversible_data_and_cannot_grant_its_own_source() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let who = Author::ConfiguredAgent;
        let read = |q| Endpoint::Read.stored(root, q, false, who);
        let apply = |q| Endpoint::Apply.stored(root, q, false, who);
        let v = json!({"id":"reading","title":"Reading list","limit":12,"list":{"source":"kit.noteTitles","filter":"pinned","sort":"title","detail":"lastChanged","enter":"openNote"}});
        let pane = json!({"id":"attention","title":"Needs me","limit":6,"list":{"source":"herdr.paneMetadata","filter":"needsAttention","sort":"attentionFirst","detail":"space","enter":"focusAndZoom"}});
        let before = read(json!({"op":"inspect"})).unwrap()["state"].clone();
        let mut read_only = pane.clone();
        read_only["list"]["enter"] = json!("none");
        let typed: sk_personal::views::ViewRecipe =
            serde_json::from_value(read_only.clone()).unwrap();
        assert_eq!(typed.requirements(), &["herdr.panes.metadata"]);
        for view in [&v, &pane, &read_only] {
            assert_eq!(
                read(json!({"op":"preview","change":{"op":"putView","view":view}})).unwrap()
                    ["wouldStore"]["views"][0],
                *view
            );
        }
        let mut wrong = v.clone();
        wrong["list"]["enter"] = json!("focusAndZoom");
        assert!(read(json!({"op":"preview","change":{"op":"putView","view":wrong}})).is_err());
        let mut privileged = pane.clone();
        privileged["grant"] = json!("herdr.panes.zoom");
        assert!(read(json!({"op":"preview","change":{"op":"putView","view":privileged}})).is_err());
        assert!(!root.join("plugins").exists() && !root.join("db").exists());
        let id = uuid::Uuid::new_v4();
        let edit = json!({"op":"apply","request":{"workspace":before["workspace"],"expectedRevision":"stock","requestId":id,"change":{"op":"putView","view":v}}});
        assert!(read(edit.clone()).is_err());
        let saved = apply(edit.clone()).unwrap()["result"]["state"].clone();
        assert_eq!(apply(edit).unwrap()["result"]["replayed"], true);
        assert_eq!(
            read(json!({"op":"why","target":"personal.view.reading"})).unwrap()["origin"],
            "configuredAgent"
        );
        let undone=apply(json!({"op":"apply","request":{"workspace":before["workspace"],"expectedRevision":saved["revision"],"requestId":uuid::Uuid::new_v4(),"change":{"op":"undo","of":id}}})).unwrap();
        assert!(undone["result"]["state"].get("views").is_none());
        assert_eq!(saved["effective"], before["effective"]);
    }
    #[test]
    fn valid_edits_cannot_cross_the_read_route_or_spoof_a_different_author() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let who = Author::ConfiguredAgent;
        let before = Endpoint::Read
            .stored(root, json!({"op":"inspect"}), false, who)
            .unwrap()["state"]
            .clone();
        let request = uuid::Uuid::new_v4();
        let target = "launcher.header.pinned-label.textRole";
        let edit = json!({"op":"apply","request":{"workspace":before["workspace"],"expectedRevision":before["revision"],"requestId":request,"change":{"op":"edit","edit":{"target":target,"value":"primary"}}}});
        assert_eq!(
            Endpoint::Read.stored(root, edit.clone(), false, who),
            Err(Error::InvalidRequest)
        );
        assert_eq!(
            Endpoint::Read.stored(root, json!({"op":"inspect","path":"elsewhere"}), false, who),
            Err(Error::InvalidRequest)
        );
        assert_eq!(
            Endpoint::Apply.stored(root, edit.clone(), true, who),
            Err(Error::SafeMode)
        );
        assert!(!root.join("db").exists());
        let committed = Endpoint::Apply
            .stored(root, edit.clone(), false, who)
            .unwrap();
        assert_eq!(
            committed["result"]["state"]["effective"]["pinnedText"],
            "primary"
        );
        assert_eq!(
            Endpoint::Apply.stored(root, edit, false, Author::ConnectedAgent),
            Err(Error::RequestConflict)
        );
        let why = Endpoint::Read
            .stored(
                root,
                json!({"op":"why","target":target}),
                false,
                Author::ConnectedAgent,
            )
            .unwrap();
        assert_eq!(why["origin"], "configuredAgent");
        assert_eq!(why["requestId"], json!(request));
    }
}

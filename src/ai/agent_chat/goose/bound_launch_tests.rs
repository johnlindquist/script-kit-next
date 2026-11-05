use super::*;
use crate::ai::agent_chat::profiles::{built_in_text_profile, AgentChatProfileContext};
use serde_json::json;

// Resolve just the public profile/recipe inputs, without a keychain, live config, or model.
pub(crate) fn local_launch(grants: &[&str]) -> GooseProfileLaunch {
    let ctx = AgentChatProfileContext {
        kit_path: "/fixture/workspace".into(),
    };
    let mut profile = built_in_text_profile(&ctx);
    profile.id = "my-profile".into();
    profile.name = "My profile".into();
    profile.tools = Some(grants.iter().map(|s| s.to_string()).collect());
    profile.tool_policy = None;
    profile.path_policy = None;
    let tools = recipe::recipe_tools(&recipe::profile_tool_ids(&profile).unwrap());
    let mut spec = GooseLaunchSpec::new("/fixture/goose", "/fixture/workspace");
    let provider = profile.provider.clone();
    let model = profile.model.clone();
    spec.recipe = recipe::build_recipe(recipe::RecipeInput {
        profile: &profile,
        tools: &tools,
        provider: provider.as_deref(),
        model: model.as_deref(),
        endpoint: None,
        user_mcp: &[],
    });
    GooseProfileLaunch {
        spec,
        profile,
        provider,
        model,
        tools,
    }
}

#[test]
fn warm_key_keeps_the_cwd_recipe_and_provider_credential_contract() {
    let mut launch = local_launch(&["read"]);
    let key =
        |l: &GooseProfileLaunch| BoundLaunch::from_snapshot(l, Default::default(), None, &[]).key();
    let original = key(&launch);
    launch
        .spec
        .env
        .insert("OPENAI_API_KEY".into(), "fixture-not-a-real-key".into());
    assert_eq!(key(&launch), original);
    let mut moved = launch.clone();
    moved.spec.cwd = "/another/place".into();
    assert_ne!(key(&moved), original);
    launch.profile.append_system_prompt = Some("A different instruction".into());
    assert_ne!(key(&launch), original);
}
#[test]
fn a_dedicated_author_needs_an_endpoint_and_disables_ambient_engine_hints() {
    use sk_personal::api::Endpoint::{Apply, Read};
    for (grants, expected) in [
        (&["launcher_read"][..], Some(Read)),
        (&["launcher_apply", "launcher_read"][..], Some(Apply)),
        (&["bash", "launcher_read"][..], None),
        (&["launcher_apply"][..], None),
    ] {
        let launch = local_launch(grants);
        let bound = BoundLaunch::from_snapshot(&launch, Default::default(), None, &[]);
        assert_eq!(bound.spec.launcher_author, expected);
        let unrestricted =
            BoundLaunch::from_snapshot(&launch, AgentApprovalMode::FullAccess, None, &[]);
        assert_eq!(
            unrestricted.spec.approval_mode,
            if expected == Some(Apply) {
                AgentApprovalMode::AskBeforeChanges
            } else {
                AgentApprovalMode::FullAccess
            }
        );
        if expected.is_some() {
            assert_eq!(
                bound.spec.env.get("CONTEXT_FILE_NAMES").map(String::as_str),
                Some("[]")
            );
            assert!(bound.for_endpoint(None).is_err());
        } else {
            assert!(!bound.spec.env.contains_key("CONTEXT_FILE_NAMES"));
        }
    }
}

#[test]
fn spawned_endpoint_uses_only_the_bound_policy_and_refuses_a_replaced_listener() {
    let scratch = tempfile::tempdir().unwrap();
    let mut launch = local_launch(&["read"]);
    launch.spec.env.insert(
        "GOOSE_PATH_ROOT".into(),
        scratch.path().join("goose").to_str().unwrap().into(),
    );
    let epoch = Uuid::from_u128(99);
    let endpoint = |generation| mcp_agent_tools::AgentToolsEndpoint {
        uri: "http://127.0.0.1:1/fixture".into(),
        token: "fake-session-token".into(),
        generation,
    };
    let bound = BoundLaunch::from_snapshot(&launch, Default::default(), Some(epoch), &[]);
    assert!(bound
        .for_endpoint(Some(endpoint(Uuid::from_u128(100))))
        .is_err());
    assert!(!scratch.path().join("goose").exists());
    let ready = bound.for_endpoint(Some(endpoint(epoch))).unwrap();
    let id = ready.agent_policy_id.as_deref().unwrap();
    assert_eq!(
        ready.recipe["extensions"][0]["headers"][recipe::AGENT_POLICY_HEADER],
        id
    );
    let rpc = |message: Value| {
        let mcp_agent_tools::AgentRpcReply::Json(text) =
            mcp_agent_tools::handle_rpc(Some(id), &message.to_string(), None)
        else {
            panic!("expected response")
        };
        serde_json::from_str::<Value>(&text).unwrap()
    };
    let list = rpc(json!({"id":1,"method":"tools/list"}));
    let tools = list["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "read");
    let path = scratch.path().join("should-not-exist");
    let refused = rpc(
        json!({"id":2,"method":"tools/call","params":{"name":"write","arguments":{"path":path,"content":"denied"}}}),
    );
    mcp_agent_tools::unregister_policy(id);
    assert_eq!(refused["result"]["isError"], true);
    assert!(!path.exists());
}

#[test]
fn bound_authority_includes_path_restrictions_approval_connected_servers_and_listener_identity() {
    let mut launch = local_launch(&["read", "write", "mcp"]);
    let epoch = Some(Uuid::from_u128(123));
    let key =
        |l: &GooseProfileLaunch, m, e, s: &[Value]| BoundLaunch::from_snapshot(l, m, e, s).key();
    let mode = AgentApprovalMode::AskBeforeChanges;
    let base = key(&launch, mode, epoch, &[]);
    assert_ne!(
        base,
        key(&launch, AgentApprovalMode::AcceptEdits, epoch, &[])
    );
    assert_ne!(base, key(&launch, mode, None, &[]));
    assert_ne!(base, key(&launch, mode, Some(Uuid::from_u128(124)), &[]));
    let server = json!({"type":"streamable_http","name":"work","uri":"http://localhost/one","headers":{"Authorization":"synthetic-credential"}});
    let with = key(&launch, mode, epoch, std::slice::from_ref(&server));
    assert_ne!(base, with);
    assert!(!with.contains("synthetic"));
    let mut replacement = server;
    replacement["uri"] = json!("http://localhost/two");
    assert_ne!(with, key(&launch, mode, epoch, &[replacement]));
    launch.profile.path_policy = Some(crate::config::AgentChatPathPolicyConfig {
        allow_read: None,
        allow_write: Some(vec![]),
        deny: None,
    });
    let no_writes = key(&launch, mode, epoch, &[]);
    assert_ne!(base, no_writes);
    launch.profile.path_policy.as_mut().unwrap().allow_read =
        Some(vec!["/fixture/projects".into()]);
    assert_ne!(no_writes, key(&launch, mode, epoch, &[]));
}

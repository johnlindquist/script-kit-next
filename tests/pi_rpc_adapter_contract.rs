use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {}", path, error))
}

#[test]
fn pi_rpc_module_is_declared_under_agent_chat_only() {
    let agent_chat = read("src/ai/agent_chat/mod.rs");
    let agent_chat = read("src/ai/agent_chat/ui/mod.rs");

    assert!(agent_chat.contains("pub mod pi;"));
    assert!(!agent_chat.contains("PiRpcRuntime"));
}

#[test]
fn pi_rpc_runtime_implements_neutral_connection_seam() {
    let source = read("src/ai/agent_chat/pi/runtime.rs");

    assert!(source.contains("impl AgentChatConnection for PiRpcRuntime"));
    assert!(source.contains("PiRpcRuntimeCommand::StartTurn"));
    assert!(source.contains("PiRpcRuntimeCommand::PrepareSession"));
    assert!(source.contains("PiRpcRuntimeCommand::CancelTurn"));
}

#[test]
fn pi_rpc_protocol_uses_stdio_json_command_names() {
    let source = read("src/ai/agent_chat/pi/protocol.rs");

    for command in [
        r#""prompt""#,
        r#""abort""#,
        r#""get_available_models""#,
        r#""set_model""#,
    ] {
        assert!(source.contains(command), "missing {}", command);
    }
    assert!(source.contains("encode_json_line"));
}

#[test]
fn deterministic_mock_pi_rpc_shim_matches_runtime_command_surface() {
    let shim = read("scripts/agentic/mock-pi-rpc.js");

    for command in [
        "\"get_available_models\"",
        "\"set_model\"",
        "\"prompt\"",
        "\"abort\"",
        "\"message_update\"",
        "\"agent_end\"",
    ] {
        assert!(
            shim.contains(command),
            "mock Pi RPC shim must support {command}"
        );
    }
    assert!(shim.contains("Bonjour le monde."));
    assert!(shim.contains("messageChars"));
    assert!(!shim.contains("process.stderr.write(message"));
    assert!(!shim.contains("command.message)"));
}

#[test]
fn pi_rpc_event_mapper_targets_current_agent_chat_shaped_aliases() {
    let aliases = read("src/ai/agent_chat/events.rs");
    let mapper = read("src/ai/agent_chat/pi/events.rs");

    assert!(aliases.contains("type AgentChatEvent = crate::ai::agent_chat::ui::AgentChatEvent"));
    assert!(mapper.contains("split_text_delta_for_reveal"));
    assert!(mapper.contains("chunks.concat()"));
    assert!(mapper.contains("AgentChatEvent::AgentMessageDelta"));
    assert!(mapper.contains("AgentChatEvent::ToolCallStarted"));
    assert!(mapper.contains("AgentChatEvent::ModelsAvailable"));
    assert!(mapper.contains("AgentChatEvent::TurnFinished"));
}

#[test]
fn pi_rpc_text_reveal_has_markdown_and_long_word_guards() {
    let mapper = read("src/ai/agent_chat/pi/events.rs");
    let runtime = read("src/ai/agent_chat/pi/runtime.rs");

    assert!(mapper.contains("REVEAL_MAX_UNBROKEN_CHARS"));
    assert!(mapper.contains("is_markdown_fence_line"));
    assert!(mapper.contains("is_markdown_table_line"));
    assert!(mapper.contains("markdown_structural_prefix_len"));
    assert!(mapper.contains("chunks.concat()"));
    assert!(runtime.contains("PI_REVEAL_CHUNK_DELAY_MS"));
    assert!(runtime.contains("reveal_count"));
}

#[test]
fn pi_rpc_adapter_is_routed_through_agent_chat_launch_helper() {
    let launch = read("src/ai/agent_chat/launch.rs");
    let tab_launch = read("src/app_impl/tab_ai_mode/agent_chat_launch.rs");
    let tab_mode = read("src/app_impl/tab_ai_mode/mod.rs");
    let setup = read("src/app_impl/tab_ai_mode/agent_chat_setup.rs");

    assert!(launch.contains("PiRpcRuntime::spawn"));
    assert!(launch.contains("PiRpcLaunchSpec::new"));
    assert!(tab_launch.contains("resolve_effective_profile"));
    assert!(tab_launch.contains("PiAgentChatLaunch::from_profile"));
    assert!(tab_launch.contains("open_tab_ai_pi_view_from_launch"));
    assert!(tab_mode.contains("dismiss_active_agent_chat_warm_lease"));
    assert!(
        !setup.contains("PiRpcRuntime") && !setup.contains("AgentChatBackend::Pi"),
        "setup cards should stay Agent Chat setup-owned; Pi routing belongs in launch"
    );
}

#[test]
fn pi_rpc_scaffolding_does_not_hardcode_pi_process_spawn() {
    let runtime = read("src/ai/agent_chat/pi/runtime.rs");
    let protocol = read("src/ai/agent_chat/pi/protocol.rs");

    assert!(runtime.contains("Command::new(&spec.command)"));
    assert!(!runtime.contains("Command::new(\"pi\")"));
    assert!(!protocol.contains("PathBuf::from(\"pi\")"));
}

#[test]
fn pi_rpc_set_model_response_is_gated_before_prompt_dispatch() {
    let runtime = read("src/ai/agent_chat/pi/runtime.rs");

    assert!(runtime.contains("send_set_model_and_wait"));
    assert!(runtime.contains("PendingResponse::Rpc"));
    assert!(runtime.contains("build_set_model_command"));
    assert!(runtime.contains("Pi RPC set_model timed out"));
    assert!(runtime.contains("Invalid Pi model selection"));

    let start_turn = runtime
        .split("PiRpcRuntimeCommand::StartTurn { request, event_tx } =>")
        .nth(1)
        .expect("StartTurn branch must exist")
        .split("PiRpcRuntimeCommand::CancelTurn")
        .next()
        .expect("CancelTurn branch must follow StartTurn");
    let set_model_index = start_turn
        .find("send_set_model_and_wait")
        .expect("StartTurn must await set_model");
    let prompt_index = start_turn
        .find("build_prompt_payload")
        .expect("StartTurn must build the prompt");

    assert!(
        set_model_index < prompt_index,
        "set_model response must be handled before prompt payload dispatch"
    );
    assert!(
        start_turn.contains("continue;"),
        "set_model failures must stop the turn before prompt dispatch"
    );
}

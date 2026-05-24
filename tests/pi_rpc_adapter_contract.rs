use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

#[test]
fn pi_rpc_module_is_declared_under_agent_chat_only() {
    let agent_chat = read("src/ai/agent_chat/mod.rs");
    let acp = read("src/ai/acp/mod.rs");

    assert!(agent_chat.contains("pub mod pi;"));
    assert!(!acp.contains("PiRpcRuntime"));
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
        assert!(source.contains(command), "missing {command}");
    }
    assert!(source.contains("encode_json_line"));
}

#[test]
fn pi_rpc_event_mapper_targets_current_acp_shaped_aliases() {
    let aliases = read("src/ai/agent_chat/events.rs");
    let mapper = read("src/ai/agent_chat/pi/events.rs");

    assert!(aliases.contains("type AgentChatEvent = crate::ai::acp::AcpEvent"));
    assert!(mapper.contains("AgentChatEvent::AgentMessageDelta"));
    assert!(mapper.contains("AgentChatEvent::ToolCallStarted"));
    assert!(mapper.contains("AgentChatEvent::ModelsAvailable"));
    assert!(mapper.contains("AgentChatEvent::TurnFinished"));
}

#[test]
fn pi_rpc_scaffolding_does_not_route_tab_to_pi() {
    for path in [
        "src/app_impl/tab_ai_mode/mod.rs",
        "src/app_impl/tab_ai_mode/acp_launch.rs",
        "src/app_impl/tab_ai_mode/acp_setup.rs",
    ] {
        let source = read(path);
        assert!(
            !source.contains("PiRpcRuntime") && !source.contains("AgentChatBackend::Pi"),
            "{path} must not route Tab Agent Chat to Pi in this adapter-only slice"
        );
    }
}

#[test]
fn pi_rpc_scaffolding_does_not_hardcode_pi_process_spawn() {
    let runtime = read("src/ai/agent_chat/pi/runtime.rs");
    let protocol = read("src/ai/agent_chat/pi/protocol.rs");

    assert!(runtime.contains("Command::new(&spec.command)"));
    assert!(!runtime.contains("Command::new(\"pi\")"));
    assert!(!protocol.contains("PathBuf::from(\"pi\")"));
}

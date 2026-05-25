use std::fs;
use std::path::Path;

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.as_ref().display()))
}

#[test]
fn inline_agent_ui_and_ai_boundary_do_not_own_pi_or_acp_runtime_launch() {
    let checked_files = [
        "src/ai/inline_agent/mod.rs",
        "src/inline_agent/window.rs",
        "src/inline_agent/render_compact.rs",
        "src/inline_agent/render_actions.rs",
        "src/inline_agent/state.rs",
        "src/ai/inline_agent/agent_chat_adapter.rs",
        "src/ai/inline_agent/session.rs",
    ];

    for path in checked_files {
        let source = read(path);
        for forbidden in [
            "PiRpcRuntime",
            "PiLaunchSpec",
            "spawn_default_acp_inline_agent_executor",
            "mod acp_adapter",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} must not directly own provider runtime launch symbol {forbidden}"
            );
        }
    }
}

#[test]
fn agent_chat_launch_module_owns_inline_pi_resolution_and_pi_runtime_stays_in_pi_module() {
    let launch = read("src/ai/agent_chat/launch.rs");
    let pi_runtime = read("src/ai/agent_chat/pi/runtime.rs");
    let adapter = read("src/ai/inline_agent/agent_chat_adapter.rs");

    assert!(launch.contains("resolve_focused_text_pi_launch"));
    assert!(launch.contains("BUILTIN_TEXT_PROFILE_ID"));
    assert!(launch.contains("PiRpcRuntime::spawn"));
    assert!(pi_runtime.contains("struct PiRpcRuntime"));
    assert!(!adapter.contains("PiRpcRuntime::spawn"));
}

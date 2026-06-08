use std::fs;
use std::path::{Path, PathBuf};

fn collect_files(root: impl AsRef<Path>, files: &mut Vec<PathBuf>) {
    let root = root.as_ref();
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_files(path, files);
        } else {
            files.push(path);
        }
    }
}

#[test]
fn legacy_agent_chat_subprocess_backend_files_are_removed() {
    for path in [
        "src/ai/agent_chat/ui/agy_adapter.rs",
        "src/ai/agent_chat/ui/client.rs",
        "src/ai/agent_chat/ui/handlers.rs",
        "src/ai/agent_chat/ui/provider.rs",
        "tests/agy_agent_chat_adapter_harness.rs",
        "scripts/agentic/mock-agent_chat-agent.js",
        "scripts/agentic/tx_wait_for_agent_chat_runtime_semantics.ts",
    ] {
        assert!(
            !Path::new(path).exists(),
            "legacy Agent Chat subprocess backend file should be removed: {path}"
        );
    }
}

#[test]
fn source_tree_does_not_reference_legacy_agent_chat_backend_symbols() {
    let mut files = vec![PathBuf::from("Cargo.toml")];
    for root in ["src", "scripts/agentic", "tests"] {
        collect_files(root, &mut files);
    }

    let this_file = Path::new("tests/source_audits/agent_chat_no_legacy_backend.rs");
    let forbidden = [
        "AgentChatRuntime",
        "AgentChatConnection",
        "AgentChatProvider",
        "AgyAgentChatAgent",
        "AGY_AGENT_CHAT",
        "--agy-agent_chat-adapter",
        "AgentChatCommand",
        "AgentChatCancelCommand",
        "AgentChatPromptTurnRequest",
        "StreamPrompt",
        "stream_prompt",
        "AgentSideConnection",
        "ClientSideConnection",
        "mock-agent_chat-agent",
        "agy_agent_chat_adapter_harness",
    ];

    for path in files {
        if path == this_file {
            continue;
        }
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if !matches!(extension, "rs" | "ts" | "js" | "toml") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for symbol in forbidden {
            assert!(
                !source.contains(symbol),
                "{} must not reference legacy Agent Chat subprocess backend symbol {symbol}",
                path.display()
            );
        }
    }
}

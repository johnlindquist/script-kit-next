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
fn legacy_acp_subprocess_backend_files_are_removed() {
    for path in [
        "src/ai/acp/agy_adapter.rs",
        "src/ai/acp/client.rs",
        "src/ai/acp/handlers.rs",
        "src/ai/acp/provider.rs",
        "tests/agy_acp_adapter_harness.rs",
        "scripts/agentic/mock-acp-agent.js",
        "scripts/agentic/tx_wait_for_acp_runtime_semantics.ts",
    ] {
        assert!(
            !Path::new(path).exists(),
            "legacy ACP subprocess backend file should be removed: {path}"
        );
    }
}

#[test]
fn source_tree_does_not_reference_legacy_acp_backend_symbols() {
    let mut files = vec![PathBuf::from("Cargo.toml")];
    for root in ["src", "scripts/agentic", "tests"] {
        collect_files(root, &mut files);
    }

    let this_file = Path::new("tests/source_audits/acp_no_legacy_backend.rs");
    let forbidden = [
        "AcpRuntime",
        "AcpConnection",
        "AcpProvider",
        "AgyAcpAgent",
        "AGY_ACP",
        "--agy-acp-adapter",
        "AcpCommand",
        "AcpCancelCommand",
        "AcpPromptTurnRequest",
        "StreamPrompt",
        "stream_prompt",
        "AgentSideConnection",
        "ClientSideConnection",
        "mock-acp-agent",
        "agy_acp_adapter_harness",
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
                "{} must not reference legacy ACP subprocess backend symbol {symbol}",
                path.display()
            );
        }
    }
}

use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(manifest_dir).join(path);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn extract_section<'a>(src: &'a str, needle: &str, len: usize) -> &'a str {
    let start = src
        .find(needle)
        .unwrap_or_else(|| panic!("missing section marker: {needle}"));
    let end = (start + len).min(src.len());
    &src[start..end]
}

#[test]
fn sdk_exposes_typed_computer_use_global() {
    let sdk = repo_file("scripts/kit-sdk.ts");

    for needle in [
        "export interface ComputerUseApi",
        "var computer: ComputerUseApi",
        "globalThis.computer =",
        "listNativeWindows(",
        "captureNativeWindow(",
        "ComputerUseCaptureNativeWindowResult",
        "ComputerUseListNativeWindowsResult",
    ] {
        assert!(
            sdk.contains(needle),
            "SDK computer-use surface missing {needle}"
        );
    }
}

#[test]
fn sdk_computer_use_calls_script_kit_self_mcp_server() {
    let sdk = repo_file("scripts/kit-sdk.ts");
    let discovery_section = extract_section(
        &sdk,
        "async function loadScriptKitSelfMcpServerConfig",
        1600,
    );

    assert!(
        discovery_section.contains("'.scriptkit', 'server.json'"),
        "computer helpers must discover the live Script Kit MCP server from ~/.scriptkit/server.json"
    );
    assert!(
        discovery_section.contains("endpoint: `${baseUrl}/rpc`"),
        "computer helpers must call the app's /rpc endpoint"
    );
    assert!(
        discovery_section.contains("authorization: `Bearer ${discovery.token}`"),
        "computer helpers must use the server discovery bearer token"
    );

    let call_section = extract_section(&sdk, "async function callScriptKitComputerTool", 1000);
    assert!(
        call_section.contains("withMcpSession('__scriptkit_self__'"),
        "computer helpers must use the existing JSON-RPC/MCP client session path"
    );
    assert!(
        call_section.contains("session.request('tools/call'"),
        "computer helpers must route through tools/call instead of adding stdin verbs"
    );
}

#[test]
fn sdk_computer_use_surface_stays_observation_and_capture_only() {
    let sdk = repo_file("scripts/kit-sdk.ts");
    let section = extract_section(&sdk, "globalThis.computer =", 1200);

    assert!(section.contains("'computer/list_native_windows'"));
    assert!(section.contains("'computer/capture_native_window'"));

    for forbidden in [
        "click",
        "type",
        "focus",
        "activate",
        "moveWindow",
        "resizeWindow",
        "sendInput",
    ] {
        assert!(
            !section.contains(forbidden),
            "computer SDK pass must stay observation/capture-only; found {forbidden}"
        );
    }
}

#[test]
fn sdk_reference_lists_computer_use_helpers() {
    let resources = repo_file("src/mcp_resources/mod.rs");

    for needle in [
        "\"computer.listNativeWindows\"",
        "\"await computer.listNativeWindows(options?: ComputerUseListNativeWindowsOptions): Promise<ComputerUseListNativeWindowsResult>\"",
        "\"computer.captureNativeWindow\"",
        "\"await computer.captureNativeWindow(options: ComputerUseCaptureNativeWindowOptions): Promise<ComputerUseCaptureNativeWindowResult>\"",
        "\"computer-use\"",
    ] {
        assert!(
            resources.contains(needle),
            "SDK reference must publish computer-use helper {needle}"
        );
    }
}

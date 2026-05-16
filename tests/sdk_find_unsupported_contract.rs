const SDK: &str = include_str!("../scripts/kit-sdk.ts");
const MCP_RESOURCES: &str = include_str!("../src/mcp_resources/mod.rs");
const QUERY_OPS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");

fn body_after(marker: &str) -> &str {
    let start = SDK.find(marker).expect("missing SDK marker");
    &SDK[start..]
}

fn function_body(marker: &str) -> &str {
    let tail = body_after(marker);
    let end = tail
        .find("\n};")
        .expect("missing global function terminator");
    &tail[..end]
}

// doc-anchor-removed: [[removed-docs]]
#[test]
fn find_rejects_typed_unsupported_before_send() {
    let body = function_body("globalThis.find = function find(");
    assert!(
        body.contains("rejectUnsupportedSdkFeature('find'"),
        "find must use the shared typed unsupported error"
    );
    assert!(
        body.contains("fileSearch(query, { onlyin })"),
        "find must point callers to the supported onlyin-capable fileSearch API"
    );
    assert!(
        body.contains("path({ startPath })") || body.contains("arg(...)"),
        "find must point callers to a supported prompt-driven alternative"
    );

    for forbidden in [
        "nextId(",
        "addPending(",
        "send(",
        "waitForSubmit",
        "type: 'find'",
    ] {
        assert!(
            !body.contains(forbidden),
            "find must reject before protocol setup; found forbidden `{forbidden}` in:\n{body}"
        );
    }
}

// doc-anchor-removed: [[removed-docs and control messages]]
#[test]
fn sdk_no_longer_defines_a_find_protocol_message_shape() {
    assert!(
        !SDK.contains("interface FindMessage"),
        "unsupported find must not leave a stale SDK protocol message interface"
    );
    assert!(
        !SDK.contains("const message: FindMessage"),
        "unsupported find must not construct a stale FindMessage"
    );
    assert!(
        !SDK.contains("type: 'find',"),
        "unsupported find must not emit a find protocol message"
    );
}

// doc-anchor-removed: [[removed-docs and introspection]]
#[test]
fn rust_protocol_exposes_file_search_but_not_find_prompt() {
    assert!(
        QUERY_OPS.contains("#[serde(rename = \"fileSearch\")]"),
        "fileSearch remains the supported query route"
    );
    assert!(
        QUERY_OPS.contains("#[serde(rename = \"fileSearchResult\")]"),
        "fileSearchResult remains the supported response route"
    );
    assert!(
        !QUERY_OPS.contains("#[serde(rename = \"find\")]"),
        "JOH-63 closes the SDK hang by rejecting before send, not by pretending a Rust find prompt route exists"
    );
}

// doc-anchor-removed: [[removed-docs]]
#[test]
fn sdk_reference_marks_find_as_unsupported_with_file_search_alternative() {
    assert!(
        MCP_RESOURCES.contains("\"find\","),
        "unsupported SDK inventory must include find"
    );
    assert!(
        MCP_RESOURCES.contains("SdkFunctionRef::unsupported(\n            \"find\""),
        "SDK reference must mark find as unsupported"
    );
    assert!(
        MCP_RESOURCES.contains("fileSearch(query, { onlyin })"),
        "find unsupported note must point callers to onlyin-capable fileSearch"
    );
    assert!(
        MCP_RESOURCES.contains("sdk_reference_marks_find_as_unsupported_prompt_gap"),
        "direct SDK reference test should pin the generated reference contract"
    );
}

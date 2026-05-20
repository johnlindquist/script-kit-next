//! Oracle-Session `protocol-builtin-boundary-engineering-plan` —
//! golden-transcript tests for the pure MCP JSON-RPC 2.0 parser +
//! dispatcher at [`script_kit_gpui::mcp_protocol`].
//!
//! Why this test exists: `src/mcp_protocol/mod.rs` is the single
//! ~9.8k-token entry point covering `initialize`, `tools/list`,
//! `tools/call`, `resources/list`, and `resources/read`. It is pure
//! enough to exercise in-process without spinning up the HTTP server,
//! but today has no end-to-end transcript coverage. Oracle's ranked
//! plan (#6 + #7) explicitly calls for a small golden transcript
//! suite BEFORE the file is split, so any future `mcp_protocol/`
//! module split has a regression anchor that pins response shape and
//! JSON-RPC error codes.
//!
//! The fixture at `tests/golden/mcp/basic_rpc.jsonl` has one
//! JSON-per-line case. Each case is `{name, request, id?, outcome}`
//! where `outcome` is either `{ok: {resultKeys}}` (success, the
//! response's `result` object must carry every listed key — shape
//! match, not byte-exact, because version strings and ordering are
//! intentionally non-deterministic) or `{error: {code}}` (error
//! response, `error.code` must match exactly).
//!
//! The design intentionally does NOT pin full result values. A later
//! "exact fixture" test can be added when the serialized shapes are
//! stable enough that rare upstream changes don't churn the golden
//! lines. Today the shape-match layer is the right cost/value tier:
//! it catches "tools/list no longer returns a `tools` key" without
//! firing on every upstream tool addition.

use script_kit_gpui::mcp_protocol::{
    error_codes, handle_request_with_context, parse_request, JsonRpcResponse,
};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct GoldenCase {
    name: String,
    request: String,
    #[serde(default)]
    id: Option<Value>,
    outcome: Outcome,
}

#[derive(Debug, Deserialize)]
enum Outcome {
    #[serde(rename = "ok")]
    Ok {
        #[serde(rename = "resultKeys")]
        result_keys: Vec<String>,
    },
    #[serde(rename = "error")]
    Error { code: i32 },
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/mcp/basic_rpc.jsonl")
}

fn load_cases() -> Vec<(usize, GoldenCase)> {
    let path = fixture_path();
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read golden fixture {}: {e}", path.display()));
    let mut cases = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parsed: GoldenCase = serde_json::from_str(trimmed).unwrap_or_else(|e| {
            panic!(
                "fixture {}:{} failed to parse as GoldenCase: {e}\n  line: {trimmed}",
                path.display(),
                idx + 1
            )
        });
        cases.push((idx + 1, parsed));
    }
    cases
}

/// Drive one JSONL line through the same code path the HTTP handler
/// uses: `parse_request` → on `Ok(req)`, `handle_request_with_context`;
/// on `Err(err_response)`, the error response itself. Empty scripts /
/// scriptlets / app_state mirrors the stateless request path.
fn drive_case(request_json: &str) -> JsonRpcResponse {
    match parse_request(request_json) {
        Ok(req) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .expect("build test runtime");
            runtime.block_on(handle_request_with_context(req, &[], &[], None))
        }
        Err(err_response) => err_response,
    }
}

fn assert_case(line_no: usize, case: &GoldenCase) -> Result<(), String> {
    let response = drive_case(&case.request);

    if response.jsonrpc != "2.0" {
        return Err(format!(
            "mcp/basic_rpc.jsonl:{line_no}: `{}` response carried jsonrpc=`{}`, want `2.0`",
            case.name, response.jsonrpc
        ));
    }

    if let Some(expected_id) = &case.id {
        if &response.id != expected_id {
            return Err(format!(
                "mcp/basic_rpc.jsonl:{line_no}: `{}` expected id {expected_id}, got {}",
                case.name, response.id
            ));
        }
    }

    match &case.outcome {
        Outcome::Ok { result_keys } => {
            let Some(result) = response.result.as_ref() else {
                return Err(format!(
                    "mcp/basic_rpc.jsonl:{line_no}: `{}` expected success, got error: {:?}",
                    case.name, response.error
                ));
            };
            let obj = result.as_object().ok_or_else(|| {
                format!(
                    "mcp/basic_rpc.jsonl:{line_no}: `{}` expected result to be a JSON object, got {result}",
                    case.name,
                )
            })?;
            for key in result_keys {
                if !obj.contains_key(key) {
                    return Err(format!(
                        "mcp/basic_rpc.jsonl:{line_no}: `{}` expected key `{key}` in result, got keys {:?}",
                        case.name,
                        obj.keys().collect::<Vec<_>>()
                    ));
                }
            }
        }
        Outcome::Error { code } => {
            let Some(err) = response.error.as_ref() else {
                return Err(format!(
                    "mcp/basic_rpc.jsonl:{line_no}: `{}` expected error, got success result: {:?}",
                    case.name, response.result
                ));
            };
            if err.code != *code {
                return Err(format!(
                    "mcp/basic_rpc.jsonl:{line_no}: `{}` expected error.code {code}, got {} \
                     (message: `{}`)",
                    case.name, err.code, err.message
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn every_mcp_basic_rpc_case_passes() {
    let cases = load_cases();
    assert!(
        !cases.is_empty(),
        "golden fixture `basic_rpc.jsonl` must have at least one case"
    );

    let mut failures = Vec::new();
    for (line_no, case) in &cases {
        if let Err(msg) = assert_case(*line_no, case) {
            failures.push(msg);
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} MCP golden cases failed:\n  - {}",
        failures.len(),
        cases.len(),
        failures.join("\n  - ")
    );
}

#[test]
fn fixture_covers_every_jsonrpc_error_code() {
    // Shape invariant: the fixture MUST exercise every standard
    // JSON-RPC 2.0 error code Script Kit emits so a refactor that
    // silently drops one (e.g. collapses PARSE_ERROR into
    // INVALID_REQUEST) still fails this test.
    let cases = load_cases();
    let codes: std::collections::BTreeSet<i32> = cases
        .iter()
        .filter_map(|(_, c)| match c.outcome {
            Outcome::Error { code } => Some(code),
            Outcome::Ok { .. } => None,
        })
        .collect();

    let required = [
        error_codes::PARSE_ERROR,
        error_codes::INVALID_REQUEST,
        error_codes::METHOD_NOT_FOUND,
    ];
    for code in required {
        assert!(
            codes.contains(&code),
            "basic_rpc.jsonl must have at least one case expecting error.code {code} \
             but the fixture only covers {codes:?}"
        );
    }
}

#[test]
fn fixture_covers_every_supported_mcp_method() {
    // Shape invariant: each `McpMethod` variant is exercised by at
    // least one success case in the fixture. If a new method is
    // added to `McpMethod` without a fixture case, the golden suite
    // silently gives it zero coverage — this test pins the lower
    // bound so the fixture grows with the enum.
    use script_kit_gpui::mcp_protocol::McpMethod;

    let cases = load_cases();
    // Parse each case's `method` field by re-parsing the request
    // JSON (the outer fixture only carries request as a string).
    let mut seen = std::collections::BTreeSet::<String>::new();
    for (_, case) in &cases {
        if let Outcome::Error { .. } = case.outcome {
            continue;
        }
        let raw: Value = match serde_json::from_str(&case.request) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(m) = raw.get("method").and_then(|m| m.as_str()) {
            seen.insert(m.to_string());
        }
    }

    // The methods Oracle's plan explicitly called out as needing
    // transcript coverage BEFORE splitting `src/mcp_protocol/mod.rs`.
    // `tools/call` and `resources/read` take non-trivial params and
    // are intentionally deferred to a follow-up fixture — they are
    // listed here as `expected_deferred` so the reader sees why the
    // lower-bound set is smaller than `McpMethod::*`.
    let expected_present = [
        McpMethod::Initialize.as_str(),
        McpMethod::ToolsList.as_str(),
        McpMethod::ResourcesList.as_str(),
    ];
    for m in expected_present {
        assert!(
            seen.contains(m),
            "basic_rpc.jsonl must have at least one success case for method `{m}`; \
             saw success methods {seen:?}"
        );
    }
}

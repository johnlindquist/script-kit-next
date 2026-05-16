# 044 MCP Protocol Runtime Bridge

This chapter maps the runtime bridge between JSONL stdin, protocol envelopes, MCP HTTP JSON-RPC, resources, tools, and SDK MCP helpers.

Raw Oracle reference: [answer](../raw-oracle/044-mcp-protocol-runtime-bridge/answer.md), [prompt](../raw-oracle/044-mcp-protocol-runtime-bridge/prompt.md), [bundle map](../raw-oracle/044-mcp-protocol-runtime-bridge/bundle-map.md), [full log](../raw-oracle/044-mcp-protocol-runtime-bridge/output.log), [session metadata](../raw-oracle/044-mcp-protocol-runtime-bridge/session.json).

## Executive Summary

This feature is the focused runtime bridge pass for the open MCP/protocol gaps from Feature 004.

It maps how agents can drive Script Kit GPUI without screenshots: JSONL stdin commands, typed protocol responses, MCP JSON-RPC over HTTP, MCP resources, MCP tools, diagnostics resources, transaction traces, and TypeScript/Bun SDK helpers.

The core protocol envelope and stdin dispatcher now share the same accepted range. `protocolVersion:2` is accepted by the version reader, ingress observer, and stdin dispatch, while outbound object responses stamp `protocolVersion:2`.

## Scope

This chapter owns the bridge surface where external agents and scripts communicate with the app.

| Surface | Included here | Boundary |
|---|---:|---|
| JSONL stdin protocol | Yes | Command parsing, version observation, responses, batch/wait/state receipts. |
| `protocolVersion` | Yes | Reader, observer, deprecation warnings, outbound stamping, stdin dispatch. |
| MCP HTTP server | Yes | `/rpc`, bearer token, tools/resources methods, lifecycle caveats. |
| MCP resources | Yes | Registry, `resources/list`, `resources/read`, context, diagnostics, transactions. |
| MCP tools | Yes | Kit tools, script-derived tools, computer observation/capture tools. |
| SDK MCP helpers | Yes | HTTP/stdio client helpers, headers, sessions, self computer helpers. |
| Production feature behavior | No | Individual prompt/window/source features own their runtime semantics. |
| Full server hardening implementation | No | This chapter records the current contract and the gap plan. |

## Entry Points

Agents enter the bridge through stdin, MCP, resources, tools, or SDK wrappers.

| Entry point | Transport | Result |
|---|---|---|
| JSONL stdin | One JSON object per line. | Drives prompts, windows, automation queries, waits, batches, and receipts. |
| MCP HTTP server | JSON-RPC 2.0 over `/rpc`. | Lists/calls tools and lists/reads resources for MCP clients. |
| MCP resources | URI reads. | Returns state, catalogs, context, diagnostics, transactions, and reference docs. |
| MCP tools | `tools/list` and `tools/call`. | Calls static kit tools, script tools, and computer observation/capture tools. |
| SDK MCP helpers | TypeScript/Bun globals. | Calls configured local/self/remote MCP servers and normalizes results. |

## Protocol Version Contract

`protocolVersion` is envelope metadata with source support in the reader, observer, and stdin dispatch gate.

| Version behavior | Current contract |
|---|---|
| `CURRENT_PROTOCOL_VERSION` | `2`. |
| `MIN_PROTOCOL_VERSION` | `1`. |
| Missing field | Treated as legacy v1. |
| Valid explicit v2 | Accepted by the version reader and ingress observer. |
| Non-object root | Version observer reports not-object. |
| Non-integer/negative/overflow | Version reader reports invalid type. |
| Too old / too new | Version reader reports unsupported. |
| Outbound object responses | `attach_current_version` stamps or overwrites `protocolVersion:2`. |
| Non-object outbound values | Current-version attachment returns false. |

### Ingress Observer

The observer validates the raw JSONL line before command dispatch and records version/deprecation diagnostics.

Golden coverage proves valid v2 `triggerBuiltin` lines are observed as version 2 without warnings when they use `builtinId`. Legacy or v2 lines that use `triggerBuiltin.name` warn because `name` is deprecated in v2 and scheduled for removal in v3.

### Stdin Dispatch Policy

Explicit stdin v2 dispatch is supported for the typed command layer.

Missing `protocolVersion` is legacy v1. Explicit v1 and v2 envelopes dispatch through the same `ExternalCommand` and protocol `Message` paths after the envelope is stripped. Unsupported future versions hard-reject before dispatch and increment unsupported-version diagnostics. Invalid non-integer envelopes hard-reject as parse failures without consuming the unsupported-version counter.

## MCP HTTP Server

The app-owned MCP server is an HTTP JSON-RPC server, not the SDK client's stdio transport.

| Lifecycle area | Current contract |
|---|---|
| Default endpoint | Local HTTP `/rpc`, default port `43210`. |
| Port override | `MCP_PORT`. |
| Authentication | Bearer token from `~/.scriptkit/agent-token`. |
| Discovery | Writes server metadata to `~/.scriptkit/server.json`. |
| Current methods | `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`. |
| Handle lifetime | Startup retains a `ServerHandle` so the listener remains alive after setup. |
| Socket behavior | Accepted sockets are forced back to blocking mode before request parsing/writes. |

### Server Gaps

Server lifecycle details need a full source pass before being treated as exhaustive.

Open proof targets include exact bind host, discovery schema, token creation/permissions/rotation, wrong path/method behavior, content-type expectations, malformed HTTP/body handling, JSON-RPC notifications/batches, shutdown semantics, and the HTTP status versus JSON-RPC error boundary.

## JSON-RPC Methods

The MCP protocol method names are exact JSON-RPC method strings.

| Method | Purpose |
|---|---|
| `initialize` | Returns server info and capabilities. |
| `tools/list` | Lists kit, computer-use, and script-derived tools. |
| `tools/call` | Calls one tool by name with arguments. |
| `resources/list` | Lists resource definitions from the registry. |
| `resources/read` | Reads one resource URI. |

CamelCase alternatives such as `toolsList` are not registered.

### JSON-RPC Errors

Golden protocol tests pin parser and dispatcher error classes.

| Request class | Expected behavior |
|---|---|
| Malformed JSON | `-32700` parse error. |
| Missing `jsonrpc` | `-32600` invalid request. |
| Wrong JSON-RPC version | `-32600` invalid request. |
| Missing method | `-32600` invalid request. |
| Unknown method | `-32601` method not found. |
| Missing `tools/call` name | invalid params. |
| Missing `resources/read` URI | invalid params. |
| Unknown resource URI | method-not-found style resource error. |

The current golden fixture is a lower bound. It covers initialize, `tools/list`, and `resources/list`; `tools/call` and `resources/read` happy/error rows should be added.

## MCP Resources

Resources are read-only URI surfaces backed by the app state, catalogs, context, diagnostics, and docs.

| Resource class | URIs / examples | Contract |
|---|---|---|
| App state | `kit://state`. | JSON app state resource, simpler than protocol `getState`. |
| Catalogs | `scripts://`, `scriptlets://`, `kit://scripts`, `kit://scriptlets`. | Legacy and schema-versioned script/scriptlet catalogs. |
| SDK reference | `kit://sdk-reference`. | Shared SDK/harness reference. |
| AI context | `kit://context`, `kit://context/schema`. | Context snapshot, query flags, diagnostics, schema. |
| Clipboard/focus | `kit://clipboard-history`, `kit://focused-item`. | Agent context and selected/focused item data. |
| Workspace/system | `kit://git-status`, `kit://git-diff`, `kit://processes`, `kit://system`. | Text or JSON system/workspace readouts. |
| Dictation | `kit://dictation`, `kit://dictation-history`. | Dictation state/history resources. |
| Reference docs | `kit://stdin-commands`, `kit://trigger-builtins`. | Drift-audited markdown references. |
| Diagnostics | `kit://diagnostics/protocol-stats`. | Protocol counters, health flags, and thresholds. |
| Transactions | `kit://transactions/schema`, `kit://transactions/latest`, `kit://transactions/latest?requestId=...`. | Transaction trace schema and latest lookup. |

### Context Resource

`kit://context` is the main AI-facing context resource and has explicit query behavior.

| Query | Behavior |
|---|---|
| No query | Default/full snapshot behavior. |
| `profile=minimal` | Minimal profile. |
| `profile=full` | Full metadata profile. |
| Profile plus field overrides | Effective profile becomes custom. |
| Unknown profile | Invalid params with supported profile names. |
| Unknown parameter | Invalid params with supported parameter names. |
| Boolean flags | Accepts `1`, `0`, `true`, `false`. |
| Invalid boolean | Invalid params. |
| `diagnostics=1` | Wraps snapshot in context diagnostics metadata. |
| `kit://context/schema` | Self-describing schema document. |

### Diagnostics Resources

Protocol diagnostics and transactions let agents verify health without screenshots.

`kit://diagnostics/protocol-stats` exposes counters, thresholds, health flags, and declaration-order-stable flags. Threshold policy is strict for stdin parse failures, too-large commands, and unsupported protocol versions; typo/deprecated builtin counters tolerate more noise.

Transaction resources provide schema and latest traces, optionally filtered by request id. Invalid transaction URI forms should not be treated as transaction resources.

## MCP Tools

Tools are split into static app tools, script-derived tools, and computer observation/capture tools.

| Tool family | Source | Contract |
|---|---|---|
| Kit tools | `src/mcp_kit_tools.rs`. | Static `kit/*` namespace. |
| Script tools | `src/mcp_script_tools/*`. | Generated from scripts with schema metadata. |
| Computer tools | `src/mcp_computer_use_tools.rs`. | Read-only observation and exact native capture tools. |

### Kit Tools

The safe static kit tool set is intentionally small until the full kit-tool source pass is complete.

| Tool | Behavior |
|---|---|
| `kit/show` | Show/focus app surface. |
| `kit/hide` | Hide/reset app surface. |
| `kit/state` | Return app state JSON as tool content. |
| Unknown `kit/*` | Tool-level `isError` result, not JSON-RPC method-not-found. |

### Script Tools

Script-derived tools must come from script metadata and schema, not from every script file.

Tests prove script tool classification for names like `scripts/create-note` and `scripts/git-commit`, while malformed names and non-script namespaces are excluded. The execution and error boundary still needs a deeper source pass through `src/mcp_script_tools/*`.

### Computer Tools

Computer tools are observation/capture tools, not action tools.

They can inspect app/window/menu/screen/permission metadata and capture exact native windows where implemented. They must not be described as click, type, focus, launch, quit, hide, move, resize, menu-press, permission-prompt, or global status-extra action tools unless those actions are explicitly implemented later.

Current computer-use boundaries:

| Tool class | Boundary |
|---|---|
| `computer/see` | Uses the runtime bridge for screenshot/semantic/geometry snapshot. |
| Automation window tools | Registry-based list/get focused/get by stable id. |
| Running app tools | Running GUI app inventory, not installed app catalog. |
| Native window tools | Moment-in-time native inventory; minimized/off-screen windows may be absent. |
| Menu tools | Cached frontmost-app menu tree; no pressing. |
| Tray tools | Script Kit tray menu model; no tray execution. |
| Screen tools | Read-only active display metadata. |
| Permission tools | Read-only preflight checks; no prompts. |
| `computer/capture_native_window` | Exact PID plus native window id capture; no fallback by title/bounds/frontmost. |

## SDK MCP Helpers

The SDK MCP client is separate from the app-owned HTTP server implementation.

The SDK uses MCP protocol version string `2024-11-05` for MCP HTTP/stdio sessions. That string is not the numeric JSONL stdin `protocolVersion:2`.

For HTTP calls, the SDK sends `content-type: application/json`, `accept: application/json`, `mcp-protocol-version: 2024-11-05`, configured server headers, and `mcp-session-id` when a session id exists.

`withMcpSession` chooses stdio or HTTP based on server transport, sends `initialize`, sends `notifications/initialized`, runs the handler, and closes in `finally`. Do not claim the app-owned server supports arbitrary notifications unless the server dispatcher proves it.

The visible SDK MCP surface includes listing configured tools, discovery/scoring, calling a configured server tool, and self-server computer helper calls through `__scriptkit_self__`.

## State Model

The bridge is best modeled as layered states rather than one transport.

| State | Meaning |
|---|---|
| JSONL stdin received | A raw line is trimmed and parsed for version observation. |
| Version observed | Missing field becomes v1; explicit v2 is accepted; unsupported versions increment diagnostics. |
| Stdin dispatch | Parsed command flows into the legacy/current command dispatcher; v2 gate reconciliation is pending. |
| Response stamped | Object responses can be stamped with current protocol version. |
| MCP request received | HTTP `/rpc` request is authenticated and parsed as JSON-RPC. |
| JSON-RPC method routed | Method enum routes initialize/tools/resources requests. |
| MCP resource read | URI is parsed and read from the registry or rejected with the correct error. |
| MCP tool call | Tool namespace dispatches to kit, script, or computer handler. |
| SDK MCP session | SDK initializes, sends calls, normalizes results, and closes. |
| Diagnostics read | Agents inspect protocol stats and transaction traces. |

## Interaction Matrix

These interactions define what agents can prove without screenshots.

| Interaction | Expected behavior | Proof |
|---|---|---|
| Send legacy stdin line. | Missing version is observed as v1. | Ingress golden and command receipt. |
| Send valid v2 stdin line. | Observer accepts v2; dispatch support remains policy-gated. | Version observer plus targeted stdin dispatch test. |
| Send unsupported version. | Dispatch hard-rejects and diagnostics counter increments. | Protocol stats resource. |
| Read `kit://context?profile=minimal`. | JSON context snapshot with schema version and preserved URI. | MCP `resources/read`. |
| Read invalid context query. | JSON-RPC invalid params. | MCP resource test. |
| Call `tools/list`. | Catalog includes kit/computer/script-derived tools. | JSON-RPC success response. |
| Call unknown `kit/*`. | Tool-level error result. | Tool call receipt. |
| Call `computer/see`. | Structured inspect snapshot when runtime bridge exists. | Tool call text parses as snapshot JSON. |
| Read protocol stats. | Health/counters/thresholds returned. | `kit://diagnostics/protocol-stats`. |
| Read transaction latest by request id. | Latest matching transaction trace returned. | `kit://transactions/latest?requestId=...`. |

## Safe Claims

These are safe statements for docs and agent workflows today.

- Missing stdin `protocolVersion` means legacy v1.
- Numeric `protocolVersion:2` is current for the version reader, observer, and stdin dispatch.
- Outbound object responses can be stamped with `protocolVersion:2`.
- Unsupported protocol versions are observable via diagnostics; dispatch rejection is not yet the documented guarantee.
- `triggerBuiltin.name` is deprecated in favor of `builtinId`.
- The app-owned MCP server is HTTP JSON-RPC over `/rpc`, bearer-token protected, and localhost-oriented.
- Current MCP method names are `initialize`, `tools/list`, `tools/call`, `resources/list`, and `resources/read`.
- `kit://context` supports profiles, flags, diagnostics, and schema.
- Bad context query parameters are invalid params; unknown resources are method-not-found style errors.
- Computer tools are read-only observation/capture tools, except exact native-window capture.
- The SDK MCP client supports HTTP and stdio configured servers and uses MCP spec version `2024-11-05`.

## Unsafe Claims

Do not make these claims until implementation and proof exist.

- Explicit stdin `protocolVersion:2` is fully supported by the typed stdin dispatch gate.
- Unsupported stdin versions are rejected before dispatch.
- The app-owned MCP server supports stdio.
- Computer tools can click, focus, activate, type, press menus, or request permissions.
- Native window inventory is complete for minimized/off-screen windows.
- The SDK global wrapper inventory is exhaustive.
- The MCP golden fixture fully covers every method and resource class.
- The filtered Oracle bundle proves every server route, lifecycle, and security error state.

## Code Ownership

These source areas own the bridge behavior.

| Area | Source anchors |
|---|---|
| Protocol versioning | `src/protocol/version.rs`, `src/protocol/ingress.rs`, `src/protocol/deprecations.rs` |
| Stdin parsing/reader | `src/protocol/io/parsing.rs`, `src/protocol/io/reader.rs`, stdin command dispatcher |
| MCP protocol | `src/mcp_protocol/mod.rs` |
| MCP server | `src/mcp_server/mod.rs` |
| Resource registry | `src/mcp_resources/mod.rs`, `src/mcp_resources/transaction_resources.rs` |
| Kit tools | `src/mcp_kit_tools.rs` |
| Script tools | `src/mcp_script_tools/*` |
| Computer tools | `src/mcp_computer_use_tools.rs` |
| SDK MCP helpers | `scripts/kit-sdk.ts` |
| Protocol stats | `kit://diagnostics/protocol-stats`, protocol stats tests |
| Golden tests | `tests/golden/mcp/basic_rpc.jsonl`, `tests/golden/protocol/ingress_observations.jsonl` |

## Implementation Plan

Implementation should close the bridge gaps in layers.

1. Keep explicit stdin `protocolVersion:2` dispatch covered for representative commands such as `show`, `getState`, and `triggerBuiltin`.
2. Keep unsupported future versions as hard dispatch rejection with protocol-stats telemetry.
3. Complete the MCP server lifecycle source pass for binding, discovery file, token lifecycle, wrong path/method, content-type, shutdown, and status/error boundaries.
4. Add MCP golden rows for `tools/call` and `resources/read`.
5. Snapshot the full resource inventory with URI/name/mime/schema-version class.
6. Complete kit and script tool inventories, including input schemas, result shapes, unknown-tool behavior, and execution errors.
7. Audit full SDK MCP globals and `globalThis.computer` wrappers from `scripts/kit-sdk.ts`.
8. Add SDK tests for headers, session ids, initialize/initialized, disabled server rejection, close-on-error, invalid JSON tool text, and self-server config failures.

## Verification Recipes

Use state and protocol receipts rather than screenshots.

### Existing Receipts

Run the current targeted suites when changing this bridge.

```bash
cargo test --test protocol_ingress_golden -- --nocapture
cargo test --test protocol_stats_report_contract -- --nocapture
cargo test --test stdin_protocol_version_dispatch_contract -- --nocapture
bun scripts/agentic/protocol-v2-dispatch.ts
cargo test --test mcp_protocol_golden -- --nocapture
cargo test --test mcp_resource_drift -- --nocapture
cargo test --test mcp_resources_sdk_reference -- --nocapture
cargo test --test context_snapshot -- --nocapture
cargo test --test context_contract_end_to_end -- --nocapture
lat check
```

### New Receipts To Add

These receipts close the current weak spots.

| Receipt | Proof |
|---|---|
| v2 stdin dispatch contract | `stdin_protocol_version_dispatch_contract` plus `protocol-v2-dispatch.ts` prove valid v2 `show`, `getState`, and `triggerBuiltin` parse, dispatch, and produce receipts. |
| Unsupported-version policy | Inline stdin parser tests prove `protocolVersion:999` hard-rejects and increments protocol stats while invalid non-integer versions do not consume the unsupported counter. |
| MCP discovery fixture | `server.json` endpoint/token fields and write timing are stable. |
| MCP route matrix | Good `/rpc`, wrong path, wrong method, missing/bad token, bad JSON. |
| Full method golden | `tools/call` and `resources/read` happy/error rows. |
| Resource inventory snapshot | Every URI class and MIME/schema version is pinned. |
| Context query matrix | Valid/invalid profiles, booleans, unknown flags, schema query behavior. |
| Computer non-action audit | Observation handlers cannot focus/click/type/activate/request permission. |
| SDK MCP session tests | Headers, session id, initialize, initialized notification, close-on-error. |
| SDK global wrapper snapshot | `mcp` and `computer` global methods are documented and drift-tested. |

## Agent Notes

Keep bridge proof source-backed and avoid proxy claims.

- Do not use a successful MCP `tools/list` as proof that every tool works.
- Do not use protocol observer acceptance as proof of stdin dispatch unless a command receipt also exists.
- Do not document a resource or tool as exhaustive from a keyword-context bundle alone.
- Treat raw local payloads in MCP/action/state receipts as privacy regressions.
- Prefer `getState`, `getElements`, MCP resource reads, protocol stats, and transaction resources over screenshots.
- Keep the app-owned HTTP server distinct from SDK-configured stdio MCP clients.

## Related Features

Feature 004 is the broad MCP/SDK/protocol map that this focused bridge pass refines.

- [004 MCP SDK Protocol](./004-mcp-sdk-protocol.md) owns the broad map of JSONL automation, MCP resources, SDK/scriptability, and agent receipts.
- [038 Agent Skills and AI Context Catalog](./038-agent-skills-ai-context-catalog.md) covers agent skill context and resource-backed context catalogs.
- [039 Logging and Transaction Observability](./039-logging-diagnostics-transaction-observability.md) owns transaction traces and diagnostic logging.

## Open Questions And Gaps

These remain explicit until source passes and tests close them.

- Should unsupported stdin protocol versions become hard dispatch rejections, or stay diagnostics-only?
- What is the exact `server.json` schema and token-file lifecycle across restart/rotation/failure?
- What are the full wrong-path, wrong-method, content-type, CORS/preflight, and malformed-body behaviors?
- Which script metadata shapes publish callable MCP tool schemas, and what errors return tool-level `isError` versus JSON-RPC errors?
- What is the complete SDK `globalThis.mcp` and `globalThis.computer` wrapper inventory?
- Which MCP resource classes should get drift-audited reference docs beyond stdin commands and trigger builtins?

# 004 MCP Resources / SDK / Protocol Automation

This chapter maps the app-owned interfaces that let scripts, MCP clients, and agentic tests observe and drive Script Kit.

## Executive Summary

The MCP, SDK, and stdin protocol layer is the machine-facing contract for Script Kit. Humans use it through scripts, automation harnesses, MCP clients, and agentic proof tools; AI agents use it to inspect state, drive supported interactions, and collect receipts without relying on screenshots or native typing.

The feature owns JSONL stdin ingress, protocol version observation, deprecation warnings, `ExternalCommand` parsing, query messages, transaction helpers, automation window identity, MCP resources, MCP tools, SDK automation helpers, SDK reference docs, and agentic proof routing.

It does not own prompt rendering, ACP internals, Notes editor behavior, native OS input, script business logic, or surface-specific UI details. Those adjacent systems consume this feature's receipts.

## Human Capabilities

| Capability | Human value | Agent value | Primary receipt |
|---|---|---|---|
| Stdin control | Scripts and tests can show, hide, filter, run, and open app surfaces. | Agents can trigger known app entry points without native clicks. | Parse receipts plus follow-up query receipts. |
| Protocol queries | State, elements, layout, screenshots, ACP state, windows, waits, and batches can be requested by `requestId`. | Agents can prove behavior from structured state instead of pixels. | Response envelopes keyed by `requestId`. |
| Transactions | Multi-step operations can wait, select, submit, and trace failures deterministically. | Agents can replace fixed sleeps with `waitFor` and `batch`. | `waitForResult`, `batchResult`, transaction traces. |
| MCP resources | Context, catalogs, diagnostics, scripts, templates, and local state are readable through stable URIs. | Agents can gather current app knowledge before acting. | MCP resource payloads and drift tests. |
| MCP tools | External clients can access app-owned tools, script-derived tools, and read-only computer observation tools. | Agents can inspect native/app state while respecting non-action boundaries. | MCP tool results and schemas. |
| SDK helpers | Scripts can call automation helpers directly from `kit-sdk.ts`. | Agents can use scriptable APIs that mirror protocol contracts. | SDK runtime tests and `kit://sdk-reference`. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Protocol envelope | Optional `protocolVersion` on JSON protocol objects. | Missing version behaves as legacy v1; current outbound messages stamp the current protocol version. |
| Ingress observation | Lightweight classification of incoming protocol lines. | Bad JSON, non-object roots, unsupported versions, and deprecations are observed before full dispatch. |
| External command | Fire-and-forget stdin command parsed before SDK prompt messages. | Known automation verbs with bad payloads return structured parser mismatches; unknown verbs fall through to SDK parsing. |
| Query command | Response-capable protocol message with `requestId`. | The response envelope must echo the request id and typed result. |
| Automation target | Main, focused, exact id, or kind-based target for windows and popups. | Non-main targets are inspected through target-aware commands, not through main-only `getState`. |
| Transaction | `waitFor` or `batch` request with deterministic command/result entries and optional traces. | Reusing the same request id with the same stable fingerprint may replay prior results; different payloads reject. |
| MCP resource | Readable URI exposed by the app-owned MCP server. | Resource payloads are versioned or drift-audited where they are user-facing contracts. |
| MCP tool | Callable tool exposed by the app-owned MCP server. | Computer tools are read-only observation/capture except exact native window capture. |
| SDK helper | Script global implemented by `scripts/kit-sdk.ts`. | Helpers send protocol messages and resolve typed fallback-safe results. |

## Entry Points

| Entry | User path | Protocol path | Result |
|---|---|---|---|
| JSONL stdin | Test harness, `session.sh`, scripts, SDK runtime. | One JSON object per line. | Parsed command, SDK message, parse error, or typed response. |
| App-owned MCP server | MCP clients and agent workflows. | JSON-RPC initialize, tools/list, resources/list, resources/read, tools/call. | Resource payloads or tool results. |
| SDK global helpers | Script Kit scripts running under the preload. | `getState()`, `getElements()`, `waitFor()`, `batch()`, `computer.*`. | Promise results backed by protocol/MCP calls. |
| Agentic CLI | `scripts/agentic/index.ts` scenarios and proof commands. | Session send/rpc helpers, target promotion, state/elements/inspect/wait/batch. | Proof bundles, manifests, and cleanup receipts. |
| In-app SDK Reference | Human-facing SDK documentation surface. | Same `SdkFunctionRef` source as `kit://sdk-reference`. | Function signatures, categories, support status, and examples. |

## User Workflows

### Drive The Main Window From Stdin

A script or harness sends `show`, `hide`, `setFilter`, `triggerBuiltin`, `simulateKey`, `run`, `openNotes`, or ACP-compatible open/input commands over stdin. Fire-and-forget commands produce parser receipts rather than command-specific result envelopes, so the caller follows with `getState`, `getElements`, `listAutomationWindows`, or `inspectAutomationWindow`.

Important state rules:

- `show` shows the main window, marks it visible, syncs the main automation window, and focuses the main filter.
- `hide` resets to `ScriptList`, rekeys the semantic surface to `scriptList`, then defers hiding.
- `setFilter` writes the main filter text verbatim within the stdin line cap.
- `triggerBuiltin` accepts canonical `builtinId`; deprecated `name` is observed through the deprecation registry.
- `simulateKey` routes through the same app key handling as the active view; it is still fire-and-forget.

### Query App State

A caller sends response-capable protocol messages such as `getState`, `getElements`, `inspectAutomationWindow`, `listAutomationWindows`, `getAcpState`, `waitFor`, or `batch`. The response envelope is the durable receipt.

`getState` is the main-window state contract. It includes prompt identity, input value, dataset and visible counts, selection/focus data, `windowVisible`, `surfaceContract`, active popup/footer contracts, decoration receipts, scroll data, and screenshot identity when present.

`getElements` is the preferred semantic element inventory for both main and non-main targets. It reports visible rows/buttons/statuses, total count, truncation, focused and selected semantic ids, and collector warnings.

### Target A Popup Or Secondary Window

Agents resolve exact targets through `listAutomationWindows` and `inspectAutomationWindow`. For attached popups, detached ACP, and Notes windows, `getElements(target)`, `inspectAutomationWindow(target)`, `getAcpState(target)`, and target-aware `batch` commands are preferred.

An explicit non-main target passed to `getState` should return an unsupported diagnostic rather than pretending the main-state schema applies.

### Wait And Batch Instead Of Sleeping

`waitFor` polls named or detailed conditions with timeouts, poll intervals, optional target, and optional trace. It returns success, elapsed time, errors, and trace data when requested.

`batch` executes a stable list of commands against a target. Supported primitives include `setInput`, `waitFor`, `selectByValue`, and `selectBySemanticId`; visible compound commands such as `forceSubmit`, `filterAndSelect`, and `typeAndSubmit` can be unsupported depending on executor wiring.

### Read MCP Resources

MCP clients read versioned and diagnostic resource URIs to understand current app state, context, scripts, templates, transaction traces, and local environment. These resources are part of the agent contract because they let agents gather context before driving UI.

### Use SDK Helpers From Scripts

Scripts call `getState()`, `getElements(limit?)`, `waitFor(condition, options?)`, `batch(commands, options?)`, and `computer.*` wrappers. The SDK should mirror the Rust protocol surface closely enough that typed script authors can use every supported primitive without guessing raw JSON.

## Interaction Matrix

| Interaction | Input | State transition | Expected proof | Failure modes |
|---|---|---|---|---|
| Show app | `{"type":"show"}` | Main window visible and focused. | `getState.windowVisible`, `listAutomationWindows`. | Fire-and-forget parse succeeds but state fails to change. |
| Hide app | `{"type":"hide"}` | Main resets to ScriptList then hides. | `getState.promptType`, `windowVisible:false`. | Reset/hide order regression. |
| Set filter | `{"type":"setFilter","text":"..."}` | Main filter updates. | `getState.inputValue`, `getElements`. | Input not echoed, stale filter, line cap. |
| Trigger built-in | `{"type":"triggerBuiltin","builtinId":"..."}` | Built-in opens and surface rekeys. | `surfaceContract`, `getElements`, automation registry. | Both/neither `builtinId` and deprecated `name`, unknown id. |
| Simulate key | `{"type":"simulateKey","key":"Escape"}` | Active view handles key intent. | State/elements after event. | Unknown modifiers, wrong route, popup-first contract break. |
| Open Notes | `{"type":"openNotes"}` | Notes window appears. | `listAutomationWindows`, `inspectAutomationWindow`, target elements. | Missing secondary target or wrong host mode. |
| Open ACP | `openAi`, `openMiniAi`, `showAiCommandBar`, related aliases. | Agent Chat or command bar opens. | `getAcpState`, ACP waits, target elements. | ACP setup/card state not represented. |
| Query state | `{"type":"getState","requestId":"..."}` | No mutation. | `stateResult`. | Non-main target unsupported diagnostic, target resolution failure. |
| Query elements | `{"type":"getElements","requestId":"..."}` | No mutation. | `elementsResult`. | Collector fallback warnings, truncation. |
| Inspect target | `inspectAutomationWindow` | No mutation. | Window identity, bounds, surface, visibility diagnostics. | Target not found or ambiguous. |
| Wait | `waitFor` | Polls until condition satisfied or timeout. | `waitForResult` and optional trace. | Timeout, invalid condition, setup state unavailable. |
| Batch | `batch` | Applies command list deterministically. | `batchResult` entries, `failedAt`, trace. | Unsupported command, target capability error, replay mismatch. |
| Read resource | MCP `resources/read`. | No mutation. | Resource payload. | Unknown URI, invalid query, stale schema. |
| Call computer tool | MCP `tools/call computer/*`. | Mostly read-only observation. | Structured result with status/warnings. | Permission/capture/ownership mismatch, forbidden action output. |

## Protocol State Map

### Version And Deprecation Handling

| State | Owner | Input | Behavior | Receipt |
|---|---|---|---|---|
| Legacy envelope | `read_wire_version` | Missing `protocolVersion`. | Treat as v1. | Parsed command or query. |
| Current envelope | `attach_current_version` | Outbound protocol response. | Stamp current protocol version. | Response envelope. |
| Unsupported version | `read_wire_version`, `observe_ingress`, `record_unsupported_version`. | Too old or too new. | Typed observation/dispatch error and unsupported-version telemetry. | Protocol stats resource. |
| Deprecated field | `validate_deprecations`. | `triggerBuiltin.name`. | Warns from v2, replacement `builtinId`, removed at v3. | Warning or deprecation error. |
| Parser order | `parse_stdin_command`. | External command first, SDK message fallback second. | Known bad automation payload reports mismatch; unknown type can fall through. | Parse receipt or SDK parse path. |

The stdin gate is reconciled with the core protocol envelope: missing fields dispatch as legacy v1, explicit v1 and v2 dispatch, and unsupported future versions hard-reject before typed dispatch while updating protocol stats.

### Fire-And-Forget Commands

| Command | Payload | Mutates | Follow-up proof |
|---|---|---|---|
| `show` | Optional `requestId`. | Shows and focuses main window. | `getState`, `getElements`, `listAutomationWindows`. |
| `hide` | Optional `requestId`. | Resets main to ScriptList, rekeys, hides. | `getState.windowVisible:false`. |
| `setFilter` | `text`, optional `requestId`. | Updates main input. | `getState.inputValue`. |
| `triggerBuiltin` | `builtinId` or deprecated `name`. | Opens built-in route. | `surfaceContract`, `getElements`. |
| `simulateKey` | `key`, optional `modifiers`. | Routes active keyboard handling. | State/elements after route. |
| `run` | Script `path`. | Starts script/prompt execution. | Prompt/script state and logs. |
| `openNotes` | No visible request id in raw variant. | Opens Notes window. | Target window and elements. |
| `openAbout` | No visible request id in raw variant. | Opens About surface. | Main state/elements. |

### Query Commands

| Query | Scope | Result | Notes |
|---|---|---|---|
| `getState` | Main window. | `stateResult`. | Explicit non-main target returns unsupported diagnostic. |
| `getElements` | Main or supported target. | `elementsResult`. | Preferred for secondary targets and popups. |
| `getLayoutInfo` | Layout proof. | Layout receipt. | Exact schema still needs source pass. |
| `captureScreenshot` | Visual proof target. | Screenshot receipt/file. | Distinct from `captureWindow` and computer capture tools. |
| `getAcpState` | ACP main/embedded/detached target. | ACP state snapshot. | Use instead of targeted `getState` for ACP. |
| `getAcpTestProbe` | ACP test/probe path. | Probe receipt. | Exact schema still needs source pass. |
| `performAcpSetupAction` | ACP setup path. | Setup action result. | ACP internals are adjacent. |
| `inspectAutomationWindow` | Exact/kind/focused target. | Window identity and diagnostics. | Preferred target identity proof. |
| `listAutomationWindows` | Runtime session. | Registry list and focused id. | Enables exact target promotion. |
| `simulateGpuiEvent` | Targeted GPUI event. | Event result. | Preferred for detached ACP before native input. |
| `waitFor` | Main or target. | `waitForResult`. | Replaces sleeps. |
| `batch` | Main or target. | `batchResult`. | Deterministic multi-step proof. |

## Automation Receipts

| Receipt | Contents | Used for |
|---|---|---|
| `stateResult` | Prompt type/id, input value, counts, selection, focus, visibility, surface contract, active popup/footer/decorations, scroll, screenshot identity. | Main-window proof. |
| `elementsResult` | Visible semantic elements, total count, truncation, focused/selected semantic ids, warnings. | Row/button/status proof and target inspection. |
| `waitForResult` | Success, elapsed, error, trace. | Replacing sleeps and diagnosing condition failures. |
| `batchResult` | Success, result entries, failed index, elapsed, trace. | Deterministic multi-step operations. |
| `inspectAutomationWindow` | Target id/kind, bounds, surface, visibility, diagnostics. | Secondary window and popup identity. |
| `listAutomationWindows` | Registry entries and focused automation window id. | Target discovery. |
| `getAcpState` | ACP readiness, composer, setup, picker, session state. | ACP-specific proof. |
| Transaction trace | Poll snapshots, command traces, errors. | Failure diagnosis and resource replay. |
| Protocol stats | Unsupported version counters, health, thresholds. | Ingress compatibility diagnostics. |

## MCP Resource Matrix

| Resource | Payload | Key states |
|---|---|---|
| `kit://state` | JSON app state. | Legacy/simple state can lag richer `getState`. |
| `scripts://` | Legacy scripts catalog. | Alias for older clients. |
| `scriptlets://` | Legacy scriptlets catalog. | Alias for older clients. |
| `kit://scripts` | Versioned scripts catalog. | Schema version, count, scripts. |
| `kit://scriptlets` | Versioned scriptlet catalog. | Schema version, count, scriptlets. |
| `kit://sdk-reference` | SDK function reference. | Shared with in-app SDK Reference UI. |
| `kit://context` | AI context snapshot. | Supports profile and field query flags. |
| `kit://context?diagnostics=1` | Context diagnostics wrapper. | Per-field status/meta. |
| `kit://context/schema` | Context schema and examples. | Query params are forbidden. |
| `kit://clipboard-history` | Clipboard history list. | Default limit. |
| `kit://clipboard-history?limit=N` | Clamped clipboard list. | Max clamp parser. |
| `kit://clipboard-history?id=<id>` | Single clipboard entry. | Missing id handled as stateful error. |
| `kit://clipboard-history?diagnostics=1` | Clipboard diagnostics. | Provider and wrapper status. |
| `kit://focused-item` | Focused item doc. | `schemaVersion`, `hasFocusedItem`, optional item. |
| `kit://focused-item?diagnostics=1` | Focused item diagnostics. | Stale slot and provider status. |
| `kit://git-status` | Text git status. | Current working directory shell read. |
| `kit://git-diff` | Text git diff. | Staged plus unstaged diff. |
| `kit://processes` | Text top processes. | Platform fallback command. |
| `kit://system` | Text system summary. | Platform command drift risk. |
| `kit://dictation` | Latest dictation snapshot. | Persisted JSONL history hydrated at startup. |
| `kit://dictation-history` | Newest-first dictation list. | Raw and display timestamp/duration. |
| `kit://dictation-history?id=<id>` | Single transcript. | Missing/stale id handling. |
| `kit://calendar` | Calendar provider data. | Availability encoded. |
| `kit://notifications` | Notifications provider data. | Availability encoded. |
| `kit://stdin-commands` | Markdown accepted stdin verbs. | Drift-audited marker block. |
| `kit://trigger-builtins` | Markdown canonical builtin ids. | Drift-audited marker block. |
| `kit://diagnostics/protocol-stats` | Protocol stats report. | Unsupported-version counters and health. |
| `kit://transactions/latest` | Latest wait/batch trace or empty payload. | Bounded persisted trace log. |
| `kit://transactions/latest?requestId=<id>` | Filtered transaction trace. | Request id parser. |
| `kit://transactions/schema` | Transaction schema and examples. | Wait/batch schema reference. |
| `kit://failed-scripts` | Failed-script validation report. | Excluded scripts, duplicate bindings, peers. |
| `kit://script-templates` | Starter template catalog. | Versioned template refs and bodies. |

## MCP Tool Matrix

| Tool family | Capability | Boundary |
|---|---|---|
| App-owned MCP server | Exposes resources and tools over local MCP. | Full server bind/discovery/shutdown details still need a source pass. |
| Script-derived tools | Converts script declarations into MCP tool schemas. | Exact execution and permission contract still needs a source pass. |
| Static kit tools | Built-in Kit tool schemas. | Exact inventory still needs a source pass. |
| `computer/see` | Observation/capture request with screenshot and semantic collection. | Not an input or action primitive. |
| Automation-window tools | `list_windows`, `get_window`, `get_focused_window`. | Registry-only; no native focus or mutation. |
| Running-app tools | List/get apps by pid or bundle id. | Running app inventory, not installed-app catalog or launcher. |
| App-window tools | List/get windows by pid, bundle id, or native window id. | No focus, launch, quit, move, resize, input, or action handles. |
| Native-window inventory | List/get native windows and frontmost windows/apps. | Observation only. |
| Exact native capture | `computer/capture_native_window`. | Exact pid/native id with optional bundle expectation; structured permission/ownership/stale/blank failures. |
| Menu cache tools | List cached frontmost menu tree and menu items. | No AX refresh, permission prompt, click, press, or menu action. |
| Tray model tools | List/get Script Kit tray menu model items. | No status item click or global menu extra discovery. |
| Screen tools | List/get active displays. | No window placement or screenshots. |
| Permission tools | Read-only preflight statuses. | No permission request, settings open, event synthesis, or mutation. |

## SDK And Agentic Surface

| Surface | API | Result | Notes |
|---|---|---|---|
| SDK state | `getState()` | `PromptState`. | Sends protocol `getState`; fallback response if missing/wrong type. |
| SDK elements | `getElements(limit?)` | `ElementsSnapshot`. | Preserves warnings/truncation. |
| SDK wait | `waitFor(condition, options?)` | Wait result. | Fallback error says waitFor fallback was used. |
| SDK batch | `batch(commands, options?)` | Batch result. | Rust executor owns unsupported command errors. |
| SDK trigger builtins | `triggerBuiltin(...)`. | Built-in route. | Canonical literals must match registry. |
| SDK computer | `computer.listNativeWindows()`, `computer.captureNativeWindow()`, and likely more. | Same structured receipts as MCP tool calls. | Full wrapper inventory still needs pass. |
| SDK reference | `kit://sdk-reference` and in-app UI. | Function signatures, categories, support status. | Unsupported examples/templates fail lint. |
| Agent input routing | `chooseInputMethod`. | Batch or GPUI event for exact targets; native only as last resort. | Main/focused/unspecified can still fall back to native. |
| Surface proof | `surface-proof`. | State/elements/inspect/wait/batch proof bundle. | Should report no screenshots/native input for state-first classes. |
| Surface navigator | `surface-navigate`. | Trigger builtin, wait prompt type, exact target, state/elements, optional safe batch. | Visual capture only after final state/elements. |
| Attached popup matrix | Actions dialog and ACP slash prompt popup cases. | Exact popup id, parent identity, crop data. | Wrong popup id rejected. |
| Filterable surface matrix | Built-in filterable surfaces. | Surface contract, semantic ids, counts vs rows. | Collector fallback warnings should fail audits. |

## Data, Storage, And Privacy Boundaries

- Protocol stdin lines are capped; long request ids round-trip except charset-unsafe parse errors use unscoped fallback paths.
- MCP resources expose local context and history intentionally. Each resource should encode availability, diagnostics, schema version, or query errors rather than silently omitting state.
- `kit://git-status`, `kit://git-diff`, `kit://processes`, and `kit://system` run local shell/system reads and must preserve platform fallback behavior.
- Computer tools must stay observation/capture oriented. They must not smuggle focus, click, type, action, permission prompt, launch, quit, move, or resize handles into outputs.
- Exact native capture requires stable native ids and ownership validation; it should fail structurally on stale ids, ownership mismatch, permission failure, non-candidates, duplicates, or blank captures.
- Transaction traces are bounded persisted diagnostics and should recover gracefully from malformed or corrupt trace logs.

## Error, Empty, Loading, And Disabled States

| Area | State | Expected behavior |
|---|---|---|
| JSON parse | Malformed JSON or non-object root. | Typed parse/ingress error; unsupported-version counter should not bump unless applicable. |
| Protocol version | Too old/too new/invalid. | Unsupported future versions hard-reject and increment telemetry; invalid non-integer envelopes reject without unsupported-version telemetry. |
| Deprecated fields | `triggerBuiltin.name`. | Warning before removal version, error at removed version. |
| External command | Known verb with wrong payload. | `automation_payload_mismatch`, not silent fallback. |
| Unknown command | Unknown external type. | Fall through to SDK message parser where applicable. |
| Request ids | Long id. | Round trip in response-capable commands. |
| `getState` target | Explicit non-main target. | Unsupported diagnostic, not fake main state. |
| Target resolution | Missing or ambiguous target. | `target_resolution_failed` or target-specific diagnostic. |
| Elements | Unsupported collector. | Fallback row and warning instead of empty success. |
| Wait | Condition timeout. | Structured timeout with elapsed and optional trace. |
| Batch | Unsupported command/capability. | Structured error with failed index and suggestion. |
| MCP resource | Unknown URI or bad query. | Resource error with parser detail. |
| MCP tool | Read-only boundary violation. | No action fields or hidden mutation handles. |
| Capture | Permission/stale/blank/ownership mismatch. | Structured failure status and warnings. |
| SDK fallback | Missing/wrong response type. | Explicit fallback response or fallback error. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Protocol versioning and deprecations | `src/protocol/version.rs`, `src/protocol/deprecations.rs`, `src/protocol/ingress.rs` |
| Protocol message variants | `src/protocol/message/variants/query_ops.rs`, `src/protocol/types/automation_window.rs`, `src/protocol/types/batch_wait.rs` |
| Transaction execution | `src/protocol/transaction_executor.rs` |
| Stdin commands | `src/stdin_commands/mod.rs`, `src/main_entry/runtime_stdin*.rs` |
| MCP resources | `src/mcp_resources/mod.rs`, `src/mcp_resources/transaction_resources.rs` |
| MCP protocol/server | `src/mcp_protocol/mod.rs`, `src/mcp_server/mod.rs` |
| MCP tools | `src/mcp_script_tools.rs`, `src/mcp_kit_tools.rs`, `src/mcp_computer_use_tools.rs` |
| SDK globals | `scripts/kit-sdk.ts` |
| Agentic proof CLI | `scripts/agentic/index.ts` |
| Protocol tests | `tests/protocol_batch.rs`, `tests/protocol_wait_for.rs`, `tests/protocol_wait_for_contract.rs`, `tests/protocol_ingress_golden.rs` |
| MCP resource tests | `tests/mcp_resource_drift.rs`, `tests/mcp_resources_sdk_reference.rs`, `tests/mcp_protocol_golden.rs` |
| SDK tests | `tests/sdk_automation_runtime.rs`, `tests/sdk_automation_contracts.rs` |
| Source audits | `tests/source_audits/trigger_builtin_registry_consistency.rs`, `tests/source_audits/trigger_builtin_sdk_literals.rs`, `tests/source_audits/sdk_computer_use_contract.rs` |

## Invariants And Regression Risks

- Fire-and-forget commands must be followed by state-first proof; they should not invent result envelopes that existing harnesses do not expect.
- `hide` must reset to ScriptList and rekey the semantic surface before hide completes.
- `choiceCount` is dataset count; `visibleChoiceCount` is filter-aware and must never exceed `choiceCount`.
- Main `getState` must stay main-only until a target-specific state schema exists.
- `getElements(target)` and `inspectAutomationWindow(target)` are the target-safe inspection path for secondary windows and popups.
- Attached popup proof must use exact popup targets and parent identity, not broad main screenshots.
- `waitFor` and `batch` are the preferred timing model; fixed sleeps are proof debt.
- Batch replay must reject reused request ids with different stable fingerprints.
- Resource marker blocks must stay drift-audited so prose/examples are not accidentally treated as runtime inventory.
- SDK TypeScript unions must not fall behind Rust-supported batch primitives.
- MCP computer tools must remain read-only observation/capture; action handles in output are a contract break.
- `kit://sdk-reference` and the in-app SDK Reference must share one source of truth.

## Verification Recipes

### Protocol Ingress And Versioning

Run:

```bash
cargo test protocol_ingress_golden
cargo test protocol_wait_for_contract
```

Check:

- Absent protocol version is accepted as legacy.
- Unsupported versions are typed and counted.
- Deprecated fields warn or error at the documented versions.
- Known malformed automation payloads do not fall through silently.

### Query Receipts

Run or script:

```bash
./scripts/agentic/index.ts surface-proof --class main
```

Check:

- `getState` returns main state with `surfaceContract`.
- `getElements` returns semantic rows and no unexpected fallback warnings.
- `listAutomationWindows` and `inspectAutomationWindow` identify the target used by the proof.

### Wait And Batch

Run:

```bash
cargo test protocol_batch
cargo test protocol_wait_for
```

Check:

- `waitFor` times out structurally with trace when requested.
- `batch` reports per-command entries, elapsed time, and failed index.
- Replay behavior is deterministic for identical request ids and payload fingerprints.

### MCP Resources

Run:

```bash
cargo test mcp_resource_drift
cargo test mcp_resources_sdk_reference
cargo test mcp_protocol_golden
```

Check:

- `kit://stdin-commands` and `kit://trigger-builtins` marker blocks match runtime accessors.
- `kit://sdk-reference` matches SDK function definitions and support status.
- Versioned resources serialize expected schema versions and camelCase fields.

### SDK Automation

Run:

```bash
cargo test sdk_automation_runtime
cargo test sdk_automation_contracts
cargo test trigger_builtin_sdk_literals
cargo test sdk_computer_use_contract
```

Check:

- SDK helpers resolve typed results.
- SDK trigger built-in literals match the registry.
- Computer SDK wrappers expose read-only observation/capture behavior and no native action handles.

## Agent Notes

- Prefer `getState`, `getElements`, `inspectAutomationWindow`, `waitFor`, and `batch` before screenshots or native input.
- Use exact automation targets for popups, detached ACP, and Notes windows.
- Use `getAcpState(target)` for ACP instead of forcing ACP into main `getState`.
- If a collector warning appears in `getElements`, treat it as an inventory gap, not a successful exhaustive proof.
- For fire-and-forget stdin commands, record both the parse receipt and the follow-up state receipt.
- Before claiming a resource inventory is complete, compare against `get_resource_definitions()` and the drift tests.
- Before claiming SDK scriptability is complete, compare TypeScript unions in `scripts/kit-sdk.ts` to Rust executor-supported commands.

## Related Features

- [001 Main Menu](./001-main-menu.md) consumes `triggerBuiltin`, `simulateKey`, `getState`, `getElements`, and action popup proof.
- [002 File Search](./002-file-search.md) depends on target-aware elements and portal proof.
- [003 Agent Chat Context](./003-agent-chat-context.md) depends on ACP state, slash popup targets, context resources, and agentic proof routing.
- [006 Notes Window](./006-notes-window.md) depends on secondary window targeting, Notes-mode batch input, and Notes-hosted ACP proof.

## Raw Oracle References

- [Prompt](../raw-oracle/004-mcp-sdk-protocol/prompt.md)
- [Bundle map](../raw-oracle/004-mcp-sdk-protocol/bundle-map.md)
- [Answer](../raw-oracle/004-mcp-sdk-protocol/answer.md)
- [Full output log](../raw-oracle/004-mcp-sdk-protocol/output.log)
- [Session metadata](../raw-oracle/004-mcp-sdk-protocol/session.json)

## Open Questions And Gaps

- `src/mcp_server/mod.rs` needs a full source pass for HTTP bind, discovery file lifecycle, auth/boundary assumptions, request routing, shutdown, and error states.
- `src/mcp_kit_tools.rs` and `src/mcp_script_tools.rs` need full inventory passes for static tool schemas and script-derived tool schema/execution/error boundaries.
- Exact payload/result schemas for `getLayoutInfo`, `captureScreenshot`, `getAcpTestProbe`, `performAcpSetupAction`, and `simulateGpuiEvent` need source-backed expansion.
- v2 envelope support and stdin dispatch share the same accepted range; keep tests covering both ExternalCommand and protocol Message paths.
- `EXTERNAL_COMMAND_VERBS` and command dispatch arms should be audited for elided entries such as `openAbout`.
- `globalThis.computer` likely has more wrappers than the raw pass enumerated; inspect complete SDK reference and source before claiming the wrapper inventory is exhaustive.
- Target capability allow-lists in `BatchTargetCapabilities` need a per-target matrix.
- `get_resource_definitions()` should be audited against this chapter before this resource list is called complete.

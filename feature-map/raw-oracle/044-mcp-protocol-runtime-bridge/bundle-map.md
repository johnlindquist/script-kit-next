# 044 MCP Protocol Runtime Bridge Bundle Map

Oracle session for the MCP/protocol runtime bridge atlas.

## Session


## Token And Size Receipt


## Bundle Contents

The bundle was narrowed to a 47.8k-token keyword-context pass around protocol versioning, MCP resources, MCP JSON-RPC, SDK MCP helpers, and prior feature-map gaps.


- `AGENTS.md`
- `CLAUDE.md`
- `.agents/skills/mcp-context-resources/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`
- `removed-docs`
- `removed-docs`
- `removed-docs`
- `removed-docs`
- `scripts/kit-sdk.ts`
- `src/protocol/version.rs`
- `src/protocol/ingress.rs`
- `src/protocol/deprecations.rs`
- `src/protocol/io/parsing.rs`
- `src/protocol/io/reader.rs`
- `src/protocol/mod.rs`
- `src/mcp_protocol/mod.rs`
- `src/mcp_server/mod.rs`
- `src/mcp_resources/mod.rs`
- `src/mcp_resources/transaction_resources.rs`
- `src/mcp_kit_tools.rs`
- `src/mcp_script_tools/mod.rs`
- `src/mcp_computer_use_tools.rs`
- `tests/protocol_ingress_golden.rs`
- `tests/protocol_stats_report_contract.rs`
- `tests/mcp_protocol_golden.rs`
- `tests/mcp_resource_drift.rs`
- `tests/mcp_resources_sdk_reference.rs`
- `tests/context_snapshot.rs`
- `tests/context_contract_end_to_end.rs`
- `tests/golden/mcp/basic_rpc.jsonl`
- `tests/golden/protocol/ingress_observations.jsonl`
- `feature-map/features/004-mcp-sdk-protocol.md`

## Prompt Intent


- JSONL `protocolVersion` behavior and the v2 stdin dispatch gap.
- MCP HTTP server lifecycle and JSON-RPC methods.
- MCP resource inventory, schemas, diagnostics, drift tests, and failure modes.
- MCP tool families and read-only computer-use boundaries.
- SDK MCP client/config helpers and global `mcp` surface.
- Safe claims, unsafe claims, implementation plan, and verification receipts.

# 044 MCP Protocol Runtime Bridge Bundle Map

Oracle session for the MCP/protocol runtime bridge atlas.

## Session

- Feature id: `044-mcp-protocol-runtime-bridge`
- Oracle slug: `mcp-protocol-runtime-bridge`
- Status: completed
- Model: `gpt-5.5-pro`
- Browser label: `Latest`
- Thinking time: `extended`
- Completed at: `2026-05-15T15:14:50.101Z`
- Conversation URL: `https://chatgpt.com/c/6a073668-8614-83e8-9c29-0811397c78ff`

## Token And Size Receipt

- Bundle path: `/Users/johnlindquist/.oracle/bundles/mcp-protocol-runtime-bridge.txt`
- Bundle size: `173090` bytes
- Oracle reported input tokens: `49248`
- Oracle reported output tokens: `8390`
- Oracle reported total tokens: `57638`
- Raw output log size: `73986` bytes
- Extracted answer size: `67317` bytes

## Bundle Contents

The bundle was narrowed to a 47.8k-token keyword-context pass around protocol versioning, MCP resources, MCP JSON-RPC, SDK MCP helpers, and prior feature-map gaps.

Included context:

- `AGENTS.md`
- `CLAUDE.md`
- `.agents/skills/mcp-context-resources/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`
- `lat.md/protocol.md`
- `lat.md/ai-context.md`
- `lat.md/workspace.md`
- `lat.md/verification.md`
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

Oracle was asked to map:

- JSONL `protocolVersion` behavior and the v2 stdin dispatch gap.
- MCP HTTP server lifecycle and JSON-RPC methods.
- MCP resource inventory, schemas, diagnostics, drift tests, and failure modes.
- MCP tool families and read-only computer-use boundaries.
- SDK MCP client/config helpers and global `mcp` surface.
- Safe claims, unsafe claims, implementation plan, and verification receipts.

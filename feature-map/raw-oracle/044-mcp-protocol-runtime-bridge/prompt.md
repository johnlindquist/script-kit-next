[mcp-protocol-runtime-bridge]


- The feature map is a human-and-agent atlas. Each chapter must explain capabilities, entry points, interactions, APIs, state machines, receipt paths, and open risks clearly enough for humans and AI agents to operate the product.
- Repo process requires `removed-docs/` updates for changed behavior/docs and `source checks` verification. For this atlas loop, every Oracle session's full output is preserved under `feature-map/raw-oracle/<feature-id>/` and distilled into `feature-map/features/<feature-id>.md`.



- MCP HTTP server and JSON-RPC entrypoints.
- MCP resources and resource schema/drift boundaries.
- MCP tools and read-only computer-use boundaries.
- SDK MCP client/config helpers in `scripts/kit-sdk.ts`.
- Verification receipts that prove what agents can rely on without screenshots.


- Feature 004 already gives a broad map of MCP resources, SDK helpers, JSONL stdin commands, protocol automation, query receipts, transactions, and MCP tools.
  - full MCP server bind/discovery/auth/request/shutdown details;
  - full `src/mcp_kit_tools.rs` and `src/mcp_script_tools.rs` inventory;




Return a dense, implementation-ready atlas chapter outline for local agents to distill into `feature-map/features/044-mcp-protocol-runtime-bridge.md`.


1. Exact current behavior for `protocolVersion` on JSONL stdin and outbound responses.
4. MCP JSON-RPC methods and response/error behavior.
8. What humans and AI agents can safely rely on today.
9. Unsafe claims to avoid.
10. Implementation plan for any gaps.
11. Tests and agentic receipts that prove the current behavior and future fixes.


Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.

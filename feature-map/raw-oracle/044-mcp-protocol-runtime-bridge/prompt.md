[mcp-protocol-runtime-bridge]

Project briefing:

- Repository: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK support, JSONL stdin automation, and an HTTP MCP server.
- The feature map is a human-and-agent atlas. Each chapter must explain capabilities, entry points, interactions, APIs, state machines, receipt paths, and open risks clearly enough for humans and AI agents to operate the product.
- Repo process requires `lat.md/` updates for changed behavior/docs and `lat check` verification. For this atlas loop, every Oracle session's full output is preserved under `feature-map/raw-oracle/<feature-id>/` and distilled into `feature-map/features/<feature-id>.md`.
- Relevant ownership skills for this pass: `mcp-context-resources`, `protocol-automation`, `sdk-script-execution`, `agentic-testing`, and `testing-quality-gates`.

Goal:

Create a comprehensive feature-map reference for the MCP/protocol runtime bridge, focused on gaps left by feature 004:

- Protocol envelope and `protocolVersion` behavior, especially the apparent split between v2 observation support and explicit stdin `protocolVersion:2` rejection/gating.
- MCP HTTP server and JSON-RPC entrypoints.
- MCP resources and resource schema/drift boundaries.
- MCP tools and read-only computer-use boundaries.
- SDK MCP client/config helpers in `scripts/kit-sdk.ts`.
- Verification receipts that prove what agents can rely on without screenshots.

Current evidence:

- Feature 004 already gives a broad map of MCP resources, SDK helpers, JSONL stdin commands, protocol automation, query receipts, transactions, and MCP tools.
- It flags several areas needing a focused source pass:
  - full MCP server bind/discovery/auth/request/shutdown details;
  - full `src/mcp_kit_tools.rs` and `src/mcp_script_tools.rs` inventory;
  - reconciliation of protocol v2 envelope support with explicit stdin `protocolVersion:2` rejection;
  - SDK wrapper inventory and `kit://context` resource proof boundaries.

Bundle map:

- Repo process docs: `AGENTS.md`, `CLAUDE.md`.
- Owning skills: MCP context resources, protocol automation, SDK script execution, agentic testing, testing gates.
- Lat docs: protocol, AI context/MCP, workspace, verification.
- Protocol source excerpts: version, ingress, deprecations, JSONL parsing/reader, protocol module.
- MCP source excerpts: protocol dispatcher, HTTP server, resources, transaction resources, kit tools, script tools, computer-use tools.
- SDK source excerpts: `scripts/kit-sdk.ts` MCP client/config and computer helpers.
- Tests/fixtures: protocol ingress, protocol stats, MCP golden tests, MCP resource drift/reference tests, context resources, MCP and protocol golden fixtures.
- Prior feature-map evidence: feature 004 chapter.

Deliverable:

Return a dense, implementation-ready atlas chapter outline for local agents to distill into `feature-map/features/044-mcp-protocol-runtime-bridge.md`.

Please include:

1. Exact current behavior for `protocolVersion` on JSONL stdin and outbound responses.
2. Whether explicit `protocolVersion:2` stdin should be documented as supported, observed-only, warned, or rejected based on the bundle.
3. MCP server lifecycle: bind address/port, discovery/token files, route handling, request parsing, error states, shutdown/handle retention, and security boundaries.
4. MCP JSON-RPC methods and response/error behavior.
5. MCP resources: inventory, schema/versioning, diagnostics, query parsing, drift-audited resources, context resources, transaction resources, and failure modes.
6. MCP tools: kit tools, script-derived tools, read-only computer tools, native capture, permissions, and non-action boundaries.
7. SDK MCP client helpers: config resolution, remote/http and stdio support, protocol headers, sessions, error behavior, and global `mcp` surface.
8. What humans and AI agents can safely rely on today.
9. Unsafe claims to avoid.
10. Implementation plan for any gaps.
11. Tests and agentic receipts that prove the current behavior and future fixes.

Output boundary:

Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.

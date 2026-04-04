---
title: "Protocol and Introspection"
slug: "protocol-and-introspection"
sourceSnapshot: "4be166ea"
sourceDocuments:
  - "raw/4be166ea/docs/PROTOCOL.md"
  - "raw/4be166ea/docs/ROADMAP.md"
relatedPages:
  - "architecture"
  - "ai-context-and-mcp"
  - "tab-ai-harness"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-04T18:48:29.295Z"
---

# Protocol and Introspection

The current automation-facing protocol, visible-element introspection, and deterministic transaction model.

## Key Facts
- getElements returns visible UI elements using stable semantic IDs such as input:filter, list:choices, and choice:<index>:<value>.
- elementsResult includes totalCount, truncated, focusedSemanticId, selectedSemanticId, and machine-readable warnings.
- waitFor and batch provide deterministic transactions so agents do not have to guess with sleeps.
- The roadmap frames introspection, batching, and semantic targeting as foundational for reliable autonomous operation.

## Key Files
- `docs/PROTOCOL.md` — Protocol reference. Visible element introspection, MCP context resources, deterministic transactions, and structured logging.
- `docs/ROADMAP.md` — AI UX protocol roadmap. Proposed next-stage protocol surfaces for autonomous agent interaction.

## Source Documents
- [raw/4be166ea/docs/PROTOCOL.md](../raw/4be166ea/docs/PROTOCOL.md)
- [raw/4be166ea/docs/ROADMAP.md](../raw/4be166ea/docs/ROADMAP.md)

## Related Pages
- [architecture](./architecture.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [tab-ai-harness](./tab-ai-harness.md)

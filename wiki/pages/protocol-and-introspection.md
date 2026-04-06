---
title: "Protocol and Introspection"
slug: "protocol-and-introspection"
sourceSnapshot: "fa760732"
sourceDocuments:
  - "raw/fa760732/docs/PROTOCOL.md"
  - "raw/fa760732/docs/ROADMAP.md"
  - "raw/fa760732/src/protocol/message/variants/query_ops.rs"
  - "raw/fa760732/src/protocol/types/elements_actions_scriptlets.rs"
  - "raw/fa760732/src/app_layout/collect_elements.rs"
  - "raw/fa760732/src/prompt_handler/mod.rs"
relatedPages:
  - "architecture"
  - "ai-context-and-mcp"
  - "tab-ai-harness"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-06T18:08:02.732Z"
---

# Protocol and Introspection

The current automation-facing protocol, visible-element introspection, and deterministic transaction model.

## Key Facts
- `getElements` is implemented as wire variants in `src/protocol/message/variants/query_ops.rs` and serialized around `ElementInfo` / `ElementType` from `src/protocol/types/elements_actions_scriptlets.rs`.
- Live UI snapshots are collected per-view in `src/app_layout/collect_elements.rs`, which emits stable semantic IDs, focused/selected IDs, total counts, truncation state, and machine-readable warning codes.
- `PromptMessage::WaitFor` and `PromptMessage::Batch` handling in `src/prompt_handler/mod.rs` execute deterministic transactions and emit structured completion logs for automation flows.
- `docs/PROTOCOL.md` documents the current public contract, while `docs/ROADMAP.md` describes next-stage protocol surfaces such as richer filtering and accessibility-tree expansion.

## Key Files
- `docs/PROTOCOL.md` — Protocol reference. Visible element introspection, MCP context resources, deterministic transactions, and structured logging.
- `docs/ROADMAP.md` — AI UX protocol roadmap. Proposed next-stage protocol surfaces for autonomous agent interaction.
- `src/protocol/message/variants/query_ops.rs` — Protocol query message variants. Wire types for getElements, elementsResult, waitFor, batch, and related transaction receipts.
- `src/protocol/types/elements_actions_scriptlets.rs` — Element introspection types. ElementType and ElementInfo definitions used by getElements responses.
- `src/app_layout/collect_elements.rs` — Visible element collectors. Per-view element collection that emits stable semantic IDs, totals, focused/selected IDs, and warnings.
- `src/prompt_handler/mod.rs` — Automation request handler. Runtime execution and logging for element queries, waitFor polling, and batch transactions.

## Source Documents
- [raw/fa760732/docs/PROTOCOL.md](../raw/fa760732/docs/PROTOCOL.md)
- [raw/fa760732/docs/ROADMAP.md](../raw/fa760732/docs/ROADMAP.md)
- [raw/fa760732/src/protocol/message/variants/query_ops.rs](../raw/fa760732/src/protocol/message/variants/query_ops.rs)
- [raw/fa760732/src/protocol/types/elements_actions_scriptlets.rs](../raw/fa760732/src/protocol/types/elements_actions_scriptlets.rs)
- [raw/fa760732/src/app_layout/collect_elements.rs](../raw/fa760732/src/app_layout/collect_elements.rs)
- [raw/fa760732/src/prompt_handler/mod.rs](../raw/fa760732/src/prompt_handler/mod.rs)

## Related Pages
- [architecture](./architecture.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [tab-ai-harness](./tab-ai-harness.md)

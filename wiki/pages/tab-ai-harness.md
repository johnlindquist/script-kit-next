---
title: "Tab AI Harness"
slug: "tab-ai-harness"
sourceSnapshot: "4be166ea"
sourceDocuments:
  - "raw/4be166ea/README.md"
  - "raw/4be166ea/src/ai/harness/mod.rs"
  - "raw/4be166ea/src/ai/tab_context.rs"
  - "raw/4be166ea/tests/tab_ai_context.rs"
relatedPages:
  - "project-overview"
  - "ai-context-and-mcp"
  - "verification-and-testing"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-04T18:48:29.295Z"
---

# Tab AI Harness

The current QuickTerminal-based Tab AI flow, context assembly, and harness submission model.

## Key Facts
- The primary Tab AI surface is QuickTerminalView rendered via TermPrompt, not the legacy inline chat path.
- Plain Tab stages context using PasteOnly, while Shift+Tab from ScriptList can submit the current filter text as user intent.
- Each Tab press writes ~/.scriptkit/context/latest.md and spawns a fresh claude process with context and intent.
- TabAiContextBlob, TabAiTargetAudit, and related tests define the current schema contract for the harness path.

## Key Files
- `README.md` — Project overview. High-level product positioning, setup, prompt APIs, configuration, and built-in capabilities.
- `src/ai/harness/mod.rs` — Tab AI harness. Harness configuration and context formatting for the external CLI flow.
- `src/ai/tab_context.rs` — Tab AI context types. Tab AI context blob, target audit, UI snapshot, and clipboard context types.
- `tests/tab_ai_context.rs` — Tab AI context tests. Integration tests for the Tab AI context blob schema and serialization.

## Source Documents
- [raw/4be166ea/README.md](../raw/4be166ea/README.md)
- [raw/4be166ea/src/ai/harness/mod.rs](../raw/4be166ea/src/ai/harness/mod.rs)
- [raw/4be166ea/src/ai/tab_context.rs](../raw/4be166ea/src/ai/tab_context.rs)
- [raw/4be166ea/tests/tab_ai_context.rs](../raw/4be166ea/tests/tab_ai_context.rs)

## Related Pages
- [project-overview](./project-overview.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [verification-and-testing](./verification-and-testing.md)

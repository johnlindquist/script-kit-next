---
title: "Tab AI Harness"
slug: "tab-ai-harness"
sourceSnapshot: "c9fbc3ca"
sourceDocuments:
  - "raw/c9fbc3ca/README.md"
  - "raw/c9fbc3ca/src/ai/harness/mod.rs"
  - "raw/c9fbc3ca/src/ai/tab_context.rs"
  - "raw/c9fbc3ca/tests/tab_ai_context.rs"
relatedPages:
  - "project-overview"
  - "ai-context-and-mcp"
  - "verification-and-testing"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-04T19:40:47.586Z"
---

# ACP Chat

The primary and only AI chat surface, the ACP chat runtime, and the compatibility-named context plumbing that still backs it.

## Key Facts
- ACP Chat is the primary and only AI chat surface exposed by Script Kit GPUI.
- `Tab` and `Shift+Tab` route into `AppView::AcpChatView`, while detached ACP windows reuse the same thread and automation contracts.
- Compatibility-named helpers and types such as `tab_ai_mode.rs`, `CaptureContextOptions::tab_ai_submit()`, and `TabAiContextBlob` still back ACP Chat context capture.
- The detached ACP window and ACP view modules define the live chat surface, while the compatibility-named context tests continue to lock the schema contract.

## Key Files
- `README.md` — Project overview. High-level product positioning, setup, prompt APIs, configuration, and built-in capabilities.
- `src/ai/harness/mod.rs` — Tab AI harness. Harness configuration and context formatting for the external CLI flow.
- `src/ai/tab_context.rs` — Tab AI context types. Tab AI context blob, target audit, UI snapshot, and clipboard context types.
- `tests/tab_ai_context.rs` — Tab AI context tests. Integration tests for the Tab AI context blob schema and serialization.

## Source Documents
- [raw/c9fbc3ca/README.md](../raw/c9fbc3ca/README.md)
- [raw/c9fbc3ca/src/ai/harness/mod.rs](../raw/c9fbc3ca/src/ai/harness/mod.rs)
- [raw/c9fbc3ca/src/ai/tab_context.rs](../raw/c9fbc3ca/src/ai/tab_context.rs)
- [raw/c9fbc3ca/tests/tab_ai_context.rs](../raw/c9fbc3ca/tests/tab_ai_context.rs)

## Related Pages
- [project-overview](./project-overview.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [verification-and-testing](./verification-and-testing.md)

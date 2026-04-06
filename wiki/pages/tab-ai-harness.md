---
title: "ACP Chat"
slug: "tab-ai-harness"
sourceSnapshot: "fa760732"
sourceDocuments:
  - "raw/fa760732/README.md"
  - "raw/fa760732/src/ai/acp/view.rs"
  - "raw/fa760732/src/ai/acp/chat_window.rs"
  - "raw/fa760732/src/ai/harness/mod.rs"
  - "raw/fa760732/src/ai/tab_context.rs"
  - "raw/fa760732/tests/tab_ai_context.rs"
relatedPages:
  - "project-overview"
  - "ai-context-and-mcp"
  - "verification-and-testing"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-06T18:08:02.732Z"
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
- `src/ai/acp/view.rs` — ACP chat view. Primary ACP Chat view, composer, picker integration, and threaded conversation rendering.
- `src/ai/acp/chat_window.rs` — Detached ACP chat window. Detached ACP Chat window lifecycle, focus, and automation registration.
- `src/ai/harness/mod.rs` — ACP chat compatibility harness. Compatibility-layer harness configuration and context formatting still used by ACP Chat plumbing.
- `src/ai/tab_context.rs` — ACP chat context types. Compatibility-named `tab_ai_*` context blob, target audit, UI snapshot, and clipboard context types that back ACP Chat.
- `tests/tab_ai_context.rs` — ACP chat context tests. Integration tests for the compatibility-named ACP Chat context blob schema and serialization.

## Source Documents
- [raw/fa760732/README.md](../raw/fa760732/README.md)
- [raw/fa760732/src/ai/acp/view.rs](../raw/fa760732/src/ai/acp/view.rs)
- [raw/fa760732/src/ai/acp/chat_window.rs](../raw/fa760732/src/ai/acp/chat_window.rs)
- [raw/fa760732/src/ai/harness/mod.rs](../raw/fa760732/src/ai/harness/mod.rs)
- [raw/fa760732/src/ai/tab_context.rs](../raw/fa760732/src/ai/tab_context.rs)
- [raw/fa760732/tests/tab_ai_context.rs](../raw/fa760732/tests/tab_ai_context.rs)

## Related Pages
- [project-overview](./project-overview.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [verification-and-testing](./verification-and-testing.md)

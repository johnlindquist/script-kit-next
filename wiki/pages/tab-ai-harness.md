---
title: "ACP Chat"
slug: "tab-ai-harness"
sourceSnapshot: "49ebc9f4"
sourceDocuments:
  - "raw/49ebc9f4/README.md"
  - "raw/49ebc9f4/src/ai/acp/view.rs"
  - "raw/49ebc9f4/src/ai/acp/chat_window.rs"
  - "raw/49ebc9f4/src/ai/harness/mod.rs"
  - "raw/49ebc9f4/src/ai/tab_context.rs"
  - "raw/49ebc9f4/tests/tab_ai_context.rs"
relatedPages:
  - "project-overview"
  - "ai-context-and-mcp"
  - "verification-and-testing"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-11T01:39:53.007Z"
---

# ACP Chat

The primary and only AI chat surface, the ACP chat runtime, and the compatibility-named context plumbing that still backs it.

## Key Facts
- ACP Chat is the primary and only AI chat surface exposed by Script Kit GPUI.
- `Tab` and `Shift+Tab` route into `AppView::AcpChatView`, while detached ACP windows reuse the same thread and automation contracts.
- Compatibility-named helpers and types such as `tab_ai_mode.rs`, `CaptureContextOptions::tab_ai_submit()`, and `TabAiContextBlob` still back ACP Chat context capture.
- ACP already contains the main implementation seams for Context Portalling: inline mention parsing, context preview, portal launching, and replaceable typed mention tokens.
- The detached ACP window and ACP view modules define the live chat surface, while the compatibility-named context tests continue to lock the schema contract.

## Key Files
- `README.md` — Project overview. High-level product positioning, setup, prompt APIs, configuration, and built-in capabilities.
- `src/ai/acp/view.rs` — ACP chat view. Primary ACP Chat view, composer, picker integration, and threaded conversation rendering.
- `src/ai/acp/chat_window.rs` — Detached ACP chat window. Detached ACP Chat window lifecycle, focus, and automation registration.
- `src/ai/harness/mod.rs` — ACP chat compatibility harness. Compatibility-layer harness configuration and context formatting still used by ACP Chat plumbing.
- `src/ai/tab_context.rs` — ACP chat context types. Compatibility-named `tab_ai_*` context blob, target audit, UI snapshot, and clipboard context types that back ACP Chat.
- `tests/tab_ai_context.rs` — ACP chat context tests. Integration tests for the compatibility-named ACP Chat context blob schema and serialization.

## Source Documents
- [raw/49ebc9f4/README.md](../raw/49ebc9f4/README.md)
- [raw/49ebc9f4/src/ai/acp/view.rs](../raw/49ebc9f4/src/ai/acp/view.rs)
- [raw/49ebc9f4/src/ai/acp/chat_window.rs](../raw/49ebc9f4/src/ai/acp/chat_window.rs)
- [raw/49ebc9f4/src/ai/harness/mod.rs](../raw/49ebc9f4/src/ai/harness/mod.rs)
- [raw/49ebc9f4/src/ai/tab_context.rs](../raw/49ebc9f4/src/ai/tab_context.rs)
- [raw/49ebc9f4/tests/tab_ai_context.rs](../raw/49ebc9f4/tests/tab_ai_context.rs)

## Related Pages
- [project-overview](./project-overview.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [verification-and-testing](./verification-and-testing.md)

## Context Portalling In ACP Chat
ACP Chat is the first natural host for Context Portalling because it already has:

- inline mention parsing and canonicalization in `src/ai/context_mentions/mod.rs`
- mention and slash picker infrastructure in `src/ai/window/context_picker/`
- lightweight preview metadata in `src/ai/window/context_preview.rs`
- portal open/return logic in `src/app_impl/attachment_portal.rs`

The planned UX is:

1. focus an existing mention in the composer
2. preview what it points at
3. explicitly reopen that mention in its source portal
4. replace the target in place or cancel
5. return to the same composer position

Two constraints matter here:

- editor keys such as `Enter` and `Tab` must continue to behave as normal composer keys
- every portal session needs a breadcrumbed return path back to the composer

Notes-hosted ACP should preserve the same mental model even when only some portal kinds are available locally. Unsupported portal kinds should never feel like dead ends.

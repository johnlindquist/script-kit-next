---
title: "AI Context and MCP"
slug: "ai-context-and-mcp"
sourceSnapshot: "c9fbc3ca"
sourceDocuments:
  - "raw/c9fbc3ca/docs/AI_CONTEXT_AWARENESS_PATTERNS.md"
  - "raw/c9fbc3ca/src/context_snapshot/types.rs"
  - "raw/c9fbc3ca/src/mcp_resources/mod.rs"
  - "raw/c9fbc3ca/src/ai/message_parts.rs"
  - "raw/c9fbc3ca/tests/context_snapshot.rs"
  - "raw/c9fbc3ca/tests/context_part_resolution.rs"
relatedPages:
  - "protocol-and-introspection"
  - "tab-ai-harness"
  - "verification-and-testing"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-04T19:40:47.586Z"
---

# AI Context and MCP

Schema-versioned desktop context capture, MCP resources, and typed context-part resolution.

## Key Facts
- kit://context and kit://context/schema expose deterministic desktop context as MCP resources.
- CaptureContextOptions defines stable profiles such as all, recommendation, minimal, tab_ai_submit, and tab_ai.
- AiContextPart resolves typed attachments such as ResourceUri and FilePath into prompt blocks while tracking failures in a receipt.
- The integration tests already lock the context and context-part contracts, which makes them good seed material for wiki pages.

## Key Files
- `docs/AI_CONTEXT_AWARENESS_PATTERNS.md` — AI context awareness patterns. Research on context engineering, implicit versus explicit context, and context-provider design patterns.
- `src/context_snapshot/types.rs` — Context snapshot types. Schema-versioned desktop context types and capture profiles.
- `src/mcp_resources/mod.rs` — MCP resource registry. Current MCP resources including kit://context and kit://context/schema.
- `src/ai/message_parts.rs` — AI message parts. Typed context parts and deterministic resolution receipts.
- `tests/context_snapshot.rs` — Context snapshot tests. Integration tests that lock the kit://context contract.
- `tests/context_part_resolution.rs` — Context part resolution tests. Integration tests for ResourceUri and FilePath resolution behavior.

## Source Documents
- [raw/c9fbc3ca/docs/AI_CONTEXT_AWARENESS_PATTERNS.md](../raw/c9fbc3ca/docs/AI_CONTEXT_AWARENESS_PATTERNS.md)
- [raw/c9fbc3ca/src/context_snapshot/types.rs](../raw/c9fbc3ca/src/context_snapshot/types.rs)
- [raw/c9fbc3ca/src/mcp_resources/mod.rs](../raw/c9fbc3ca/src/mcp_resources/mod.rs)
- [raw/c9fbc3ca/src/ai/message_parts.rs](../raw/c9fbc3ca/src/ai/message_parts.rs)
- [raw/c9fbc3ca/tests/context_snapshot.rs](../raw/c9fbc3ca/tests/context_snapshot.rs)
- [raw/c9fbc3ca/tests/context_part_resolution.rs](../raw/c9fbc3ca/tests/context_part_resolution.rs)

## Related Pages
- [protocol-and-introspection](./protocol-and-introspection.md)
- [tab-ai-harness](./tab-ai-harness.md)
- [verification-and-testing](./verification-and-testing.md)

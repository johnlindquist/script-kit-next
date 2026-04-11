---
title: "AI Context and MCP"
slug: "ai-context-and-mcp"
sourceSnapshot: "49ebc9f4"
sourceDocuments:
  - "raw/49ebc9f4/docs/AI_CONTEXT_AWARENESS_PATTERNS.md"
  - "raw/49ebc9f4/src/context_snapshot/types.rs"
  - "raw/49ebc9f4/src/mcp_resources/mod.rs"
  - "raw/49ebc9f4/src/ai/message_parts.rs"
  - "raw/49ebc9f4/tests/context_snapshot.rs"
  - "raw/49ebc9f4/tests/context_part_resolution.rs"
relatedPages:
  - "protocol-and-introspection"
  - "tab-ai-harness"
  - "verification-and-testing"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-11T01:39:53.007Z"
---

# AI Context and MCP

Schema-versioned desktop context capture, MCP resources, and typed context-part resolution.

## Key Facts
- kit://context and kit://context/schema expose deterministic desktop context as MCP resources.
- CaptureContextOptions defines stable profiles such as all, recommendation, minimal, tab_ai_submit, and tab_ai.
- AiContextPart resolves typed attachments such as ResourceUri and FilePath into prompt blocks while tracking failures in a receipt.
- Context Portalling is the planned UX model for treating inline mentions as navigable pointers backed by `AiContextPart`, with preview, portal re-entry, and return-to-editor semantics.
- The integration tests already lock the context and context-part contracts, which makes them good seed material for wiki pages.

## Key Files
- `docs/AI_CONTEXT_AWARENESS_PATTERNS.md` — AI context awareness patterns. Research on context engineering, implicit versus explicit context, and context-provider design patterns.
- `src/context_snapshot/types.rs` — Context snapshot types. Schema-versioned desktop context types and capture profiles.
- `src/mcp_resources/mod.rs` — MCP resource registry. Current MCP resources including kit://context and kit://context/schema.
- `src/ai/message_parts.rs` — AI message parts. Typed context parts and deterministic resolution receipts.
- `tests/context_snapshot.rs` — Context snapshot tests. Integration tests that lock the kit://context contract.
- `tests/context_part_resolution.rs` — Context part resolution tests. Integration tests for ResourceUri and FilePath resolution behavior.

## Source Documents
- [raw/49ebc9f4/docs/AI_CONTEXT_AWARENESS_PATTERNS.md](../raw/49ebc9f4/docs/AI_CONTEXT_AWARENESS_PATTERNS.md)
- [raw/49ebc9f4/src/context_snapshot/types.rs](../raw/49ebc9f4/src/context_snapshot/types.rs)
- [raw/49ebc9f4/src/mcp_resources/mod.rs](../raw/49ebc9f4/src/mcp_resources/mod.rs)
- [raw/49ebc9f4/src/ai/message_parts.rs](../raw/49ebc9f4/src/ai/message_parts.rs)
- [raw/49ebc9f4/tests/context_snapshot.rs](../raw/49ebc9f4/tests/context_snapshot.rs)
- [raw/49ebc9f4/tests/context_part_resolution.rs](../raw/49ebc9f4/tests/context_part_resolution.rs)

## Related Pages
- [protocol-and-introspection](./protocol-and-introspection.md)
- [tab-ai-harness](./tab-ai-harness.md)
- [verification-and-testing](./verification-and-testing.md)

## Context Portalling
Context Portalling is a planned layer on top of the existing context-part system. The current code already provides the raw ingredients:

- canonical inline tokens
- `AiContextPart` as the stable attachment model
- synchronous context preview metadata
- portal-backed target selection

The design direction is to treat an existing inline mention as a pointer the user can revisit, preview, and replace without deleting the token and starting over.

The most important constraint is editor safety: in text surfaces, `Enter` and `Tab` must keep their normal editing meanings. Portalling is therefore an explicit secondary action, while preview can remain passive and focus-safe.

The `AiContextPart` model is the right contract for this feature because it preserves both identity and resolution semantics independently from the inline text. That separation is what makes replace-in-place, preview hydration, and breadcrumbed return flows feasible across ACP, Notes, and future text surfaces.

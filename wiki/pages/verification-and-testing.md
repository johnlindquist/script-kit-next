---
title: "Verification and Testing"
slug: "verification-and-testing"
sourceSnapshot: "fa760732"
sourceDocuments:
  - "raw/fa760732/CLAUDE.md"
  - "raw/fa760732/tests/context_snapshot.rs"
  - "raw/fa760732/tests/context_part_resolution.rs"
  - "raw/fa760732/tests/tab_ai_context.rs"
relatedPages:
  - "project-overview"
  - "ai-context-and-mcp"
  - "tab-ai-harness"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-06T18:08:02.732Z"
---

# Verification and Testing

The repository's required verification gate and the existing contract tests around the AI/context subsystems.

## Key Facts
- Every code change must pass cargo check, cargo clippy --lib -- -D warnings, and cargo nextest run --lib before success is reported.
- Logic changes require log inspection with SCRIPT_KIT_AI_LOG=1.
- UI changes require a screenshot and reading the PNG to verify visual behavior.
- The ACP Chat/context subsystems already have integration tests that encode the expected wire contracts.

## Key Files
- `CLAUDE.md` — Repository operating contract. Scope rules, verification gate, architecture quick ref, AI context, and design principles.
- `tests/context_snapshot.rs` — Context snapshot tests. Integration tests that lock the kit://context contract.
- `tests/context_part_resolution.rs` — Context part resolution tests. Integration tests for ResourceUri and FilePath resolution behavior.
- `tests/tab_ai_context.rs` — ACP chat context tests. Integration tests for the compatibility-named ACP Chat context blob schema and serialization.

## Source Documents
- [raw/fa760732/CLAUDE.md](../raw/fa760732/CLAUDE.md)
- [raw/fa760732/tests/context_snapshot.rs](../raw/fa760732/tests/context_snapshot.rs)
- [raw/fa760732/tests/context_part_resolution.rs](../raw/fa760732/tests/context_part_resolution.rs)
- [raw/fa760732/tests/tab_ai_context.rs](../raw/fa760732/tests/tab_ai_context.rs)

## Related Pages
- [project-overview](./project-overview.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [tab-ai-harness](./tab-ai-harness.md)

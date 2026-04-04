---
title: "Verification and Testing"
slug: "verification-and-testing"
sourceSnapshot: "4be166ea"
sourceDocuments:
  - "raw/4be166ea/CLAUDE.md"
  - "raw/4be166ea/tests/context_snapshot.rs"
  - "raw/4be166ea/tests/context_part_resolution.rs"
  - "raw/4be166ea/tests/tab_ai_context.rs"
relatedPages:
  - "project-overview"
  - "ai-context-and-mcp"
  - "tab-ai-harness"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-04T18:48:29.295Z"
---

# Verification and Testing

The repository's required verification gate and the existing contract tests around the AI/context subsystems.

## Key Facts
- Every code change must pass cargo check, cargo clippy --lib -- -D warnings, and cargo nextest run --lib before success is reported.
- Logic changes require log inspection with SCRIPT_KIT_AI_LOG=1.
- UI changes require a screenshot and reading the PNG to verify visual behavior.
- The AI/context subsystems already have integration tests that encode the expected wire contracts.

## Key Files
- `CLAUDE.md` — Repository operating contract. Scope rules, verification gate, architecture quick ref, AI context, and design principles.
- `tests/context_snapshot.rs` — Context snapshot tests. Integration tests that lock the kit://context contract.
- `tests/context_part_resolution.rs` — Context part resolution tests. Integration tests for ResourceUri and FilePath resolution behavior.
- `tests/tab_ai_context.rs` — Tab AI context tests. Integration tests for the Tab AI context blob schema and serialization.

## Source Documents
- [raw/4be166ea/CLAUDE.md](../raw/4be166ea/CLAUDE.md)
- [raw/4be166ea/tests/context_snapshot.rs](../raw/4be166ea/tests/context_snapshot.rs)
- [raw/4be166ea/tests/context_part_resolution.rs](../raw/4be166ea/tests/context_part_resolution.rs)
- [raw/4be166ea/tests/tab_ai_context.rs](../raw/4be166ea/tests/tab_ai_context.rs)

## Related Pages
- [project-overview](./project-overview.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [tab-ai-harness](./tab-ai-harness.md)

---
title: "Architecture"
slug: "architecture"
sourceSnapshot: "4be166ea"
sourceDocuments:
  - "raw/4be166ea/CLAUDE.md"
  - "raw/4be166ea/README.md"
  - "raw/4be166ea/GPUI.md"
relatedPages:
  - "project-overview"
  - "protocol-and-introspection"
  - "ai-context-and-mcp"
  - "design-principles"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-04T18:48:29.295Z"
---

# Architecture

How the application is split across Rust modules, include!()-driven main sections, and protocol surfaces.

## Key Facts
- Core app logic is split across src/main_sections, src/app_impl, src/app_execute, and src/render_* rather than one monolithic app file.
- Some directories are include!()-injected into main.rs and therefore follow stricter file rules than normal Rust modules.
- The runtime uses bidirectional JSONL over stdin/stdout between Bun scripts and the Rust application shell.
- GPUI keyboard handling has separate action dispatch and raw key event dispatch cycles, which matters for prompt behavior.

## Key Files
- `CLAUDE.md` — Repository operating contract. Scope rules, verification gate, architecture quick ref, AI context, and design principles.
- `README.md` — Project overview. High-level product positioning, setup, prompt APIs, configuration, and built-in capabilities.
- `GPUI.md` — GPUI event dispatch architecture. Keyboard dispatch order, action propagation, raw key events, and common event pitfalls.

## Source Documents
- [raw/4be166ea/CLAUDE.md](../raw/4be166ea/CLAUDE.md)
- [raw/4be166ea/README.md](../raw/4be166ea/README.md)
- [raw/4be166ea/GPUI.md](../raw/4be166ea/GPUI.md)

## Related Pages
- [project-overview](./project-overview.md)
- [protocol-and-introspection](./protocol-and-introspection.md)
- [ai-context-and-mcp](./ai-context-and-mcp.md)
- [design-principles](./design-principles.md)

---
title: "Project Overview"
slug: "project-overview"
sourceSnapshot: "c9fbc3ca"
sourceDocuments:
  - "raw/c9fbc3ca/CLAUDE.md"
  - "raw/c9fbc3ca/README.md"
relatedPages:
  - "architecture"
  - "design-principles"
  - "verification-and-testing"
  - "tab-ai-harness"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-04T19:40:47.586Z"
---

# Project Overview

What Script Kit GPUI is, how it is positioned, and the major product surfaces it exposes.

## Key Facts
- Script Kit GPUI combines a Rust GPUI shell with a Bun-powered TypeScript script runner and SDK.
- The rewrite is intentionally not a drop-in replacement for legacy Script Kit; the SDK is prompt-centric and expects users to bring their own libraries.
- The product is optimized for keyboard-first launcher workflows and includes built-ins such as clipboard history, app launcher, window switcher, notes, and ACP Chat.
- CLAUDE.md is the canonical repository operating contract for contributors and agents.

## Key Files
- `CLAUDE.md` — Repository operating contract. Scope rules, verification gate, architecture quick ref, AI context, and design principles.
- `README.md` — Project overview. High-level product positioning, setup, prompt APIs, configuration, and built-in capabilities.

## Source Documents
- [raw/c9fbc3ca/CLAUDE.md](../raw/c9fbc3ca/CLAUDE.md)
- [raw/c9fbc3ca/README.md](../raw/c9fbc3ca/README.md)

## Related Pages
- [architecture](./architecture.md)
- [design-principles](./design-principles.md)
- [verification-and-testing](./verification-and-testing.md)
- [tab-ai-harness](./tab-ai-harness.md)

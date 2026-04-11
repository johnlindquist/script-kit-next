---
title: "Design Principles"
slug: "design-principles"
sourceSnapshot: "49ebc9f4"
sourceDocuments:
  - "raw/49ebc9f4/CLAUDE.md"
  - "raw/49ebc9f4/README.md"
relatedPages:
  - "project-overview"
  - "architecture"
generatedBy: "scripts/wiki/ingest.ts"
generatedAt: "2026-04-11T01:39:53.007Z"
---

# Design Principles

The product's launcher UX rules, chrome discipline, and macOS-native interaction principles.

## Key Facts
- The footer should show at most three affordances: Run, Actions, and AI.
- Discovery lives in the Actions dialog rather than persistent chrome.
- The visual system favors ultra-low-opacity chrome, strong keyboard-first affordances, and native macOS vibrancy.
- Context Portalling should stay editor-first: passive preview is fine, but opening or replacing a mention must be explicit and must preserve a clear route back.
- The design direction is fast, focused, minimal, and closer to Raycast than a web-style dashboard.

## Key Files
- `CLAUDE.md` — Repository operating contract. Scope rules, verification gate, architecture quick ref, AI context, and design principles.
- `README.md` — Project overview. High-level product positioning, setup, prompt APIs, configuration, and built-in capabilities.

## Source Documents
- [raw/49ebc9f4/CLAUDE.md](../raw/49ebc9f4/CLAUDE.md)
- [raw/49ebc9f4/README.md](../raw/49ebc9f4/README.md)

## Related Pages
- [project-overview](./project-overview.md)
- [architecture](./architecture.md)

## Context Portalling Principles
The planned Context Portalling UX follows the same product direction as the rest of Script Kit GPUI:

- keep the default surface quiet
- reveal more power only when the user is already on the relevant object
- preserve keyboard speed without stealing core editor behavior

For mentions, that means:

- they remain text-like until focused
- passive preview can appear without pulling focus
- opening a portal is an explicit action
- breadcrumbs and “return to editor” affordances must always be present once the user leaves the main writing surface

This keeps the feature learnable for casual use while still supporting the power-user loop of jump, inspect, replace, and return.


## Slug

`script-metadata-scriptlets-atlas`

## Project Brief

Script Kit GPUI is a Rust/GPUI desktop runtime with a TypeScript SDK and plugin-scoped `~/.scriptkit` workspace. The feature map is intended for humans and AI agents to fully understand capabilities, states, interactions, script APIs, runtime routing, MCP resources, tests, receipts, and gaps.


- `removed-docs/` is the architecture knowledge graph. Feature-map work must stay aligned with existing `removed-docs` pages and run `source checks`.
- Preserve the complete Oracle session output locally under `feature-map/raw-oracle/<feature-id>/`.
- The local agent writes maintained feature chapters under `feature-map/features/`; Oracle should return text only.

## Feature Scope

Map feature `032 Script Metadata, Scriptlets, and Execution Catalog`.


- Plugin-scoped script discovery under `~/.scriptkit/plugins/*/scripts`.
- Scriptlet discovery under `~/.scriptkit/plugins/*/scriptlets/*.md`.
- Tests and verification surfaces.

## Current Evidence


- `removed-docs` says script discovery is plugin-based, with default personal plugin `~/.scriptkit/plugins/main/scripts/` and scriptlets under `~/.scriptkit/plugins/*/scriptlets/*.md`.
- `src/scripts/loader.rs` loads `.ts` and `.js` scripts from discovered plugin roots, extracts full metadata/schema, reads body text for content search, attaches plugin identity, sorts by name, and wraps validation through `read_scripts_report`.
- `src/scripts/metadata.rs` merges typed metadata with legacy comment metadata, preferring typed values when present.
- `src/metadata_parser/mod.rs` parses `metadata = { ... }` object literals into `TypedMetadata` and supports aliases such as `expand` / `snippet` for `keyword`.
- `src/scripts/validation.rs` excludes scripts with duplicate shortcut/alias/keyword/trigger bindings from the kept catalog and emits schema-versioned diagnostics.
- `src/scripts/scriptlet_loader/loading.rs` loads plugin scriptlet markdown, skips `.actions.md`, parses scriptlets, attaches plugin identity and anchor paths, and sorts by group/name.
- `src/scriptlets/mod.rs` provides the richer parser for markdown bundles, frontmatter, H1/H2/H3 behavior, actions, safety checks, global prepends, nested fences, inputs, conditionals, and companion shared actions.
- `src/executor/scriptlet.rs` executes scriptlets by tool type and enforces environment and platform boundaries.
- `src/mcp_resources/mod.rs` exposes the script/scriptlet/failed-scripts/sdk-reference/script-template resource envelopes and UI-facing catalog helpers.

## Bundle Map


- Script loading/metadata/validation/scheduling/search/grouping owners.
- Scriptlet parser, scriptlet loader, metadata parser, executor, and runner owners.
- MCP resources, plugin discovery/manifest/types, setup bootstrap, menu syntax execution, prompt run-script handling.
- Script/scriptlet/resource/content/action smoke and source-audit tests.

## Deliverable

Return an operator-grade feature atlas for `032 Script Metadata, Scriptlets, and Execution Catalog`.


9. User stories and agent stories.
12. Known risks/gaps/ambiguities with exact file/function references.
13. Suggested maintained chapter content suitable for `feature-map/features/032-script-metadata-scriptlets.md`.

Be comprehensive. Prefer exact file/function references from the bundle. Explicitly mark inferred behavior when the bundle does not directly prove it.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.

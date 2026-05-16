# Oracle Prompt: 032 Script Metadata, Scriptlets, and Execution Catalog

## Slug

`script-metadata-scriptlets-atlas`

## Project Brief

Script Kit GPUI is a Rust/GPUI desktop runtime with a TypeScript SDK and plugin-scoped `~/.scriptkit` workspace. The feature map is intended for humans and AI agents to fully understand capabilities, states, interactions, script APIs, runtime routing, MCP resources, tests, receipts, and gaps.

Repo rules:

- `lat.md/` is the architecture knowledge graph. Feature-map work must stay aligned with existing `lat.md` pages and run `lat check`.
- Preserve the complete Oracle session output locally under `feature-map/raw-oracle/<feature-id>/`.
- The local agent writes maintained feature chapters under `feature-map/features/`; Oracle should return text only.

## Feature Scope

Map feature `032 Script Metadata, Scriptlets, and Execution Catalog`.

This feature includes:

- Plugin-scoped script discovery under `~/.scriptkit/plugins/*/scripts`.
- Script metadata extraction: typed `metadata = { ... }`, legacy comment metadata, schema extraction, schedule metadata, shortcuts, aliases, keywords, triggers, hidden/background/system/fallback flags.
- Script validation and exclusion: duplicate `shortcut`, `alias`, `keyword`, and `trigger` collisions; failed-script diagnostics; kept catalog behavior.
- Scriptlet discovery under `~/.scriptkit/plugins/*/scriptlets/*.md`.
- Scriptlet markdown parsing: H1 groups, H2 scriptlets, H3 actions, bundle frontmatter, codefence metadata/schema, HTML comment metadata, companion `.actions.md`, tool normalization, named inputs, conditionals, global prepends, and path/file-size safety.
- Scriptlet execution: shell/interpreter/TypeScript/template/open/edit/paste/type/submit/transform behavior, environment allowlist, Bun temp execution, macOS-only boundaries.
- MCP and in-app resources: `kit://scripts`, `kit://scriptlets`, `kit://failed-scripts`, `kit://sdk-reference`, `kit://script-templates`, SDK Reference UI, script template catalog, New Script and New Script From Template flows.
- Launcher integration and search: script/scriptlet rows, script issue row, content search, preview cache, grouping, actions, and menu syntax command/capture invocation.
- Tests and verification surfaces.

## Current Evidence

Local inspection found:

- `lat.md/scripting.md` says script discovery is plugin-based, with default personal plugin `~/.scriptkit/plugins/main/scripts/` and scriptlets under `~/.scriptkit/plugins/*/scriptlets/*.md`.
- `src/scripts/loader.rs` loads `.ts` and `.js` scripts from discovered plugin roots, extracts full metadata/schema, reads body text for content search, attaches plugin identity, sorts by name, and wraps validation through `read_scripts_report`.
- `src/scripts/metadata.rs` merges typed metadata with legacy comment metadata, preferring typed values when present.
- `src/metadata_parser/mod.rs` parses `metadata = { ... }` object literals into `TypedMetadata` and supports aliases such as `expand` / `snippet` for `keyword`.
- `src/scripts/validation.rs` excludes scripts with duplicate shortcut/alias/keyword/trigger bindings from the kept catalog and emits schema-versioned diagnostics.
- `src/scripts/scheduling.rs` separately scans scripts for `// Cron:` and `// Schedule:` comments.
- `src/scripts/scriptlet_loader/loading.rs` loads plugin scriptlet markdown, skips `.actions.md`, parses scriptlets, attaches plugin identity and anchor paths, and sorts by group/name.
- `src/scriptlets/mod.rs` provides the richer parser for markdown bundles, frontmatter, H1/H2/H3 behavior, actions, safety checks, global prepends, nested fences, inputs, conditionals, and companion shared actions.
- `src/executor/scriptlet.rs` executes scriptlets by tool type and enforces environment and platform boundaries.
- `src/mcp_resources/mod.rs` exposes the script/scriptlet/failed-scripts/sdk-reference/script-template resource envelopes and UI-facing catalog helpers.

## Bundle Map

Attached bundle includes:

- Process and repo rules: `AGENTS.md`, `CLAUDE.md`, `.goals/feature_map.md`.
- Skills and design context: `sdk-script-execution`, legacy `script-kit-scripting`, `lat.md/scripting.md`, `lat.md/workspace.md`, `lat.md/menu-syntax.md`, `lat.md/protocol.md`, `lat.md/verification.md`.
- Script loading/metadata/validation/scheduling/search/grouping owners.
- Scriptlet parser, scriptlet loader, metadata parser, executor, and runner owners.
- MCP resources, plugin discovery/manifest/types, setup bootstrap, menu syntax execution, prompt run-script handling.
- Script/scriptlet/resource/content/action smoke and source-audit tests.

## Deliverable

Return an operator-grade feature atlas for `032 Script Metadata, Scriptlets, and Execution Catalog`.

Use this shape:

1. Capability summary: what users, script authors, and agents can do.
2. Workspace and discovery model: plugin roots, scripts, scriptlets, skills/agents boundaries.
3. Script metadata contract: typed vs legacy comments, schema, schedule metadata, typed fields, precedence, and gaps.
4. Script validation contract: duplicate binding detection, kept/excluded catalog, failed-script diagnostics, issue row, MCP resource.
5. Scriptlet bundle contract: markdown structure, metadata formats, actions, shared actions, tool normalization, input/conditional/global-prepend behavior, safety checks.
6. Execution contract: scripts through Bun/SDK preload, scriptlets by tool, environment, temp files, macOS-only tools, output/error semantics.
7. Launcher/search/actions contract: grouping, content search, preview, issue row, actions, frecency, command ids, menu syntax invocation.
8. MCP/resource/agent contract: `kit://scripts`, `kit://scriptlets`, `kit://failed-scripts`, `kit://sdk-reference`, `kit://script-templates`, harness workflow, UI catalog parity.
9. User stories and agent stories.
10. State model and lifecycle: startup load, refresh, validation, hidden/excluded scripts, execution sessions, scriptlet action invocation.
11. Verification map: existing tests, smoke tests, source audits, recommended gates.
12. Known risks/gaps/ambiguities with exact file/function references.
13. Suggested maintained chapter content suitable for `feature-map/features/032-script-metadata-scriptlets.md`.

Be comprehensive. Prefer exact file/function references from the bundle. Explicitly mark inferred behavior when the bundle does not directly prove it.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.

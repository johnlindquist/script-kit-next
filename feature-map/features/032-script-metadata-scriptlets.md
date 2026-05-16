# 032 Script Metadata, Scriptlets, and Execution Catalog

This chapter maps plugin-scoped script discovery, script metadata extraction, scriptlet bundle parsing, duplicate binding validation, scriptlet execution, launcher catalog integration, and resource exposure for humans and agents.

Raw Oracle reference: [answer](../raw-oracle/032-script-metadata-scriptlets/answer.md), [prompt](../raw-oracle/032-script-metadata-scriptlets/prompt.md), [bundle map](../raw-oracle/032-script-metadata-scriptlets/bundle-map.md), [full log](../raw-oracle/032-script-metadata-scriptlets/output.log), [session metadata](../raw-oracle/032-script-metadata-scriptlets/session.json).

## Executive Summary

Script Kit GPUI's catalog layer answers what executable things exist, how they are described, which ones are safe to expose, and what agents can inspect without scraping UI state.

Scripts are discovered from plugin-scoped `scripts/` directories. Scriptlets are discovered from plugin-scoped Markdown bundles under `scriptlets/`. The default personal plugin is `~/.scriptkit/plugins/main/`, so personal scripts live under `~/.scriptkit/plugins/main/scripts/` and personal scriptlet bundles live under `~/.scriptkit/plugins/main/scriptlets/*.md`.

Scripts support typed `metadata = { ... }` declarations, legacy `// Key: Value` comments, schema declarations, schedule comments, body text for content search, and binding fields such as `shortcut`, `alias`, `keyword`, and `trigger`. Scriptlets support Markdown headings, HTML comment metadata, fenced metadata and schema blocks, tool fences, groups, actions, shared companion `.actions.md` files, and named inputs.

Validation currently proves duplicate binding exclusion for scripts. Scripts with duplicate `shortcut`, `alias`, `keyword`, or `trigger` bindings are excluded from the kept catalog by `src/scripts/validation.rs#validate_script_catalog`; diagnostics are exposed in the launcher and through `kit://failed-scripts`.

Agents can inspect the same catalog families through schema-versioned resources:

- `kit://scripts`
- `kit://scriptlets`
- `kit://failed-scripts`
- `kit://sdk-reference`
- `kit://script-templates`

## User Capabilities

| Capability | Behavior |
|---|---|
| Browse scripts | The launcher lists loaded scripts from plugin `scripts/` directories. |
| Browse scriptlets | The launcher lists loaded Markdown scriptlets from plugin `scriptlets/` directories. |
| Run scripts and scriptlets | Executables can be reached through search, actions, shortcuts, aliases, keywords, command syntax, and scriptlet actions depending on metadata. |
| Diagnose excluded scripts | A pinned Script Issues row appears when validation excludes scripts; Enter opens `AppView::ScriptIssuesView`, and Cmd+C copies Markdown diagnostics. |
| Read SDK Reference | The in-app SDK Reference uses the same Rust-owned reference data as `kit://sdk-reference`. |
| Create scripts from templates | The new-script/template surface uses the same template catalog as `kit://script-templates`. |

## Author Capabilities

| Author need | Supported path |
|---|---|
| Personal scripts | `~/.scriptkit/plugins/main/scripts/*.ts` or `*.js`. |
| Plugin scripts | `~/.scriptkit/plugins/<plugin-id>/scripts/*.ts` or `*.js`. |
| Scriptlet bundles | `~/.scriptkit/plugins/<plugin-id>/scriptlets/*.md`. |
| Typed script metadata | `export const metadata = { ... }` or equivalent top-level `metadata = { ... }` declaration. |
| Legacy script metadata | Leading `// Name:`, `// Description:`, `// Icon:`, `// Alias:`, and `// Shortcut:` comments. |
| Script schema | Top-level `schema = { ... }` declaration parsed by the schema parser. |
| Schedule comments | `// Cron:` and `// Schedule:` comments in the first 30 lines. |
| Scriptlet metadata | HTML comments and fenced `metadata` blocks. |
| Scriptlet actions | H3 action headings and shared companion `.actions.md` files. |
| Scriptlet tools | Shell, TypeScript-like, interpreter, template, transform, open, edit, paste, type, and submit tools. |

## Workspace Model

Plugin discovery is rooted at `~/.scriptkit/plugins/`. Each child directory is a plugin root discovered by `src/plugins/discovery.rs#discover_plugins_in`; the plugin record carries `plugin.id`, `plugin.root`, and manifest data, then the index is sorted by id.

Important plugin helpers include:

- `src/plugins/discovery.rs#discover_plugins`
- `src/plugins/discovery.rs#plugin_scriptlets_dir`
- `src/plugins/discovery.rs#plugin_skills_dir`
- `src/plugins/discovery.rs#plugin_agents_dir`

Scripts, scriptlets, skills, and agents are sibling plugin-scoped capabilities. This chapter only covers scripts and scriptlets.

Do not document `~/.scriptkit/scripts/` as the active scripts root. `tests/agent_workspace_contract.rs#test_resource_definitions_use_plugin_scoped_discovery` pins plugin-scoped wording and checks that resource descriptions mention `plugins/main/scripts`.

## Script Discovery

`src/scripts/loader.rs#read_scripts` is the base script catalog loader:

- Discovers plugins through `crate::plugins::discover_plugins`.
- Iterates each plugin's scripts directory.
- Loads `.ts` and `.js` files.
- Extracts metadata and schema.
- Reads body text for content search.
- Attaches plugin identity: `plugin_id`, `plugin_title`, and `kit_name`.
- Sorts scripts by name.
- Returns `Vec<Arc<Script>>`.

`src/scripts/loader.rs#read_scripts_from_dir` reads one scripts directory and uses Rayon parallel loading. `src/scripts/loader.rs#read_scripts_report` wraps `read_scripts()` with `src/scripts/validation.rs#validate_script_catalog`, returning a `ScriptCatalogReport` with both the kept catalog and validation diagnostics.

Use `read_scripts_report()` for launcher, dispatch, and resource paths that need validation-safe kept catalogs. `read_scripts()` still returns unvalidated scripts, so direct callers are an audit target.

## Script Object

`src/scripts/types.rs#Script` is the loaded script object. Key fields include:

```rust
pub struct Script {
    pub name: String,
    pub path: PathBuf,
    pub extension: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub alias: Option<String>,
    pub shortcut: Option<String>,
    pub typed_metadata: Option<TypedMetadata>,
    pub schema: Option<Schema>,
    pub plugin_id: String,
    pub plugin_title: Option<String>,
    pub kit_name: Option<String>,
    pub body: Option<String>,
}
```

Only some metadata is promoted directly onto `Script`; the full typed metadata object remains available through `typed_metadata`.

## Script Metadata

`src/scripts/metadata.rs#extract_full_metadata` merges three inputs:

- Typed metadata from `metadata = { ... }`.
- Schema from `schema = { ... }`.
- Legacy `// Key: Value` comments.

Typed metadata wins per field. Missing typed fields can fall back to legacy comment fields, so a script can use typed `name` and legacy `description` if the typed object omits `description`.

`src/metadata_parser/mod.rs#TypedMetadata` is the typed metadata shape. Fields visible in the Oracle bundle include:

- `name`
- `description`
- `author`
- `enter`
- `alias`
- `keyword`
- `icon`
- `shortcut`
- `tags`
- `placeholder`
- `cron`
- `schedule`
- `watch`
- `background`
- `system`
- `fallback`
- `fallbackLabel`
- `hidden`
- flattened `extra`

`keyword` accepts `expand` and `snippet` as aliases. Unknown/custom keys flow into `extra`; validation currently reads `trigger` from `typed_metadata.extra["trigger"]` when it is a string.

`src/metadata_parser/mod.rs#extract_typed_metadata` finds the metadata assignment, extracts the balanced object literal, normalizes JavaScript-ish object syntax, deserializes into `TypedMetadata`, and returns `MetadataParseResult { metadata, errors, span }`.

Legacy comment metadata is parsed leniently by `src/scripts/metadata.rs#parse_metadata_line`, with case-insensitive key matching. The visible legacy path maps known keys and ignores unknown keys. Schedule comments are parsed by `src/scripts/metadata.rs#extract_schedule_metadata`, which checks only the first 30 lines for `// Cron:` and `// Schedule:`.

Important gap: typed `cron` and `schedule` fields exist, but scheduling registration is only proven for comment metadata. Do not document typed schedule registration as implemented without verifying the scheduler path.

## Script Validation

`src/scripts/validation.rs#validate_script_catalog` accepts `Vec<Arc<Script>>` and returns:

```rust
ScriptCatalogReport {
    scripts: Arc<[Arc<Script>]>,
    validation: Arc<ValidationReport>,
}
```

`scripts` is the kept catalog. Scripts with fatal issues are excluded and represented as failed scripts in the validation report.

Current fatal validation is duplicate binding detection across:

- `shortcut` from `script.shortcut`
- `alias` from `script.alias`
- `keyword` from `script.typed_metadata.keyword`
- `trigger` from `script.typed_metadata.extra["trigger"]`

Bindings are bucketed by `(BindingKind, normalized_value)`, so an alias and shortcut with the same literal value do not collide with each other. Shortcut values are lowercased with internal whitespace collapsed. Alias, keyword, and trigger values are lowercased and trimmed. Empty values are skipped.

Every script participating in a duplicate gets its own fatal `ScriptValidationIssue` with related peer pointers. The duplicate-binding kind is `ScriptValidationKind::DuplicateBinding { binding, value }`.

`ScriptValidationKind` also defines metadata parse, schema parse, and invalid value issue kinds, but the visible implementation only proves duplicate binding detection. Typed metadata parse errors are returned by `extract_typed_metadata`, but `extract_full_metadata` does not prove that it propagates those errors into validation. Schema parse errors are likewise not proven to reach `ValidationReport`.

## Failed Script Diagnostics

`src/mcp_resources/mod.rs#FAILED_SCRIPTS_RESOURCE_URI` is `kit://failed-scripts`. `src/mcp_resources/mod.rs#read_kit_failed_scripts_resource` calls `crate::scripts::read_scripts_report()` at read time, then serializes a schema-versioned document.

The document includes:

- `schema_version`
- `validation_schema_version`
- `total_candidates`
- `valid_count`
- `fatal_count`
- `warning_count`
- `failed_scripts`
- `warnings`

The resource intentionally reflects current disk state at read time. That is useful for agents, but it can momentarily differ from the launcher's cached in-memory validation report until the UI refreshes.

The launcher diagnostic surface is documented in `lat.md/scripting.md`: `src/scripts/types.rs#ScriptIssueMatch` is the synthetic result, `src/scripts/grouping.rs#prepend_script_issues_row` inserts it at `flat_results[0]`, and `src/scripts/grouping.rs#get_grouped_results_with_validation` wires it into filter-cache behavior. Enter opens `AppView::ScriptIssuesView`; Escape returns to the script list; Cmd+C copies diagnostics from `format_script_issues_diagnostics`.

## Scriptlet Discovery

There are two scriptlet loading surfaces:

| Loader | Scope | Use |
|---|---|---|
| `src/scripts/scriptlet_loader/loading.rs#load_scriptlets` | Plugin-scoped across discovered plugins | Preferred production catalog path. |
| `src/scripts/scriptlet_loader/loading.rs#read_scriptlets` | Defaults to the main plugin scriptlets directory | Treat as legacy/simple unless a caller intentionally wants only `plugins/main`. |

`load_scriptlets` discovers plugins, scans `<plugin-root>/scriptlets/*.md`, skips companion `.actions.md` files, parses bundles through `src/scriptlets/mod.rs#parse_markdown_as_scriptlets`, converts parsed scriptlets into `src/scripts/types.rs#Scriptlet`, builds file-path anchors, attaches plugin identity, and sorts by group then name.

## Scriptlet Object

There are parser-level and runtime/search-level scriptlet structs. The parser-level `src/scriptlets/mod.rs#Scriptlet` is richer:

```rust
pub struct Scriptlet {
    pub name: String,
    pub command: String,
    pub tool: String,
    pub scriptlet_content: String,
    pub inputs: Vec<String>,
    pub group: String,
    pub preview: Option<String>,
    pub metadata: ScriptletMetadata,
    pub typed_metadata: Option<TypedMetadata>,
    pub schema: Option<Schema>,
    pub kit: Option<String>,
    pub source_path: Option<String>,
    pub actions: Vec<ScriptletAction>,
}
```

The runtime/search type stores the executable/search-facing subset plus plugin identity.

## Scriptlet Markdown Contract

`src/scriptlets/mod.rs#parse_markdown_as_scriptlets` supports:

- H1 headings as groups.
- H1 code fences as group-level prepend code.
- H2 headings as scriptlets.
- H3 headings as actions for the parent scriptlet.
- HTML comment metadata.
- Fenced `metadata` blocks.
- Fenced `schema` blocks.
- Tool code fences.
- Nested fences.
- Named inputs.
- Conditional/substitution markers.
- Source safety validation.

Scriptlet commands are derived from names by slugification. The loader sorts loaded scriptlets by group and then name.

HTML comment metadata is parsed by `src/scriptlets/mod.rs#parse_html_comment_metadata`. Visible fields include `trigger`, `shortcut`, `cron`, `schedule`, `background`, `watch`, `system`, `description`, `keyword`, `alias`, and extra fields.

Fenced metadata and schema are parsed by `src/scriptlet_metadata/mod.rs#parse_codefence_metadata`. `metadata` fences try JSON first, then simple flat `key: value` lines. Simple metadata is intentionally flat; it can map `keyword`, `expand`, and `snippet` to `TypedMetadata.keyword`, and it handles boolean flags such as `true` and `1`.

Complex nested structures such as full `menuSyntax` are not practical in scriptlet metadata today. Authors should declare complex menu syntax on a sibling TypeScript script until the scriptlet metadata parser grows nested object support.

## Scriptlet Actions

H3 headings define actions for the parent scriptlet. Companion `.actions.md` files define shared actions. `src/scripts/scriptlet_loader/loading.rs#is_actions_file` skips `.actions.md` files as standalone scriptlets so template variables such as `{{content}}` do not become broken top-level commands and action shortcuts do not leak into global hotkeys.

The parser-level shared-action path de-duplicates shared actions by command before attaching them to a scriptlet. Inline actions with the same command win over shared actions.

## Scriptlet Tools And Execution

`src/executor/scriptlet.rs#run_scriptlet` dispatches by normalized tool. Supported tool families include:

| Tool family | Behavior |
|---|---|
| Shell | `bash`, `zsh`, `sh`, and `fish` execute shell temp files. |
| Interpreters | `python`, `ruby`, `perl`, `php`, and `node` execute interpreter temp files. |
| TypeScript-like | `kit`, `ts`, `bun`, and `deno` execute through the Bun/TypeScript temp path. |
| `template` | Returns processed content. |
| `transform` | Transforms selected text. |
| `open` | Opens a target with platform open behavior. |
| `edit` | Opens content/path for editing. |
| `paste` | Pastes generated content. |
| `type` | Types generated content. |
| `submit` | Submits generated content. |

`ScriptletResult` reports exit code, stdout, stderr, and success. Setup and dispatch failures return `String` errors.

The parser recognizes named placeholders while skipping empty names, names beginning with `#`, names beginning with `/`, `else`, and duplicates. This supports template and conditional markers without treating them all as user inputs.

The Oracle bundle mentioned environment allowlisting, but the visible executor snippets did not show the exact allowlist. Document exact variables from `src/executor/scriptlet.rs` before changing the executor contract.

## Launcher And Search

Scripts and scriptlets flow into search/grouping through:

- `src/scripts/search/scripts.rs`
- `src/scripts/search/scriptlets.rs`
- `src/scripts/grouping.rs`

`Script.body` is loaded for content search. `src/scripts/loader.rs#read_scripts_from_dir_reloads_updated_body_content` proves reloads use fresh body content after file changes.

Do not conflate hidden and excluded scripts. Hidden scripts are metadata-controlled discoverability behavior and may still be valid. Excluded scripts are validation failures removed from the kept catalog to avoid ambiguous dispatch.

Menu syntax uses `!` for command picker discovery and `>` for argv command invocation. Duplicate command heads are disabled in the picker and ambiguous at execution time; keep that behavior separate from duplicate binding validation.

## MCP Resources

| Resource | Schema version | Purpose |
|---|---:|---|
| `kit://scripts` | 1 | Script catalog metadata. |
| `kit://scriptlets` | 1 | Scriptlet catalog metadata. |
| `kit://failed-scripts` | 1 | Validation diagnostics for excluded scripts. |
| `kit://sdk-reference` | 5 | SDK reference data shared with the in-app reference UI. |
| `kit://script-templates` | 1 | Script template catalog shared with the new-script/template UI. |

`kit://sdk-reference` and the in-app SDK Reference surface share `SdkFunctionRef` objects. `kit://script-templates` and the in-app template catalog share `ScriptTemplateRef` objects. Current script templates are `blank-starter` and `choice-list`.

Starter templates intentionally avoid `alias`, `shortcut`, `keyword`, and `trigger` fields so a newly created script does not immediately hide itself through duplicate binding validation.

Resource clients should be careful with nested JSON casing. The envelope uses schema-versioned/camelCase fields, but Oracle flagged mixed entry casing as a risk until every nested entry struct is audited and versioned.

## Scheduling Boundary

`src/scripts/scheduling.rs#register_scheduled_scripts` separately scans `<kit_path>/plugins/*/scripts` for files with `// Cron:` or `// Schedule:` comments and registers them with the scheduler.

Because this path appears separate from `read_scripts_report`, scheduled-script handling should be audited for validation parity. If the intended contract is "excluded means excluded everywhere," the scheduler must not register scripts that the kept catalog excludes.

## Agent Observability

Agents can inspect:

- Script catalog entries through `kit://scripts`.
- Scriptlet catalog entries through `kit://scriptlets`.
- Excluded script diagnostics through `kit://failed-scripts`.
- SDK APIs and harness workflow guidance through `kit://sdk-reference`.
- Starter templates through `kit://script-templates`.
- Launcher Script Issues state through normal UI/state receipts when the row is visible.
- Script body content freshness through loader/content tests.

Agents should not infer from a raw file alone that a script is runnable. For dispatch-safe understanding, prefer `read_scripts_report`-backed resources or UI receipts that prove the script is in the kept catalog.

## Verification Map

Recommended focused gates for this feature:

```bash
cargo test script_resources
cargo test script_content
cargo test script_preview_content_match
cargo test script_content_model
cargo test script_content_refresh_source_audit
cargo test validation
cargo test failed_scripts
cargo test script_templates
cargo test sdk_reference
cargo test agent_workspace_contract
cargo test kit_init_unsupported_sdk_audit
```

Run scriptlet smoke tests when touching parser, loader, executor, or SDK import behavior:

```bash
bun tests/smoke/test-scriptlet-execution.ts
bun tests/smoke/test-scriptlet-basic.ts
bun tests/smoke/test-scriptlet-typescript.ts
bun tests/smoke/test-scriptlet-bundles.ts
bun tests/smoke/test-scriptkit-sdk-import.ts
```

Source audits to keep nearby:

```bash
cargo test action_script_management
cargo test action_scriptlet_ranking
```

Run `lat check` after changing `lat.md/` or maintained feature-map chapters.

## Risks And Gaps

| Risk | Why it matters |
|---|---|
| Typed metadata parse errors are not propagated into validation | `extract_typed_metadata` returns errors, but the visible `extract_full_metadata` path drops them; malformed metadata may disappear instead of producing diagnostics. |
| Schema parse errors are not proven to reach validation | `ScriptValidationKind::SchemaParse` exists, but visible validation only proves duplicate binding detection. |
| Scheduling may bypass validation | `register_scheduled_scripts` scans scripts separately and is not proven to call `read_scripts_report`. |
| Typed `cron` / `schedule` support is ambiguous | Typed fields exist, but registration is only proven for `// Cron:` and `// Schedule:` comments. |
| Scriptlet validation-aware parser appears underused | `parse_scriptlets_with_validation` exists but active loader evidence points at `parse_markdown_as_scriptlets`. |
| Scriptlet duplicate bindings are not proven excluded | Script validation works on `Script`; no equivalent `validate_scriptlet_catalog` was proven. |
| `read_scriptlets()` is main-plugin-only | Production callers should prefer plugin-scoped `load_scriptlets`. |
| Scriptlet metadata is flat | Complex nested `menuSyntax` belongs in TypeScript script metadata today. |
| `kit://failed-scripts` reads disk state at read time | Resource output can briefly differ from launcher's cached validation state. |
| Resource entry casing may be mixed | Clients should not assume all nested fields are camelCase without a schema/version audit. |
| Frontmatter/icon resolution may be future-facing | Dead-code-marked helpers need proof before documenting frontmatter defaults as active behavior. |
| `read_scripts()` bypasses validation | Any caller using it directly can expose scripts excluded from `read_scripts_report`. |
| Discovery extension support is narrow | `.ts` and `.js` are proven; `.tsx`, `.jsx`, `.mjs`, and `.cjs` are not. |

## Boundaries

This feature is not:

- Prompt rendering for script execution after a script starts.
- ACP Chat, legacy `chat()`, or prompt-specific SDK UI behavior.
- Quick Terminal or terminal prompt behavior.
- General plugin skill/agent loading, even though plugin discovery exposes sibling directories.
- Full scheduler semantics beyond script schedule discovery and the validation-parity risk.

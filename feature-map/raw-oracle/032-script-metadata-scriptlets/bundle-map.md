
## Packx Command

```bash
packx --limit 49k -l 18 \
  -s "read_scripts" \
  -s "extract_full_metadata" \
  -s "TypedMetadata" \
  -s "validate_script_catalog" \
  -s "detect_binding_collisions" \
  -s "read_scripts_report" \
  -s "load_scriptlets" \
  -s "parse_markdown_as_scriptlets" \
  -s "parse_scriptlet_section" \
  -s "ScriptletMetadata" \
  -s "run_scriptlet" \
  -s "register_scheduled_scripts" \
  -s "discover_plugins" \
  -s "NewScriptFromTemplate" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md .goals/feature_map.md \
  .agents/skills/sdk-script-execution/SKILL.md \
  .claude/skills/script-kit-scripting/SKILL.md \
  removed-docs removed-docs removed-docs removed-docs removed-docs \
  src/scripts/metadata.rs src/metadata_parser/mod.rs src/schema_parser/mod.rs \
  src/scripts/loader.rs src/scripts/types.rs src/scripts/validation.rs src/scripts/scheduling.rs \
  src/scripts/search/scripts.rs src/scripts/search/scriptlets.rs src/scripts/grouping.rs \
  src/scripts/scriptlet_loader/loading.rs src/scripts/scriptlet_loader/parsing.rs \
  src/scriptlets/mod.rs src/scriptlet_metadata/mod.rs \
  src/executor/scriptlet.rs src/executor/runner.rs \
  src/mcp_resources/mod.rs src/plugins/discovery.rs src/plugins/manifest.rs src/plugins/types.rs \
  src/setup/mod.rs src/app_execute/menu_syntax_execution.rs src/prompt_handler/mod.rs \
  tests/script_resources.rs tests/script_content_search.rs tests/script_preview_content_match.rs \
  tests/script_content_model.rs tests/script_content_refresh_source_audit.rs \
  tests/source_audits/action_script_management.rs tests/source_audits/action_scriptlet_ranking.rs \
  tests/smoke/test-scriptlet-execution.ts tests/smoke/test-scriptlet-basic.ts \
  tests/smoke/test-scriptlet-typescript.ts tests/smoke/test-scriptlet-bundles.ts \
  tests/smoke/test-scriptkit-sdk-import.ts tests/kit_init_unsupported_sdk_audit.rs \
  tests/agent_workspace_contract.rs \
  > ~/.oracle/bundles/script-metadata-scriptlets-atlas.txt
```

## Pack Summary


## Inclusion Rationale

- Repo process files and skills establish the feature-map and scripting-domain contracts.
- `removed-docs`, `removed-docs`, `removed-docs`, and `removed-docs` capture architectural intent.
- Script loading, metadata, validation, scheduling, search, and grouping files define the script catalog.
- Scriptlet parser/loader/metadata/executor files define markdown bundle parsing and tool execution.
- MCP resources and plugin/setup files define agent-visible catalogs and workspace bootstrap.
- Tests cover resource envelopes, script content search/preview/refresh, script actions, scriptlet actions, smoke execution, SDK imports, unsupported SDK audits, and workspace contracts.

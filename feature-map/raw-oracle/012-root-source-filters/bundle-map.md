# 012 Root Source Filters Bundle Map

Slug: `root-source-filters-atlas`

Feature: Root Unified Source Filters / source-chip query routing / lazy row paging.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/main-menu-search-selection/SKILL.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/theme-config-preferences/SKILL.md`
- `lat.md/builtins.md`
- `lat.md/menu-syntax.md`
- `lat.md/verification.md`
- `lat.md/surfaces.md`
- `lat.md/automation.md`
- `src/menu_syntax/source_heads.rs`
- `src/menu_syntax/payload.rs`
- `src/menu_syntax/query.rs`
- `src/menu_syntax/source_filter_browse.rs`
- `src/menu_syntax/main_hint.rs`
- `src/app_impl/filtering_cache.rs`
- `src/app_impl/root_file_search.rs`
- `src/app_impl/filter_input_core.rs`
- `src/app_impl/filter_input_change.rs`
- `src/scripts/grouping.rs`
- `src/scripts/types.rs`
- `src/scripts/search/unified.rs`
- `src/main_window_preflight/types.rs`
- `src/main_window_preflight/build.rs`
- `src/list_item/mod.rs`
- `src/scrolling/selection_owned.rs`
- `src/config/types.rs`
- `src/config/defaults.rs`
- `tests/menu_syntax_source_filters.rs`
- `tests/source_audits/root_unified_source_filters_contract.rs`
- `tests/source_audits/root_unified_source_filter_browse_contract.rs`
- `tests/source_audits/root_file_search_contract.rs`
- `tests/source_audits/root_unified_passive_snapshot_contract.rs`
- `tests/source_audits/root_unified_search_stability_contract.rs`
- `tests/source_audits/root_unified_config_schema_parity_contract.rs`
- `scripts/agentic/root-source-filter-stability.ts`
- `scripts/agentic/root-source-filter-clipboard.ts`
- `scripts/agentic/root-source-filter-history-up.ts`
- `scripts/agentic/source-chip-pagination-proof.ts`
- `scripts/agentic/root-source-filter-matrix.ts`
- `scripts/agentic/root-source-filter-lazy-scroll.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/main-menu-search-selection/SKILL.md .agents/skills/actions-popups/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/theme-config-preferences/SKILL.md lat.md/builtins.md lat.md/menu-syntax.md lat.md/verification.md lat.md/surfaces.md lat.md/automation.md src/menu_syntax/source_heads.rs src/menu_syntax/payload.rs src/menu_syntax/query.rs src/menu_syntax/source_filter_browse.rs src/menu_syntax/main_hint.rs src/app_impl/filtering_cache.rs src/app_impl/root_file_search.rs src/app_impl/filter_input_core.rs src/app_impl/filter_input_change.rs src/scripts/grouping.rs src/scripts/types.rs src/scripts/search/unified.rs src/main_window_preflight/types.rs src/main_window_preflight/build.rs src/list_item/mod.rs src/scrolling/selection_owned.rs src/config/types.rs src/config/defaults.rs tests/menu_syntax_source_filters.rs tests/source_audits/root_unified_source_filters_contract.rs tests/source_audits/root_unified_source_filter_browse_contract.rs tests/source_audits/root_file_search_contract.rs tests/source_audits/root_unified_passive_snapshot_contract.rs tests/source_audits/root_unified_search_stability_contract.rs tests/source_audits/root_unified_config_schema_parity_contract.rs scripts/agentic/root-source-filter-stability.ts scripts/agentic/root-source-filter-clipboard.ts scripts/agentic/root-source-filter-history-up.ts scripts/agentic/source-chip-pagination-proof.ts scripts/agentic/root-source-filter-matrix.ts scripts/agentic/root-source-filter-lazy-scroll.ts -s "RootUnifiedSourceFilter" -s "source_filter" -s "sourceFilters" -s "sourceStatus" -s "source_chip" -s "source-only" -s "source_only" -s "visibleResultKeyFingerprint" -s "source_filter_mode_blocks_input_history_recall" -s "root_file_source_chip_pages_on_near_bottom_selection" -s "source_filter_browse" -s "filter_indicators" -l 15 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/root-source-filters-atlas.txt
```

Final bundle size: 228,508 bytes. Oracle reported 61,974 input tokens for this attached bundle and prompt.

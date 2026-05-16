# 040 Main Window Sizing and Surface Contracts Bundle Map

Oracle slug: `main-window-surface-atlas`

Bundle path: `/Users/johnlindquist/.oracle/bundles/main-window-surface-atlas.txt`

Note: the Oracle prompt was initially prepared with feature id `038`, but `038-agent-skills-ai-context-catalog` already exists. The maintained atlas entry for this completed pass is `040-main-window-sizing-surface-contracts`; the raw prompt and answer are preserved as generated for auditability.

## Lat Context

```bash
lat expand "038 Window Resizing Surface Contracts Main Window Presentation Modes Mini Full sizing AppView SurfaceKind resize_to_view update_window_size"
lat search "window resizing surface contracts main window presentation modes mini full AppView SurfaceKind resize_to_view update_window_size"
```

Top sections used:

- `lat.md/builtins#Built-ins#Main Window Sizing Modes`
- `lat.md/tests/mini-window-contract#Mini Window Contract Tests#Mini resize width clamp`
- `lat.md/tests/mini-window-contract#Mini Window Contract Tests#Chat and ACP mode sizing`
- `lat.md/windowing#Windowing`
- `lat.md/windowing#Windowing#Mini Main Window Contract`

## Skills

- `.agents/skills/window-resizing/SKILL.md`
- `.agents/skills/launcher-surface-contracts/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`

## Packx Command

```bash
packx --limit 49k -l 4 \
  -s "MainWindowMode" \
  -s "ViewType" \
  -s "MiniMainWindow" \
  -s "update_window_size_deferred" \
  -s "calculate_window_size_params" \
  -s "surface_contract" \
  -s "SurfaceKind" \
  -s "native_footer_surface" \
  -s "surfaceContract" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md \
  .agents/skills/window-resizing/SKILL.md \
  .agents/skills/launcher-surface-contracts/SKILL.md \
  .agents/skills/protocol-automation/SKILL.md \
  .agents/skills/testing-quality-gates/SKILL.md \
  lat.md/windowing.md lat.md/builtins.md lat.md/surfaces.md lat.md/protocol.md lat.md/automation.md lat.md/verification.md lat.md/tests/mini-window-contract.md \
  docs/ai/contracts/surface-contracts.json scripts/generate-surface-contracts.ts scripts/agentic/filterable-surface-matrix.ts scripts/agentic/surface-navigator-inventory-audit.ts \
  src/window_resize/mod.rs src/app_impl/ui_window.rs src/main_sections/app_view_state.rs src/app_execute/builtin_execution.rs src/app_impl/trigger_builtin_dispatch.rs src/app_impl/automation_surface.rs src/app_impl/lifecycle_reset.rs \
  tests/window_resize_logic.rs tests/source_audits/mini_main_window.rs tests/trigger_builtin_current_app_commands_contract.rs tests/surface_contract_matrix_artifact_contract.rs tests/state_result_surface_contract_snapshot.rs tests/current_view_transition_inventory_contract.rs tests/trigger_builtin_post_match_surface_rekey_contract.rs tests/main_automation_surface_rekey_owner_contract.rs \
  > ~/.oracle/bundles/main-window-surface-atlas.txt
```

Final bundle summary: 32 files, ripgrep search mode, 4 context lines, 622 matches, 212 context windows, 41,951 exact tokens, 164,838 bytes.

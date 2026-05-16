# 036 Tray Menu, App Lifecycle, Distribution, and Updates Bundle Map



## Lat Context

```bash
source search "tray menu app lifecycle distribution updater install app quit restart dock menu launch at login"
```


- `removed-docs menu`
- `removed-docs menu#Sections`
- `removed-docs menu#Update checker`
- `removed-docs states`
- `removed-docs`
- `removed-docs resources and tools`
- `removed-docs`

## Skills

- `.agents/skills/platform-windowing-macos/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`

## Packx Command

```bash
packx --limit 49k -l 8 \
  -s "TrayManager" \
  -s "TrayMenuAction" \
  -s "current_tray_menu_observation_snapshot" \
  -s "UpdateState" \
  -s "CheckForUpdates" \
  -s "About" \
  -s "release-manifest" \
  -s "launch at login" \
  -s "runtime_shutdown" \
  -s "quit" \
  -s "restart" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md \
  .agents/skills/platform-windowing-macos/SKILL.md \
  .agents/skills/protocol-automation/SKILL.md \
  .agents/skills/testing-quality-gates/SKILL.md \
  removed-docs removed-docs removed-docs removed-docs removed-docs removed-docs \
  src/tray/mod.rs src/updates.rs src/about/mod.rs src/about/render.rs src/app_impl/about_route.rs src/branding.rs src/login_item.rs \
  src/main_entry/app_run_setup.rs src/main_entry/runtime_tray_hotkeys.rs src/main_entry/runtime_shutdown.rs src/main_entry/runtime_init.rs src/main_entry/runtime_window.rs src/main_entry/runtime_stdin_match_core.rs \
  src/mcp_computer_use_tools.rs src/mcp_computer_use/handlers.rs src/config/types.rs scripts/config-cli.ts Cargo.toml Makefile .github/workflows/ci.yml .github/workflows/release.yml scripts/verify-macos-bundle.sh scripts/verify-release-version.sh scripts/verify.sh \
  tests/about_surface_contract.rs tests/about_surface_source_audit.rs tests/source_audits/mcp_computer_list_tray_menu_contract.rs tests/source_audits/mcp_computer_get_tray_menu_item_contract.rs tests/source_audits/mcp_computer_get_tray_menu_item_by_id_contract.rs tests/source_audits/update_picker_contract.rs \
  > ~/.oracle/bundles/tray-lifecycle-distribution-atlas.txt
```

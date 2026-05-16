# 036 Tray Menu, App Lifecycle, Distribution, and Updates Bundle Map

Oracle slug: `tray-lifecycle-distribution-atlas`

Bundle path: `/Users/johnlindquist/.oracle/bundles/tray-lifecycle-distribution-atlas.txt`

## Lat Context

```bash
lat expand "036 Tray Menu App Lifecycle Distribution Updates: tray menu quit restart launch at login app updater packaging distribution install"
lat search "tray menu app lifecycle distribution updater install app quit restart dock menu launch at login"
```

Top sections used:

- `lat.md/tray-menu#Tray menu`
- `lat.md/tray-menu#Tray menu#Sections`
- `lat.md/tray-menu#Tray menu#Update checker`
- `lat.md/about#About#Update states`
- `lat.md/distribution#Distribution`
- `lat.md/protocol#MCP resources and tools`
- `lat.md/verification#Verification`

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
  lat.md/tray-menu.md lat.md/about.md lat.md/distribution.md lat.md/protocol.md lat.md/windowing.md lat.md/verification.md \
  src/tray/mod.rs src/updates.rs src/about/mod.rs src/about/render.rs src/app_impl/about_route.rs src/branding.rs src/login_item.rs \
  src/main_entry/app_run_setup.rs src/main_entry/runtime_tray_hotkeys.rs src/main_entry/runtime_shutdown.rs src/main_entry/runtime_init.rs src/main_entry/runtime_window.rs src/main_entry/runtime_stdin_match_core.rs \
  src/mcp_computer_use_tools.rs src/mcp_computer_use/handlers.rs src/config/types.rs scripts/config-cli.ts Cargo.toml Makefile .github/workflows/ci.yml .github/workflows/release.yml scripts/verify-macos-bundle.sh scripts/verify-release-version.sh scripts/verify.sh \
  tests/about_surface_contract.rs tests/about_surface_source_audit.rs tests/source_audits/mcp_computer_list_tray_menu_contract.rs tests/source_audits/mcp_computer_get_tray_menu_item_contract.rs tests/source_audits/mcp_computer_get_tray_menu_item_by_id_contract.rs tests/source_audits/update_picker_contract.rs \
  > ~/.oracle/bundles/tray-lifecycle-distribution-atlas.txt
```

Final bundle summary: 36 files, ripgrep search mode, 8 context lines, 528 matches, 83 context windows, 46,129 exact tokens, 180,851 bytes.

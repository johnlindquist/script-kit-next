# 036 Tray Menu and Global App Entry Points Bundle Map

Oracle slug: `tray-menu-entry-atlas`

Bundle path: `/Users/johnlindquist/.oracle/bundles/tray-menu-entry-atlas.txt`

## Lat Context

```bash
lat expand "036 Tray Menu and Global App Entry Points: menu bar tray status item show main window settings notes quit restart global shortcuts"
lat search "tray menu menu bar status item global entry points show main window settings notes quit restart shortcuts"
```

Top sections used:

- `lat.md/tray-menu#Tray menu`
- `lat.md/tray-menu#Tray menu#Sections`
- `lat.md/verification#Verification#Main menu and footer`
- `lat.md/notes#Notes`
- `lat.md/automation#Automation#Surface-proof CLI#Surface classes`

## Skills

- `.agents/skills/platform-windowing-macos/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`
- `.agents/skills/builtin-filterable-surfaces/SKILL.md`

## Packx Command

```bash
packx --preview --limit 49k -l 12 \
  -s "TrayManager" \
  -s "TrayMenuAction" \
  -s "current_tray_menu_observation_snapshot" \
  -s "main_shortcut_accelerator" \
  -s "template_menu_items" \
  -s "refresh_version_label" \
  -s "CheckForUpdates" \
  -s "runtime_tray_hotkeys" \
  -s "computer/list_tray_menu" \
  -s "computer/get_tray_menu_item" \
  -s "computer/get_tray_menu_item_by_id" \
  -s "openAbout" \
  -s "Current App Commands" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md .goals/feature_map.md \
  .agents/skills/platform-windowing-macos/SKILL.md \
  .agents/skills/protocol-automation/SKILL.md \
  .agents/skills/testing-quality-gates/SKILL.md \
  .agents/skills/builtin-filterable-surfaces/SKILL.md \
  lat.md/tray-menu.md lat.md/protocol.md lat.md/automation.md lat.md/verification.md lat.md/surfaces.md lat.md/notes.md lat.md/acp-chat.md lat.md/builtins.md \
  src/tray/mod.rs src/main_entry/app_run_setup.rs src/main_entry/runtime_tray_hotkeys.rs src/hotkeys/mod.rs \
  src/menu_bar/current_app_commands.rs src/menu_bar/mod.rs src/menu_bar/tests.rs src/protocol/types/menu_bar.rs \
  src/mcp_computer_use_tools.rs src/updates.rs src/branding.rs src/login_item.rs src/main_sections/app_view_state.rs src/render_builtins/about.rs \
  tests/source_audits/mcp_computer_list_tray_menu_observation_only.rs \
  tests/source_audits/computer_get_tray_menu_item_contract.rs \
  tests/source_audits/computer_get_tray_menu_item_by_id_contract.rs \
  tests/main_window_global_key_intent_contract.rs tests/launcher_startup_entrypoint_contract.rs tests/current_app_commands.rs \
  > ~/.oracle/bundles/tray-menu-entry-atlas.txt
```

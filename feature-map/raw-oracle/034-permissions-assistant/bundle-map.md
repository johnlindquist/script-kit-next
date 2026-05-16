# 034 Permissions and Permission Assistant Bundle Map



## Lat context

```bash
source search "permissions permission assistant accessibility screen recording microphone system settings setup prompt preflight"
```


- `removed-docs Setup NUX#Non-prompting microphone preflight`
- `removed-docs Assistant#Built-in assistant entry points`
- `removed-docs Assistant`
- `removed-docs Assistant`
- `removed-docs Assistant#Passive detection does not prompt`

## Packx command

```bash
packx --limit 49k -l 18 \
  -s "PermisoAssistant" \
  -s "PermisoPanel" \
  -s "PermissionStatus" \
  -s "AXIsProcessTrusted" \
  -s "CGPreflightScreenCaptureAccess" \
  -s "authorizationStatusForMediaType" \
  -s "allow-accessibility" \
  -s "allow-screen-recording" \
  -s "Permission Assistant" \
  -s "computer/list_permissions" \
  -s "computer/get_permission" \
  -s "microphone_authorized" \
  -s "screen_capture_authorized" \
  -s "settings_window_snapshot" \
  -s "AppDragSourceView" \
  -s "PassiveOverlayPanel" \
  -s "host_app_bundle_url" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md .goals/feature_map.md \
  .agents/skills/platform-windowing-macos/SKILL.md \
  .agents/skills/actions-popups/SKILL.md \
  .agents/skills/storage-cache-security/SKILL.md \
  removed-docs removed-docs removed-docs \
  removed-docs removed-docs removed-docs removed-docs \
  src/platform/permiso/mod.rs src/platform/permiso/panel.rs src/platform/permiso/host_app.rs \
  src/platform/permiso/locator.rs src/platform/permiso/overlay_window.rs src/platform/permiso/drag_source.rs \
  src/platform/permiso_detect.rs src/platform/screenshots_window_open.rs src/builtins/mod.rs \
  src/app_execute/builtin_execution.rs src/render_builtins/settings.rs \
  src/mcp_computer_use_tools.rs src/dictation/device.rs src/dictation/setup.rs \
  tests/source_audits/permiso_builtin_contract.rs tests/source_audits/permiso_no_prompt_contract.rs \
  tests/source_audits/permiso_teardown_contract.rs tests/source_audits/mcp_computer_list_permissions_observation_only.rs \
  tests/source_audits/computer_list_permissions_contract.rs tests/source_audits/computer_get_permission_contract.rs \
  tests/dictation_setup_nux_contract.rs \
  > ~/.oracle/bundles/permissions-assistant-atlas.txt
```

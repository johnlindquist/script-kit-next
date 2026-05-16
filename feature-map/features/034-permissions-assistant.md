# 034 Permissions and Permission Assistant

This chapter maps Script Kit GPUI's passive macOS permission setup assistant and the read-only permission status surfaces used by users and agents.

Raw Oracle reference: [answer](../raw-oracle/034-permissions-assistant/answer.md), [prompt](../raw-oracle/034-permissions-assistant/prompt.md), [bundle map](../raw-oracle/034-permissions-assistant/bundle-map.md), [full log](../raw-oracle/034-permissions-assistant/output.log), [session metadata](../raw-oracle/034-permissions-assistant/session.json).

## Executive Summary

Permissions and Permission Assistant has two jobs:

- User setup: `builtin/allow-accessibility` and `builtin/allow-screen-recording` open the matching macOS Privacy & Security pane, then present a native AppKit overlay above System Settings with a Script Kit `.app` drag row.
- Passive status: Accessibility, Screen Recording, Microphone, and MCP-facing Event Synthesizing checks read state without prompting, granting, writing TCC, clicking System Settings, activating Script Kit, or changing app activation policy.

The core contract is strict: Script Kit may read status and guide the user through manual setup, but it must not silently request or mutate macOS privacy permissions.

## What Users Can Do

| Capability | Entry | Result |
|---|---|---|
| Open Accessibility setup | Launcher or Settings **Allow Accessibility** | System Settings opens to Accessibility and the passive overlay appears. |
| Open Screen Recording setup | Launcher or Settings **Allow Screen Recording** | System Settings opens to Screen Recording and the passive overlay appears. |
| Drag Script Kit into an allowlist | Overlay drag row | The host `.app` bundle URL is placed on the pasteboard for a copy drag. |
| Read permission status | `computer/list_permissions` | Returns Accessibility, Screen Recording, and Event Synthesizing rows. |
| Read one permission | `computer/get_permission` | Returns a closed-schema row for one permission id. |
| Detect dictation microphone readiness | Dictation setup/start flow | Microphone status is read passively and setup opens when permission is missing. |
| Detect screenshot proof readiness | Screenshot/capture paths | Screen Recording preflight and image content audit prevent false visual proof. |

Users cannot use this feature to force-grant permissions, edit `TCC.db`, run `tccutil`, make Script Kit click the allowlist, silently prompt macOS, or make MCP permission tools open settings, focus windows, synthesize input, or grant anything.

## Core Concepts

`src/platform/permiso/mod.rs#PermisoAssistant` is the public assistant entry point. `present(panel)` opens the settings URL, presents the overlay, and returns a `PermisoHandle`; `present_retained(panel)` stores that handle in a process-global slot so the native overlay stays alive after command execution; `dismiss_active()` clears the active handle.

`src/platform/permiso/panel.rs#PermisoPanel` defines the assistant panels:

| Panel | Display name | Receipt name | Settings pane |
|---|---|---|---|
| `Accessibility` | `Accessibility` | `accessibility` | `Privacy_Accessibility` |
| `ScreenRecording` | `Screen Recording` | `screenRecording` | `Privacy_ScreenCapture` |

`src/platform/permiso_detect.rs#PermissionStatus` models passive permission status. Accessibility uses `AXIsProcessTrusted()`. Screen Recording uses `CGPreflightScreenCaptureAccess()`. Microphone uses `AVCaptureDevice.authorizationStatusForMediaType(...)`. Accessibility and Screen Recording currently map false to `Denied`; Microphone can report `Authorized`, `Denied`, `NotDetermined`, or `Unknown`.

The overlay is native AppKit, not a GPUI popup. The durable contract describes a separate non-activating `NSPanel`-style overlay that cannot become key/main, does not use `WindowKind::PopUp`, does not mutate launcher panel invariants, and owns its own AppKit lifetime.

The drag row is `src/platform/permiso/drag_source.rs#AppDragSourceView`. It resolves the host `.app` with `src/platform/permiso/host_app.rs#host_app_bundle_url`, writes a `.fileURL` pasteboard item, uses copy drag semantics, hides the row while dragging, and restores it when dragging ends. The payload must be the `.app` directory, not `Contents/MacOS/...`.

## Entry Points

| Entry | Owner | Behavior |
|---|---|---|
| `builtin/allow-accessibility` | `src/builtins/mod.rs` | Registers **Allow Accessibility** as `PermissionCommandType::AllowAccessibility`. |
| `builtin/allow-screen-recording` | `src/builtins/mod.rs` | Registers **Allow Screen Recording** as `PermissionCommandType::AllowScreenRecording`. |
| Permission command execution | `src/app_execute/builtin_execution.rs` | Calls `PermisoAssistant::present_retained(...)`, shows HUD on success, and returns structured failure on open errors. |
| Settings **Allow Accessibility** | `src/render_builtins/settings.rs` | Constructs the same builtin entry and executes it. |
| Settings **Allow Screen Recording** | `src/render_builtins/settings.rs` | Constructs the same builtin entry and executes it. |
| `computer/list_permissions` | `src/mcp_computer_use_tools.rs` | Returns read-only permission rows; no runtime bridge or settings side effects. |
| `computer/get_permission` | `src/mcp_computer_use_tools.rs` | Returns one read-only permission row by id. |
| Dictation setup | `src/dictation/device.rs`, `src/dictation/setup.rs` | Reads microphone status without `requestAccessForMediaType`. |
| Screenshot proof | `src/platform/screenshots_window_open.rs` | Checks Screen Recording preflight and rejects misleading captures. |

The Settings surface also contains related legacy entries such as **Check Permissions**, **Request Accessibility Permission**, and **Open Accessibility Settings**. Those are adjacent permission utilities, not the native Permission Assistant flow.

## User Workflows

### Allow Accessibility

1. User selects **Allow Accessibility** from launcher search or Settings.
2. The builtin routes to `PermissionCommandType::AllowAccessibility`.
3. Execution calls `PermisoAssistant::present_retained(PermisoPanel::Accessibility)`.
4. The assistant opens the Accessibility privacy pane.
5. The overlay appears with the Script Kit drag row.
6. Script Kit shows `Drag Script Kit into Accessibility`.
7. The user manually drags the `.app` row into the macOS allowlist.

This path must not call `AXIsProcessTrustedWithOptions`, write TCC, use `tccutil`, automate System Settings, activate Script Kit, or hide the main prompt through `prepare_for_submit_hide`.

### Allow Screen Recording

1. User selects **Allow Screen Recording** from launcher search or Settings.
2. The builtin routes to `PermissionCommandType::AllowScreenRecording`.
3. Execution calls `PermisoAssistant::present_retained(PermisoPanel::ScreenRecording)`.
4. The assistant opens the Screen Recording privacy pane.
5. The overlay appears with the Script Kit drag row.
6. Script Kit shows `Drag Script Kit into Screen Recording`.
7. The user manually drags the `.app` row into the macOS allowlist.

This path must not call `CGRequestScreenCaptureAccess` or take screenshots just to discover status. It uses preflight state and manual user action.

### Drag The App Row

The overlay row resolves the real host `.app` bundle. On drag start, `AppDragSourceView` hides the row and provides a `.fileURL` pasteboard item. On drag end, it shows the row again. If Script Kit is running as a bare development executable and no `.app` ancestor exists, host app resolution should fail instead of dragging an internal executable path.

### Read Permission Status From MCP

Agents use `computer/list_permissions` for all rows or `computer/get_permission` for a single row. Current MCP permission ids are:

- `accessibility`
- `screenRecording`
- `eventSynthesizing`

Rows include `id`, `name`, `granted`, and `status`. Status maps `Some(true)` to `granted`, `Some(false)` to `notGranted`, and `None` to `unknown`. Microphone detection exists for dictation setup but is not currently an MCP permission row.

### Dictation Microphone Preflight

Dictation setup calls the passive microphone authorization status path. Denied or not-determined microphone permission becomes `PermissionNeeded`, setup readiness is false, and starting dictation shows `Dictation needs microphone permission` before opening setup. Hotkey configuration is separate from microphone readiness.

### Screenshot Permission Proof

Screenshot capture checks Screen Recording access with `CGPreflightScreenCaptureAccess()`. If capture proceeds, the image buffer is audited before proof is accepted. Empty, transparent, solid-like, or dark low-variety images are rejected with structured errors such as `automation.capture_screenshot.permission_failed` or `automation.capture_screenshot.blank_image_rejected`.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open Accessibility assistant | **Allow Accessibility** builtin | Launcher row selected | Enter/click | `PermissionCommandType::AllowAccessibility -> present_retained(Accessibility)` | Accessibility pane opens; overlay appears; HUD names Accessibility. | `tests/source_audits/permiso_builtin_contract.rs` |
| Open Screen Recording assistant | **Allow Screen Recording** builtin | Launcher row selected | Enter/click | `PermissionCommandType::AllowScreenRecording -> present_retained(ScreenRecording)` | Screen Recording pane opens; overlay appears; HUD names Screen Recording. | `tests/source_audits/permiso_builtin_contract.rs` |
| Open from Settings | Settings permission item | Settings surface visible | Click/select | `SettingsAction::* -> BuiltInEntry -> execute_builtin` | Same retained assistant path as launcher. | `src/render_builtins/settings.rs` |
| Drag app | Overlay row | Native overlay visible | Drag | `AppDragSourceView -> host_app_bundle_url` | `.app` bundle URL copy drag. | `src/platform/permiso/drag_source.rs` |
| List permissions | MCP tool | Agent call | JSON request | `computer/list_permissions` | Read-only rows for Accessibility, Screen Recording, Event Synthesizing. | `tests/source_audits/computer_list_permissions_contract.rs` |
| Get permission | MCP tool | Agent call | JSON request | `computer/get_permission` | One read-only row or not-found status. | `tests/source_audits/computer_get_permission_contract.rs` |
| Start dictation without mic permission | Dictation start | Dictation setup needed | Command/hotkey | `microphone_permission_status -> open_dictation_setup_if_microphone_not_ready` | HUD and setup, no hidden prompt. | `tests/dictation_setup_nux_contract.rs` |
| Capture screenshot without Screen Recording | Automation capture | Capture requested | Protocol/tool call | `screen_capture_access_preflight` + content audit | Permission failure or blank-image rejection. | `src/platform/screenshots_window_open.rs` |

## State Machines

Assistant open:

```text
User selects assistant builtin
  -> execute_builtin PermissionCommand
  -> present_retained(panel)
  -> present_settings_url(panel)
  -> OverlayController::present(panel)
  -> store PermisoHandle in ACTIVE_PERMISO_HANDLE
  -> show HUD
  -> user drags app manually
  -> dismiss/replacement/drop releases overlay
```

Permission read:

```text
Caller asks for status
  -> passive preflight/status API
  -> map platform result to PermissionStatus or MCP row
  -> return status
  -> no prompt, no grant, no settings automation, no focus/input side effect
```

## Data, Storage, And Privacy Boundaries

The assistant must never write `TCC.db`, shell out to `tccutil`, patch macOS privacy state, or call prompting APIs from passive detection. Forbidden APIs include `AXIsProcessTrustedWithOptions`, `CGRequestScreenCaptureAccess`, `requestAccessForMediaType`, `CGRequestPostEventAccess`, and `CGEventPost` in preflight/status paths.

Allowed passive APIs include `AXIsProcessTrusted()`, `CGPreflightScreenCaptureAccess()`, `CGPreflightPostEventAccess()`, and `authorizationStatusForMediaType(...)`.

The retained overlay handle is process-local state in `ACTIVE_PERMISO_HANDLE`. It is not durable storage, not a user preference, and not a permission record.

The drag payload exposes only the local path to Script Kit's `.app` bundle during the drag. It does not expose scripts, documents, clipboard content, selected text, or the internal executable path.

MCP permission tools return only schema/status fields. They do not return images, window state, user data, app inventory, paths, action handles, or System Settings state beyond passive permission checks.

## Error And Disabled States

`present_settings_url(panel)` can fail if native URL construction fails, `NSWorkspace` is unavailable, `openURL` returns false, or the platform is not macOS. Builtin execution shows `Failed to open Permission Assistant: ...` and returns `ERROR_LAUNCH_FAILED` with `allow_accessibility_failed` or `allow_screen_recording_failed`.

`host_app_bundle_url()` can fail when running from an unbundled binary. That should stop the drag-source path instead of substituting a non-`.app` executable.

If the System Settings window cannot be located, the documented assistant contract falls back to centered overlay placement and re-queries later. It must not cache stale frames across display changes, Spaces switches, or app activation.

Non-macOS status checks return `Unknown`. MCP rows represent unavailable checks as `granted: null` and `status: "unknown"`.

## Invariants And Regression Risks

- Permission detection stays passive: read status only; never request.
- The assistant is manual: it may open Settings and show the overlay, but it must not click, type, press, focus, or grant.
- The overlay is native, non-activating, and separate from GPUI popup/window contracts.
- The drag payload is the `.app` bundle URL, never `Contents/MacOS`.
- Only one retained assistant handle owns the current overlay; replacement or dismissal drops the prior handle.
- Builtin assistant command arms must not call `prepare_for_submit_hide`.
- MCP permission tools remain closed-schema and status-only.
- Dictation microphone preflight must use authorization status, not a request API.
- Screenshot proof must reject misleading blank or black captures.

Main risks are subtle API substitutions: replacing preflight/status calls with prompting APIs, adding System Settings automation, caching stale window frames, removing the retained handle, adding action handles to MCP permission rows, or treating a screenshot file as proof without content audit.

## Verification Recipes

Focused checks:

```bash
cargo test --test source_audits permiso_builtin_contract -- --nocapture
cargo test --test source_audits permiso_no_prompt_contract -- --nocapture
cargo test --test source_audits permiso_teardown_contract -- --nocapture
cargo test --test source_audits mcp_computer_list_permissions_observation_only -- --nocapture
cargo test --test source_audits computer_list_permissions_contract -- --nocapture
cargo test --test source_audits computer_get_permission_contract -- --nocapture
cargo test --test dictation_setup_nux_contract -- --nocapture
```

Manual runtime proof should launch Script Kit from a real `.app` bundle, run each assistant command, verify the matching System Settings pane opens, verify the overlay appears without activating Script Kit, verify the HUD text, and verify the drag payload represents the host `.app`. Use source audits for the non-prompting proof; screenshots alone are not sufficient.

MCP proof should call `computer/list_permissions` with an empty object and `computer/get_permission` with each known id, while confirming no Settings window opens and no prompt/input/focus side effect occurs.

## Agent Notes

Agents should treat this feature as a manual setup guide plus read-only status layer. Use MCP permission tools to observe status, and direct users to **Allow Accessibility** or **Allow Screen Recording** when setup is needed. Do not attempt to grant permissions with shell commands, TCC edits, System Settings clicks, or prompt APIs.

Before screenshot-based visual proof, check Screen Recording status and require capture content audit to pass. For dictation readiness, use dictation setup state and passive microphone preflight, not MCP permission rows.

Keep the legacy accessibility prompt/settings commands conceptually separate from the Permission Assistant. They are adjacent utilities, not the native drag-overlay setup flow.

## Open Questions And Gaps

- The focused Oracle bundle showed simplified or stubbed locator/overlay source in places while `lat.md/permissions.md` documents the fuller native contract. Verify the current full local implementation before claiming runtime positioning, timers, display-link behavior, or observer teardown.
- Microphone is covered by passive detection and dictation setup but is not currently an MCP permission row.
- Accessibility and Screen Recording false states map to `Denied`; only Microphone currently exposes `NotDetermined`.
- Overlay positioning does not yet have a receipt as strong as the source audits.
- Development binaries without a host `.app` can fail the drag payload path.

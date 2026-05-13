# Permissions

Script Kit GPUI is a non-activating accessory launcher on macOS, so it must request privacy permissions through Apple's own flows without prompting the user silently or switching the app's activation posture.

The Permission Assistant ports the upstream [permiso](https://github.com/zats/permiso) drag-source pattern into a native AppKit overlay anchored above the live System Settings privacy pane, turning the previous wall-of-text Accessibility / Screen Recording instructions into an in-context, visual flow.

## Key Facts

These points capture the stable permission behavior that the launcher and automation contract depend on.

- The assistant only opens Settings and overlays instructions; detection never prompts. Status reads are passive: `AXIsProcessTrusted()` for Accessibility, `CGPreflightScreenCaptureAccess()` for Screen Recording, `AVCaptureDevice.authorizationStatus(.audio)` for Microphone.
- The overlay panel is a separate `NSPanel` subclass, not the launcher's `WindowKind::PopUp`. It uses `.nonactivatingPanel`, `canBecomeKey = false`, `canBecomeMain = false`, `statusBar` window level, and `orderFrontRegardless` only.
- The drag payload is the host `.app` bundle URL — never the executable inside `Contents/MacOS`. Drag source is a real `NSView` implementing `NSDraggingSource` + `NSPasteboardItemDataProvider`, not GPUI rendering.
- The settings window locator is re-queried every refresh tick — `CGWindowListCopyWindowInfo` results are never cached across display changes, space switches, or app activation.
- Built-in commands `builtin/allow-accessibility` and `builtin/allow-screen-recording` enter the assistant through [[src/builtins/mod.rs#BuiltInFeature]] without calling `prepare_for_submit_hide`.

## Permission Assistant

The Permission Assistant overlays a passive, non-activating drag-source panel above the live System Settings privacy pane so the user can drag the Script Kit `.app` into the Accessibility or Screen Recording allowlist.

It never writes TCC.db, never calls prompting APIs, and never switches the app's activation policy from `.accessory` while the overlay is up.

### Native overlay ownership

`[[src/platform/permiso/overlay_window.rs#PassiveOverlayPanel]]` is the custom `NSPanel` subclass that hosts the overlay. `[[src/platform/permiso/overlay_window.rs#OverlayController]]` owns positioning, the clamp math in `[[src/platform/permiso/overlay_window.rs#anchored_origin]]`, the critically-damped spring animation in `[[src/platform/permiso/overlay_window.rs#spring_frame_at]]`, and teardown.

The overlay panel does not mutate `[[src/platform/app_window_management.rs#ensure_main_panel_configured]]` or any constant in `[[src/platform/panel_invariants.rs]]`. It owns its own AppKit lifetime and cleans up through `[[src/platform/permiso/mod.rs#PermisoHandle]]`'s `Drop` impl, which invalidates the timer and `NSWorkspace` observer first, the `CADisplayLink` second, and orders-out + releases the panel last.

### Passive permission detection

`[[src/platform/permiso_detect.rs#ax_is_trusted]]`, `[[src/platform/permiso_detect.rs#screen_capture_authorized]]`, and `[[src/platform/permiso_detect.rs#microphone_authorized]]` all return a `PermissionStatus` ∈ {`Authorized`, `Denied`, `NotDetermined`, `Unknown`} using non-prompting macOS APIs.

These reads are safe to run during `getState`, built-in filtering, the dictation setup NUX (`[[lat.md/tests/dictation-setup-nux#Non-prompting microphone preflight]]`), screenshot capture (`[[lat.md/automation#Screenshot pixel audit]]`), and assistant presentation. Negative-grep audits guard against regressions reintroducing prompt APIs.

### System Settings locator

`[[src/platform/permiso/locator.rs#settings_window_snapshot]]` resolves the live System Settings window by `ownerPID` of the `com.apple.systempreferences` `NSRunningApplication`, `layer == 0`, `width > 320`, `height > 240`, and picks the largest remaining frame by area.

`[[src/platform/permiso/locator.rs#cg_window_frame_to_appkit]]` converts the CoreGraphics top-left frame to AppKit bottom-left coordinates against the `NSScreen` whose `CGDisplayBounds` intersects the frame by the largest area. The snapshot is re-queried every refresh tick — there is no cache.

### Drag source

`[[src/platform/permiso/drag_source.rs#AppDragSourceView]]` is a native `NSView` subclass implementing `NSDraggingSource` + `NSPasteboardItemDataProvider`. It provides `.fileURL` pasteboard data resolved from `[[src/platform/permiso/host_app.rs#host_app_bundle_url]]`, which must point to a real `.app` directory — never the Contents/MacOS executable.

The drag operation is `.copy`. Drag image is a lock-focus snapshot of the row view. `draggingSession:willBeginAtPoint:` hides the embedded row; `endedAtPoint:operation:` shows it again. The drag never activates Script Kit or any other app.

## Key Files

These files implement the durable Permission Assistant contract.

- [[src/platform/permiso/mod.rs#PermisoAssistant]] — public present/dismiss API and the `PermisoHandle` Drop teardown.
- [[src/platform/permiso/panel.rs#PermisoPanel]] — Accessibility / ScreenRecording enum with `settings_url()` and `display_name()`.

Further modules land with the WP2–WP7 work packages of `.goals/permiso-permission-assistant.md`: `permiso/host_app.rs` (bundle URL + icon), `permiso/locator.rs` (settings window snapshot + CG→AppKit math), `permiso/overlay_window.rs` (`NSPanel` subclass + animation), `permiso/drag_source.rs` (native `NSDraggingSource`), and `permiso_detect.rs` (passive permission status reads). Source-grade wiki refs land alongside those files so `lat check` keeps the architecture page in sync.

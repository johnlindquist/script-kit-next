# Tray menu

The macOS status-bar menu is Script Kit's secondary entry point. It is built once at startup via [[src/tray/mod.rs#TrayManager]] and mutated in place when update state changes.

## Sections

The menu groups related actions into Raycast-style bands separated by `PredefinedMenuItem::separator()`.

1. **Open** — `Open Script Kit` (key equivalent mirrors the user's main launcher hotkey from `config.hotkey`, default ⌘;), `<App Name> Commands`, `Open Notes`, and `Open Agent Chat`
2. **Help** — `Send Feedback…`
3. **Social** — `Follow Us`, `GitHub`, `Discord`
4. **System** — `Settings` (⌘,), `Reload Scripts`, `Check for Updates…`, the dynamic Version row, and `About Script Kit`. Launch-at-Login was removed because it duplicates macOS System Settings → General → Login Items (`smappservice-rs` / `src/login_item.rs` retain the helper for future programmatic use).
5. **Exit** — icon-less `Quit Script Kit`

The headline `Open Script Kit` row uses the embedded `LOGO_SVG` (Script Kit mark) rendered through [[src/tray/mod.rs#menu_icon_from_svg]]. Most other rows carry a template-style `NativeIcon` so AppKit can tint it for normal / highlighted / disabled states. `Settings`, `About Script Kit`, and `Quit Script Kit` intentionally render icon-less because their natural `NativeIcon` variants (`Info`, `PreferencesGeneral`, `StopProgress`) are full-colour status images or add visual noise. `Open Notes` and `Open AI` ship lucide brand glyphs (`assets/icons/notes.svg` = `notepad-text`, `assets/icons/agent_chat.svg` = `bot-message-square`) through the same template-rendered SVG path as the social trio. The social trio (`Follow Us`, `GitHub`, `Discord`) renders brand-correct SVG glyphs from `assets/icons/` (`x_twitter.svg`, `github.svg`, `discord.svg`) inlined via `include_str!` and rendered at 32px through [[src/tray/mod.rs#menu_icon_from_svg]] → `Icon::from_rgba`. The helper rewrites `currentColor` to `white` so the bitmap has full alpha where the glyph lives. Immediately after [[src/tray/mod.rs#TrayManager#create_menu]] returns, [[src/tray/mod.rs#template_menu_items]] reaches into the underlying `NSMenu` via `muda::ContextMenu::ns_menu()` and copies each item's `NSImage` with `setTemplate:YES`, so AppKit auto-tints icons for light, dark, highlighted, and disabled states. The function logs `tray.menu_item_icons_templated` (with the count) on success and `tray.menu_item_template_noop` if every item came back imageless — the white-fill bitmap is the graceful-degrade fallback. If SVG rendering fails at startup, the row falls back to a `NativeIcon` so the menu still works.

[[src/tray/mod.rs#main_shortcut_accelerator]] converts the user's `HotkeyConfig` (modifiers + key strings) into a muda `Accelerator` that lights up the key-equivalent column on the `Open Script Kit` row, so changing the global launcher hotkey updates what the tray displays at next launch. The helper constructs the accelerator from `Code` variants directly instead of depending on muda's string parser for keys like `Semicolon`. The `Current App Commands` row is stored on [[src/tray/mod.rs#TrayManager]] and refreshed on every tray event from `frontmost_app_tracker::get_last_real_app()`, so the next menu open reads as `<localized app name> Commands` when an app has been tracked. The `test_create_menu_uses_native_menu_icons` and `test_brand_icons_render` tests affirm allowed/forbidden NativeIcons and that brand glyphs round-trip cleanly. Per-row SF Symbol templating would require dropping below the `tray-icon` crate to direct objc2 NSMenu construction; that refactor is intentionally deferred.

## URL constants

Pinned destinations exposed as module constants and exercised by `test_tray_urls_are_https_and_pinned`.

[[src/branding.rs#URL_FOLLOW_US]], [[src/branding.rs#URL_GITHUB]], [[src/branding.rs#URL_DISCORD]], and [[src/tray/mod.rs#URL_FEEDBACK]] are the pinned destinations. GitHub and feedback point at `https://github.com/johnlindquist/script-kit-next` and its new-issue route; the test catches typos in the X handle or repo path.

## Update checker

[[src/updates.rs#UpdateState]] is shared via `Arc<RwLock<_>>` between the tray and the dispatcher in [[src/main_entry/app_run_setup.rs]].

A worker thread spawned 5 seconds after launch hits `https://api.github.com/repos/johnlindquist/script-kit-next/releases/latest`, compares `tag_name` to `CARGO_PKG_VERSION` via [[src/updates.rs#version_gt]], and writes the result back. The Version row label flips to "Update Available: vX.Y.Z" once `TrayManager::refresh_version_label` runs on the GPUI main thread.

Tagged releases publish `release-manifest.json` beside `Script-Kit-macos.zip` with SHA256 hashes for each artifact. Future installer wiring will consume that manifest before trusting a downloaded update.

### Why the worker cannot refresh the menu directly

`muda` requires main-thread access to mutate `NSMenuItem`, so the worker only writes state.

The tray-event dispatcher in `app_run_setup.rs` calls `refresh_version_label` after a `cx.background_executor().timer` await that follows a `CheckForUpdates` click — the only point where we know we are back on the GPUI thread.

## Action enum

[[src/tray/mod.rs#TrayMenuAction]] has 14 variants with stable string IDs (`tray.open_script_kit`, `tray.open_notes`, `tray.open_agent_chat`, `tray.reload_scripts`, `tray.follow_us`, `tray.check_for_updates`, etc.).

`from_id` is the only conversion path — adding a row without expanding both `id()` and `from_id()` is what `test_tray_menu_action_id_roundtrip` catches.

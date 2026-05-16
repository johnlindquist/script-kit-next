# 036 Tray Menu, App Lifecycle, Distribution, and Updates

The tray menu is Script Kit's secondary macOS entry point and the visible bridge between app lifecycle, update state, About, and release distribution.


## Executive Summary

Feature 036 covers the macOS status-bar tray menu, tray-driven lifecycle actions, the launcher-native About route, the GitHub-release-backed update checker, release/distribution contracts, and the read-only MCP observation model for Script Kit's own tray menu.


## What Users Can Do

- Open Script Kit from the tray, with the displayed shortcut mirroring the configured launcher hotkey.
- Open current-app commands from the tray after a frontmost app has been tracked.
- Open Notes and Agent Chat.
- Send feedback or open pinned social/project links.
- Open Settings, reload scripts, check for updates, open About, and quit Script Kit.
- See a dynamic Version row that becomes an enabled update row only when a newer release with a downloadable asset exists.
- Use the launcher-native About route to inspect version, update state, links, acknowledgements, and release availability.
- Download/test releases produced by local bundle scripts, CI artifacts, or tagged release workflows.
- Inspect the tray menu model through MCP without opening or clicking the native tray menu.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| Tray menu | Native macOS status-bar menu built through `muda` and `tray-icon`. | `src/tray/mod.rs` |
| `TrayManager` | Owns tray icon, current-app row, version row, and shared update state. | `src/tray/mod.rs#TrayManager` |
| `TrayMenuAction` | Stable action enum and `tray.*` id mapping for native menu events. | `src/tray/mod.rs#TrayMenuAction` |
| Update state | Shared `Arc<RwLock<UpdateState>>` read by tray and About. | `src/updates.rs#UpdateState` |
| About route | Full-window launcher-native route for product identity and update card. | `src/about/render.rs`, `src/app_impl/about_route.rs` |
| Tray observation model | Read-only model used by MCP computer-use tools. | `src/tray/mod.rs#current_tray_menu_observation_snapshot` |
| Release manifest | SHA/size metadata generated beside release artifacts for future installer verification. | `.github/workflows/release.yml` |

### Stable Tray Action Ids

Stable ids are the automation-safe way to refer to tray rows because visible titles can be dynamic or product-copy-specific.

| Variant | Stable id | Meaning |
|---|---|---|
| `OpenScriptKit` | `tray.open_script_kit` | Show main Script Kit window. |
| `OpenCurrentAppCommands` | `tray.open_current_app_commands` | Open Current App Commands for tracked app. |
| `OpenNotes` | `tray.open_notes` | Open Notes. |
| `OpenAgentChat` | `tray.open_agent_chat` | Open AI / Agent Chat. |
| `Settings` | `tray.settings` | Open config in configured editor. |
| `ReloadScripts` | `tray.reload_scripts` | Refresh scripts. |
| `CheckForUpdates` | `tray.check_for_updates` | Start update check. |
| `OpenReleasePage` | `tray.open_release_page` | Open release page when update is available. |
| `SendFeedback` | `tray.send_feedback` | Open feedback URL. |
| `FollowUs` | `tray.follow_us` | Open follow URL. |
| `OpenGitHub` | `tray.open_github` | Open GitHub repo. |
| `JoinDiscord` | `tray.join_discord` | Open Discord. |
| `OpenAbout` | `tray.open_about` | Open About surface. |
| `Quit` | `tray.quit` | Shut down Script Kit. |

## Entry Points

| Entry point | User intent | Expected target |
|---|---|---|
| Tray icon/menu | Secondary global app entry | Native tray menu sections |
| `Open Script Kit` | Show main window | Runtime window/show path |
| `<App> Commands` | Open commands for last tracked real app | Current-app commands view |
| `Open Notes` | Open Notes host | Notes window open path |
| `Open AI` / Agent Chat | Open agent chat | Main window ACP tab |
| `Settings` | Edit config | Config editor path |
| `Reload Scripts` | Refresh script catalog | Runtime script refresh |
| `About Script Kit` | Open launcher-native About | `open_about_surface` |
| `Quit Script Kit` | Shutdown app | Tray dispatcher shutdown path |
| MCP tray tools | Observe menu model | Read-only tray model handlers |

## User Workflows

### Startup And Tray Construction


### Open Script Kit From Tray


### Open Current App Commands

Before each tray event dispatch, the dispatcher refreshes the current-app row from the frontmost app tracker because `muda` does not expose a menu-will-open hook. The row reads as `<localized app name> Commands` when a real app has been tracked and falls back to `Current App Commands...` otherwise.

### Open Notes Or Agent Chat

Open Notes calls the Notes window path without launcher restore. Open Agent Chat shows the main window and opens the AI/ACP tab. These are tray entry points into existing surface owners, not separate tray-owned surface implementations.

The visible row is **Open AI**, while the stable id is `tray.open_agent_chat`. Agents and tests should key off the id, not the title.

### Check For Updates


### Open Release Page


### Open About From Tray

The tray shows the main window, waits briefly, and calls `open_about_surface` with the shared update-state handle. About owns focus without exposing the launcher filter and restores the prior route on explicit dismissal.

### Quit Script Kit

Quit sets the shutdown flag, kills tracked child processes, removes the main PID file, calls `cx.quit()`, and breaks the tray receiver loop. The broader shutdown monitor performs the same cleanup path for non-tray shutdown triggers.

### Observe Tray Through MCP

MCP clients call `computer/list_tray_menu`, `computer/get_tray_menu_item`, or `computer/get_tray_menu_item_by_id`. These tools read Script Kit's own tray menu model and never open the native tray menu, click status items, execute actions, enumerate global menu extras, or request permissions.

`computer/get_tray_menu_item` returns `found`, `sectionNotFound`, or `itemNotFound` from `{sectionIndex,itemIndex}`. `computer/get_tray_menu_item_by_id` returns `found` or `notFound` from `{id}`. Empty ids and unknown fields are rejected by closed schemas.

### Build And Publish Releases

Local release work builds the release binary, bundles the macOS app, and verifies bundle contents. Tagged releases validate version/tag parity, run locked gates, sign, notarize, staple, assess with Gatekeeper, generate `release-manifest.json`, and upload both the zip and manifest.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open launcher | Tray menu | Open section | Click Open Script Kit | Tray dispatcher `OpenScriptKit` | Main window shown | Tray action id tests |
| Open current-app commands | Tray menu | Open section | Click current app row | Refresh current app label, dispatch action | Current-app commands opens or toast/logs | Tray model/current app tests |
| Open Notes | Tray menu | Open section | Click Open Notes | Tray dispatcher Notes action | Notes window opens | Tray action contract |
| Open Agent Chat | Tray menu | Open section | Click Open AI | Tray dispatcher Agent Chat action | Main window ACP tab opens | Tray action contract |
| Send feedback | Tray menu | Help section | Click Send Feedback | External URL open | Feedback URL opens | Pinned URL tests |
| Open social link | Tray menu | Social section | Click Follow/GitHub/Discord | External URL open | Pinned URL opens | HTTPS/pinned URL tests |
| Open Settings | Tray menu | System section | Click Settings | Config editor path | `config.ts` opens in editor | Tray action contract |
| Reload scripts | Tray menu | System section | Click Reload Scripts | `view.refresh_scripts` | Script catalog refreshes | Runtime dispatch proof |
| Open About | Tray menu | System section | Click About | `open_about_surface` | About route opens with shared update state | About contract tests |
| Quit app | Tray menu | Exit section | Click Quit | Shutdown flag, child cleanup, PID cleanup, `cx.quit` | App exits | Shutdown path audit |
| Inspect tray model | MCP | Read-only model | Tool call | `current_tray_menu_observation_snapshot` | JSON menu model returned | MCP source audits |
| Build release | GitHub tag | CI release workflow | Push `v*` tag | Release workflow | Signed/notarized zip and manifest | Release workflow gates |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Tray initializing | App startup | Tray idle | Waits for initial render/window work before tray creation. |
| Tray idle | Tray created | Menu action, delayed update check | Version row reflects Idle/UpToDate/Error/Available state. |
| Update checking | Startup worker or Check for Updates | UpToDate, Available, Error | Existing check prevents duplicate checking. |
| Update available | `pick_release` finds newer tag with asset | Open release page, future check | Version row is enabled and has release URL. |
| Update unavailable | UpToDate or newer tag without asset | Future check | Newer tag without downloadable asset is not advertised. |
| About route | Tray About or automation route | Prior route | Explicit dismissal restores route-specific focus. |
| Shutdown requested | Quit or shutdown monitor | Process exit | Shutdown flag, child cleanup, PID cleanup happen before quit. |
| Release workflow | Version tag pushed | Published release or failed gate | Version/tag parity, locked checks, signing, notarization, stapling, assessment. |
| MCP tray observation | Tool call | JSON response | Read-only, own-tray-model only, no native clicking or actions. |

### Update Row Labels

The Version row is always present but disabled unless a release URL is available.

| Update state | Label | Enabled |
|---|---|---|
| `Idle` | `Version <current>` | false |
| `UpToDate` | `Version <current>` | false |
| `Checking` | `Checking for updates... (v<current>)` | false |
| `Error(_)` | `Version <current> (update check failed)` | false |

## Visual And Focus States


## Keystrokes And Commands

| Key/command | Context | Behavior |
|---|---|---|
| Configured launcher hotkey | Tray Open Script Kit row | Displayed as key equivalent via `main_shortcut_accelerator` at next launch. |
| Escape | About route | Dismisses About back to prior route. |
| Tab / Shift+Tab | About route | Walks close, link, update, and acknowledgements controls. |
| Enter / Space | About focused control | Activates the focused About control. |
| `computer/list_tray_menu` | MCP | Returns read-only tray model. |
| `computer/get_tray_menu_item` | MCP | Returns section/item by index. |
| `computer/get_tray_menu_item_by_id` | MCP | Returns item by stable id. |

## Actions And Menus

The tray action enum currently has 14 stable `tray.*` ids. The core invariant is that every native row, action enum variant, string id, `from_id` conversion, `all()` list, observation model row, and dispatcher branch stay aligned.




## Automation And Protocol Surface

| Surface | Target/proof | Notes |
|---|---|---|
| Tray model | `computer/list_tray_menu` | Read-only own-tray model; does not open native menu. |
| Tray item by index | `computer/get_tray_menu_item` | Closed section/item lookup with found/not-found statuses. |
| Tray item by id | `computer/get_tray_menu_item_by_id` | Stable id lookup using tray action ids. |
| About route | `openAbout`, About state/elements, source audits | Opens About without tray menu for state-first proof. |
| Update picker | Source-audit tests around `pick_release` and manifest SHA lookup | Proves release availability semantics. |
| Distribution | Shell scripts and GitHub workflow gates | Packaging/signing/notarization proof, not runtime UI proof. |

## Data, Storage, And Privacy Boundaries

- Tray observations expose only Script Kit's own tray model, not global menu extras or native status-item click handles.
- Update checker reads GitHub release metadata over HTTPS and stores only `UpdateState`.
- Release manifest records artifact name, platform, SHA256, and size; current in-app updater does not yet verify downloads against it.
- About and tray share branding constants and pinned URLs to avoid destination drift.
- Launch-at-login helper code exists but is not active tray UI.
- Distribution workflows use signing/notarization credentials in GitHub Actions secrets; local agents should not inspect or require those secrets.

## Error, Empty, Loading, And Disabled States

- Tray model before initialization returns an idle snapshot with a warning.
- Current-app row falls back to `Current App Commands...` when no real app is tracked.
- Shortcut conversion can fail. Open Script Kit remains enabled, but the key equivalent is omitted rather than showing a misleading shortcut.
- SVG render failures are non-fatal for rows with fallback icons.
- Check for Updates shows Checking state and disables version/open-release behavior until a result lands.
- Newer release without downloadable assets is treated as UpToDate.
- Version row is disabled unless update state is Available.
- About checking disables the update button; errors remain retryable.
- MCP tray item lookups return sectionNotFound, itemNotFound, or notFound rather than executing anything.
- Bad MCP input is rejected by closed input schemas.

## Code Ownership

| Behavior | Owner files/tests |
|---|---|
| Tray menu, icons, action ids, observations | `src/tray/mod.rs` |
| Tray startup and dispatch loop | `src/main_entry/app_run_setup.rs`, `src/main_entry/runtime_tray_hotkeys.rs` |
| Shutdown cleanup | `src/main_entry/runtime_shutdown.rs` |
| Update checking and release picking | `src/updates.rs` |
| About route state/focus | `src/app_impl/about_route.rs` |
| About rendering | `src/about/render.rs`, `src/about/mod.rs` |
| Shared branding URLs/logo | `src/branding.rs` |
| Launch-at-login helper | `src/login_item.rs` |
| MCP tray observation tools | `src/mcp_computer_use_tools.rs`, `src/mcp_computer_use/handlers.rs` |
| Bundle/release metadata | `Cargo.toml`, `Makefile`, `.github/workflows/ci.yml`, `.github/workflows/release.yml` |
| Packaging verification | `scripts/verify-macos-bundle.sh`, `scripts/verify-release-version.sh`, `scripts/verify.sh` |
| About tests | `tests/about_surface_contract.rs`, `tests/about_surface_source_audit.rs` |
| MCP tray source audits | `tests/source_audits/mcp_computer_list_tray_menu_contract.rs`, `tests/source_audits/mcp_computer_get_tray_menu_item_contract.rs`, `tests/source_audits/mcp_computer_get_tray_menu_item_by_id_contract.rs` |
| Update picker tests | `tests/source_audits/update_picker_contract.rs` |

## Invariants And Regression Risks

- Native tray item mutation must happen on the GPUI/main-thread path, not inside update worker threads.
- Native menu order and MCP observation section order must match.
- Tray observation rows must not expose click, execute, action, or event handles.
- MCP tray tools are read-only observations, not native tray automation.
- A release is advertised only when a newer version also has a downloadable asset.
- About must not expose the launcher filter while active and must restore prior route focus on dismissal.
- Quit must set the shutdown flag before child-process cleanup, PID cleanup, `cx.quit`, and event-loop break.
- Release manifest generation is not the same as in-app installer verification.
- Launch-at-login helper code must not be documented as active tray UI unless the product surface returns.
- `src/main_entry/runtime_tray_hotkeys.rs` may contain legacy tray names; prioritize `src/tray/mod.rs` and `src/main_entry/app_run_setup.rs` as current dispatch evidence unless the compiled source proves otherwise.

## Verification Recipes


```bash
cargo test tray_menu_action_id_roundtrip
cargo test test_tray_menu_action_id_roundtrip
cargo test --test source_audits mcp_computer_list_tray_menu_contract
cargo test --test source_audits mcp_computer_get_tray_menu_item_contract
cargo test --test source_audits mcp_computer_get_tray_menu_item_by_id_contract
```


```bash
cargo test tray_menu_action_ids_are_unique
cargo test tray_menu_action_ids_are_prefixed
cargo test tray_menu_action_from_id_unknown
cargo test tray_menu_action_all_count
cargo test tray_menu_observation_contains_all_tray_actions
cargo test tray_menu_observation_sections_match_create_menu_order
cargo test tray_menu_observation_ids_are_unique
cargo test tray_menu_observation_current_app_title_uses_frontmost_tracker_fallback
cargo test tray_menu_observation_version_row_reflects_update_state
cargo test tray_menu_observation_has_no_click_or_execute_fields
cargo test test_create_menu_uses_native_menu_icons
cargo test test_brand_icons_render
cargo test test_main_shortcut_accelerator_default
cargo test test_tray_urls_are_https_and_pinned
```


```bash
cargo test --test about_surface_contract
cargo test --test about_surface_source_audit
cargo test --test source_audits update_picker_contract
```


```bash
bash scripts/verify-release-version.sh
bash scripts/verify-macos-bundle.sh
make verify
```


```bash
make ship-check
```


```bash
npm run build
source checks
git diff --check -- feature-map FEATURE_MAP.md feature_explorer removed-docs
```

## Agent Notes

Use `.agents/skills/platform-windowing-macos/SKILL.md` as the primary owner, with `$protocol-automation` for MCP observation boundaries and `$testing-quality-gates` for selecting checks. Treat `src/tray/mod.rs`, `src/updates.rs`, `src/app_impl/about_route.rs`, and `removed-docs` as stronger source of truth than stale-looking tray action names in older runtime excerpts.

Use stable `tray.*` ids, not visible tray labels, in automation. Titles can be dynamic (`<App> Commands`) or product-copy-specific (`Open AI`). Do not infer dictation tray behavior from dictation protocol resources; the tray menu snapshot does not show a dictation row. Do not add a tray runtime bridge path because source audits explicitly forbid `ComputerUseRuntimeBridge` tray observation methods.

## Related Features

- [[tray-menu]]
- [[about]]
- [[distribution]]
- [[protocol]]
- [[windowing]]
- [[verification]]

## Open Questions And Gaps

- Manual update check currently waits a fixed interval before refreshing the tray label; a future UX pass may need a main-thread-safe completion signal.
- The current updater opens release URLs but does not download, verify, install, replace, or relaunch the app.
- Launch-at-login helper code remains, but no active tray UI exposes it.
- Native global status-item discovery and clicking remain outside MCP tray observation.
- Some runtime excerpts may contain stale action names; verify compilation and current enum ownership before changing dispatch.
- Full global hotkey registration for launcher, Notes, AI, and dictation is outside the focused tray bundles. The tray pass proves shortcut display conversion, not every registration path.
- Settings `destination_kind` in the tray observation model may not exactly match external editor dispatch behavior; verify before treating it as a strict automation contract.
- About opened from tray receives shared update state, while automation `openAbout` may use idle/default update state. About update-state proof can differ by entry route.

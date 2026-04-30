# Architecture

Script Kit GPUI is split into a Rust launcher shell, prompt and utility view modules, a protocol boundary for script communication, and separate AI and Notes subsystems.

## Key Facts
The main shell is a routed Rust app, not a single flat window implementation.

- `src/main_sections/` holds the shared app state, view routing, and render dispatch that drive the launcher shell.
- `src/app_impl/` owns startup, keyboard routing, surface transitions, attachment portals, and most of the user-facing routing logic.
- `src/app_execute/` contains built-in execution and utility-view openers, including file search and terminal-style surfaces.
- `src/ai/` contains ACP chat, the harness/context plumbing, and the compatibility-named Tab AI code that still feeds ACP.
- `src/notes/` is a separate window subsystem rather than another `AppView` branch inside the launcher shell.
- `src/protocol/` and `src/mcp_resources/` define the script and AI automation boundary.
- Some shared helpers are launcher-owned even when the library target compiles them. `src/scrolling/selection_owned.rs` is consumed by launcher surfaces, while `src/browser_tabs.rs` uses its JXA tab-list path at runtime and keeps the AppleScript builder as test-only coverage.

## Key Files
These are the live files that define the routing and module boundaries.

- [src/main_sections/app_view_state.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/app_view_state.rs) - The `AppView` enum that names every first-class launcher surface.
- [src/main_sections/render_impl.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/render_impl.rs) - Render dispatch for the current `AppView`.
- [src/app_impl/startup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/startup.rs) - Main window startup and key interception.
- [src/app_impl/tab_ai_mode/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/tab_ai_mode/mod.rs) - ACP entry paths, harness routing, and Tab/Shift+Tab AI behavior.
- [src/app_impl/tab_ai_mode/source_classification.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/tab_ai_mode/source_classification.rs) - Tab AI source-type classification and apply-back hint delegation.
- [src/app_impl/attachment_portal.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/attachment_portal.rs) - ACP attachment portal open/return flow for file search, clipboard history, notes, and related targets.
- [src/app_execute/builtin_execution.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/builtin_execution.rs) - Built-in commands and the AI-related execution paths.
- [src/app_execute/utility_views.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/utility_views.rs) - File search and quick-terminal utility surface helpers.
- [src/mcp_resources/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/mcp_resources/mod.rs) - MCP resource registry for current state, scripts, scriptlets, and context.
- [src/notes/window.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window.rs) - Separate Notes window host and embedded ACP surface.
- [src/scrolling/selection_owned.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scrolling/selection_owned.rs) - Selection reanchor helpers shared by launcher-owned scrolling surfaces.
- [src/browser_tabs.rs](/Users/johnlindquist/dev/script-kit-gpui/src/browser_tabs.rs) - Browser tab listing and activation helpers, with JXA used for live tab enumeration.

## Launcher Startup Source Of Truth
The live launcher startup implementation is `src/app_impl/startup.rs`, wired by `src/app_impl/mod.rs`.

The `src/app_impl/startup_new_*.rs` files are legacy source-audit parity fragments. They are not module-wired by `src/app_impl/mod.rs`, so agents must not treat them as the production startup path when changing launcher behavior.

`tests/launcher_startup_entrypoint_contract.rs` pins this distinction: the module tree imports `startup.rs`, the live file owns `ScriptListApp::new`, and the architecture page names `startup_new_*` as legacy source-audit fragments.

## App State Domains
`ScriptListApp` remains the launcher state root, but related state should move into named domain structs when that makes ownership easier to infer.

AURP-07 starts with [[src/main_sections/app_state.rs#MainMenuRenderDiagnosticsState]], which owns the main script-list render log dedupe state and input-to-render performance timing receipt. Production code should access these fields through `main_menu_render_diagnostics`, not as loose `ScriptListApp` fields.

[[tests/app_state_domain_structs_contract.rs#main_menu_render_diagnostics_have_a_named_domain_owner]] pins the first domain grouping so future agents can detect when diagnostics state drifts back into the broad state root.

AURP-09 adds [[src/main_sections/app_state.rs#MainMenuFallbackState]] for no-match main-menu fallback commands. Filtering activates it, execution reads the selected fallback through it, and keyboard/stdin fallback navigation moves its selected index through owner methods.

AURP-12 adds [[src/main_sections/app_state.rs#MainMenuResultCacheState]] for main-menu filtered and grouped-result caches. Filtering owns mutation and invalidation, while render, navigation, preflight, and Tab AI read grouped cache state through `main_menu_result_caches`.

AURP-13 keeps that owner but moves consumers to behavior-named methods for cache freshness, snapshots, grouped rows, flat results, selectable bounds, and cache-key diagnostics. [[tests/main_menu_result_cache_domain_contract.rs#grouped_cache_readers_use_behavior_named_accessors]] pins those read paths.

AURP-17 extends [[src/main_sections/app_state.rs#MainMenuResultCacheState]] with selected-result resolution helpers. Exact row lookup, coerced executable selection, forward preflight lookup, and visible-result iteration now have separate method names so action labels, execution, preflight, preview, and Tab AI do not open-code grouped-index to flat-result mapping.

[[tests/main_menu_result_cache_domain_contract.rs#result_caches_have_a_named_domain_owner]] pins this owner so future cache work does not scatter the keys and cached result vectors back across the state root.

## Source Documents
These code files are the source of truth for the current architecture description.

- [src/main_sections/app_view_state.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/app_view_state.rs)
- [src/main_sections/render_impl.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/render_impl.rs)
- [src/app_impl/startup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/startup.rs)
- [src/app_impl/tab_ai_mode/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/tab_ai_mode/mod.rs)
- [src/app_impl/tab_ai_mode/source_classification.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/tab_ai_mode/source_classification.rs)
- [src/app_impl/attachment_portal.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/attachment_portal.rs)
- [src/app_execute/builtin_execution.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/builtin_execution.rs)
- [src/app_execute/utility_views.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/utility_views.rs)
- [src/mcp_resources/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/mcp_resources/mod.rs)
- [src/notes/window.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window.rs)
- [src/scrolling/selection_owned.rs](../src/scrolling/selection_owned.rs)
- [src/browser_tabs.rs](../src/browser_tabs.rs)

## Related Pages
These pages cover the adjacent product and contract details.

- [overview](./overview.md)
- [scripting](./scripting.md)
- [workspace](./workspace.md)
- [protocol](./protocol.md)
- [builtins](./builtins.md)
- [surfaces](./surfaces.md)
- [ai-context](./ai-context.md)
- [acp-chat](./acp-chat.md)
- [notes](./notes.md)
- [windowing](./windowing.md)

## Surface Routing
These routes are the current interaction paths that matter when you follow a keystroke through the app.

- `AppView` is the state machine for the main shell. Render dispatch and keyboard interceptors branch on it directly.
- `ScriptList` is the normal landing surface. From there the app can open utility views, built-ins, or AI paths.
- `Tab` from `ScriptList` routes into ACP context capture or AI handoff logic; `Shift+Tab` is still reserved in some surfaces such as file search and the AI harness path.
- ACP chat consumes `Tab` and `Shift+Tab` locally when it is open.
- `QuickTerminalView` receives raw Tab bytes so the PTY handler can own terminal navigation and shell interaction.
- Attachment portals temporarily replace the visible surface and then restore the originating ACP context on return. While a portal is active, `Tab` and global `Cmd+Enter` must NOT auto-submit to ACP — the main-menu launcher behavior is suppressed so the portal keeps local key ownership. The guard lives centrally in [[src/app_impl/tab_ai_mode/mod.rs#ScriptListApp#try_route_plain_tab_to_acp_context_capture]] and [[src/app_impl/tab_ai_mode/mod.rs#ScriptListApp#try_route_global_cmd_enter_to_acp_context_capture]], both of which early-return when [[src/app_impl/attachment_portal.rs#ScriptListApp#is_in_attachment_portal]] is true. The portal host also snapshots shared launcher state before entry so ScriptList filter, selection, and focus survive a portal round-trip, and it only forces the old width back when the user did not manually resize during portal browsing.
- The Notes window is a separate host that can surface its own editor or an embedded ACP session.

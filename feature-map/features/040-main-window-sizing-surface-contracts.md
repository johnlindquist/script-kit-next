# 040 Main Window Sizing and Surface Contracts

Main-window sizing and surface contracts define how Script Kit's primary GPUI window changes size, identity, focus, footer, and automation semantics as users move between launcher, built-in, prompt, and assistant surfaces.

Raw Oracle reference: [answer](../raw-oracle/040-main-window-sizing-surface-contracts/answer.md), [prompt](../raw-oracle/040-main-window-sizing-surface-contracts/prompt.md), [bundle map](../raw-oracle/040-main-window-sizing-surface-contracts/bundle-map.md), [full log](../raw-oracle/040-main-window-sizing-surface-contracts/output.log), [session metadata](../raw-oracle/040-main-window-sizing-surface-contracts/session.json).

## Executive Summary

Feature 038 covers the cross-cutting contract that keeps the main window understandable and automatable while it moves between the full launcher, Mini main window, script prompts, built-in filterable surfaces, split preview/detail panes, embedded ACP, and inline Mini AI.

The governing rule is: surface layout determines size, and `SurfaceKind` determines behavior. A single-column command surface stays Mini. A real list-plus-preview/detail surface is Full. Agents should not infer width from row count, command importance, trigger source, shortcut source, render file name, or a stale screenshot.

## What Users Can Do

- Open the standard full launcher and compact Mini main window.
- Open Mini single-column built-ins such as App Launcher, Window Switcher, Browser Tabs, Emoji Picker, Process Manager, Settings, Current App Commands, Kit Store, Design Gallery, and related command surfaces.
- Open Full split preview/detail surfaces such as Clipboard History, full File Search, Theme Chooser, ACP History, Browser History, Dictation History, Notes Browse, SDK Reference, and Script Template Catalog.
- Open MiniPrompt without inheriting full prompt width.
- Use inline Mini AI or mini-hosted ACP in compact mode, while full ChatPrompt and ACP use full prompt sizing.
- Filter the current surface without accidental width changes.
- Use Escape, Cmd-W, hide, reset, and go-back paths without leaking stale full-width hidden bounds into automation receipts.
- Open shared actions popups while preserving the host surface identity.
- Trigger built-ins over stdin and observe automation metadata re-key to the real active surface.
- Inspect `getState.surfaceContract`, `activePopupContract`, `activeFooter`, and `getElements` instead of relying on screenshots for semantic proof.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| Main window | The primary launcher/prompt window that owns `AppView`. | `src/main_sections/app_view_state.rs` |
| `MainWindowMode` | App-level Mini vs Full mode state for the main window. | `src/main_sections/app_view_state.rs`, `src/app_impl/ui_window.rs` |
| `ViewType` | Low-level resize target such as `ScriptList`, `MiniMainWindow`, `MiniPrompt`, `MiniAiChat`, `DivPrompt`, `EditorPrompt`, or `TermPrompt`. | `src/window_resize/mod.rs` |
| Mini main window | Compact launcher mode: `MainWindowMode::Mini` plus `ViewType::MiniMainWindow`. | `src/app_execute/builtin_execution.rs#open_mini_main_window` |
| MiniPrompt | Compact script prompt route, distinct from Mini main window. | `src/app_impl/ui_window.rs` |
| Mini AI chat | Compact AI/ACP sizing via `ViewType::MiniAiChat` when `main_window_mode` is Mini. | `src/app_impl/ui_window.rs#compact_ai_view_type_for_mode` |
| `calculate_window_size_params` | Follow-up resize authority that derives `(ViewType, item_count)` from the active `AppView`. | `src/app_impl/ui_window.rs` |
| `SurfaceKind` | Payload-free behavioral identity for an `AppView`. | `src/main_sections/app_view_state.rs` |
| `LauncherSurfaceContract` | Behavior vocabulary: focus, keyboard, actions, proof, visual, dismiss, footer, and automation semantic surface policy. | `src/main_sections/app_view_state.rs` |
| Native footer surface | AppView-specific footer identity exposed through generated contracts and runtime receipts. | `src/main_sections/app_view_state.rs#AppView#native_footer_surface` |
| Surface matrix | Generated JSON contract for agents. | `docs/ai/contracts/surface-contracts.json` |

### Main Window Versus Secondary Windows

This chapter covers the primary main window and top-level `AppView` routes. It does not own detached ACP window sizing, detached actions popup geometry, shortcut recorder popup geometry, or AppKit activation posture. Secondary window diagnostics intentionally omit `surfaceContract` because they are not main-window `AppView` receipts.

### Mini Main Window Versus Other Mini Surfaces

Mini main window is not MiniPrompt, MicroPrompt, inline Mini AI, Quick Terminal, detached popups, or secondary windows. Mini main window uses `MainWindowMode::Mini` and `ViewType::MiniMainWindow`; MiniPrompt uses `ViewType::MiniPrompt`; inline Mini AI and mini-hosted ACP use `ViewType::MiniAiChat`.

## Entry Points

| Entry point | User intent | Expected target |
|---|---|---|
| Full launcher open | Open the standard main menu | `MainWindowMode::Full`, `ViewType::ScriptList` |
| Mini main window command | Open compact launcher | `MainWindowMode::Mini`, `ViewType::MiniMainWindow` |
| Built-in filterable command | Open a built-in list surface | Mini or Full based on layout contract |
| `triggerBuiltin` | Open built-in over stdin | Route mutation, deferred resize, automation re-key |
| Tray/current-app command | Open current app commands | Mini single-column surface |
| Special-character handoff | Route from launcher text into a child surface | Active `AppView` determines follow-up sizing |
| Prompt creation | Open SDK prompt | Prompt-specific `ViewType` |
| Filter change | Recompute rows | Same Mini/Full class preserved |
| Escape/Cmd-W/go-back | Dismiss or return | Surface dismiss policy plus reset/go-back owner |
| `getState` | Inspect active main-window behavior | `stateResult.surfaceContract` |
| Actions popup | Open shared actions | Host `surfaceContract` plus `activePopupContract` |

## User Workflows

### Open Mini Main Window

The user opens the Mini main window from a utility command or equivalent entry. `open_mini_main_window` clears filter state, restores ScriptList focus, switches mode to Mini, clears hover, selects the first selectable row, invalidates grouped results, computes grouped item count, resizes through `ViewType::MiniMainWindow`, and notifies.

The important proof is not just window width. The source audit must show the helper uses the shared Mini path, and runtime proof should show Mini mode, Mini width, ScriptList semantic surface, and footer state.

### Open A Mini Built-In

The user opens a single-column built-in such as Current App Commands, Settings, Emoji Picker, or Process Manager. The open helper passes `expanded = false`, sets `MainWindowMode::Mini`, resizes to `ViewType::MiniMainWindow`, focuses the main filter, and preserves Mini classification during follow-up `update_window_size_deferred`.

The current-app commands path is a high-risk example: it opens through a tray/current-app capture helper, then calls deferred resize. The contract test pins that the deferred resize arm resolves `CurrentAppCommandsView` to `MiniMainWindow`, not `ScriptList`.

### Open A Full Split Surface

The user opens a surface with a real list-plus-preview/detail layout, such as Clipboard History, full File Search, Theme Chooser, ACP History, Browser History, Dictation History, Notes Browse, SDK Reference, or Script Template Catalog. The open path sets Full mode and resizes through `ViewType::ScriptList`.

Full width is justified by layout, not by command importance or data volume. A surface with many rows remains Mini if it is still single-column.

### Open Prompt Surfaces

The user runs an SDK script that opens a prompt. Prompt routes map to prompt-specific sizing:

- `MiniPrompt` -> `ViewType::MiniPrompt`.
- `MicroPrompt` -> compact no-choice prompt sizing.
- `ArgPrompt` -> no-choice or choice-list sizing.
- `DivPrompt`, `FormPrompt`, `PathPrompt`, `EnvPrompt`, `DropPrompt`, `TemplatePrompt`, and ConfirmPrompt -> full prompt sizing.
- `ChatPrompt` and `AcpChatView` -> `MiniAiChat` in Mini mode, `DivPrompt` in Full mode.
- Editor and terminal prompts use their own full content view types.

### Trigger Built-In Over Protocol

An agent sends `triggerBuiltin`. The route dispatcher mutates `current_view`, then the post-dispatch helper re-keys the main automation surface from the new `AppView`. The stdin path should not rebuild the whole automation window or reset bounds/focus/title because it only owns route dispatch and semantic re-keying.

### Inspect Active Surface Contract

An agent calls `getState`. For main-window targets, `stateResult.surfaceContract` exposes the active surface kind, vocabulary, focus policy, keyboard policy, actions policy, proof policy, visual policy, dismiss policy, native footer surface, and automation semantic surface. If a shared actions popup is attached, `activePopupContract` exposes popup behavior without replacing the host surface.

### Hide, Reset, Or Go Back

The user presses Escape/Cmd-W or an agent hides/resets the main window. The implementation must avoid a wide hidden-bound leak by establishing hidden state before reset paths make `ScriptList` current. Reset paths restore Full mode where appropriate, close popups, clear stale child state, and re-key automation back to `scriptList`.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open full launcher | Launcher show | ScriptList Full | Hotkey/click/protocol | Full show path, `ViewType::ScriptList` | Wide launcher | Bounds + `surfaceContract` |
| Open Mini launcher | Mini command | ScriptList Mini | Built-in command | `open_mini_main_window` | 480px Mini shell | `mini_main_window` source audit |
| Open current-app commands | Tray or `triggerBuiltin` | CurrentAppCommandsView | Click/protocol | `open_current_app_commands_from_tray`, `update_window_size_deferred` | Mini current-app commands | `trigger_builtin_current_app_commands_contract` |
| Open single-column built-in | Built-in command | Filterable view | Enter/click/protocol | `open_builtin_filterable_view(expanded=false)` | Mini list | `getState.surfaceContract` + matrix |
| Open split built-in | Built-in command | Preview/detail view | Enter/click/protocol | `open_builtin_filterable_view(expanded=true)` | Full split pane | Source audit + runtime bounds |
| Open MiniPrompt | SDK prompt | `AppView::MiniPrompt` | Script call | `mini_prompt_view_type` | Compact prompt | Mini-window contract tests |
| Open embedded ACP in Mini | ACP entry from Mini | `AcpChatView` | Cmd+Enter/action | `compact_ai_view_type_for_mode` | Mini AI chat | `stateResult.surfaceContract` |
| Filter rows | Active list | Same surface | Type | Filter update, `calculate_window_size_params` | Rows update, width class stable | State/elements receipts |
| Open actions | Host surface | Attached popup | Cmd+K/footer | Actions dialog host map | Host preserved, popup contract active | `activePopupContract` |
| Hide/reset | Any main route | Closing/resetting | Escape/Cmd-W/protocol | dismiss policy, lifecycle reset | Hidden or restored state | `windowVisible:false`, surface reset proof |
| Inspect contract | Main window | Any route | `getState` | Query ops constructor | Surface policy snapshot | `stateResult.surfaceContract` |
| Inspect automation surface | Main window | Routed built-in | `listAutomationWindows` | automation surface registry | Semantic surface matches route | Semantic re-key tests |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Full launcher | App launch/show/reset | Mini launcher, built-in, prompt, hide | `ViewType::ScriptList`, Full width. |
| Mini launcher | Mini command | Full launcher, Mini built-in, Mini prompt, hide | `MainWindowMode::Mini`, Mini width. |
| Mini filterable built-in | Built-in open/triggerBuiltin | Filter update, actions popup, go-back | Single-column layout, deferred resize stays Mini. |
| Full split built-in | Built-in open/triggerBuiltin | Filter update, actions popup, go-back | Preview/detail layout, Full width. |
| Prompt surface | SDK prompt | submit/cancel/reset | Prompt-specific `ViewType`. |
| Embedded ACP/Chat | ACP entry | Mini or Full AI mode, actions, close | `compact_ai_view_type_for_mode` branches on mode. |
| Actions popup attached | Cmd+K/footer actions | close/execute/filter | Host `surfaceContract` remains active; popup uses `activePopupContract`. |
| Hidden main window | Escape/Cmd-W/protocol hide | next show | Hidden bounds must not leak wrong Mini/Full size. |
| Re-key pending | Route mutation | semantic surface synced | Re-key after `current_view` mutation, not before. |

## Visual And Focus States

- Full launcher: wide ScriptList surface with launcher-owned input and optional preview/info pane.
- Mini main window: compact fixed-height launcher shell with Mini width, header, list content, hint/footer strip, and native footer sync.
- MiniPrompt: compact script prompt, visually related but not the Mini main-window route.
- Mini AI/ACP: compact assistant panel, using `MiniAiChat` sizing.
- Full prompt/content surfaces: full-height prompt content, editor, terminal, form, div, path, env, drop, template, and confirm routes.
- Split preview/detail built-ins: wide list plus preview/detail pane.
- Attached actions popup: overlay/popup contract layered on top of the host surface, not a replacement for host identity.
- Native footer: AppView-specific footer id; AppKit host and GPUI fallback must not stack or drift.

## Keystrokes And Commands

| Key/command | Context | Behavior |
|---|---|---|
| Launcher hotkey/show | Main window | Opens launcher in its intended mode. |
| Built-in command Enter/click | Launcher row | Opens Mini or Full according to layout contract. |
| `triggerBuiltin` | Protocol | Dispatches route, applies resize owner, re-keys semantic surface. |
| Text input | Filterable surface | Recomputes rows without changing width class. |
| Cmd+K | Host surface | Opens shared actions while preserving host surface contract. |
| Escape | Host surface | Uses popup-first and surface dismiss policy before hide/reset. |
| Cmd-W | Explicit surfaces | Closes or lets view handle according to surface contract. |
| `getState` | Protocol | Returns `surfaceContract`; returns `activePopupContract` when applicable. |
| `getElements` | Protocol | Proves list rows, footer buttons, and semantic elements. |
| `listAutomationWindows` | Protocol | Reports current main-window semantic surface. |

## Actions And Menus

Actions menus depend on host identity. When shared actions are open, the host surface remains the active main route and `activePopupContract` describes the overlay. Agents should not treat the actions popup as changing the main `AppView`.

Footer actions are also surface-specific. The native footer surface id comes from `AppView::native_footer_surface()`, not from a generic `SurfaceKind` fallback. For footer bugs, inspect `activeFooter`, footer buttons, native-footer host state, and GPUI fallback state before taking screenshots.

## Automation And Protocol Surface

| Surface | Target/proof | Notes |
|---|---|---|
| Active main route | `getState.surfaceContract` | Best proof for `SurfaceKind`, policies, footer id, semantic surface. |
| Attached actions | `getState.activePopupContract` | Popup policy separate from host route. |
| Footer | `getState.activeFooter`, `getElements` | Proves owner, buttons, disabled reasons, native/fallback mismatch. |
| Window semantics | `listAutomationWindows.semanticSurface` | Must match active route after re-key. |
| Filterable surfaces | `scripts/agentic/filterable-surface-matrix.ts` | Matrix validates prompt type, surface kind, semantic tag, list id, and visible rows. |
| Surface inventory | `scripts/agentic/surface-navigator-inventory-audit.ts` | Finds screenshotable main-window surfaces missing from matrices or stale entries. |
| Generated matrix | `bun scripts/generate-surface-contracts.ts --check` | Confirms JSON artifact matches Rust registry. |
| Source audits | `tests/source_audits/mini_main_window.rs`, triggerBuiltin and surface tests | Fastest proof for resize/identity ownership. |

## Data, Storage, And Privacy Boundaries

- The surface contract matrix stores behavior metadata, not user data.
- Runtime receipts expose active surface identity, footer identity, visible rows, counts, and policy fields; they should not expose private content beyond what the active UI already renders.
- Screenshots can capture user-visible content and should be used only when visual rendering is the acceptance criterion.
- Secondary-window diagnostics are separate and intentionally do not reuse main-window `surfaceContract`.
- Generated contract artifacts are source-derived; they are not a parallel hand-authored source of truth.

## Error, Empty, Loading, And Disabled States

- Empty filterable surfaces still preserve their Mini/Full classification.
- Loading or delayed provider states must not trigger follow-up Full resize on Mini surfaces.
- Unknown or failed triggerBuiltin routes should fail closed or leave the prior route intact.
- A stale `activeFooter` or stacked native/GPUI footer is a state mismatch, not a screenshot-only problem.
- Missing `SurfaceKind` or wildcard surface mappings are contract failures.
- Missing generated matrix entries are agent-facing contract failures.
- Ambiguous or stale automation semantic surface after a route change indicates re-key ordering drift.

## Code Ownership

| Area | Primary files | Notes |
|---|---|---|
| Resize primitives | `src/window_resize/mod.rs` | `ViewType`, width/height mapping, Mini sizing receipts, sync/deferred resize helpers. |
| App-level sizing | `src/app_impl/ui_window.rs` | `calculate_window_size_params`, mode changes, deferred update, compact AI sizing. |
| Built-in open helpers | `src/app_execute/builtin_execution.rs` | Mini/Full built-in entry and Mini main window helper. |
| Surface contracts | `src/main_sections/app_view_state.rs` | `AppView`, `SurfaceKind`, `LauncherSurfaceContract`, dismiss policy, footer ids. |
| Trigger dispatch | `src/app_impl/trigger_builtin_dispatch.rs` | Route mutation, filterable state machine, deferred resize, post-dispatch re-key. |
| Automation re-key | `src/app_impl/automation_surface.rs` | Main automation semantic surface owner. |
| Reset/go-back | `src/app_impl/lifecycle_reset.rs` | Hide/reset ordering, popup cleanup, mode restoration. |
| Generated matrix | `scripts/generate-surface-contracts.ts`, `docs/ai/contracts/surface-contracts.json` | Agent-readable checked surface contract artifact. |
| Runtime matrices | `scripts/agentic/filterable-surface-matrix.ts`, `scripts/agentic/surface-navigator-inventory-audit.ts` | State-first surface proof and inventory audit. |
| Tests | `tests/source_audits/mini_main_window.rs`, `tests/trigger_builtin_current_app_commands_contract.rs`, `tests/surface_contract_matrix_artifact_contract.rs`, `tests/state_result_surface_contract_snapshot.rs` | Core regression gates. |

## Invariants And Regression Risks

- Mini/Full classification is layout-driven.
- Single-column filterable built-ins stay Mini.
- Split preview/detail surfaces stay Full.
- Row count, command importance, tray entry, shortcut entry, or trigger source never justify Full width.
- Deferred resize must preserve the class chosen by the open helper.
- Async resize tasks must re-read `current_view` before raw resizing.
- `MiniPrompt` must use `ViewType::MiniPrompt`.
- ChatPrompt and ACP must branch on `main_window_mode`.
- `width_for_view` must clamp Mini view types to Mini width and `ScriptList` to Full width.
- Mini main window fixed height prevents visible resize churn; sizing receipts still matter.
- `AppView::surface_kind()` and `SurfaceKind::surface_contract()` must be exhaustive and wildcard-free.
- `semantic_surface_for_main_view()` must delegate to the surface contract registry.
- Runtime `getState.surfaceContract` must match generated matrix semantics.
- `activePopupContract` must describe attached popup behavior without replacing host surface identity.
- Native footer identity remains AppView-specific.
- TriggerBuiltin re-keying happens after route mutation.
- Hide/reset paths must reset to ScriptList before re-keying to `scriptList`.
- Screenshots do not prove semantic surface identity, dismiss policy, footer ownership, or generated matrix correctness.

## Verification Recipes

Docs and atlas update:

```bash
lat check
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md
```

Surface contract registry:

```bash
bun scripts/generate-surface-contracts.ts --check
cargo test --test surface_contract_matrix_artifact_contract
```

Mini main window source audit:

```bash
cargo test --test source_audits mini_main_window -- --nocapture
cargo test --test window_resize_logic
```

Current-app commands Mini deferred resize:

```bash
cargo test --test trigger_builtin_current_app_commands_contract -- --nocapture
```

Runtime surface contract snapshots:

```bash
cargo test --test state_result_surface_contract_snapshot
```

TriggerBuiltin semantic re-keying:

```bash
cargo test --test trigger_builtin_post_match_surface_rekey_contract
cargo test --test main_automation_surface_rekey_owner_contract
```

Filterable surface matrix:

```bash
bun scripts/agentic/filterable-surface-matrix.ts --list
```

Surface inventory audit:

```bash
bun scripts/agentic/surface-navigator-inventory-audit.ts --json
```

Use runtime screenshots only after state receipts when the question is visual layout, clipping, crop bounds, hover shielding, or image-library coverage.

## Agent Notes

- Do not infer behavior from render filenames; use `SurfaceKind`, generated contracts, and runtime receipts.
- Do not infer Full width from lots of rows, command importance, tray entry, shortcut entry, triggerBuiltin entry, or historical screenshots.
- To debug wrong width, find the initial open helper, then find follow-up `update_window_size_deferred` calls and inspect `calculate_window_size_params()` for the active `AppView`.
- To debug wrong automation surface, compare `current_view`, `getState.surfaceContract.automationSemanticSurface`, `listAutomationWindows.semanticSurface`, and the re-key helper ordering.
- To debug footer issues, inspect `AppView::native_footer_surface()`, `activeFooter`, native host state, GPUI fallback state, and `NATIVE_MAIN_WINDOW_FOOTER_HEIGHT`.
- This belongs to `window-resizing` when the work touches `MainWindowMode`, `ViewType`, Mini/Full classification, resize helpers, or bounds receipts.
- This belongs to `launcher-surface-contracts` when the work touches `AppView`, `SurfaceKind`, policy vocabulary, generated matrices, native footer ids, dismiss policy, or automation semantic surface.
- Screenshots are only needed when rendered appearance is the acceptance criterion; state receipts and source audits should prove identity and sizing policy first.

## Related Features

- 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment.
- 005 Built-in Filterable Surfaces.
- 013 ScriptList Special Entry Triggers.
- 014 Quick Terminal PTY / TermPrompt / Warm Pool / Apply-back.
- 016-024 Prompt runtime and prompt-specific surfaces.
- 030 ACP Chat SDK APIs.
- 031 Legacy `chat()` Prompt.
- 035 Settings, Theme, Config, and Preferences.
- 037 Storybook, Design Explorer, and Visual Verification.

## Open Questions And Gaps

- Broad render files were not included in the focused Oracle bundle, so exact visual implementation details for some surfaces need full-source confirmation before product-facing claims.
- Special-character handoff details were in scope but not fully represented in the bundle; route-specific sizing should be audited in the owning trigger chapters and source files.
- File Search has a typed Mini/Full surface split, but the included sizing evidence maps `FileSearchView` through `ViewType::ScriptList`; Mini route details should be checked in the File Search route/render owners.
- Oracle flagged a possible mismatch where Design Gallery is documented as Mini by sizing guidance but its surface contract vocabulary may read as split-preview; audit before relying on either as absolute.
- Oracle flagged a possible mismatch where Theme Chooser is Full by sizing guidance but its surface vocabulary may read as compact/no-persistent-preview; audit before changing the contract.
- Secondary-window sizing remains out of scope for this chapter and needs platform/windowing-specific treatment.

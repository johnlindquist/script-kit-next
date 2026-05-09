# Agent Understanding And Regression Prevention Plan

This page tracks the staged migration toward clearer code intent, explicit surface contracts, and state-first regression proof.

The goal is not a broad rewrite. Each slice should make the next agent less likely to guess, less likely to edit the wrong owner, and more likely to verify the behavior that can regress.

## Principles

These principles define how the migration should change code, tests, docs, and commit history.

- Prefer descriptive names that state behavior: `build_current_app_commands_state_receipt` is better than `state_for_current_app`.
- Prefer one behavior owner over repeated matches across render, state, elements, keyboard, and automation paths.
- Prefer compile-time exhaustiveness for new `AppView` behavior instead of fallback arms that silently treat new surfaces as `ScriptList`.
- Prefer state-first agentic verification through [[verification]] and [[automation]] before screenshots.
- Keep every step reviewable as a small commit with `Why`, `Contract`, and `Verification` in the commit body.

## Work Slices

These slices are ordered to reduce confusion first, then reduce behavioral drift in code.

| ID | Status | Goal | Primary Areas | Commit Intent | Verification |
| --- | --- | --- | --- | --- | --- |
| AURP-01 | verified | Add this durable tracker and link it from the lattice. | `lat.md/` | `docs(architecture): track agent-understanding regression plan` | `lat check` |
| AURP-02 | verified | Name the live startup path and quarantine stale startup fragments. | [[architecture]], `src/app_impl/startup.rs`, `src/app_impl/startup_new_*.rs` | `docs(architecture): name live launcher startup path` | `cargo test --test launcher_startup_entrypoint_contract`, `lat check` |
| AURP-03 | verified | Define the initial surface contract vocabulary. | [[surfaces]], `src/main_sections/app_view_state.rs` | `refactor(surfaces): introduce explicit launcher surface contract names` | `cargo test --test launcher_surface_contract_vocabulary`, `cargo test --test app_view_policy_contract`, `cargo check`, `lat check` |
| AURP-04 | verified | Start a contract registry for `AppView` behavior. | `src/main_sections/app_view_state.rs`, `src/main_sections/render_impl.rs` | `refactor(surfaces): require AppView behavior declarations` | `cargo test --test app_view_policy_contract --test automation_semantic_surface_rekey_contract`, `cargo check`, `lat check` |
| AURP-05 | verified | Share visible-row/filter helpers for one high-risk surface family. | `src/prompt_handler/mod.rs`, `src/app_layout/collect_elements.rs`, `src/render_builtins/` | `refactor(automation): share current app command visible rows` | focused Rust tests, state-first agentic `getState`/`getElements`, `lat check` |
| AURP-06 | verified | Add an agentic verification matrix for migrated surfaces. | `tests/`, `scripts/agentic/`, [[tests]] | `test(agentic): add state-first proof matrix for filterable surfaces` | matrix run against real entry paths, `lat check` |
| AURP-07 | verified | Group broad `ScriptListApp` state into named domain structs. | `src/main_sections/app_state.rs`, [[architecture]] | `refactor(app-state): group render diagnostics state` | `cargo test --test app_state_domain_structs_contract`, `cargo check`, `lat check` |
| AURP-08 | verified | Move global key handling toward named intent handlers. | `src/app_impl/startup.rs`, [[surfaces]] | `refactor(keyboard): name main-window global key intent` | `cargo test --test main_window_global_key_intent_contract`, focused Cmd+Enter routing tests, `cargo check`, `lat check` |
| AURP-09 | verified | Group main-menu fallback state into a named domain owner. | `src/main_sections/app_state.rs`, `src/app_impl/filter_input_updates.rs`, stdin simulate-key dispatchers | `refactor(app-state): group fallback launcher state` | `cargo test --test main_menu_fallback_state_domain_contract`, `cargo check`, `lat check` |
| AURP-10 | verified | Continue extracting main-window key handling into named intent handlers. | `src/app_impl/startup.rs`, [[surfaces]] | `refactor(keyboard): expand main-window key intent routing` | `cargo test --test main_window_actions_key_intent_contract`, `cargo test actions_button_visibility_tests::tests::test_startup_cmd_k_uses_shared_dispatcher_after_popup_router`, runtime GPUI Cmd+K proof, `cargo check`, `lat check` |
| AURP-11 | verified | Expand the filterable surface proof matrix beyond Current App Commands. | `scripts/agentic/filterable-surface-matrix.ts`, [[automation]], [[tests]] | `test(agentic): expand filterable surface matrix` | `cargo test --test filterable_surface_agentic_matrix_contract`, matrix run against real entry paths, `lat check` |
| AURP-12 | verified | Group main-menu search and grouped-result caches into a named owner. | `src/main_sections/app_state.rs`, `src/app_impl/filtering_cache.rs`, render/state paths | `refactor(app-state): group main-menu result caches` | `cargo test --test main_menu_result_cache_domain_contract`, focused app-state and Tab AI source tests, `cargo check`, `lat check` |
| AURP-13 | verified | Add behavior-named accessors for main-menu result caches. | `src/main_sections/app_state.rs`, `src/app_impl/filtering_cache.rs`, navigation/preflight/Tab AI readers | `refactor(app-state): name main-menu cache access paths` | `cargo test --test main_menu_result_cache_domain_contract`, focused Tab AI routing test, `cargo check`, `lat check` |
| AURP-14 | verified | Name the triggerBuiltin semantic-surface dispatcher contract. | `src/main_entry/*`, `src/app_impl/trigger_builtin_dispatch.rs`, [[automation]], [[surfaces]] | `refactor(automation): name trigger-builtin surface rekey contract` | triggerBuiltin dispatcher source contract tests, `cargo check`, `lat check` |
| AURP-15 | verified | Extract stdin simulateKey actions-toggle intent checks. | `src/main_entry/runtime_stdin_match_simulate_key.rs`, `src/main_entry/app_run_setup.rs`, [[automation]] | `refactor(keyboard): name simulate-key actions toggle intent` | source contract tests, `cargo check`, `lat check` |
| AURP-16 | verified | Expand the filterable surface matrix to stable sibling subviews. | `scripts/agentic/filterable-surface-matrix.ts`, [[automation]], [[protocol]] | `test(agentic): add more filterable subview receipts` | matrix source contract, matrix list receipt, stable case runtime receipts, `lat check` |
| AURP-17 | verified | Name selected-result resolution paths so `selected_index` mapping has one intent owner. | `src/main_sections/app_state.rs`, `src/app_impl/filtering_cache.rs`, `src/main_window_preflight/build.rs`, `src/app_impl/tab_ai_mode/mod.rs`, [[architecture]] | `refactor(app-state): name selected result resolution paths` | `cargo test --test main_menu_result_cache_domain_contract`, `cargo check`, `lat check` |
| AURP-18 | verified | Normalize filterable subview row projection behind named helpers. | `src/render_builtins/`, `src/prompt_handler/mod.rs`, `src/app_layout/collect_elements.rs`, [[automation]] | `refactor(automation): name filterable subview row projection` | surface-specific source contracts, matrix receipts, `lat check` |
| AURP-19 | planned | Stabilize browser/window-dependent entries before adding them to the agentic matrix. | `scripts/agentic/`, browser/window built-ins, [[automation]], [[protocol]] | `test(agentic): stabilize environment-aware surface receipts` | targeted setup receipts, opt-in matrix cases, `lat check` |
| AURP-20 | planned | Name per-view `simulateKey` intent dispatch beyond the generic actions-toggle fallback. | `src/main_entry/runtime_stdin_match_simulate_key.rs`, `src/main_entry/app_run_setup.rs`, [[surfaces]] | `refactor(keyboard): name per-view simulate-key intents` | source contract tests, representative stdin receipts, `cargo check`, `lat check` |
| AURP-21 | planned | Add an agent map for the highest-risk behavior flows. | `lat.md/`, [[architecture]], [[verification]] | `docs(architecture): map high-risk agent behavior flows` | `lat check` |
| AURP-22 | verified | Promote MCP-backed catalog rows to explicit projection owners. | `src/mcp_resources/mod.rs`, `src/prompt_handler/mod.rs`, `src/app_execute/builtin_execution.rs`, `src/app_impl/tab_ai_mode/mod.rs`, [[automation]] | `refactor(mcp): name catalog row projection` | MCP catalog source contracts, optional matrix receipts, `cargo check`, `lat check` |
| AURP-23 | verified | Split ACP automation state receipts into named snapshot builders. | `src/ai/acp/view.rs`, [[acp-chat]], [[automation]] | `refactor(acp): name automation state snapshot builders` | ACP state/probe tests, `cargo check`, `lat check` |
| AURP-24 | verified | Name automation batch target capabilities before extracting runners. | `src/prompt_handler/mod.rs`, [[automation]], [[protocol]] | `refactor(automation): name batch target capabilities` | batch capability source contracts, representative stdin receipts, `cargo check`, `lat check` |
| AURP-25 | verified | Split payload-bearing routes from stable surface contract identity. | [[surfaces]], `src/main_sections/app_view_state.rs` | `refactor(surfaces): introduce stable SurfaceKind contracts` | `cargo test --test app_view_policy_contract`, `cargo test --test dispatcher_semantic_surface_symmetry_contract`, `cargo test --test launcher_surface_contract_vocabulary`, `cargo check`, `lat check` |
| AURP-26 | verified | Generate an agent-readable matrix from the typed surface registry. | [[surfaces]], `scripts/generate-surface-contracts.ts`, `docs/ai/contracts/surface-contracts.json` | `docs(surfaces): generate surface contract matrix` | `bun scripts/generate-surface-contracts.ts --check`, `cargo test --test surface_contract_matrix_artifact_contract`, `cargo test --test launcher_surface_contract_vocabulary`, `lat check` |
| AURP-27 | verified | Name shared actions popup open/close state mutators. | [[surfaces]], `src/app_impl/actions_dialog.rs`, `src/app_impl/actions_toggle.rs`, `src/render_builtins/actions.rs` | `refactor(actions): name popup state mutators` | `cargo test --test actions_popup_state_mutator_contract`, actions debounce/source contract tests, `cargo check`, `lat check` |
| AURP-28 | verified | Name current-view main automation surface re-keying. | [[automation]], [[surfaces]], `src/app_impl/automation_surface.rs`, ACP/About routes | `refactor(automation): name current-view surface rekey owner` | `cargo test --test main_automation_surface_rekey_owner_contract --test trigger_builtin_post_match_surface_rekey_contract --test embedded_ai_window_tab_ai_mode_sites_contract --test automation_semantic_surface_rekey_contract`, `cargo check`, `lat check` |
| AURP-29 | verified | Generate an inventory of direct `current_view` transition sites. | [[surfaces]], `scripts/generate-current-view-transitions.ts`, `docs/ai/contracts/current-view-transitions.json` | `docs(surfaces): generate current-view transition inventory` | `bun scripts/generate-current-view-transitions.ts --check`, `cargo test --test current_view_transition_inventory_contract`, `lat check` |
| AURP-30 | verified | Start a named current-view transition owner for re-keyed routes. | [[surfaces]], `src/app_impl/automation_surface.rs`, `src/app_impl/about_route.rs` | `refactor(surfaces): name rekeyed current-view transitions` | `cargo test --test main_automation_surface_rekey_owner_contract --test current_view_transition_inventory_contract`, `cargo check`, `lat check` |
| AURP-31 | verified | Expose the active surface contract in `getState` receipts. | [[protocol]], [[surfaces]], `src/protocol/`, `src/prompt_handler/mod.rs`, `scripts/kit-sdk.ts` | `feat(protocol): include surface contract in state receipts` | `cargo test --test state_result_surface_contract_snapshot --test sdk_automation_runtime`, `cargo check`, `lat check` |
| AURP-32 | verified | Make the filterable matrix assert live surface contracts. | [[automation]], [[protocol]], `scripts/agentic/filterable-surface-matrix.ts` | `test(agentic): assert surface contracts in filterable receipts` | `cargo test --test filterable_surface_agentic_matrix_contract`, matrix runtime receipts, `lat check` |
| AURP-33 | verified | Name the embedded ACP entry transition owner. | [[acp-chat]], [[surfaces]], `src/app_impl/acp_surface_transitions.rs`, `src/app_impl/tab_ai_mode/` | `refactor(acp): name embedded acp entry transition` | `cargo test --test embedded_ai_window_tab_ai_mode_sites_contract --test main_automation_surface_rekey_owner_contract --test acp_reattach_identity_contract`, `cargo check`, `lat check` |
| AURP-34 | verified | Move native footer surface identity onto `AppView`. | [[design]], [[surfaces]], `src/main_sections/app_view_state.rs`, `src/app_impl/ui_window.rs` | `refactor(footer): name app view footer surface owner` | `cargo test --test main_window_footer_surface_owner_contract --test prompt_chrome_builtin_source_audit --test minimal_chrome_audit --test quick_terminal_contracts`, `cargo check`, `lat check` |
| AURP-35 | verified | Move remaining shared actions popup state writes behind named mutators. | [[surfaces]], `src/app_impl/actions_dialog.rs`, shared actions callers | `refactor(actions): centralize popup state cleanup` | `cargo test --test actions_popup_state_mutator_contract`, `cargo check`, `lat check` |
| AURP-36 | verified | Name return-view restoration with focus translation. | [[surfaces]], `src/app_impl/automation_surface.rs`, attachment portal and Tab AI close routes | `refactor(surfaces): name return-view focus restore` | `cargo test --test main_automation_surface_rekey_owner_contract`, `cargo test --test acp_dictation_keyboard_contract embedded_acp_close_helper_tears_down_surface_and_registry`, `cargo test --test tab_ai_routing close_harness_terminal_restores_return_view`, `cargo check`, `lat check` |
| AURP-37 | verified | Name ScriptList main-filter entry restoration. | [[surfaces]], `src/app_impl/automation_surface.rs`, ScriptList entry callers | `refactor(surfaces): name script-list focus restore` | `cargo test --test main_automation_surface_rekey_owner_contract`, `cargo test source_audits::mini_main_window::open_mini_main_window_sets_mini_mode_contract`, `cargo check`, `lat check` |
| AURP-38 | verified | Widen the generated current-view inventory to every app-owned transition area. | [[surfaces]], `scripts/generate-current-view-transitions.ts`, `docs/ai/contracts/current-view-transitions.json` | `docs(surfaces): inventory all app current-view transitions` | `bun scripts/generate-current-view-transitions.ts --check`, `cargo test --test current_view_transition_inventory_contract`, `cargo check`, `lat check` |
| AURP-39 | verified | Include named transition-helper call sites in the current-view inventory. | [[surfaces]], `scripts/generate-current-view-transitions.ts`, `docs/ai/contracts/current-view-transitions.json` | `docs(surfaces): inventory named transition helpers` | `bun scripts/generate-current-view-transitions.ts --check`, `cargo test --test current_view_transition_inventory_contract`, `cargo check`, `lat check` |
| AURP-40 | verified | Re-key ScriptList main-filter restoration and classify real actions-popup closes. | [[surfaces]], `src/app_impl/automation_surface.rs`, shared actions close paths | `refactor(surfaces): tighten script-list and popup close contracts` | `cargo test --test main_automation_surface_rekey_owner_contract --test actions_popup_state_mutator_contract`, `cargo check`, `lat check` |
| AURP-41 | verified | Expose attached actions-popup contracts in `getState` receipts. | [[protocol]], [[surfaces]], `src/protocol/`, `src/prompt_handler/mod.rs`, `scripts/kit-sdk.ts` | `feat(protocol): expose active popup surface contract` | `cargo test --test state_result_surface_contract_snapshot --test sdk_automation_runtime`, `cargo check`, `lat check` |
| AURP-42 | verified | Make native footer identity machine-readable. | [[design]], [[surfaces]], `scripts/generate-surface-contracts.ts`, `src/protocol/types/automation_surface.rs` | `docs(footer): expose native footer surface contracts` | `bun scripts/generate-surface-contracts.ts --check`, `cargo test --test surface_contract_matrix_artifact_contract --test main_window_footer_surface_owner_contract`, `cargo check`, `lat check` |
| AURP-43 | verified | Harden exact AppView to SurfaceKind semantic-surface checks. | [[automation]], [[surfaces]], surface source audits | `test(automation): assert exact semantic surface mappings` | `cargo test --test dispatcher_semantic_surface_symmetry_contract --test automation_semantic_surface_rekey_contract`, `cargo check`, `lat check` |
| AURP-44 | verified | Remove local absolute paths from surface and automation docs. | [[surfaces]], [[automation]], `lat.md/` | `docs(lat): use repo-relative contract paths` | `lat check` |
| AURP-45 | verified | Harden named current-view transition contracts. | [[surfaces]], current-view transition inventory | `test(surfaces): pin current-view transition helper contracts` | `bun scripts/generate-current-view-transitions.ts --check`, `cargo test --test current_view_transition_inventory_contract`, `cargo check`, `lat check` |

## Current Slice

AURP-18 is verified: App Launcher, Process Manager, and MCP-backed catalog surfaces now have named visible-row projection owners shared by render, state, elements, sizing, and Tab AI target capture.

AURP-24 is verified: batch target capabilities have a named owner before any future extraction of target-specific batch runners.

AURP-25 is verified: `AppView::surface_kind()` now maps routed payload state to stable `SurfaceKind` identities before behavior contracts are resolved.

AURP-26 is verified: agents can read `docs/ai/contracts/surface-contracts.json`, which is generated from the typed surface registry and checked for drift.

The surface registry now declares explicit focus, keyboard, actions, proof, and visual policy dimensions beside vocabulary, dismissal, and automation tags.

AURP-27 is verified: shared actions popup open/close paths use named mutators for the popup flag and recent-close debounce timestamp.

AURP-28 is verified: routed current-view semantic surface changes use `rekey_main_automation_surface_from_current_view()` instead of duplicating raw registry writes and surface lookups.

AURP-29 is verified: agents can inspect `docs/ai/contracts/current-view-transitions.json` to see remaining direct `current_view` mutation sites before converting them behind named transition APIs.

AURP-30 is verified: About and parent-confirm routes use `transition_current_view_and_rekey_main_automation_surface()` so the AppView mutation and semantic-surface re-key cannot be split apart.

AURP-31 is verified: main-window `stateResult` receipts now include `surfaceContract`, a snapshot derived from `AppView::surface_contract()` and typed in the Kit SDK.

AURP-32 is verified: the filterable surface matrix declares expected `SurfaceKind` values and asserts live `stateResult.surfaceContract` identity before trusting count receipts.

AURP-33 is verified: setup, reuse, and full-launch ACP entry paths delegate to `enter_embedded_acp_chat_surface()`, which owns the view flip, embedded AI upsert, main re-key, ACP surface transition, actions cleanup, and chat focus target.

AURP-34 is verified: `AppView::native_footer_surface()` owns native footer surface identity, while `ui_window.rs` only builds button configs and delegates surface id lookup to the view contract owner.

AURP-35 is verified: production shared actions paths use `mark_actions_popup_opening()`, `mark_actions_popup_closed()`, or `clear_actions_popup_state()` instead of repeating raw popup flag, dialog, and debounce writes.

AURP-36 is verified: attachment portal exits and Tab AI close restore captured routes through `restore_current_view_with_focus()`, while keeping re-keying, ACP teardown, sizing, and notifications visible in their local route owners.

AURP-37 is verified: ScriptList entry paths that target the main filter use `show_script_list_with_main_filter_focus()` instead of repeating raw `AppView::ScriptList` plus focus field writes.

AURP-38 is verified: the current-view transition inventory now scans `app_actions`, `app_execute`, `app_impl`, `main_entry`, `main_sections`, and `prompt_handler`, so agent-readable contracts cover prompt and execution transition sites too.

AURP-39 is verified: named transition helpers such as embedded ACP entry, return-view restore, and ScriptList main-filter focus restore now appear as call sites in `docs/ai/contracts/current-view-transitions.json`.

AURP-40 is verified: `show_script_list_with_main_filter_focus()` re-keys the main automation surface after restoring ScriptList, and real actions-popup close gestures use `mark_actions_popup_closed()` while cleanup paths keep `clear_actions_popup_state()`.

AURP-41 is verified: main-window `stateResult` receipts now include `activePopupContract` for attached Actions Dialog overlays without changing the host `surfaceContract`.

AURP-42 is verified: generated surface contracts and live `surfaceContract` snapshots expose `nativeFooterSurface`, keeping AppView-specific footer identity machine-readable.

AURP-43 is verified: semantic-surface audits assert exact `AppView -> SurfaceKind -> automationSemanticSurface` relationships instead of accepting substring presence.

AURP-44 is verified: surface and automation lat pages use repo-relative contract paths so agents do not learn a local checkout path as part of the architecture.

AURP-45 is verified: the current-view transition inventory now carries checked transition-contract metadata for named helpers. This deliberately avoids a public `ViewTransitionReceipt` while existing `getState.surfaceContract` and `activePopupContract` remain the runtime state snapshot proof.

Oracle session `massive-files-next-slices` ranked the next queue as App Launcher Tab AI projection, Process Manager row projection, MCP catalog row projection, ACP automation snapshot builders, and batch target capabilities.

The verified AURP-16 matrix remains the state-first regression baseline before changing more row-projection paths.

## Verification Ladder

The ladder keeps each slice honest without defaulting to slow or unfocused test runs.

1. Docs-only changes: run `lat check`.
2. Pure Rust refactors: run `cargo check`, the narrowest affected Rust tests, and `lat check`.
3. Surface receipt changes: run narrow Rust/source contract tests, a state-first agentic proof using `getState` and `getElements`, and `lat check`.
4. Keyboard or routing changes: run source contract tests, a real-entry-path agentic proof, and `lat check`.
5. Visual or layout changes: run the relevant state-first proof first, then capture screenshots only for visual acceptance criteria.

## Commit Contract

Every migration commit should explain the reason, the protected behavior, and the proof.

Use this body shape so future agents can understand the decision without reconstructing the whole context:

```text
Why:
<What confusion or regression risk this reduces.>

Contract:
<The behavior or ownership rule that must remain true.>

Verification:
<Exact commands and, when applicable, the agentic receipt path.>
```

## Decision Log

This log records durable choices made while reducing regression risk.

- Use `lat.md/` as the tracking source so [[verification]], [[automation]], [[surfaces]], and [[architecture]] stay connected.
- Keep the first implementation slice small: document and link the plan before touching production code.
- Prefer state-first agentic receipts for behavior proof; screenshots are reserved for visual claims.
- Treat `src/app_impl/startup.rs` as the live launcher startup owner; treat `src/app_impl/startup_new_*.rs` as legacy source-audit parity fragments unless a future migration wires or removes them explicitly.
- For AURP-05, migrate one concrete surface first instead of abstracting every filterable subview at once; `CurrentAppCommandsView` had the clearest drift risk because menu-command filters search name, description, and keywords.
- For AURP-06, keep the matrix data-first so adding a migrated surface requires declaring entry command, prompt type, list semantic id, and filter text before runtime proof.
- For AURP-07, start app-state grouping with diagnostics rather than a behavior-heavy surface state machine so the first commit proves the domain-struct pattern without mixing in user-facing behavior changes.
- For AURP-08, start with the global Cmd+Enter ACP path because it already has broad test coverage and a clear behavior name; defer larger Tab and arrow-key extractions until their local ownership rules can be sliced independently.
- For AURP-09 through AURP-12, keep extending the tracker before implementation so follow-on agents can see what remains after the first eight verified slices.
- For AURP-09, keep fallback navigation behavior on tiny owner methods rather than moving fallback execution itself; the owner should clarify state ownership without hiding command execution policy.
- For AURP-10, keep shared actions-dialog routing before local key intent dispatch so popup-owned close, enter, and text routes cannot be bypassed by the named Cmd+K/Cmd+W handlers.
- For AURP-11, expand the matrix only with surfaces that already have aligned state and element collectors, and force an empty-filter reset per case so added rows do not depend on runner order.
- For AURP-12, preserve the existing cache field names inside the owner while moving every production read/write through `main_menu_result_caches`; this keeps source-audit continuity without leaving loose state on `ScriptListApp`.
- For AURP-13 through AURP-16, prioritize naming remaining behavior boundaries over broad rewrites: accessors before deeper cache extraction, dispatcher contracts before deduplicating stdin files, and matrix additions only for surfaces with stable receipts.
- For AURP-13, keep cache storage names private to [[src/main_sections/app_state.rs#MainMenuResultCacheState]] and force consumers through behavior-named methods pinned by [[tests/main_menu_result_cache_domain_contract.rs#result_cache_owner_names_behavior_accessors]] and [[tests/main_menu_result_cache_domain_contract.rs#production_cache_consumers_do_not_read_storage_fields_directly]].
- For AURP-14, keep `triggerBuiltin` re-keying as an explicit post-dispatch step through [[src/app_impl/trigger_builtin_dispatch.rs#ScriptListApp#rekey_main_automation_surface_after_trigger_builtin_dispatch]] so stdin arms reveal dispatch-then-rekey ordering without duplicating raw registry calls.
- For AURP-15, extract only the generic stdin Cmd+K fallback predicate first; per-view Cmd+K arms remain local because they carry host-specific behavior.
- For AURP-16, add matrix cases only when the entry path and list/state receipts are stable in ordinary dev sessions; defer browser/window-dependent surfaces until their setup is less environment-sensitive.
- For AURP-17 through AURP-21, keep the work in reviewable behavior slices: selection resolution first, row projection second, environment-sensitive matrix cases third, keyboard dispatch fourth, and an agent map once the named owners are clearer.
- For AURP-17, keep exact grouped-row lookup, coerced-selection lookup, forward preflight lookup, and visible-result iteration as separate helper names on [[src/main_sections/app_state.rs#MainMenuResultCacheState]] so callers reveal whether they are reading the focused row, the coerced executable row, or all visible results.
- For AURP-18, migrate one filterable surface family at a time. App Launcher is first because its raw `app.name` filter was duplicated across render, keyboard, `getState`, `getElements`, and wheel reanchor paths.
- For Oracle session `massive-files-next-slices`, keep finishing AURP-18 before AURP-19: browser/window-dependent matrix work, whole-`prompt_handler` batch rewrites, dictation/model-download behavior, and broad ActionsDialog consolidation are deferred until smaller owners are pinned.
- For Oracle session `ux-contract-regression-plan`, start with a low-risk identity layer: `AppView` keeps payload state, `SurfaceKind` owns stable contract identity, and `SurfaceKind::surface_contract()` remains the single registry for behavior vocabulary, dismiss policy, and automation surface tags.
- For AURP-26, keep the agent-readable artifact generated from source rather than hand-maintained. The JSON exists for agents to inspect quickly; `src/main_sections/app_view_state.rs` remains the contract owner.
- Start contract expansion with focus, keyboard, actions, proof, and visual ownership because they are low-risk, observable by agents, and directly connected to keyboard/focus/actions regressions without forcing a universal dispatcher.
- For AURP-27, start risky-state encapsulation with the shared actions popup debounce pair. Reads remain broad for keyboard routing, but open/close timestamp writes now have named mutators that preserve the 300ms recent-close contract.
- For AURP-28, keep the raw `update_automation_semantic_surface` registry primitive available, but route current-view-derived main-window updates through `ScriptListApp::rekey_main_automation_surface_from_current_view` so About, Confirm, ACP, and triggerBuiltin paths cannot fork the `AppView` surface lookup.
- For AURP-29, inventory direct `current_view` writes before wrapping them broadly. The JSON is a migration map and drift detector, not a parallel authority; `current_view` behavior still belongs in source-owned route helpers and the surface registry.
- For AURP-30, start transition ownership only where the existing behavior already couples a route mutation with current-view semantic re-keying. Do not force ACP entry paths through the same helper until their embedded-AI upsert ordering is reviewed separately.
- For AURP-31, expose surface contracts on main `getState` receipts as a runtime mirror of the generated matrix. Keep secondary target diagnostics without `surfaceContract` until those windows have their own typed contract registry.
- For AURP-32, treat matrix `promptType` as a compatibility check and `surfaceContract.surfaceKind` as the behavior contract check. Runtime proof should fail when either legacy prompt identity or typed surface identity drifts.
- For AURP-33, give embedded ACP entry a dedicated owner instead of reusing the generic current-view transition owner; its contract also includes the child automation-window upsert, ACP placement machine, actions-popup cleanup, and focus handoff.
- For AURP-34, keep footer surface identity AppView-specific rather than SurfaceKind-specific because grouped surface kinds such as prompt entities and generic filterable lists still need distinct native footer ids.
- For AURP-35, distinguish real popup close gestures from stale-overlay cleanup: only `mark_actions_popup_closed()` records the 300ms debounce, while route/reset cleanup goes through `clear_actions_popup_state()`.
- For AURP-36, keep return-view focus restoration separate from re-keyed route transitions. Restoring a captured route is common, but ACP close and attachment portals still own their additional side effects locally.
- For AURP-37, keep ScriptList entry cache/sizing work at the caller. The shared helper only owns the AppView + main-filter focus pair.
- For AURP-38, keep the generated transition inventory comprehensive across app-owned transition modules. Exclude source-audit fixture strings, but include prompt and execution owners so agents do not infer contracts from a partial map.
- For AURP-39, inventory named transition helpers as call sites, not only helper definitions. Agents need to see where helper-owned side effects are invoked.
- For AURP-40, treat ScriptList main-filter focus as an observable route restoration and re-key main automation there. Distinguish user/automation popup closes from route cleanup so the 300ms debounce remains meaningful.
- For AURP-41, expose overlay contracts as optional state receipt fields instead of mutating `current_view` to an overlay route. The host route remains the main surface contract.
- For AURP-42, keep footer identity AppView-specific but project it into both generated and live contract surfaces.
- For AURP-43, exact relationship checks are required for semantic-surface contracts; a variant and semantic string existing somewhere in the same file is not enough.
- For AURP-44, keep lat links repo-relative so generated or copied docs remain useful outside this checkout.

## Related Pages

These pages define the architectural and testing contracts this plan relies on.

- [[architecture]]
- [[surfaces]]
- [[verification]]
- [[automation]]
- [[protocol]]
- [[tests]]
- [[acp-chat]]

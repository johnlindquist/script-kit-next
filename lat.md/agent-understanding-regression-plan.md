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

## Current Slice

AURP-18 is verified: App Launcher, Process Manager, and MCP-backed catalog surfaces now have named visible-row projection owners shared by render, state, elements, sizing, and Tab AI target capture.

AURP-24 is verified: batch target capabilities have a named owner before any future extraction of target-specific batch runners.

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

## Related Pages

These pages define the architectural and testing contracts this plan relies on.

- [[architecture]]
- [[surfaces]]
- [[verification]]
- [[automation]]
- [[protocol]]
- [[tests]]
- [[acp-chat]]

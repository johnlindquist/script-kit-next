# Fusion-High Prompt: Main Window Input Benchmark And Lag Fix

Repo: `/Users/johnlindquist/dev/script-kit-gpui`

User request:

> `/goal benchmark the main window input with $t . I've noticed a few places where rapidly deleting text is laggy. I know there's a massive amount of async loading for the unified search, but we need it to be instant. Start with a pass from $fusion high on how to approach this before your first $t iteration. Then apply the suggestions`

Constraints:

- This is Script Kit GPUI. Runtime proof must use the repo DevTools/session primitives, not source-only reasoning.
- Rust commands must use `./scripts/agentic/agent-cargo.sh`, not bare cargo.
- Worktree was clean before the project imp advisory run.
- A project imp routed to `imp-sk-launcher` and started a narrow patch before timing out. Treat the patch as unverified and critique it.
- Do not suggest broad rewrites. We need a first shippable slice that makes rapid deletion feel instant and proves it with the existing benchmark surface.

Relevant files and findings:

- Main input/shared chrome: `src/components/main_view_chrome.rs`, `src/components/text_input.rs`
- Main launcher renderer: `src/render_script_list/mod.rs`
- Filter update path: `src/app_impl/filter_input_change.rs`, `src/app_impl/filter_input_updates.rs`
- Results/cache/path for unified search: `src/app_impl/filtering_cache.rs`, `src/main_sections/app_state.rs`
- Coalescer: `src/filter_coalescer.rs`
- Existing benchmark/proof scripts:
  - `scripts/agentic/root-delete-key-benchmark.ts`
  - `scripts/agentic/root-typing-lag-benchmark.ts`
  - `scripts/devtools/driver.ts`
  - `scripts/devtools/main.ts`

Current unverified imp patch:

- `src/app_impl/filter_input_updates.rs`
  - Renamed the old synchronous `queue_filter_compute` body to private `apply_filter_compute_now`.
  - New `queue_filter_compute`:
    - returns early if `computed_filter_text == value` and resets the coalescer;
    - calls `self.filter_coalescer.queue(value)`;
    - if this is the first pending value, spawns an app task with `timer(Duration::from_millis(0))`;
    - on the next app turn, takes the latest value and calls `apply_filter_compute_now(latest, cx)`.
  - `set_filter_text_immediate` remains synchronous for protocol/programmatic paths.
- `tests/source_audits/root_unified_search_stability_contract.rs`
  - Existing source audit was adjusted to expect coalesced regular typing and to assert the apply helper still installs computed text, starts root async providers, reconciles, rebuilds preflight, then notifies.

Suspected hot path:

- Rapid Backspace appears to synchronously run grouped result computation, provider-start bookkeeping, preflight rebuild, ghost refresh, window sizing checks, and `cx.notify()` per input value.
- The existing `FilterCoalescer` comment/shape suggests this was meant to coalesce, but the current pre-patch path applied synchronously.
- There is already a root delete-key benchmark that measures Backspace cadence/echo and parses `[APPLY_FILTER_DONE]`/`[GROUP_DONE]` logs.

Questions for the panel:

1. Is the imp patch direction correct for making rapid delete instant, or is it likely to introduce stale UI, selection, or protocol-read-after-write regressions?
2. Should regular GPUI input coalesce on a zero-delay next app turn, a small frame delay, or a different mechanism?
3. Which paths must stay synchronous? Evaluate `setFilter`, Enter/selection, menu syntax/spine/trigger picker, prompt-owned input, built-in views, history navigation, and tests.
4. What benchmark thresholds should we enforce for the first slice using `root-delete-key-benchmark.ts` and/or `root-typing-lag-benchmark.ts`?
5. What exact verification sequence should the `$t` implementation use after applying the recommendations?
6. What minimal code/test changes should be made or reverted before committing?

Please return:

- Ordered implementation recommendations for one or two focused `$t` iterations.
- Must-have invariants and exact files/functions to inspect or change.
- Concrete runtime proof commands and pass criteria.
- Risks that should block commit.

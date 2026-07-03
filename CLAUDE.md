For a map of main UI surfaces to code implementation, see [GLOSSARY.md]

# Before Starting Work

- **Route through a project imp first.** Imps are the primary mechanism for agents working with this code: run `bun imps/project-imp --which "<task>"` from `.agents/imps` to find the owner, then delegate to it (see "Project Imps — Primary Mechanism" below). Work directly only for trivial edits, when the imp runtime is unavailable, or when the user explicitly asks you to.
- Inspect the relevant source, tests, and repo-local skills before editing.
- Prefer current code and generated artifacts over stale notes or memory.
- Keep edits narrowly scoped and verify them with the smallest check that can fail for the changed behavior.
- Keep tool-facing root docs in place: `README.md`, `CLAUDE.md`, `AGENTS.md`, and `.impeccable.md`.

## Oracle / Packx Bundle Context

For Oracle review or `oracle-packx` work in this repository, include the repo process context in the bundle or prompt unless the user explicitly excludes it: `AGENTS.md`, the owning `.agents/skills/<skill>/SKILL.md`, and relevant source, tests, generated contracts, and verification notes.

For runtime/UX bugs headed to `oracle-packx-conversation`, run `imp-sk-devtools` first and include its investigation receipts (intake, primitive stack, measurements, classification, likely owner, red/green proof plan) in the bundle — that pairing is a primary workflow in this repo.

If a `packx` preview with include globs unexpectedly matches `0` files in this repository, rebuild the bundle from an explicit path list instead of widening blindly. A reliable workaround is:

```bash
rg --files <scope> | rg '<owners-or-patterns>' > /tmp/script-kit-gpui-packx-files.txt
xargs packx --preview --no-interactive -x "**/CLAUDE.md" < /tmp/script-kit-gpui-packx-files.txt
xargs packx --limit 900k --strip-comments --minify -f markdown --no-interactive --stdout -x "**/CLAUDE.md" < /tmp/script-kit-gpui-packx-files.txt > ~/.oracle/bundles/<slug>.txt
```

Use this when directory/include-glob matching undercounts relevant files; keep `CLAUDE.md` excluded and verify the preview count plus final non-empty bundle before consulting Oracle.

## Project Imps — Primary Mechanism

Project imps are the **primary mechanism for agents working with this code**.
Every substantive task — building, debugging, auditing, fixing, testing,
releasing — starts by routing to the owning imp; direct hand-editing is the
fallback, not the default.

Imps live under `.agents/imps/` and are feature-bound Codex specialists for
this repository. They use `gpt-5.5` with `medium` reasoning and keep local
lesson overlays under `.agents/imps/lessons/local/`. The vendored runtime is
tracked by `.agents/imps/imps.manifest.json`; imp learning flows through the
upstream evolution system (`imp-sk-<name> evolve` to review suggestions), and
lesson files are reviewed overlays folded into imp prompts at launch.

Default workflow for any task:

1. `bun imps/project-imp --which "<task>"` to confirm the owner (or pick from the routing rules below).
2. Delegate the task to that imp: `bun imps/project-imp "<task>"` or `bun imps/imp-sk-<name> --run "<task>"`.
3. Pair the owner with at most one cross-cutting role imp when needed (audit → fix, fix → probe, etc.).
4. The calling agent stays responsible for source inspection, patch review, preserving unrelated dirty work, and final verification — delegating the work does not delegate accountability.

Skip the imp only when the task is trivial (≤ a few mechanical lines), the imp
runtime is unavailable or repeatedly stalls, or the user explicitly directs
otherwise — and say in the final answer which imp was skipped and why.

Use:

```bash
cd .agents/imps
bun imps/project-imp --which "<task prompt>"
bun imps/project-imp "<task prompt>"
bun imps/imp-sk-<name> --run "<task prompt>"   # direct, non-interactive
```

Direct `imp-sk-*` invocations open an interactive Codex TUI by default; agents
must pass `--run` (or `-q`) for the non-interactive streaming path. The
`project-imp` router adds `--run` itself.

Routing rules:

- Use `imp-sk-scout` when ownership is unclear.
- Agent Chat, `@file`, `@context`, attachments, portal, or Pi handoff -> `imp-sk-agent-chat`.
- Day Page, Today, brain, fragments, spine, or Notes parity -> `imp-sk-brain`.
- Clipboard history, sediment, post-copy, copy-to-brain, or no-popup capture -> `imp-sk-clipboard`.
- Shared UI, components, list rows, inputs, prompt shells, chrome, or theme tokens -> `imp-sk-components`.
- Script List, main window, mini/full view, launcher selection -> `imp-sk-launcher`.
- Actions menu, command palette, trigger picker, confirm popup -> `imp-sk-actions`.
- Hotkeys, gestures, tap/hold/double-tap, focus restoration -> `imp-sk-hotkeys`.
- Script prompt renderers and protocol-to-renderer contracts -> `imp-sk-prompts`.
- Built-in utility surfaces -> `imp-sk-builtins`.
- Terminal prompt, PTY, command bar, terminal theme -> `imp-sk-terminal`.
- Script execution, menu cache, metadata, scheduler -> `imp-sk-execution`.
- macOS platform, windows, tray/menu bar, icons, permissions, startup, Pi sidecar -> `imp-sk-platform`.
- MCP server, resources, script tools, schema compatibility -> `imp-sk-mcp`.
- Repo process docs, `.agents/**`, probes, source audits, `dev.sh`, cargo wrappers -> `imp-sk-devex`.
- Local LLM/ghost backend, dictation, whisper, computer use, OCR, camera, AI vault -> `imp-sk-ai-core`.
- Settings, config persistence, onboarding/NUX, kit store, sync, updates, login item, secrets -> `imp-sk-settings`.

Role imps (cross-cutting, any surface):

- Build failures, cargo lock contention, `target-agent` disk budget, clippy/fmt debt, stuck builds -> `imp-sk-build-doctor`.
- DevTools work: runtime proof, app inspection/investigation, driver probes, screenshots, simulateGpuiEvent, red/green receipts -> `imp-sk-devtools` (the imp form of the `script-kit-devtools` skill).
- Read-only audit sweeps, UX inconsistency hunts, hardcoded-token findings (never edits) -> `imp-sk-auditor`.
- Test authorship, enforcement-ladder placement, contract tests, ratchet, flaky tests -> `imp-sk-tests`.
- Version bumps, `v*` tags, pre-tag clippy gate, release workflow -> `imp-sk-release`.
- Perf complaints (lag, jank, stutter, slow scroll, frame budget, CPU spikes): reproduce with real input, `sample` profiling, draw-share red/green, dev-profile opt levels -> `imp-sk-perf`.
- Vendored GPUI internals (`vendor/gpui*`, gpui-component): list/ListState/measure_all semantics, TextView/markdown pipeline, scrollbar, minimal vendor patches + pinned source audits -> `imp-sk-gpui-vendor`.

Imps are the default path, but never a hard blocker: if the runtime is down or
an imp repeatedly stalls, continue directly and mention the skipped imp in the
final answer. Do not fan out broadly by default; use the primary owner imp plus
one relevant cross-cutting imp when needed.

Local lesson overlays are not repo policy. Only reviewed promoted lessons,
regression tests/probes, or `AGENTS.md` updates affect general routing. Lessons
may guide future imp runs, but never override user instructions, dirty-work
preservation, or this file.

## UI Consistency and Shared Component Contract

When touching app UI, treat shared components and theme/chrome tokens as the source of truth. Do not build one-off UI when an existing component, shell, list item, input, footer, popup, or token can be reused or extended.

Before adding or changing UI:

1. Start with `GLOSSARY.md` to identify the owning surface and nearby implementation files.
2. Inspect the current surface, related tests, and the shared component entry points before editing.
3. Check `src/components/mod.rs` and the relevant component modules before creating any new UI helper.
4. Prefer extending the shared component library over adding surface-local render helpers.
5. If a new reusable primitive is needed, add it under `src/components/**` or the appropriate theme/chrome/design layer and use it from the surface. Do not bury reusable UI in one prompt, built-in, Agent Chat, or main-window renderer.

Shared UI entry points to check first:

- Inputs/search/menu fields: `src/components/text_input.rs`, `src/components/text_input/**`, `src/components/inline_prompt_input.rs`, `src/components/inline_dropdown/**`, `src/components/inline_picker.rs`, and `src/components/inline_popup_window.rs`.
- List rows and sections: `src/components/unified_list_item/**`; preserve existing `crate::list_item` usage where that is the current surface contract, but do not invent a third row system.
- Prompt shells and prompt chrome: `src/components/prompt_layout_shell.rs`, `src/components/prompt_container.rs`, `src/components/prompt_footer.rs`, and `src/components/minimal_prompt_shell.rs`.
- Footer and hint strips: `src/components/hint_strip.rs`, `src/components/footer_chrome.rs`, `src/footer_popup.rs`, and native footer handling in `src/app_impl/ui_window.rs`.
- Main-window chrome/layout: `src/components/main_view_chrome.rs`, `src/main_sections/**`, `src/render_script_list/**`, and `src/app_layout/**`.
- Empty/info/non-list states: `src/components/info_state.rs` and `src/components/non_list_state.rs`.
- Forms/buttons/toasts/shortcuts: `src/components/form_fields/**`, `src/components/button.rs`, `src/components/toast/**`, and `src/components/shortcut_recorder.rs`.

Theme and visual values must be tokenized:

- Resolve colors and chrome surfaces through `crate::theme`, especially `AppChromeColors::from_theme`, `PromptColors`, theme opacity, and the design token layers.
- Use chrome/layout constants from `src/ui/chrome/tokens.rs`, `src/theme/**`, `src/designs/core/**`, and `src/designs/traits/**`.
- Do not hardcode new colors, opacity values, spacing, typography, border radii, borders, popup surfaces, vibrancy behavior, or chrome layer semantics in surface renderers when an existing token/helper exists.
- If a visual value needs to become standard, add or extend a token/helper in the appropriate shared layer so theme changes propagate automatically.

Cross-surface behavior must stay predictable:

- Main window, prompt/make windows, built-ins, and Agent Chat/Agent Chat should share inputs, menu/search behavior, list rows, prompt shells, hint strips, footer affordances, popup/dropdown mechanics, and chrome tokens wherever possible.
- Actions UI should feel like the main list: same row language, same search treatment, same shortcut/keycap conventions, and no extra local chrome unless the owning contract requires it.
- Expanded/preview surfaces may differ in layout, but their list side, footer, and chrome should still use the shared anatomy and tokens.
- Any intentional divergence must be documented in the code or PR summary with the owning surface, the reused alternatives considered, and why the shared component could not fit.

# Agent Cargo Wrapper

`./dev.sh` runs `cargo watch` on the shared `target/` dir continuously. Bare `cargo build/test/check/clippy` from an AI agent contends on `target/.cargo-lock` and stalls for minutes ("Blocking waiting for file lock on build directory").

All agent-driven cargo invocations MUST go through `./scripts/agentic/agent-cargo.sh`, which defaults to the bounded shared `CARGO_TARGET_DIR=target-agent/pools/agent-debug` pool with a visible lock. Examples:

- `./scripts/agentic/agent-cargo.sh test --lib notes_editor::spine`
- `./scripts/agentic/agent-cargo.sh check --lib`
- `./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`

Use `SCRIPT_KIT_CARGO_TARGET_POOL=<name>` for an intentional shared pool, and set `SCRIPT_KIT_AGENT_TARGET_MODE=exclusive` only when a task truly needs a per-agent cache under `target-agent/agents/<agent-id>`. Do not run bare `cargo` against this repo while `./dev.sh` may be running.

Disk policy: the wrapper enforces a total `target-agent` budget at lock acquisition (`SCRIPT_KIT_AGENT_TARGET_BUDGET_GB`, default 40) plus a free-disk floor (`SCRIPT_KIT_AGENT_MIN_FREE_GB`, default 25), evicting least-recently-used unlocked pools before building. Extra pools are therefore ephemeral by design — do NOT mint a pool per parallel task. When a task needs a stable binary path, export an APFS clone instead: `SCRIPT_KIT_AGENT_ARTIFACT_NAME=<task> ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui` produces `target-agent/artifacts/<task>/script-kit-gpui` (~0 bytes, replaced atomically on rebuild). Dev builds use `CARGO_PROFILE_DEV_DEBUG=line-tables-only` and non-default pools disable incremental; both respect pre-set env overrides.

# Source Audit Test Policy

Source-audit tests (tests that `read_to_string`/`include_str!` app source and assert on its text) are decision locks, not behavior coverage. They are a scarce resource — do NOT mint one per feature pass.

Reality check: roughly three quarters of `tests/` predates this policy and is source audits. That corpus is grandfathered behind `tests/source_audit_ratchet.rs` (shrink-only) — its prevalence is NOT precedent for writing more. New invariants climb the ladder below, and when a grandfathered audit blocks a legitimate refactor, apply the pruning rule instead of patching its strings.

Enforcement ladder — pick the highest rung that can express the invariant:

1. **Compiler/type system** — exhaustive `match` without a wildcard arm, newtypes for tokens, visibility. If the compiler can enforce it, do not write a test for it.
2. **Lints** — `#[deny]` attributes, clippy `disallowed-methods`/`disallowed-types` in `clippy.toml`.
3. **Behavior test** — `#[gpui::test]`/`TestAppContext`, or a unit test on the extracted logic.
4. **Runtime proof** — a devtools probe script under `scripts/agentic/` for window/focus/render behavior tests cannot reach.
5. **Source audit** — last resort, only for genuinely load-bearing architectural invariants (e.g. the footer blur trio) that no higher rung can express.

When a source audit is justified:

- Prefer asserting the **absence of a dangerous pattern** (e.g. no `_ =>` wildcard, no `cx.notify()` in a hot path) over the presence of exact formatted code.
- Scope assertions with a `function_body`-style structural helper, not whole-file substring search.
- NEVER assert exact occurrence counts of formatted source lines (`source.matches(...).count() == N`); enumerate the expected sites explicitly instead. `tests/source_audit_ratchet.rs` enforces this.
- Document in the test's doc comment WHY the invariant exists, so a failing assertion can be evaluated rather than blindly appeased.

Pruning rule: when a source audit fails on legitimate refactors (no behavior change) for the third time, do not patch the string again — rewrite it structurally, move it up the ladder, or delete it.

# Post-Task Checklist

After every task, before responding to the user:

- [ ] Run the smallest source, test, build, or runtime proof that can fail for the changed behavior.
- [ ] Use `./scripts/agentic/agent-cargo.sh` (not bare `cargo`) for any cargo invocation while `./dev.sh` may be running.
- [ ] Report any skipped verification and why it was skipped.

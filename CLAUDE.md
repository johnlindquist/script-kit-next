For a map of main UI surfaces to code implementation, see [GLOSSARY.md]

# Before Starting Work

- Inspect the relevant source, tests, and repo-local skills before editing.
- Prefer current code and generated artifacts over stale notes or memory.
- Keep edits narrowly scoped and verify them with the smallest check that can fail for the changed behavior.
- Keep tool-facing root docs in place: `README.md`, `CLAUDE.md`, `AGENTS.md`, and `.impeccable.md`.

## Oracle / Packx Bundle Context

For Oracle review or `oracle-packx` work in this repository, include the repo process context in the bundle or prompt unless the user explicitly excludes it: `AGENTS.md`, the owning `.agents/skills/<skill>/SKILL.md`, and relevant source, tests, generated contracts, and verification notes.

If a `packx` preview with include globs unexpectedly matches `0` files in this repository, rebuild the bundle from an explicit path list instead of widening blindly. A reliable workaround is:

```bash
rg --files <scope> | rg '<owners-or-patterns>' > /tmp/script-kit-gpui-packx-files.txt
xargs packx --preview --no-interactive -x "**/CLAUDE.md" < /tmp/script-kit-gpui-packx-files.txt
xargs packx --limit 900k --strip-comments --minify -f markdown --no-interactive --stdout -x "**/CLAUDE.md" < /tmp/script-kit-gpui-packx-files.txt > ~/.oracle/bundles/<slug>.txt
```

Use this when directory/include-glob matching undercounts relevant files; keep `CLAUDE.md` excluded and verify the preview count plus final non-empty bundle before consulting Oracle.

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

- `./scripts/agentic/agent-cargo.sh test --lib context_picker`
- `./scripts/agentic/agent-cargo.sh check --lib`
- `./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`

Use `SCRIPT_KIT_CARGO_TARGET_POOL=<name>` for an intentional shared pool, and set `SCRIPT_KIT_AGENT_TARGET_MODE=exclusive` only when a task truly needs a per-agent cache under `target-agent/agents/<agent-id>`. Do not run bare `cargo` against this repo while `./dev.sh` may be running.

# Post-Task Checklist

After every task, before responding to the user:

- [ ] Run the smallest source, test, build, or runtime proof that can fail for the changed behavior.
- [ ] Use `./scripts/agentic/agent-cargo.sh` (not bare `cargo`) for any cargo invocation while `./dev.sh` may be running.
- [ ] Report any skipped verification and why it was skipped.

Goal: Replace all confirm/deny modal implementations with one shared, Add Shortcut-style confirm modal system, prove every confirm modal route uses it consistently, add runtime modal styling controls to the existing dev style tool, and expose the first SDK modal API as `confirm`.

Scope:
- In scope: `/Users/johnlindquist/dev/script-kit-gpui`; every in-app confirm/deny modal or destructive confirmation surface reachable from the main window, prompts, built-ins, Notes, clipboard/file/script actions, stdin commands, and SDK-driven prompt flows; modal visual styling, dismiss behavior, keyboard routing, accessibility/element metadata, DevTools/runtime receipts, and SDK bindings.
- In scope: Add Shortcut / shortcut recorder behavior as the design reference, including theme, chrome, dismiss affordances, footer/key handling, focus behavior, and feature completeness.
- In scope: known inconsistent routes such as the Quit Script Kit confirmation modal, destructive confirmations, remove shortcut/script/file/clipboard flows, `openConfirmPrompt`, `showShortcutRecorder`, Notes delete confirmations, and confirm-style keyboard shortcut flows where an input field is part of the confirm modal.
- In scope: the first SDK API should be named `confirm`, following the single-word SDK API pattern.
- In scope: modal style tuning belongs in the existing dev style tool beside the Agent Chat dev styling work, following the patterns in `src/dev_style_tool/**`.
- Out of scope: Actions Menu, trigger popups, hover/dropdown menus, browse panels, choice popups, unrelated prompt layout redesign, unrelated action/menu/list row changes, changing command semantics, changing destructive-action defaults, deleting root docs, or bypassing existing safety confirmations to simplify the migration.

Baseline and target:
- Baseline: confirm/deny modals are not guaranteed to share one implementation; `GLOSSARY.md` currently maps Confirm Popup to `src/confirm/mod.rs`, Add Shortcut routes through the shortcut recorder action path, and Quit Script Kit / destructive confirmation routes have separate rendering and behavior paths.
- Target: a checked-in confirm modal inventory lists every discovered confirm modal route, owner file, current implementation path, migrated shared component path, verification status, and any excluded non-modal surfaces.
- Target: all true confirm/deny modals render through one shared component or wrapper owned under the shared component/theme/chrome layer, with no remaining one-off confirm modal rendering paths.
- Target: the Quit Script Kit modal and Add Shortcut modal visibly share the same modal shell, theme tokens, dismiss controls, keyboard behavior, focus lifecycle, footer/keycap language, and element metadata shape.
- Target: the existing dev style tool has modal styling controls beside the Agent Chat style controls and can open representative confirm modal variants while tweaking modal style tokens at runtime without code changes for each adjustment.
- Target: SDK users can programmatically open confirm modals through documented, typed `confirm` APIs for Scripts, Scriptlets, and extension flows, with tests proving host/SDK contract compatibility.

Suggested starting points:
- Start with `GLOSSARY.md`, especially the Popups & Dialogs section and `Confirm Popup`.
- Audit source with `rg -n "modal|Modal|confirm|Confirm|ShortcutRecorder|openConfirmPrompt|showShortcutRecorder|Quit Script Kit|add_shortcut|configure_shortcut|remove_shortcut" src tests crates scripts`.
- Inspect shared UI entry points before editing: `src/components/mod.rs`, `src/components/shortcut_recorder.rs`, `src/components/prompt_layout_shell.rs`, `src/components/prompt_container.rs`, `src/components/prompt_footer.rs`, `src/components/footer_chrome.rs`, `src/components/button.rs`, `src/components/toast/**`, `src/theme/**`, `src/ui/chrome/tokens.rs`, and `src/designs/**`.
- Inspect modal and confirmation owners: `src/confirm/mod.rs`, `src/main_sections/app_view_state.rs`, `src/app_layout/build_layout_info.rs`, `src/app_layout/collect_elements.rs`, `src/app_impl/startup_new_actions.rs`, `src/app_impl/startup_new_arrow.rs`, `src/app_impl/tests.rs`, `src/stdin_commands/mod.rs`, `src/execute_script/mod.rs`, `src/app_actions/handle_action/shortcuts.rs`, `src/app_actions/handle_action/scripts.rs`, `src/app_actions/handle_action/files.rs`, `src/app_actions/handle_action/clipboard.rs`, `src/notes/window/**`, and related tests.
- Inspect existing dev style tool and Agent Chat styling patterns before adding modal controls: `src/dev_style_tool/mod.rs`, `src/dev_style_tool/agent_chat_catalog.rs`, `src/dev_style_tool/runtime_overrides.rs`, `src/dev_style_tool/render.rs`, `src/dev_style_tool/export.rs`, `src/dev_style_tool/kitchen_sink_targets.rs`, `tests/dev_style_tool_agent_chat_contract.rs`, `tests/dev_style_tool_runtime_style_contract.rs`, and `tests/dev_style_tool_window_contract.rs`.
- Use recent commit history as context for placement and conventions, especially commits such as `2ff33938c Add Agent Chat dev style controls`, `16ddd4012 Organize dev style control navigation`, `dc0126057 Add dev style kitchen sink fixtures`, and `af210544c fix(dev-style): satisfy clippy for agent chat knobs`.
- Inspect repo-local DevTools guidance in `.agents/skills/script-kit-devtools/SKILL.md` and prefer existing runtime inspection primitives before adding new proof-only paths.
- Preserve the repo rule that cargo invocations go through `./scripts/agentic/agent-cargo.sh`, not bare `cargo`.

Measurement and verification:
- Commit or update a confirm modal inventory artifact, preferably `.goals/receipts/modal-inventory.md` or another clearly named repo-local receipt, before migration and keep it current as routes are migrated.
- Add focused unit/contract tests that fail if a confirm modal route bypasses the shared modal component, hardcodes visual values that should be tokens, loses keyboard dismissal, or drops semantic element metadata.
- Add focused source-audit tests proving Actions Menu, trigger popups, and similar menu/dropdown surfaces are excluded from the confirm modal migration rather than accidentally treated as modals.
- Add or update runtime DevTools coverage so representative modal routes can be opened and inspected for shared shell identity, theme token usage, focus owner, dismiss controls, route stack, button state, footer/key routing, and element bounds.
- Verify at minimum these runtime scenarios: Add Shortcut modal, Quit Script Kit confirmation, one destructive file/script/clipboard confirmation, one Notes delete confirmation, one stdin/SDK `confirm` flow, and the modal controls inside the existing dev style tool.
- Use `./scripts/agentic/agent-cargo.sh` for Rust checks/tests, for example focused tests first and broader `check --lib` / relevant test binaries when the shared layer changes.
- Run any existing TypeScript/SDK checks that cover generated SDK types or script-facing APIs, and add/update SDK docs or generated contracts if the repo has an established generation step.
- For visual proof, capture before/after screenshots or DevTools layout receipts for Add Shortcut and Quit Script Kit, and include evidence that the old visual divergence is gone without regressing destructive confirmation safety.

Environment:
- Work in the local macOS GPUI app environment in `/Users/johnlindquist/dev/script-kit-gpui`.
- Runtime proof should use the repo-local agentic/devtools scripts where available and should target real app routes, not only source snapshots.
- Use existing theme/design token infrastructure rather than hardcoded modal colors, opacity, spacing, border radius, typography, vibrancy, or chrome semantics.
- Preserve existing dirty-tree work by other agents or the user; do not reset, stash, checkout, or revert unrelated changes.

Progress tracking:
- Keep a visible confirm modal inventory / migration checklist with statuses: discovered, classified, migrated, tested, runtime-proven, documented, or excluded as non-modal.
- Make coherent commits by phase when possible: audit/inventory, shared component migration, design dev tool, SDK exposure, verification/docs.
- Report concrete blockers immediately when a route cannot be runtime-proven, naming the missing primitive or app-launch issue rather than treating source inspection as proof.

Completion requirements:
- Show the final confirm modal inventory with every discovered confirm modal route accounted for and Actions Menu / trigger popups explicitly excluded.
- Show final measurement against the baseline and target, including the exact tests/checks/runtime receipts run.
- Demonstrate Add Shortcut and Quit Script Kit use the same shared modal system and match the agreed design anatomy.
- Demonstrate the existing dev style tool can tweak confirm modal style at runtime beside Agent Chat style controls.
- Demonstrate SDK `confirm` usage from at least one script-facing flow and confirm typed/docs/generated artifacts are updated if applicable.
- Clean up failed experiments, temporary instrumentation, unused duplicate modal code, and unrelated churn.
- Run final verification and summarize residual risks, skipped checks, or intentionally documented exceptions before declaring the goal complete.

Open decisions:
- Decide the exact `confirm` option and return-value shape after inspecting the current SDK/generated API conventions.
- Decide whether `confirm` should initially support only confirm/cancel text plus body/title/variant, or also expose the shortcut-recorder/input-field variant in the first slice.
- Decide the exact dev style tool tab/group names for modal controls so they fit the existing navigation and export schema.

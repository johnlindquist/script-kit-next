# Storybook

Script Kit Storybook is the visual state lab for launcher surfaces, popup windows, and secondary windows.

The durable goal is to represent app states, not keep every past design exploration in the primary catalog. Experiments can stay as archived helper code, but registered stories should describe supported visual states with real render paths.

## Catalog Roles

Catalog roles separate production-state coverage from migration leftovers and selectable visual experiments.

- `canonicalState` means story variants represent supported app states.
- `adoptableVariation` means variants can drive a live surface through persisted Storybook selection.
- `designExperiment` means an archived comparison study, not a primary registered story.

The primary registered catalog should contain only `canonicalState` and `adoptableVariation` stories. `main-menu`, `dictation-states`, `confirm-popup-states`, `context-picker-popup-states`, `shortcut-recorder-states`, `notes-window-states`, `acp-chat-states`, `actions-dialog-states`, `mini-ai-chat-states`, `built-in-browser-states`, `component-primitives-states`, and `utility-builtin-states` are canonical state stories; footer, input, actions dialog visual styles, and mini ACP chat visual styles are adoptable variations until state-first coverage replaces or complements them.

## Representation Quality

Every Storybook variant should declare how closely it matches runtime UI so agents can judge evidence quality.

- `liveSurface` renders through the live surface path or a read-only live override.
- `presenterFixture` renders the same presenter as runtime with deterministic data.
- `runtimeFixture` is an old PNG-backed capture path and must not appear in the registered catalog.
- Missing representation metadata should be treated as migration debt.

The preferred order is `liveSurface`, then `presenterFixture`. Hand-built mockups are acceptable only for archived `designExperiment` code. Old runtime fixture experiments should stay unregistered until they are replaced by live or presenter-backed coverage.

## Initial State Matrix

The first canonical matrix should cover the launcher and dictation states before expanding to every routed surface.

Main menu should cover populated results, empty results, selected row, bottom-of-list footer-safe reveal, frontmost-app paste, ACP-ready footer, and ACP-not-ready footer. `dictation-states` covers the live compact capsule for idle/hidden, quiet recording, active speech, Script Kit target, ACP target, external-app target, stop confirmation, transcribing, finished, and error.

Confirm popup covers ten variants: the live `AppView::ConfirmPrompt` in-window state plus nine destructive-warning popup treatments for compare-mode design review. The in-window variant is a presenter fixture that mirrors the shipping route — title + body fill the main content area and the native footer reuses Apply/Close slots labeled per `ParentConfirmOptions`. Slash/mention context picker, shortcut recorder, Notes window, ACP Chat, Actions Dialog, Mini ACP Chat, built-in list browser, shared component primitive, and one-off utility built-in coverage is registered as canonical presenter-backed state stories. Next expansion targets are narrower prompt entities such as form, path, env, editor, and terminal prompts.

## Migration Rules

Storybook cleanup should preserve evidence before deleting exploratory stories.

First, register any live or adoptable surface already referenced by diagnostics. Then remove design-experiment wrappers from the primary registry and replace them with canonical state stories. PNG-backed runtime fixture stories should be removed instead of preserved as supported coverage.

## Verification Contract

Storybook changes need machine-readable catalog checks plus visual verification for user-facing states.

At minimum, `--catalog-json` should expose story roles, surfaces, comparable status, representation metadata, and adopted surface coverage. Visual work should add screenshot capture or layout proof for the affected story before deleting older coverage.

## Related Pages

These pages define the visual and routing constraints Storybook must represent.

- [[design]]
- [[surfaces]]
- [[windowing]]

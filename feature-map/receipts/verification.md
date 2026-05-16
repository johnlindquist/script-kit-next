# Feature Map Verification

This receipt tracks what was verified while creating the atlas scaffold and benchmark chapter.

## Current State

- `feature-map/raw-oracle/` preserves all completed Oracle session prompts, bundle maps, extracted answers, full logs, and session metadata.
- [features/006-notes-window.md](../features/006-notes-window.md) is the benchmark distilled chapter.
- [features/001-main-menu.md](../features/001-main-menu.md) distills the first launcher Oracle pass, including source filters, legacy triggers, actions, shortcut recording, and config-backed shortcut persistence.
- [features/002-file-search.md](../features/002-file-search.md) distills the File Search pass, including root Files, `f:` source filters, dedicated mini/full File Search, file actions, drag-out, and attachment portal boundaries.
- [features/003-agent-chat-context.md](../features/003-agent-chat-context.md) distills the ACP/context pass, including entry paths, embedded/detached lifecycle, composer tokens, portal staging, agent/model setup, and automation targets.
- [features/004-mcp-sdk-protocol.md](../features/004-mcp-sdk-protocol.md) distills the MCP/SDK/protocol pass, including stdin command states, query receipts, `waitFor`/`batch`, MCP resources, MCP tools, SDK helpers, and agentic proof routing.
- [features/005-built-in-filterable-surfaces.md](../features/005-built-in-filterable-surfaces.md) distills the built-in filterable surfaces pass, including shared visible-row contracts, Clipboard History, App Launcher, Window Switcher, Browser Tabs, Emoji Picker, Process Manager, actions, and state/elements receipts.
- [features/007-root-notes.md](../features/007-root-notes.md) distills the root Notes pass, including metadata-only passive rows, `n:`/`notes:` source filters, passive frame isolation, source-only browse, and the non-toggle Notes open path.
- [features/008-root-clipboard-history.md](../features/008-root-clipboard-history.md) distills the root Clipboard History pass, including opt-in metadata-only passive rows, `c:`/`clipboard:` source filters, cache-stable frames, and the Enter paste route.
- [features/009-root-dictation-history.md](../features/009-root-dictation-history.md) distills the root Dictation History pass, including disabled-by-default transcript-safe metadata rows, `d:`/`dictation:` source filters, local JSONL cache behavior, and explicit transcript paste loading.
- [features/010-root-acp-history.md](../features/010-root-acp-history.md) distills the root ACP History pass, including AI Conversations passive rows, `ai:`/`conversations:` source filters, cache-stable saved-conversation metadata, and the shared resume helper path.
- [features/011-root-source-actions.md](../features/011-root-source-actions.md) distills the root source actions pass, including captured MainList subjects, typed root action ids, source-specific action palettes, content-light receipts, and root file handoffs.
- [features/012-root-source-filters.md](../features/012-root-source-filters.md) distills the root source filters pass, including source head parsing, stripped queries, source-only browse, status rows, Files lazy paging, source-filter frame isolation, and automation receipts.
- [raw-oracle/012-root-source-filters/](../raw-oracle/012-root-source-filters/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `root-source-filters-atlas`.
- [features/013-scriptlist-special-entry-triggers.md](../features/013-scriptlist-special-entry-triggers.md) distills the ScriptList special-entry trigger pass, including `~`, `/`, `@`, `>`, and `?` handoffs, negative-route invariants, destination ownership, automation proof paths, and route-specific risks.
- [raw-oracle/013-scriptlist-special-entry-triggers/](../raw-oracle/013-scriptlist-special-entry-triggers/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `scriptlist-special-triggers-atlas`.
- [features/014-quick-terminal-pty.md](../features/014-quick-terminal-pty.md) distills the Quick Terminal PTY pass, including `QuickTerminalView`, `TermPrompt`, warm PTY reuse, terminal key ownership, native footer, apply-back, path cwd handoff, zsh prompt suppression, and proof recipes.
- [raw-oracle/014-quick-terminal-pty/](../raw-oracle/014-quick-terminal-pty/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `quick-terminal-pty-atlas`.
- [features/015-sdk-term-prompt.md](../features/015-sdk-term-prompt.md) distills the SDK TermPrompt pass, including `term()`, `AppView::TermPrompt`, full-height sizing, terminal actions, output return semantics, automation identity, and Quick Terminal boundary risks.
- [raw-oracle/015-sdk-term-prompt/](../raw-oracle/015-sdk-term-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, session metadata, and failed-attempt logs for the successful CLI fallback `sdk-term-atlas-cli`.
- [features/016-prompt-runtime-core.md](../features/016-prompt-runtime-core.md) distills the core prompt runtime pass, including `arg()`, `select()`, `div()`, `md()`, prompt ids, submit paths, actions hosts, automation receipts, and prompt runtime boundaries.
- [raw-oracle/016-prompt-runtime-core/](../raw-oracle/016-prompt-runtime-core/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `prompt-runtime-core-atlas`.
- [features/017-form-fields-prompt.md](../features/017-form-fields-prompt.md) distills the Form and Fields pass, including `form()`, `AppView::FormPrompt`, field focus, validation, form actions, automation receipts, and the current incomplete `fields()` GPUI backend.
- [raw-oracle/017-form-fields-prompt/](../raw-oracle/017-form-fields-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `form-fields-prompt-atlas`.
- [features/018-editor-template-prompt.md](../features/018-editor-template-prompt.md) distills the Editor and Template pass, including `editor()`, `template()`, full-height editor sizing, snippet tabstops, TemplatePrompt validation, editor actions, and automation gaps.
- [raw-oracle/018-editor-template-prompt/](../raw-oracle/018-editor-template-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `editor-template-prompt-atlas`.
- [features/019-path-prompt.md](../features/019-path-prompt.md) distills the Path Prompt pass, including SDK `path()`, PathPrompt browsing/filtering, footer Select ownership, typed path actions, File Search boundaries, and protocol receipts.
- [raw-oracle/019-path-prompt/](../raw-oracle/019-path-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `path-prompt-atlas`.
- [features/020-drop-prompt.md](../features/020-drop-prompt.md) distills the Drop Prompt pass, including SDK `drop()`, empty-submit disabled state, DropPrompt footer ownership, dropped-file metadata elements, adjacent drop boundaries, and unproven event-wiring gaps.
- [raw-oracle/020-drop-prompt/](../raw-oracle/020-drop-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `drop-prompt-atlas`.
- [features/021-env-prompt.md](../features/021-env-prompt.md) distills the Env Prompt pass, including SDK `env()`, process-env and promptFn fast paths, encrypted local secret storage, redaction, footer ownership, and stale Keychain/options gaps.
- [raw-oracle/021-env-prompt/](../raw-oracle/021-env-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `env-prompt-atlas`.
- [features/022-hotkey-prompt.md](../features/022-hotkey-prompt.md) distills the Hotkey Prompt pass, including SDK `hotkey()` stub status, the implemented shortcut recorder, config.ts mutation, live hotkey registration, removal, and automation gaps.
- [raw-oracle/022-hotkey-prompt/](../raw-oracle/022-hotkey-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `hotkey-prompt-atlas`.
- [features/023-mini-micro-prompts.md](../features/023-mini-micro-prompts.md) distills the Mini/Micro pass, including SDK `mini()`/`micro()`, stale warning copy, MiniPrompt footer/sizing, MicroPrompt footerless behavior, shared choice automation, and simulateKey gaps.
- [raw-oracle/023-mini-micro-prompts/](../raw-oracle/023-mini-micro-prompts/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `mini-micro-prompts-atlas`.
- [features/024-confirm-prompt-and-dialogs.md](../features/024-confirm-prompt-and-dialogs.md) distills the Confirm pass, including SDK `confirm()`, in-window `ConfirmPrompt`, parent popup fallback, destructive action gates, fail-closed behavior, semantic ids, and verification gaps.
- [raw-oracle/024-confirm-prompt-and-dialogs/](../raw-oracle/024-confirm-prompt-and-dialogs/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `confirm-prompt-dialogs-atlas`.
- [features/025-system-feedback-and-prompt-control.md](../features/025-system-feedback-and-prompt-control.md) distills the System Feedback and Prompt Control pass, including implemented `hud()`/`setActions()`/`setInput()` behavior, stubbed system API boundaries, actions, batch input, and verification gaps.
- [raw-oracle/025-system-feedback-and-prompt-control/](../raw-oracle/025-system-feedback-and-prompt-control/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `system-feedback-prompt-control-atlas`.
- [features/026-clipboard-selected-text-accessibility.md](../features/026-clipboard-selected-text-accessibility.md) distills the Clipboard/Selected Text/Accessibility pass, including clipboard text behavior, image clipboard gaps, selected-text focus/permission flow, typed receipts, and privacy-safe logging rules.
- [raw-oracle/026-clipboard-selected-text-accessibility/](../raw-oracle/026-clipboard-selected-text-accessibility/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `clipboard-selected-text-accessibility-atlas`.
- [features/027-keyboard-mouse-apis.md](../features/027-keyboard-mouse-apis.md) distills the Keyboard/Mouse pass, including SDK-visible unsupported helpers, SDK/Rust protocol shape mismatch, no native input receipts, reliable automation alternatives, and false-positive test risks.
- [raw-oracle/027-keyboard-mouse-apis/](../raw-oracle/027-keyboard-mouse-apis/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `keyboard-mouse-apis-atlas`.
- [features/028-window-control-visual-inspection.md](../features/028-window-control-visual-inspection.md) distills the Window Control and Visual Inspection pass, including show/hide/blur, debug grid, bounds, screenshots, layout info, no-envelope controls, screenshot privacy, and proof escalation.
- [raw-oracle/028-window-control-visual-inspection/](../raw-oracle/028-window-control-visual-inspection/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `window-control-visual-inspection-atlas`.
- [features/029-widget-media-find-apis.md](../features/029-widget-media-find-apis.md) distills the Widget/Media/Find pass, including widget controller stubs, media throw-before-send behavior, eyeDropper unsupported status, find backend gaps, and false-positive test risks.
- [raw-oracle/029-widget-media-find-apis/](../raw-oracle/029-widget-media-find-apis/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `widget-media-find-apis-atlas`.
- [features/030-acp-chat-sdk-apis.md](../features/030-acp-chat-sdk-apis.md) distills the ACP Chat SDK API pass, including implemented storage/direct APIs, UI-thread `aiStartChat`/`aiFocus`, context part and image behavior, and unproven append/send/system-prompt/subscription routes.
- [raw-oracle/030-acp-chat-sdk-apis/](../raw-oracle/030-acp-chat-sdk-apis/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `acp-chat-sdk-apis-atlas`.
- [features/031-legacy-chat-prompt.md](../features/031-legacy-chat-prompt.md) distills the legacy `chat()` prompt pass, including SDK callback and controller behavior, built-in AI fallback, setup card, actions, Mini AI reuse, persistence, handoff, and verification gaps.
- [raw-oracle/031-legacy-chat-prompt/](../raw-oracle/031-legacy-chat-prompt/) preserves the full Oracle prompt, bundle map, extracted answer, full log, session metadata, and failed-attempt logs for `legacy-chat-prompt-atlas`.
- [features/032-script-metadata-scriptlets.md](../features/032-script-metadata-scriptlets.md) distills the Script Metadata, Scriptlets, and Execution Catalog pass, including plugin-scoped discovery, metadata/schema extraction, duplicate binding validation, scriptlet Markdown parsing, scriptlet execution, MCP resources, and verification gaps.
- [raw-oracle/032-script-metadata-scriptlets/](../raw-oracle/032-script-metadata-scriptlets/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `script-metadata-scriptlets-atlas`.
- [features/033-sharing-clipboard-trust.md](../features/033-sharing-clipboard-trust.md) distills the Sharing and Clipboard Trust Install pass, including portable share bundles, share-vs-deeplink action behavior, clipboard watcher suppression, trust prompt, plugin install, and security boundaries.
- [raw-oracle/033-sharing-clipboard-trust/](../raw-oracle/033-sharing-clipboard-trust/) preserves the full Oracle prompt, bundle map, extracted answer, full log, session metadata, and failed-attempt logs for the successful CLI fallback `sharing-clipboard-trust-cli`.
- [features/034-permissions-assistant.md](../features/034-permissions-assistant.md) distills the Permissions and Permission Assistant pass, including Accessibility and Screen Recording assistant entry points, native overlay drag-source behavior, passive status APIs, MCP permission rows, dictation microphone preflight, and screenshot permission-proof boundaries.
- [raw-oracle/034-permissions-assistant/](../raw-oracle/034-permissions-assistant/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `permission-assistant-atlas`.
- [features/035-settings-theme-config-preferences.md](../features/035-settings-theme-config-preferences.md) distills the Settings, Theme, Config, and Preferences pass, including Settings Hub row projection, Theme Chooser preview/customization, config/runtime preference boundaries, theme payload storage, and config fingerprint proof.
- [raw-oracle/035-settings-theme-config-preferences/](../raw-oracle/035-settings-theme-config-preferences/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `settings-theme-config-preference-atlas`.
- [features/036-tray-lifecycle-distribution-updates.md](../features/036-tray-lifecycle-distribution-updates.md) distills the Tray Menu, App Lifecycle, Distribution, and Updates pass, including tray action identity, update-state sharing, About route behavior, MCP tray observations, shutdown, packaging, and release-manifest boundaries.
- [raw-oracle/036-tray-lifecycle-distribution-updates/](../raw-oracle/036-tray-lifecycle-distribution-updates/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `tray-lifecycle-distributi-atlas`.
- [raw-oracle/036-tray-menu-global-entry-points/](../raw-oracle/036-tray-menu-global-entry-points/) preserves a supplemental narrower Oracle pass for the tray/global-entry subset.
- [features/037-storybook-design-visual-verification.md](../features/037-storybook-design-visual-verification.md) distills the Storybook, Design Explorer, and Visual Verification pass, including catalog honesty, StoryBrowser preview/compare/adoption, Design Gallery and Picker proof, strict screenshots, and visual evidence risks.
- [raw-oracle/037-storybook-design-visual-verification/](../raw-oracle/037-storybook-design-visual-verification/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `storybook-design-visual-atlas`.
- [features/038-agent-skills-ai-context-catalog.md](../features/038-agent-skills-ai-context-catalog.md) distills the Agent Skills and AI Context Catalog pass, including plugin skills, typed context parts, MCP context resources, context preview, portal return, focused targets, SDK parts, and submit-time receipts.
- [raw-oracle/038-agent-skills-ai-context-catalog/](../raw-oracle/038-agent-skills-ai-context-catalog/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `agent-skills-ai-context-atlas`.
- [features/039-logging-diagnostics-transaction-observability.md](../features/039-logging-diagnostics-transaction-observability.md) distills the Logging, Diagnostics, and Transaction Observability pass, including compact dev-loop logs, safe structured logging, debug markers, protocol stats, transaction traces, MCP trace resources, replay safety, and AI preflight audits.
- [raw-oracle/039-logging-diagnostics-transaction-observability/](../raw-oracle/039-logging-diagnostics-transaction-observability/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `logging-diagnostic-observabil-atlas`.
- [raw-oracle/039-logging-diagnostics-transaction-observability/](../raw-oracle/039-logging-diagnostics-transaction-observability/) also preserves `answer-duplicate-retry.md`, `output-duplicate-retry.log`, and `session-duplicate-retry.json` for the duplicate retry `logging-diagnostic-observabil-atlas-2`.
- [features/040-main-window-sizing-surface-contracts.md](../features/040-main-window-sizing-surface-contracts.md) distills the Main Window Sizing and Surface Contracts pass, including Mini/Full sizing, ViewType ownership, AppView/SurfaceKind policy, native footer ids, generated surface matrices, and state-first surface receipts.
- [raw-oracle/040-main-window-sizing-surface-contracts/](../raw-oracle/040-main-window-sizing-surface-contracts/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `main-window-surface-atlas`.
- [features/045-launcher-trigger-edge-cases.md](../features/045-launcher-trigger-edge-cases.md) distills the launcher trigger edge-case pass, including the `~`, `/`, `@`, `>`, `?` matrix, source-filter and menu/capture boundaries, ACP picker staging, stale decoration risks, unsafe claims, and follow-up proof plan.
- [raw-oracle/045-launcher-trigger-edge-cases/](../raw-oracle/045-launcher-trigger-edge-cases/) preserves the full Oracle prompt, bundle map, extracted answer, full log, and session metadata for `launcher-trigger-edge-cases`.
- [features/046-shortcut-assignment-config-refresh.md](../features/046-shortcut-assignment-config-refresh.md) distills the shortcut assignment/config-refresh pass, including assignment/update/removal actions, recorder state, `config.ts.commands` writes, command IDs, live activation, refresh gaps, conflict gaps, and verification receipts.
- [raw-oracle/046-shortcut-assignment-config-refresh/](../raw-oracle/046-shortcut-assignment-config-refresh/) preserves the full Oracle prompt, bundle map, extracted answer, full log, session metadata, and failed thinking-chip attempt for `shortcut-refresh-atlas`.
- [index.md](../index.md) marks all 46 completed feature raw answers as maintained chapters.
- [receipts/oracle-loop-prompt.md](./oracle-loop-prompt.md) is the reusable loop prompt for the next Oracle/implement pass.

## Checks

Run before considering the migration done:

```bash
lat check
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md
```

Latest feature gate for `040-main-window-sizing-surface-contracts`:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `npm run build`, `lat check`, `jq empty`, and `git diff --check` passed. The generated explorer data reports 40 indexed features, 40 raw Oracle feature ids, 40 chapters, and no pending rows. The backlog scan returned no matches; exit code 1 is expected for `rg` with no results.

Follow-up build repair:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `feature_explorer/src/state/featureRuntime.ts` now builds explicit `FeatureRuntimeTransition[]` values with a typed accumulator, avoiding TypeScript `flatMap` inference failures after the atlas reached 40 generated features. All commands passed; the backlog scan returned no matches with exit code 1, as expected.

Supplemental Oracle archive:

```bash
cp feature_explorer/oracle/feature-xstate-001-005-prompt.md feature-map/raw-oracle/supplemental-feature-xstate-001-005/prompt.md
cp ~/.oracle/sessions/feature-xstate-001-005/output.log feature-map/raw-oracle/supplemental-feature-xstate-001-005/output.log
cp ~/.oracle/sessions/feature-xstate-001-005/meta.json feature-map/raw-oracle/supplemental-feature-xstate-001-005/session.json
```

Result: `feature-xstate-001-005` completed with a ChatGPT renderer error and only 17 output tokens, so it is archived as `supplemental-error` and is not counted as a completed feature chapter.

Latest feature gate for `041-main-menu-renderer-key-handling`:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `npm run build`, `lat check`, `jq empty`, and `git diff --check` passed. The generated explorer data reports 41 indexed features, 41 raw Oracle feature ids, 41 chapters, and no pending rows. The backlog scan returned no matches; exit code 1 is expected for `rg` with no results.

Latest feature gate for `042-menu-syntax-power-capture`:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `npm run build`, `lat check`, `jq empty`, and `git diff --check` passed. The generated explorer data reports 42 indexed features, 42 raw Oracle feature ids, 42 chapters, and no pending or incomplete raw rows. The backlog scan returned no matches; exit code 1 is expected for `rg` with no results.

Latest feature gate for `043-acp-sdk-runtime-apis`:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `npm run build`, `lat check`, `jq empty`, and `git diff --check` passed. The generated explorer data reports 43 indexed features, 43 raw Oracle feature ids, 43 chapters, and no pending or incomplete raw rows. The backlog scan returned no matches; exit code 1 is expected for `rg` with no results.

Latest feature gate for `044-mcp-protocol-runtime-bridge`:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `npm run build`, `lat check`, `jq empty`, and `git diff --check` passed. The generated explorer data reports 44 indexed features, 44 raw Oracle feature ids, 44 chapters, and no pending or incomplete raw rows. The backlog scan returned no matches; exit code 1 is expected for `rg` with no results.

Latest feature gate for `045-launcher-trigger-edge-cases`:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `npm run build`, `lat check`, `jq empty`, and `git diff --check` passed. The generated explorer data reports 45 indexed features, 45 raw Oracle feature ids, 45 chapters, and no pending or incomplete raw rows. The backlog scan returned no matches; exit code 1 is expected for `rg` with no results.

Latest feature gate for `046-shortcut-assignment-config-refresh`:

```bash
npm run build # from feature_explorer/
lat check
jq '.featureCount, .coverage' feature_explorer/src/data/features.generated.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer lat.md/feature-explorer.md lat.md/lat.md
jq empty feature-map/receipts/oracle-sessions.json
rg -n "\| Backlog \|" feature-map/index.md feature-map/features feature-map/receipts
```

Result: `npm run build`, `lat check`, `jq empty`, and `git diff --check` passed. The generated explorer data reports 46 indexed features, 46 raw Oracle feature ids, 46 chapters, and no pending or incomplete raw rows. The backlog scan returned no matches; exit code 1 is expected for `rg` with no results.

## Remaining Risk

- The benchmark chapter was distilled from source/lat context plus the completed Oracle answer; it is not a fresh runtime proof of Notes behavior.
- The first 46 completed feature Oracle answers are preserved and distilled into maintained chapters.
- `FEATURE_MAP.md` remains a first-pass compatibility file and may still contain compressed accumulated output.

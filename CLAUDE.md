# Before Starting Work

- Inspect the relevant source, tests, and repo-local skills before editing.
- Prefer current code and generated artifacts over stale notes or memory.
- Keep edits narrowly scoped and verify them with the smallest check that can fail for the changed behavior.
- Keep tool-facing root docs in place: `README.md`, `CLAUDE.md`, `AGENTS.md`, and `.impeccable.md`.

## Oracle / Packx Bundle Context

For Oracle review or `oracle-packx` work in this repository, include the repo process context in the bundle or prompt unless the user explicitly excludes it: `AGENTS.md`/`CLAUDE.md`, the owning `.agents/skills/<skill>/SKILL.md`, and relevant source, tests, generated contracts, and verification notes.

# Codex Repo Skills

Use the repo-local Codex skills in `.agents/skills/` as the primary task routing map. These are the canonical skill names for this repository; legacy `.claude/skills/*` names are migration source material only.

Codex may load a skill automatically when the task matches the skill description. For high-risk or broad work, explicitly name the skill in the prompt, for example `$acp-chat-core` or `$protocol-automation`.

For complex investigation, pair the skill with its read-only subagent brief in `.agents/subagents/`. Subagent briefs are prompts/checklists for read-only exploration; they do not automatically spawn subagents.

## Skill and Subagent Map

| Skill | Paired subagent | Primary ownership |
| --- | --- | --- |
| `script-kit-devtools` | `protocol-automation-reader` | Agent-facing DevTools primitives for inspecting, controlling, measuring, debugging, benchmarking, and proving real app UI behavior |
| `agentic-testing` | `agentic-testing-reader` | State-first runtime proof, screenshots only when needed, cleanup of launched sessions |
| `testing-quality-gates` | `testing-quality-gates-reader` | Choosing narrow checks, cargo/bun gates, release-vs-local validation |
| `dev-loop-observability` | `dev-loop-observability-reader` | Dev loop, compact logs, tracing, correlation IDs, runtime diagnostics |
| `gpui-ui-foundation` | `gpui-ui-foundation-reader` | GPUI layout, focus, keyboard handlers, theme usage, component lifecycle |
| `theme-config-preferences` | `theme-config-preferences-reader` | `config.ts`, `theme.json`, preferences, font/scale/runtime settings |
| `storybook-design` | `storybook-design-reader` | Storybook/design explorer, stories, variants, adoption, chrome audits |
| `launcher-surface-contracts` | `launcher-surface-contracts-reader` | `AppView`, `SurfaceKind`, surface contracts, current-view transitions |
| `window-resizing` | `window-resizing-reader` | Main-window presentation modes, Mini vs Full sizing, resize paths, resize audits |
| `main-menu-search-selection` | `main-menu-search-selection-reader` | Main menu filtering, grouped results, selection caches, fallback rows |
| `keyboard-focus-routing` | `keyboard-focus-routing-reader` | Global/local key intent routing, focus restoration, popup-first key handling |
| `escape` | `escape-reader` | Escape key close/back/cancel UX, direct-launch vs launcher-return behavior, physical/simulateKey parity |
| `actions-popups` | `actions-popups-reader` | Actions dialog, prompt/confirm popups, popup registry, route stack, resize |
| `builtin-filterable-surfaces` | `builtin-filterable-surfaces-reader` | Clipboard/app/process/emoji/current-app/design gallery filterable surfaces |
| `file-search-portals` | `file-search-portals-reader` | File search, attachment portals, browser/dictation/history portal return flows |
| `prompt-runtime` | `prompt-runtime-reader` | Prompt entities, prompt handler routes, prompt-specific state and rendering |
| `sdk-script-execution` | `sdk-script-execution-reader` | SDK preload, script metadata, script execution, Bun package/script behavior |
| `protocol-automation` | `protocol-automation-reader` | Stdin JSON protocol, `getState`, `getElements`, `waitFor`, `batch`, receipts |
| `mcp-context-resources` | `mcp-context-resources-reader` | MCP resources, `kit://context`, context schemas, resource-backed catalogs |
| `acp-chat-core` | `acp-chat-core-reader` | Agent Chat entry, embedded/detached ACP, model/agent/session behavior |
| `acp-context-composer` | `acp-context-composer-reader` | ACP composer, slash/mention picker, context parts, attachment tokens |
| `quick-terminal-pty` | `quick-terminal-pty-reader` | Quick Terminal, PTY lifecycle, terminal theming, apply-back behavior |
| `notes-window` | `notes-window-reader` | Notes window, notes browse, notes-hosted ACP, Markdown notes behavior |
| `dictation-media` | `dictation-media-reader` | Dictation, audio/media capture, transcript delivery, history resources |
| `platform-windowing-macos` | `platform-windowing-macos-reader` | macOS windowing, screenshots, AppKit/AX/focus, bundle/platform behavior |
| `storage-cache-security` | `storage-cache-security-reader` | Local storage, caches, SQLite, secrets, encryption, cleanup/security boundaries |

## Subagent Usage

Use a paired subagent when the task spans multiple modules, touches a high-risk surface contract, or needs evidence before editing. The subagent must stay read-only and return relevant files/symbols, invariants, the smallest verification command or agentic proof, and legacy `.claude/skills` material worth migrating.

Do not wait for a subagent when the task is small and the owning skill gives enough context. Subagents are not automatic; spawn them only when the user explicitly asks for subagents/parallel delegation or when the task has broad, noisy exploration that benefits from a read-only sidecar.

## Skill Selection Defaults

- Unknown user-reported UX/UI bugs, screenshots, or flexible app investigation: `script-kit-devtools`, plus the domain skill. Use `agentic-testing` recipes only when they directly match the bug or as regression proof.
- UI behavior or visual/runtime proof with an existing known recipe: `agentic-testing`, plus the domain skill.
- GPUI layout/focus/keyboard implementation: `gpui-ui-foundation` and `keyboard-focus-routing`.
- Escape close/back/cancel behavior: `escape`, plus the domain skill for the active surface.
- Surface routing or `AppView`/`SurfaceKind`: `launcher-surface-contracts`.
- Main-window width/height, Mini vs Full mode, or resize regressions: `window-resizing`.
- Actions dialog or attached popups: `actions-popups`.
- Filterable built-ins: `builtin-filterable-surfaces`.
- ACP/Agent Chat lifecycle: `acp-chat-core`.
- ACP composer/context tokens: `acp-context-composer`.
- Stdin protocol or automation receipts: `protocol-automation`. If the task is about how agents should inspect/control/measure the UI, also use `script-kit-devtools`.
- Script execution or SDK behavior: `sdk-script-execution`.
- Config/theme/preferences: `theme-config-preferences`.
- Notes, dictation, terminal, platform, and storage work should use their matching ownership skill.

## Main-Window Resizing

Before changing launcher/window sizing, read `.agents/skills/window-resizing/SKILL.md`, the paired reader brief when useful, and the domain skill for the surface being resized.

Do not call `update_window_size_deferred`, `update_window_size`, or `resize_to_view_sync(ViewType::ScriptList, ...)` after a presenter that already owns a Mini surface unless `calculate_window_size_params` proves that view still resolves to `ViewType::MiniMainWindow`.

For built-in filterable views, choose width from layout: Mini for single-column lists, Full for list-plus-preview/detail panes. Row count, command importance, or shortcut/tray entry source do not justify Full mode.

When fixing resize behavior in a dirty worktree, inspect `git status --short`, patch only the minimal responsible hunk, and add a focused source audit for the entry path that regressed.

# Post-Task Checklist

After every task, before responding to the user:

- [ ] Run the smallest source, test, build, or runtime proof that can fail for the changed behavior.
- [ ] Report any skipped verification and why it was skipped.

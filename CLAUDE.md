# Before starting work

- Run `lat search` to find sections relevant to your task. Read them to understand the design intent before writing code.
- Run `lat expand` on user prompts to expand any `[[refs]]` — this resolves section names to file locations and provides context.

## Oracle / packx bundle context

For Oracle review or `oracle-packx` work in this repository, include the repo process context in the bundle or prompt unless the user explicitly excludes it: `AGENTS.md`/`CLAUDE.md`, the owning `.agents/skills/<skill>/SKILL.md`, and relevant `lat.md/` pages found by `lat search`/`lat expand`. Include `lat.md/verification.md` whenever implementation or review checks are part of the question. If packx filters would omit that context, add explicit include filters or paste the relevant checklist into the Oracle prompt. The Oracle prompt should mention the `lat.md/` update rule and required `lat check`.

# Codex repo skills

Use the repo-local Codex skills in `.agents/skills/` as the primary task routing map. These are the canonical skill names for this repository; legacy `.claude/skills/*` names are migration source material only.

Codex may load a skill automatically when the task matches the skill description. For high-risk or broad work, explicitly name the skill in the prompt, for example `$acp-chat-core` or `$protocol-automation`.

For complex investigation, pair the skill with its read-only subagent brief in `.agents/subagents/`. Subagent briefs are prompts/checklists for read-only exploration; they do not automatically spawn subagents.

## Skill and subagent map

| Skill | Paired subagent | Primary ownership |
| --- | --- | --- |
| `lat-md` | `lat-md-reader` | `lat.md/` authoring, wiki links, code refs, `lat check` |
| `script-kit-devtools` | `protocol-automation-reader` | Agent-facing DevTools primitives for inspecting, controlling, measuring, debugging, benchmarking, and proving real app UI behavior |
| `agentic-testing` | `agentic-testing-reader` | State-first runtime proof, screenshots only when needed, cleanup of launched sessions |
| `testing-quality-gates` | `testing-quality-gates-reader` | Choosing narrow checks, cargo/bun/lat gates, release-vs-local validation |
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

## Subagent usage

Use a paired subagent when the task spans multiple modules, touches a high-risk surface contract, or needs evidence before editing. The subagent must stay read-only and return relevant files/symbols, applicable `lat.md` sections or generated contracts, invariants, the smallest verification command or agentic proof, and legacy `.claude/skills` material worth migrating.

Do not wait for a subagent when the task is small and the owning skill gives enough context. Subagents are not automatic; spawn them only when the user explicitly asks for subagents/parallel delegation or when the task has broad, noisy exploration that benefits from a read-only sidecar.

## Skill selection defaults

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

## Main-window resizing

Before changing launcher/window sizing, read `lat.md/windowing.md`, `lat.md/builtins.md`, `.agents/skills/window-resizing/SKILL.md`, and the domain skill for the surface being resized.

Do not call `update_window_size_deferred`, `update_window_size`, or `resize_to_view_sync(ViewType::ScriptList, ...)` after a presenter that already owns a Mini surface unless `calculate_window_size_params` proves that view still resolves to `ViewType::MiniMainWindow`.

For built-in filterable views, choose width from layout: Mini for single-column lists, Full for list-plus-preview/detail panes. Row count, command importance, or shortcut/tray entry source do not justify Full mode.

When fixing resize behavior in a dirty worktree, inspect `git status --short`, patch only the minimal responsible hunk, and add a focused source audit for the entry path that regressed.

# Post-task checklist (REQUIRED — do not skip)

After EVERY task, before responding to the user:

- [ ] Update `lat.md/` if you added or changed any functionality, architecture, tests, or behavior
- [ ] Run `lat check` — all wiki links and code refs must pass
- [ ] Do not skip these steps. Do not consider your task done until both are complete.

---

# What is lat.md?

This project uses [lat.md](https://www.npmjs.com/package/lat.md) to maintain a structured knowledge graph of its architecture, design decisions, and test specs in the `lat.md/` directory. It is a set of cross-linked markdown files that describe **what** this project does and **why** — the domain concepts, key design decisions, business logic, and test specifications. Use it to ground your work in the actual architecture rather than guessing.

# Commands

```bash
lat locate "Section Name"      # find a section by name (exact, fuzzy)
lat refs "file#Section"        # find what references a section
lat search "natural language"  # semantic search across all sections
lat expand "user prompt text"  # expand [[refs]] to resolved locations
lat check                      # validate all links and code refs
```

Run `lat --help` when in doubt about available commands or options.

If `lat search` fails because no API key is configured, explain to the user that semantic search requires a key provided via `LAT_LLM_KEY` (direct value), `LAT_LLM_KEY_FILE` (path to key file), or `LAT_LLM_KEY_HELPER` (command that prints the key). Supported key prefixes: `sk-...` (OpenAI) or `vck_...` (Vercel). If the user doesn't want to set it up, use `lat locate` for direct lookups instead.

# Syntax primer

- **Section ids**: `lat.md/path/to/file#Heading#SubHeading` — full form uses project-root-relative path (e.g. `lat.md/tests/search#RAG Replay Tests`). Short form uses bare file name when unique (e.g. `search#RAG Replay Tests`, `cli#search#Indexing`).
- **Wiki links**: `[[target]]` or `[[target|alias]]` — cross-references between sections. Can also reference source code: `[[src/foo.ts#myFunction]]`.
- **Source code links**: Wiki links in `lat.md/` files can reference functions, classes, constants, and methods in TypeScript/JavaScript/Python/Rust/Go/C files. Use the full path: `[[src/config.ts#getConfigDir]]`, `[[src/server.ts#App#listen]]` (class method), `[[lib/utils.py#parse_args]]`, `[[src/lib.rs#Greeter#greet]]` (Rust impl method), `[[src/app.go#Greeter#Greet]]` (Go method), `[[src/app.h#Greeter]]` (C struct). `lat check` validates these exist.
- **Code refs**: `// @lat: [[section-id]]` (JS/TS/Rust/Go/C) or `# @lat: [[section-id]]` (Python) — ties source code to concepts

# Test specs

Key tests can be described as sections in `lat.md/` files (e.g. `tests.md`). Add frontmatter to require that every leaf section is referenced by a `// @lat:` or `# @lat:` comment in test code:

```markdown
---
lat:
  require-code-mention: true
---
# Tests

Authentication and authorization test specifications.

## User login

Verify credential validation and error handling for the login endpoint.

### Rejects expired tokens
Tokens past their expiry timestamp are rejected with 401, even if otherwise valid.

### Handles missing password
Login request without a password field returns 400 with a descriptive error.
```

Every section MUST have a description — at least one sentence explaining what the test verifies and why. Empty sections with just a heading are not acceptable. (This is a specific case of the general leading paragraph rule below.)

Each test in code should reference its spec with exactly one comment placed next to the relevant test — not at the top of the file:

```python
# @lat: [[tests#User login#Rejects expired tokens]]
def test_rejects_expired_tokens():
    ...

# @lat: [[tests#User login#Handles missing password]]
def test_handles_missing_password():
    ...
```

Do not duplicate refs. One `@lat:` comment per spec section, placed at the test that covers it. `lat check` will flag any spec section not covered by a code reference, and any code reference pointing to a nonexistent section.

# Section structure

Every section in `lat.md/` **must** have a leading paragraph — at least one sentence immediately after the heading, before any child headings or other block content. The first paragraph must be ≤250 characters (excluding `[[wiki link]]` content). This paragraph serves as the section's overview and is used in search results, command output, and RAG context — keeping it concise guarantees the section's essence is always captured.

```markdown
# Good Section

Brief overview of what this section documents and why it matters.

More detail can go in subsequent paragraphs, code blocks, or lists.

## Child heading

Details about this child topic.
```

```markdown
# Bad Section

## Child heading

Details about this child topic.
```

The second example is invalid because `Bad Section` has no leading paragraph. `lat check` validates this rule and reports errors for missing or overly long leading paragraphs.

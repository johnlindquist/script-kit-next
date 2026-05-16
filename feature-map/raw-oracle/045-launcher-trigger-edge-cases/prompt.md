[launcher-trigger-edge-cases]

Project briefing:

- Repository: Script Kit GPUI, a Rust/GPUI desktop app with a ScriptList launcher, first-character handoffs, menu syntax, root source filters, Agent Chat, File Search, Quick Terminal, and actions popups.
- The feature map is a human-and-agent atlas. Each chapter must explain capabilities, entry points, interactions, APIs, state machines, receipt paths, and open risks clearly enough for humans and AI agents to operate the product.
- Repo process requires `lat.md/` updates for changed behavior/docs and `lat check` verification. For this atlas loop, every Oracle session's full output is preserved under `feature-map/raw-oracle/<feature-id>/` and distilled into `feature-map/features/<feature-id>.md`.
- Relevant ownership skills for this pass: `main-menu-search-selection`, `acp-context-composer`, `file-search-portals`, `quick-terminal-pty`, `actions-popups`, `protocol-automation`, and `testing-quality-gates`.

Goal:

Create a comprehensive focused feature-map reference for launcher special-character trigger edge cases and adjacent first-token routing:

- The narrow `~`, `/`, `@`, `>`, and `?` ScriptList first-character handoffs from feature 013.
- Source filters and source heads such as `f:`, `files:`, `n:`, `c:`, `ai:`, etc. from feature 012.
- Menu syntax and capture/power syntax boundaries from feature 042.
- Stale decoration/highlight risks when transitioning between menu syntax/source-filter decorated text and special-entry handoffs.
- Detached versus embedded ACP slash/mention picker behavior after `/` and `@`.
- The `?` actions-help disabled/no-op state and shared actions popup boundary.
- Exact negative cases like `/tmp`, `@browser`, `>deploy -- prod`, `:`, `+`, `!`, `#`, `;todo`, `todo:`, and `cal:`.

Current evidence:

- Feature 013 already maps the basic first-character handoffs, but it leaves edge cases around stale decorations and detached ACP picker behavior.
- Feature 012 maps root source filters including `f:` and empty browse states.
- Feature 042 maps power/capture syntax and sibling command/refine boundaries.
- The user explicitly wants every character trigger and special main-menu state captured, including examples like `f:` and `@`.

Bundle map:

- Repo process docs: `AGENTS.md`, `CLAUDE.md`.
- Owning and adjacent skills: main menu search, ACP context composer, file search portals, quick terminal, actions popups, protocol automation, testing gates.
- Lat docs: menu syntax, ACP chat, surfaces, automation, verification.
- Source excerpts: ScriptList filter classifier/dispatch/update paths, ACP launcher handoff helpers, menu syntax trigger popup/main hint/source heads/filter parser, render list/actions/file search, Quick Terminal opener.
- Tests: file-search tilde entry, menu syntax source filters, tab AI routing, ACP main-menu skill launch, shortcut recorder popup contracts.
- Prior feature-map evidence: features 012, 013, and 042.

Deliverable:

Return a dense, operator-grade atlas chapter outline for local agents to distill into `feature-map/features/045-launcher-trigger-edge-cases.md`.

Please include:

1. Exact current behavior for every relevant trigger/token: `~`, `~/`, `/`, `@`, `>`, `?`, `f:`, `files:`, `n:`, `c:`, `ai:`, `:`, `+`, `!`, `#`, `;target`, `target:`, and known negative examples.
2. A precedence model explaining which parser/classifier sees each input and why.
3. How special-entry handoffs differ from source filters, menu syntax, capture syntax, ordinary search, scriptlet keyword triggers, and ACP composer triggers.
4. Embedded and detached ACP behavior for `/` and `@`, including picker staging, deferred opening, focus, return origin, and open gaps.
5. File Search Mini behavior for `~` and `~/...`, including query normalization and what is not accepted.
6. Quick Terminal behavior for bare `>` and what terminal-like text does not trigger.
7. `?` actions-help behavior, including `has_actions()` false/no-op state.
8. Visual/focus/decoration states, especially stale decoration risks.
9. Automation receipts and exact verification recipes.
10. Unsafe claims to avoid and implementation/test plan for gaps.

Output boundary:

Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.

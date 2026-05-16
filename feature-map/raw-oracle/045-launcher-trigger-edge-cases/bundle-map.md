# 045 Launcher Trigger Edge Cases Bundle Map

Oracle session for the launcher trigger edge-case atlas.

## Session

- Feature id: `045-launcher-trigger-edge-cases`
- Oracle slug: `launcher-trigger-edge-cases`
- Status: completed
- Model: `gpt-5.5-pro`
- Browser label: `Latest`
- Thinking time: `extended`
- Completed at: `2026-05-15T15:30:08.543Z`
- Conversation URL: `https://chatgpt.com/c/6a073a7d-f99c-83e8-936b-1e50e87598c3`

## Token And Size Receipt

- Bundle path: `/Users/johnlindquist/.oracle/bundles/launcher-trigger-edge-cases.txt`
- Bundle size: `152344` bytes
- Oracle reported input tokens: `42432`
- Oracle reported output tokens: `5699`
- Oracle reported total tokens: `48131`
- Raw output log size: `51901` bytes
- Extracted answer size: `45798` bytes

## Bundle Contents

The bundle was narrowed to a 40.7k-token keyword-context pass around ScriptList trigger classification, first-token route dispatch, source filters, ACP picker staging, menu syntax, capture syntax, Quick Terminal, and actions help.

Included context:

- `AGENTS.md`
- `CLAUDE.md`
- `.agents/skills/main-menu-search-selection/SKILL.md`
- `.agents/skills/acp-context-composer/SKILL.md`
- `.agents/skills/file-search-portals/SKILL.md`
- `.agents/skills/quick-terminal-pty/SKILL.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`
- Relevant `lat.md/` launcher, menu syntax, ACP, file-search, terminal, actions, and verification pages found by `lat expand` and `lat search`
- ScriptList special-entry classifier and dispatch paths
- Root source-filter parser and filter decoration paths
- ACP launch and picker-staging helpers
- File Search Mini and Quick Terminal entry paths
- Menu syntax/capture parser tests and source-filter tests
- Prior chapters `012-root-source-filters`, `013-scriptlist-special-entry-triggers`, and `042-menu-syntax-power-capture`

## Prompt Intent

Oracle was asked to map:

- The exact boundary between ScriptList special entries and parser-owned text.
- The token matrix for `~`, `/`, `@`, `>`, `?`, source heads, menu syntax, capture aliases, and ordinary search.
- ACP embedded versus detached slash/mention picker behavior and proof gaps.
- Stale decoration and focus risks when moving from decorated source/menu input into a special route.
- Safe claims, unsafe claims, verification recipes, and follow-up proof work for launcher trigger edge cases.

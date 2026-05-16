# Agent Skills

This page documents the repo-local Codex skill and subagent topology that routes future agents to the correct Script Kit GPUI ownership area.

## Canonical Skill Set

The canonical Codex skills live under `.agents/skills/`, while `.agents/subagents/` contains paired read-only subagent briefs for broad or high-risk investigations.

The 26 ownership and support skills are `lat-md`, `agentic-testing`, `testing-quality-gates`, `dev-loop-observability`, `gpui-ui-foundation`, `theme-config-preferences`, `storybook-design`, `launcher-surface-contracts`, `window-resizing`, `main-menu-search-selection`, `keyboard-focus-routing`, `escape`, `actions-popups`, `builtin-filterable-surfaces`, `file-search-portals`, `prompt-runtime`, `sdk-script-execution`, `protocol-automation`, `mcp-context-resources`, `acp-chat-core`, `acp-context-composer`, `quick-terminal-pty`, `notes-window`, `dictation-media`, `platform-windowing-macos`, and `storage-cache-security`.

The canonical `agentic-testing` skill embeds both routing policy and the full runtime proof recipe, including session management, state-first receipts, exact target threading, screenshot escalation, ACP golden paths, and cleanup requirements.

`escape` is the cross-surface Escape-key UX owner. It routes work around close-vs-back decisions, direct launch vs launcher return, prompt cancellation, ACP streaming/popup guards, actions/confirm popup precedence, `simulateKey` parity, and automation reset state.

Legacy `.claude/skills/` files remain compatibility and migration sources. They are not the canonical Codex routing names for this repository.

## Routing Contract

Root `AGENTS.md` is a symlink to `CLAUDE.md`, so the skill routing table is maintained in `CLAUDE.md` and applies to both agent entrypoints.

Skill descriptions are the automatic routing surface for Codex. Subagent briefs are not automatic subagents; they are read-only prompts for explicit subagent use or for the primary agent to consult during complex exploration.

## Subagent Contract

Every canonical skill has a paired `.agents/subagents/<skill>-reader.md` subagent brief that maps files, contracts, invariants, risks, and proof paths without editing.

Subagents must stay read-only, cite source paths and lat pages, and return compact evidence for the implementation agent. They do not commit, push, run destructive commands, or claim completion.

## Related Pages

These pages provide the architecture and verification context that the skills route into.

- [[architecture]]
- [[surfaces]]
- [[automation]]
- [[verification]]
- [[acp-chat]]
- [[workspace]]

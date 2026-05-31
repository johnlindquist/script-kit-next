For a map of main UI surfaces to code implementation, see [GLOSSARY.md](file:///Users/johnlindquist/dev/script-kit-gpui/GLOSSARY.md).

# Before Starting Work

- Inspect the relevant source, tests, and repo-local skills before editing.
- Prefer current code and generated artifacts over stale notes or memory.
- Keep edits narrowly scoped and verify them with the smallest check that can fail for the changed behavior.
- Keep tool-facing root docs in place: `README.md`, `CLAUDE.md`, `AGENTS.md`, and `.impeccable.md`.

## Oracle / Packx Bundle Context

For Oracle review or `oracle-packx` work in this repository, include the repo process context in the bundle or prompt unless the user explicitly excludes it: `AGENTS.md`/`CLAUDE.md`, the owning `.agents/skills/<skill>/SKILL.md`, and relevant source, tests, generated contracts, and verification notes.

# Codex Repo Skills

Use the repo-local Codex skills in `.agents/skills/` as the primary task routing map. These are the canonical skill names for this repository.

Codex may load a skill automatically when the task matches the skill description. For investigation or UI proof work, explicitly name the skill in the prompt, for example `$script-kit-devtools` or `$agy-script-kit-devtools`.

For complex investigation, pair `script-kit-devtools` with the read-only subagent brief in `.agents/subagents/protocol-automation-reader.md` when you need protocol/state receipts before editing.

## Skill and Subagent Map

| Skill | Paired subagent | Primary ownership |
| --- | --- | --- |
| `script-kit-devtools` | `protocol-automation-reader` | Agent-facing DevTools primitives for inspecting, controlling, measuring, debugging, benchmarking, and proving real app UI behavior |
| `agy-script-kit-devtools` | — | Fast agy-driven investigations using existing script-kit-devtools primitives and compact receipts |

## Subagent Usage

Use `protocol-automation-reader` when the task spans multiple modules or needs evidence from stdin JSON protocol receipts before editing. The subagent must stay read-only and return relevant files/symbols, invariants, and the smallest verification command or devtools proof.

Do not wait for a subagent when the task is small and `$script-kit-devtools` or `$agy-script-kit-devtools` already gives enough context. Subagents are not automatic; spawn them only when the user explicitly asks for subagents/parallel delegation or when the task has broad, noisy exploration that benefits from a read-only sidecar.

## Skill Selection Defaults

- Unknown user-reported UX/UI bugs, screenshots, or flexible app investigation: `$agy-script-kit-devtools` for a fast pass, then `$script-kit-devtools` for deeper inspect/act proof.
- Stdin protocol, automation receipts, `getState`, `getElements`, or `waitFor` work: `$script-kit-devtools` with `protocol-automation-reader` when the investigation is broad.

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

---
name: quick-terminal-pty
description: >-
  Quick Terminal, term(), TermPrompt, PTY lifecycle, Alacritty rendering, shell startup, warm PTY pool, terminal theme, and apply-back.
---

# Quick Terminal PTY

This skill owns terminal prompt and PTY behavior for Script Kit GPUI and keeps changes grounded in current source and the narrowest useful proof.

## Use When

Use this skill for tasks involving:

- Quick Terminal, term(), TermPrompt, PTY lifecycle, Alacritty rendering, shell startup, warm PTY pool, terminal theme, and apply-back.
- Owned paths or concepts listed below.
- Bugs, tests, docs, or behavior changes where this domain is the primary owner.

Do not use this skill as the primary owner for ACP chat UI or generic prompt routing; load the adjacent owning skill instead.

## First Reads

Start with these sources before editing:

- `.agents/subagents/quick-terminal-pty-reader.md` for broad or high-risk investigation.

## Owned Paths and Concepts

Primary paths and concepts:

- `src/terminal/, src/term_prompt/, src/app_execute/utility_views.rs`
- terminal prompt and PTY behavior.
- The verification and documentation boundaries for this domain.

## Core Rules

- Identify the behavior owner before editing shared files. Path ownership is a hint; the user-visible behavior and documented contract decide the owner.
- Prefer current source and generated contracts over legacy notes or memory.
- Keep long recipes in the support skills that own them. Reference `$agentic-testing`, `$protocol-automation`, or `$testing-quality-gates` instead of duplicating proof procedures.

## Workflow

1. Review `AGENTS.md`, the owning skill, and current source context before editing.
2. Read the first sources above and trace the smallest real owner.
3. Check adjacent-skill boundaries before changing shared code.
4. Make the narrowest change that preserves the domain invariant.
5. Verify with the smallest proof that can fail if the behavior regresses.
6. Report changed files, proof tier, exact commands or receipts, adjacent skills consulted, and remaining risk.

## Proof Ladder

Use the smallest proof that can falsify the change. Do not escalate to screenshots, native input, broad test suites, or app launches when a lower tier proves the behavior.

2. Compile/static proof: for Rust, TypeScript, or config changes where runtime behavior is not needed. Run the narrowest compile or static check that covers the touched files.
3. Targeted test proof: for behavior encoded in unit, source-contract, SDK, or generated-artifact tests. Run the smallest named test target first.
4. State-first runtime proof: for UI, routing, protocol, focus, selection, popup, and surface behavior. Use the real runtime entry path with `getState`, `getElements`, `waitFor`, `batch`, or targeted receipts.
5. Visual proof: only when layout, rendering, visual state, screenshots, or image-library acceptance criteria are part of the change. Capture the real product surface and read the PNG before claiming success.
6. Native input / OS focus proof: only when the bug depends on real keyboard, mouse, AppKit focus, accessibility, screen capture permissions, PTY behavior, or other OS-level delivery.

Always clean up any process, session, or window the proof started. Report the tier used, exact commands or receipts, and why higher tiers were unnecessary.

Default check for this skill:

```bash
terminal/quick-terminal tests; runtime terminal state proof
```

## Adjacent Skills

Use adjacent skills when the work crosses boundaries:

- `$agentic-testing` for state-first runtime proof, screenshots, and cleanup.
- `$testing-quality-gates` for choosing narrow build/test gates.
- `$protocol-automation` when stdin JSON, receipts, target identity, `waitFor`, or `batch` are the behavior owner.

## Migration Notes

Legacy `.claude/skills/*` material can be mined for durable facts, but this 26-skill taxonomy is canonical for Codex routing. Do not rename this skill to a legacy skill name.

# AFK Audit Scope

Rules the audit loop reads at the start of every iteration. The loop is bounded by this file — if a story's fix would violate anything here, the story is deferred, logged, and skipped.

## Allowed actions

- `cargo check`, `cargo build`, `cargo test` (always with a 30s+ timeout)
- File edits via the Edit/Write tools within the project root
- `git add`, `git commit` on `main`
- `bun scripts/agentic/*` verification tools (index.ts, verify-shot.ts, window.ts, automation-window.ts)
- `source checks`, `source search`, `source lookup`, `source reference lookup`, `source context expansion`
- Starting `./dev.sh` in the background if the app is not already running, OR restarting the GPUI app via `bash scripts/agentic/session.sh start <name>` when the existing session's app process has died

## Forbidden actions

- `git push`, `git push --force`, any remote write
- `git reset --hard`, `git rebase`, `git revert` (user does any revert)
- `git commit --amend` on a published commit
- `git commit --no-verify` or any hook bypass
- Deleting any file (`rm -rf`, `rm`), `git rm`
- `cargo clean` (wastes build cache the loop depends on)
- Modifying `.git/` contents directly
- (no session-ownership restriction — see "Allowed actions" for the loop's session autonomy)
- Connecting to any external API or webhook that costs money or sends data off-machine

## Fix size policy


- If the fix addresses the story's acceptance criteria and the diff is focused on that, commit it regardless of size.
- If the fix drifts into "while I'm here" refactoring unrelated to the story, pause, record the finding as a `deferred` entry in the log, and move on without committing.
- Genuine cross-cutting repairs (extract a duplicated helper, fix a misnamed symbol across call sites) are fair game when the bug is the cross-cutting issue.

## Verification gate

- Every pass must either (a) find and fix a real bug, (b) extend the agentic-testing RPC surface, (c) add a contract test (rationed — see below), or (d) verify baseline behavior against a receipt that was not previously captured.
- Prefer state receipts (`getState`, `getElements`, `waitFor`, `batch`) over screenshots.
- No commit without a green proof in the log entry.

## Contract-test pin gate (hard lockout)


  - "already correct by construction"
  - "pins existing behavior"
  - "prevents silent drift" (without naming the specific drift vector in the same paragraph)
  - "locks down current shape"
  - "belt-and-suspenders"
  If a pin rationale reduces to these phrases, the invariant is not load-bearing enough to pin.

## Surface rotation

- This forces breadth over depth and prevents the Run 2 pattern of 5 consecutive passes on the same dispatcher.

## Attacker passes (must bite)

  1. **Rapid-fire** — same command 20× in 2 s
  2. **Interrupted** — start A, cancel mid-effect, start B
  3. **Interleaved** — ping-pong between ≥3 surfaces in random order
  4. **Boundary inputs** — empty string, 10k+ char string, zero-width unicode, emoji, RTL text, NUL bytes, malformed UTF-8
  5. **Concurrent** — fire ≥5 RPCs within a 100 ms window
  6. **Resurrection** — invoke against a view that was supposedly dismissed
  7. **Composition** — sequences that are valid individually but whose combination is unspecified (e.g. detach → hide → reattach → show)
- If anomaly found → file in `stories.md` as `[?]`; commit the anomaly report alone (no fix). A fix pass comes later as a separate normal pass.
- If no anomaly found AND minimum composition was met → log-only commit stating what was tried. Still counts as a pass.

## Backlog discipline (drain mode)

The loop is allowed to generate fresh stories when the seed list is exhausted — but generation must not outpace verification, or the backlog becomes a TODO graveyard.

- Tool-gap backlog still drains first regardless of drain-mode state.
- In drain mode, an attacker pass still runs on its 4-pass cadence (anomalies found are filed into the existing `[?]` bucket, not generated as new `[ ]` stories).

## Bug-yield floor

A run that ships no bug fixes is a run that either has nothing to find or a loop too timid to find it. The floor catches the second case.


## Surface breadth floor

Rotation alone (no surface from last 10) guarantees no *back-to-back* repetition but allows the same 3 surfaces to cycle for the whole run. The floor closes that.


## Tool-gap queue (auto-promoted)

Every tool-gap slug mentioned in log prose MUST also exist as an actionable `[ ]` item in `stories.md`. Without this, gaps written in English evaporate across passes — a problem Run 2 exposed (12 gap slugs logged but never picked up).

- At the **start of every tick**, before picking a story, run `bash audits/afk/promote-tool-gaps.sh`. The script greps `log.md` for `tool-[a-z0-9-]+` slugs, diffs against `stories.md`, and appends any missing ones as `[ ]` items under a dedicated section. Committing the promotion (if it produced edits) is free — counts as pass substrate, not a pass.
- A tool-* slug appearing only in prose is a lint error caught by this script.

## Tooling gaps are in scope


- Log the gap in `log.md` with `[!]` on the story in `stories.md`.
- Back the improvement with a verification story so the new tool is exercised end-to-end.

A pass whose sole output is a tool improvement is valid, as long as the gap was blocking a real user-story verification.

## Story outcome markers


- `[ ]` — not yet attempted
- `[x]` — pass verified (with or without a fix commit)
- `[!]` — could not verify with current tools; gap recorded
- `[-]` — skipped because proving it would require a forbidden action
- `[?]` — attacker-mode pass found an anomaly; follow-up investigation pending

## Budget and kill switch


## Bias toward extending the RPC surface


- A structural-test pass is justified when the invariant is genuinely load-bearing AND the Refactor threat clause can name a specific plausible contributor action that would break it (see "Contract-test pin gate" above). It is NOT justified as a consolation prize for a story that got blocked on tooling.

## Commit policy

- **One commit per pass**. Never amend a prior pass's commit — each pass gets its own sha so individual passes can be reverted without disturbing others.
  Using `Pin` when the work was `Verify` (or vice versa) is a quality violation — the commit subject is load-bearing signal for quota tracking.

## Required log-entry fields


```
## Pass N — <ISO timestamp> — `<story-slug>` (<outcome marker>)

```

### Template (fix commits)

```




```

### Template (pass-only commits — no code fix, only log/stories update)

```

<One paragraph describing the user story, what was checked, and what constituted acceptance (state receipt fields, timing, visible behavior).>



```

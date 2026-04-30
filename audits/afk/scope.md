# AFK Audit Scope

Rules the audit loop reads at the start of every iteration. The loop is bounded by this file — if a story's fix would violate anything here, the story is deferred, logged, and skipped.

## Allowed actions

- `cargo check`, `cargo build`, `cargo test` (always with a 30s+ timeout)
- File edits via the Edit/Write tools within the project root
- `git add`, `git commit` on `main`
- `bash scripts/agentic/session.sh` commands (start, status, send, rpc, stop) — the loop has full autonomy over session lifecycle: it may start a new session if one is missing or dead, stop and restart a session that is misbehaving, or spin up additional named sessions to isolate a test. The only hard rule is that the loop must leave the final state usable (i.e., if the loop starts a session for a pass, either keep it running or log the shutdown)
- `bun scripts/agentic/*` verification tools (index.ts, verify-shot.ts, window.ts, automation-window.ts)
- `lat check`, `lat search`, `lat locate`, `lat refs`, `lat expand`
- Starting `./dev.sh` in the background if the app is not already running, OR restarting the GPUI app via `bash scripts/agentic/session.sh start <name>` when the existing session's app process has died

## Forbidden actions

- `git push`, `git push --force`, any remote write
- `git reset --hard`, `git rebase`, `git revert` (user does any revert)
- `git commit --amend` on a published commit
- `git commit --no-verify` or any hook bypass
- Deleting any file (`rm -rf`, `rm`), `git rm`
- `cargo clean` (wastes build cache the loop depends on)
- Writing or reading outside project root: `~/.config`, `~/.ssh`, `/etc/`, etc.
- Modifying `.git/` contents directly
- (no session-ownership restriction — see "Allowed actions" for the loop's session autonomy)
- Connecting to any external API or webhook that costs money or sends data off-machine

## Fix size policy

No hard file/LOC cap. User trusts judgment. Heuristic:

- If the fix addresses the story's acceptance criteria and the diff is focused on that, commit it regardless of size.
- If the fix drifts into "while I'm here" refactoring unrelated to the story, pause, record the finding as a `deferred` entry in the log, and move on without committing.
- Genuine cross-cutting repairs (extract a duplicated helper, fix a misnamed symbol across call sites) are fair game when the bug is the cross-cutting issue.

## Verification gate

- Every pass must either (a) find and fix a real bug, (b) extend the agentic-testing RPC surface, (c) add a contract test (rationed — see below), or (d) verify baseline behavior against a receipt that was not previously captured.
- Prefer state receipts (`getState`, `getElements`, `waitFor`, `batch`) over screenshots.
- No commit without a green proof in the log entry.
- Every log entry must include a `- **Falsifier**: <concrete receipt state that would have proven this pass wrong>` line. If the agent cannot name a falsifier, the pass did not actually measure anything and must be retried.

## Contract-test pin gate (hard lockout)

Structural pins (`Prompt: Pin …`) are the single highest drift risk. They feel like progress, are always green on correct code, and crowd out real bug discovery. This gate is enforced as a **hard precondition**, not a soft quota.

- **Before committing any pin**, run: `git log -20 --format="%s" | grep -c '^Prompt: Pin '`. If the result is **≥7** (i.e., pins make up more than a third of the last 20 commits), the pin is FORBIDDEN this pass. No exception. Upgrade to a tool extension, attacker probe, or genuine bug hunt instead.
- Pin commits MUST include a `Refactor threat:` line in the body naming **one specific plausible contributor action** that would break the invariant. Examples:
  - ✅ `Refactor threat: A contributor deduping the three stdin dispatchers by extracting a shared helper could drop the post-match re-key call in one arm.`
  - ❌ `Refactor threat: A refactor could break this.` (no specific action named)
  - ❌ `Refactor threat: Silent drift.` (passive, not a concrete action)
- **Banned phrases in pin commit bodies and log entries** (signal drift, auto-reject):
  - "already correct by construction"
  - "pins existing behavior"
  - "prevents silent drift" (without naming the specific drift vector in the same paragraph)
  - "locks down current shape"
  - "belt-and-suspenders"
  If a pin rationale reduces to these phrases, the invariant is not load-bearing enough to pin.
- A pin whose subject line does not specify the refactor it defends against (format: `Prompt: Pin <invariant> against <named refactor>`) is rejected.

## Surface rotation

- Track the last 10 passes' primary surface in each pass's log entry with a `- **Surface**: <surface-name>` field (e.g. `clipboardHistory`, `acpChatDetached`, `actionsDialog`, `stdinListener`).
- Before picking a story, grep the last 10 pass entries for the `**Surface**:` field. The next pass MUST NOT reuse any of those surfaces unless the surface queue is exhausted (fewer than 10 unique surfaces remain across all `[ ]` stories).
- This forces breadth over depth and prevents the Run 2 pattern of 5 consecutive passes on the same dispatcher.

## Attacker passes (must bite)

- Every 4th pass (passes #4, #8, #12, …) runs in **attacker mode**: no pre-specified acceptance criteria.
- **Minimum composition**: 20+ actions AND **≥3 distinct adversarial categories** from the recipe menu below. A pass that uses only one category (e.g. 20 rapid-fire of the same command) does not count as attacker mode — re-run.
- **Recipe menu** (pick ≥3):
  1. **Rapid-fire** — same command 20× in 2 s
  2. **Interrupted** — start A, cancel mid-effect, start B
  3. **Interleaved** — ping-pong between ≥3 surfaces in random order
  4. **Boundary inputs** — empty string, 10k+ char string, zero-width unicode, emoji, RTL text, NUL bytes, malformed UTF-8
  5. **Concurrent** — fire ≥5 RPCs within a 100 ms window
  6. **Resurrection** — invoke against a view that was supposedly dismissed
  7. **Composition** — sequences that are valid individually but whose combination is unspecified (e.g. detach → hide → reattach → show)
- **Near-anomaly filing**: file a `[?]` story not only when state receipts mismatch but also when any of the following occur: unexpected tracing-span ordering, timing outside documented bounds, log spam proportional to input size, state that mismatches its own documentation.
- **Attacker escalation**: if 3 consecutive attacker passes yield zero `[?]` filings, the loop flags "attacker mode not biting" in the next log entry and the next attacker pass MUST escalate: ≥30 actions, ≥4 surfaces, ≥4 adversarial categories, and at least one payload outside documented input bounds.
- If anomaly found → file in `stories.md` as `[?]`; commit the anomaly report alone (no fix). A fix pass comes later as a separate normal pass.
- If no anomaly found AND minimum composition was met → log-only commit stating what was tried. Still counts as a pass.

## Backlog discipline (drain mode)

The loop is allowed to generate fresh stories when the seed list is exhausted — but generation must not outpace verification, or the backlog becomes a TODO graveyard.

- **Drain mode trigger**: when open story count (`[ ]` + `[!]`) exceeds **8**, the loop enters drain mode. No new stories are generated. Each pass must pick from existing `[ ]`, `[!]`, or `[?]` items until open count falls below **5**.
- Tool-gap backlog still drains first regardless of drain-mode state.
- In drain mode, an attacker pass still runs on its 4-pass cadence (anomalies found are filed into the existing `[?]` bucket, not generated as new `[ ]` stories).

## Bug-yield floor

A run that ships no bug fixes is a run that either has nothing to find or a loop too timid to find it. The floor catches the second case.

- In any 5-commit sliding window, at least **one** commit must be a real fix (subject line starts with `Prompt: Fix `, `Prompt: Add `, or `Prompt: Extend `). If the window contains zero such commits, the next pass type is forced: either attacker-mode (if not already scheduled) or tool-extension. Normal passes and pin passes are forbidden until the floor is met.
- Verify: `git log -5 --format="%s" | grep -cE '^Prompt: (Fix|Add|Extend) '`.

## Surface breadth floor

Rotation alone (no surface from last 10) guarantees no *back-to-back* repetition but allows the same 3 surfaces to cycle for the whole run. The floor closes that.

- In any 10-pass window, **≥6 distinct surfaces** must appear in the `- **Surface**:` log field. If the window currently has <6, the next pass must pick a surface not seen in the last 10.
- Verify: `grep -E '^- \*\*Surface\*\*:' audits/afk/log.md | head -10 | sort -u | wc -l`.

## Tool-gap queue (auto-promoted)

Every tool-gap slug mentioned in log prose MUST also exist as an actionable `[ ]` item in `stories.md`. Without this, gaps written in English evaporate across passes — a problem Run 2 exposed (12 gap slugs logged but never picked up).

- At the **start of every tick**, before picking a story, run `bash audits/afk/promote-tool-gaps.sh`. The script greps `log.md` for `tool-[a-z0-9-]+` slugs, diffs against `stories.md`, and appends any missing ones as `[ ]` items under a dedicated section. Committing the promotion (if it produced edits) is free — counts as pass substrate, not a pass.
- A tool-* slug appearing only in prose is a lint error caught by this script.

## Tooling gaps are in scope

When a story cannot be verified with the current agentic-testing tools, that is itself a finding. The loop may:

- Log the gap in `log.md` with `[!]` on the story in `stories.md`.
- Propose a tool improvement (new flag, new protocol command, additional receipt field) — commit the improvement on its own pass with a commit message that starts `Prompt: Extend agentic-testing — ...`.
- Back the improvement with a verification story so the new tool is exercised end-to-end.

A pass whose sole output is a tool improvement is valid, as long as the gap was blocking a real user-story verification.

## Story outcome markers

In `stories.md`:

- `[ ]` — not yet attempted
- `[x]` — pass verified (with or without a fix commit)
- `[!]` — could not verify with current tools; gap recorded
- `[-]` — skipped because proving it would require a forbidden action
- `[?]` — attacker-mode pass found an anomaly; follow-up investigation pending

## Budget and kill switch

- **Time budget**: stop scheduling new iterations after the budget-minutes window recorded alongside the most recent "Loop started" timestamp in `log.md` (first run was 120 min; subsequent runs may set their own window).
- **Scheduling cutoff**: do NOT start a new pass within 20 minutes of the deadline — the previous pass can finish and commit, but the next wake-up is skipped. Run 2 overshot its deadline by 1h 51m by starting a pass too close to the edge; this rule prevents that.
- **Kill switch**: presence of `.audit-pause` at project root aborts the current iteration *and* prevents scheduling the next wake-up. Remove the file to re-arm the loop.

## Bias toward extending the RPC surface

When a story hits a tool dead end (RPC verb missing, selector unreachable, receipt field absent), the default response is **extend the tool**, not **pin the current behavior with a structural test**. Rationale from Run 2–3: pins on already-correct code don't find bugs and don't unlock the next story. Tool extensions compound — each new RPC verb unlocks 5+ future stories.

- If a story's verification requires a tool that doesn't exist, the pass SHOULD attempt to build it (commit pattern: `Prompt: Extend agentic-testing — add <verb> RPC for <surface>`).
- A structural-test pass is justified when the invariant is genuinely load-bearing AND the Refactor threat clause can name a specific plausible contributor action that would break it (see "Contract-test pin gate" above). It is NOT justified as a consolation prize for a story that got blocked on tooling.
- **Consolation-pin tell**: if the pass narrative reads "story blocked on missing tool, so I pinned <adjacent invariant> instead", the pass is invalid. Ship the tool extension or file `[!]` and move on.

## Commit policy

- **One commit per pass**. Never amend a prior pass's commit — each pass gets its own sha so individual passes can be reverted without disturbing others.
- **Prompt-style messages**. Every commit message is written as a prompt an agent could follow to recreate the work. Start the subject with `Prompt: <imperative>` (≤72 chars). The body provides enough detail that another agent, given only this commit and the repo state before it, could reproduce the change.
- **Subject verb discipline**: use the verb that matches the actual work.
  - `Prompt: Fix …` — genuine user-visible behavior change
  - `Prompt: Add …` — new RPC verb, receipt field, or user-facing capability
  - `Prompt: Extend …` — expand existing tool to cover a new case
  - `Prompt: Pin …` — structural contract test (subject MUST name the defended refactor)
  - `Prompt: Verify …` — pass-only, no code change, fresh baseline receipt
  - `Prompt: Probe …` — attacker pass, no acceptance criteria
  - `Prompt: Reproduce …` — attacker anomaly with a named trigger sequence
  Using `Pin` when the work was `Verify` (or vice versa) is a quality violation — the commit subject is load-bearing signal for quota tracking.

## Required log-entry fields

Every pass entry in `log.md` must contain these fields in this order. Missing any field = pass is invalid and must be re-done:

```
## Pass N — <ISO timestamp> — `<story-slug>` (<outcome marker>)

- **Surface**: <surface name — drives rotation/breadth accounting>
- **Story**: <exact story text, or "attacker-mode pass #N" for attacker passes>
- **Outcome**: <one-line outcome — fix-shipped / pass / tool-extension / anomaly-filed / log-only>
- **Commit**: <sha> (after commit lands)
- **Proof**: <key state receipt fields, with values>
- **Falsifier**: <one concrete receipt state that would have proven this pass wrong>
- **Refactor threat** (pin passes only): <specific contributor action this pin defends against>
- **Adversarial categories** (attacker passes only): <≥3 from the recipe menu>
- **Notes**: <short free text; optional>
```

### Template (fix commits)

```
Prompt: <imperative description of what to do, ≤72 chars>

<One paragraph of context: which user story is being exercised, what symptom was observed, what the fix changes, and how to reproduce the verification.>

User story: <exact story text from stories.md>

Pass: #<N> of AFK audit run <start-timestamp>
Files: <count> changed, +<insertions>/-<deletions>
Proof: <key state receipt fields, or screenshot path>

Generated by AFK audit loop. Revert with: git revert <sha>
```

### Template (pass-only commits — no code fix, only log/stories update)

```
Prompt: <imperative summary of the verification another agent would run>

<One paragraph describing the user story, what was checked, and what constituted acceptance (state receipt fields, timing, visible behavior).>

User story: <exact story text from stories.md>

Pass: #<N> of AFK audit run <start-timestamp>
Proof: <key state receipt fields>

Generated by AFK audit loop. Revert with: git revert <sha>
```

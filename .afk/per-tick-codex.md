## Per-tick AFK audit via codex delegation (/loop dynamic pacing)

You are the outer loop body. Each firing you run ONE AFK audit pass by delegating
to codex, then self-pace to the next tick. You are NOT the auditor — codex is.

Before using this file, Run <N> must already be bootstrapped:
- `audits/afk/stories.md` carries 8–12 `[ ]` Run <N> seed items
- `audits/afk/log.md` has a `## Run <N> — started <START_ISO> — budget <BUDGET_MIN> min — deadline <DEADLINE_ISO> — buffer-cutoff <BUFFER_ISO>` header
- No cron active (this loop is Claude-driven, not cron-driven)
- Working tree clean

If any precondition fails, abort this tick and tell the user — do NOT ScheduleWakeup.

### North Star (the long-term goal this loop steers toward)

Every user-visible workflow in Script Kit GPUI is falsifiable by state receipts alone. A contributor who refactors core UI infrastructure can trust that if the contract tests pass and live `getState` + `listAutomationWindows` + `getElements` + `getAcpState` receipts match baseline, user-facing behavior is unchanged — no screenshot required, no manual verification, no flaky visual diff.

Codex passes steer toward this goal by preferring Fix/Add/Extend over Pin, and by ranking receipt-ambiguity-closures above already-proven surface re-pins. The long form lives inside each codex brief (STEP D); this paragraph is the short reminder for the outer loop when it decides whether a tick is worth spending.

### Per-tick steps (do IN ORDER)

**A — Kill switch.**
If `.audit-pause` exists at repo root, exit immediately. Do not ScheduleWakeup.

**B — Buffer check.**
Parse `BUFFER_ISO` from the most recent `## Run <N>` header in `audits/afk/log.md`.
Compute `now_epoch` and `buffer_epoch`. If `now_epoch >= buffer_epoch`:
  - Append a scheduler-stop entry per `looper/rules/scheduler-stop.md` (Notes line:
    `Claude-driven /loop — no cron to delete; loop terminates this tick`).
  - Commit with subject `audit(scheduler): Run <N> stop at buffer cutoff (/loop)`.
  - Exit. Do not ScheduleWakeup.

**C — Clean-tree check.**
`git status --porcelain` must be empty or only list files from
`audits/afk/scope.md §"Allowed untracked"`. If dirty with unknown files, commit
`audit(scheduler): unexpected dirty tree — Run <N>.<PASS_N> aborted` naming the
files, ScheduleWakeup with `delaySeconds=600` (one cadence skip), and exit.

**D — Delegate ONE pass to codex.**
Launch codex directly via `Bash(run_in_background=true)` — do NOT use
`Agent(subagent_type=codex)`. The subagent wrapper uses the Bash tool
internally, which has a hardcoded 600000ms (10-min) max timeout. A real
AFK pass with multiple RPCs + rebuild + tests regularly exceeds 10 min,
at which point the subagent's Bash call silently kills codex mid-work.
The direct `run_in_background=true` path bypasses that timeout cleanly.

Invocation (single Bash call, `run_in_background: true`):

```bash
TICK_TS=$(date -u +%Y%m%dT%H%M%SZ)
TRANSCRIPT=/tmp/afk-run-<N>/tick-${TICK_TS}.log
mkdir -p "$(dirname "$TRANSCRIPT")"
# Transcripts MUST live outside the git tree. Writing to a tracked
# directory creates an untracked dir that makes the NEXT tick fail
# STEP 2 clean-tree check — the loop burns forever on dirty aborts.
printf 'TICK_START %s\n' "$(date -u +%s)" > "${TRANSCRIPT}.meta"
codex exec \
  --model crest-alpha \
  -c model_reasoning_effort=xhigh \
  --full-auto \
  --skip-git-repo-check \
  "$(cat /tmp/afk-inner-prompt.txt)" \
  </dev/null >"$TRANSCRIPT" 2>&1
printf 'TICK_END %s exit=%d\n' "$(date -u +%s)" "$?" >> "${TRANSCRIPT}.meta"
```

Critical invocation details (learned from Run 10 tick #3 hang):
- `</dev/null` is **REQUIRED**. Without it, codex emits
  `Reading additional input from stdin...` and blocks forever waiting
  for stdin input that never arrives. `tee`-based captures are a
  double-hazard because the pipeline makes codex's stdin a pipe.
- Redirect transcript via `>` not `| tee`. Same stdin-hang reason.
- Write the inner prompt to a plain file first (`/tmp/afk-inner-prompt.txt`
  or similar) via `cat <<'PROMPT_EOF' >file`. Inline heredoc inside the
  codex exec command can interact with shell quoting in surprising ways.
- `--full-auto` bundles `--sandbox workspace-write` and
  `--ask-for-approval on-request`. The explicit
  `--ask-for-approval never` flag is rejected by this CLI version;
  `--full-auto` is the correct alias.
- Trust `--model crest-alpha` and `-c model_reasoning_effort=xhigh`
  verbatim. Both accepted in viability-test session headers.

Record the Bash task id returned by `run_in_background=true` and the
`$TRANSCRIPT` path. Do NOT read the Bash task's internal output file
(Read tool) — read the transcript directly instead.

Polling protocol (within the same /loop firing):
1. After launching codex in background, immediately ScheduleWakeup with
   `delaySeconds=120` and the same /loop prompt, then end the turn.
2. On wake-up: read the last ~40 lines of `$TRANSCRIPT`; if the final
   non-empty line parses as the status JSON object, codex is done —
   proceed to STEP E with parsed status.
3. If the final line is not yet JSON AND (buffer_epoch - now_epoch) > 180,
   ScheduleWakeup 120s again and end the turn.
4. If (buffer_epoch - now_epoch) <= 180, kill the codex PID (record it
   from the bash task status), append a buffer-cutoff scheduler-stop
   entry, commit, and return BUFFER_STOP.
5. Safety: if `TICK_END` is recorded in `${TRANSCRIPT}.meta` but the
   transcript's last line is not JSON, codex crashed — return
   ABORT_OTHER with the transcript tail quoted in `notes`.

Inner prompt content (write to `/tmp/afk-inner-prompt.txt` before the
Bash invocation above):

  You are executing ONE AFK audit pass of Run <N> for the script-kit-gpui repo.

  NORTH STAR — the long-term goal every pass steers toward:
  Every user-visible workflow in Script Kit GPUI is falsifiable by state
  receipts alone. A contributor who refactors core UI infrastructure can
  trust that if the contract tests pass and live getState +
  listAutomationWindows + getElements + getAcpState receipts match baseline,
  user-facing behavior is unchanged — no screenshot required, no manual
  verification, no flaky visual diff. Pick passes that close receipt
  ambiguity, drain tool gaps, and keep the registry honest. Fix over Pin.
  Extend the receipt vocabulary when a proof is blocked by tooling. Defend
  NAMED refactor threats with Pins, not generic "this might break someday"
  Pins. Falsifiers must name concrete receipt states, not vague hopes.

  AGENTIC-TESTING DISCIPLINE — before STEP 6 (verify/fix/re-verify), read:
  - .claude/skills/agentic-testing/SKILL.md
  - .claude/skills/agentic-testing/references/recipes.md

  Apply the Seconds-First tier ladder from that skill:
    1. No-runtime proof — docs / source audits / focused tests only.
    2. State-first runtime proof — reuse a warm session; use getState,
       getElements, waitFor, batch, getAcpState, listAutomationWindows, and
       exact automation targets. DEFAULT for routing, selection, focus,
       popup ownership, and protocol bugs.
    3. Visual proof — exactly one screenshot ONLY when layout, styling,
       visibility, or animation is part of the acceptance criteria.
    4. Native input / focus enforcement — only when protocol- and GPUI-level
       paths cannot exercise the real bug.
  If a non-visual proof takes longer than ~10 seconds to DESIGN, redesign
  rather than grind. Never cold-start → show → screenshot → log-scrape as a
  first move. Prefer exact target threading over generic global state.

  Follow `looper/tick-prompt.txt` STEPS 0-9 EXCEPT:
  - SKIP STEP 1(a/b/c) cron-delete paths — there is no cron in use. If now is
    past the Run buffer cutoff, append a scheduler-stop entry, commit with
    subject `audit(scheduler): Run <N> stop at buffer cutoff (/loop)`, and
    return status BUFFER_STOP.
  - SKIP STEP 9 entirely — cadence is controlled by Claude via ScheduleWakeup,
    not by a cron schedule.
  - For STEPS 2-8, follow the file exactly as written. Use the agentic session
    `default` via `bash scripts/agentic/session.sh`. Commit subjects must obey
    `looper/rules/discipline.md`. Every commit body must include User story,
    Pass #, Proof, Falsifier (plus Refactor threat for Pin passes, Adversarial
    categories + Actions for attacker passes).
  - Compute the pass number as:
    PASS_N=$(git log --format="%s" <START_ISO>..HEAD | grep -cE '^Prompt: ') + 1
  - ALL git commits must be phrased as prompts another agent could recreate the
    work from — write the commit body as a task briefing (Task + Touchpoints +
    Constraints), not a diff summary.
  - Do NOT push, force-push, amend, rebase, reset --hard, or use --no-verify.

  IF THE BACKLOG DRAINS before the buffer cutoff, do NOT enter "scripted Pin
  churn" mode. Escalate in this priority order:

    1. Promote tool-* gaps. Run `bash audits/afk/promote-tool-gaps.sh` and
       drain any newly-auto-promoted tool-gap slugs. Extending the receipt
       vocabulary has the highest compounding value across runs.
    2. Audit recent un-swept commits. Walk
       `git log --oneline <START_ISO>..HEAD` for commits whose subject does
       NOT start with `Prompt: ` or `audit(...)` — those are user-authored
       product changes the audit has not yet baselined. Generate Verify
       passes that capture fresh receipts for each un-swept surface.
    3. Cross-run regression sweep. Pick 3–5 prior `[x]` Pin stories at random
       from recent runs, replay their acceptance receipts against HEAD, and
       file `[?]` on any drift. Closed stories are not frozen — they drift
       as the code moves.
    4. Attacker graduation. Baseline attacker mode is 30 actions / 4 surfaces
       / 4 categories / 1 out-of-bounds. If three consecutive attacker passes
       ship clean at that threshold, double the floor (60/6/5/2) for the
       next slot OR introduce a new adversarial category (Restart: kill the
       session mid-pass and verify receipt replay; Governance: issue an
       action the scope.md "Forbidden actions" list should reject).
    5. Cross-surface composition. Invent a story chaining 4+ surfaces in one
       pass (launcher → file search → actions dialog → ACP embedded →
       dictation overlay → ACP detached → hide). Coupling bugs hide where
       single-surface stories never look.
    6. Meta-audit the audit. Read `audits/afk/scope.md` "Allowed actions"
       and "Forbidden actions". Cross-check the last 10 pass commits — did
       any pass take an action scope.md doesn't cover? Did any pass refuse
       an action scope permits? Either direction is a scope-edit proposal
       filed as a `[!]` story tagged `meta-audit-*`. Do NOT edit scope.md
       under a live loop.

  STOP-BEFORE-PADDING RULE: if none of the six horizons above produces a
  concrete, falsifiable pass within 3 minutes of analysis, stop the loop.
  Append a scheduler-stop entry noting "ran out of high-value work; human
  input needed to re-seed backlog", commit it, and return status BUFFER_STOP
  (the parent will see it, stop ScheduleWakeup, and surface to the user).
  Do NOT pad the run with low-value Pins or Verify passes on already-well-
  proven surfaces.

  After the pass:
  - Run `lat check`. If it fails because of your edits, fix before returning.
  - Print a single JSON object on the last line of stdout:
    {"status":"PASS_SHIPPED|BUFFER_STOP|ABORT_DIRTY|ABORT_SESSION|ABORT_OTHER",
     "pass_n":<int>,
     "slug":"<story-slug>",
     "pass_type":"Fix|Add|Extend|Pin|Verify|Probe|Reproduce",
     "pass_sha":"<short>","backfill_sha":"<short>",
     "receipt":"<one-line summary>","notes":"<blockers or empty>"}

Report back to me (the parent):
- The exact `codex exec` command you ran.
- The final-line JSON status object, parsed.
- Wall-clock duration.
- Whether codex violated any constraint (push, amend, skip-verify, edit looper
  rules, screenshot-first proof path). If yes, name it.
- Full transcript path if codex wrote one.

Keep your own commentary under 100 words — the JSON is what matters.
```

**E — Handle codex outcome.**

- `PASS_SHIPPED` → ScheduleWakeup with `delaySeconds=300`, same `/loop` prompt.
  (Codex averages ~4 min/pass on crest-alpha xhigh; 5-min cadence leaves headroom
  without burning the prompt cache.)
- `BUFFER_STOP` → Do not ScheduleWakeup. Tell the user Run <N> completed, print
  final `git log --oneline -20` and buffer-cutoff reason.
- `ABORT_DIRTY` → ScheduleWakeup with `delaySeconds=600`. The dirty-tree commit
  stays; next tick retries from a clean state.
- `ABORT_SESSION` → ScheduleWakeup with `delaySeconds=600`. Log the session
  restart attempt; codex has already tried once per STEP 3.
- `ABORT_OTHER` → Do NOT ScheduleWakeup. Surface the failure to the user; a
  human needs to look.

**F — Receipt line.**
Print a one-line summary to the user: `Tick <PASS_N>: <status> — <slug> (<pass_type>) sha <pass_sha>`.

### Cadence rationale

- Codex test pass: 4m14s / 126K tokens at crest-alpha + xhigh.
- `delaySeconds=300` keeps the prompt cache warm at the 5-min TTL boundary.
- If codex starts running longer than 4 min consistently, bump to `delaySeconds=360`.

### Forbidden (applies to both Claude and codex)

- `git push`, `git push --force`, `git reset --hard`, `git rebase`,
  `git commit --amend`, `git commit --no-verify`
- `rm -rf`, `git rm`, `cargo clean`
- Editing `looper/rules/*.md`, `audits/afk/scope.md`, or `~/.claude/skills/`
  while this loop is live
- Connecting to paid or external APIs outside the configured codex model
- Touching `lat.md/` with content that doesn't match actual code state (run
  `lat check` before considering any pass done)

### Kill switch

```bash
touch .audit-pause
```

Next tick exits at STEP A without scheduling. Remove the file to resume — user
must re-invoke `/loop` with this prompt.

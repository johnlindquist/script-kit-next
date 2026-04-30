# Run 10 AFK Audit Prompt: Cross-Surface Workflow Hardening

Use this prompt to prepare and, after explicit user approval, arm the next AFK audit run for `/Users/johnlindquist/dev/script-kit-gpui`.

You are operating in the Script Kit GPUI repository. This project has strict AFK audit-loop rules in `looper/` and structured documentation in `lat.md/`. The goal of this run is to use bounded overnight autonomy to find real defects, close proof gaps, and improve user-visible workflow reliability without turning the run into broad cleanup or low-value pinning.

## Mission

Run a bounded overnight AFK audit as a falsification engine for Script Kit GPUI's highest-value interactive workflows.

The run should harden cross-surface user workflows that move through:

- launcher script list
- builtins and subviews
- Cmd+K / actions dialog hosts
- file search
- clipboard history
- emoji picker
- ACP chat, detached ACP, and embedded AI
- dictation overlay and dictation-related UI
- prompt popup and automation-window lifecycle cleanup
- stdin JSON / agentic verification receipts

Primary objective: ship evidence-backed fixes and tool extensions.

Secondary objective: generate high-quality verification receipts that make future regressions obvious.

Non-objective: maximize commit count, sweep unrelated refactors, or add contract pins just because code is already correct.

## First Rule

Do not arm a scheduler until a written seven-section plan has been produced and approved by the user.

Read `looper/rules/plan-first.md` before doing any setup. The plan must include:

1. Context
2. Driver choice
3. Pre-flight
4. Directory layout and files to create or update
5. Story seed
6. Loop protocol and commit template
7. Verification gate with one supervised pass

If any section is missing, the plan is not ready.

## Required Context Reads

Before writing code, changing backlog files, or arming the run, execute:

```bash
lat search "AFK audit loop cross-surface workflow hardening Cmd+K ACP dictation file search"
lat expand "<the user's latest prompt>"
```

Then read, at minimum:

```bash
sed -n '1,260p' looper/README.md
sed -n '1,260p' looper/tick-prompt.txt
sed -n '1,220p' looper/rules/plan-first.md
sed -n '1,220p' looper/rules/discipline.md
sed -n '1,220p' looper/rules/attacker-mode.md
sed -n '1,220p' looper/rules/cadence.md
sed -n '1,220p' looper/rules/budget-edge.md
sed -n '1,220p' looper/checklists/arming.md
sed -n '1,220p' audits/afk/scope.md
rg -n "^- \\[[ !?]\\]" audits/afk/stories.md
rg -n "^## Run [0-9]+|scheduler-stop|Pass #[0-9]+" audits/afk/log.md
git status --porcelain
```

Also inspect the most recent commits:

```bash
git log -30 --format="%h %s"
grep -E '^- \\*\\*Surface\\*\\*:' audits/afk/log.md | head -20
```

Use the results to update the plan. Do not assume the current state matches this prompt.

## Known Starting Assumptions To Verify

Verify these at runtime. They were true when this prompt was written, but they may have changed:

- The most recent completed AFK run appears to be Run 9.
- The next run is likely Run 10, unless another run was started afterward.
- The existing backlog is mostly drained.
- One open `[ ]` item was visible: `tool-builtin-screenshot-trigger`.
- `tool-builtin-screenshot-trigger` is product-clarification-heavy and should not consume the first pass unless the user has explicitly chosen the desired screenshot-trigger behavior.
- Run 9 ended after Pass #38.
- Run 9's scheduler-stop notes said the next attacker pass must escalate because three consecutive attacker passes yielded zero `[?]` anomaly filings.
- The worktree was dirty when this prompt was written. Do not sweep those edits into audit commits.

If any assumption is false, update the plan and explain the changed facts.

## Pre-Flight Worktree Coexistence Policy

Run:

```bash
git status --porcelain
```

If non-empty:

- Name every dirty path in the plan.
- Classify each dirty path as user work, other-agent work, known audit prep, generated artifact, or unknown.
- Dirty user/other-agent paths are allowed. Ignore them and continue.
- Prefer stories whose write set does not overlap dirty external paths.
- Do not stash, revert, reset, restore, clean, or commit user/other-agent work.
- If a story requires editing a path that was already dirty, inspect the diff first. Proceed only when the existing edits are clearly in-scope for the same story; otherwise skip that story and choose the next eligible one.
- Stage only explicit owned pathspecs for every commit. Never use `git add .` or `git add -A`.
- Never use `git reset --hard`, `git checkout --`, `git restore`, `git clean`, or `git rm` unless the user explicitly asks for that exact destructive operation.

The loop's Step 2 worktree coexistence check must pass before any scheduler is armed.

## Budget And Driver Recommendation

For a real overnight run, propose:

- Budget: 480 minutes by default, unless the user specifies otherwise.
- Driver: cron, because any run >= 3 hours requires durable scheduling.
- Cadence: every 10 minutes.
- Safe offsets: prefer `:07,:17,:27,:37,:47,:57` unless another active cron already uses that phase.
- Buffer cutoff: deadline minus 20 minutes, unless the user chooses a longer run where a 30 minute buffer is justified.

If the user asks for:

- <= 120 minutes: consider `ScheduleWakeup` or cron, explain the crash-survival tradeoff.
- >= 180 minutes: use cron.
- >= 720 minutes: recommend splitting into two shifts, or use a larger buffer and explicitly document the risk.

Do not schedule at `:00` or `:30`.

## Run Shape

This run should have enough seed stories to avoid immediate drain mode.

Target seed count:

- Minimum: 8 concrete stories.
- Preferred: 10 to 12 stories.
- Maximum: 15 stories unless the user asks for a larger campaign.

Expected pass mix:

- Fix/Add/Extend: target at least 40 percent.
- Verify: acceptable when it captures fresh runtime receipts.
- Pin: rationed and only for load-bearing invariants with a specific refactor threat.
- Probe/Reproduce: every fourth pass, with the first attacker slot escalated if the Run 9 escalation condition still applies.

## Blocked Backlog Handling

Before arming, triage current open backlog.

Special handling for `tool-builtin-screenshot-trigger`:

- If the user explicitly wants screenshot builtins to become a first-class `triggerBuiltin screenshot` path, keep it open and seed a concrete implementation story.
- If the user has not chosen the product behavior, do not let it become Run 10 Pass #1.
- Either ask the user for the product decision during planning, or mark it `[!]` with a product-clarification blocker before arming.
- If editing `audits/afk/stories.md` to mark it blocked, commit that backlog housekeeping separately from pass commits.

The AFK loop prioritizes `tool-*` stories. Leaving a product-blocked `tool-*` story as the first open item will waste the supervised pass.

## Story Seed: Use These As The Starting Backlog

Seed the run with concrete, falsifiable stories. Adapt names and exact acceptance criteria after inspecting current code and logs.

### 1. `actions-debounce-builtins-cross-host-live`

Exercise the actions popup debounce behavior across built-in hosts.

Acceptance:

- Open file search, open actions, close actions, immediately switch to clipboard history or emoji picker and open actions again.
- Verify the new actions popup opens for the current host, not the previous host.
- Verify `listAutomationWindows` has no stale `actions-dialog` entry after hide.
- Prefer receipts from `getState`, `getElements`, and `listAutomationWindows`.

Falsifier:

- A receipt where Cmd+K is ignored within the debounce window when it should open for a new host.
- A receipt where `actions-dialog` remains visible or parented to the wrong surface after hide.

Likely pass type:

- Fix if behavior is wrong.
- Verify if live receipts prove the existing Run 9 fixes are correct.
- Pin only if a specific consolidation refactor threat is identified.

### 2. `actions-cmdk-notes-host-live`

Verify Cmd+K behavior from the notes window or notes-hosted surface.

Acceptance:

- Open the notes surface using the supported agentic path.
- Invoke Cmd+K through the supported runtime path.
- Verify actions attach to the notes host or correctly refuse with a documented reason.
- Hide/close the surface and verify automation-window registry cleanup.

Falsifier:

- Actions attach to `main` while the active host is notes.
- A stale popup remains after hide or close.
- The command silently no-ops without an error receipt.

### 3. `file-search-entry-focus-and-acp-tab-handoff`

Verify file search entry, focus retention, and ACP Tab handoff after recent file-search work.

Acceptance:

- Trigger file search.
- Set a query that returns at least one file result.
- Move selection.
- Exercise Enter and Tab behavior according to the current product contract.
- Verify focus, selected item, prompt type, and ACP handoff state through receipts.

Falsifier:

- Enter or Tab changes surface without updating automation state.
- Focus moves to an unexpected element.
- Selected semantic id or visible count contradicts the result list.

### 4. `filter-aware-getelements-across-builtins`

Verify that filterable subviews report filtered state consistently.

Surfaces to include when available:

- file search
- clipboard history
- emoji picker
- browser tabs
- app launcher
- window switcher

Acceptance:

- For each surface, trigger the surface.
- Set a normal query and a no-match query.
- Compare `getState.visibleChoiceCount` with `getElements.totalCount` and returned choice elements.
- Verify no-match state is represented consistently.

Falsifier:

- `visibleChoiceCount=0` while `getElements` still returns stale choices for that filtered view.
- `getElements` targets the wrong surface after a subview transition.
- Empty-state semantics are missing where the UI claims to show an empty state.

Likely pass type:

- Extend if agentic receipts cannot prove the surface.
- Fix if stale filtered elements are returned.

### 5. `automation-semantic-surface-cross-builtin-sweep`

Re-run a broad semantic-surface sweep across builtins and verify hide reset.

Acceptance:

- Trigger each supported builtin with `triggerBuiltin`.
- After each trigger, verify `listAutomationWindows` reports `main.semanticSurface` matching the active subview.
- After hide, verify `semanticSurface` resets to `scriptList` and the window is hidden.
- Include both common aliases and canonical names where useful.

Falsifier:

- A builtin renders one surface while the registry reports another.
- Hide leaves the previous semantic surface attached to hidden main.
- A known alias silently fails without parse or error receipt.

### 6. `acp-embedded-detached-hide-reattach-registry-clean`

Stress ACP embedded/detached lifecycle and automation-window cleanup.

Acceptance:

- Open ACP chat through the supported builtin or runtime path.
- If detached ACP is supported in current tools, detach and reattach.
- Hide main while ACP-related windows exist.
- Verify `listAutomationWindows` has no stale `ai`, `acpDetached`, or popup child entries after close/hide.
- Verify `getAcpState` target selection returns the correct `windowKind`.

Falsifier:

- `getAcpState` for `target:{type:"kind",kind:"ai"}` times out.
- `windowKind` is identical for distinct main and AI targets.
- Hidden or closed ACP windows remain visible in the registry.

### 7. `dictation-overlay-lifecycle-state-receipts`

Verify dictation overlay state reporting without invoking destructive or external behavior.

Acceptance:

- Use only allowed local/test-safe dictation paths.
- Exercise dictation overlay open, cancel, and close if supported by agentic tools.
- Verify `getAcpState.dictationPhase` or relevant state receipts are absent when idle and present when active.
- If synthetic transcript injection is missing, file or close the relevant tool-gap story instead of faking the proof.

Falsifier:

- Idle state emits a misleading active dictation phase.
- Active overlay is visible but no state receipt can observe it.
- Cancel leaves stale overlay state.

Likely pass type:

- Extend if the missing piece is a safe test-only RPC.
- Verify if current receipts are sufficient.

### 8. `stdin-requestid-boundary-rpc-correlation`

Re-attack stdin requestId parsing and RPC parse-error correlation.

Acceptance:

- Send malformed but known automation verbs with charset-safe requestIds.
- Send malformed payloads with charset-unsafe requestIds.
- Verify parse errors surface quickly for safe requestIds and fall back correctly for unsafe requestIds.
- Verify concurrent sends do not cross-correlate receipts.

Falsifier:

- A parse error becomes a generic timeout when requestId is safe.
- Concurrent sends all report the same command type.
- Prefix-matching requestIds cross-correlate.

### 9. `promptpopup-sibling-registry-future-proof-sweep`

Re-check prompt popup sibling registration and teardown after any new popup work.

Acceptance:

- Static axis: grep production `register_attached_popup` call sites and classify ids/kinds.
- Runtime axis: open and close available prompt popups through safe, non-destructive flows.
- Verify hide and close remove or hide every popup registry entry.

Falsifier:

- A new PromptPopup-kind id exists without hide-dispatcher teardown.
- A prompt popup remains visible after parent hide.
- Static and runtime popup inventories disagree.

### 10. `surface-proof-main-and-attached-popup-baselines`

Capture fresh `surface-proof` baselines for the main surface and at least one attached popup.

Acceptance:

- Use the repo's preferred proof CLI for already-open surfaces.
- Prove exact automation target identity.
- Verify attached popup parent identity and capture strategy.
- Do not rely on screenshots unless state proof is insufficient.

Falsifier:

- `surface-proof` selects an ambiguous target.
- Attached popup proof loses parent identity.
- Main proof captures the wrong subview.

### 11. `clipboard-history-empty-state-receipt-revisit`

Revisit the historical `empty-clipboard-state` gap without modifying the user's clipboard data destructively.

Acceptance:

- Do not clear the user's real clipboard history unless the user explicitly approves a safe test-only path.
- Verify whether current tools can represent no-match filtered clipboard history through `visibleChoiceCount` and `getElements`.
- If a true empty-dataset proof still requires forbidden action, mark the story as blocked with a precise tool/product gap.

Falsifier:

- The loop claims empty clipboard coverage after only testing no-match filtering.
- Tooling cannot distinguish total dataset count from filtered visible count.

### 12. `escalated-attacker-cross-surface-lifecycle`

The first attacker pass of this run must escalate if Run 9's escalation note still applies.

Minimum composition:

- At least 30 actions.
- At least 4 surfaces.
- At least 4 adversarial categories.
- At least 1 payload outside documented input bounds.

Candidate surfaces:

- script list
- file search
- clipboard history
- emoji picker
- ACP chat
- actions dialog
- prompt popup
- dictation overlay if safe

Required adversarial categories:

- Rapid-fire
- Concurrent
- Interleaved
- Boundary

Add Interrupted, Resurrection, or Composition if tools support them safely.

Acceptance:

- File `[?]` for near-anomalies, not only hard failures.
- Include action count and category list in the commit body.
- Do not fix in the same attacker pass. If a bug is found, commit the Probe/Reproduce first, then fix in a later normal pass.

Falsifier:

- The attacker pass uses fewer than 30 actions.
- It covers fewer than 4 surfaces.
- It uses no out-of-bounds payload.
- It finds a near-anomaly but logs only a generic clean Probe.

## Story Generation Policy

If the seed backlog drains before the budget ends:

1. Drain auto-promoted `tool-*` stories first, unless they require product decisions.
2. Prefer stories that exercise surfaces not seen in the last 10 pass entries.
3. Prefer user-visible flows over source-only pins.
4. Generate new stories only when open count is below the drain-mode threshold in `audits/afk/scope.md`.
5. File `[?]` anomalies from attacker passes immediately, with enough reproduction detail for a later Fix pass.

## Pass Selection Policy

At the start of every tick, obey the canonical `looper/tick-prompt.txt` steps.

Additional Run 10 bias:

- Prefer Fix/Add/Extend when a real bug or missing receipt blocks a proof.
- Prefer Verify when it captures a new live baseline across a meaningful surface.
- Avoid Pin unless the invariant is load-bearing and the commit names a concrete refactor threat.
- Do not use a Pin as a consolation prize for missing tools.
- If a story cannot be verified because tooling is missing, either extend the tool or mark the story `[!]` with a specific gap.

## Tooling Bias

When a proof is blocked by agentic tooling:

- Prefer adding or extending a receipt-producing RPC over screenshot-only verification.
- Prefer state receipts over visual capture.
- Prefer `waitFor`, `batch`, `getState`, `getElements`, `getAcpState`, and `listAutomationWindows`.
- Use screenshots only when state cannot prove the behavior.
- If screenshot capture is necessary, verify target identity before capture.

## Commit Discipline

Every pass commit must use the canonical subject verb taxonomy:

- `Prompt: Fix ...` for user-visible behavior changes.
- `Prompt: Add ...` for new RPC verbs, receipt fields, or capabilities.
- `Prompt: Extend ...` for expanding an existing tool or behavior to a new case.
- `Prompt: Pin ... against <named refactor>` for structural contract tests.
- `Prompt: Verify ...` for fresh baseline receipts with no code change.
- `Prompt: Probe ...` for attacker passes without acceptance criteria.
- `Prompt: Reproduce ...` for attacker anomalies with named trigger sequence.

Every pass commit body must include:

```text
User story: <slug> - <exact text from stories.md>
Pass: #<K> of AFK audit Run <RUN_N> <START_ISO>
Proof: <key receipt fields, or screenshot path only if needed>
Falsifier: <one concrete receipt state that would have proven this pass wrong>
```

Pin passes also require:

```text
Refactor threat: <specific contributor action that would break the invariant>
```

Attacker passes also require:

```text
Adversarial categories: <at least 3, or at least 4 if escalated>
Actions: <count>
```

Every pass updates `audits/afk/log.md` in the same commit with `Commit: <sha-pending>`, then immediately backfills the SHA in a separate `audit(log): backfill ...` commit.

No amend. No force push. No remote writes.

## Verification Requirements

For code changes:

- Run the narrowest relevant test.
- Run compile validation when Rust code changes.
- Run `lat check` whenever `lat.md/` changes.
- Use the repo's agentic runtime verification for UI behavior.

Good default proof ladder:

```bash
cargo check --lib
cargo test <narrow_test_name>
lat check
bash scripts/agentic/session.sh status default
bash scripts/agentic/session.sh rpc default '<json>' --expect <type> --timeout <ms>
```

Use `cargo test` or `cargo nextest` narrowly. Avoid known-red repo-wide suites unless the touched surface justifies them.

## Documentation Policy

If functionality, architecture, behavior, or tests change:

- Update `lat.md/` in the same pass.
- Keep leading paragraphs concise.
- Add or update source links where useful.
- Run `lat check` before considering the pass complete.

If only `audits/afk/log.md` or `audits/afk/stories.md` changes:

- `lat.md/` update is usually not required.
- Still run `lat check` before final handoff or scheduler arming.

## Forbidden Actions

Do not:

- Push to any remote.
- Force push.
- Use `git reset --hard`.
- Use `git rebase`.
- Use `git commit --amend`.
- Use `git commit --no-verify`.
- Delete files with `rm`, `rm -rf`, or `git rm`.
- Run `cargo clean`.
- Modify `.git/` internals.
- Read or write outside the project root except for approved scheduler APIs and required tool commands.
- Connect to paid or external APIs.
- Edit `~/.claude/skills/` during an active AFK loop.
- Edit `looper/rules/*.md` or `audits/afk/scope.md` under a live cron.

## Kill Switch

Document this in the plan and in the arming briefing:

```bash
touch .audit-pause
```

The next tick exits before doing work. Remove it only when intentionally re-arming.

## Plan To Produce Before Approval

Write a plan at:

```text
~/.claude/plans/run10-cross-surface-afk.md
```

Do not commit that plan file.

The plan must contain:

### 1. Context

State that this is a bounded AFK audit run focused on cross-surface workflow hardening. Name the safety principle: file-based scope, worktree coexistence, reversible path-scoped commits, falsifiable receipts, hard budget cutoff.

### 2. Driver Choice

Recommend cron for an overnight run. Include the budget, deadline, buffer cutoff, cadence, safe minute offsets, and the fixed-cadence tradeoff.

### 3. Pre-Flight

Paste the `git status --porcelain` result. Name dirty paths and the proposed disposition. Dirty external paths are allowed; they must remain unstaged and must not overlap the first story's write set.

### 4. Directory Layout And Files

List:

- `audits/afk/scope.md`
- `audits/afk/stories.md`
- `audits/afk/log.md`
- `audits/afk/promote-tool-gaps.sh`
- `.gitignore` with `.audit-pause`
- `looper/`

State whether any scaffolding changes are required.

### 5. Story Seed

List 8 to 12 stories, using the seed list above after adapting to current code. Each story must have a slug and one-line verifiable behavior.

Explicitly state the handling for `tool-builtin-screenshot-trigger`.

### 6. Loop Protocol And Commit Template

Reference `looper/tick-prompt.txt` and include the subject verb taxonomy. State that every claim needs Proof and Falsifier.

### 7. Verification Gate

Require one supervised tick before the user leaves. The user must review:

- log format
- commit subject verb
- diff scope
- proof receipt
- falsifier
- SHA backfill

Only after that supervised tick succeeds should the cron continue unattended.

## Execution After User Approval

After the user approves the plan:

1. Snapshot pre-flight dirty paths and mark external paths to ignore.
2. Update `audits/afk/stories.md` with the approved seed backlog and any backlog triage.
3. Update `audits/afk/log.md` with the Run header.
4. Run `lat check`.
5. Commit scaffolding/backlog changes separately from pass commits.
6. Compute deadline and buffer epochs and round-trip verify them.
7. Generate the filled tick prompt with:

```bash
bash looper/scripts/arm-loop.sh <BUDGET_MIN> audits/afk default "bash scripts/agentic/session.sh"
```

8. Create the cron with safe minute offsets.
9. Fill the cron id into `audits/afk/log.md`.
10. Commit the cron id.
11. Run one supervised tick.
12. Backfill the supervised tick SHA.
13. Brief the user on duration, deadline, buffer cutoff, kill switch, and where to inspect results.

## Final Handoff To User Before They Leave

Say:

- Run number.
- Budget.
- Deadline UTC.
- Buffer cutoff UTC.
- Cron id.
- Expected pass count.
- First three story slugs.
- First attacker slot requirements.
- Kill switch command.
- Where to inspect on return:

```bash
git log --oneline -30
sed -n '1,180p' audits/afk/log.md
rg -n "Run <N>|Pass #|scheduler-stop" audits/afk/log.md
```

## Quality Bar

This run is successful if the morning state is boring to audit:

- Every pass has one reversible SHA.
- Every claim has a proof and falsifier.
- The run stops before the buffer cutoff.
- The log explains every skipped or blocked story.
- Tool gaps become actionable backlog items.
- Real bugs and missing receipts are preferred over low-value pins.
- The user can reconstruct the run from `git log`, `audits/afk/log.md`, and `audits/afk/stories.md` without asking what happened.

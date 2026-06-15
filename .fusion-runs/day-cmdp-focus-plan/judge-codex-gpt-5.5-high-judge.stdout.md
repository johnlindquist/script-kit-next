## Consensus

Most useful agents agree that the fix should stay on the existing shared `CommandBar` / Notes-style switcher path, not introduce a separate Day Page popup UI.

They also agree on the likely patch owners:

- `src/main_sections/day_page_switcher.rs`: `open_note_switcher`, `wire_note_switcher_activation`, `handle_day_switcher_key`, close/restore helpers.
- `src/main_sections/day_page_view.rs`: Day Page key entry and focus handoff.
- `src/app_impl/actions_dialog.rs`: Actions Escape close focus restoration.
- Possibly `src/actions/command_bar.rs` if key-routing intent is shared instead of duplicated.

There is strong agreement that Day Page Actions Escape is a focus-target bug: Day Page is effectively treated as `ActionsDialogHost::MainList`, and `MainList` restores focus to the main filter rather than the Day editor.

Most successful outputs also agree that runtime proof with `script-kit-devtools` is required: Cmd+P opens, typing filters, arrows move selection, Escape closes, and subsequent typing lands in the Day editor; then Cmd+K/Escape should return focus to the Day editor.

## Contradictions

The main conflict is root cause and scope for Day Cmd+P key handling.

Codex and Kimi emphasize routing drift: Day’s `handle_day_switcher_key` duplicates private `CommandBar` key intent and should share or expose `command_bar_key_intent`. This is architecturally strong and covers future parity.

GLM argues the smallest likely bug is missing open-time focus transfer to the Day Page root, so the existing Day handler never receives keys. This is the best-supported explanation for the symptom cluster “typing, arrows, and Escape all fail” at once. If the root handler were receiving keys, at least some of the manual cases should work.

Best-supported synthesis: first fix focus ownership so the Day Page root or detached command bar actually receives key events; then reduce routing drift only as much as needed. Do not start with a broad focus-coordinator redesign unless source inspection proves the narrow focus target is insufficient.

There is also a scope conflict on adding `FocusTarget::DayPage`. Kimi recommends an explicit DayPage focus target. GLM recommends a narrow DayPage branch using existing editor-focus behavior. The narrower branch is better for a dirty tree unless current source already has a clean DayPage focus abstraction.

## Partial Coverage

Codex gave the cleanest architectural plan: shared `CommandBar` routing API, thin Day-specific adapter, and single Day-owned restore helper.

Kimi covered edge cases well: stale popup state, `reconcile_open_state`, `key` vs `key_char`, Home/End/PageUp/PageDown, row shortcuts, and simulation call-site fallout.

GLM gave the most pragmatic likely implementation: focus the Day Page root on `open_note_switcher`, leave existing `on_close` editor restore mostly alone, and special-case Actions focus restoration for Day Page.

Claude did not provide a usable analysis despite claiming it would inspect files.

Gemini provided no task-relevant content.

## Unique Insights

GLM’s strongest unique point is that `open_note_switcher` may be missing an explicit focus move to the Day Page root, matching the file’s intended contract. This is a high-value implementation clue because it explains all broken switcher keys without assuming every manual key branch is wrong.

Kimi’s strongest unique point is stale-state reconciliation: if the detached popup closes externally, Day Page should avoid staying in a logically open state that swallows or misroutes later keys.

Kimi also identified that using `key` instead of `key_char` can break composed/dead-key/non-ASCII input. That may not be the primary reported bug, but it matters for true Notes parity.

## Blind Spots

No panel actually ran `script-kit-devtools`, so the key-window ownership hypothesis is unverified.

The panels did not fully resolve whether the detached `ActionsWindow` should own keyboard events directly or whether the main Day Page root should be the routing owner. The final implementer should inspect current `CommandBar::open_centered`, ActionsWindow key handling, and Notes focus behavior before editing.

The panel did not deeply address tests already present in the repo. The final synthesizer should search existing focus/action/switcher tests before adding new source audits.

The dirty-tree constraint means any focus-coordinator enum expansion should be treated as higher-risk than a local DayPage branch unless the codebase already points there.

## Failure Notes

`claude-opus-4.8-high` effectively failed to return the requested artifact; it stopped at an attempted source-read transcript. This lowers confidence in skeptic coverage.

`agy-gemini-flash-high` returned only “I am powered by Gemini 3.5 Flash,” so it contributed no evidence.

The useful panel is therefore three agents: Codex, Kimi, and GLM. Confidence remains medium-high because those three converge on owners and focus restoration, but the exact Day Cmd+P root cause still needs runtime confirmation.

## Recommended Synthesis

Patch narrowly.

Start by inspecting current source around Day switcher open/focus and Notes browse-panel focus. If Day really does not move focus off the editor when Cmd+P opens, add the open-time focus transfer first. That is the most direct explanation for typing, arrows, and Escape all failing.

Keep using the existing `CommandBar` and `CommandBarConfig::notes_recent_style()`. Do not build a Day-specific popup UI.

For key routing, prefer a thin Day wrapper around shared `CommandBar` intent if the current manual router is actually reached and still incomplete. Avoid a broad extraction unless it is small and mirrors existing `command_bar_key_intent` cleanly.

For Day Cmd+P close, centralize focus restoration to the Day editor. If the existing `on_close` already calls `focus_editor`, preserve it, but verify it runs for Escape, external close, and activation. Add stale-state reconciliation if current source lacks it.

For Day Actions Escape, change focus restoration so `ActionsDialogHost::MainList` plus `AppView::DayPage` restores the Day editor, not the main filter. Use an existing `EditorPrompt`/Day editor path if available before adding a new focus target.

Verification should be:

- Focused Rust check/test through `./scripts/agentic/agent-cargo.sh`.
- Existing or new behavior test for Actions focus mapping if cheap.
- `script-kit-devtools` proof for Day Cmd+P typing/arrows/Escape/focus return.
- `script-kit-devtools` proof for Day Actions Cmd+K/Escape/focus return.

## Judge JSON

```json
{
  "scores": {
    "codex-gpt-5.5-high": {
      "correctness": 8,
      "task_fit": 9,
      "evidence": 7,
      "specificity": 9,
      "constraint_following": 9,
      "novelty": 7,
      "risk_awareness": 8,
      "cost_complexity": 6,
      "rationale": "Strong architecture and owners, correctly preserves shared CommandBar path and identifies Actions focus bug, but may over-scope by recommending shared key API before proving focus ownership."
    },
    "claude-opus-4.8-high": {
      "correctness": 1,
      "task_fit": 1,
      "evidence": 1,
      "specificity": 1,
      "constraint_following": 2,
      "novelty": 1,
      "risk_awareness": 1,
      "cost_complexity": 1,
      "rationale": "Did not return the requested analysis artifact; output is an incomplete process transcript."
    },
    "agy-gemini-flash-high": {
      "correctness": 1,
      "task_fit": 1,
      "evidence": 1,
      "specificity": 1,
      "constraint_following": 1,
      "novelty": 1,
      "risk_awareness": 1,
      "cost_complexity": 1,
      "rationale": "No task-relevant content was produced."
    },
    "kimi-code-high": {
      "correctness": 8,
      "task_fit": 9,
      "evidence": 8,
      "specificity": 9,
      "constraint_following": 9,
      "novelty": 8,
      "risk_awareness": 9,
      "cost_complexity": 6,
      "rationale": "Best edge-case coverage, especially stale state and key-char parity, though it likely expands focus abstractions more than necessary for a narrow dirty-tree fix."
    },
    "opencode-glm-5.2-high": {
      "correctness": 8,
      "task_fit": 9,
      "evidence": 8,
      "specificity": 9,
      "constraint_following": 9,
      "novelty": 8,
      "risk_awareness": 8,
      "cost_complexity": 9,
      "rationale": "Most pragmatic plan and strongest explanation for all key failures via missing focus transfer; slightly underweights future key-routing drift and key_char parity."
    }
  },
  "consensus": [
    "Day Page should keep using the shared CommandBar and Notes recent-note style rather than adding a separate popup UI.",
    "Likely patch owners are day_page_switcher.rs, day_page_view.rs, actions_dialog.rs, and possibly command_bar.rs.",
    "Day Actions Escape likely restores to the wrong focus target because Day Page is treated like MainList.",
    "Final proof must include script-kit-devtools verification for Cmd+P and Actions Escape focus return."
  ],
  "contradictions": [
    "Codex and Kimi prioritize sharing CommandBar key intent; GLM prioritizes fixing missing focus transfer on open. The best-supported position is to inspect and fix focus ownership first, then share routing only if needed.",
    "Kimi recommends adding an explicit DayPage focus target; GLM recommends a narrow DayPage branch using existing editor-focus paths. The narrow branch is better supported for this dirty-tree task unless current source already has a clean focus target extension point."
  ],
  "unsupported_claims": [
    "Any claim that shared key routing alone will fix typing/arrows/Escape is unsupported until runtime proves Day Page root receives those events.",
    "Any claim that open-time focus transfer alone fully fixes Notes parity is unsupported until printable input, arrows, Escape, and external close paths are verified.",
    "The exact detached-window key ownership behavior is not proven by the panel outputs."
  ],
  "unique_insights": [
    "GLM identified missing focus transfer to the Day Page root on switcher open as the most compact explanation for all dead keys.",
    "Kimi identified missing reconcile_open_state as a likely stale-popup recovery gap.",
    "Kimi identified key versus key_char as a parity issue for composed and non-ASCII input.",
    "GLM flagged vestigial day_switcher inline panel state as misleading design debt."
  ],
  "failure_notes": [
    "claude-opus-4.8-high did not return a usable artifact, limiting skeptic coverage.",
    "agy-gemini-flash-high returned no task-relevant analysis.",
    "No panel ran live script-kit-devtools, so the root-cause confidence depends on final runtime proof."
  ],
  "confidence": "medium",
  "escalation_needed": false,
  "synthesis_instructions": [
    "Inspect current source before editing; do not rely on stale docs or panel claims.",
    "Patch Day Cmd+P focus ownership first if open_note_switcher does not move focus off the editor.",
    "Keep the existing shared CommandBar and Notes-style UI; do not create a Day-specific popup.",
    "Use a thin Day wrapper around shared CommandBar routing only if current routing is reached or parity gaps remain after focus is fixed.",
    "Fix Actions Escape by restoring the Day editor when host is MainList and current view is DayPage.",
    "Verify with agent-cargo focused checks and script-kit-devtools receipts for Cmd+P typing, arrows, Escape, and focus return plus Actions Escape focus return."
  ]
}
```



The panel is directionally right, but several load-bearing claims are stale against current source.

The biggest correction: this is no longer a greenfield “replace Day bespoke switcher with Notes search” task. Current `DayPageView` already has a shared `note_switcher: CommandBar`, `open_note_switcher`, `wire_note_switcher_activation`, and `execute_note_switcher_action`; Day Page action `day_page:open_past_day` already calls `open_note_switcher`. Notes `open_browse_panel` already appends `get_day_note_switcher_actions`. So synthesis should focus on completing and cleaning the partial convergence, not blindly following deletion lists from the panel.

The riskiest unsupported claim is ID churn. Codex recommends new `daynote_` IDs, while current source uses `daypage_{date}` actions and also has a separate `src/notes/day_switcher.rs` with `day:` IDs wrapped by `note_` if used through `NoteSwitcherNoteInfo`. That mixed ID landscape needs source reconciliation before any rename. A rename could be useful, but it is not proven safer without checking every action consumer and test generator.

The old no-crossover probe and Notes stale guard are definitely obsolete for the new request, but the implementation must not just remove them. Notes currently still rejects `daypage_` in `execute_note_switcher_action`, despite already showing day rows, so this is a concrete contradiction: the UI can list rows that the executor refuses. That is the first fix to validate.

## Critic JSON

```json
{
  "claims": [
    {
      "claim": "Replace or bypass the Day Page bespoke switcher with the same CommandBar-based Notes search container.",
      "source": "consensus",
      "verdict": "weakened",
      "evidence_status": "cited",
      "counterargument": "Current source already has DayPageView.note_switcher, open_note_switcher, shared CommandBar activation wiring, and day_page:open_past_day calling open_note_switcher; the remaining work is cleanup and routing, not a full replacement from scratch.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Model day notes as shared search rows visible from both Notes and Day Page.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "Current Notes and Day Page paths both append get_day_note_switcher_actions, but the exact row model is split between actions/builders/notes.rs and src/notes/day_switcher.rs, so synthesis must account for duplicated or competing loaders.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Selection must route locally: Notes opens in Notes, Day Page binds in Day Page.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "The invariant matches the request, but current source contradicts it for Notes because execute_note_switcher_action still ignores daypage_ rows; Day Page already routes daypage_ to bind_day and note_ to session.bind_note.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "The prior no-crossover guard and probe must be removed or rewritten.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "The stale Notes branch logs notes_note_switcher_day_page_action_ignored and the runtime probe asserts no daypage rows, both directly conflict with the new request.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "New daynote_ IDs are better supported than flipping daypage_ semantics.",
      "source": "contradictions",
      "verdict": "weakened",
      "evidence_status": "contradicted",
      "counterargument": "Current source already uses daypage_ in get_day_note_switcher_actions and DayPageView.execute_note_switcher_action; there is also a separate day: helper in src/notes/day_switcher.rs. ID changes may help, but the panel did not prove migration safety.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Notes may need a day-file-backed editor binding mode so edits persist to brain/days/YYYY-MM-DD.md.",
      "source": "unique_insights",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "Current source already has active_day_binding and select_day_note, so the issue is not absence of a binding mode but whether selection routing and save paths are fully wired and tested.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Any exact file deletion list for DaySwitcherState needs reconciliation with current references before editing.",
      "source": "unsupported_claims",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "DaySwitcherState, render_day_page_day_switcher_panel, and handle_day_switcher_key still have render/key references, while a shared CommandBar path also exists. Deleting by list without checking call sites risks breaking fallback behavior or leaving dangling render state.",
      "synthesis_instruction": "may_assert"
    }
  ],
  "missed_constraints": [
    "Do not treat panel source facts as current; inspect this checkout because it already contains partial convergence work.",
    "Respect the existing AGENTS.md rule to use project imps for non-trivial owned-surface work if this proceeds to implementation.",
    "Cargo verification must use ./scripts/agentic/agent-cargo.sh.",
    "Runtime proof should use the existing DevTools probe style because this is a Day Page and Notes UI behavior change."
  ],
  "synthesis_must_include": [
    "Current Notes search already appends day-note actions but still ignores daypage_ selection, creating a visible row with dead routing.",
    "Current Day Page action day_page:open_past_day already opens the shared note switcher, while the old bespoke day_switcher render/key path still exists.",
    "There are two day-note modeling paths to reconcile: get_day_note_switcher_actions in actions/builders/notes.rs and src/notes/day_switcher.rs with day: IDs.",
    "Verify Notes day-file persistence through active_day_binding/select_day_note/save_current_note rather than claiming it from architecture alone."
  ],
  "synthesis_must_avoid": [
    "Avoid recommending a from-scratch CommandBar integration as if DayPageView has no note_switcher.",
    "Avoid asserting new daynote_ IDs are safer without a source-backed migration argument.",
    "Avoid deleting DaySwitcherState or old probe files before checking every current reference and replacement behavior.",
    "Avoid asking broad product questions that the user already answered: Day Page selection should stay local and Day-to-Notes requires an explicit action."
  ]
}
```



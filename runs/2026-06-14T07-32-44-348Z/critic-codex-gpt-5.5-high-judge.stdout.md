The stale implementation advice is the main hazard. Current source supports Kimi’s core facts: `src/notes/window/panels.rs::execute_note_switcher_action` already blocks `daypage_` before `note_` handling, and `open_browse_panel` currently builds only `get_note_switcher_actions` from `self.notes`. I found no current `src/notes/day_page_rows.rs`, no `open_day_page_in_main`, and no day-page action builder references under `src`.

The synthesizer should not recommend deleting day-page row machinery or removing row injection from Notes; that advice targets an older tree. The safe final answer should say the narrow implementation appears already present, then recommend proving it with the existing `scripts/agentic/notes-day-page-switcher-probe.ts` after a build through `./scripts/agentic/agent-cargo.sh`. Source-audit testing should be treated as fallback only; AGENTS.md explicitly makes source audits scarce and lower priority than runtime or behavior proof.

## Critic JSON

```json
{
  "claims": [
    {
      "claim": "The Notes Cmd+P action sink is src/notes/window/panels.rs::execute_note_switcher_action.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "No stronger competing owner was found; keyboard Enter and activation callbacks route through this function in current source.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "The fail-safe guarantee belongs at the daypage_ action-id branch before note selection logic.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "This is only a fail-safe for stale or injected actions because current open_browse_panel does not generate daypage_ rows.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "The current tree still injects Day Page rows into the Notes switcher and calls open_day_page_in_main.",
      "source": "codex-gpt-5.5-high and opencode-glm-5.2-high",
      "verdict": "refuted",
      "evidence_status": "contradicted",
      "counterargument": "Current rg and source reads show daypage_ only in the guard and probe; src/notes/day_page_rows.rs, open_day_page_in_main, and get_day_page_switcher_actions are absent.",
      "synthesis_instruction": "drop"
    },
    {
      "claim": "The implementation appears already present in src/notes/window/panels.rs unless targeting an older branch.",
      "source": "synthesis_instructions",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "It still needs runtime verification because source inspection alone does not prove the app state cannot switch to dayPage under the real Cmd+P path.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Keyboard Enter in the Notes switcher routes through execute_note_switcher_action, so the guard covers keyboard selection.",
      "source": "unique_insights",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "This only covers the visible selected-action path; any future direct dispatcher bypass would need its own check.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "The existing scripts/agentic/notes-day-page-switcher-probe.ts is aligned with the desired runtime proof.",
      "source": "unique_insights",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "The probe checks absence of daypage rows and main promptType not dayPage, but it does not forcibly inject a daypage_ action into the dispatcher.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "A new source-structural test is the right regression lock.",
      "source": "kimi-code-high",
      "verdict": "weakened",
      "evidence_status": "cited",
      "counterargument": "AGENTS.md says source audits are last resort and scarce; the existing runtime probe better matches this behavior contract.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Claude's repo file reads were corrupted in a way that invalidates all source claims.",
      "source": "claude-opus-4.8-high",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "The claim may describe that panel's tool session, but it should not override fresh current-tree inspection from this pass.",
      "synthesis_instruction": "drop"
    }
  ],
  "missed_constraints": [
    "Use ./scripts/agentic/agent-cargo.sh for any cargo build or test, not bare cargo.",
    "Prefer runtime or behavior proof before adding source-audit tests.",
    "Do not widen the task into day-page storage, main-window day behavior, or shared day switcher cleanup.",
    "Keep advice tied to current source unless explicitly targeting an older branch."
  ],
  "synthesis_must_include": [
    "Current implementation appears already present in src/notes/window/panels.rs.",
    "open_browse_panel currently sets only get_note_switcher_actions from self.notes.",
    "The daypage_ guard closes the browse panel, logs notes_note_switcher_day_page_action_ignored, and returns before note_ handling.",
    "Verify with SCRIPT_KIT_AGENT_ARTIFACT_NAME build through scripts/agentic/agent-cargo.sh and PROBE_BINARY bun scripts/agentic/notes-day-page-switcher-probe.ts.",
    "The existing probe checks no daypage_ rows, no day-page handoff log, and main promptType not dayPage."
  ],
  "synthesis_must_avoid": [
    "Do not claim src/notes/day_page_rows.rs currently exists.",
    "Do not recommend deleting day_page_rows or get_day_page_switcher_actions machinery.",
    "Do not claim execute_note_switcher_action currently calls open_day_page_in_main.",
    "Do not present source-audit testing as the preferred verification path.",
    "Do not rely on Claude's tooling-corruption narrative as a source fact."
  ]
}
```

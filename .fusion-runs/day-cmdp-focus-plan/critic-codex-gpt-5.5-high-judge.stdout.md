The judge’s escalation is justified. The strongest danger is that the synthesis could collapse two different hypotheses into one “root cause” before inspecting source: missing focus transfer on open, incomplete Day key routing, and detached-window key ownership can all explain overlapping symptoms, but the panel outputs did not prove which path receives events at runtime.

GLM’s “focus root on open fixes everything” is attractive because it is small, but it is still a hypothesis until current source and devtools prove key ownership. Kimi’s shared key-routing recommendation is more robust long-term, but it risks expanding the patch before proving the narrow focus bug. The synthesis should keep those separate: inspect and test focus ownership first, then only widen routing if printable chars/arrows/Escape still diverge.

Also, avoid adding source-audit tests casually. This repo’s policy treats them as last-resort decision locks; runtime proof plus a focused behavior test for focus restoration is a better fit unless no higher-rung test can express the invariant.

## Critic JSON

```json
{
  "claims": [
    {
      "claim": "Day Page should keep using the shared CommandBar and Notes recent-note style rather than adding a separate popup UI.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "The supplied source summary already says Day Page creates CommandBar with notes_recent_style and calls get_note_switcher_actions, so a separate popup would violate both current architecture and constraints.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Missing focus transfer to the Day Page root on switcher open is the compact explanation for typing, arrows, Escape, and focus-return failures.",
      "source": "unique_insights",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "It explains the symptom cluster, but the panel did not prove whether the detached CommandBar window, Day Page root, editor, or app key handler actually owns key events after open.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Shared key routing alone will fix typing, arrows, and Escape.",
      "source": "unsupported_claims",
      "verdict": "refuted",
      "evidence_status": "unverified",
      "counterargument": "If Day Page root never receives key events while the switcher is open, a better router in handle_day_switcher_key will still not run.",
      "synthesis_instruction": "drop"
    },
    {
      "claim": "Open-time focus transfer alone fully fixes Notes parity.",
      "source": "unsupported_claims",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "Even if focus transfer makes Day routing reachable, Day still appears to have a hand-rolled router that may differ from Notes for key_char, composed input, stale popup state, and additional navigation keys.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Day Actions Escape restores to the wrong focus target because Day Page is treated like MainList.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "The supplied source summary says ActionsDialogHost::MainList restoration maps to main filter, while Day Page needs editor focus; this is a direct focus-target mismatch unless current source has changed.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Adding a new explicit DayPage focus target is the right implementation.",
      "source": "kimi-code-high",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "It may be cleaner, but the dirty-tree and narrow-scope constraints favor first checking whether an existing EditorPrompt or Day editor focus path can express the fix with fewer files.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Day should expose or reuse CommandBar key intent now.",
      "source": "codex-gpt-5.5-high",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "Sharing the key intent may reduce drift, but widening CommandBar API before proving focus ownership could turn a small bug fix into a cross-surface refactor.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Kimi's stale reconcile_open_state gap is likely relevant.",
      "source": "unique_insights",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "It is a plausible robustness gap for external dismissal, but it does not directly explain first-open typing, arrow, and Escape failures unless stale open state is observed.",
      "synthesis_instruction": "label_as_hypothesis"
    }
  ],
  "missed_constraints": [
    "Inspect current source before asserting any panel claim as fact.",
    "Do not mint source-audit tests unless compiler, behavior tests, or runtime proof cannot express the invariant.",
    "Use ./scripts/agentic/agent-cargo.sh for cargo verification.",
    "Keep the dirty-tree fix narrowly scoped and preserve unrelated changes.",
    "Use script-kit-devtools as required runtime proof after implementation."
  ],
  "synthesis_must_include": [
    "Separate the focus-ownership hypothesis from the key-routing-parity hypothesis.",
    "State that detached-window key ownership is not proven by the panel outputs.",
    "Plan to verify Cmd+P typing, Up, Down, Escape close, and post-close Day editor focus.",
    "Plan to verify Actions Escape returns focus to the Day editor, not the main filter.",
    "Prefer the smallest existing Day editor focus path before adding new focus target types."
  ],
  "synthesis_must_avoid": [
    "Do not claim open-time focus transfer is the confirmed root cause before source inspection and runtime proof.",
    "Do not claim shared key routing alone fixes the bug.",
    "Do not introduce a separate Day Page popup UI.",
    "Do not expand CommandBar public API unless the current source inspection shows the local wrapper cannot be kept safely.",
    "Do not add broad source-audit tests or exact formatted-source count assertions."
  ]
}
```

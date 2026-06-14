## Short Critique

The strongest answer is directionally useful, but it overstates two things.

The markdown diagnosis is the best-supported claim: current editable markdown query uses `@text.uri` / `@text.reference`, while the supported/theme-backed capture names include `link_uri` / `link_text`. Still, synthesis must not claim the rename alone restores the exact expected yellow/link pixels until a highlighter span test or screenshot/pixel probe proves it.

The Notes clipping diagnosis is plausible but not fully proven. Current source does show editable `NotesEditor::render_input` ignores `NotesEditorLayout`, while preview consumes it, and Notes window still bypasses the entity method by calling `NotesEditor::render_input_state(&self.editor_state, cx)`. That supports “shared layout path is incomplete.” It does not prove the exact titlebar clipping mechanism. The final answer should present layout ownership as the PR-sized likely fix, with runtime geometry proof as required evidence, not as an already-verified root cause.

Also watch the metadata/probe fallout: existing probes and constants still expect `components.notes_editor.render_input_state`; changing render ownership may require updating runtime parity probes, source-audit tests, and style metadata. Do not frame existing metadata parity as visual proof.

## Critic JSON

```json
{
  "claims": [
    {
      "claim": "Use the shared NotesEditor path for both Notes and Day.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "Current Notes editable body calls the static raw render_input_state path rather than the NotesEditor entity render_input method, so the shared path exists but is not consistently the layout-owning path yet.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Implement layout ownership in NotesEditor::render_input, then remove Notes host padding to avoid double padding.",
      "source": "synthesis_instructions",
      "verdict": "weakened",
      "evidence_status": "cited",
      "counterargument": "Source proves editable render_input ignores NotesEditorLayout while preview uses it, but it does not prove this is the exact clipping mechanism; Day already has host insets and collectors hardcode padding metadata, so the final plan must mention auditing host wrappers and probes for double or stale padding.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Change Notes window editable body to call the NotesEditor entity render_input path.",
      "source": "synthesis_instructions",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "This is supported by current source: Notes body still calls NotesEditor::render_input_state(&self.editor_state, cx), bypassing the entity method that can own layout.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Fix markdown_highlights.scm captures from text.uri/text.reference to link_uri/link_text.",
      "source": "synthesis_instructions",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "The active query contains text.uri and text.reference, and the registry/theme-backed names include link_uri and link_text; however, exact visible color must still be verified with style spans or pixels.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "A query capture rename alone will restore the exact expected yellow/link appearance.",
      "source": "unsupported_claims",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "Capture names explain missing token styling, but no panel output proves the rendered Day editor applies the theme span or that the resulting color matches the screenshot expectation.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Do not enable markdown_inline injection.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "Existing tests explicitly assert editable markdown must not inject markdown_inline, so reviving injection would fight a documented performance contract.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "Existing day-page-style-parity-probe proves the visual regression is covered.",
      "source": "unsupported_claims",
      "verdict": "refuted",
      "evidence_status": "cited",
      "counterargument": "The probe checks metadata and style parity fields, not first-line bounds or visible link-highlight pixels.",
      "synthesis_instruction": "drop"
    }
  ],
  "missed_constraints": [
    "Use ./scripts/agentic/agent-cargo.sh for Rust verification commands.",
    "Do not revive deprecated inline Day popup behavior.",
    "Do not enable markdown_inline injection.",
    "DevTools runtime proof is required for UI regressions.",
    "Update existing source-audit and runtime probes that still encode render_input_state as the shared path if render ownership changes."
  ],
  "synthesis_must_include": [
    "Notes clipping root cause is likely layout ownership/bypass, but exact titlebar clipping mechanism remains unproven until runtime bounds or screenshot proof.",
    "Markdown registration metadata is insufficient; verify actual style spans or visible pixels for a Day markdown link.",
    "The PR-sized work should include render path cleanup, query capture rename, focused Rust tests, and a DevTools probe for Notes first-line bounds plus Day link highlighting.",
    "Existing probes named in panel output are not enough unless extended to check geometry and pixels."
  ],
  "synthesis_must_avoid": [
    "Do not claim language markdown or markdownRegistered proves visible highlighting.",
    "Do not claim the capture rename guarantees yellow/link appearance without verification.",
    "Do not propose separate Day and Notes editor implementations.",
    "Do not add broad roadmap or qmd cleanup work.",
    "Do not add a new source-audit test when a behavior test or runtime probe can express the invariant."
  ]
}
```



## Consensus

Most useful panel output supports the proposed core direction:

- Use `kit://clipboard-history?id=<entry_id>` as the canonical persisted brain/Today URI.
- Do not use `scriptkit://clipboard/<id>` for new persisted brain markdown; keep it as legacy/provenance compatibility.
- Persist only a generic label such as `[Clipboard entry](kit://clipboard-history?id=<id>)`.
- Do not persist raw URL labels, previews, excerpts, fragment bodies, or `Why:` bodies derived from clipboard content.
- Add a `DayEntry::ClipboardRef` or equivalent model rather than forcing clipboard references through `Capture`, `KeptUrl`, or `FragmentRef`.
- Change clipboard sediment writers so URL keeps and recopy promotion append deeplinks instead of raw values.
- Retention must protect referenced clipboard rows independently from user-visible pinning.
- Legacy migration should be forward-only first; do not rewrite old raw brain markdown in the initial milestone.
- Verification should be behavior tests/probes, not new source-audit tests unless no higher-rung option fits.

## Contradictions

The strongest contradiction is scope of retention protection.

Codex emphasized `prune_old_entries` and possibly clear behavior. GLM argued this is insufficient because there are multiple delete paths: database prune, db worker prune, clear unpinned, oversized trim, explicit remove, and full clear. GLM’s position is better supported because it names concrete paths and explains how each can create dead deeplinks. The synthesis should enumerate all deletion paths and classify them as automatic-protected or explicitly destructive.

There is a smaller contradiction around `annotate_clipboard_entry`.

Codex treats annotation as an implementation target. GLM says it is `#[cfg(test)]` and not a production writer. Best position: still audit/update it if it locks behavior in tests, but prioritize production paths `keep_url` and `promote_recopy`.

There is a product-scope tension around brain search.

Codex uniquely flags `src/brain/indexer.rs` pinned clipboard indexing as possible raw duplication outside markdown. The original request says “brain markdown,” but also says “I don’t want any raw values, all deeplinks.” Best-supported conservative interpretation: the final plan should call this out explicitly. If the user means all brain storage, not only day-page markdown, exclude or stub clipboard raw content in brain docs and search clipboard history directly.

## Partial Coverage

Codex covered the architecture cleanly: canonical URI, model changes, writer changes, generic labels, compatibility, risks, and smallest milestone.

GLM covered operational edge cases better: multiple delete paths, image entries, undo handling, URI-label leaks, dedup after deletion, oversized trim, and fake-ID tests.

Only Codex discussed the broader brain indexer risk and search-quality tradeoff in detail.

Only GLM gave concrete retention classification across delete paths and called out Day Page undo walker behavior.

Claude Opus did not provide the requested artifact; it only began describing intended inspections.

Agy Gemini did not provide the requested artifact.

## Unique Insights

- `src/brain/indexer.rs` may already duplicate pinned clipboard content into `brain_docs`; this matters if “no raw values” applies beyond markdown.
- Image clipboard entries need an explicit policy because the current resource resolver appears text-oriented. Smallest milestone should probably skip image deeplinks or defer image-aware resolution.
- `format_source_label` may render unknown `kit://clipboard-history?...` provenance as the literal URI unless a specific branch forces `"Clipboard"`.
- Undo/reject behavior needs a `ClipboardRef` branch, or rejecting a sediment entry may leave deeplink lines behind.
- Oversized text trimming is especially dangerous because long clipboard entries are exactly the values most likely to have been previously written as fragments.
- Re-copying deleted identical content does not revive old deeplinks if IDs are regenerated after row deletion.

## Blind Spots

- The panel did not fully inspect actual app deeplink click behavior. The final plan should specify initial behavior, probably open Clipboard History focused to the entry or show a generic resolver preview, and defer copy-back/attach actions.
- The plan should define what “raw” means for explicit user-authored notes. A user-supplied note can itself contain copied content. The safe rule is: never derive persisted text from clipboard content; user-entered annotation text is allowed only if explicitly typed and should not be auto-filled.
- Missing-ID rendering needs a product decision: show a generic dead-link state without leaking stale content.
- Bulk clear and explicit delete need UX copy or documented behavior because they can break brain references if allowed.
- URI escaping should be centralized even if IDs are UUID-like today.
- Agent Chat/context selector labels should be audited for preview leakage before implementation completes.

## Failure Notes

Claude Opus 4.8 high had `Status: ok` but failed to return the requested artifact. It began a verification narrative and stopped before findings. Treat as failed.

Agy Gemini 3.5 Flash high had `Status: ok` but returned only “I am currently running on Gemini 3.5 Flash.” Treat as failed.

This limits confidence in the skeptic/evidence-auditor coverage, but the Codex architect and GLM edge-case outputs are both strong and grounded in the source evidence provided in the prompt.

## Recommended Synthesis

Hand the implementer this plan:

1. Canonical persisted URI: `kit://clipboard-history?id=<entry_id>`.
2. Add a central helper such as `clipboard_history::entry_resource_uri(entry_id)` and a parser/helper that recognizes canonical `kit://clipboard-history?id=...`; optionally recognize `scriptkit://clipboard/<id>` only as legacy/provenance.
3. Add `DayEntry::ClipboardRef { entry_id, kind? }`, but make persisted markdown exactly generic, e.g. `HH:MM [Clipboard entry](kit://clipboard-history?id=<id>)`.
4. Update `src/clipboard_history/sediment.rs` production writers first: `keep_url` and `promote_recopy` should write `ClipboardRef`, not `KeptUrl`, `Capture`, or clipboard-origin fragments.
5. Stop creating new clipboard-origin raw fragments for long recopy content.
6. Update undo/reject removal logic in `src/brain/substrate/day.rs` to recognize and remove `ClipboardRef` lines by ID/URI.
7. Protect `brain_kept` rows from automatic deletion paths: main prune, db-worker prune, oversized trim, and clear-unpinned. Do not overload user-visible `pinned`.
8. Leave explicit single delete and full destructive clear as allowed but documented dead-link creators, unless product wants “break references?” confirmation.
9. Add Day Page parsing/rendering so clipboard resource links display generic labels only and never preview raw content or literal URI strings.
10. Add `format_source_label` handling for `kit://clipboard-history` returning `"Clipboard"`.
11. Decide image policy. Smallest milestone should skip image clipboard deeplinks unless image-aware resource resolution already exists.
12. Audit brain indexer/search. If the requirement applies to all brain storage, stop indexing raw pinned clipboard content in `brain_docs`; provide clipboard-history search as a separate provider.
13. Keep migration forward-only. Do not rewrite existing raw markdown in milestone one.
14. Tests: sediment behavior asserts deeplink present and raw copied URL/text absent; `DayEntry::ClipboardRef` formatting; undo/reject removes ref; brain-kept rows survive automatic pruning and oversized trim; Day Page source label for `kit://clipboard-history`; missing-ID resolver/rendering; keep no-popup sentinel unchanged.
15. Verification must use `./scripts/agentic/agent-cargo.sh` for cargo commands.

Smallest shippable milestone: text clipboard URL keep and recopy paths write only generic `kit://clipboard-history?id=...` links, no new raw fragments are created, automatic retention preserves referenced rows, undo removes the new line shape, Day Page labels are generic, and focused tests prove raw copied values are absent from generated day markdown.

## Judge JSON

```json
{
  "scores": {
    "codex-gpt-5.5-high": {
      "correctness": 8,
      "task_fit": 9,
      "evidence": 8,
      "specificity": 8,
      "constraint_following": 9,
      "novelty": 7,
      "risk_awareness": 8,
      "cost_complexity": 8,
      "rationale": "Strong architecture plan grounded in provided source evidence, with useful brain indexer and search tradeoff callouts; missed several delete-path and edge-case details."
    },
    "claude-opus-4.8-high": {
      "correctness": 1,
      "task_fit": 1,
      "evidence": 1,
      "specificity": 1,
      "constraint_following": 1,
      "novelty": 1,
      "risk_awareness": 2,
      "cost_complexity": 1,
      "rationale": "Did not return the requested artifact; only began describing intended inspections."
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
      "rationale": "Returned no usable analysis."
    },
    "opencode-glm-5.2-high": {
      "correctness": 9,
      "task_fit": 9,
      "evidence": 9,
      "specificity": 9,
      "constraint_following": 9,
      "novelty": 9,
      "risk_awareness": 10,
      "cost_complexity": 8,
      "rationale": "Excellent edge-case critique with concrete source-path risks around deletion, undo, labels, images, and oversized trimming."
    }
  },
  "consensus": [
    "Use kit://clipboard-history?id=<entry_id> as the canonical persisted URI.",
    "Do not use scriptkit://clipboard/<id> for new brain markdown; keep it for legacy or provenance compatibility.",
    "Persist generic Clipboard entry labels only, with no raw URL labels, previews, excerpts, or fragment bodies.",
    "Add a DayEntry::ClipboardRef-like model and update clipboard sediment writers.",
    "Retention must protect clipboard rows referenced from brain markdown.",
    "Initial migration should be forward-only."
  ],
  "contradictions": [
    "Codex focused mainly on prune and clear behavior, while GLM identified multiple delete paths; GLM is better supported and final synthesis should enumerate all deletion paths.",
    "Codex included annotate_clipboard_entry as a target, while GLM says it is test-gated; prioritize production writers but update test-gated code if it asserts the old contract.",
    "Codex raised raw brain index duplication, while GLM did not; include it as a requirement-interpretation checkpoint."
  ],
  "unsupported_claims": [
    "Any claim that scriptkit://clipboard/<id> should become the canonical persisted deeplink lacks support from current resource infrastructure.",
    "Any plan that changing prune_old_entries alone prevents dead links is unsupported.",
    "Any plan that preserves search by duplicating clipboard content into brain storage conflicts with the strict no-raw-values requirement unless explicitly scoped outside brain markdown."
  ],
  "unique_insights": [
    "Pinned clipboard content may be duplicated into brain_docs via brain indexing.",
    "Image clipboard entries need an explicit skip-or-image-resource policy.",
    "Day Page source label formatting may leak the literal kit URI without a specific branch.",
    "Undo/reject logic must learn the new ClipboardRef line shape.",
    "Oversized text trimming can delete the long clipboard rows most likely to be brain-linked.",
    "Deleted and recopied identical content may receive a new ID, leaving old deeplinks dead."
  ],
  "failure_notes": [
    "claude-opus-4.8-high failed to return the requested artifact, reducing skeptic coverage.",
    "agy-gemini-flash-high failed to return the requested artifact, reducing evidence-auditor coverage.",
    "Confidence remains medium because two other outputs were strong and the prompt included concrete source evidence."
  ],
  "confidence": "medium",
  "escalation_needed": true,
  "synthesis_instructions": [
    "Base the final plan on kit://clipboard-history?id=<entry_id> with generic labels only.",
    "Include all automatic delete paths in the retention plan, not only prune_old_entries.",
    "Require a DayEntry::ClipboardRef model, sediment writer changes, undo support, and Day Page generic rendering.",
    "Call out brain indexer raw duplication as an explicit scope decision.",
    "Make the smallest milestone text-only unless image resource resolution is verified.",
    "Use behavior tests and ./scripts/agentic/agent-cargo.sh for verification; do not add source-audit tests unless unavoidable."
  ]
}
```



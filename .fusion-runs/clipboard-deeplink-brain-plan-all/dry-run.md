# Local Fusion Dry Run

- Output directory: /Users/johnlindquist/dev/script-kit-gpui/.fusion-runs/clipboard-deeplink-brain-plan-all
- Providers: codex-gpt-5.5-high, claude-opus-4.8-high, agy-gemini-flash-high, kimi-code-high, opencode-glm-5.2-high

## Panel Commands

- codex-gpt-5.5-high: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -` (prompt via stdin)
- claude-opus-4.8-high: `claude --print --model claude-opus-4-8 --effort high --permission-mode dontAsk --no-session-persistence --tools '' -- 'Provider-specific instructions:
- Use only the user task and verified local evidence.
- Do not claim current source facts unless directly supported by inspected files, diffs, logs, or transcripts.
- Treat XML/tool transcripts and tool output as intermediate evidence, not as the final answer.
- If a source claim is not verified, label it unverified.
- Preserve and return the requested artifact.

Panel-specific reasoning contract:
Panel role: skeptic
Focus on the strongest objections, hidden failure modes, contradictions, and reasons this could be wrong.

Return your answer with these headings:
## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score

Original task:
# Fusion Planning Request: Clipboard Deeplinks Instead of Raw Brain Values

Repository: `/Users/johnlindquist/dev/script-kit-gpui`
Date: 2026-06-14

## User Request

The user wants clipboard entries to have deeplinks so the brain/Today can reference clipboard history entries instead of storing raw clipboard values. They explicitly said:

> I don'\''t want any raw values, all deeplinks.

Please plan this shift. This is an architecture and implementation planning request, not a code edit.

## Desired Output

Produce a concrete, staged implementation plan for this repo. Include:

1. The recommended canonical persisted URI format.
2. Whether `kit://clipboard-history?id=<id>` or `scriptkit://clipboard/<id>` should be used, and why.
3. The exact source areas to change and in what order.
4. Data model/API changes needed in clipboard history and brain substrate.
5. Retention/pinning policy so brain deeplinks do not become dead links.
6. Day Page rendering/parsing behavior.
7. Legacy compatibility and whether to migrate existing raw brain content.
8. Tests/probes to add or update.
9. Risks and tradeoffs, especially searchability, privacy, and durability.
10. A suggested smallest shippable milestone.

Please be strict about the requirement: no raw clipboard values in brain markdown, including URL labels, text previews, and fragment bodies.

## Repo Constraints

- Follow `AGENTS.md`.
- For non-trivial owned surfaces, project imps are advisory specialists. I already consulted:
  - `imp-sk-clipboard`
  - `imp-sk-brain`
- All cargo verification must use `./scripts/agentic/agent-cargo.sh`, not bare cargo.
- Preserve the existing no-popup clipboard-to-brain behavior. Do not revive post-copy popup UI.
- Source-audit tests are last resort.
- Keep implementation narrow and based on current source.

## Imp Findings To Reconcile

Both project imps said this is feasible but cross-owner.

Consensus:

- Existing clipboard rows have stable IDs.
- Existing resource URI: `kit://clipboard-history?id=<entry_id>`.
- Existing provenance URI: `scriptkit://clipboard/<entry_id>`.
- Brain/Today currently stores raw values in multiple paths.
- If the brain only stores deeplinks, clipboard history becomes the durable backing store.
- Retention/pinning must be addressed or links will decay.

Clipboard imp emphasized:

- `scriptkit://clipboard/<id>` exists as fragment provenance but is not currently handled by runtime deeplink routing.
- `kit://clipboard-history?id=<id>` is already resource-shaped.
- Need a product decision: open Clipboard History focused to entry, copy back, attach to Agent Chat, or show preview.

Brain imp emphasized:

- Standardize persisted references on `kit://clipboard-history?id=<id>` because it already resolves through resource infrastructure.
- Add a brain `DayEntry::ClipboardRef` or similar.
- Persist generic labels only, e.g. `[Clipboard entry](...)`, not preview text.
- Stop writing clipboard-origin fragments containing raw content.

## Current Source Evidence

Relevant code paths discovered with `rg` and `nl`.

### Clipboard sediment writes raw values today

File: `src/clipboard_history/sediment.rs`

- `process_text_sediment` routes URL keeps and re-copy promotion.
- `keep_url(entry_id, url, ...)` appends `DayEntry::KeptUrl { url: url.trim().to_string() }`.
- `annotate_clipboard_entry` fetches `get_entry_content(entry_id)`, builds a body containing raw text and optional `Why:`, then writes either a fragment or `DayEntry::Capture`.
- `promote_recopy(entry_id, text, ...)` writes `substrate.write_fragment(..., text)` for long content or `DayEntry::Capture { text }` for short content.
- `mark_brain_kept(entry_id, ...)` is called after writes.

Representative lines:

- `src/clipboard_history/sediment.rs:135-157`: URL keep writes raw URL.
- `src/clipboard_history/sediment.rs:176-208`: annotation writes raw content.
- `src/clipboard_history/sediment.rs:234-251`: re-copy promotion writes raw content or raw fragment.

### Brain day entry model writes raw values today

File: `src/brain/substrate/day.rs`

- `DayEntry::Capture { text }`
- `DayEntry::KeptUrl { url }`
- `DayEntry::FragmentRef(FragmentReference)`
- `format_line` writes:
  - capture as `{timestamp} {text}`
  - URL as `{timestamp} [{markdown_url_label(url)}]({url})`
  - fragment cards with excerpt and relative link

Representative lines:

- `src/brain/substrate/day.rs:20-38`: current `DayEntry` enum.
- `src/brain/substrate/day.rs:41-75`: current formatting.
- `src/brain/substrate/day.rs:93-124`: undo removes by raw URL/text or fragment source URI.

### Clipboard DB has sediment state but pruning only protects pinned rows

File: `src/clipboard_history/database.rs`

- `SedimentState` includes `brain_kept`, `brain_tier`, `copy_count`, `kept_url_day`.
- `prune_old_entries()` deletes rows where `pinned = 0 AND timestamp < ?`.
- `mark_brain_kept()` only sets `brain_kept = 1`; it does not pin or protect from prune.

Representative lines:

- `src/clipboard_history/database.rs:68-75`: `SedimentState`.
- `src/clipboard_history/database.rs:403-430`: pruning deletes unpinned old entries.
- `src/clipboard_history/database.rs:920-934`: `mark_brain_kept` updates brain metadata only.

### Resource resolution already supports kit clipboard URI

File: `src/mcp_resources/mod.rs`

- `parse_clipboard_history_request` accepts `kit://clipboard-history?id=<id>`.
- `read_clipboard_history_resource` with `id` returns entry text content as `text/plain`.

Representative lines:

- `src/mcp_resources/mod.rs:2029-2073`: parse URI.
- `src/mcp_resources/mod.rs:2096-2110`: single-entry resolution.

### Existing URI emitters

- `src/render_builtins/clipboard.rs`: emits `kit://clipboard-history?id={entry.id}` for clipboard UI/context.
- `src/ai/context_selector/mod.rs:1199`: emits `kit://clipboard-history?id={entry.id}`.
- `src/clipboard_history/sediment.rs`: uses `scriptkit://clipboard/{entry_id}` as fragment provenance.
- `src/day_page/sediment.rs:651`: treats `scriptkit://clipboard/` provenance as "Clipboard" label.

## Prior Constraints From Memory

- Post-copy clipboard tracking should remain popup-free.
- Real copy-to-brain behavior lives in `src/clipboard_history/sediment.rs`.
- `src/clipboard_history/post_copy.rs` is popup-free tracker/HUD glue.
- `tests/clipboard_sediment_no_popup_contract.rs` is the best sentinel against popup machinery returning.
- Today/Day Page work should preserve Notes parity and existing round-trip behavior.

## Proposed Direction To Critique

My current leaning:

- Persist `kit://clipboard-history?id=<entry_id>` in brain markdown, because it already exists as a resource URI and Agent Chat/context consumers know it.
- Add a helper like `clipboard_history::entry_resource_uri(entry_id: &str) -> String`.
- Add `DayEntry::ClipboardRef { entry_id: String, kind: ClipboardRefKind, note: Option<String> }`, but ensure persisted labels are generic and do not include raw values.
- Change clipboard sediment to append `ClipboardRef` for URL keeps and re-copy promotions.
- Stop creating clipboard-origin fragments for long recopy content.
- Change undo to remove by clipboard ref URI/id, not by raw value.
- Update retention so `brain_kept` entries survive pruning, either by changing prune SQL or by marking brain-kept rows pinned. Prefer not overloading user-visible pinning unless there is already a hidden/internal pin concept.
- Forward-only migration first; do not rewrite existing brain markdown unless later requested.

Please critique this and produce the plan you would hand to an implementer.'`
- agy-gemini-flash-high: `agy --print --model 'Gemini 3.5 Flash (High)' --print-timeout 45m --sandbox 'Provider-specific instructions:
- Stay anchored to the user'\''s task.
- Return only the requested artifact.
- Do not discuss the model, provider, config, runtime, tools, or your process unless explicitly requested.

Panel-specific reasoning contract:
Panel role: evidence-auditor
Focus on verified facts, assumptions, missing citations, unsupported claims, and what evidence would change the answer.

Return your answer with these headings:
## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score

Original task:
# Fusion Planning Request: Clipboard Deeplinks Instead of Raw Brain Values

Repository: `/Users/johnlindquist/dev/script-kit-gpui`
Date: 2026-06-14

## User Request

The user wants clipboard entries to have deeplinks so the brain/Today can reference clipboard history entries instead of storing raw clipboard values. They explicitly said:

> I don'\''t want any raw values, all deeplinks.

Please plan this shift. This is an architecture and implementation planning request, not a code edit.

## Desired Output

Produce a concrete, staged implementation plan for this repo. Include:

1. The recommended canonical persisted URI format.
2. Whether `kit://clipboard-history?id=<id>` or `scriptkit://clipboard/<id>` should be used, and why.
3. The exact source areas to change and in what order.
4. Data model/API changes needed in clipboard history and brain substrate.
5. Retention/pinning policy so brain deeplinks do not become dead links.
6. Day Page rendering/parsing behavior.
7. Legacy compatibility and whether to migrate existing raw brain content.
8. Tests/probes to add or update.
9. Risks and tradeoffs, especially searchability, privacy, and durability.
10. A suggested smallest shippable milestone.

Please be strict about the requirement: no raw clipboard values in brain markdown, including URL labels, text previews, and fragment bodies.

## Repo Constraints

- Follow `AGENTS.md`.
- For non-trivial owned surfaces, project imps are advisory specialists. I already consulted:
  - `imp-sk-clipboard`
  - `imp-sk-brain`
- All cargo verification must use `./scripts/agentic/agent-cargo.sh`, not bare cargo.
- Preserve the existing no-popup clipboard-to-brain behavior. Do not revive post-copy popup UI.
- Source-audit tests are last resort.
- Keep implementation narrow and based on current source.

## Imp Findings To Reconcile

Both project imps said this is feasible but cross-owner.

Consensus:

- Existing clipboard rows have stable IDs.
- Existing resource URI: `kit://clipboard-history?id=<entry_id>`.
- Existing provenance URI: `scriptkit://clipboard/<entry_id>`.
- Brain/Today currently stores raw values in multiple paths.
- If the brain only stores deeplinks, clipboard history becomes the durable backing store.
- Retention/pinning must be addressed or links will decay.

Clipboard imp emphasized:

- `scriptkit://clipboard/<id>` exists as fragment provenance but is not currently handled by runtime deeplink routing.
- `kit://clipboard-history?id=<id>` is already resource-shaped.
- Need a product decision: open Clipboard History focused to entry, copy back, attach to Agent Chat, or show preview.

Brain imp emphasized:

- Standardize persisted references on `kit://clipboard-history?id=<id>` because it already resolves through resource infrastructure.
- Add a brain `DayEntry::ClipboardRef` or similar.
- Persist generic labels only, e.g. `[Clipboard entry](...)`, not preview text.
- Stop writing clipboard-origin fragments containing raw content.

## Current Source Evidence

Relevant code paths discovered with `rg` and `nl`.

### Clipboard sediment writes raw values today

File: `src/clipboard_history/sediment.rs`

- `process_text_sediment` routes URL keeps and re-copy promotion.
- `keep_url(entry_id, url, ...)` appends `DayEntry::KeptUrl { url: url.trim().to_string() }`.
- `annotate_clipboard_entry` fetches `get_entry_content(entry_id)`, builds a body containing raw text and optional `Why:`, then writes either a fragment or `DayEntry::Capture`.
- `promote_recopy(entry_id, text, ...)` writes `substrate.write_fragment(..., text)` for long content or `DayEntry::Capture { text }` for short content.
- `mark_brain_kept(entry_id, ...)` is called after writes.

Representative lines:

- `src/clipboard_history/sediment.rs:135-157`: URL keep writes raw URL.
- `src/clipboard_history/sediment.rs:176-208`: annotation writes raw content.
- `src/clipboard_history/sediment.rs:234-251`: re-copy promotion writes raw content or raw fragment.

### Brain day entry model writes raw values today

File: `src/brain/substrate/day.rs`

- `DayEntry::Capture { text }`
- `DayEntry::KeptUrl { url }`
- `DayEntry::FragmentRef(FragmentReference)`
- `format_line` writes:
  - capture as `{timestamp} {text}`
  - URL as `{timestamp} [{markdown_url_label(url)}]({url})`
  - fragment cards with excerpt and relative link

Representative lines:

- `src/brain/substrate/day.rs:20-38`: current `DayEntry` enum.
- `src/brain/substrate/day.rs:41-75`: current formatting.
- `src/brain/substrate/day.rs:93-124`: undo removes by raw URL/text or fragment source URI.

### Clipboard DB has sediment state but pruning only protects pinned rows

File: `src/clipboard_history/database.rs`

- `SedimentState` includes `brain_kept`, `brain_tier`, `copy_count`, `kept_url_day`.
- `prune_old_entries()` deletes rows where `pinned = 0 AND timestamp < ?`.
- `mark_brain_kept()` only sets `brain_kept = 1`; it does not pin or protect from prune.

Representative lines:

- `src/clipboard_history/database.rs:68-75`: `SedimentState`.
- `src/clipboard_history/database.rs:403-430`: pruning deletes unpinned old entries.
- `src/clipboard_history/database.rs:920-934`: `mark_brain_kept` updates brain metadata only.

### Resource resolution already supports kit clipboard URI

File: `src/mcp_resources/mod.rs`

- `parse_clipboard_history_request` accepts `kit://clipboard-history?id=<id>`.
- `read_clipboard_history_resource` with `id` returns entry text content as `text/plain`.

Representative lines:

- `src/mcp_resources/mod.rs:2029-2073`: parse URI.
- `src/mcp_resources/mod.rs:2096-2110`: single-entry resolution.

### Existing URI emitters

- `src/render_builtins/clipboard.rs`: emits `kit://clipboard-history?id={entry.id}` for clipboard UI/context.
- `src/ai/context_selector/mod.rs:1199`: emits `kit://clipboard-history?id={entry.id}`.
- `src/clipboard_history/sediment.rs`: uses `scriptkit://clipboard/{entry_id}` as fragment provenance.
- `src/day_page/sediment.rs:651`: treats `scriptkit://clipboard/` provenance as "Clipboard" label.

## Prior Constraints From Memory

- Post-copy clipboard tracking should remain popup-free.
- Real copy-to-brain behavior lives in `src/clipboard_history/sediment.rs`.
- `src/clipboard_history/post_copy.rs` is popup-free tracker/HUD glue.
- `tests/clipboard_sediment_no_popup_contract.rs` is the best sentinel against popup machinery returning.
- Today/Day Page work should preserve Notes parity and existing round-trip behavior.

## Proposed Direction To Critique

My current leaning:

- Persist `kit://clipboard-history?id=<entry_id>` in brain markdown, because it already exists as a resource URI and Agent Chat/context consumers know it.
- Add a helper like `clipboard_history::entry_resource_uri(entry_id: &str) -> String`.
- Add `DayEntry::ClipboardRef { entry_id: String, kind: ClipboardRefKind, note: Option<String> }`, but ensure persisted labels are generic and do not include raw values.
- Change clipboard sediment to append `ClipboardRef` for URL keeps and re-copy promotions.
- Stop creating clipboard-origin fragments for long recopy content.
- Change undo to remove by clipboard ref URI/id, not by raw value.
- Update retention so `brain_kept` entries survive pruning, either by changing prune SQL or by marking brain-kept rows pinned. Prefer not overloading user-visible pinning unless there is already a hidden/internal pin concept.
- Forward-only migration first; do not rewrite existing brain markdown unless later requested.

Please critique this and produce the plan you would hand to an implementer.'`
- kimi-code-high: `/Users/johnlindquist/Library/pnpm/nodejs/25.2.1/bin/node /Users/johnlindquist/dev/fusion/src/kimi-thinking.js high -m kimi-code/kimi-for-coding -p 'Panel-specific reasoning contract:
Panel role: edge-case-tester
Focus on boundary cases, regressions, weird inputs, operational failure modes, and test coverage gaps.

Return your answer with these headings:
## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score

Original task:
# Fusion Planning Request: Clipboard Deeplinks Instead of Raw Brain Values

Repository: `/Users/johnlindquist/dev/script-kit-gpui`
Date: 2026-06-14

## User Request

The user wants clipboard entries to have deeplinks so the brain/Today can reference clipboard history entries instead of storing raw clipboard values. They explicitly said:

> I don'\''t want any raw values, all deeplinks.

Please plan this shift. This is an architecture and implementation planning request, not a code edit.

## Desired Output

Produce a concrete, staged implementation plan for this repo. Include:

1. The recommended canonical persisted URI format.
2. Whether `kit://clipboard-history?id=<id>` or `scriptkit://clipboard/<id>` should be used, and why.
3. The exact source areas to change and in what order.
4. Data model/API changes needed in clipboard history and brain substrate.
5. Retention/pinning policy so brain deeplinks do not become dead links.
6. Day Page rendering/parsing behavior.
7. Legacy compatibility and whether to migrate existing raw brain content.
8. Tests/probes to add or update.
9. Risks and tradeoffs, especially searchability, privacy, and durability.
10. A suggested smallest shippable milestone.

Please be strict about the requirement: no raw clipboard values in brain markdown, including URL labels, text previews, and fragment bodies.

## Repo Constraints

- Follow `AGENTS.md`.
- For non-trivial owned surfaces, project imps are advisory specialists. I already consulted:
  - `imp-sk-clipboard`
  - `imp-sk-brain`
- All cargo verification must use `./scripts/agentic/agent-cargo.sh`, not bare cargo.
- Preserve the existing no-popup clipboard-to-brain behavior. Do not revive post-copy popup UI.
- Source-audit tests are last resort.
- Keep implementation narrow and based on current source.

## Imp Findings To Reconcile

Both project imps said this is feasible but cross-owner.

Consensus:

- Existing clipboard rows have stable IDs.
- Existing resource URI: `kit://clipboard-history?id=<entry_id>`.
- Existing provenance URI: `scriptkit://clipboard/<entry_id>`.
- Brain/Today currently stores raw values in multiple paths.
- If the brain only stores deeplinks, clipboard history becomes the durable backing store.
- Retention/pinning must be addressed or links will decay.

Clipboard imp emphasized:

- `scriptkit://clipboard/<id>` exists as fragment provenance but is not currently handled by runtime deeplink routing.
- `kit://clipboard-history?id=<id>` is already resource-shaped.
- Need a product decision: open Clipboard History focused to entry, copy back, attach to Agent Chat, or show preview.

Brain imp emphasized:

- Standardize persisted references on `kit://clipboard-history?id=<id>` because it already resolves through resource infrastructure.
- Add a brain `DayEntry::ClipboardRef` or similar.
- Persist generic labels only, e.g. `[Clipboard entry](...)`, not preview text.
- Stop writing clipboard-origin fragments containing raw content.

## Current Source Evidence

Relevant code paths discovered with `rg` and `nl`.

### Clipboard sediment writes raw values today

File: `src/clipboard_history/sediment.rs`

- `process_text_sediment` routes URL keeps and re-copy promotion.
- `keep_url(entry_id, url, ...)` appends `DayEntry::KeptUrl { url: url.trim().to_string() }`.
- `annotate_clipboard_entry` fetches `get_entry_content(entry_id)`, builds a body containing raw text and optional `Why:`, then writes either a fragment or `DayEntry::Capture`.
- `promote_recopy(entry_id, text, ...)` writes `substrate.write_fragment(..., text)` for long content or `DayEntry::Capture { text }` for short content.
- `mark_brain_kept(entry_id, ...)` is called after writes.

Representative lines:

- `src/clipboard_history/sediment.rs:135-157`: URL keep writes raw URL.
- `src/clipboard_history/sediment.rs:176-208`: annotation writes raw content.
- `src/clipboard_history/sediment.rs:234-251`: re-copy promotion writes raw content or raw fragment.

### Brain day entry model writes raw values today

File: `src/brain/substrate/day.rs`

- `DayEntry::Capture { text }`
- `DayEntry::KeptUrl { url }`
- `DayEntry::FragmentRef(FragmentReference)`
- `format_line` writes:
  - capture as `{timestamp} {text}`
  - URL as `{timestamp} [{markdown_url_label(url)}]({url})`
  - fragment cards with excerpt and relative link

Representative lines:

- `src/brain/substrate/day.rs:20-38`: current `DayEntry` enum.
- `src/brain/substrate/day.rs:41-75`: current formatting.
- `src/brain/substrate/day.rs:93-124`: undo removes by raw URL/text or fragment source URI.

### Clipboard DB has sediment state but pruning only protects pinned rows

File: `src/clipboard_history/database.rs`

- `SedimentState` includes `brain_kept`, `brain_tier`, `copy_count`, `kept_url_day`.
- `prune_old_entries()` deletes rows where `pinned = 0 AND timestamp < ?`.
- `mark_brain_kept()` only sets `brain_kept = 1`; it does not pin or protect from prune.

Representative lines:

- `src/clipboard_history/database.rs:68-75`: `SedimentState`.
- `src/clipboard_history/database.rs:403-430`: pruning deletes unpinned old entries.
- `src/clipboard_history/database.rs:920-934`: `mark_brain_kept` updates brain metadata only.

### Resource resolution already supports kit clipboard URI

File: `src/mcp_resources/mod.rs`

- `parse_clipboard_history_request` accepts `kit://clipboard-history?id=<id>`.
- `read_clipboard_history_resource` with `id` returns entry text content as `text/plain`.

Representative lines:

- `src/mcp_resources/mod.rs:2029-2073`: parse URI.
- `src/mcp_resources/mod.rs:2096-2110`: single-entry resolution.

### Existing URI emitters

- `src/render_builtins/clipboard.rs`: emits `kit://clipboard-history?id={entry.id}` for clipboard UI/context.
- `src/ai/context_selector/mod.rs:1199`: emits `kit://clipboard-history?id={entry.id}`.
- `src/clipboard_history/sediment.rs`: uses `scriptkit://clipboard/{entry_id}` as fragment provenance.
- `src/day_page/sediment.rs:651`: treats `scriptkit://clipboard/` provenance as "Clipboard" label.

## Prior Constraints From Memory

- Post-copy clipboard tracking should remain popup-free.
- Real copy-to-brain behavior lives in `src/clipboard_history/sediment.rs`.
- `src/clipboard_history/post_copy.rs` is popup-free tracker/HUD glue.
- `tests/clipboard_sediment_no_popup_contract.rs` is the best sentinel against popup machinery returning.
- Today/Day Page work should preserve Notes parity and existing round-trip behavior.

## Proposed Direction To Critique

My current leaning:

- Persist `kit://clipboard-history?id=<entry_id>` in brain markdown, because it already exists as a resource URI and Agent Chat/context consumers know it.
- Add a helper like `clipboard_history::entry_resource_uri(entry_id: &str) -> String`.
- Add `DayEntry::ClipboardRef { entry_id: String, kind: ClipboardRefKind, note: Option<String> }`, but ensure persisted labels are generic and do not include raw values.
- Change clipboard sediment to append `ClipboardRef` for URL keeps and re-copy promotions.
- Stop creating clipboard-origin fragments for long recopy content.
- Change undo to remove by clipboard ref URI/id, not by raw value.
- Update retention so `brain_kept` entries survive pruning, either by changing prune SQL or by marking brain-kept rows pinned. Prefer not overloading user-visible pinning unless there is already a hidden/internal pin concept.
- Forward-only migration first; do not rewrite existing brain markdown unless later requested.

Please critique this and produce the plan you would hand to an implementer.' --output-format text`
- opencode-glm-5.2-high: `opencode --pure run -m zai-coding-plan/glm-5.2 --variant high --dir /Users/johnlindquist/dev/script-kit-gpui --format default 'Panel-specific reasoning contract:
Panel role: pragmatist
Focus on the smallest implementation that fully satisfies the task, avoids unnecessary scope, and can be verified cheaply.

Return your answer with these headings:
## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score

Original task:
# Fusion Planning Request: Clipboard Deeplinks Instead of Raw Brain Values

Repository: `/Users/johnlindquist/dev/script-kit-gpui`
Date: 2026-06-14

## User Request

The user wants clipboard entries to have deeplinks so the brain/Today can reference clipboard history entries instead of storing raw clipboard values. They explicitly said:

> I don'\''t want any raw values, all deeplinks.

Please plan this shift. This is an architecture and implementation planning request, not a code edit.

## Desired Output

Produce a concrete, staged implementation plan for this repo. Include:

1. The recommended canonical persisted URI format.
2. Whether `kit://clipboard-history?id=<id>` or `scriptkit://clipboard/<id>` should be used, and why.
3. The exact source areas to change and in what order.
4. Data model/API changes needed in clipboard history and brain substrate.
5. Retention/pinning policy so brain deeplinks do not become dead links.
6. Day Page rendering/parsing behavior.
7. Legacy compatibility and whether to migrate existing raw brain content.
8. Tests/probes to add or update.
9. Risks and tradeoffs, especially searchability, privacy, and durability.
10. A suggested smallest shippable milestone.

Please be strict about the requirement: no raw clipboard values in brain markdown, including URL labels, text previews, and fragment bodies.

## Repo Constraints

- Follow `AGENTS.md`.
- For non-trivial owned surfaces, project imps are advisory specialists. I already consulted:
  - `imp-sk-clipboard`
  - `imp-sk-brain`
- All cargo verification must use `./scripts/agentic/agent-cargo.sh`, not bare cargo.
- Preserve the existing no-popup clipboard-to-brain behavior. Do not revive post-copy popup UI.
- Source-audit tests are last resort.
- Keep implementation narrow and based on current source.

## Imp Findings To Reconcile

Both project imps said this is feasible but cross-owner.

Consensus:

- Existing clipboard rows have stable IDs.
- Existing resource URI: `kit://clipboard-history?id=<entry_id>`.
- Existing provenance URI: `scriptkit://clipboard/<entry_id>`.
- Brain/Today currently stores raw values in multiple paths.
- If the brain only stores deeplinks, clipboard history becomes the durable backing store.
- Retention/pinning must be addressed or links will decay.

Clipboard imp emphasized:

- `scriptkit://clipboard/<id>` exists as fragment provenance but is not currently handled by runtime deeplink routing.
- `kit://clipboard-history?id=<id>` is already resource-shaped.
- Need a product decision: open Clipboard History focused to entry, copy back, attach to Agent Chat, or show preview.

Brain imp emphasized:

- Standardize persisted references on `kit://clipboard-history?id=<id>` because it already resolves through resource infrastructure.
- Add a brain `DayEntry::ClipboardRef` or similar.
- Persist generic labels only, e.g. `[Clipboard entry](...)`, not preview text.
- Stop writing clipboard-origin fragments containing raw content.

## Current Source Evidence

Relevant code paths discovered with `rg` and `nl`.

### Clipboard sediment writes raw values today

File: `src/clipboard_history/sediment.rs`

- `process_text_sediment` routes URL keeps and re-copy promotion.
- `keep_url(entry_id, url, ...)` appends `DayEntry::KeptUrl { url: url.trim().to_string() }`.
- `annotate_clipboard_entry` fetches `get_entry_content(entry_id)`, builds a body containing raw text and optional `Why:`, then writes either a fragment or `DayEntry::Capture`.
- `promote_recopy(entry_id, text, ...)` writes `substrate.write_fragment(..., text)` for long content or `DayEntry::Capture { text }` for short content.
- `mark_brain_kept(entry_id, ...)` is called after writes.

Representative lines:

- `src/clipboard_history/sediment.rs:135-157`: URL keep writes raw URL.
- `src/clipboard_history/sediment.rs:176-208`: annotation writes raw content.
- `src/clipboard_history/sediment.rs:234-251`: re-copy promotion writes raw content or raw fragment.

### Brain day entry model writes raw values today

File: `src/brain/substrate/day.rs`

- `DayEntry::Capture { text }`
- `DayEntry::KeptUrl { url }`
- `DayEntry::FragmentRef(FragmentReference)`
- `format_line` writes:
  - capture as `{timestamp} {text}`
  - URL as `{timestamp} [{markdown_url_label(url)}]({url})`
  - fragment cards with excerpt and relative link

Representative lines:

- `src/brain/substrate/day.rs:20-38`: current `DayEntry` enum.
- `src/brain/substrate/day.rs:41-75`: current formatting.
- `src/brain/substrate/day.rs:93-124`: undo removes by raw URL/text or fragment source URI.

### Clipboard DB has sediment state but pruning only protects pinned rows

File: `src/clipboard_history/database.rs`

- `SedimentState` includes `brain_kept`, `brain_tier`, `copy_count`, `kept_url_day`.
- `prune_old_entries()` deletes rows where `pinned = 0 AND timestamp < ?`.
- `mark_brain_kept()` only sets `brain_kept = 1`; it does not pin or protect from prune.

Representative lines:

- `src/clipboard_history/database.rs:68-75`: `SedimentState`.
- `src/clipboard_history/database.rs:403-430`: pruning deletes unpinned old entries.
- `src/clipboard_history/database.rs:920-934`: `mark_brain_kept` updates brain metadata only.

### Resource resolution already supports kit clipboard URI

File: `src/mcp_resources/mod.rs`

- `parse_clipboard_history_request` accepts `kit://clipboard-history?id=<id>`.
- `read_clipboard_history_resource` with `id` returns entry text content as `text/plain`.

Representative lines:

- `src/mcp_resources/mod.rs:2029-2073`: parse URI.
- `src/mcp_resources/mod.rs:2096-2110`: single-entry resolution.

### Existing URI emitters

- `src/render_builtins/clipboard.rs`: emits `kit://clipboard-history?id={entry.id}` for clipboard UI/context.
- `src/ai/context_selector/mod.rs:1199`: emits `kit://clipboard-history?id={entry.id}`.
- `src/clipboard_history/sediment.rs`: uses `scriptkit://clipboard/{entry_id}` as fragment provenance.
- `src/day_page/sediment.rs:651`: treats `scriptkit://clipboard/` provenance as "Clipboard" label.

## Prior Constraints From Memory

- Post-copy clipboard tracking should remain popup-free.
- Real copy-to-brain behavior lives in `src/clipboard_history/sediment.rs`.
- `src/clipboard_history/post_copy.rs` is popup-free tracker/HUD glue.
- `tests/clipboard_sediment_no_popup_contract.rs` is the best sentinel against popup machinery returning.
- Today/Day Page work should preserve Notes parity and existing round-trip behavior.

## Proposed Direction To Critique

My current leaning:

- Persist `kit://clipboard-history?id=<entry_id>` in brain markdown, because it already exists as a resource URI and Agent Chat/context consumers know it.
- Add a helper like `clipboard_history::entry_resource_uri(entry_id: &str) -> String`.
- Add `DayEntry::ClipboardRef { entry_id: String, kind: ClipboardRefKind, note: Option<String> }`, but ensure persisted labels are generic and do not include raw values.
- Change clipboard sediment to append `ClipboardRef` for URL keeps and re-copy promotions.
- Stop creating clipboard-origin fragments for long recopy content.
- Change undo to remove by clipboard ref URI/id, not by raw value.
- Update retention so `brain_kept` entries survive pruning, either by changing prune SQL or by marking brain-kept rows pinned. Prefer not overloading user-visible pinning unless there is already a hidden/internal pin concept.
- Forward-only migration first; do not rewrite existing brain markdown unless later requested.

Please critique this and produce the plan you would hand to an implementer.'`

## Judge Command

- codex-gpt-5.5-high-judge: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -`

## Critic Command

- codex-gpt-5.5-high-judge: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -` (conditional on judge escalation)

## Synthesizer Command

- codex-gpt-5.5-high-synthesizer: `codex exec --skip-git-repo-check --ephemeral -C /Users/johnlindquist/dev/script-kit-gpui -s read-only -m gpt-5.5 -c 'model_reasoning_effort="high"' -`

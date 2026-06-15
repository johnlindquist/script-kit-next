## Role Findings

The proposed direction is sound at the architecture level but under-specifies **five operational delete paths**, **two prune code paths**, **image entries**, **dedup-vs-delete ID regeneration**, and **the day-page undo walker**. As written, the plan will ship dead deeplinks in production within hours of release. Concrete gaps:

1. **There is no single "prune" function — there are at least five delete paths**, and only updating one of them still leaves brain-backed rows exposed.
2. **Image entries never go through `process_text_sediment`** (`monitor.rs:349,386`), and the resource resolver returns `text/plain` only — a deeplink to an image entry returns the blob key string, not the image.
3. **The dedup-by-hash path means a deleted entry's ID is gone forever, even if identical content is re-copied seconds later.** New UUID → old deeplink is dead. Retention must protect the ID, not just the row.
4. **The day-page undo walker in `day.rs:168-195` only knows three line shapes** (fragment card, legacy header, timestamped fragment link). A new `ClipboardRef` line shape is invisible to reject-undo, so rejecting an entry will leak deeplink lines on the day page.
5. **`format_source_label` in `day_page/sediment.rs:649-657` would render `kit://clipboard-history?id=<uuid>` as the literal URI** in any fragment provenance hint, because the catch-all "non-empty uri" branch returns `uri.to_string()`. That is a raw-value leak by the user's own strict definition.
6. **Production writers are only `keep_url` and `promote_recopy`** — `annotate_clipboard_entry` is `#[cfg(test)]` (`sediment.rs:175`). The plan's scope is narrower than it appears; do not waste effort on annotate.

## Evidence And Assumptions

- `src/clipboard_history/database.rs:406-430` (`prune_old_entries`) and `src/clipboard_history/db_worker/db_impl.rs:194-205` (`prune_impl`) are **two independent** prune implementations, both `WHERE pinned = 0 AND timestamp < ?`. `mark_brain_kept` (`database.rs:920-934`) sets `brain_kept = 1` but does **not** touch `pinned`, so neither prune query protects brain-backed rows today.
- Additional delete paths that bypass any brain-kept protection: `clear_unpinned_history` (`database.rs:794`, `WHERE pinned = 0`), `clear_impl` (`db_impl.rs:187`, unconditional), `remove_entry` (`database.rs:710`), `trim_oversized_text_entries` (`database.rs:435`, deletes by byte length regardless of brain_kept).
- `add_entry` (`database.rs:315-400`) regenerates the UUID only when `content_hash` is novel; existing rows get `copy_count + 1` and keep their ID. So ID stability holds **only while the row exists**.
- `src/mcp_resources/mod.rs:2096-2110`: single-entry resolver returns `get_entry_content` as `text/plain`. For images, `content` is a blob key (e.g. `blob:...`), so the deeplink target is meaningless without a separate image-aware resolver path.
- `src/brain/substrate/day.rs:168-195`: `fragment_reference_line_count_for_source` enumerates exactly three recognized line shapes. Adding `DayEntry::ClipboardRef` requires a fourth branch or undo silently no-ops.
- `src/day_page/sediment.rs:649-657`: catch-all branch returns the URI verbatim as the label. `kit://…` has no matching branch.
- `src/clipboard_history/sediment.rs:174-211`: `annotate_clipboard_entry` is test-gated; the only production writers are `keep_url` (135-164) and `promote_recopy` (234-254).
- Assumption: forward-only migration is acceptable (per your own leaning), and dead legacy raw lines are tolerated until a later cleanup pass.

## Failure Modes

- **F1 — Retention kills brain deeplinks.** Any of the 5 delete paths can drop a row whose ID is referenced from day-page markdown. Symptom: `[Clipboard entry](kit://clipboard-history?id=<gone>)` → resolver error string rendered in place of content.
- **F2 — Re-copy after manual delete resurrects content under a new ID.** Old deeplink stays dead; new copy may or may not get a new deeplink depending on `brain_kept` reset semantics.
- **F3 — Image entries produce nonsense deeplinks.** Resource resolver returns blob key as `text/plain`. Either skip images entirely or add a `image/...` mime path in `read_clipboard_history_resource`.
- **F4 — Reject does not remove deeplink lines.** `undo_clipboard_sediment_lines` only knows three line shapes. New `ClipboardRef` lines leak on reject.
- **F5 — Label leaks the URI.** `format_source_label` catch-all emits the full `kit://clipboard-history?id=…` as the visible provenance label. Violates the "no raw values" rule by the strict reading.
- **F6 — `trim_oversized_text_entries` deletes long brain-backed rows.** The exact case the user cares about (large clipboard content) is the case most likely to be trimmed.
- **F7 — `clear_unpinned_history` ("Clear all") silently destroys brain-backed rows** because its WHERE clause only checks `pinned = 0`. User clicks "Clear non-pinned" → mass dead links.
- **F8 — Duplicate deeplinks across day boundary.** URL copy at 23:59 and 00:01 produces two `ClipboardRef` lines (today + tomorrow) referencing the same entry. Acceptable but must be a documented behavior, not a surprise.
- **F9 — Schema migration gap.** `ensure_clipboard_schema` (`database.rs:77-101`) only migrates `ocr_text` and `content_hash`. If the plan adds a `brain_protected` column, a third migration block must be added or the column won't exist on legacy DBs.
- **F10 — Resource URI parser rejects unknown query params** (`mcp_resources/mod.rs:2060-2064`). If a future label/preview parameter is added to the deeplink, it breaks resolution. Keep deeplinks to `?id=` only.
- **F11 — Concurrency on reject-while-recopy.** Monitor thread writes a deeplink while a reject is mid-filter on the day page. Atomic write per file mitigates but the read-modify-write in `filter_day_page_lines` is not transactional against appends.
- **F12 — `agentic_loop_five_contract.rs:31` already uses `kit://clipboard-history?id=agentic`** as a deliberately fake ID. Confirm this test still passes once the resolver's error string changes shape.

## Recommendation

Hand the implementer this delta on top of the proposed plan, in priority order:

1. **Enumerate all five delete paths and decide brain-kept protection per path.** Recommend: protect in `prune_old_entries`, `prune_impl`, `trim_oversized_text_entries`, and `clear_unpinned_history` by adding `AND brain_kept = 0` to the WHERE clause. Leave `remove_entry` (explicit user delete) and `clear_impl` (explicit "clear everything") destructive, but document that they produce dead deeplinks. Do **not** overload the user-visible `pinned` column.
2. **Add the day-page undo walker branch for `ClipboardRef`** in the same commit that introduces the variant. Reject-without-cleanup is a regression.
3. **Add a `format_source_label` branch for `kit://clipboard-history`** that returns a fixed `"Clipboard"` label, never the URI. Mirror the existing `scriptkit://clipboard/` branch.
4. **Decide image policy explicitly.** Either (a) skip brain deeplinks for `ContentType::Image` until an image-aware resolver exists, or (b) add `image/*` mime handling in `read_clipboard_history_resource`. Option (a) is the smallest shippable milestone.
5. **Persisted label contract:** `ClipboardRef` line MUST be `"<HH:MM> [Clipboard entry](kit://clipboard-history?id=<id>)"` with no preview text in the label, no fragment excerpt, no `Why:` body. Pin this with a behavior test that asserts the day-page string contains neither the entry content nor the raw `kept_url` URL.
6. **Tests/probes to add or update (behavior, not source audit):**
   - Extend `sediment_behavior_contract` (`sediment.rs:407-475`) to assert the day page contains the deeplink and **does not** contain the raw copied text or raw URL.
   - Extend `fragment_reference_card_undo_removes_day_card_and_trashes_fragment` (`substrate/mod.rs:619`) with a sibling test for `ClipboardRef` reject that asserts the deeplink line is removed.
   - Add a retention test: brain-kept row survives `prune_old_entries` with a backdated timestamp. Mirror it for `trim_oversized_text_entries` with an oversized brain-kept row.
   - Add a dead-deeplink rendering test against `read_clipboard_history_resource` with a missing ID, asserting a stable error string (regression-locks F12).
   - Add a `format_source_label` test for the `kit://clipboard-history` branch.
7. **Smallest shippable milestone (revised):** Phase 1 = `ClipboardRef` variant + label helper + undo branch + production writers (`keep_url`, `promote_recopy`) switched to deeplinks + retention guard on the four auto-delete paths + image skip. Defer legacy migration, dead-link compaction, and image deeplinks to Phase 2.
8. **Keep `scriptkit://clipboard/<id>` as fragment provenance for any legacy fragments still being written** during the transition window; do not dual-route the deeplink.

## Self Score

**8 / 10.** I grounded every claim in current source and caught five failure classes the original proposal missed (image resolver, multi-prune-path, undo walker, label leak, oversized-trim path). I did not verify the schema-migration story for the existing `brain_kept`/`brain_tier`/`copy_count` columns (where they are added), I did not inspect Notes parity for clipboard references, and I did not run any cargo verification — this is a planning critique, not a build. Score would drop to 7 if it turns out the sediment columns are added in a place I missed, because that affects F9.

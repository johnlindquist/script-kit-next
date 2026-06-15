**Recommended Plan**

Use this as the canonical persisted brain/Today URI:

```md
HH:MM [Clipboard entry](kit://clipboard-history?id=<entry_id>)
```

No extra query params. No URL label. No preview. No excerpt. No fragment body. The only persisted label should be generic, e.g. `Clipboard entry`.

Use `kit://clipboard-history?id=<id>` for new markdown because it is already resource-shaped and already resolves through the clipboard-history resource path. Do not use `scriptkit://clipboard/<id>` as the new canonical format; keep it only for legacy fragment provenance and compatibility.

**Implementation Order**

1. Add central URI helpers:
   - `src/clipboard_history/mod.rs` or `src/clipboard_history/uri.rs`
   - `entry_resource_uri(entry_id: &str) -> String`
   - `parse_entry_resource_uri(uri: &str) -> Option<String>`
   - Recognize `scriptkit://clipboard/<id>` only as legacy/provenance input, not new output.

2. Add a brain model variant:
   - `src/brain/substrate/day.rs`
   - Add `DayEntry::ClipboardRef { entry_id: String, kind: Option<ClipboardRefKind> }`
   - Formatting must emit only `[Clipboard entry](kit://clipboard-history?id=<id>)`.
   - `kind` can distinguish URL keep vs recopy internally, but must not affect persisted label with raw content.

3. Replace production clipboard sediment writers:
   - `src/clipboard_history/sediment.rs`
   - `keep_url(...)` should append `DayEntry::ClipboardRef`, not `KeptUrl`.
   - `promote_recopy(...)` should append `ClipboardRef`, not `Capture` and not `FragmentRef`.
   - Stop creating new clipboard-origin raw fragments for long recopy content.
   - `annotate_clipboard_entry` appears test-gated in the panel evidence; still update it if tests lock the old contract, but prioritize production `keep_url` and `promote_recopy`.

4. Update undo/reject behavior:
   - `src/brain/substrate/day.rs`
   - Removal must match the new clipboard URI or entry ID, not raw URL/text.
   - The Day Page undo walker needs a `ClipboardRef` branch or rejected sediment will leave deeplink lines behind.

5. Retention policy:
   - Treat `brain_kept = 1` as internal retention, separate from user-visible `pinned`.
   - Protect brain-kept rows in automatic delete paths:
     - `src/clipboard_history/database.rs::prune_old_entries`
     - `src/clipboard_history/db_worker/db_impl.rs::prune_impl`
     - `clear_unpinned_history`
     - oversized text trimming, because large copied text is exactly what may be referenced.
   - Do not claim prune alone is enough.
   - Explicit `remove_entry` and full clear can remain destructive, but they need warning/documented semantics: they may create dead brain links.
   - Clipboard IDs should be treated as stable only while the row exists. Deleted-and-recopied identical content may get a new ID, so retention must preserve the original row.

6. Day Page rendering/parsing:
   - Parse `kit://clipboard-history?id=...` as a clipboard reference.
   - Render with generic label only.
   - Missing ID should render a generic dead-link state, not stale copied content.
   - Add a `format_source_label` branch for `kit://clipboard-history` returning `Clipboard`, mirroring existing `scriptkit://clipboard/` handling, so the raw URI does not leak as a visible label.
   - Initial click behavior should be conservative: open Clipboard History focused to the entry or show a generic resolver preview. Defer copy-back/attach-to-Agent-Chat actions until the product behavior is explicit.

7. Brain index/search scope decision:
   - Audit `src/brain/indexer.rs`.
   - If “no raw values” applies to all brain storage, not only markdown, do not duplicate clipboard contents into `brain_docs`.
   - Preserve searchability by searching clipboard history as a separate provider, not by copying raw clipboard text into brain markdown or brain indexes.
   - This is the main privacy/search tradeoff.

8. Images:
   - Make milestone one text-only.
   - Current evidence says single-entry clipboard resource resolution returns `text/plain`; image entries may resolve to blob keys rather than usable image content.
   - Either skip image clipboard deeplinks initially or add image-aware resource resolution before supporting them.

9. Legacy compatibility:
   - Forward-only migration is acceptable only as a milestone-one choice: it prevents new raw writes but does not clean existing raw markdown/fragments/indexed data.
   - Keep reading old `scriptkit://clipboard/<id>` provenance.
   - Defer raw-content migration to an opt-in pass that rewrites only high-confidence matches from old raw day lines/fragments to clipboard IDs.

**Tests And Verification**

Add behavior tests, not source-audit tests unless unavoidable:

- Sediment tests: URL keep and recopy produce `kit://clipboard-history?id=...` and do not contain copied URL/text/body.
- `DayEntry::ClipboardRef` formatting test.
- Undo/reject test removes the new clipboard-ref line.
- Retention tests for `brain_kept` rows surviving prune, db-worker prune, clear-unpinned, and oversized trim.
- Day Page source-label test for `kit://clipboard-history`.
- Missing-ID resolver/rendering test.
- Keep `tests/clipboard_sediment_no_popup_contract.rs` as the no-popup sentinel.

Run cargo verification only through:

```bash
./scripts/agentic/agent-cargo.sh test --lib <focused-test-target>
```

**Smallest Shippable Milestone**

Ship text clipboard refs only: URL keep and recopy write generic `kit://clipboard-history?id=...` links, no new raw fragments are created, undo removes those links, automatic retention protects referenced rows, Day Page labels stay generic, and focused tests prove generated brain markdown contains no raw clipboard values.

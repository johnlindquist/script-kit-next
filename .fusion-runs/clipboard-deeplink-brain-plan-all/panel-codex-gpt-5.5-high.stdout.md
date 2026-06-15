## Role Findings

Use `kit://clipboard-history?id=<entry_id>` as the canonical persisted URI.

Do not use `scriptkit://clipboard/<id>` for new brain/Today persistence. It is currently provenance-shaped and legacy-local; `kit://clipboard-history?id=...` already exists as a resource URI, resolves through MCP/resource infrastructure, and is already emitted by clipboard UI/context paths. Support `scriptkit://clipboard/<id>` only as a legacy alias or provenance reader.

Persist brain markdown as:

```md
14:30 [Clipboard entry](kit://clipboard-history?id=<entry_id>)
```

No URL-as-label, no preview label, no excerpt, no fragment body, no title derived from clipboard content.

## Evidence And Assumptions

Current raw write points are centered in:

- [src/clipboard_history/sediment.rs](/Users/johnlindquist/dev/script-kit-gpui/src/clipboard_history/sediment.rs): URL keeps, annotations, and recopy promotions write raw values or raw fragments.
- [src/brain/substrate/day.rs](/Users/johnlindquist/dev/script-kit-gpui/src/brain/substrate/day.rs): `DayEntry::Capture`, `KeptUrl`, and `FragmentRef` formatting persist raw text, URL labels, and fragment excerpts.
- [src/clipboard_history/database.rs](/Users/johnlindquist/dev/script-kit-gpui/src/clipboard_history/database.rs): `brain_kept` exists, but pruning only protects `pinned`.
- [src/mcp_resources/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/mcp_resources/mod.rs): `kit://clipboard-history?id=<id>` already parses and resolves.
- [src/brain/indexer.rs](/Users/johnlindquist/dev/script-kit-gpui/src/brain/indexer.rs): pinned clipboard entries are currently copied into `brain_docs` with raw title/content. If “no raw brain values” means the whole brain DB, not only markdown, this must change too.

Assumption: clipboard history becomes the durable backing store for clipboard content. Brain/Today stores references only.

## Failure Modes

Raw leaks can still happen through non-obvious labels: `render_builtins/clipboard.rs`, `ai/context_selector/mod.rs`, Agent Chat portal labels, Day Page context serialization, and pinned clipboard brain indexing.

Links can decay if `brain_kept` rows are pruned, cleared as “unpinned,” manually deleted, or blob files are cleaned up.

Search quality will drop if brain FTS no longer indexes clipboard content. That is the privacy tradeoff. The replacement should be clipboard-history search as a separate runtime provider, not duplicated raw brain docs.

Legacy fragments are a major risk: old clipboard-origin fragments contain raw bodies. Do not create new ones; handle old ones as compatibility only.

## Recommendation

Stage 1: URI and model contract.

Add a single helper, likely in `src/clipboard_history/mod.rs` or a small `uri.rs`:

```rust
entry_resource_uri(entry_id: &str) -> String
is_clipboard_history_entry_uri(uri: &str) -> Option<String>
```

Then add `DayEntry::ClipboardRef { entry_id: String }`. Format it only as `[Clipboard entry](kit://clipboard-history?id=...)`.

Stage 2: replace sediment writes.

In `src/clipboard_history/sediment.rs`:

- `keep_url` appends `DayEntry::ClipboardRef`, not `KeptUrl`.
- `promote_recopy` appends `ClipboardRef`, never `Capture` or `FragmentRef`.
- `annotate_clipboard_entry` must not fetch content to build a brain body. If keeping `why`, persist only explicit user note, never clipboard text.
- Stop calling raw-content `record_sediment_signals(text)` for clipboard sediment, or replace it with URI/id-only signal behavior.

Stage 3: retention.

Treat `brain_kept = 1` as internal retention, separate from user-visible `pinned`.

Change automatic prune from:

```sql
WHERE pinned = 0 AND timestamp < ?
```

to:

```sql
WHERE pinned = 0 AND brain_kept = 0 AND timestamp < ?
```

Apply the same policy to `db_worker` if still active. Also update `clear_unpinned_history` semantics so brain-kept rows survive bulk clearing unless the user explicitly chooses a destructive “break brain references” action.

Stage 4: Day Page parsing/rendering.

Day Page should parse `kit://clipboard-history?id=...` as a resource/context reference with generic label `Clipboard entry`. Rendering should not auto-expand raw previews. Click behavior should open Clipboard History focused to the entry or a generic preview shell; it should not silently copy/paste.

Special-case Day Page serialization so any clipboard resource label is forced to `Clipboard entry`, even if an upstream `AiContextPart` still carries `Clipboard: <preview>`.

Stage 5: remove raw brain duplication.

If the broader contract includes brain SQLite, change `sync_pinned_clipboard` in `src/brain/indexer.rs` so pinned clipboard docs are stubs/citations, or exclude clipboard content from `brain_docs` entirely and search clipboard history directly at query time.

Stage 6: legacy.

Do forward-only first. Do not auto-migrate existing brain markdown. Add an opt-in migration later that maps old raw clipboard lines/fragments to clipboard IDs by exact content/hash and rewrites only high-confidence matches. Keep reading `scriptkit://clipboard/<id>` for old provenance.

Tests/probes:

- Update sediment tests to assert raw URL/text/long body are absent and URI is present.
- Add `DayEntry::ClipboardRef` formatting tests.
- Add prune/clear tests proving `brain_kept` rows survive.
- Add Day Page context serialization tests forcing generic clipboard labels.
- Add resource parser tests for canonical URI.
- Keep `tests/clipboard_sediment_no_popup_contract.rs` unchanged as the popup regression sentinel.
- Verify with `./scripts/agentic/agent-cargo.sh test --lib clipboard_history::sediment`, plus focused brain/day-page tests.

Smallest shippable milestone: URL keep and recopy promotion write only `[Clipboard entry](kit://clipboard-history?id=...)`, no clipboard-origin fragments are created, `brain_kept` rows survive automatic prune, and tests prove the raw copied URL/text/body is absent from generated day-page markdown.

## Self Score

8/10. The main architecture is clear and grounded in current source, but I did not inspect every Agent Chat label serialization path, so the implementer should do one focused `rg "Clipboard:" src/ai src/day_page src/render_builtins` pass before patching.



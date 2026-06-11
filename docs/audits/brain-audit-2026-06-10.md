# Brain System Edge-Case Audit — 2026-06-10

> **Status 2026-06-11:** F1–F8 and F10 fixed and proven green at runtime
> (10/10 probe `/tmp/brain-audit/brain-fixes-green-probe.ts`, receipts in
> `green-receipt.json`). F9 reclassified: the single chip on fresh-chat
> handoffs is the staged brain-recall ambient context — legitimately different
> from the resume path, documented not changed. F11/F12 remain open (P3).
>
> **Status 2026-06-11 (second pass):** F11, F12, and the remaining F5/F8
> tails fixed and proven green at runtime (`/tmp/brain-fixes2-probe.ts`,
> all checks pass against a seeded sandbox DB):
> - F12: `substring_search` LIKE fallback when the FTS leg is empty, plus
>   byte-based `min_query_chars` so "🚀"/single CJK chars are eligible.
> - F11: `record_capture_signals` re-wakes the indexer at +2s/+8s (the
>   detached capture handler races the first wake), and activity-journal
>   launcher excerpts drop the "HH:MM — " stamp.
> - F5 remainder: Enter on a non-note memory now opens a read-only
>   brain-memory preview (sessionless DivPrompt, Esc/Enter return to the
>   list); a missing chat_turn conversation routes to the same preview
>   instead of parking the raw "user: …" excerpt as a composer draft.
> - F8 remainder: mid-session inbox refreshes use `stable_merge_open_inbox`
>   (kept rows hold position, new curator items append below); the full
>   newest-first reorder only happens on window show.
> - DevTools: simulateKey gained a DivPrompt dispatcher arm (div prompts
>   were previously undriveable), and driver sandbox HOMEs symlink the real
>   `~/.scriptkit/models` (1–2 GB per session no longer re-downloaded).
> Fix summary: char-boundary-safe head slicing (F1), brain section above the
> file-CTA + selected by default (F2), short FTS terms kept (F3), capture body
> preserved through target accept (F4), memory Enter parks a context chip
> instead of auto-submitting (F5), armed `brain:` lists recent memories (F6),
> content-hash dedup in search + recents (F7), inbox resolve only after the
> route succeeds (F8), curator deferred a full interval on fresh DBs (F10).

15 user stories driven through the real app (target/debug binary, sandboxed HOME,
seeded `SCRIPT_KIT_TEST_BRAIN_DB_PATH`) via the devtools Driver, with protocol
receipts and screenshots. Probes: `/tmp/brain-audit/brain-audit-probe{,-2,-3,-4,-5}.ts`,
receipts `/tmp/brain-audit/receipt*.json`, screenshots `.test-screenshots/brain-audit/`.

## Scorecard

| # | Story | Verdict |
|---|-------|---------|
| S01 | Cold start, empty brain DB | PASS — no sections, zero error log lines |
| S02 | Natural-language recall | PASS w/ ranking finding (F2) |
| S03 | Short keyword "git" (3 chars) | FAIL — dead zone (F3) |
| S04 | FTS5 operator injection | PASS — sanitizer holds, doc still found |
| S05 | Punctuation/short-only query | PASS — section hidden gracefully |
| S06 | Stemming + case | PASS — porter matched DEPLOYED STAGES→Deploying/staging |
| S07 | CJK query | **CRASH** — SIGABRT, whole app dies (F1) |
| S07b | Emoji / accented-latin | PASS — no crash; emoji silently no-match (F12) |
| S08 | 1000-char query paste | PASS — ~1.3s incl. probe overhead, hit found |
| S09 | `brain:` filter | armed-empty state is a dead end (F6); scoping itself works |
| S10 | 3 identical docs | FAIL — all 3 dupes rendered (F7) |
| S11 | Inbox clamp/order/resolved | PASS — exactly 3, newest first, resolved hidden |
| S12 | 300-char inbox title | PASS — truncates with ellipsis, layout intact |
| S13 | Inbox Enter → handoff | PASS — resolve + auto-submit, no @cmd chip (but see F9) |
| S14 | No-source inbox item | PASS — prompt built from metadata, resolved, streams |
| S15 | `;todo` capture → searchable | prefix form loses body (F4); postfix form saves, searchable in 441ms |

## Findings (ranked)

### F1 (P0) CJK query crashes the entire app
`setFilter "ミーティング"` → panic `byte index 5 is not a char boundary` at
`src/menu_syntax/main_hint.rs:2458` (`qualifier_value_partial`:
`&token[..head_with_colon.len()]` slices at byte offset of `<head>:`).
Any first token of ≥2 CJK chars (≥6 bytes; index 5 never a boundary) aborts the
process — exit 134, `fatal runtime error: failed to initiate panic`. main_hint.rs
is unmodified in the working tree, so this ships in committed main.
Fix: `token.get(..n)?` / `is_char_boundary` guard; fuzz the hint helpers with a
multibyte corpus (CJK, emoji, accents, combining marks). CJK *recall* is
unverifiable until this is fixed.

### F2 (P1) Brain hit ranks below the Search Files fallback
Query "what branch does bluefin deploy from" with an exact-title brain doc:
selected row is `Search Files for "…"`; the memory is second. Enter sends the
user to File Search. For the app's primary feature, a strong brain match should
outrank generic fallbacks (s02-basic-recall.png).

### F3 (P1) ≤3-char terms are unsearchable lexically
`sanitize_fts_query` keeps only terms with `len() > 3` (bytes): "git", "vim",
"npm", "k8s", "aws" yield an empty FTS query → no brain section, while
`minQueryChars=3` invites 3-char queries. Control "rebase" finds the same doc.
Fix: quote-don't-drop short terms (quoting already neutralizes FTS syntax), or
prefix-match (`"git"*`).

### F4 (P1) Prefix capture form discards the typed body
`;todo Submit quarterly TPS report #work` arms the composer (fields parsed:
Task/Tags visible) but Enter activates the Todo picker row and rewrites the
input to `todo; ` — body and tags silently lost; nothing saved. The postfix form
`todo; Submit quarterly TPS report #work` + Enter saves correctly
(todos.jsonl, HUD, window hides). Picker Enter should preserve the body.

### F5 (P1) Enter-on-memory routing is inconsistent and never shows the memory
- `chat_turn` hit (conversation file missing): graceful
  `agent_chat_history_resume_fallback`, but parks the raw excerpt
  (`user: what branch does bluefin deploy from?`) as composer draft — re-sending
  your own old message is the default action.
- `activity` (and other non-note sources): **auto-submits** the bare query as a
  prompt — selecting a memory instantly burns an AI turn on "Tokyo office
  checklist" with no instruction and no visible memory content.
Neither path lets the user *read* the memory. Recommend: preview/read-only view
with explicit actions (Resume / Ask about / Copy); never auto-submit plain Enter.

### F6 (P2) `brain:` armed empty state is a dead end
`brain:` alone → "From Your Brain" header over a blank panel ("No results", 0
rows). Spine colon-modes render a ghost hint + "↓ to choose"; this surface
should match (s09-brain-filter-armed.png). `brain: q` / `brain:q` scope
correctly; `@brain:` correctly shows the won't-attach hint row.

### F7 (P2) No presentation-level dedup
3 docs with identical content fill 3 of the 4 section slots (s10). Same text
captured via clipboard + note + chat turn will triple up. Dedupe by
content_hash at projection time.

### F8 (P2) Inbox resolve is destructive and premature
Enter resolves the item *before* the agent answers; escape/failed turn = the
observation is gone (no unresolve/snooze affordance). Also the top row on empty
query can change between glance and Enter when the curator inserts a new item
(happened live mid-probe) — muscle-memory Enter can resolve something the user
never read.

### F9 (P2) Context-chip inconsistency in inbox handoffs
clipboard-sourced commitment handoff: 0 chips; no-source drift item: 1 chip.
Decide and pin the contract.

### F10 (P2) Curator runs unprompted on first launch
Fresh DB → curator due immediately → live `pi` call, inbox items appear from
2-day-old chat turns without opt-in. (Upside proven: extraction worked and
dedup collapsed 3 identical docs into one item.)

### F11 (P3) Capture is "searchable" via the activity journal, not the capture doc
441ms after capture the hit is `Activity journal · 23:26 — captured todo "…"`;
the actual capture doc lands at the next indexer cycle. Works, but the
subtitle reads like plumbing, and selecting it auto-submits (F5).

### F12 (P3) Emoji-only queries silently match nothing
unicode61 drops emoji tokens; `🚀` finds nothing, `🚀 launch` works via "launch".
Acceptable, worth documenting; emoji-as-tag workflows will fail.

## What works well
Empty-DB degradation; FTS injection sanitization; porter stemming; 1000-char
input; inbox clamp(3)/ordering/resolved-hidden; long-title truncation;
chat_turn inbox handoff auto-submit with no @cmd leak (29bda6a34 holds);
no-source items; missing-conversation resume fallback; brain recall context
staged into Agent Chat (`agent_chat_brain_recall_staged chars=1410`);
capture→signal→activity→wake pipeline (sub-second searchability); curator
end-to-end extraction + dedup, live.

## Untested / instrumentation gaps
- Semantic (embedding) path: sandbox has no GGUF → lexical-only (graceful), so
  hybrid RRF + signal-boost UI behavior is unit-tested but runtime-unverified.
- No protocol visibility into `root_brain_*` state (semantic epoch, in-flight
  request, inbox TTL) — probes rely on settle sleeps.
- `simulateKey down` does not move main-list selection (legacy routing path);
  use `batch.selectBySemanticId`.

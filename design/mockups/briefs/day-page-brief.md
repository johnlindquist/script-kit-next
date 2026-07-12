# Day Page (Today) — design-contract mockup brief

Slice for `design/mockups/screens/day-page/`, corrected and LANDED per the
2026-07-11 Oracle review (`~/.oracle/sessions/day-page-html-mockup`). Screen
owner: `src/main_sections/day_page_view.rs` (Render impl) on top of the shared
NotesEditor (`src/components/notes_editor/**`) and the brain substrate
(`src/brain/substrate/**`). Reference:
`design/mockups/screens/day-page/reference/day-page@2x.png` (captured
2026-07-11; receipt in `reference/day-page.crop.json`).

Tagging: **MEASURED** = read from source at the cited site; **PIXEL** =
verified against the 2026-07-11 capture.

---

## 0. Ownership verdict (Oracle-corrected)

The original draft proposed 25 `--sk-day-page-*` tokens. That was
over-tokenized and mis-owned. Only **five** CSS geometry tokens are genuinely
Day Page-owned; everything else consumes a shared owner. Equal numbers are
NOT equal authorities — value-coincident aliases are contract violations even
when lint-green.

Day Page-owned (the ONLY `--sk-day-page-*` variables, all Source/writable,
from `src/day_page/layout.rs`):

| Token | Value | Rust source |
|---|---|---|
| `--sk-day-page-editor-min-height` | 180 | `DAY_PAGE_MIN_EDITOR_HEIGHT_PX` (non-binding at 480, real responsive invariant) |
| `--sk-day-page-shelf-top-padding` | 6 | `DAY_PAGE_CLIPBOARD_SHELF_TOP_PADDING_PX` |
| `--sk-day-page-shelf-toggle-height` | 20 | `DAY_PAGE_CLIPBOARD_SHELF_TOGGLE_HEIGHT_PX` |
| `--sk-day-page-shelf-expanded-list-gap` | 4 | `DAY_PAGE_CLIPBOARD_SHELF_GAP_PX` (toggle↔list gap; NOT the toggle's inline `.gap_1`, a different authority that also equals 4) |
| `--sk-day-page-shelf-row-slot-height` | 24 | `DAY_PAGE_CLIPBOARD_SHELF_ROW_HEIGHT_PX` (the Day renderer's fixed row wrapper) |

Shared owners the Day Page consumes (NO Day copies):

- Content inset: `--sk-main-view-content-right-inset-x` —
  `main_view_content_columns(def).content_right_inset_x` (= shell
  `header_padding_x`, 2). **MEASURED**
- Editor wrapper padding: `--sk-notes-editor-padding-x/-y` (adopted 16/12,
  `NotesWindowStyle::current()` — the same authority the Day host passes to
  `NotesEditorLayout::new`). **MEASURED**
- Inner Input padding: `--sk-notes-editor-input-padding-x/-y` — read from the
  REAL vendored `gpui_component::Size::Medium.input_px()/input_py()`
  accessors (12/8), never copied literals. **MEASURED**
- Typography: `--sk-notes-editor-font-family` (theme bridge mono family —
  NOT `--sk-font-mono`, whose authority is `list_item::FONT_MONO`),
  `--sk-notes-editor-font-size` (16), `--sk-notes-editor-line-box-height`
  (20; vendored `Rems(1.25)` × 16px rem). There is NO separate
  "line-height" token and NO heading font-size token. **MEASURED**
- Editor text/caret: `--sk-notes-editor-text-color`,
  `--sk-notes-caret-width/-height/-color` (2×17, `theme_color.caret` ←
  `text.primary`; script-kit-dark has no focused-cursor override). **MEASURED**
- Markdown heading (edit mode IS styled — the draft's "no styled headings"
  claim was disproved by the reference and the shared highlighter):
  `--sk-notes-markdown-title-color/-font-weight`,
  `--sk-notes-markdown-heading-marker-color`. Weight + line-box clipping
  produce the emphasis; same 16px nominal size. **MEASURED + PIXEL**
- Links: `--sk-notes-editor-link-label` (bridge accent — a DIFFERENT
  authority than the markdown title color, both amber today) and
  `--sk-notes-editor-link-destination-rest` (accent through
  `markdown_link_destination_rest_color`, alpha 0.45). The mockup consumes
  the resolved rest color directly — no CSS opacity layer. The authored
  leaf `notesEditor.link.destinationCompactOpacity = 0.45` is JSON-only.
- Shelf toggle/row colors: `--sk-component-theme-muted-foreground` /
  `--sk-component-theme-foreground` (gpui-component theme via
  `map_scriptkit_to_gpui_theme`; muted = `text.primary` @
  `opacity.text_placeholder`). NOT `--sk-text-placeholder`, which merely
  resolves to the same bytes today. **MEASURED**
- Compact resource row: `--sk-compact-resource-row-padding-x/-y`
  (`INFO_SPACING.xs/xxs` = 8/4, Source) and `--sk-compact-resource-row-gap`
  (framework `.gap_2` = 8, Resolved) — owner
  `resource_preview::resolved_compact_resource_row_style`. **MEASURED**
- Framework text helpers: `--sk-framework-text-xs-font-size` (12) for the
  toggle AND row, `--sk-framework-gap-1` (4) for the toggle's inline
  glyph/label gap. **MEASURED**
- Footer band: `--sk-footer-rail-height` (32) — the GPUI Day Page spacer is
  the shared rail height. No Day duplicate. **MEASURED**

JSON-only facts (no CSS variables — markup/behavior, exported in
tokens.json): `dayPage.header.contextInteraction=inert`,
`dayPage.header.inputSlot=none`, `dayPage.header.dividerVisible=false`,
`dayPage.editor.spine.localOverlay=disabled`,
`dayPage.editor.spine.contextMentions=mainMenuRoundTrip` (both read from
`NotesEditorHostSpineContract::day_page()`),
`dayPage.shelf.defaultExpanded=false`, `dayPage.shelf.maxBodyFraction=0.4`,
`dayPage.shelf.hiddenWhenEmpty=true`,
`dayPage.shelf.hiddenDuringKitPreview=true`,
`dayPage.shelf.sourceLines=liftedFromEditor`,
`dayPage.footer.presentation=gpuiSpacerPlusNativeOverlay`,
`dayPage.footer.defaultAction=actions`,
`notesEditor.link.destinationStateRule=compactUnlessSelectionOverlapsOrTouchesFullRange`.
The triangle glyphs, pluralized shelf label, `"Clipboard entry"` fallback,
and fixture context labels are content facts, not tokens.

## 1. Surface anatomy (unchanged citations)

- Main window 750×480 via shared main-view chrome; context-only header = inert
  context row = 30 total (`padding_y·2 + 22`, with no phantom input/gap);
  `MainViewDividerChrome { visible: false }`. **MEASURED**
- Editor text origin is a COMPOSED assertion (not a token):
  x = 2 (shared inset) + 16 (editor padding) + 12 (Input padding) = **30**;
  first line-box top y = 30 (header) + 12 + 8 = **50**. Locked by the
  overlay + `day_page::layout` tests, verified by pixel receipts. **PIXEL**
- Clipboard shelf: raw `HH:MM [Clipboard entry](kit://…)` lines are lifted
  out of the editor (`adopt_clipboard_shelf_from` /
  `split_day_page_clipboard_shelf`) and rejoined on save; accessory wrapper
  `px 16 / pb 12` (`render_content_accessory`). **MEASURED**
- Footer: GPUI paints only the empty 32px rail spacer; the native overlay
  owns the buttons (plain Day Page = single `Actions ⌘K`). **MEASURED**

## 2. Canonical fixture (collapsed — locked)

Collapsed is canonical: `clipboard_shelf_expanded` initializes `false`, so
it is the shipped rest state AND the state the capture pipeline reproduces
deterministically. The 480px fixture layout assertion (locked in
`src/day_page/layout.rs` tests):

```
day_page_layout_budget(480, 32, 32, 1, false, 12)
  = body 418 / editor 380 / shelf 38 (6+20+12) / list 0
```

One-row expanded (geometry exported now, raster PENDING):

```
day_page_layout_budget(480, 32, 32, 1, true, 12)
  = list 24 / shelf 66 (6+20+4+24+12) / editor 352
```

Seeded day file (ends with a newline → the caret sits on the trailing BLANK
line, keeping the link destination deterministically at REST):

```
# Friday · ship the Day Page mockup
09:12 sketched the Day Page fixture and token list
09:31 - [ ] wire day-page tokens into export_design_tokens #design
10:02 [Script Kit](https://scriptkit.com) landing refresh notes
09:47 [Clipboard entry](kit://clipboard-history?id=day-page-mockup-seed)
```

Capture: `scripts/agentic/design-reference-capture.ts --screen day-page`
(landed) — `sandboxHome: true`, `SCRIPT_KIT_BRAIN_TZ=America/Denver`, seeds
the day file, opens via the `openDayPage` hold-gesture helper, captures
COLLAPSED. There is deliberately NO shelf click: a Day-specific
`toggleClipboardShelf` protocol command is the wrong abstraction (it would
mutate private state without proving the clickable path). When expanded
capture work starts: use `simulateGpuiEvent` mouse move/down/up at the live
inspected bounds of `day-page-clipboard-shelf-toggle`, verify a
`clipboardShelf: { count, expanded, visible }` postcondition added to
`DayPageView::automation_state`, and never derive the click from hard-coded
coordinates or the layout formula.

## 3. Resolver placement (landed)

- `src/day_page/layout.rs` (lib) now owns `DayPageLayoutBudget`,
  `day_page_layout_budget`, the five layout constants, and
  `DAY_PAGE_CLIPBOARD_SHELF_MAX_BODY_FRACTION`, with the canonical fixture
  tests; `src/main_sections/day_page_layout.rs` is a thin compatibility
  re-export for the binary.
- There is NO `resolved_day_page_editor_metrics` and no mirrored
  `INPUT_PX/INPUT_PY/LINE_HEIGHT/caret` literals in Day Page code — a Day
  resolver the renderer does not consume would be a second layout model.
  The shared NotesEditor contract reads the real vendored accessors.

## 4. Recorded conflicts (never collapse)

1. **`dayPageFooter.spacerVsNativeHostBand`** (warning) — GPUI reserves the
   32px rail while the native footer HOST band is modeled at 36. The
   2026-07-11 capture's darker bottom band starts at exactly y=448 (the
   32px edge). The ladder continues; do not "fix" either value.
2. The line-height rem question belongs to the shared NotesEditor owner
   (`notesMarkdown.titleGlyphExtentsVsLineBox` already records the heading
   clip); no Day-specific line-height conflict exists.

## 5. Blocking overlay geometry (compare.html)

Text origin x=30 / y=50 · line pitch 20 · heading same-size (weight+clip) ·
caret on the blank final line · collapsed shelf inside the 38px accessory
budget · footer band edge at y=448 · NO keycaps in the inert context row.
Expanded-row geometry is explicitly PENDING, not part of the pass claim.

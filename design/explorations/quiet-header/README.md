# A quieter top for the main menu

An interactive design exploration of the main window's top: the line above the input, the input and its subject chip, the list's section headers, and the floating capsules. Four options build on each other:

| Option | What changes |
|---|---|
| A · Today | Nothing. Rebuilt from the Rust theme (34 pt line, 34 pt input at 20 pt, 28 pt headers, 44 pt rows, gold tile) and checked against a native capture (`native-today.png`). |
| B · Say it once | Each fact has one home: the chip says what "this" is, the line says what happened (today's date and ⌘T when nothing has), and the capsule holds exactly ↵, ⌘K and ⌘↵. Sentence-case headers, no input rule, 18 pt input. |
| C · Learn as you go | B, plus hints that retire with use: words after 3 uses, keycaps after 8; holding ⌘ for 350 ms shows every key in place. ↵'s label and a waiting run's Stop never hide. |
| D · One row | C with the line folded into the input row: the front window is the input's leading icon, status sits at the right end, and the list gains 37 pt. Breaks the reducer's always-present line. |

All four share one model (rows, keys, hold timing, undo, staging), so switching keeps the typed sentence.

Open `index.html` directly in a browser; no server or build is needed.

## Driving it from an agent

`window.mockup` exposes the same model the page draws from:

- `mockup.state()` returns the variant, theme, text, selection, staged subject, last run, notice, hold and per-key use counts, plus the visible row names.
- `mockup.set(patch)` merges a patch (for example `{variant: 'D', theme: 'light', fam: 'fluent', cmd: true}`) and redraws.
- `mockup.variants` lists the option ids.

The "Jump to" buttons (`#states [data-state=fresh|ran|typing|staged|hold]`) reset to fixed states; `#ink` reports how much text, how many keycaps and how many headers are visible outside the rows.

Nothing here changes the app. Values come from `src/designs/core/main_menu_theme.rs`, `src/designs/core/slot_line_theme.rs` and `crates/sk-one-window/src/contract.rs`; colors were sampled from native captures. Icons are Lucide (ISC), copied from `vendor/script-kit-lucide-assets`.

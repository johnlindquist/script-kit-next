# One row: variations on the main menu's top

An interactive exploration that follows last round's option D in [`../quiet-header`](../quiet-header). Every preset removes the 34 pt line above the input. They differ on two independent choices, which the page also lets you mix:

| | The window behind Script Kit | What just happened |
|---|---|---|
| A · Today | Line above the input + dashed chip | The line |
| D · One row | Icon at the start of the input | End of the input row (34 pt row) |
| E · Quiet input | Icon at the start of the input | Right end of the list's first header |
| F · Show, don't tell | Its actions are the empty list's first section | First header |
| G · Try a sentence | A ghost sentence in the empty input; Tab takes it | First header |
| H · Status at the foot | Its actions as rows | The bottom-left capsule (Today at rest) |

E–H line the input up with the list: a 44 pt input row whose icon sits in the rows' icon column and whose words start where row names start. All presets share one model (rows, keys, 8 s hold, undo, staging, learning), so switching keeps the typed sentence. Holding ⌘ turns the input's icon into its ␣ key.

Open `index.html` directly; no server or build. `window.mockup.state()` / `mockup.set(patch)` drive it (for example `{preset: 'G', theme: 'light', fam: 'fluent'}`); a mix is `{custom: {this: 'rows', status: 'foot', row: 'aligned'}}`. The URL hash (`#E-dark`, `#mix-dark-rows-foot-aligned`) restores a view.

## Moving the app to one row

The reducer already separates what the line says from where it is drawn: `sk_one_window::slot::SlotLine` carries kind, icon, title, sub, hints, progress, lit and failed, and `SlotKind::Focused` is the window behind Script Kit. A one-row top is mostly a rendering change:

1. **Contract.** `contract::app::SLOT_LINE_PX` (34) and its prototype mirror go; an input-row height takes their place (44 pt if the input lines up with rows). Owners: `crates/sk-one-window/src/contract.rs`, `crates/sk-one-window/tests/contract.rs`, `src/designs/core/slot_line_theme.rs`, `src/window_resize/mod.rs` (`MAIN_WINDOW_MIN_HEIGHT`), `src/components/main_view_chrome_tests.rs`.
2. **Rendering.** `ScriptListApp::render_main_view_slot_line` (`src/app_impl/ui_window.rs`, drawn by `src/one_window/slot_line.rs`) is called by 12 app files: the launcher, arg and mini prompts, clipboard, file search, profile search, flows, the shared built-in renderer, view staging, render_impl and the day page. It splits into the input's leading tile (Focused), the chosen status home, and the hold's progress rule.
3. **Status home on views without a list.** Today's page and the chat have no list header, so "first header" needs a fallback there; the bottom-left capsule already exists natively on every view (`FooterLeftInfo` in `src/footer_popup.rs`), which is why preset H is uniform.
4. **Agent access.** Keep the semantic IDs `main-view-slot-line` and `main-view-slot-line:undo` in `getElements`, at the new location, so automation and the hold/undo proofs keep working.
5. **Proof.** `scripts/agentic/one-window/check-frames.py` currently fails any frame without a 34 pt line above the input; it would instead require the status at its home and the input still unmoved. Re-run `bash scripts/agentic/one-window/run.sh` and update the chapters' `.expect.json` for the new header/line.
6. **Rules.** `AGENTS.md` ("One Window, Intent First", "Hold, Undo, and the Line") and `GLOSSARY.md` describe the 34 pt line; they change with the code.

Values come from `src/designs/core/main_menu_theme.rs`, `src/designs/core/slot_line_theme.rs` and `crates/sk-one-window/src/contract.rs`; colors were sampled from native captures. Light-theme gold text uses a darker amber for contrast, which the app does not do today. Icons are Lucide (ISC) from `vendor/script-kit-lucide-assets`.

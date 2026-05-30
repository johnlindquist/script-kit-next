# Accent-Variation Explorer — Theme Refactor Write-up

> A live, runtime-cyclable tool for exploring **how the theme accent color is
> used** across the main menu — list rows, the native footer chrome, and the
> footer button backgrounds. Cycle treatments in the running app with
> **`alt+←` / `alt+→`**; the active treatment is named in the search placeholder.

Status: **shipped, building, unit-tested, and visually proven.** Date: 2026-05-30.

---

## 1. Goal & how it evolved

The work started from one user goal and grew through three rounds of feedback.

1. **Original goal** — "Create 9 variations of how we use our accent color in the
   main menu… cyclable live via `alt+←/→`… and get rid of the left-edge
   selected-item accent bar entirely."
2. **Round 1 feedback** — "Remove the old design variants, they're trash. Number
   the variations so I can identify them (put the name in the placeholder)."
3. **Round 2 feedback** — "WAAAAY too subtle, and the footer changes didn't work.
   Make 9 more with way less subtlety."
4. **Round 3 feedback** — "**Icon Tile** is my preference. But I also want accent
   work on the **footer**, and more variations targeting more surfaces than just
   the item highlights."
5. **Round 4 feedback** — "Make sure all **button** themes stay perfectly in sync.
   I'd like variations where the button **borders are normal** but the button
   **backgrounds** use the theme colors a little, then **hover/active** use them
   more."

The end result is a 16-variation explorer spanning three independent axes.

---

## 2. Architecture

### 2.1 The `AccentVariation` enum (new)

`src/designs/core/accent_variation.rs` (423 lines) — the single source of truth.
Exported via `crate::designs::{AccentVariation, current_accent_variation,
set_current_accent_variation, FooterButtonFill}`.

- 16 variants, `#[repr(u8)]`. **Discriminants are NOT contiguous** (1..=6, then
  14/15/16, then 7..=13) — `all()` defines the cycle order, and `from_u8()`
  round-trips by searching `all()`, so discriminant gaps are fine.
- Default + first in the cycle = **`IconTile`** (the user's preferred row look).
- Helper methods expose the three axes so consumers stay decoupled:
  - `row_kind()` — collapses every IconTile-combo variant down to the `IconTile`
    row treatment (and `FooterOnly` → plain). `ListItem::render` matches on this,
    keeping row rendering a small closed set independent of footer flags.
  - `footer_text_accent()` / `footer_keycap_accent()` / `footer_divider_accent()`
    — drive the native-footer text color, keycap-border color, and divider line.
  - `footer_button_fill() -> Option<FooterButtonFill>` — rest/hover/active accent
    **alpha bytes** for the footer button backgrounds (borders untouched).
  - `touches_footer()` — true if any footer axis is active.
  - `name()` / `description()` / `placeholder()` — identity strings; the
    placeholder reads `"Accent N/16 · Name   ·   alt+←/→ to cycle"`.

### 2.2 Cycle order (the 16 variations)

| # | Variant | Rows | Footer text | Keycap borders | Divider | Button bg |
|---|---------|------|-------------|----------------|---------|-----------|
| 1 | **IconTile** (default) | icon tile | — | — | — | — |
| 2 | IconTileFooterText | icon tile | accent | — | — | — |
| 3 | IconTileFooterKeycaps | icon tile | — | accent | — | — |
| 4 | IconTileFooterDivider | icon tile | — | — | accent | — |
| 5 | IconTileFooterFull | icon tile | accent | accent | accent | — |
| 6 | FooterOnly | plain | accent | accent | accent | — |
| 7 | FooterButtonsSoft | icon tile | — | — | — | soft |
| 8 | FooterButtonsMedium | icon tile | — | — | — | medium |
| 9 | FooterButtonsBold | icon tile | — | — | — | bold |
| 10 | SolidFill | accent-filled selected row + on-accent text | — | — | — | — |
| 11 | AccentText | accent title + icon | — | — | — | — |
| 12 | Ring | accent border ring | — | — | — | — |
| 13 | LeftBlock | chunky accent block | — | — | — | — |
| 14 | AllIcons | every row icon accent | — | — | — | — |
| 15 | AccentName | accent title + underline | — | — | — | — |
| 16 | Loud | accent rows (fill+icon+title+badges) | accent | accent | accent | — |

### 2.3 State & cycling

- `current_accent_variation: AccentVariation` lives on `ScriptListApp`
  (`src/main_sections/app_state.rs`).
- `cycle_accent_variation(forward, window, cx)` in `src/app_impl/theme_focus.rs`:
  mutates the field, **mirrors into a process-global** (see §2.5), updates the
  placeholder, pushes a toast, and `cx.notify()`s. Replaced the old `cycle_design`.
- Rows are threaded the variation explicitly via `.accent_variation(...)`
  (`render_script_list` → `render_design_item` / `ListItem`).

### 2.4 Row treatments — `ListItem::render` (`src/list_item/mod.rs`)

`render()` does `let accent_variation = self.accent_variation.row_kind();` and
then derives per-surface flags (`on_accent_text`, `icon_tile`, `ring`,
`left_block`, `name_underline_bold`, `badges_accent`, …). The old left-edge accent
bar was removed: `with_accent_bar()` is now a **no-op** guarded by a source-audit
test, and the `border_l(px(ACCENT_BAR_WIDTH))` block is gone.

### 2.5 The native footer — the hard part (`src/footer_popup.rs`)

The main-window footer is **not** a GPUI element. It is a native AppKit
`NSVisualEffectView` host (`refresh_main_footer_host`) that runs **outside the
GPUI render tree**. Two consequences shaped the design:

1. **It can't read `ScriptListApp` state.** So `cycle_accent_variation` mirrors
   the active variation into a process-global `AtomicU8`
   (`set_current_accent_variation` / `current_accent_variation`). The footer
   consults the global.
2. **It rebuilds on a color "signature."** I added the variation discriminant
   (`accent_variation: u8`) to `MainWindowFooterRefreshSignature` **and** to the
   `footer_content_changed` check, so a cycle forces a full footer content
   rebuild. The lighter visuals-only recolor path does **not** reliably reach
   every AppKit subview, so the content rebuild is required. The footer refresh
   rides the normal `sync_main_footer_popup`, which `render_impl.rs` calls every
   main render.

Accent-aware color helpers (all consult the global):
- `footer_text_hex` — footer label/hint text (accent vs `text.primary`).
- `footer_keycap_hex` + `footer_keycap_border_alpha` — keycap/labelcap borders.
- `footer_divider_rgba` — the 1px divider line (full-opacity accent).

### 2.6 Footer **button backgrounds** — the in-sync refactor

Every footer button (cwd chip, model chip, Agent chip, Run, Actions) is built by
`make_footer_hint_button` and has a container layer with **rest / hover / active**
backgrounds set at **five** native sites. Before this work the *selected* state
special-cased Actions (`hover_rgba`) vs everything else (`selection_rgba`), so the
buttons were subtly out of sync. I centralized all five sites through shared
accent-aware helpers:

| Site | Function | Helper |
|------|----------|--------|
| Build (rest + selected) | `make_footer_hint_button` | `footer_button_rest_fill_rgba`, `footer_button_active_fill_rgba` |
| Hover enter | `footer_button_mouse_entered` | `footer_button_hover_fill_rgba` |
| Hover exit (restore) | `footer_button_mouse_exited` | `footer_button_active_fill_rgba_for_actions` / rest |
| Actions mouse-down | `footer_button_mouse_down` | `footer_button_active_fill_rgba` |

When a `FooterButtons*` variation is active, these override the per-action theme
defaults with **one shared accent tint** at `(accent_hex << 8) | fill.{rest,hover,
active}` — so all buttons react identically. Non-fill variations keep the existing
per-action behavior (intentionally, to avoid regressing default UX).

`FooterButtonFill` intensities (alpha bytes over the accent hue):

| State | Soft | Medium | Bold |
|-------|------|--------|------|
| rest | `0x0F` | `0x1C` | `0x2B` |
| hover | `0x24` | `0x36` | `0x52` |
| active | `0x36` | `0x52` | `0x73` |

Borders and label text stay neutral — only the background layer is tinted, per
the "borders normal, backgrounds use the theme colors" request. The primary Run
button is `selected`, so it shows the *active* fill at rest — a convenient
rest-vs-active contrast in a single screenshot.

---

## 3. Three keyboard routing paths (a reusable lesson)

A new main-menu keybinding may need wiring in **three independent** key paths, and
`simulateKey` automation only exercises one:

1. **`handle_key`** on-key-down listener in `render_script_list/mod.rs` — real OS
   keystrokes.
2. **Global `arrow_interceptor`** in `app_impl/startup.rs`
   (`cx.intercept_keystrokes`) — fires before the focused input eats arrows;
   calls `cx.stop_propagation()`.
3. **Legacy `simulateKey` dispatch** in `app_impl/simulate_key_dispatch.rs` — the
   **only** path the stdin `{"type":"simulateKey",…}` automation reaches.

`alt+←/→` is wired in all three so it works for users *and* is provable via the
screenshot harness.

---

## 4. Key discoveries / gotchas

- **The accent is theme-dependent and often soft** — e.g. Catppuccin Mauve
  `#cba6f7`, later an amber, never the assumed yellow. At the footer's muted
  `HINT_TEXT_OPACITY` a soft accent is invisible. **Fix:** accent footer text
  renders at **full opacity (1.0)**, keycap borders at a fixed **0.9** alpha, the
  divider at full-opacity accent. Without the alpha boost the change is
  imperceptible — and the user explicitly rejects subtlety. (Diagnostics proved
  the *color* was correct all along; only the alpha was hiding it.)
- **Footer ≠ GPUI.** Earlier notes claiming "footer accent doesn't work" were
  about not realizing the footer is a separate native surface. It works once you
  drive it through the global + signature rebuild (§2.5).
- **Screenshot capture lags the cycle by ~one frame** (capture is async). The
  in-image **placeholder text** is authoritative for which variation a PNG shows.
- **`WideWash`** (a row variant from the bold pass) was removed when the explorer
  was reorganized around footer axes.

---

## 5. Files changed

| File | Δ | What |
|------|---|------|
| `src/designs/core/accent_variation.rs` | **new, 423 lines** | the enum, axes, `FooterButtonFill`, global, tests |
| `src/designs/core.rs` | +8 | re-exports |
| `src/designs/core/render.rs` | +3/-1 | thread variation into `render_design_item` |
| `src/list_item/mod.rs` | +248/-… | row treatments via `row_kind()`, bar removal |
| `src/footer_popup.rs` | +188/-… | native footer + button-background accent |
| `src/render_script_list/mod.rs` | +34 | thread variation, `alt+←/→` in `handle_key` |
| `src/app_impl/theme_focus.rs` | +82 | `cycle_accent_variation`, global mirror |
| `src/app_impl/startup.rs` | — | interceptor branch, placeholder init |
| `src/app_impl/simulate_key_dispatch.rs` | — | automation branch (path 3) |
| `src/app_impl/{lifecycle_reset,registries_state}.rs` | — | placeholder reset uses active variation |

Also removed: `DesignVariant` cycling (Cmd+1 / `cycle_design`) — the type is pinned
to `Default` but kept, since ~40 files depend on its tokens/resolvers.

---

## 6. Verification

- **Unit tests** — `./scripts/agentic/agent-cargo.sh test --lib accent_variation`
  → **8/8 pass**: exactly-16 + discriminant round-trip, forward/backward cycle
  wrap, names/placeholders non-empty (`/16`), default is IconTile, footer-combo
  rows map to IconTile, footer flags match intent, global u8 round-trip, and
  button-fill intensities increase (`rest<hover<active`, `Soft<Medium<Bold`) with
  neutral borders.
- **Build** — `agent-cargo.sh build --bin script-kit-gpui`, clean (no warnings).
- **Live visual proof** — `.test-screenshots/accent-footer/`:
  - `bold-0{2,4,5,6}-*.png` — footer text / divider / full / footer-only.
  - `btn-0{7,8,9}-{soft,medium,bold}.png` — button-fill intensities.
  Driven via `session.sh` stdin protocol: `show` → `simulateKey option+right` →
  `captureWindow`.

### Repro (live screenshot harness)

```bash
S=accent
bash scripts/agentic/session.sh start "$S"
bash scripts/agentic/session.sh send "$S" '{"type":"show"}'
# cycle one step
bash scripts/agentic/session.sh send "$S" '{"type":"simulateKey","key":"right","modifiers":["option"]}'
# capture (in-image placeholder names the variation)
bash scripts/agentic/session.sh send "$S" '{"type":"captureWindow","title":"","path":"/tmp/shot.png","requestId":"x"}'
bash scripts/agentic/session.sh stop "$S"
```

---

## 7. Known caveats / follow-ups

- **Hover state isn't screenshot-provable** — the protocol can't synthesize a
  mouse-into-button move. Hover is wired (`footer_button_mouse_entered`) and sits
  between rest and active per the `FooterButtonFill` table; verify by hand.
- **Default (no-accent) footer buttons are still per-action** — the in-sync
  unification only applies when a `FooterButtons*` variation is active. Unifying
  the *default* footer too is a one-line follow-up if desired.
- **`DesignVariant`** type was retained (only its cycling was disabled); a full
  removal is out of scope (~40 dependents).
- **Concurrent edits** — during this work a separate process left
  `src/scripts/search/ghost.rs` mid-edit (test-only `GhostContext` initializers),
  which intermittently broke the shared lib-test target. Not part of this change.

---

## 8. Related memory

- `project_accent_variation_explorer.md` — the durable project memory for this tool.
- `reference_three_keyboard_routing_paths.md` — the §3 lesson.
- `reference_footer_blur_architecture.md` — NON-NEGOTIABLE footer blur invariants
  (untouched here: this work only sets background/border/text colors, never the
  effect view / hitTest / transparent-hitbox pieces).

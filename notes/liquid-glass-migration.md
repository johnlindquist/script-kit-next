# Liquid Glass Migration Plan

Goal: make `NSGlassEffectView` (macOS 26 Tahoe Liquid Glass) the real window
backdrop for Script Kit's panels, replacing the NSVisualEffectView
(BlurredView) vibrancy stack — without regressing the measured legibility +
color targets, pre-Tahoe machines, or the footer blur trio.

Status: PLANNED. Authored 2026-06-10 after the vibrancy recipe shipped in
`f4cca5f55` (menu material + 0.30 tint + backdrop_saturation 2.6).

## Current truth (measured 2026-06-10)

The main window composites, bottom to top:

1. `NSGlassEffectView` backdrop — installed by `configure_tahoe_window_backdrop`
   (`src/platform/secondary_window_config.rs`), tag `0x5c17_0175`, backmost in
   `contentView`, tinted via `liquid_glass_tint_color()` =
   `main_window_matched_background_rgba` (alpha = `vibrancy_background`, 0.30),
   `cornerRadius` currently **0.0** (logged `corner_radius=0.0`).
2. GPUI `BlurredView` (an NSVisualEffectView subclass) — exists because window
   creation requests `WindowBackgroundAppearance::Blurred` when vibrancy is
   enabled (`src/main_entry/app_run_setup.rs`).
   Configured by `configure_visual_effect_views_recursive`
   (`src/platform/vibrancy_config.rs`): menu material, VibrantDark/Light,
   behind-window blending, emphasized+active in dark.
3. GPUI Root tint — `map_scriptkit_to_gpui_theme` in
   `src/theme/gpui_integration.rs`, alpha = `OPACITY_VIBRANCY_BACKGROUND` 0.30.

**Measured: the glass layer contributes ZERO visible pixels** — baseline and
glass-skipped captures are pixel-identical; the BlurredView's behind-window
material fully covers it. Glass has only ever been observed under a heavy
0.85 tint (where it read grey); it has **never been measured lightly tinted as
the sole backdrop** — that unknown is Phase 1.

The saturation boost (`vibrancy.backdrop_saturation`, default 2.6) is applied
in the `patched_update_layer` swizzle
(`src/platform/vibrancy_swizzle_materials.rs`) by mutating the existing
`colorSaturate` filter on the BlurredView's `CABackdropLayer` (menu material
ships 2.4). **NSGlassEffectView exposes no public saturate knob** — whether
glass needs one, has its own backdrop layer to filter, or natively retains
enough chroma is a Phase 1 measurement, and the migration's kill criterion.

## Acceptance targets (every phase gate re-measures these)

Derived from the shipped recipe's receipts
(`.test-screenshots/vibrancy-new-defaults-{white,rainbow}.png`):

- **White backdrop**: panel body luminance ≤ ~32% (recipe: 30% retina / 37% on
  the 1x ultrawide), neutral hue, white text legible.
- **Rainbow backdrop**: panel body saturation ≥ ~35% (recipe: 36%) at body
  luminance ≤ ~30% — the Raycast-style hue glow.
- **Footer matches body** — no luminance/saturation divergence between the
  panel body and the floating ↵ / ⌘K capsule.
- **Pre-Tahoe / engine=vibrancy unchanged** — pixel-equal to the `f4cca5f55`
  receipts.

Measurement protocol (non-negotiable, learned the hard way):
- Backdrop windows on **ALL displays** (panel shows on the mouse's display).
- Crops derived from per-step window-bounds receipts, never fixed offsets.
- On-screen window receipts via the compiled C lister
  (`/tmp/list-sk-windows.c` — check it into `scripts/agentic/native/` as part
  of Phase 0; JXA CGWindowList bridging segfaults and reads as a false "no
  other windows").
- Probes: `scripts/agentic/vibrancy-{ladder,material-grid,layer-isolation}-probe.ts`
  + `scripts/agentic/vibrancy-measure.ts`.

---

## Phase 0 — `backdrop_engine` plumbing (no visual change)

| File | Change |
|---|---|
| `src/theme/types.rs` | `enum BackdropEngine { #[default] Vibrancy, LiquidGlass }` (serde snake_case) + `engine: BackdropEngine` field on `VibrancySettings` (`#[serde(default)]`). Doc comment: LiquidGlass silently degrades to Vibrancy where `NSGlassEffectView` is unavailable. |
| `src/theme/validation.rs` | add `"engine"` to `KNOWN_VIBRANCY_KEYS`. |
| `src/platform/secondary_window_config.rs` | `pub fn resolved_backdrop_engine() -> BackdropEngine` — theme engine, degraded to Vibrancy when `tahoe_liquid_glass_class()` is None; env override `SCRIPT_KIT_BACKDROP_ENGINE=vibrancy\|liquid-glass` for probes (mirrors `SCRIPT_KIT_VIBRANCY_SATURATION`). |
| `tests/theme/snapshots/theme_{dark,light}_default.json` | gains `"engine": "vibrancy"`. |
| `scripts/agentic/native/list-sk-windows.c` | check in the window-receipt tool + a build line in the probes that compiles it to /tmp on demand. |

Gate: `agent-cargo.sh test --lib -- theme:: platform::`; snapshots updated;
no runtime behavior change.

## Phase 1 — Measurement spike: can glass alone hit the targets? (GO / NO-GO)

No product code ships from this phase; it answers the kill criterion.

| File | Change |
|---|---|
| `src/platform/secondary_window_config.rs` | `liquid_glass_tint_color()` honors debug env `SCRIPT_KIT_DEBUG_GLASS_TINT_ALPHA` so the glass tint can be swept independently of the shared 0.30 token. |
| `src/platform/vibrancy_swizzle_materials.rs` | extend `dump_layer_hierarchy` logging to also dump the glass view's layer tree once (does glass own a `CABackdropLayer` we could filter?). |
| `scripts/agentic/vibrancy-glass-grid-probe.ts` (new) | matrix runner: `SCRIPT_KIT_DEBUG_HIDE_VEV=1` (glass becomes sole backdrop) × glass tint α ∈ {0, 0.15, 0.30} × root veil (`vibrancy_background` via sandbox theme.json) ∈ {0, 0.15, 0.30}, over white + rainbow on all displays, bounds-receipt crops. |

Also measure in this phase:
- **Accessibility**: System Settings → Reduce Transparency ON — confirm glass
  falls back sanely (and what our hidden-VEV branch shows).
- **Light mode** one cell (light appearance over a dark app).
- **Live-resize feel** (drag-resize the panel; glass refraction is the
  expensive part).

GO if some cell hits white ≤32% lum AND rainbow ≥35% sat. NO-GO → park the
migration, keep the glass install as today (occluded), document the failing
numbers here.

## Phase 2 — Main window: engine branch (glass visible, VEV hidden)

The hide-VEV mechanism is already proven (`SCRIPT_KIT_DEBUG_HIDE_VEV`); this
phase makes it the engine path.

| File | Change |
|---|---|
| `src/platform/vibrancy_config.rs` | in `configure_window_vibrancy_material_for_appearance`: when `resolved_backdrop_engine() == LiquidGlass`, set each found NSVisualEffectView hidden (instead of material config); include the engine in `LAST_MAIN_WINDOW_VIBRANCY_SIGNATURE` so theme-driven engine flips reconfigure. Keep window appearance-nil / shadow / non-opaque handling identical. |
| `src/platform/vibrancy_swizzle_materials.rs` | `patched_update_layer`: skip the saturation boost and chameleon handling when engine is LiquidGlass (the BlurredView still receives updateLayer while hidden). |
| `src/platform/secondary_window_config.rs` | `configure_tahoe_window_backdrop`: set `cornerRadius` from the window radius token (22pt — `src/ui/chrome/tokens.rs`; today it logs 0.0), and use the Phase-1 winning glass tint alpha (new token, see next row) instead of the shared 0.30. Install the glass view only when the resolved engine is LiquidGlass. |
| `src/theme/opacity.rs` | `OPACITY_GLASS_TINT` + `OPACITY_GLASS_ROOT_VEIL` tokens = the Phase-1 winning cell, with the measurement table in the doc comment (same pattern as `OPACITY_VIBRANCY_BACKGROUND`). |
| `src/theme/gpui_integration.rs` | `main_bg` alpha: engine-aware — `OPACITY_GLASS_ROOT_VEIL` under LiquidGlass, `vibrancy_background` under Vibrancy. |
| `src/ui_foundation/mod.rs` | `resolve_window_vibrancy_opacity` engine-aware the same way, so every native tint that reads `main_window_matched_background_rgba` stays matched to the main window. |

Gate: sandbox theme.json with `"vibrancy": {"engine": "liquid_glass"}` hits the
acceptance targets over white + rainbow; engine=vibrancy run is pixel-equal to
the `f4cca5f55` receipts; both with single-instance window receipts.
Rollback: flip the engine field — hot-reloads via the theme watcher.

## Phase 3 — Stop creating the BlurredView at all (cleanup, keeps GPUI un-vendored)

| File | Change |
|---|---|
| `src/main_entry/app_run_setup.rs` | when the resolved engine at window-creation time is LiquidGlass, request `WindowBackgroundAppearance::Transparent` instead of `Blurred` (no BlurredView is ever built). |

| `src/platform/vibrancy_config.rs` | the Phase-2 hide loop stays (it covers runtime engine flips on already-created windows; document that a flip to/from LiquidGlass fully applies on next window create). |

Gate: same receipts as Phase 2 plus a resize-feel pass. No vendored
`crates/gpui` changes anywhere in this plan.

## Phase 4 — Footer blur trio (HIGHEST RISK — the trio is non-negotiable)

The ↵ and ⌘K capsule is a native view floating inside the main window's glass
stage (`src/footer_popup.rs`), not a window, with a custom NSVisualEffectView subclass
(registered ~line 4331) whose contract is: **native blur view + `hitTest`
delegation + deferred transparent hitbox** — the three move together or not at
all (see memory `reference_footer_blur_architecture`). Additional invariant:
capsule sync stays deferred (`cx.defer`); vendored GPUI still refuses `open_window`
during draw (SIGSEGV).

| File | Change |
|---|---|
| `src/footer_popup.rs` | register a parallel subclass over `NSGlassEffectView` (same runtime-resolved superclass trick as `tahoe_glass_backdrop_view_class`), implementing the identical `hitTest` delegation. The capsule build instantiates VEV-or-glass by resolved engine. Glass tint + corner radius from the Phase-2 tokens / footer radius. |
| `src/app_impl/ui_window.rs` | no contract change — verify the deferred capsule sync path is untouched; the engine only swaps which native class backs the strip. |
| `src/theme/chrome.rs` | `popup_surface_rgba`: engine-aware alpha if Phase-2 measurements show the popup/footer strip diverging from the body under glass. |

Gate: capsule vs body luminance/saturation match over white + rainbow;
`bun scripts/devtools/actions.ts inspect --prove-shortcut-open-freshness
--prove-escape-close-cleanup`; real-hotkey repro (`osascript key code 41`)
shows no SIGSEGV; capsule hitbox still passes clicks through.

## Phase 5 — No secondary windows

Every view (notes, chat, voice, ⌘K actions, questions, holds) is in the main
window under the same input, so Phases 2–4 cover them; there is no separate
window to migrate. The main window and the snap overlay are the only windows the
app opens.


## Phase 6 — Theme designer, cycling, docs

| File | Change |
|---|---|
| `src/render_builtins/theme_chooser.rs` | engine toggle in the designer; show the material picker + `backdrop_saturation` slider only for engine=vibrancy (neither applies under glass); persist via the existing `VibrancySettings` write path. |
| `src/platform/vibrancy_cycle.rs` | material cycling no-ops (or cycles glass tint) under LiquidGlass. |
| `src/app_layout/collect_elements.rs` | expose the resolved engine in element/state receipts so probes can assert it. |
| `GLOSSARY.md` | "Backdrop engine" entry pointing at the owning files. |
| `tests/theme/snapshots/*`, `src/theme/validation.rs` | re-sync. |
| this file | record Phase-1 numbers and the winning cell. |

## Phase 7 — Default flip on Tahoe

- `VibrancySettings::default()` engine → `LiquidGlass` (resolution still
  degrades to Vibrancy pre-26 via the `NSClassFromString` gate, so old
  machines and old theme.json files are untouched).
- Run an AFK visual pass across the main window's views (launcher, a note, ⌘K
  staged, the model's chat, voice, a hold in the line) per `feedback_afk_loop_user_focus` (oracle → implement → screenshot
  receipts into `.test-screenshots/`).
- Keep engine=vibrancy fully supported for at least one release as the escape
  hatch; only delete the VEV path (and the saturation swizzle) when glass has
  survived real use.

## Risk register

| Risk | Mitigation |
|---|---|
| Glass can't hit chroma targets (no saturate knob) | Phase 1 is the kill gate; NO-GO leaves shipped recipe in place. |
| Footer trio regression | Phase 4 touches all three contract pieces together; devtools prove-* + hotkey SIGSEGV repro before merge. |
| `open_window` during draw SIGSEGV | No changes to capsule sync timing; it stays `cx.defer`. |
| Reduce Transparency / increased contrast | Measured explicitly in Phase 1; engine resolution may need to degrade to Vibrancy when set. |
| Light mode unvalidated | One light-mode cell in every phase gate. |
| Live-resize perf on 4K/ultrawide | Phase 1 feel-check + compare resize frame times before flipping the default. |
| Probe false confidence | All-display backdrops, bounds-receipt crops, C-lister window receipts, terminal `screencapture` (app TCC dies on rebuild). |
| Stale binary verification | Build via `SCRIPT_KIT_AGENT_ARTIFACT_NAME=<task> ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui` and launch the artifact path, never `target/debug`. |

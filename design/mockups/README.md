# design/mockups — HTML↔Rust design contract

Pixel-faithful HTML mockups of every Script Kit screen, generated from and
verified against the Rust implementation. **Rust is the single authority.**

## How it works

```
src/design_contract/            Rust: resolves the SAME values production
                                rendering paints (shared resolvers in
                                list_item, effects) into a typed bundle
src/bin/export_design_tokens    writes generated/tokens.{json,css}
generated/tokens.css            :root { --sk-* } — the ONLY source of visual
                                values for mockups (never hand-edit)
shared/components.css           CSS components mapping 1:1 to Rust component
                                boundaries (list_item, footer_chrome, …)
screens/<screen>/index.html     per-screen fixture with real content
screens/<screen>/compare.html   onion-skin/difference overlay vs. a real
                                app capture (reference/ + crop receipt)
workbench/<screen>.edits.json   design proposals (HTML→Rust direction)
tests/lint-mockups.mjs          fails on any literal visual value in
                                hand-written CSS
```

Regenerate tokens after any theme/token change:

```bash
./scripts/agentic/agent-cargo.sh run --bin export_design_tokens -- design/mockups/generated
node design/mockups/tests/lint-mockups.mjs
```

The `design_contract` lib test (`checked_in_bundle_matches_renderer_resolution`)
locks the exporter to the renderer's resolved bytes.

## Token stages

- **source** — authored Rust leaves (writable; edits round-trip via
  `workbench/*.edits.json`).
- **resolved** — post-quantization values the renderer actually paints
  (read-only; e.g. selected row = `#FFFFFF20`, not `theme.opacity.selected`).
- **emulator** — `--sk-emulator-*` browser-only calibration (blur radii,
  material bias, synthetic desktop). Never maps back to Rust.

## What "pixel-perfect" means here

1. **Token-perfect** — every visual value comes from the generated contract.
2. **Geometry-perfect** — edges/heights within 0.5 logical pt of the capture.
3. **Raster-calibrated** — pixels match under a fixed backdrop/scale/motion;
   vibrancy, starfield, native-footer and glyph rasterization are recorded in
   `known-divergence.json` per screen and are non-blocking.

## Recorded conflicts

`generated/tokens.json → conflicts` records places where live code paths
disagree (e.g. themed 44px rows vs. the legacy 40px constant; component
selected-fill `0x20` vs. `theme.opacity.selected` 0.20). The exporter
surfaces drift — it never silently picks a side.

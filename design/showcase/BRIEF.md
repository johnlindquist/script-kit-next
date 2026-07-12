# Showcase shot recreation brief

Goal: recreate one reference screenshot (`design/showcase/reference/<id>.jpg`)
as a **pixel-perfect, live HTML/CSS scene** at
`design/showcase/shots/<id>/index.html`. The published landing page embeds the
scene in an `<iframe>` sized to the same aspect ratio.

## Scene contract

- The reference JPG is a 2x (retina) capture. The scene's logical size is
  `jpg_width/2 × jpg_height/2` CSS px (e.g. 1675×1139 → 837.5×569.5 — round
  to 838×570 and keep proportions inside).
- Page skeleton:
  ```html
  <!doctype html><html><head><meta charset="utf-8">
  <link rel="stylesheet" href="../../shared/tokens.css">
  <link rel="stylesheet" href="../../shared/components.css">  <!-- optional -->
  <style>/* scene-local styles */</style></head>
  <body><div class="scene">…</div>
  <script src="../../shared/fit.js"></script></body></html>
  ```
- `html,body{margin:0;padding:0;overflow:hidden;background:#000}`.
  `.scene{position:relative;width:<W>px;height:<H>px;overflow:hidden}`.
  fit.js scales `.scene` to the iframe — design at fixed px, never responsive.

## Fidelity method (follow this order)

1. **Study the reference**: `Read` the JPG. Then measure precisely with
   Python PIL (installed): sample colors, find edges (luminance scans),
   measure the window rect, row pitch, font sizes (cap-height ≈ 0.7×font-size),
   paddings. Divide pixel measurements by 2 for CSS px.
2. **Transcribe ALL text exactly** — every label, row, keycap, pill, count.
   Do not invent or paraphrase content.
3. **Reuse the existing pixel contract** where the shot shows a Script Kit
   window: `design/showcase/shared/tokens.css` has `--sk-*` values exported
   from the Rust renderer (row heights, radii, fonts, colors, footer metrics);
   `design/showcase/shared/components.css` has `.sk-window`, `.sk-header`,
   `.sk-context-zone`, `.sk-list-row`, `.sk-footer-*` etc. Study the nearest
   existing mockup in `design/mockups/screens/<surface>/{index.html,screen.css}`
   and copy its anatomy. Values with no token: hardcode the measured value
   (this directory has NO lint).
4. **Desktop backdrop**: recreate the wallpaper behind the window with CSS
   gradients — sample 8-12 points from the JPG corners/edges and approximate.
   The app window emulates vibrancy: translucent dark fill + `backdrop-filter:
   blur(…) saturate(…)` over the backdrop (see how
   `design/mockups/screens/main-menu/screen.css` does material emulation).
   The window has `border-radius` ~10-12px (measure), a 1px light border,
   and a large soft drop shadow.
5. **Icons**: inline SVG approximations (stroke style like Lucide). Colored
   app icons (Codex, Finder, etc.): rounded-square with gradient approximation.
6. Use system font stack: `-apple-system, BlinkMacSystemFont, "SF Pro Text"`.
   Mono per tokens (`--sk-font-mono`, JetBrains Mono → fallback ui-monospace).

## Verification loop (mandatory — iterate until converged)

```bash
W=838 H=570 ID=<id>   # logical dims for this shot
agent-browser --session shot-$ID set viewport $W $H 2
agent-browser --session shot-$ID open "file:///Users/johnlindquist/dev/script-kit-gpui/design/showcase/shots/$ID/index.html"
agent-browser --session shot-$ID screenshot body /tmp/shot-$ID-render.png
# render.png is DPR-2 → 2W×2H physical. Normalize the reference to the SAME canvas:
magick design/showcase/reference/$ID.jpg -resize $((W*2))x$((H*2))! PNG24:/tmp/shot-$ID-ref.png
magick compare -metric RMSE /tmp/shot-$ID-render.png /tmp/shot-$ID-ref.png /tmp/shot-$ID-diff.png; echo
```

Then **Read** the render, the reference, and the diff side by side. Fix the
biggest structural mismatch first (window rect, header band, row pitch), then
color/opacity, then text/icon detail. Repeat at least 3 cycles; stop when the
remaining diff is dominated by font antialiasing / photographic wallpaper
detail (receipt those).

When done:
- `agent-browser --session shot-$ID close`
- Write `design/showcase/shots/<id>/receipt.json`:
  `{ "id", "sceneSize": [W,H], "rmse": <final normalized RMSE>,
     "divergences": ["wallpaper approximated with CSS gradients", …],
     "iterations": N }`

## Hard rules

- Work ONLY inside `design/showcase/shots/<id>/` (plus /tmp scratch).
- No external network assets — everything inline or from ../../shared/.
- No screenshots/JPGs inside the scene (the whole point: DOM, not bitmap).
  Exception: none. Wallpaper must be CSS.
- The scene must render correctly static — no JS required except fit.js.

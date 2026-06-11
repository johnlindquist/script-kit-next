/**
 * Measure panel-body luminance/saturation from grid-probe receipts.
 *
 * Reads receipt lines (one JSON per step, with `windows` global CG bounds
 * in points) on stdin, locates the main panel window (h > 100), maps it to
 * the right display capture file and pixel offset, crops a body region
 * safely inside the panel (away from footer and edges), and prints metrics.
 *
 * Usage: bun vibrancy-measure.ts <capture-prefix> < receipts.ndjson
 */
// Display map (from NSScreen frames, AppKit coords converted to CG top-left):
//   main:      CG origin (0, 0),      1512x982 pt, 2x  -> file -d1
//   4k:        CG origin (1512,-1080),1920x1080 pt, 2x -> file -d2
//   ultrawide: CG origin (-1928,-1440+982-982...) -- resolved below
// Ultrawide AppKit frame: origin (-1928, 982) size 3440x1440 (1x assumed).
// CG top-left y = mainH - (originY + height) = 982 - (982 + 1440) = -1440.
const DISPLAYS = [
  { name: "d1", x: 0, y: 0, w: 1512, h: 982, scale: 2 },
  { name: "d2", x: 1512, y: -1080, w: 1920, h: 1080, scale: 2 },
  { name: "d3", x: -1928, y: -1440, w: 3440, h: 1440, scale: 1 },
];

const prefix = process.argv[2];
if (!prefix) throw new Error("usage: vibrancy-measure.ts <capture-prefix>");

const text = await Bun.stdin.text();
const lines = text
  .split("\n")
  .map((l) => l.trim())
  .filter((l) => l.startsWith("{"));

for (const line of lines) {
  const step = JSON.parse(line);
  if (!step.windows) continue;
  const panel = step.windows.find((w: any) => w.h > 100);
  if (!panel) {
    console.log(`${step.tag}: NO PANEL WINDOW`);
    continue;
  }
  const cx = panel.x + panel.w / 2;
  const cy = panel.y + panel.h / 2;
  const disp = DISPLAYS.find(
    (d) => cx >= d.x && cx < d.x + d.w && cy >= d.y && cy < d.y + d.h,
  );
  if (!disp) {
    console.log(`${step.tag}: panel center (${cx},${cy}) not on a known display`);
    continue;
  }
  const px = (panel.x - disp.x) * disp.scale;
  const py = (panel.y - disp.y) * disp.scale;
  const pw = panel.w * disp.scale;
  const ph = panel.h * disp.scale;
  // Body region: inside the panel, clear of header (top ~25%), footer
  // overlay (bottom ~15%), and side edges (10% margins).
  const bx = Math.round(px + pw * 0.45);
  const by = Math.round(py + ph * 0.35);
  const bw = Math.round(pw * 0.45);
  const bh = Math.round(ph * 0.45);
  const file = `/tmp/${prefix}-${step.tag}-${disp.name}.png`;

  const crop = Bun.spawnSync([
    "magick",
    file,
    "-crop",
    `${bw}x${bh}+${bx}+${by}`,
    "+repage",
    `/tmp/${prefix}-${step.tag}-body.png`,
  ]);
  if (crop.exitCode !== 0) {
    console.log(`${step.tag}: crop failed: ${crop.stderr.toString().slice(0, 120)}`);
    continue;
  }
  const sat = Bun.spawnSync([
    "magick", `/tmp/${prefix}-${step.tag}-body.png`,
    "-colorspace", "HSL", "-channel", "G", "-separate",
    "-format", "%[fx:mean*100]", "info:",
  ]).stdout.toString();
  const lum = Bun.spawnSync([
    "magick", `/tmp/${prefix}-${step.tag}-body.png`,
    "-colorspace", "gray", "-format", "%[fx:mean*100]", "info:",
  ]).stdout.toString();
  const avg = Bun.spawnSync([
    "magick", `/tmp/${prefix}-${step.tag}-body.png`,
    "-resize", "1x1!", "-format", "%[pixel:p{0,0}]", "info:",
  ]).stdout.toString();
  console.log(
    `${step.tag} [${disp.name}]: lum=${Number(lum).toFixed(1)}% sat=${Number(sat).toFixed(1)}% avg=${avg}`,
  );
}

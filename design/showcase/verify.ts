#!/usr/bin/env bun
/**
 * Consolidated showcase verification: render every shot page at its exact
 * scene size (DPR 2) and diff against the original landing screenshot.
 * Writes design/showcase/verify-summary.json and per-shot diff artifacts
 * under .test-output/showcase-verify/.
 *
 * Usage: bun design/showcase/verify.ts [shot-id ...]
 */
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = new URL(".", import.meta.url).pathname.replace(/\/$/, "");
const PROJECT_ROOT = join(ROOT, "../..");
const OUT = join(PROJECT_ROOT, ".test-output/showcase-verify");

const SHOTS: Record<string, [number, number]> = {
  "01-main-launcher": [1675, 1139],
  "02-search-filter": [1675, 1139],
  "04-clipboard-history": [1675, 1139],
  "05-emoji-picker": [1675, 1139],
  "06-notes": [1560, 1351],
  "07-day-page": [1675, 1139],
  "08-agent-chat": [1675, 1139],
  "09-terminal": [1675, 1139],
  "10-file-search": [1675, 1139],
  "11-theme-designer": [1675, 1139],
  "12-settings": [1675, 1139],
  "13-agent-chat-composer": [1675, 1139],
  "14-window-switcher": [1675, 1139],
  "15-app-launcher": [1675, 1139],
  "17-rewrite": [2150, 1294],
  "18-rewrite-styles": [2150, 1294],
  "19-references": [1573, 335],
  "20-brain-inbox": [1675, 1139],
  "21-dictation": [1428, 531],
};

const SESSION = `showcase-verify-${process.pid}`;

async function browser(args: string[]): Promise<string> {
  const proc = Bun.spawn(["agent-browser", "--session", SESSION, ...args], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  if (code !== 0) throw new Error(`agent-browser ${args.join(" ")}: ${stderr || stdout}`);
  return stdout.trim();
}

async function magick(args: string[]): Promise<{ code: number; out: string }> {
  const proc = Bun.spawn(["magick", ...args], { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return { code, out: (stdout + stderr).trim() };
}

const filters = process.argv.slice(2);
const ids = Object.keys(SHOTS).filter(
  (id) => filters.length === 0 || filters.some((f) => id.startsWith(f)),
);

mkdirSync(OUT, { recursive: true });
const summary: any[] = [];
for (const id of ids) {
  const [pw, ph] = SHOTS[id];
  const w = Math.round(pw / 2);
  const h = Math.round(ph / 2);
  const page = join(ROOT, "shots", id, "index.html");
  const render = join(OUT, `${id}-render.png`);
  const refPng = join(OUT, `${id}-ref.png`);
  const diff = join(OUT, `${id}-diff.png`);
  try {
    readFileSync(page);
  } catch {
    summary.push({ id, status: "missing" });
    continue;
  }
  try {
    await browser(["set", "viewport", String(w), String(h), "2"]);
    await browser(["open", `file://${page}`]);
    await Bun.sleep(400);
    await browser(["screenshot", "body", render]);
    await magick([join(ROOT, "reference", `${id}.jpg`), "-resize", `${w * 2}x${h * 2}!`, `PNG24:${refPng}`]);
    const cmp = await magick(["compare", "-metric", "RMSE", render, refPng, diff]);
    const m = cmp.out.match(/\(([\d.eE+-]+)\)/);
    const rmse = m ? Number(m[1]) : null;
    summary.push({ id, status: "ok", rmse, render, diff });
    console.error(`${id}  rmse=${rmse}`);
  } catch (err) {
    summary.push({ id, status: "error", error: String(err) });
    console.error(`${id}  ERROR ${err}`);
  }
}
await browser(["close"]).catch(() => {});
writeFileSync(join(ROOT, "verify-summary.json"), JSON.stringify(summary, null, 2));
console.log(JSON.stringify(summary, null, 2));

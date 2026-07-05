/**
 * Script metadata extraction, mirroring the app's "typed wins, comments fill
 * gaps" merge (src/scripts/metadata.rs + src/metadata_parser/mod.rs) closely
 * enough to validate that a port preserved every field the launcher reads.
 */

import type { ScriptMetadata } from "./types.ts";

const COMMENT_KEYS = new Set([
  "name",
  "description",
  "icon",
  "alias",
  "shortcut",
  "cron",
  "schedule",
  "keyword",
  "background",
]);

/** The Rust parser scans the head of the file for comment metadata. */
const COMMENT_SCAN_LINES = 30;

export function extractCommentMetadata(source: string): ScriptMetadata {
  const meta: ScriptMetadata = {};
  const lines = source.split("\n").slice(0, COMMENT_SCAN_LINES);
  for (const line of lines) {
    const m = line.match(/^\s*\/\/\s*([A-Za-z]+)\s*:\s*(.+?)\s*$/);
    if (!m) continue;
    const key = m[1].toLowerCase();
    if (!COMMENT_KEYS.has(key)) continue;
    if (!(key in meta)) {
      meta[key as keyof ScriptMetadata] = m[2];
    }
  }
  return meta;
}

/**
 * Tolerant extraction of the typed `metadata = { ... }` block: string-literal
 * values only, which matches what the launcher surfaces (name, shortcut, ...).
 */
export function extractTypedMetadata(source: string): ScriptMetadata {
  const meta: ScriptMetadata = {};
  const start = source.match(
    /(?:export\s+)?(?:const\s+|let\s+|var\s+)?metadata\s*(?::\s*[\w.]+\s*)?=\s*\{/,
  );
  if (!start || start.index === undefined) return meta;

  // Find the balanced closing brace.
  let depth = 0;
  let end = -1;
  for (let i = start.index + start[0].length - 1; i < source.length; i++) {
    if (source[i] === "{") depth += 1;
    else if (source[i] === "}") {
      depth -= 1;
      if (depth === 0) {
        end = i;
        break;
      }
    }
  }
  if (end === -1) return meta;

  const block = source.slice(start.index + start[0].length, end);
  const pairRe = /(?:^|[,{\n])\s*["']?([A-Za-z]+)["']?\s*:\s*(["'`])((?:\\.|(?!\2).)*)\2/g;
  let m: RegExpExecArray | null;
  while ((m = pairRe.exec(block)) !== null) {
    const key = m[1].toLowerCase();
    if (COMMENT_KEYS.has(key) && !(key in meta)) {
      meta[key as keyof ScriptMetadata] = m[3];
    }
  }
  return meta;
}

/** Effective metadata: typed values win, comment values fill the gaps. */
export function extractEffectiveMetadata(source: string): ScriptMetadata {
  return { ...extractCommentMetadata(source), ...extractTypedMetadata(source) };
}

/** Keys present in `original` but missing or changed in `ported`. */
export function metadataLosses(
  original: ScriptMetadata,
  ported: ScriptMetadata,
): string[] {
  const losses: string[] = [];
  for (const [key, value] of Object.entries(original)) {
    const after = ported[key as keyof ScriptMetadata];
    if (after === undefined) {
      losses.push(`lost ${key}: "${value}"`);
    } else if (after !== value) {
      losses.push(`changed ${key}: "${value}" → "${after}"`);
    }
  }
  return losses;
}

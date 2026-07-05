/**
 * Static v1-API scanner. Pure: (source, compat map) → Classification.
 *
 * Used twice per script — once on the v1 source to build context for the port
 * prompt, and once on the agent's output as the api-scan validator. Both sides
 * read the same compat-map.json, so the agent and the validator cannot disagree
 * about which APIs are allowed.
 *
 * Deliberately regex-based, not a full parse: comments are stripped, and the
 * typecheck validator backstops anything a token scan can miss (e.g. globals
 * the v2 SDK never declares).
 */

import type {
  Classification,
  CompatEntry,
  CompatMap,
  CompatStatus,
  Finding,
} from "./types.ts";
import compatMapJson from "./compat-map.json";

export function loadCompatMap(): CompatMap {
  const apis: Record<string, CompatEntry> = {};
  for (const [key, value] of Object.entries(compatMapJson.apis)) {
    apis[key] = value as CompatEntry;
  }
  return { apis };
}

/** Object-shaped globals whose mere property access counts as usage. */
const OBJECT_GLOBALS = new Set(["clipboard", "memoryMap"]);

/** Roots whose unknown methods are still unsupported in v2 (native input injection). */
const STUB_ROOTS = new Set(["keyboard", "mouse"]);

const KIT_IMPORT_RE =
  /(?:import\s*\(?\s*|from\s+|require\(\s*)['"]@johnlindquist\/kit(?:\/[^'"]*)?['"]/;

/** Replace comments with spaces, preserving newlines so line numbers survive. */
export function stripComments(source: string): string {
  let out = "";
  let i = 0;
  let mode: "code" | "line" | "block" | "string" = "code";
  let quote = "";
  while (i < source.length) {
    const ch = source[i];
    const next = source[i + 1];
    if (mode === "code") {
      if (ch === "/" && next === "/") {
        mode = "line";
        out += "  ";
        i += 2;
        continue;
      }
      if (ch === "/" && next === "*") {
        mode = "block";
        out += "  ";
        i += 2;
        continue;
      }
      if (ch === '"' || ch === "'" || ch === "`") {
        mode = "string";
        quote = ch;
      }
      out += ch;
      i += 1;
      continue;
    }
    if (mode === "line") {
      if (ch === "\n") {
        mode = "code";
        out += "\n";
      } else {
        out += " ";
      }
      i += 1;
      continue;
    }
    if (mode === "block") {
      if (ch === "*" && next === "/") {
        mode = "code";
        out += "  ";
        i += 2;
        continue;
      }
      out += ch === "\n" ? "\n" : " ";
      i += 1;
      continue;
    }
    // string mode: keep content (template interpolations may hold real calls)
    if (ch === "\\") {
      out += ch + (next ?? "");
      i += 2;
      continue;
    }
    if (ch === quote) {
      mode = "code";
    }
    out += ch;
    i += 1;
    continue;
  }
  return out;
}

function escapeRe(name: string): string {
  return name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function lineOf(source: string, index: number): number {
  let line = 1;
  for (let i = 0; i < index && i < source.length; i++) {
    if (source[i] === "\n") line += 1;
  }
  return line;
}

interface Pattern {
  api: string;
  re: RegExp;
  entry: CompatEntry;
}

function buildPatterns(map: CompatMap): Pattern[] {
  const patterns: Pattern[] = [];
  for (const [api, entry] of Object.entries(map.apis)) {
    if (api.includes(".")) {
      const [obj, method] = api.split(".");
      patterns.push({
        api,
        entry,
        re: new RegExp(
          `(?<![\\w$])${escapeRe(obj)}\\s*\\.\\s*${escapeRe(method)}\\s*\\(`,
          "g",
        ),
      });
    } else if (OBJECT_GLOBALS.has(api)) {
      patterns.push({
        api,
        entry,
        re: new RegExp(`(?<![.\\w$])${escapeRe(api)}\\s*\\.`, "g"),
      });
    } else if (api === "$") {
      // zx/execa-style: both `$(`...`)` calls and $`cmd` tagged templates.
      // The dot in the lookbehind exempts property access — `Bun.$` is the
      // recommended v2 replacement, not the missing global.
      patterns.push({
        api,
        entry,
        re: /(?<![.\w$])\$\s*[(`]/g,
      });
    } else {
      patterns.push({
        api,
        entry,
        re: new RegExp(`(?<![.\\w$])${escapeRe(api)}\\s*\\(`, "g"),
      });
    }
  }
  return patterns;
}

/**
 * Local bindings introduced by `import` statements. A compat-map finding is
 * about a missing v1 GLOBAL; an explicit import of the same name (e.g.
 * `import { $ } from "bun"`, `import { formatDate } from "date-fns"` — both
 * recommended by the compat map itself) shadows the global and must not be
 * flagged.
 */
export function importedBindings(code: string): Set<string> {
  const bindings = new Set<string>();
  const importRe = /import\s+([^'"]+?)\s+from\s+['"][^'"]+['"]/g;
  let m: RegExpExecArray | null;
  while ((m = importRe.exec(code)) !== null) {
    let clause = m[1].trim();
    const braceStart = clause.indexOf("{");
    if (braceStart !== -1) {
      const braceEnd = clause.indexOf("}");
      const named = clause.slice(braceStart + 1, braceEnd === -1 ? undefined : braceEnd);
      for (const spec of named.split(",")) {
        const parts = spec.trim().split(/\s+as\s+/);
        const local = (parts[1] ?? parts[0]).trim();
        if (local) bindings.add(local);
      }
      clause = clause.slice(0, braceStart);
    }
    const nsMatch = clause.match(/\*\s+as\s+(\S+)/);
    if (nsMatch) bindings.add(nsMatch[1]);
    const defaultName = clause.replace(/[,{}*]/g, " ").trim().split(/\s+/)[0];
    if (defaultName && defaultName !== "as") bindings.add(defaultName);
  }
  return bindings;
}

interface ShapePattern {
  api: string;
  re: RegExp;
  status: CompatStatus;
  replacement?: string;
  note: string;
}

/**
 * v1 call shapes that break on v2 APIs of the same name (2026-07-04 parity
 * audit). `stub` = crashes or renders garbage, api-scan fails; `caveat` =
 * silently degraded, api-scan warns and the port prompt gets the note.
 */
const SHAPE_PATTERNS: ShapePattern[] = [
  {
    api: "path.<node-method>",
    re: /(?<![\w$])path\s*\.\s*(join|dirname|basename|resolve|extname|relative|parse|format|normalize|isAbsolute|sep)\b/g,
    status: "stub",
    replacement: "import * as nodePath from 'node:path'",
    note: "v2's global `path` is ONLY the file-picker prompt — v1 merged it with node's path module, v2 does not, so path.join()/dirname()/... crash. Add `import * as nodePath from 'node:path'` and rewrite these calls to nodePath.*; keep bare path() calls as the picker.",
  },
  {
    api: "editor({config})",
    re: /(?<![\w$])editor\s*\(\s*\{/g,
    status: "stub",
    note: "v2 editor(content, language?, actions?) takes strings — a v1 EditorConfig object renders as [object Object]. Unwrap it: pass config.value as content and config.language as language; onInit/shortcuts/other fields have no v2 equivalent (declare in behavior_changes).",
  },
  {
    api: "editor.<method>",
    re: /(?<![\w$])editor\s*\.\s*(setSuggestions|append|setText|getSelection|replaceRange|moveCursor|insertText|setCodeHint)\s*\(/g,
    status: "stub",
    note: "v1's editor helper methods (editor.append/setText/...) do not exist in v2. Restructure the flow (declare in behavior_changes).",
  },
  {
    api: "select/mini/micro({config})",
    re: /(?<![\w$])(select|mini|micro)\s*\(\s*\{/g,
    status: "stub",
    note: "v2 select/mini/micro take (placeholder: string, choices) — a v1 config object becomes the literal placeholder '[object Object]'. Unwrap: pass config.placeholder as the string; other config fields have no v2 equivalent.",
  },
  {
    api: "fields({config})",
    re: /(?<![\w$])fields\s*\(\s*\{/g,
    status: "stub",
    note: "v2 fields() takes an ARRAY of field defs — v1's config-object form ({fields: [...], ...}) crashes (.map on object). Unwrap: pass config.fields as the array.",
  },
  {
    api: "arg({config})",
    re: /(?<![\w$])arg\s*\(\s*\{/g,
    status: "caveat",
    note: "v2 arg reads only placeholder/choices/actions/onSubmit from a config object — hint, preview, shortcuts, flags, onChange, onEscape, preventSubmit are silently ignored, and onSubmit is (value)=>void (input/state args and symbol returns unsupported). Keep what works; declare dropped behavior.",
  },
  {
    api: "chat(...)",
    re: /(?<![\w$])chat\s*\(/g,
    status: "caveat",
    note: "v2 chat() resolves to a ChatResult OBJECT, not v1's Message[] — any .map()/array use of the result crashes. Also chat.pushToken/setMessage/setMessages don't exist (only addMessage/getMessages), and actions move into the options arg.",
  },
  {
    api: "notify({options})",
    re: /(?<![\w$])notify\s*\(\s*\{/g,
    status: "caveat",
    note: "v2 notify() honors only title and body — sound, silent, icon, subtitle, and actions are silently dropped (declare in behavior_changes if used).",
  },
  {
    api: "compile(template)",
    re: /(?<![\w$])compile\s*\(/g,
    status: "caveat",
    note: "v2 compile() is a flat {{key}} replacer, NOT Handlebars — {{#each}}, {{#if}}, {{a.b}} and helpers silently render empty. If the template uses any of those, import handlebars explicitly (bun auto-installs) instead of using the global.",
  },
  {
    api: "memoryMap.<Map-method>",
    re: /(?<![\w$])memoryMap\s*\.\s*(has|size|forEach|entries|keys|values)\b/g,
    status: "stub",
    note: "v2 memoryMap is a plain {get,set,delete,clear} object, not a real Map — has/size/forEach/entries/keys/values crash. Restructure around get/set.",
  },
  {
    api: "say(text, {options})",
    re: /(?<![\w$])say\s*\(\s*[^,()]+,\s*\{/g,
    status: "caveat",
    note: "v2 say(text, voice?) takes a voice STRING second — a v1 options object gets passed as the voice name. Extract options.voice or drop the object.",
  },
  {
    api: "form(html, formData)",
    re: /(?<![\w$])form\s*\(\s*[^,()]+,\s*\{/g,
    status: "caveat",
    note: "v2 form(html, actions?) has no formData parameter — v1's initial-value prefill object is silently dropped. Inline initial values into the HTML instead.",
  },
];

const SEVERITY: Record<CompatStatus, number> = {
  supported: 0,
  renamed: 1,
  caveat: 1,
  stub: 2,
  removed: 2,
};

export function classify(source: string, map?: CompatMap): Classification {
  const compat = map ?? loadCompatMap();
  const code = stripComments(source);
  const findings: Finding[] = [];
  const seen = new Set<string>();

  const imported = importedBindings(code);
  for (const { api, re, entry } of buildPatterns(compat)) {
    // An explicit import shadows the v1 global of the same (bare) name.
    if (!api.includes(".") && imported.has(api)) continue;
    re.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = re.exec(code)) !== null) {
      const key = `${api}@${match.index}`;
      if (seen.has(key)) break;
      seen.add(key);
      findings.push({
        api,
        line: lineOf(code, match.index),
        status: entry.status,
        replacement: entry.replacement,
        note: entry.note,
        snippet: entry.snippet,
      });
    }
  }

  // Unknown keyboard.*/mouse.* methods: native input injection is categorically
  // unsupported in v2, even for methods the map doesn't enumerate.
  const rootRe = /(?<![\w$])(keyboard|mouse)\s*\.\s*(\w+)\s*\(/g;
  let rootMatch: RegExpExecArray | null;
  while ((rootMatch = rootRe.exec(code)) !== null) {
    const api = `${rootMatch[1]}.${rootMatch[2]}`;
    if (compat.apis[api]) continue; // already matched above
    if (!STUB_ROOTS.has(rootMatch[1])) continue;
    findings.push({
      api,
      line: lineOf(code, rootMatch.index),
      status: "stub",
      note: `Native ${rootMatch[1]} injection is unsupported in v2 (UnsupportedSdkFeatureError).`,
    });
  }

  // Call-shape mismatches (the env() class, found by the 2026-07-04 v1↔v2
  // SDK parity audit): APIs that exist in v2 but break on specific v1 call
  // shapes. Name-level scanning can't see these; each needs its own pattern.
  for (const shape of SHAPE_PATTERNS) {
    shape.re.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = shape.re.exec(code)) !== null) {
      findings.push({
        api: shape.api,
        line: lineOf(code, m.index),
        status: shape.status,
        replacement: shape.replacement,
        note: shape.note,
      });
    }
  }

  findings.sort((a, b) => a.line - b.line || a.api.localeCompare(b.api));

  const hasKitImport = KIT_IMPORT_RE.test(code);
  const worst = Math.max(
    0,
    hasKitImport ? 1 : 0,
    ...findings.map((f) => SEVERITY[f.status]),
  );
  const bucket =
    worst >= 2 ? "needs-rewrite" : worst === 1 ? "needs-changes" : "ready";

  return { bucket, findings, hasKitImport };
}

/** Findings rendered for prompts and receipts: one line each, with the fix. */
export function formatFindings(c: Classification): string {
  const lines: string[] = [];
  if (c.hasKitImport) {
    lines.push(
      `- import "@johnlindquist/kit" — REMOVE: v2 preloads the SDK; scripts must not import it.`,
    );
  }
  for (const f of c.findings) {
    if (f.status === "supported") continue;
    const fix = f.replacement ? ` → ${f.replacement}` : "";
    const note = f.note ? ` (${f.note})` : "";
    lines.push(`- line ${f.line}: ${f.api} [${f.status}]${fix}${note}`);
  }
  return lines.length > 0 ? lines.join("\n") : "(no incompatible APIs found)";
}

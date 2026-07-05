You are porting a Script Kit **v1** script to Script Kit **v2** (the GPUI app). Your output is consumed by a machine pipeline — follow the output contract exactly.

Your port must pass four mechanical validators, in order:
1. **tsc** against the real v2 SDK types (no undeclared globals, no type errors).
2. **API scan** — only APIs listed as available below may appear in your output.
3. **Metadata check** — every launcher-visible metadata field of the original (`// Name:`, `// Description:`, `// Icon:`, `// Alias:`, `// Shortcut:`, `// Cron:`, `// Schedule:`, or a typed `metadata = {}` object) must survive with identical values.
4. **Live run** — the script is executed under bun with the v2 SDK preloaded; it must reach its first prompt (or exit 0) without crashing, and complete when every prompt is auto-submitted.

## Rules

- v2 preloads the SDK: **remove any `import ... from "@johnlindquist/kit"`** — SDK globals are ambient.
- Preserve metadata comments verbatim, in the same head-of-file position.
- **No new npm dependencies** unless unavoidable; prefer bun/node built-ins (`Bun.$`, `fetch`, `Bun.file`, `Bun.write`, `Bun.Glob`, `node:fs`, `node:path`). Bun auto-installs plain imports, so an existing v1 `npm("pkg")` becomes a normal `import`.
- v2 scripts run under **bun**. TypeScript is fine.
- Preserve user-visible behavior. Where an API has no v2 equivalent, degrade gracefully and **declare every change in `behavior_changes`** — an empty list is a claim of identical behavior and is separately audited.
- Do not rewrite working code for style. Change only what compatibility requires.
- Do not add comments explaining the migration inside the code (the pipeline adds a provenance line itself).

## v2 API surface (everything available as ambient globals)

Prompts: `arg(placeholder?, choices?)`, `select(placeholder, choices)`, `div(html, actions?)`, `editor(options?)`, `mini()`, `micro()`, `fields([...])`, `form(html)`, `path()`, `hotkey()`, `drop()`, `template(str)`, `env(key, config?)` (v1-compatible: keep `{hint, placeholder, secret}` objects or prompt strings verbatim), `term(command?)`, `chat(options?)`, `webcam()`, `mic()`, `eyeDropper()`
UI: `md(markdown)`, `setInput`, `setPanel`, `setPreview`, `setPrompt`, `setActions`, `submit(value)`, `hide()`, `show()`, `blur()`, `exit(code?)`, `hud(message)`, `find()`
System: `notify(opts)`, `say(text)`, `beep()`, `clipboard.readText()/writeText()/readImage()/writeImage()`, `copy(text)`, `paste()`, `setSelectedText(text)` (pastes at cursor in the frontmost app), `getSelectedText()`, `browse(url)`, `editFile(path)`, `run(scriptName, ...args)`, `inspect(data)`
Windows/displays: `getWindows()`, `focusWindow(id)`, `getDisplays()`, `getFrontmostWindow()`, `moveWindow`, `resizeWindow`, `tileWindow`
Paths/util: `home(...)`, `skPath(...)` (~/.scriptkit), `kitPath(...)`, `tmpPath(...)`, `isFile`, `isDir`, `isBin`, `uuid()`, `memoryMap`
Clipboard history: `clipboardHistory()`, `clipboardHistoryPin/Unpin/Remove/Clear`

Known v2 limitations to port around (the scan findings below carry specifics): the global `path` is ONLY the picker prompt (node path methods need `import * as nodePath from 'node:path'`); `md()` renders a markdown subset (GFM tables, task lists, and deeply nested lists degrade — simplify the markdown or declare it); `clipboard` has only readText/writeText/readImage/writeImage; `compile()` is a flat {{key}} replacer, not Handlebars.

**Not available in v2** (validators reject these): `widget`, `vite`, `db`, `store`, `exec`, `$` global, `npm`/`attemptImport`, `get/post/put/patch/del`, `download`, `trash`, `degit`, `replace`, `globby`, `chalk`, `highlight`, `wait`, `textarea`, `edit`, `dev`, `toast`, `onTab`, `onExit`, `registerShortcut`, `keyboard.*`, `mouse.*`, `formatDate`, `formatDateToNow`, `createChoiceSearch`, `groupChoices`, `selectFile`, `selectFolder`, `mainScript`, `kenvPath`.

## Findings from the static scan of this script

{{FINDINGS}}

## Migration guidance for the APIs this script uses

{{COMPAT_GUIDANCE}}

## Worked example

v1 input:
```ts
// Name: Save Note
import "@johnlindquist/kit";
let note = await arg("Note?");
let notes = await db({ notes: [] });
notes.data.notes.push(note);
await notes.write();
await toast("Saved");
```

v2 output:
```ts
// Name: Save Note
const note = await arg("Note?");
const dbPath = skPath("db", "save-note.json");
let data: { notes: string[] } = { notes: [] };
try {
  data = JSON.parse(await Bun.file(dbPath).text());
} catch {}
data.notes.push(note);
await Bun.write(dbPath, JSON.stringify(data, null, 2));
hud("Saved");
```
with note: `{"summary":"db() → JSON file at skPath('db'); toast() → hud()","behavior_changes":[],"confidence":"high"}` — no behavior changes because the data still persists and the user still gets feedback.

## The script to port — `{{FILENAME}}`

```ts
{{SCRIPT_SOURCE}}
```

## Output contract

Respond with EXACTLY these two blocks and nothing else:

===PORTED_SCRIPT===
<the complete ported file, ready to save>
===END_PORTED_SCRIPT===
===MIGRATION_NOTE===
{"summary": "<one sentence: what changed and why>", "behavior_changes": ["<each user-visible difference, empty array if truly identical>"], "confidence": "high" | "medium" | "low"}
===END_MIGRATION_NOTE===

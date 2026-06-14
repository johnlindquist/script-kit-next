## Role Findings

The task is over-specified on *strategy* and under-specified on *what to actually build*. The strategic direction is locked, so the brief's job is to convert that into the **smallest set of artifacts a builder can implement in one sitting and publish today**. Three findings drive my recommendation:

1. **One route is enough.** The constraints (lean, no heavy assets, no SaaS hero, early artifact) and the wzrrd anonymous limits (5 MB / 50 files) both point at a single-page site. A second route only earns its keep if content genuinely doesn't fit one scroll — it doesn't here.
2. **The interactive components are where most of the cost/value lives.** A marketing page that "feels like a native command surface" must *behave* like one: a fake prompt list, a fake arg() flow, a live SDK snippet. These are cheap in Svelte and are the entire proof of the "programmable workbench" claim. I'm specifying exactly the data shapes so the builder doesn't invent them.
3. **Visual direction must be stolen from the app, not designed fresh.** I pulled the real theme tokens (background `#111217`, amber accent `#FBBF24`, gray border `#464647`, WCAG-AA-aware contrast). Reusing the app's own palette is both cheaper and more honest than a bespoke landing palette.

## Evidence And Assumptions

**Evidence (grounded in repo):**
- App theme tokens: `background.main = 0x111217`, `accent.selected = 0xFBBF24` (amber), `ui.border ≈ 0x464647`, contrast is explicitly tuned to WCAG AA large-text 3.0:1 (`src/theme/tests.rs:286`, `gpui_integration.rs:275-282`).
- SDK prompt APIs are the stated core: `arg, div, editor, term, fields, form, drop, hotkey, path, chat, mic, webcam` (README + project facts).
- README is explicit: "NOT a drop-in replacement", "Bring Your Own Libraries", Bun runtime, macOS-first (Linux/Windows planned), under active dev.
- wzrrd pipeline: SvelteKit + mdsvex + `adapter-static`, `prerender = true`, `trailingSlash = 'always'`, anonymous publish expires in 24h unless claimed.

**Assumptions (flagged):**
- I assume the builder has Bun/npm and `wzrrd` installed; the skill covers install if not.
- I assume "good enough to publish today" means **no custom illustration, no video, no external fonts** — SVG/CSS only. This is consistent with the stated constraints.
- I assume the live product is pre-release, so CTAs must route to *getting the source / trying a script*, not a polished download. The repo is the artifact.

## Failure Modes

The ways this brief (if built naively) goes wrong, in priority order:

1. **Generic SaaS hero** — gradient blob, centered headline, "Get Started" button. Violates the explicit constraint and dilutes the "native workbench" positioning. *Mitigation: hero IS a working mock prompt list, not a headline over a screenshot.*
2. **Over-promising on AI** — Agent Chat presented as finished product. Breaks trust with a dev audience that'll catch it instantly. *Mitigation: Agent Chat framed as "the differentiating bet," proof-oriented, with explicit trust/anti-positioning copy.*
3. **Route bloat** — adding `/docs`, `/scripts`, `/brain` routes that duplicate what the repo already serves. *Mitigation: one route; deep-link to the repo for depth.*
4. **Asset weight** — bundled fonts or screenshots blow the 5 MB / 1 MB-per-file anonymous limit. *Mitigation: system mono stack, inline SVG, no raster.*
5. **Fake precision in positioning** — "faster than Raycast" or "AI OS" leaks in. *Mitigation: the trust/anti-positioning block is copy-locked in the brief.*

## Recommendation

Build-ready brief follows. Single route, four interactive components, ~9 content blocks, honest competitive copy. A competent Svelte dev can implement and publish this in one focused session.

---

### 1. Hero copy, CTAs, subcopy

**Layout:** NOT a centered headline. The hero is a **live mock of the launcher** on the left, copy on the right (60/40). This is the entire "feels like a native command surface" move.

**Eyebrow (mono, accent-colored, above everything):**
```
script-kit-gpui  ·  v0  ·  macOS-first  ·  under active development
```

**Headline (the one marketing line, large, sans):**
```
A native programmable desktop workbench
for scripts, agents, and local memory.
```

**Subcopy (one sentence, secondary text):**
```
Write Bun-powered TypeScript, run it from a native command surface, and let
agents act on your desktop through verifiable, inspectable steps. macOS today.
Not a drop-in replacement — a rewrite.
```

**CTAs (two, inline, workbench-styled — NOT big gradient buttons):**
- Primary: `→ Browse the repo` (links to the GitHub repo)
- Secondary: `⌘C  Copy a starter script` (copies a 6-line `arg()` example to clipboard; tiny toast confirms)

**In the mock launcher (the left 60%),** the visible items are real, illustrative script names — this *is* the hero visual:
```
▸ open-in-editor
▸ screenshot-region
▸ summarize-clipboard
▸ day-page
▸ tile-window-left
```
with a blinking caret in a search field showing `> sum` filtering the list to `summarize-clipboard`. This single interaction communicates the entire product.

---

### 2. Section order and layout direction

Single page, top-to-bottom, asymmetric grid throughout (never centered three-columns). Order chosen to move claim → proof → trust → action:

| # | Block | Layout direction | Purpose |
|---|-------|------------------|---------|
| 1 | **Hero launcher** | 60/40 split, mock list left, copy right | Establishes "command surface" instantly |
| 2 | **The pitch in one prompt** | Full-width, centered text block, narrow column | The category claim, restated once |
| 3 | **SDK prompt API grid** | Responsive grid of API "chips" | Proves programmability is real & scoped |
| 4 | **Interactive `arg()` demo** | Full-width interactive panel | Live proof: type → it returns |
| 5 | **Agent Chat / desktop context** | 50/50, copy left, annotated mock transcript right | The differentiating bet, proof-oriented |
| 6 | **Memory layer (Brain / Day Page / clipboard sediment)** | Three stacked rows, each one-line + detail toggle | Supporting layer, not headline |
| 7 | **Honest positioning** | Full-width comparison table | Sharp but factual |
| 8 | **Trust / what it isn't** | Full-width, monospaced "not" list | Anti-positioning, sets expectations |
| 9 | **Footer / CTA** | Left-aligned, repo + status | Calm exit |

Sticky top bar is intentionally **absent** — a single-page site doesn't need nav chrome and it would clutter the workbench feel.

---

### 3. Interactive Svelte components (the four worth building)

I'm cutting this to **four**, not five. A fifth (e.g., a sortable comparison table) isn't worth the cost; the positioning block can be static markup.

#### 3a. `<LauncherMock />` — the hero (highest value)

A non-interactive-by-default list that *becomes* a filterable search when the user clicks the search field. This single component sells the product.

```ts
// data/launcher-items.ts
type LauncherItem = {
  id: string;          // stable semantic id — mirrors the app's real IDs
  name: string;        // script name, kebab-case
  description: string;
  kind: "script" | "builtin" | "agent";
  hotkey?: string;     // e.g. "⌘⇧S"
};
export const launcherItems: LauncherItem[] = [
  { id: "open-in-editor", name: "open-in-editor", description: "Open the frontmost file in your $EDITOR", kind: "script", hotkey: "⌘E" },
  { id: "screenshot-region", name: "screenshot-region", description: "Drag a screen region, save PNG to ~/Desktop", kind: "script" },
  { id: "summarize-clipboard", name: "summarize-clipboard", description: "Send clipboard text to an agent, paste the summary", kind: "agent" },
  { id: "day-page", name: "day-page", description: "Open today's Brain memory page", kind: "builtin", hotkey: "⌘D" },
  { id: "tile-window-left", name: "tile-window-left", description: "AX-driven window tiling", kind: "builtin", hotkey: "⌘⌥←" },
  { id: "clipboard-history", name: "clipboard-history", description: "Browse recent clips", kind: "builtin", hotkey: "⌘⇧V" },
];
```
**Behavior:** `$state` query string; `nucleo`-style fuzzy match is overkill for 6 items — simple `includes()` filter with name-weighted ranking. On focus, show caret blink (CSS). Selecting an item shows a one-line toast like `▸ ran open-in-editor` (fake, clearly labeled). Keyboard: ↑/↓ to move, Enter "runs".

#### 3b. `<ArgDemo />` — live `arg()` proof (second-highest value)

Shows the SDK's core primitive working in the browser. Left: code. Right: the rendered result of that code, live.

```ts
// The demo cycles a hardcoded snippet and renders its "output":
const snippets = [
  { label: "arg() — pick one", code: `await arg("Open which?", apps)` },
  { label: "div() — render HTML", code: `await div(\`<h2>Hello</h2>\`)` },
  { label: "fields() — structured form", code: `await fields([{ name: "title" }])` },
];
```
**Behavior:** a segmented control switches the snippet; the right panel re-renders a mocked prompt UI for that API (a list for `arg`, rendered HTML for `div`, a labeled input for `fields`). This proves "prompts are the core" without a backend. Purely `$state`-driven, no fetch.

#### 3c. `<ApiGrid />` — the SDK surface, as data (cheap, high signal)

```ts
type ApiEntry = { name: string; kind: string; blurb: string; };
const apis: ApiEntry[] = [
  { name: "arg()", kind: "choice", blurb: "Pick from a list" },
  { name: "div()", kind: "html", blurb: "Render arbitrary HTML" },
  { name: "editor()", kind: "text", blurb: "Inline code/text editor" },
  { name: "term()", kind: "shell", blurb: "Embedded terminal prompt" },
  { name: "fields()", kind: "form", blurb: "Structured form input" },
  { name: "form()", kind: "form", blurb: "Multi-field submitted form" },
  { name: "drop()", kind: "io", blurb: "Drop target for files/text" },
  { name: "hotkey()", kind: "input", blurb: "Capture a global hotkey" },
  { name: "path()", kind: "fs", blurb: "Filesystem path picker" },
  { name: "chat()", kind: "agent", blurb: "Agent Chat surface" },
  { name: "mic()", kind: "media", blurb: "Audio capture / dictation" },
  { name: "webcam()", kind: "media", blurb: "Camera capture" },
];
```
**Behavior:** static grid; the only interactivity is a `kind` filter row (`all · choice · form · agent · media`) that dims non-matching chips. Reusable, data-driven — exactly what the wzrrd skill says to prefer over hardcoded markup.

#### 3d. `<AgentTranscript />` — the differentiating bet, proof-oriented

A static-but-annotated mock of one Agent Chat turn, with **callouts that name the trust mechanism**. Not interactive (keeps cost down); the annotations are the point.

```ts
type Step = {
  id: string;          // "step-1" — the stable semantic ID the product really uses
  action: string;      // "tile-window-left"
  target: string;      // semantic target
  status: "verified" | "pending";
  note: string;        // the trust/inspection callout
};
const transcript: Step[] = [
  { id: "step-1", action: "read focused window", target: "ax:focused-window", status: "verified", note: "Context read via MCP resource — no mutation." },
  { id: "step-2", action: "tile-window-left", target: "window:frontmost", status: "verified", note: "UI transaction — replayable by semantic ID." },
  { id: "step-3", action: "append to Day Page", target: "brain:day-page", status: "pending", note: "Write requires your approval." },
];
```
**Behavior:** each step renders as a row; clicking a row expands `note`. The visual grammar: green ✓ for verified reads/replays, amber ◑ for pending writes. This *shows* "trust-aware, verifiable" instead of asserting it.

*(I'm deliberately **not** specifying a fifth sortable positioning table — see §4, that block is static markup and cheaper as prose.)*

---

### 4. Competitive positioning copy (honest but sharp)

A compact comparison, **factual columns only**, no cell that says "better". Let the reader draw the conclusion.

**Section heading:** `Where it sits`
**Subhead:** `One row per product. No asterisks.`

| | Script Kit GPUI | Classic Script Kit | Raycast | Alfred |
|---|---|---|---|---|
| **Core model** | Programmable workbench (you write the app) | Programmable launcher | Extension marketplace | Workflow + keyword launcher |
| **Script runtime** | Bun (JS/TS), bring-your-own deps | Node, bundled `kit` global | Node-ish extensions | Native, limited scripting |
| **Source** | Open, Rust/GPUI rewrite, one repo | Open | Closed | Closed |
| **Memory layer** | Local markdown Brain + Day Page | — | — | — |
| **Agent surface** | First-class Agent Chat + desktop context via MCP | — | Extensions | — |
| **Status** | v0, macOS-only, not a drop-in replacement | Mature | Mature | Mature |

**Closing line under the table:**
```
Not trying to replace what you have. Trying to be the thing you reach for
when "I'll just write a script for that" should take 30 seconds.
```

---

### 5. Trust / anti-positioning copy

**Section heading:** `What this is not`
**Subhead (secondary):** `Set early expectations. We're early.`

Rendered as a monospaced list, each line struck-through or prefixed with `✕`, then a single positive reframe:

```
✕ Not a Raycast clone.
✕ Not an "AI OS."
✕ Not a second-brain app.
✕ Not no-code automation.
✕ Not a productivity super-app.
✕ Not a drop-in replacement for classic Script Kit.
✕ Not "works everywhere." macOS today. Linux/Windows planned, not shipped.
✕ Not "faster than X." We don't bench against closed apps.

What it is: a native surface where your scripts, your agents, and your
local notes share one command palette. Early, honest, and yours to extend.
```

**Status line (small, secondary, in footer):**
```
v0 · under active development · macOS 13+ · Bun · Rust/GPUI
```

---

### 6. Visual direction (lean, stolen from the app)

Grounded in the repo's real theme tokens — not a new design system:

**Palette (CSS custom properties, from `src/theme`):**
```css
:root {
  --bg:        #111217;  /* background.main, app's true base */
  --bg-elev:   #1a1b22;  /* one step up — panels/cards */
  --bg-input:  #1a2b3c;  /* search_box token — input surfaces */
  --border:    #2a2b33;  /* border, softened from 0x464647 for web legibility */
  --text:      #e6e6e6;  /* text.primary */
  --text-dim:  #8a8b93;  /* text.secondary */
  --accent:    #FBBF24;  /* accent.selected — amber. Used SPARINGLY: caret, active row, primary CTA only */
  --ok:        #4ade80;  /* ui.success — verified steps */
  --warn:      #facc15;  /* ui.warning — pending steps */
}
```
**Rule:** amber is the *only* color used for emphasis. No gradients. No second accent. The page should read almost-grayscale with amber as the single "live" signal — exactly how the real launcher feels.

**Typography (system stack, no external fonts — constraint-compliant):**
```css
--font-sans: ui-sans-serif, system-ui, -apple-system, "SF Pro Text", sans-serif;
--font-mono: ui-monospace, "SF Mono", "JetBrains Mono", Menlo, Consolas, monospace;
```
- Headlines: sans, tight tracking, left-aligned, never centered.
- All code, all eyebrow text, all "status" microcopy, all launcher rows: **mono**.
- Body: sans, relaxed line-height, max-width ~62ch.

**Interaction motifs (cheap, all CSS/Svelte):**
- **Blinking caret** in every mock input (`@keyframes`, 1s, accent color).
- **Active-row accent bar** — a 2px amber left-border on the focused list item, mirrors the app.
- **No hover scale, no shadows on buttons.** Buttons are bordered mono pills. The "workbench" reads as flat and instrument-like, not bubbly.
- **Subtle row separators** (`1px solid var(--border)`), no cards-with-shadows.
- **Toast** on the copy-CTA: bottom-left, mono, auto-dismiss 1.5s. No animations library.

**Density target:** the page should feel closer to a terminal than to a landing page. Generous vertical rhythm, but within each block, density is high (the launcher, the API grid, the transcript are all information-dense by design).

---

### 7. Complete content blocks + microcopy

Every block, with its microcopy, in order. This is the builder's checklist.

1. **Hero launcher** — eyebrow / headline / subcopy / 2 CTAs / 6 mock launcher items (all copy in §1 & §3a).
2. **The pitch (one restatement),** narrow column:
   `It's the place where "I'll write a quick script for that" actually is quick.`
3. **SDK surface grid** — heading `Prompts are the core. Everything else you \`bun add\`.` + 12 API chips (§3c) + filter row.
4. **Live `arg()` demo** — heading `Type it. Run it.` + segmented control + 3 mocked prompts (§3b).
5. **Agent Chat** — heading `Agents that show their work.` + subcopy `Agent Chat acts through stable, inspectable steps. Reads are verifiable. Writes ask first.` + annotated transcript (§3d).
6. **Memory layer** — heading `Memory, locally, in markdown.` + three toggling rows:
   - `Brain` — `~/​.scriptkit/brain — your notes, as files you already know how to use.`
   - `Day Page` — `Today's surface. Memory and diary in one markdown file.`
   - `Clipboard sediment` — `Quietly remembers useful URLs and repeat-copy signals. No popup. No noise.`
7. **Honest positioning** — heading + table + closing line (§4).
8. **What this is not** — heading + struck list + reframe + status line (§5).
9. **Footer** — `→ Browse the repo` / `v0 · under active development · macOS 13+` / `noindex`. One line.

**Microcopy rules (give to builder):**
- Every status/version line is mono, `--text-dim`, ~12px.
- Never use "powerful", "seamless", "supercharge", "unleash", "10x".
- Hotkeys in `<kbd>`-styled mono pills with `--border`.
- The word "AI" appears **zero** times in body copy; use "agents" / "Agent Chat" only.

---

### 8. SvelteKit / mdsvex implementation notes

**Route count: 1.** Single page at `src/routes/+page.svx`. No second route — none of the content justifies it, and it keeps the build inside wzrrd's anonymous limits. If the builder insists on a second route later, the only defensible one is `/install/` with copy-pasteable setup commands; everything else belongs in the repo.

**Scaffold (exact, from the wzrrd skill):**
```bash
npx sv create site --template minimal --types ts --no-add-ons --install npm
cd site
npx sv add mdsvex sveltekit-adapter="adapter:static" --install npm
```

**Prerender config — create `src/routes/+layout.ts`:**
```ts
export const prerender = true;
export const trailingSlash = 'always';
```
(Single route, but `trailingSlash` is harmless and prevents future foot-guns.)

**File layout (small, deliberate):**
```
src/
  routes/
    +layout.ts            # prerender + trailingSlash
    +layout.svelte        # ONE place: <slot/> wrapped in global styles, no nav
    +page.svx             # the whole page, mdsvex prose interleaved with components
  lib/
    components/
      LauncherMock.svelte
      ArgDemo.svelte
      ApiGrid.svelte
      AgentTranscript.svelte
    data/
      launcher-items.ts
      apis.ts
      transcript.ts
  app.html                # set <html lang="en">, dark bg to avoid white flash
  app.css                 # the :root tokens from §6 + base resets ONLY
```

**Component discipline (matches the wzrrd skill's "structure data as data"):**
- All list/table content lives in `lib/data/*.ts` as typed exports — never hardcoded in markup.
- Each component is props-in, `$state`-internal, zero fetch. Purely client-side; static output.
- Use Svelte 5 runes (`$state`, `$derived`) — `sv create` ships Svelte 5.
- Transitions: Svelte's built-in `fade`/`slide` only, for the transcript detail expansion. No animation lib.

**Zero-asset rule (enforced):**
- No `static/` images. No webfonts. Icons are inline SVG or unicode glyphs (`▸ ✕ ✓ ◑ →`).
- The only "graphic" is CSS: the launcher's accent caret bar and bordered pills.
- This keeps total weight well under wzrrd's 5 MB / 1 MB-per-file cap.

**Build + publish:**
```bash
npm run build                 # -> build/
npx vite preview              # restart after each rebuild (skill gotcha)
wzrrd publish --file ./build --slug scriptkit-gpui
```
Anonymous publish → returns live URL + `claimUrl`. Open the claim URL within 24h to make it permanent (or `wzrrd login` first for a permanent publish). Site is `noindex` by default — correct for an early artifact; do **not** add `--index`.

**Verify after publish (content, not status — wzrrd 200s unknown paths):**
```bash
curl -s https://scriptkit-gpui.wzrrd.sh/ | grep -o '<title>[^<]*</title>'
```

---

### Self Score

**8.5 / 10.** This brief is build-ready: exact copy, four components with data shapes, real palette tokens stolen from the app, one route, publish commands, and a verification step. A builder can start and finish without re-deciding anything strategic. What holds it back from a 9.5: (a) I can't verify the *exact* repo URL for the primary CTA — the builder must drop it in; (b) the Agent Chat transcript is necessarily a mock, so "proof-oriented" is honestly bounded — it proves the *framing*, not shipped behavior; (c) a true pragmatist would want to A/B the hero (mock-launcher vs. plain headline), but that's out of scope for "publish today." Scope is right-sized: I cut a fifth component and a second route that would have added cost without adding clarity. The smallest thing that fully satisfies the task is exactly one page with these four components — no more.

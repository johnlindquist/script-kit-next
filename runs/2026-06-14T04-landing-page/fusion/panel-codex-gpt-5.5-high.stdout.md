## Role Findings

The strongest page shape is a single-route product brief that feels like opening a native command surface, not reading a SaaS pitch. The page should sell the category in one sentence, then prove it through concrete surfaces: scripts, prompt APIs, Agent Chat, desktop context, markdown memory, and quiet clipboard sediment.

Hero copy:

Eyebrow: `macOS-first • Rust/GPUI • Bun-powered JS/TS`

H1: `Script Kit GPUI`

Hero line: `A native programmable desktop workbench for scripts, agents, and local workflow memory.`

Subcopy: `Write Bun-powered JavaScript and TypeScript, run it from a fast native command surface, and give agents structured desktop context they can act on and verify.`

Primary CTA: `Explore the workbench`

Secondary CTA: `View the prompt APIs`

Small trust line: `Active rewrite. Developer preview. Not a drop-in replacement for classic Script Kit.`

Hero interactive: a command-surface mock with a prompt input, list rows, semantic IDs, and a right-side execution receipt.

Recommended section order:

1. Hero command surface: product name, one-line positioning, CTA pair, live workbench mock.
2. Programmability first: “Scripts are the product surface.”
3. Prompt API explorer: arg, div, editor, term, fields, form, drop, hotkey, path, chat, mic, webcam.
4. Agent execution with receipts: Agent Chat, MCP/protocol context, semantic IDs, verifiable transactions.
5. Native workbench surfaces: launcher/list, actions, clipboard history, tools, terminal prompt, notes, dictation, permissions.
6. Local workflow memory: Brain, Day Page, clipboard sediment.
7. Competitive positioning: not Raycast, not an AI OS, not a second brain.
8. Status and trust: active development, macOS-first, bring your own libraries, SDK and app together.
9. Final CTA: “Build scripts that can become real desktop tools.”

## Evidence And Assumptions

Content blocks and microcopy:

`Programmability first`
Headline: `Scripts are the interface.`
Copy: `Script Kit GPUI starts with code: Bun-powered JS/TS scripts that can open prompts, render UI, run terminals, collect files, listen for hotkeys, and call the libraries you already use.`
Microcopy chips: `bun add anything`, `prompt APIs`, `native command surface`, `SDK + app in one repo`.

`Prompt API Explorer`
Headline: `Prompt APIs for real desktop work.`
Intro: `Compose prompts from focused primitives instead of rebuilding window chrome every time.`
API labels:
`arg` - `Ask for a value from the launcher.`
`div` - `Render custom HTML-like prompt content.`
`editor` - `Open an editing surface.`
`term` - `Run terminal workflows inside a prompt.`
`fields` - `Collect structured inputs.`
`form` - `Build richer form flows.`
`drop` - `Accept files and paths.`
`hotkey` - `Bind repeatable commands.`
`path` - `Pick files and directories.`
`chat` - `Work with conversational context.`
`mic` - `Capture voice input.`
`webcam` - `Capture camera input.`

`Agent Execution`
Headline: `Agents get context they can prove.`
Copy: `Agent Chat is not positioned as magic. It is a workbench surface that can receive structured desktop and UI context, call protocol/MCP resources, and produce verifiable UI transactions through stable semantic IDs.`
Receipt labels: `context read`, `action proposed`, `semantic target`, `transaction verified`.

`Surfaces`
Headline: `One workbench, many native surfaces.`
Rows: `Launcher/list`, `Prompt shells`, `Actions menu`, `Clipboard history`, `App/window/file tools`, `Terminal prompt`, `Notes`, `Dictation`, `Permissions wizard`.

`Memory`
Headline: `Memory that stays local and useful.`
Copy: `Script Kit Brain stores local markdown under ~/.scriptkit/brain. Day Page gives today a working surface. Clipboard sediment quietly keeps useful copied URLs and repeated-copy signals without popup UI.`
This should be supporting material, not the headline category.

`Competitive Positioning`
Headline: `Sharper than a launcher. Smaller than an OS.`
Copy blocks:
`Compared with launchers:` `Script Kit GPUI is built around programmable scripts and prompt APIs, not a fixed catalog of commands.`
`Compared with no-code automation:` `It assumes developers want code, packages, terminals, files, and typed APIs.`
`Compared with AI shells:` `Agent features are proof-oriented: context, stable targets, receipts, and local workflows before claims of autonomy.`
`Compared with second brains:` `Markdown memory supports the workbench. It does not try to become the category.`

`Trust / Anti-positioning`
Headline: `Clear about what it is.`
Bullets:
`macOS-first today.`
`Under active development.`
`Not a drop-in replacement for old Script Kit.`
`Not a Raycast clone.`
`Not an AI OS.`
`Not no-code automation.`
`Not “for everyone.”`
`Bring your own libraries with bun add.`

## Failure Modes

The main failure mode is making the page too abstract. “AI desktop workbench” or “local memory layer” alone will blur the product into existing categories. The hero needs to show a scriptable command surface immediately.

Second risk: over-selling Agent Chat. The better angle is trust and proof: agents receive structured desktop context, act through known surfaces, and leave verifiable transactions. Avoid autonomy language unless paired with verification.

Third risk: memory taking over the story. Brain, Day Page, and sediment are retention features. They should make the product feel useful over time, but programmability remains the headline.

Fourth risk: a heavy marketing build. For wzrrd, keep this to one SvelteKit route, mdsvex content, lightweight Svelte components, CSS/SVG-only visuals, no stock imagery, no external fonts, and no server runtime.

## Recommendation

Build one route: `/`, authored as `src/routes/+page.svx`, with shared components in `src/lib/components`. Use `src/routes/+layout.ts` with prerender and trailing slash support even for the single route.

Interactive Svelte components worth building:

`CommandWorkbench.svelte`
Data:
```ts
type CommandItem = {
  id: string;
  title: string;
  kind: 'script' | 'agent' | 'memory' | 'tool';
  shortcut?: string;
  description: string;
  receipt?: string[];
};
```
Behavior: searchable command list, selectable rows, right panel shows script snippet and verification receipt.

`PromptApiExplorer.svelte`
Data:
```ts
type PromptApi = {
  name: string;
  group: 'input' | 'ui' | 'system' | 'agent' | 'media';
  oneLiner: string;
  sample: string;
};
```
Behavior: segmented filter by group, clicking an API updates a compact code panel.

`AgentReceiptTimeline.svelte`
Data:
```ts
type ReceiptStep = {
  label: string;
  detail: string;
  status: 'context' | 'proposal' | 'action' | 'verified';
};
```
Behavior: stepper/timeline showing context read -> target selected -> action run -> transaction verified.

`SurfaceMap.svelte`
Data:
```ts
type Surface = {
  name: string;
  role: string;
  connectsTo: string[];
};
```
Behavior: clickable surface grid. Selecting a surface highlights related APIs and context paths.

`MemoryStack.svelte`
Data:
```ts
type MemoryLayer = {
  name: 'Brain' | 'Day Page' | 'Clipboard sediment';
  pathOrSignal: string;
  promise: string;
  antiPromise: string;
};
```
Behavior: three-column local memory explainer with toggles for “what it keeps” and “what it avoids.”

Visual direction:

Palette: near-black graphite background, off-white text, muted steel borders, restrained cyan for active command focus, amber only for receipts/status, and green only for verified states. Avoid purple-blue AI gradients.

Typography: system stack only, SF-like feel: `font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", sans-serif;`. Use compact developer UI sizing, not oversized hero typography beyond the product name.

Motifs: command rows, keycaps, semantic ID pills, receipt timelines, terminal/editor panes, native-panel borders, subtle focus rings. No stock screenshots. No bubbly cards. No decorative blobs.

Implementation notes:

Use mdsvex for narrative blocks and import Svelte components inline. Keep all component data in local `.ts` files or inside components as typed arrays. Use CSS variables for palette tokens in `+layout.svelte` or `app.css`. Keep animations to hover/focus transitions and small timeline reveals. Route count should be one unless the builder wants `/api/` as a generated reference appendix; for publishing today, one route is better.

Final CTA block:

Headline: `Build scripts that understand your desktop.`
Copy: `Script Kit GPUI is for developers who want their command surface, prompt APIs, agents, and local workflow memory in one programmable native workbench.`
Primary CTA: `Explore the source`
Secondary CTA: `Read the API map`

Self Score: `9/10`. The brief is build-ready, honest about product state, and specific enough for a SvelteKit/mdsvex implementation. The only remaining gap is live repo URL/install URL confirmation, which the builder should verify before wiring CTAs.



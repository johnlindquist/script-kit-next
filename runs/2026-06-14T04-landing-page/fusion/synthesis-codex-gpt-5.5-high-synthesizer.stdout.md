**Landing Page Brief: Script Kit GPUI**

Build one route: `/`. Treat it as a publish-today static landing page, not a docs site. Use `+page.svx` for narrative sections and four local Svelte components for the native workbench feel.

**Hero**
Eyebrow:
`macOS-first · Rust/GPUI rewrite · Bun-powered JavaScript/TypeScript`

H1:
`Script Kit GPUI`

Hero line:
`A native programmable desktop workbench for scripts, agents, and local workflow memory.`

Subcopy:
`Write Bun-powered JavaScript and TypeScript, run scripts from a native command surface, and give Agent Chat structured desktop context it can act on through inspectable, verifiable steps.`

Primary CTA:
`Explore the source`

Secondary CTA:
`View prompt APIs`

Optional tertiary CTA if clipboard fallback is implemented:
`Copy starter script`

Trust line under CTAs:
`Active rewrite. Under active development. macOS-first today. Not a drop-in replacement for classic Script Kit.`

Hero layout: make the launcher/workbench mock the first-viewport signal. Copy sits beside it on desktop and above it on mobile. Label the mock subtly: `Example workbench UI`.

**Section Order**
1. Hero workbench mock  
   Product name, positioning, CTAs, native command surface mock.

2. Programmability first  
   Heading: `Scripts are the interface.`  
   Copy: `Script Kit GPUI starts with code: Bun-powered JavaScript and TypeScript scripts that open prompts, render UI, collect files, run terminal flows, bind hotkeys, call libraries, and become native desktop tools.`  
   Chips: `bun add anything`, `prompt APIs`, `native command surface`, `SDK + app in one repo`.

3. Prompt API explorer  
   Heading: `Prompt APIs for real desktop work.`  
   Copy: `Compose focused prompts instead of rebuilding window chrome for every workflow.`

4. Live prompt demo  
   Heading: `Type it. Run it. Keep the script.`  
   Show a small `arg()` / `fields()` / `term()` style demo with code on one side and a rendered prompt mock on the other.

5. Agent Chat with receipts  
   Heading: `Agents that show their work.`  
   Copy: `Agent Chat is framed around context, stable targets, approval, and verification. Reads are inspectable. Writes should be explicit.`

6. Workbench surfaces  
   Heading: `One workbench, many native surfaces.`  
   Items: launcher/list, prompt shells, actions menu, clipboard history, app/window/file tools, terminal prompt, notes, dictation, permissions wizard.

7. Local memory layer  
   Heading: `Memory that stays local and useful.`  
   Copy: `Script Kit Brain stores local markdown under ~/.scriptkit/brain. Day Page gives today a working diary surface. Clipboard sediment quietly keeps useful copied URLs and repeated-copy signals without popup UI.`

8. Competitive positioning  
   Heading: `Where it sits.`  
   Use category comparison, not named competitor claims unless verified.

   Copy blocks:
   - `Compared with launchers:` `Script Kit GPUI is built around programmable scripts and prompt APIs, not a fixed catalog of commands.`
   - `Compared with no-code automation:` `It assumes developers want code, packages, terminals, files, and typed APIs.`
   - `Compared with agent shells:` `Agent features are proof-oriented: structured context, stable targets, receipts, and approvals before claims of autonomy.`
   - `Compared with second-brain tools:` `Markdown memory supports the workbench. It is not the main category.`

9. Trust / anti-positioning  
   Heading: `Clear about what it is.`  
   Bullets:
   `macOS-first today.`  
   `Under active development.`  
   `Not a drop-in replacement for old Script Kit.`  
   `Not no-code automation.`  
   `Not a productivity super-app.`  
   `Not a generic agent shell.`  
   `Bring your own libraries with bun add.`

10. Final CTA  
   Heading: `Build scripts that understand your desktop.`  
   Copy: `For developers who want scripts, prompt APIs, Agent Chat, and local workflow memory in one programmable native workbench.`  
   CTAs: `Explore the source` and `View prompt APIs`.

**Four Svelte Components**
1. `LauncherMock.svelte`

```ts
type LauncherItem = {
  id: string;
  name: string;
  description: string;
  kind: "script" | "builtin" | "agent" | "memory";
  shortcut?: string;
};
```

Behavior: searchable command list, arrow-key selection, Enter shows simulated run receipt. Include label: `Example workbench UI`.  
No-JS fallback: render the full static command list.  
Reduced motion: disable caret blink and row transition.

2. `PromptApiExplorer.svelte`

```ts
type PromptApi = {
  name: string;
  group: "input" | "ui" | "system" | "agent" | "media";
  oneLiner: string;
  sample: string;
};
```

APIs: `arg`, `div`, `editor`, `term`, `fields`, `form`, `drop`, `hotkey`, `path`, `chat`, `mic`, `webcam`.  
Behavior: filter by group, selected API updates code panel.  
No-JS fallback: show all APIs as static chips with one-line descriptions.

3. `ArgDemo.svelte`

```ts
type DemoSnippet = {
  label: string;
  code: string;
  renderedState: "list" | "form" | "terminal";
};
```

Behavior: segmented control switches between `arg()`, `fields()`, and `term()` examples. Render a simulated prompt result beside the code.  
Keyboard: tabs and arrow keys work in the segmented control.  
No-JS fallback: show the default `arg()` code and static rendered prompt.

4. `AgentReceiptTimeline.svelte`

```ts
type ReceiptStep = {
  id: string;
  label: string;
  target: string;
  status: "context" | "proposed" | "approved" | "verified";
  detail: string;
};
```

Behavior: click or keyboard-expand each step.  
Example labels: `context read`, `semantic target selected`, `action proposed`, `transaction verified`.  
Include label: `Illustrative Agent Chat receipt`.  
No-JS fallback: expanded static timeline.  
Reduced motion: no staggered animation.

**Visual Direction**
Use graphite native-panel styling, system fonts, dense rows, mono microcopy, keycaps, code panes, semantic ID pills, and receipt timelines.

Palette direction, not verified token values: near-black graphite background, off-white text, muted borders, amber for active/focus states, green for verified states. Verify exact app theme tokens before claiming they match the product.

No stock imagery. No external fonts. No heavy screenshots. No purple-blue gradients. No bubbly cards. Keep the page closer to a command surface than a SaaS homepage.

**Implementation Notes**
Use one route: `src/routes/+page.svx`.

Suggested files:
```txt
src/routes/+page.svx
src/routes/+layout.ts
src/lib/components/LauncherMock.svelte
src/lib/components/PromptApiExplorer.svelte
src/lib/components/ArgDemo.svelte
src/lib/components/AgentReceiptTimeline.svelte
src/lib/data/apis.ts
src/lib/data/launcher-items.ts
src/lib/data/agent-receipts.ts
src/app.css
```

Set prerendering for static output. Keep data in typed local arrays. Components should not fetch. Avoid syntax highlighter bundles unless lazy-loaded. Target a small first-load JS budget.

Before publishing, verify the real repo URL, API/docs links, install state, theme tokens, macOS support floor, and current wzrrd-publish behavior. Do not hardcode wzrrd limits, claim URL behavior, noindex defaults, or publish commands into the brief unless checked against the current tooling.

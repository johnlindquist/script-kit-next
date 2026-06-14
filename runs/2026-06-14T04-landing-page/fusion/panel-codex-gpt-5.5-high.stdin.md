Panel-specific reasoning contract:
Panel role: architect
Focus on the complete design, tradeoffs, implementation shape, and how the pieces fit together.

Return your answer with these headings:
## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score

Original task:
You are a high-end product marketing + web design implementation panel.

Build a concrete landing-page brief for a wzrrd-publish static site for Script Kit GPUI. The builder will implement this as a SvelteKit + mdsvex static app with client-side Svelte components and then publish it to wzrrd.sh.

Project facts:
- Script Kit GPUI is a macOS-first programmable desktop workbench for developers.
- Users write Bun-powered JavaScript/TypeScript scripts and run them from a native command surface.
- It is a complete rewrite of Script Kit using Rust/GPUI, combining SDK and app in one repo.
- The SDK is focused on prompt APIs: arg, div, editor, term, fields, form, drop, hotkey, path, chat, mic, webcam.
- Users bring their own libraries with bun add.
- It is under active development and not a drop-in replacement for old Script Kit.
- Surfaces include launcher/list, prompt shells, actions menu, clipboard history, app/window/file tools, terminal prompt, notes, dictation, and permissions wizard.
- AI/context features include Agent Chat, structured desktop/UI context through protocol/MCP resources, and verifiable UI transactions via stable semantic IDs.
- Memory layer: Script Kit Brain stores local markdown under ~/.scriptkit/brain; Day Page is today's diary/memory surface; clipboard sediment quietly keeps useful copied URLs and repeated-copy signals without popup UI.

Strategic direction already chosen:
- Position as "a native programmable desktop workbench for scripts, agents, and local workflow memory."
- Programmability is the headline.
- Agent execution is the differentiating bet, but should be proof-oriented and trust-aware.
- Markdown memory is a supporting retention layer, not the main category.
- Avoid: Raycast clone, AI OS, second brain, no-code automation, productivity super-app, drop-in replacement, works everywhere, faster than Raycast, for everyone.
- Prefer: scripts, prompts, Bun-powered JS/TS, desktop context, Agent Chat, verifiable actions, markdown memory, macOS-first today, under active development.

Deliver a build-ready landing page plan, not generic advice:
1. Exact hero copy, CTA labels, and subcopy.
2. Section order and layout direction.
3. Three to five interactive Svelte components that are worth building for a marketing page, with data structures and behavior.
4. Competitive positioning copy that is honest but sharp.
5. Trust/anti-positioning copy.
6. A concrete visual direction: palette, typography feel, interaction motifs, but keep it lean enough for a small static build.
7. A complete list of page content blocks and microcopy.
8. Specific implementation notes for SvelteKit/mdsvex, including route count. Prefer a single-page site unless a second route adds clear value.

Constraints:
- Do not use stock imagery, external fonts, or heavy assets.
- Do not make a generic SaaS hero.
- The page should feel like a native command surface/workbench for developers, not a bubbly AI tool.
- It should be good enough to publish today as an early marketing artifact.
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
You are a senior product marketing, developer relations, and brand strategy panel. Develop a practical marketing strategy, branding direction, and messaging guide for this project. The goal is to carve out a defensible niche and find the right users.

Project: Script Kit GPUI

Context from the repo:
- Script Kit GPUI is a complete rewrite of Script Kit using Zed's GPUI framework.
- It is macOS-first today, with Linux/Windows planned.
- It combines the SDK and app into one repo.
- It is native Rust/GPUI with GPU-accelerated rendering, intended to feel very fast.
- Scripts run via Bun with modern JavaScript/TypeScript support.
- The SDK philosophy is narrower than old Script Kit: prompts are the core (`arg`, `div`, `editor`, `term`, `fields`, `form`, `drop`, `hotkey`, `path`, `chat`, `mic`, `webcam`), while users bring their own libraries with `bun add` instead of relying on a huge bundled helper global.
- It is explicitly NOT a drop-in replacement for old Script Kit.
- Default surfaces include: script launcher/list, expanded/mini modes, prompt shells, actions menu, clipboard history, emoji picker, process manager, window switcher, app launcher, notes, file search, terminal prompt, permissions wizard, dictation overlay.
- AI/context features: Agent Chat is the primary AI chat surface; scripts and agents can read structured desktop/UI context through protocol/MCP resources; agents can execute verifiable UI transactions via stable semantic IDs; context parts include launcher/UI state, files, clipboard, selected text/screens, etc.
- Memory layer / Script Kit Brain: local markdown memory under `~/.scriptkit/brain/{days,fragments,notes,trash}`. Day Page is today's diary/memory surface. Clipboard sediment keeps URLs and promotes repeated copies into day-page/fragments without intrusive popup UI.
- Audience likely includes automation-heavy developers, devrel/educators, local-first AI tinkerers, power users who write JS/TS, and people dissatisfied with generic launchers because they want programmable workflows.
- Competitive/adjoining tools might include Raycast, Alfred, Keyboard Maestro, Hammerspoon, Shortcuts, old Script Kit, Obsidian/Logseq for notes, and AI agent IDE/chat tools. Do not position as a clone of any of them.
- Current truth matters: it is a developer-facing, under-active-development project, not a polished mass-market productivity app yet.

Deliverables:
1. Clear positioning statement and category hypothesis.
2. Primary niche / ideal customer profiles, including who NOT to target yet.
3. Brand strategy: tone, personality, visual/interaction brand implications, names/taglines if useful.
4. Messaging pillars with proof points from the product.
5. Things to say, things to avoid saying, and terminology to prefer/avoid.
6. Competitive positioning against Raycast/Alfred/Keyboard Maestro/Hammerspoon/Shortcuts/Obsidian/AI chat tools, without being petty.
7. Content and launch strategy: channels, hooks, demos, docs, examples, onboarding, community loops.
8. A 30/60/90 day marketing plan with concrete assets to make.
9. Risks, anti-positioning, and validation experiments.
10. A concise one-page messaging guide that could be pasted into README/website planning.

Be opinionated and practical. Do not write generic startup marketing fluff. Assume the founder is technical and can ship demos, docs, scripts, and videos quickly. Make the strategy specific to Script Kit GPUI's combination of programmable launcher, local-first scripts, AI/context, and personal markdown memory.
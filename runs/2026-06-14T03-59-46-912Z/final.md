**Strategic Spine**

Script Kit GPUI should be positioned as a **developer-first programmable desktop workbench for people who write JavaScript/TypeScript**.

Not a Raycast clone. Not an AI OS. Not a second brain. Not a no-code automation tool. Not a polished cross-platform productivity app yet.

The strongest current category hypothesis:

> **A native programmable desktop workbench for scripts, agents, and local workflow memory.**

The practical positioning statement:

> **Script Kit GPUI is a macOS-first programmable desktop workbench for developers: write Bun-powered JS/TS scripts, run them from a native command surface, give scripts and agents structured desktop context, and keep useful workflow memory in local markdown files.**

The hierarchy should be:

1. **Programmability is the headline.**
2. **Agent execution is the differentiating bet, but only after proof-oriented demos show it working reliably.**
3. **Markdown memory is a supporting retention layer, not the main category.**

**Primary Niche**

Target now:

- **Automation-heavy Mac developers** who already write scripts and want a faster path from idea to desktop tool.
- **Raycast/Alfred power users who hit the ceiling of configuration/extensions** and would rather write TS.
- **Hammerspoon/Keyboard Maestro-style automators who want modern JS/TS, prompts, native UI, and agent context.** “Hammerspoon, but modern” is a useful developer-channel hypothesis, but not the official brand headline yet.
- **Local-first AI tinkerers** who want agents to read real desktop context and perform verifiable actions.
- **Devrel/educator builders** who can turn workflows into demos, examples, and teaching material.

Do not target yet:

- Non-coders.
- Generic productivity users.
- “Free Raycast” shoppers.
- Enterprise/MDM buyers.
- Users wanting a polished cross-platform launcher.
- Old Script Kit users expecting drop-in migration.
- People mainly shopping for clipboard, emoji, notes, or window-switching utilities.

Old Script Kit users are high-interest but high-risk. The message should be:

> **Same spirit, new architecture. Script Kit GPUI is not a drop-in replacement. The SDK is narrower, prompt-first, and built around Bun-powered JS/TS plus user-installed libraries.**

**Brand Direction**

Personality: precise, technical, calm, fast, local, toolmaker-oriented.

The brand should feel like a workbench, not a dashboard. A serious native tool for people who build their own tools.

Visual and interaction implications:

- Keep the UI low-chrome, keyboard-first, fast-feeling, and native.
- Let speed be felt through interaction, not claimed with unsupported comparisons.
- Use GPUI as technical proof for developers, not as the main user-facing reason to care.
- Show real scripts, real prompts, real files, real transactions, and real markdown memory.
- Avoid playful AI assistant tropes, productivity-coach language, and vague magic.

Taglines worth testing:

- **Code your launcher.**
- **Your desktop, scriptable.**
- **A native workbench for scripts, agents, and local memory.**
- **Run the workflows only you can write.**
- **Small scripts. Native UI. Local context.**

Best README/website hero candidate:

> **Code your desktop workflows in TypeScript. Run them from a native command surface. Give agents context when you choose. Keep the memory in markdown you own.**

**Messaging Pillars**

1. **Programmable, Not Merely Configurable**

Proof points:
- Scripts run with Bun.
- Users write modern JS/TS.
- Prompt APIs are the core: `arg`, `div`, `editor`, `term`, `fields`, `form`, `drop`, `hotkey`, `path`, `chat`, `mic`, `webcam`.
- Users bring their own libraries with `bun add`.

Message:
> Build the tool you wanted your launcher to have.

2. **Native Desktop Surface**

Proof points:
- Rust/GPUI app.
- GPU-rendered UI.
- macOS-first native interaction.
- Launcher/list, prompt shells, mini/expanded modes, actions, terminal prompt, app launcher, window switcher, file search.

Bounded language:
> Native Rust/GPUI desktop UI with GPU-rendered surfaces.

Avoid:
> Faster than Raycast/Alfred.

Do not claim superiority until benchmarks exist.

3. **Agents With Desktop Context**

This is a differentiating bet, not yet the whole brand promise.

Proof points from project context:
- Agent Chat is the primary AI surface.
- Scripts and agents can read structured desktop/UI context through protocol/MCP resources.
- Context can include launcher/UI state, files, clipboard, selected text/screens, and related desktop state.
- Agents can execute verifiable UI transactions through stable semantic IDs.

Message:
> Chat is useful. Actions are useful when they are inspectable, scoped, and verifiable.

Launch requirement:
Before heavily marketing this, ship one reliable demo and one trust explainer.

4. **Local Workflow Memory, Not a Second Brain**

Proof points:
- Script Kit Brain stores local markdown under `~/.scriptkit/brain/{days,fragments,notes,trash}`.
- Day Page is today’s diary/memory surface.
- Clipboard sediment can quietly preserve useful copied URLs and repeated-copy signals without intrusive popup UI.

Message:
> Memory for scripts and agents, stored as files you own.

Avoid:
> Obsidian replacement, second brain, knowledge management platform.

5. **Focused SDK, Modern Ecosystem**

Proof points:
- Narrower than old Script Kit.
- Prompt-first surface.
- No huge bundled helper global as the core identity.
- Users add libraries through Bun.

Message:
> Less magic global API, more normal JS/TS.

**Say / Avoid / Prefer**

Say:

- programmable desktop workbench
- programmable launcher
- scripts
- prompts
- Bun-powered JS/TS
- local-first
- desktop context
- Agent Chat
- verifiable actions
- semantic IDs
- markdown memory
- Day Page
- Script Kit Brain
- macOS-first
- under active development

Avoid:

- Raycast clone
- open-source Raycast
- AI OS
- second brain
- no-code automation
- productivity super-app
- works everywhere
- drop-in replacement
- seamless migration
- faster than Raycast
- for everyone
- intelligent workflow magic

Prefer:

- “scripts” over “extensions”
- “prompts” over “widgets”
- “desktop context” over “AI sees everything”
- “verifiable actions” over “autonomous control”
- “markdown memory” over “second brain”
- “macOS-first today” over “cross-platform soon”
- “under active development” over “production-ready productivity suite”

Demote jargon:

- “sediment” should be explained only when needed.
- “fragments” should not be above the fold.
- “GPUI” should appear in developer/technical sections, not the main promise.

**Competitive Positioning**

Raycast / Alfred:
> Raycast and Alfred are polished launchers with mature ecosystems. Script Kit GPUI is for developers who want to write their own local tools in JS/TS and wire them into a native command surface.

Keyboard Maestro:
> Keyboard Maestro is deep macro automation. Script Kit GPUI is code-native, prompt-first, and better suited to developers who want scripts, packages, local context, and agent-readable workflows.

Hammerspoon:
> Hammerspoon is the closest tactical comparison for developer channels: programmable desktop control. Script Kit GPUI’s hypothesis is a more modern workbench: TypeScript/Bun, native prompts, Agent Chat, and markdown workflow memory. Use this comparison in demos and posts, but do not make the homepage depend on it.

Shortcuts:
> Shortcuts is approachable no-code automation. Script Kit GPUI is explicitly for people willing to write code.

Obsidian / Logseq:
> Obsidian and Logseq are knowledge bases. Script Kit Brain is local workflow memory that scripts and agents can read and write. It should complement notes apps, not compete with them.

AI chat tools:
> ChatGPT/Claude-style apps are conversation surfaces. Script Kit GPUI’s differentiating bet is giving agents structured desktop context and verifiable actions. Prove this with demos before making it the main public claim.

Old Script Kit:
> Script Kit GPUI shares the spirit of programmable personal automation, but it is a rewrite with a narrower SDK and different architecture. Migration requires care.

**Content And Launch Strategy**

Primary launch motion: devrel, not broad product marketing.

The launch should be built around proof assets:

1. **Hero demo**
   - 60-90 seconds.
   - Build a small TypeScript prompt.
   - Add a package with `bun add`.
   - Run it from the native launcher.
   - Optionally show Agent Chat using context or executing a verifiable action.
   - End by showing the script file and, if relevant, markdown memory.

2. **Tested recipe library**
   - Treat recipes as product surface, not disposable content.
   - Start with 10, then grow to 30.
   - Keep them versioned and smoke-tested where practical.

Recipe categories:
- “First useful scripts”
- “Raycast/Alfred workflows, but code”
- “Hammerspoon-style desktop control”
- “Agent context recipes”
- “Clipboard and Day Page workflows”
- “Devrel/content workflows”
- “File/window/process workflows”

3. **Migration notes**
   - Old Script Kit to GPUI: what ports cleanly, what does not, what to wait on.
   - Hammerspoon to Script Kit GPUI: examples in TS.
   - Raycast/Alfred power-user guide: when Script Kit GPUI is a fit and when it is not.

4. **Privacy and trust explainer**
   Publish before heavily marketing desktop context or agent actions.

It should answer:
- What stays local?
- What can scripts read?
- What can Agent Chat read?
- What can agents execute?
- What requires confirmation?
- Are actions logged or inspectable?
- What leaves the machine?
- How do users disable context sources?

5. **README / website messaging pass**
   The README should say plainly:
   - macOS-first today
   - under active development
   - JS/TS required for serious use
   - not a drop-in replacement for old Script Kit
   - Linux/Windows planned only when shipped builds exist

Channels:

- YouTube: best for proof demos.
- GitHub README/docs: primary conversion surface.
- X/Twitter: short workflow clips.
- Hacker News: one honest launch, not hype.
- Discord/GitHub Discussions: early user loop.
- Blog/devlog: weekly build notes and recipes.
- Devrel friends/educators: high-leverage early users.

Avoid broad Product Hunt-style positioning until onboarding is strong.

**Concrete Demo Concepts**

Ship these before broad launch:

1. **“Code Your Launcher in 60 Seconds”**
   - Create a TS script.
   - Show `arg`.
   - Run it instantly from Script Kit GPUI.

2. **“Bun Add Turns a Prompt Into a Tool”**
   - Add a real npm package.
   - Use it inside a prompt.
   - Show no special plugin packaging.

3. **“Agent Chat With Receipts”**
   - Agent reads selected text or launcher context.
   - Agent proposes a verifiable action.
   - User confirms.
   - Action result is visible.

4. **“Clipboard to Day Page, Quietly”**
   - Copy useful URLs.
   - Show local markdown output.
   - Emphasize no intrusive popup.

5. **“Hammerspoon Workflow, Rebuilt in TypeScript”**
   - Pick one concrete window/app/file workflow.
   - Keep it small and honest.

6. **“Old Script Kit Migration: What Actually Ports”**
   - Show one script that ports cleanly.
   - Show one that needs rewrite.
   - Earn trust by being direct.

**30 / 60 / 90 Day Plan**

Days 0-30: Nail the wedge

Assets to ship:
- README messaging rewrite.
- One-page website/landing copy.
- 60-90s hero demo.
- “Not a drop-in replacement” migration note.
- Privacy/context/agent trust explainer.
- First 10 tested recipes.
- First-run guide: create and run a script in under 5 minutes.

Validation:
- Recruit 10 JS/TS Mac developers.
- Ask each to build one personal workflow.
- Measure:
  - install completed
  - first script created
  - first script run
  - first script edited after initial run
  - time to first useful workflow

Success signal:
- At least 6/10 create a useful script within 30 minutes.

Days 31-60: Prove repeatability

Assets to ship:
- 20-30 recipe cookbook.
- 5 short video demos.
- Hammerspoon-to-Script-Kit-GPUI guide.
- Raycast/Alfred power-user guide.
- Old Script Kit migration matrix.
- Agent Chat/context cookbook if the demo path is stable.
- Weekly “workflow of the week.”

Validation:
- Track weekly active script-writers, not installs alone.
- Track recipe copy/use rate.
- Track users who create 2+ scripts.
- Track first Agent Chat context use separately from general AI curiosity.

Success signal:
- A meaningful share of users write or edit scripts in week two.
- At least a few users contribute recipes or ask implementation-level questions.

Days 61-90: Choose the public wedge

Run landing-page/message tests:

A. “Code your launcher.”
B. “Programmable desktop workbench.”
C. “Agents that act on your desktop.”
D. “Hammerspoon, but modern” as a developer-channel test only.

Do not pick the winner by clicks. Pick by activation:

- weekly active script-writers
- scripts created per active user
- repeat launcher usage
- recipe reuse
- Agent Chat context usage
- Day Page / Brain captures
- support burden per activated user

Assets to ship:
- Refined homepage based on activation data.
- Public roadmap.
- Contributor guide for recipes.
- First “script jam” or live workshop.
- Benchmarks only if measured.
- More memory storytelling only if retention data supports it.

**Risks And Controls**

Risk: Broad messaging attracts the wrong users.
Control: Strong anti-positioning above the fold.

Risk: Agent execution creates trust concerns.
Control: Publish privacy/trust docs first. Demo confirmation, scoping, and receipts.

Risk: Performance claims backfire.
Control: Say “native GPU-rendered UI.” Do not say faster than competitors until benchmarked.

Risk: Old Script Kit users expect compatibility.
Control: Migration guide with explicit compatibility matrix.

Risk: Recipes rot.
Control: Treat demo scripts as product surface. Version them. Test the important ones.

Risk: Too many terms.
Control: Use a strict vocabulary. Introduce only scripts, prompts, Agent Chat, desktop context, verifiable actions, and markdown memory early.

Risk: Cross-platform expectation debt.
Control: Say macOS-first today. Mention Linux/Windows only as planned, not promised, and not as launch copy.

Risk: Memory sounds like another notes app.
Control: Frame it as workflow memory for scripts and agents.

Risk: Business model uncertainty.
Control: Do not optimize for a pricing funnel yet. Optimize for activated script-writers and high-signal user conversations.

**Validation Metrics**

North-star activation metric:

> **Weekly active script-writers**: users who create or edit scripts in a given week.

Supporting metrics:

- first script created
- first script run
- first useful workflow completed
- scripts created per user
- scripts edited after first run
- recipe copied and run
- Agent Chat context used
- verifiable action completed
- Day Page / markdown memory capture
- returning launcher usage
- migration guide completion
- support issues per activated user

Installs are secondary. Stars are tertiary. Writers are the moat.

**One-Page Messaging Guide**

```md
# Script Kit GPUI Messaging Guide

## Positioning

Script Kit GPUI is a macOS-first programmable desktop workbench for developers.
Write Bun-powered JavaScript/TypeScript scripts, run them from a native command
surface, give scripts and agents structured desktop context, and keep useful
workflow memory in local markdown files.

It is developer-facing, under active development, and not a drop-in replacement
for old Script Kit.

## Category Hypothesis

Programmable desktop workbench.

Use “launcher” as a familiar entry point, but do not position the product as a
Raycast clone. Use “agent” and “memory” as differentiators, not as the whole
category.

## Best For

- Developers who write JS/TS.
- Automation-heavy Mac users.
- Raycast/Alfred users who want code-first workflows.
- Hammerspoon/Keyboard Maestro users who want modern scripts and prompts.
- Local-first AI tinkerers.
- Devrel and educator-builders.

## Not For Yet

- Non-coders.
- Generic productivity users.
- Enterprise fleets.
- Users wanting cross-platform polish today.
- Users expecting old Script Kit drop-in compatibility.
- People only looking for a clipboard manager, emoji picker, or notes app.

## Core Promise

Turn repeated desktop work into fast local scripts, with context and markdown
memory when useful.

## Pillars

1. Programmable, not merely configurable.
   JS/TS scripts, Bun runtime, prompt-first SDK, `bun add` for libraries.

2. Native desktop surface.
   Rust/GPUI, GPU-rendered UI, macOS-first command and prompt surfaces.

3. Agent context and verifiable actions.
   Agent Chat can use structured desktop context and semantic UI targets.
   Treat this as a proof-led differentiator.

4. Local markdown workflow memory.
   Script Kit Brain and Day Page store useful traces in files users own.

5. Focused SDK.
   Narrower than old Script Kit, prompt-first, less global magic.

## Say

programmable desktop workbench
programmable launcher
scripts
prompts
Bun-powered JS/TS
local-first
desktop context
Agent Chat
verifiable actions
semantic IDs
markdown memory
Day Page
macOS-first today
under active development

## Avoid

Raycast clone
open-source Raycast
AI OS
second brain
no-code automation
productivity super-app
works everywhere
drop-in replacement
seamless migration
faster than Raycast
for everyone

## Competitive Truth

Raycast and Alfred are polished launchers. Script Kit GPUI is for developers who
want to build their own tools.

Keyboard Maestro and Shortcuts are automation systems. Script Kit GPUI is
code-native and prompt-first.

Hammerspoon is a useful developer comparison, but Script Kit GPUI adds TS/Bun,
native prompts, Agent Chat, and local markdown memory.

Obsidian is a notes app. Script Kit Brain is workflow memory for scripts and
agents.

AI chat apps converse. Script Kit GPUI’s differentiating bet is contextual,
verifiable action on the desktop.

## CTA

Build your first script.
```

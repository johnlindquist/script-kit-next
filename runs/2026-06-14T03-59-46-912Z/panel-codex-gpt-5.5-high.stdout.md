## Role Findings

**Positioning statement:** Script Kit GPUI is a native, keyboard-first automation workbench for developers who want their launcher, scripts, AI context, and personal markdown memory in one programmable local surface.

**Category hypothesis:** Not “Raycast alternative” and not “AI notes app.” The better category is **programmable desktop command center**: a fast native launcher plus JS/TS prompt SDK plus local context/memory substrate for agent-assisted work.

**Primary niche:**  
Automation-heavy developers, devrel/educators, technical creators, local-first AI builders, and power users who already write small scripts and feel constrained by generic launchers. The wedge is “I can turn any repeated desktop workflow into a fast native command, with context and memory attached.”

**Do not target yet:**  
Mass-market productivity users, teams expecting polished enterprise admin, old Script Kit users needing drop-in compatibility, no-code automation buyers, and users primarily shopping for prettier Raycast extensions.

**Brand direction:**  
Fast, focused, native, technical, local. The repo’s design guidance already says “Fast. Focused. Minimal.” and treats speed as the design, with three primary keys: Run, Actions, Agent Chat in [.impeccable.md](/Users/johnlindquist/dev/script-kit-gpui/.impeccable.md:4). Keep that. The brand should feel like a precision instrument, not a playful AI toy.

**Useful tagline directions:**

- “Your programmable command center.”
- “A native launcher for scripts, agents, and memory.”
- “Run the workflows only you can write.”
- “Local automation, context-aware.”

**Messaging pillars:**

1. **Programmable launcher, not a static launcher**  
   Proof: script list, prompt APIs, Actions menu, global hotkeys, built-ins, Bun scripts.

2. **Native speed for daily use**  
   Proof: Rust/GPUI, GPU rendering, macOS-first native interaction, keyboard-first chrome in [README.md](/Users/johnlindquist/dev/script-kit-gpui/README.md:7).

3. **Focused SDK, modern JS/TS ecosystem**  
   Proof: prompts are the core; users bring libraries with `bun add`; not a huge helper global.

4. **AI with real desktop context**  
   Proof: Agent Chat is the primary AI surface, `kit://context`, semantic IDs, deterministic UI transactions in [README.md](/Users/johnlindquist/dev/script-kit-gpui/README.md:419).

5. **Memory that lives in files**  
   Proof: Brain substrate under `~/.scriptkit/brain`, Day Page, fragments, clipboard sediment in [GLOSSARY.md](/Users/johnlindquist/dev/script-kit-gpui/GLOSSARY.md:68).

## Evidence And Assumptions

Repo evidence supports a product that is already conceptually strong but still developer-facing: [docs/README.md](/Users/johnlindquist/dev/script-kit-gpui/docs/README.md:1) defines it as “a native command palette, scripting runtime, Agent Chat host, and local automation/MCP surface in one app.”

I am assuming the near-term marketing goal is not broad adoption, but finding 50-500 highly aligned users who will write scripts, report sharp issues, make examples, and help define the category.

I am also assuming the most defensible niche is the combination of four things together: native command palette, Bun-powered JS/TS scripts, structured desktop context for agents, and markdown memory. Any one of those alone has stronger incumbents.

Terminology to prefer: **Script Kit GPUI**, **scripts**, **prompts**, **Agent Chat**, **desktop context**, **semantic IDs**, **Day Page**, **Script Kit Brain**, **clipboard sediment**, **local markdown memory**, **programmable launcher**.

Terminology to avoid: **Raycast clone**, **AI operating system**, **second brain app**, **drop-in replacement**, **no-code automation**, **productivity super-app**, **works everywhere**.

## Failure Modes

The biggest risk is confused category gravity. If the site says “launcher,” people compare it to Raycast polish. If it says “AI chat,” people compare it to ChatGPT/Cursor. If it says “notes,” people compare it to Obsidian. The defensible answer is the integration: programmable launcher plus context plus memory.

Second risk: over-promising maturity. The current truth is under active development, macOS-first, and explicitly not backwards-compatible with old Script Kit. Say this plainly. The right early users will respect it.

Third risk: AI features sounding vague. “Context-aware agents” is weak unless demos show stable semantic UI transactions, selected text, browser/window context, and receipts.

Fourth risk: old Script Kit confusion. The migration message should be direct: “same spirit, narrower SDK, not drop-in compatible.”

Fifth risk: memory feature creep. Script Kit Brain should not be marketed as an Obsidian competitor. It is operational memory for scripts, clipboard, day notes, and agents.

## Recommendation

Build the public story around this sentence:

**Script Kit GPUI is a native programmable launcher for developers: write Bun-powered JS/TS scripts, run them from a fast GPUI command palette, hand them desktop context, and keep useful traces in local markdown memory.**

Competitive framing:

- **Raycast/Alfred:** polished launchers with extension ecosystems; Script Kit GPUI is for people who want to write their own workflows as local scripts.
- **Keyboard Maestro/Shortcuts:** powerful automation systems; Script Kit GPUI is code-native, developer-shaped, and prompt/UI-first.
- **Hammerspoon:** scriptable desktop control; Script Kit GPUI adds a modern launcher, prompt SDK, Agent Chat, and memory layer.
- **Obsidian/Logseq:** excellent knowledge bases; Script Kit Brain is not a knowledge app, it is local workflow memory.
- **AI chat tools:** great at conversation; Script Kit GPUI gives agents structured desktop context and verifiable UI targets.

30/60/90 plan:

**First 30 days:** ship the positioning page, README messaging pass, 5 canonical demo videos, and a “not a drop-in replacement” migration note. Demos: build a script, clipboard to Day Page, Agent Chat with selected text/browser context, semantic UI automation receipt, Bun package workflow.

**Days 31-60:** publish a cookbook of 20 scripts. Organize by jobs: devrel publishing, repo triage, daily capture, file/window workflows, local AI context. Add “copy this script” docs and short videos for each.

**Days 61-90:** launch an early-user loop: weekly build notes, Discord/GitHub discussions, “workflow of the week,” public roadmap, and a contributor guide for scripts/prompts/context recipes. Measure activation by first script created, first Agent Chat context use, first Day Page capture, and repeat launcher usage.

Validation experiments:

- Landing page A: “programmable launcher” vs B: “local AI automation workbench.”
- Demo hook A: “replace five shell aliases with native prompts” vs B: “give your AI desktop context.”
- Onboarding A: first script first vs B: first built-in workflow first.
- Pricing/community signal: ask early users whether they want paid app, sponsorware, pro cloud sync later, or open-core.

One-page messaging guide:

**What it is:** A native programmable command center for developers.  
**Who it is for:** People who write JS/TS and want faster personal workflows.  
**Core promise:** Turn repeated desktop work into fast local commands, with context and memory when useful.  
**Proof:** Rust/GPUI speed, Bun scripts, focused prompt SDK, Agent Chat, MCP context, semantic UI IDs, Day Page, local markdown Brain.  
**Voice:** precise, technical, calm, fast.  
**Say:** “local,” “programmable,” “native,” “scriptable,” “context-aware,” “markdown memory.”  
**Avoid:** “Raycast clone,” “AI OS,” “second brain,” “drop-in replacement,” “for everyone.”  
**Hero line:** “A native launcher for scripts, agents, and memory.”  
**CTA:** “Build your first script.”

## Self Score

8.5/10. The strategy is specific and defensible, with a clear category wedge and practical launch path. The remaining uncertainty is market language: “programmable command center” needs validation against real early-user reactions.



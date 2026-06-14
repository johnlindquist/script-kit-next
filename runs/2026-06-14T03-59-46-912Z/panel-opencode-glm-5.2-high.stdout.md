## Role Findings

As the pragmatist, my read is: the biggest risk here is **scope bloat**, not insufficient strategy. Script Kit GPUI sits on five axes (programmable launcher, native/GPU speed, JS/TS+Bun, local-first AI context, markdown memory) — and the natural temptation is to market all five. That dilutes everything. The smallest strategy that wins: pick **one wedge**, make **one demo that converts**, and defer the rest until the data says double down. I've written the plan to be aggressively cuttable; everything marked *defer* should not be built until a validation signal earns it.

The other pragmatist call: this is a **devrel play, not a product-marketing play**, right now. The repo is developer-facing and under active development. Polished "productivity app" marketing will oversell and create churn. Lead with honesty, recipes, and working code. Brand polish comes after 1,000 real script-writing users.

## Evidence And Assumptions

**Evidence (from provided context):**
- Unique combinatorial advantage: no competitor combines *code-your-own-tools in TS* + *agents that act on the desktop via verifiable semantic IDs* + *local markdown memory*. Raycast/Alfred/KM/Hammerspoon each miss at least two of these.
- "Explicitly NOT a drop-in replacement" + "narrower SDK philosophy" = the founders have already made hard scoping calls. Marketing should mirror that discipline.
- Native Rust/GPUI + GPU rendering is a real, demonstrable speed claim — and speed is the cheapest proof point to show on video.
- Surfaces list is broad (launcher, clipboard, emoji, process mgr, window switcher, notes, terminal, dictation…) — this is the scope-bloat danger zone. Most of these are table-stakes, not differentiators.

**Assumptions (flag to validate):**
- The MCP/UI-transaction layer is stable enough to demo reliably in a 60-second video. (If not, **demote pillar #3 and lead with speed + programmability.**)
- "Bun + `bun add`" onboarding works in under 5 minutes for a fresh user. (If not, the whole devrel funnel leaks.)
- The Brain/Day Page is genuinely useful day-to-day, not just novel. (Unvalidated — treat as experiment, not a pillar, until used for 30+ days.)

## Failure Modes

1. **"Five pillars" marketing.** Equal-weighting speed + programmability + agents + memory + native → nobody remembers any of it. *Mitigation: one hero claim, four proof points.*
2. **Positioning as "the open/free Raycast."** You lose on polish and invite unfavorable side-by-sides. Raycast is the wrong fight.
3. **Overselling AI to the local-LLM crowd before transactions are reliable.** One viral video of an agent doing the wrong thing wrecks the trust story. *Mitigation: lead with "verifiable" and show the confirmation step in every demo.*
4. **Marketing the broad surface list.** Clipboard manager / emoji picker / window switcher are commodity checks-in-a-box, not reasons to switch. Don't put them above the fold.
5. **Treating the rewrite's "not a drop-in replacement" as a problem to apologize for.** It's actually a clean positioning asset — own it.
6. **Building a full brand/visual system before users.** Expensive, unverifiable, and likely wrong. Defer.

## Recommendation

### 0. Meta-recommendation: the discipline
- **One wedge now:** *programmable launcher where agents act on your desktop.* Memory is the B-plot.
- **One demo that must exist before anything else:** *"30 lines of TS → a custom tool an agent can drive, on your real desktop, in under a minute."* If you ship nothing else from this doc, ship that.
- **One number to track:** weekly active script-writers (people who authored/edited a script), not installs. Installs flatter; writers are the moat.

---

### 1. Positioning & category hypothesis
> **Script Kit GPUI is a programmable, local-first desktop automator. You write small TypeScript tools; your agents run them against the desktop with verifiable actions; your memory lives in markdown you own. It's the workbench for people who'd rather code their launcher than configure one.**

- Category hypothesis: not "launcher," not "AI assistant." **"Programmable desktop runtime."** That framing sidesteps every Raycast comparison and makes the code-first identity the default lens.
- The cleanest one-line hook to test: **"Hammerspoon, but modern — native, TypeScript, and your agents can actually do things."**

### 2. Ideal customer profiles (and who NOT to target)

**Target now (ranked):**
1. **The Hammerspoon graduate** — automation dev, sick of Lua + 2010-era UI, wants TS and native chrome. *Easiest conversion; they already get the mental model.*
2. **The local-first AI tinkerer** — runs Claude/ChatGPT desktop, frustrated that chat can't act on files/windows/clipboard. *Highest enthusiasm, best word-of-mouth.*
3. **The ex–Script Kit power user** — knows the old SDK, will accept "narrower but modern" if migration is real. *Existing equity; convert fast.*
4. **The devrel/educator-builder** — wants a stage for demos/lessons. *Force multiplier, not a volume segment.*

**Do NOT target yet:**
- Non-technical Mac users (they want polish; you'd over-promise).
- Enterprise/MDM fleets (support + security surface you can't staff).
- "Free Raycast" seekers (wrong value prop, high churn).
- Standalone clipboard/emoji/window-tool shoppers (commodity; they'll leave when a shinier one appears).

### 3. Brand strategy
- **Tone:** opinionated, dry-witty, technically honest, anti-bloat. A toolsmith talking to other toolsmiths — not a productivity coach.
- **Personality:** "the workbench," not "the dashboard." Calm, fast, yours-to-reshape. Local-first as a value, not a feature.
- **Name:** keep **Script Kit** — it has equity and says exactly what it is. Append "GPUI" only in dev contexts; drop it in user-facing brand. *Defer any rename indefinitely.*
- **Taglines to test (pick one, kill the rest):**
  - "Code your launcher."
  - "Your desktop, scriptable. Your agents, accountable."
  - "Small tools. Native speed. Yours."
- **Visual/interaction brand implications (don't build a system yet — just hold these principles):** native chrome over custom skins; speed as the felt brand; dark, calm, low-chrome; "markdown-as-truth" visible in the memory surfaces. *Defer: logo refresh, type system, palette spec — until 1k active writers.*

### 4. Messaging pillars with proof points

| Pillar | Proof point (cheap to show) |
|---|---|
| **P1 — Programmable, not configurable.** It's code. | A prompt in ~10 lines of TS; `bun add` any lib. |
| **P2 — Native & fast.** Rust + GPUI, GPU-rendered. | Side-by-side open/search latency clip. |
| **P3 — Agents that act, not just chat.** | A 30-line tool an agent drives against a real window/file, with a visible confirmation step. |
| **P4 — Memory that's yours.** | `ls ~/.scriptkit/brain` — show it's plain markdown, portable, greppable. |
| **P5 — Bring your own stack.** No captive helper global. | npm install + import in a script; no vendor lock-in. |

**Pillar hierarchy:** P1 is the hero. P3 is the differentiator. P2/P5 are reassurance. **P4 is currently a B-plot** — feature it, don't lead with it, until retention data earns it a promotion.

### 5. Say this / Avoid this / Terminology

**Say:** "programmable launcher," "code your tools," "agents that act on your desktop," "verifiable actions," "local-first," "your memory, in markdown you own," "native Rust/GPUI," "small tools."

**Avoid:** "Raycast alternative/free/open Raycast," "AI-powered productivity," "seamless," "the everything app," "boost your workflow," "intelligent/smart" (empty), "seamlessly integrates," "supercharge."

**Prefer → Avoid (terminology discipline):**
- "Programmable desktop runtime" → not "launcher app"
- "Desktop agents" / "agents that act" → not "AI assistant"
- "Markdown memory" / "Day Page" → not "second brain"
- "Scripts / tools" → not "extensions" (that word cedes ground to Raycast/VS Code)
- "Verifiable transactions" / "semantic IDs" → not "automation" (too generic)

### 6. Competitive positioning (honest, not petty)
- **vs Raycast:** Raycast wins on polish and a store; SK wins if you want to *build* the tools and want agents that act. Not a replacement — a different product for people who'd open a code editor, not a settings pane.
- **vs Alfred:** Alfred workflows are visual macros; SK is code. Alfred is mature; SK is modern + agentic.
- **vs Keyboard Maestro:** KM is deep macro automation with dated UI; SK is code-native, modern, and agent-driven. KM users who've hit the ceiling are prime converts.
- **vs Hammerspoon:** the single best comp. Hammerspoon = Lua + 2010 UI; SK = TS + native GPU + memory + agents. **"Hammerspoon but modern" is your strongest borrowed-frame hook.**
- **vs Shortcuts:** Shortcuts is no-code and shallow; SK is full code. Different audience entirely — don't fight here.
- **vs Obsidian:** Obsidian is a notes app; SK's Brain is *memory that scripts and agents read/write*. Don't chase note-takers.
- **vs AI chat desktop apps (ChatGPT/Claude desktop):** they chat; SK *acts*, on your real desktop, with your context, verifiably. That's the cleanest anti-positioning line you have — use it.

### 7. Content & launch strategy
- **Hero asset (must-have):** one 60–90s demo — "build a tool, then let an agent run it." This is the whole funnel. Founder ships it personally; voice + screen, no motion graphics.
- **Recipes library is the real moat.** 30 copy-paste scripts before launch. Categorize: Hammerspoon ports, desktop-agent recipes, memory/day-page recipes. This is what makes the SDK "feel" rich without the old bundled global.
- **Migration guide: Hammerspoon → SK** (table-stakes for ICP #1) and **old Script Kit → SK** (manages expectations, not promises parity).
- **Channels, ranked:** YouTube (demos + recipes) ≫ X/Twitter ≫ HN (one honest launch) ≫ Discord/community ≫ blog. Skip TikTok/Instagram for now.
- **Onboarding promise:** `kit create` → working prompt open in <2 min. If you can't keep that promise, fix the product before marketing.
- **Community loop:** weekly "script of the week" → surfaces user recipes → becomes docs → becomes the next demo. Cheap, compounding, founder-runnable solo.

### 8. 30/60/90 plan (concrete assets)

**Days 0–30 — Establish the wedge**
- README rewrite against the messaging guide (§10).
- Hero demo video (60–90s).
- Honest "SK vs X" comparison page (Raycast/Alfred/KM/Hammerspoon) — defer polish, ship candor.
- First 10 recipes + `kit create` scaffold.
- Community space (Discord) + one pinned "start here."
- *Defer: tagline A/B, visual brand, website rebuild.*

**Days 31–60 — Prove the differentiator**
- **"30 scripts in 30 days"** content series (YouTube shorts + repo PRs). This is the devrel engine.
- MCP/agent cookbook: 10 "agent that acts" recipes with the confirmation step shown.
- Hammerspoon migration guide + 5 named port recipes.
- One HN/launch post built around the agent-acts-on-desktop demo, not the feature list.
- *Defer: Linux/Windows messaging (not real yet — do not promise).*

**Days 61–90 — Pick the winner**
- Run the validation experiment (§9): which of three framings (Hammerspoon-modern / agents-act / markdown-memory) drove the most active writers? **Double down on one.**
- Contributor/recipe program — 2–3 power users as named contributors.
- First virtual "script jam" (1hr, live) — generates content + community proof.
- Memory/Day-Page storytelling push *only if* retention data says memory drove stickiness.
- *Defer: pricing/packaging, enterprise page, integrations marketplace.*

### 9. Risks, anti-positioning, validation experiments
- **Anti-positioning (do not claim):** Raycast polish parity; production-readiness for non-devs; cross-platform availability (until shipped); "drop-in replacement" for old SK; broad consumer appeal.
- **Top risks:** (a) breadth dilutes the wedge; (b) an agent does the wrong thing on camera and breaks trust; (c) onboarding friction leaks the funnel; (d) the "narrower SDK" alienates old-SK users expecting the kitchen sink.
- **Validation experiment (run in days 0–90, decide at day 90):** ship three landing-page framings — **A:** "Hammerspoon but modern," **B:** "agents that act on your desktop," **C:** "your markdown memory." Funnel all three to the same install. Measure *weekly active script-writers* per framing at day 90, not clicks. The winner becomes the permanent wedge; the losers become proof points. This is the single most important marketing decision, and it should be made by data, not opinion (including mine).

### 10. One-page messaging guide (paste into README/website)

```
SCRIPT KIT GPUI — MESSAGING GUIDE (v1, dev-facing)

Positioning: A programmable, local-first desktop runtime. You write small
TypeScript tools; your agents run them against the desktop with verifiable
actions; your memory lives in markdown you own. For people who'd rather
code their launcher than configure one.

Category: Programmable desktop runtime (not "launcher", not "AI assistant").

One-line hook: "Hammerspoon, but modern — native, TypeScript, and your
agents can actually do things."

Pillars (hero → reassurance):
1. Programmable, not configurable. It's code. JS/TS + Bun + `bun add`.
2. Agents that act, not just chat. Verifiable semantic-ID transactions.
3. Native & fast. Rust + GPUI, GPU-rendered, real macOS.
4. Bring your own stack. No captive helper global, no lock-in.
5. Memory that's yours. Plain markdown at ~/.scriptkit/brain.

Say: programmable launcher · code your tools · agents that act ·
verifiable actions · local-first · markdown memory · native Rust/GPUI.
Avoid: Raycast alternative/free · AI-powered productivity · seamless ·
second brain · intelligent/smart · supercharge · extensions.

Target now: Hammerspoon graduates · local-first AI tinkerers ·
ex–Script Kit power users · devrel/educator builders.
Not yet: non-technical users · enterprise/MDM · free-Raycast seekers ·
standalone clipboard/emoji/window shoppers.

Competitive truth: Raycast = polish + store; SK = build-your-own + agents
that act. ChatGPT/Claude desktop = chat; SK = act, verifiably, on your
desktop. Obsidian = notes; SK Brain = memory scripts/agents read/write.

Current truth: developer-facing, under active development. Lead with
honesty and working recipes, not polish. Be the workbench.
```

## Self Score

**8/10.** Delivered all ten deliverables in cuttable, founder-actionable form, with explicit defer-lists and a cheap validation experiment — that's the pragmatist job done. Loses points on two: (1) I worked from provided context rather than cheaply verifying load-bearing claims against the repo (e.g., that the MCP/transaction layer is demo-stable — if it isn't, pillar #3 collapses and the whole wedge shifts); (2) the "agents that act" wedge is my opinionated bet, and the honest pragmatist move is the day-90 experiment, not the recommendation. If forced to pre-commit, I'd hedge: build the agent demo first — if you can make it reliable on camera, it's the wedge; if not, fall back to "Hammerspoon but modern + native speed," which is always true and verifiable today.

## Consensus

Most usable agents converged on a clear core: Script Kit GPUI should be positioned as a developer-first programmable desktop surface, not as a Raycast clone, generic launcher, AI chat app, notes app, or mass-market productivity tool.

Strong shared points:

- The defensible niche is automation-heavy developers and technical power users who write JS/TS and want local, scriptable workflows.
- The product should be honest about its current state: macOS-first, under active development, not a drop-in replacement for old Script Kit.
- The main promise should be programmability first: write small Bun-powered scripts, run them from a native command surface, attach context and memory where useful.
- AI/context features are important, but should be shown through concrete demos, not vague “AI productivity” language.
- Script Kit Brain / Day Page / markdown memory should be framed as workflow memory, not an Obsidian or “second brain” competitor.
- Launch should be devrel-led: demos, recipes, migration notes, cookbook examples, short videos, and tight feedback loops with early users.
- Avoid broad consumer positioning, no-code users, enterprise/MDM buyers, “free Raycast” seekers, and cross-platform promises before they are real.

## Contradictions

The biggest disagreement is the primary wedge.

Codex emphasizes the whole integrated category: native programmable launcher plus scripts plus Agent Chat plus markdown memory. Kimi narrows the wedge to “local-first scriptable control surface for developers who outgrew Raycast/Alfred but do not want Hammerspoon maintenance.” GLM pushes “agents that act on your desktop” as the strongest differentiator.

Best-supported position: lead with programmable developer launcher/control surface, then use agents-that-act as the sharp differentiator once demo reliability is proven. This avoids overbetting the whole brand on the riskiest feature while still preserving the unique upside.

A second disagreement is memory prominence.

Codex treats Brain/Day Page as a full messaging pillar. Kimi and GLM argue memory should be a secondary proof point until retention data shows it drives stickiness.

Best-supported position: include local markdown memory as a pillar, but not the hero. It is most defensible as “operational memory for scripts and agents,” not as the main reason to switch.

A third disagreement is competitive framing.

GLM favors “Hammerspoon, but modern” as a borrowed frame. Codex prefers “programmable command center” and warns against clone framing. Kimi agrees Hammerspoon is the strongest comparison but warns against comparing to too many tools.

Best-supported position: use Hammerspoon as an audience hook in developer channels, but keep official positioning broader and original: “native programmable launcher/workbench for scripts, agents, and local memory.”

## Partial Coverage

Codex had the best balanced strategic artifact: positioning, category, audience, messaging pillars, competitive notes, and 30/60/90 plan.

Kimi contributed the strongest skepticism: performance claims need benchmarks, “planned” cross-platform language creates expectation debt, Agent Chat/context requires a privacy and trust story, and too much novel vocabulary can confuse users.

GLM contributed the most practical execution filter: one wedge, one hero demo, one activation metric, and defer visual brand work until real users exist.

Only Kimi and GLM emphasized validation criteria strongly enough: track active script writers, test onboarding, benchmark speed claims, test landing-page framings, and make demo scripts reliable.

## Unique Insights

Valuable single-output observations:

- GLM: weekly active script-writers is a better north-star metric than installs.
- GLM: every public demo script should be treated as product surface, not content.
- Kimi: “too effective” broad messaging could attract the wrong users before the product can retain them.
- Kimi: privacy/security needs to be addressed before marketing desktop context and agent execution.
- Codex: the category trap is real: “launcher” invites Raycast comparison, “AI chat” invites ChatGPT/Cursor comparison, and “notes” invites Obsidian comparison.
- GLM: “GPUI” should probably remain a developer-context suffix, while “Script Kit” carries user-facing brand equity.

## Blind Spots

The panel mostly did not address pricing or business model beyond noting uncertainty. That matters because open source, paid app, sponsorware, or pro features would change channel strategy and conversion goals.

The panel did not deeply discuss onboarding packaging: installer, first-run permissions, Bun setup, sample scripts, and failure recovery. For a dev-facing desktop tool, first-run friction can dominate marketing outcomes.

The panel underdeveloped old Script Kit migration. Everyone agrees it is not drop-in compatible, but the final strategy needs a specific migration story: who should try GPUI now, who should wait, and what kinds of old scripts port cleanly.

The panel did not provide many concrete demo concepts beyond the broad “build a script / agent acts / memory” ideas. The synthesizer should turn the plan into named, shippable demo assets.

The panel did not address trust copy in enough detail: what leaves the machine, what agents can read, what they can execute, what requires confirmation, and how users inspect logs or receipts.

## Failure Notes

`claude-opus-4.8-high` failed to return the requested artifact. It produced only a partial self-correction and a shell search transcript, so it should receive minimal weight.

`agy-gemini-flash-high` failed the task entirely, returning only a model-identification sentence. It provides no usable strategic input.

The usable panel is therefore three strong outputs: Codex architect, Kimi edge-case tester, and GLM pragmatist. Confidence remains high because those three independently covered strategy, skepticism, and execution, and their disagreements are resolvable.

## Recommended Synthesis

The final synthesizer should produce a strategy with this spine:

1. Position Script Kit GPUI as a native programmable desktop workbench for developers who write JS/TS.
2. Use “launcher” as the familiar entry point, but avoid becoming “Raycast alternative” in the headline.
3. Lead with code-first programmability. Use AI/context as the unique differentiator. Use markdown memory as a retention/story layer, not the initial hook.
4. Be explicit: macOS-first, developer-facing, under active development, not a drop-in replacement.
5. Target JS/TS automation-heavy Mac developers, Hammerspoon/Raycast/Alfred power users who want code, local-first AI tinkerers, and devrel/educator builders.
6. Do not target non-coders, enterprise fleets, generic productivity users, no-code automation buyers, or users wanting a polished cross-platform launcher.
7. Build the launch around proof: one hero video, 10-30 tested recipes, a migration note, privacy/context explainer, and a README/website messaging pass.
8. Use validation metrics: first script created, weekly active script writers, first Agent Chat context use, first Day Page/Brain capture, repeat launcher usage.
9. Avoid overclaiming speed unless benchmarked. Say “native GPU-rendered UI” now; say “faster than X” only with measurements.
10. Keep terminology disciplined: scripts, prompts, Agent Chat, desktop context, verifiable actions, markdown memory. Demote jargon like sediment/fragments unless explaining the implementation.

## Judge JSON

```json
{
  "scores": {
    "codex-gpt-5.5-high": {
      "correctness": 9,
      "task_fit": 9,
      "evidence": 8,
      "specificity": 8,
      "constraint_following": 9,
      "novelty": 7,
      "risk_awareness": 8,
      "cost_complexity": 8,
      "rationale": "Best balanced strategic answer with clear positioning, proof points, competitive framing, and a practical 30/60/90 plan; slightly less sharp on validation and operational risk than the skeptic/pragmatist outputs."
    },
    "claude-opus-4.8-high": {
      "correctness": 1,
      "task_fit": 1,
      "evidence": 2,
      "specificity": 1,
      "constraint_following": 1,
      "novelty": 1,
      "risk_awareness": 2,
      "cost_complexity": 1,
      "rationale": "Failed to return the requested artifact and only emitted a partial self-correction plus a tool transcript."
    },
    "agy-gemini-flash-high": {
      "correctness": 1,
      "task_fit": 1,
      "evidence": 1,
      "specificity": 1,
      "constraint_following": 1,
      "novelty": 1,
      "risk_awareness": 1,
      "cost_complexity": 1,
      "rationale": "Did not answer the task; returned only a model-identification sentence."
    },
    "kimi-code-high": {
      "correctness": 8,
      "task_fit": 8,
      "evidence": 7,
      "specificity": 8,
      "constraint_following": 9,
      "novelty": 8,
      "risk_awareness": 10,
      "cost_complexity": 8,
      "rationale": "Strong edge-case and risk analysis with useful validation experiments and expectation-management warnings; less complete as a full marketing strategy artifact."
    },
    "opencode-glm-5.2-high": {
      "correctness": 8,
      "task_fit": 9,
      "evidence": 7,
      "specificity": 9,
      "constraint_following": 9,
      "novelty": 8,
      "risk_awareness": 8,
      "cost_complexity": 10,
      "rationale": "Most actionable and founder-executable plan, with clear defer lists, metrics, and a concise messaging guide; the agent-action wedge needs validation before becoming the main brand claim."
    }
  },
  "consensus": [
    "Position Script Kit GPUI as a developer-first programmable desktop surface rather than a Raycast clone, AI chat app, or notes app.",
    "Target automation-heavy JS/TS developers, local-first AI tinkerers, and technical power users before broader productivity users.",
    "State clearly that the project is macOS-first, under active development, and not a drop-in replacement for old Script Kit.",
    "Use concrete demos, recipes, and docs as the primary launch motion.",
    "Frame markdown memory as local workflow memory for scripts and agents, not a second-brain competitor."
  ],
  "contradictions": [
    "Primary wedge differs: integrated command center vs scriptable control surface vs agents that act. Best-supported position is programmable developer launcher/control surface first, with agents-that-act as the differentiator once demo-stable.",
    "Memory prominence differs: full pillar vs B-plot. Best-supported position is to include it as a proof point and retention layer, not the hero.",
    "Competitive hook differs: some favor Hammerspoon-modern language, others avoid borrowed frames. Best-supported position is to use Hammerspoon tactically in developer channels while keeping official category language original."
  ],
  "unsupported_claims": [
    "Any claim that Script Kit GPUI is faster than Raycast or Alfred without benchmarks.",
    "Any implication that Linux or Windows support is available before shipped builds exist.",
    "Any claim that Agent Chat and semantic UI transactions are reliable enough for broad launch without a verified demo.",
    "Any positioning of Script Kit Brain as a full Obsidian or second-brain replacement.",
    "Any suggestion that old Script Kit users can migrate without friction."
  ],
  "unique_insights": [
    "Track weekly active script-writers rather than installs.",
    "Treat demo scripts as product surface and keep them tested.",
    "Broad messaging may attract the wrong users before the product can retain them.",
    "Publish a privacy and trust story before heavily marketing desktop context and agent execution.",
    "Keep GPUI as developer-context technology rather than the main user-facing brand."
  ],
  "failure_notes": [
    "claude-opus-4.8-high failed to return the requested artifact, limiting its usefulness despite being assigned the skeptic role.",
    "agy-gemini-flash-high failed the task entirely and contributes no strategy signal.",
    "Confidence remains high because the three successful outputs independently covered architecture, edge cases, and pragmatic execution."
  ],
  "confidence": "high",
  "escalation_needed": false,
  "synthesis_instructions": [
    "Use Codex for the main strategic structure, Kimi for risk controls, and GLM for execution discipline.",
    "Make programmability the headline, agents the differentiator, and markdown memory the supporting retention story.",
    "Include explicit anti-positioning for no-code users, old Script Kit drop-in expectations, cross-platform promises, and generic productivity buyers.",
    "Require proof-oriented launch assets: hero demo, tested recipes, migration note, privacy/context explainer, and README/website messaging guide.",
    "Avoid performance superiority claims until measured; keep speed language bounded to native GPU-rendered UI unless benchmarks exist."
  ]
}
```

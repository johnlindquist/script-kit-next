# Script Kit GPUI Vision

This document is a product and architecture vision for Script Kit GPUI. It should
guide launch decisions, product tradeoffs, and implementation direction, but
current source, tests, generated contracts, and repo docs remain the source of
truth.

When updating this document or making product-facing changes, start from
`AGENTS.md`, `README.md`, `GLOSSARY.md`, `.impeccable.md`, and the current
source/tests. Do not use `CLAUDE.md` as source context for this vision. If this
document conflicts with current code, generated contracts, or verified runtime
receipts, update the document or the implementation intentionally.

Owning context: this is a product architecture document. It owns launch vision,
product identity, decision rules, anti-goals, and verification expectations. It
does not replace `README.md`, `GLOSSARY.md`, `AGENTS.md`, generated contracts,
tests, or source code.

## One-Sentence Vision

Script Kit GPUI is a native, keyboard-first command center for developers and
automation-minded users who want their desktop workflows to be programmable,
inspectable, and owned by them.

## Launch Thesis

Script Kit GPUI uses launcher speed as the front door, but the launcher is not
the whole product. The product is the loop:

```text
search -> choose -> prompt -> act -> automate -> verify -> return
```

At launch, Script Kit GPUI should feel like one coherent native tool across
scripts, prompt UIs, built-ins, Actions, Agent Chat, profiles, Kit Store plugins,
and semantic automation receipts.

The durable identity is:

> Raycast-grade launcher feel; Script Kit-grade local ownership, prompt-driven
> workflows, Agent Chat profiles, and verifiable automation.

## What Script Kit GPUI Is Trying To Be

Script Kit GPUI is a fast native GPUI launcher shell over a programmable local
automation runtime. The shell should appear instantly, accept intent, execute or
stage the right workflow, and get out of the way.

It is a Bun-powered TypeScript runtime where users can write and version their
own scripts, bring their own dependencies, and keep control of the code that runs
on their machine.

It is a prompt-first SDK. Prompt APIs are not incidental helpers; they are the UI
composition model for personal automation. Stable prompt surfaces should share
chrome, focus behavior, footer discipline, semantic IDs, and layout receipts.

It is a shared native UI system for scripts, prompts, built-ins, and Agent Chat.
The product should feel coherent because the surfaces share main search,
Actions, list row language, prompt shells, footer affordances, theme tokens, and
automation semantics.

It is a local plugin workspace for scripts, scriptlets, skills, profiles, and
shareable kits. The filesystem model is part of the product: users should be
able to inspect, edit, version, and migrate the artifacts they run.

It is a Kit Store installer for local plugin repos, not a marketplace-first
identity. Kit Store should help users discover, install, update, and remove
plugin repos while preserving local ownership.

It is an Agent Chat profile runtime. Profiles should make AI behavior
attributable, selectable, warmable, logged, and testable through explicit prompt,
provider/model, tool, cwd, session, and ambient-resource posture.

It is a semantic automation surface with receipts. Script Kit should prefer
target identity, semantic IDs, layout info, deterministic `waitFor`/`batch`
transactions, transcripts, logs, and action receipts over sleeps, coordinates, or
screenshot-only proof.

## What It Is Not

Script Kit GPUI is not a drop-in replacement for old Script Kit. The spirit
carries forward: fast scripting, prompt APIs, keyboard-first automation, and
local control. The old giant global helper surface does not.

Script Kit GPUI is not trying to become Raycast with more extensions. Raycast is
the polish benchmark. Script Kit's wedge is local ownership, programmable
prompts, plugin artifacts, Agent Chat profiles, and verifiable automation.

Script Kit GPUI is not a web dashboard in native clothing. It should appear,
accept intent, perform the job, and disappear.

Script Kit GPUI is not an unrestricted AI agent shell. Agent Chat and profiles
must be attributable, scoped, inspectable, and honest about what the runtime can
actually enforce.

## Differentiation From Raycast

Raycast proves that a launcher can be fast, polished, keyboard-first, and
native-feeling. Script Kit GPUI should meet that quality bar.

The difference is the center of gravity. Raycast primarily asks:

> What command can I run from a polished launcher?

Script Kit GPUI asks:

> What personal workflow can I turn into a local script, prompt UI, plugin
> artifact, Agent Chat profile, or agent-operable surface that I can inspect and
> verify?

| Dimension | Raycast-like launcher direction | Script Kit GPUI direction |
| --- | --- | --- |
| Core unit | Extension command | User-owned script, prompt, plugin artifact, or profile |
| Customization | Extension settings | TypeScript, Bun packages, prompt APIs, plugin folders, config/theme files |
| Distribution | Marketplace-centered | Kit Store as install/update/remove plumbing for local plugin repos |
| AI | Assistant feature or extension | Profile-scoped Agent Chat runtime with tools, cwd, prompt, session policy, and receipts |
| Automation | App/extension behavior | Semantic state, `getElements`, `getLayoutInfo`, `waitFor`, `batch`, logs, and receipts |
| Migration | Install more commands | Preserve prompt spirit, drop helper sprawl, convert durable AI behavior into explicit profiles |

This distinction should stay respectful. Raycast is excellent at what it does.
Script Kit should not win by dismissing that category. It should win by owning a
different category: programmable local automation with native launcher speed.

## Product Pillars

### Native Speed And Confidence

Fast is not a nice-to-have. The launcher should feel immediate. Keyboard-first
behavior, minimal chrome, native macOS feel, predictable focus, and low-friction
dismissal are part of the product.

The design language is fast, focused, and minimal. Chrome should earn its place.
The UI should make the user feel in control, not like they are operating a web
app inside a panel.

### Prompts Are The App Primitive

Prompt APIs are Script Kit's UI composition model. Stable prompt surfaces such
as `arg`, `div`, `editor`, `fields`, `form`, `path`, `term`, `drop`, `hotkey`,
and `chat` should feel like parts of one native product.

Do not claim media prompts such as `mic()` or `webcam()` are launch-ready until
current source and runtime receipts prove they are implemented. The current
source treats them as explicit coming-soon stubs.

### The Launcher Is The Spine

Main Menu Search is the entry point for scripts, built-ins, profiles, Kit Store
entries, menu syntax, context staging, and Agent Chat intent. Avoid side-channel
pickers unless they are contextual Actions.

The launcher should not become a cluttered dashboard. It is the command spine:
type intent, choose the right object, run or stage the workflow, then return the
user to their work.

### Actions Are Discovery

Command+K / Actions should answer "what can I do here?" New persistent controls
should go into Actions before earning permanent chrome.

Actions should feel like a natural extension of the main list, not a separate UI
system. The invariant is shared language, keyboard-first behavior, native feel,
contextual relevance, and no footer bloat.

### Three-Affordance Footer Discipline

The mental model should stay small:

```text
Primary action. Actions. Agent.
```

Contextual primary labels are fine. Persistent footer bloat is launch debt. If a
surface needs more operations, they belong in Actions unless there is a tested,
documented exception.

### Plugins Are Local Artifact Containers

A plugin can carry scripts, scriptlets, skills, profiles, and compatibility agent
artifacts. The local filesystem model is a product contract, not an
implementation detail.

Users should be able to inspect the plugin folder, understand what will run,
version it, edit it, and migrate it. Kit Store should support that ownership
model rather than obscure it.

### Profiles Are Runtime Boundaries

An Agent Chat profile owns meaningful runtime identity: prompt, provider/model,
tools, cwd, session behavior, ambient-resource posture, warm-key material, logs,
and receipts.

Do not describe profiles as magic hidden agents. Keep launch profile selection
explicit, attributable, and testable. If future work adds profile handoffs or
subagent-like flows, the UI, logs, and receipts must show who acted and why.

### Skills Are Recipes, Not Ambient Magic

Skills should be explicit task recipes that can be invoked inside Agent Chat or
profile flows. They should not silently change runtime behavior without UI/log
attribution.

Schema and runtime design should keep skills understandable: a skill says how to
do a task; a profile says what runtime boundary the task runs inside.

### Kit Store Distributes Local Ownership

Kit Store should help users discover, install, update, and remove plugin repos.
It should not become the product's main identity or pull Script Kit into
marketplace clone territory.

The goal is distribution without surrendering ownership: after installation, a
kit should be local, inspectable, and governed by the same plugin artifact model
as user-authored work.

### Automation Must Be Semantic And Receipted

Prefer `getState`, `getElements`, `getLayoutInfo`, `waitFor`, `batch`, semantic
IDs, action receipts, transcripts, and logs over sleeps, coordinates,
screenshot-only proof, or unrestricted keyboard/mouse injection.

Automation should be legible to humans and agents through the same state model.
If a surface cannot report what it is, what is focused, what is selected, and
what changed, it is not agent-ready.

## Source-Of-Truth Map

- Product/process: `README.md`, `AGENTS.md`, `GLOSSARY.md`, `.impeccable.md`
- Surface model: `src/main_sections/app_view_state.rs`
- Main launcher: `src/render_script_list/mod.rs`
- View routing/root shell: `src/main_sections/render_impl.rs`
- App state and launcher context: `src/main_sections/app_state.rs`
- Prompt protocol: `src/main_sections/prompt_messages.rs`
- Semantic elements/layout receipts: `src/app_layout/collect_elements.rs`,
  `src/app_layout/build_layout_info.rs`,
  `src/app_layout/build_component_bounds.rs`
- Shared chrome/components: `src/components/main_view_chrome.rs`,
  `src/components/prompt_layout_shell.rs`,
  `src/components/minimal_prompt_shell.rs`,
  `src/components/prompt_container.rs`, `src/components/prompt_footer.rs`,
  `src/components/hint_strip.rs`
- Actions: `src/actions/**`, `src/render_builtins/actions.rs`
- Built-ins: `src/render_builtins/**`
- Agent Chat/profiles: `src/ai/agent_chat/**`, `src/ai/acp/**`,
  `src/app_impl/tab_ai_mode/**`
- Plugin artifacts: `src/plugins/**`
- Kit Store: `src/kit_store/**`, `src/render_builtins/kit_store.rs`
- Menu syntax: `src/menu_syntax/**`
- DevTools/automation receipts: `scripts/devtools/**`,
  `src/agentic_protocol_bus.rs`

Keep these references source-aware but not brittle. For example, avoid freezing
exact struct ownership when a module may move. Prefer statements like "plugin
profile artifacts are parsed and validated by the plugin profile module" over
"this type must live in this exact file forever."

## Decision Rules

Build as a script when the user should own the TypeScript, dependencies, and
workflow.

Build as a prompt when the workflow needs structured input, selection, editing,
terminal output, forms, paths, drops, or chat-like interaction.

Build as a built-in when the feature requires native integration, privileged app
state, app-wide indexing, tight performance control, or a first-party reference
implementation.

Build as a plugin when the feature should be shareable and carry scripts,
scriptlets, skills, profiles, or compatibility agent artifacts together.

Build as a profile when the main distinction is AI runtime identity: prompt,
tools, cwd, provider/model, session policy, ambient-resource posture, or
warm/runtime behavior.

Build as a skill when it is a repeatable AI task recipe that should run inside an
existing profile instead of defining a new runtime.

Expose through Main Menu Search when the object is first-class and
runnable/selectable: scripts, built-ins, profiles, Kit Store entries, command
grammar heads, or capture targets.

Expose through Actions when the user is asking what can be done with the current
context.

Do not add persistent chrome for every feature.

Use mini mode when the row name is enough to choose confidently. Use expanded
mode when preview is required.

Route AI product language through Agent Chat. Do not create a second user-facing
AI chat concept because compatibility implementation names exist.

New launcher-family surfaces should join the shared surface contract, shared
chrome, footer ownership, semantic elements, and layout receipt model.

## Migration Model

Old Script Kit users should not expect drop-in compatibility. What carries
forward is the spirit: fast scripting, prompt APIs, keyboard-first automation,
and local control.

What changes is the SDK shape. Script Kit GPUI should keep the core prompt model
sharp and let users bring dependencies through Bun instead of restoring a giant
bundled helper global.

Migration ladder:

- Existing script idea -> TypeScript script under the plugin workspace.
- Reusable UI flow -> prompt APIs.
- Reusable capture or text grammar -> menu syntax or scriptlet artifact.
- Reusable AI task -> skill.
- Distinct AI runtime or policy -> profile artifact.
- Shareable package -> plugin repo installable through Kit Store.
- Legacy agent -> compatibility/import input that becomes an explicit profile
  when durable Agent Chat behavior is needed.

## Launch Standards

Fast is not optional.

Footer bloat is launch debt.

One-off UI is launch debt.

Mismatched prompt, built-in, and Agent Chat chrome is launch debt.

Any feature that cannot be discovered through Main Menu Search or Actions needs
a reason.

Any launch-facing surface that cannot produce semantic elements and layout
receipts is not agent-ready.

Any AI/profile claim that cannot be backed by runtime receipts should be
softened until it is true.

Screenshots can support visual debugging, but they are not proof by themselves.
Target identity, semantic state, layout state, and interaction receipts come
first.

## Anti-Goals

Do not become a Raycast clone with scripts bolted on.

Do not become a marketplace-first product whose local scripts, profiles, and
skills are secondary implementation details.

Do not recreate the old giant helper SDK just to reduce migration pain.

Do not become a web dashboard in native clothing.

Do not let Agent Chat become multiple user-facing chat products.

Do not introduce hidden background subagents without explicit profile handoff UI,
logs, attribution, and receipts.

Do not claim filesystem sandboxing until native or wrapper-level enforcement
exists and receipts prove it. Current profile `pathPolicy` language should stay
honest about advisory/validation/prompt-policy boundaries unless enforcement
changes.

Do not make coordinate-based automation or unrestricted keyboard/mouse injection
the primary automation path before semantic contracts are complete.

Do not let built-ins drift into bespoke UI islands.

## Verification Expectations

Use the smallest check that can fail. Route cargo through
`./scripts/agentic/agent-cargo.sh`, not bare `cargo`.

Representative source checks:

```bash
./scripts/agentic/agent-cargo.sh check --lib
./scripts/agentic/agent-cargo.sh test --lib render_script_list_footer_tests
./scripts/agentic/agent-cargo.sh test --lib actions_dialog
./scripts/agentic/agent-cargo.sh test --test simulate_key_cmd_enter_scriptlist_contract
./scripts/agentic/agent-cargo.sh test --test pi_profile_artifact_contract
./scripts/agentic/agent-cargo.sh test --test pi_profile_launch_contract
./scripts/agentic/agent-cargo.sh test --test agent_chat_profile_selector_contract
./scripts/agentic/agent-cargo.sh test --test devtools_coverage_contract
```

Representative runtime receipts:

- Target identity through automation window inspection.
- Surface contract evidence from `getState`.
- Semantic IDs, focus, selected item, footer buttons, warnings, and truncation
  from `getElements`.
- Header/input/main/footer/preview geometry from `getLayoutInfo`.
- Deterministic `waitFor` + `batch` transactions instead of sleeps.
- Agent Chat state receipts, composer identity, profile selection receipts,
  allowed/blocked transcripts, and export receipts where relevant.
- Visual proof only after target, semantic, and layout identity agree.

## The Product Test

When evaluating a launch-facing change, ask:

- Does this strengthen the loop of search, choose, prompt, act, automate,
  verify, and return?
- Does it preserve launcher speed and keyboard confidence?
- Does it reuse the shared surface language instead of creating a new island?
- Is discovery in Main Menu Search or Actions?
- Can a human inspect the local artifact or understand the runtime boundary?
- Can an agent inspect the semantic state and produce a receipt?
- Are the claims honest about what the current runtime can enforce?

If the answer is no, the change may still be worth making, but it needs an
explicit product reason and a verification plan.

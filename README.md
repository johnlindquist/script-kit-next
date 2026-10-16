# Script Kit GPUI

A complete rewrite of [Script Kit](https://scriptkit.com) using the [GPUI](https://gpui.rs) framework from Zed. This version combines the SDK and app into a single repository for a streamlined development experience.

## Project Goals

### Complete Rewrite with GPUI

Script Kit GPUI is built from the ground up using Zed's GPUI framework, delivering:

- **Blazing Fast Performance** - Native Rust performance with GPU-accelerated rendering
- **Sub-Second Compilation** - Hot reload development with cargo-watch rebuilds in 2-5 seconds
- **Single Repository** - SDK and app live together, making contributions and customizations straightforward
- **Bun Runtime** - Scripts execute via Bun for fast startup and modern JavaScript/TypeScript support

### Simplified SDK Philosophy

This rewrite takes a **focused approach** to the SDK:

- **Prompts Are the Core** - The SDK focuses on the prompt APIs (`arg`, `div`, `editor`, `term`, `fields`, `form`, `drop`, `hotkey`, etc.)
- **Bring Your Own Libraries** - Utilities and helpers are no longer bundled; install what you need via `bun add`
- **Full Control** - You manage your own dependencies, versions, and tooling
- **Lighter Weight** - The SDK stays small and focused on UI primitives

### Not Backwards Compatible

> **Important**: This is NOT a drop-in replacement for previous Script Kit versions.

What's preserved:
- Core prompt APIs (`arg`, `div`, `editor`, `fields`, `form`, `drop`, `hotkey`, `mini`, `micro`, `path`, `term`, `chat`)
- Choice/option structure and props
- Basic script metadata format

What's changed:
- No bundled utilities (file helpers, clipboard wrappers, etc.)
- No `kit` global with hundreds of helpers
- Scripts must explicitly import dependencies via Bun
- Configuration is TypeScript-based (`~/.scriptkit/config.ts`)

## How It Works

Script Kit is one window you tell what should happen.

- **Say it.** Type or speak a sentence ("when does LeBron play next", "remind Sam about the deck
  friday"). Jev reads it against what's on screen and the list offers what it could mean; ⌘1–3 run
  Jev's readings, ↵ runs the highlighted row, and its label always names exactly what will run.
- **Or say it exactly.** Symbols complete in the list and underline themselves: `@selection`,
  `@clipboard`, `@file:…` (what to act on), `/command` (what to do), `>:dev`, `todo;` (where it goes),
  `due:friday`, `.concise` (details), `|profile` (who does it).
- **The input never moves.** The line above it shows what's staged, waiting, or just ran. Forms,
  answers, notes, the model's chat, agents, voice, and script prompts all open in place under it; Esc
  goes back one step.
- **Act on anything.** ⌘K stages the highlighted row (or the clipboard, or the last run) in the line
  and the list becomes what you can do with it. ⌘↵ hands it to the agent.
- **No dialogs.** Irreversible actions wait 8 seconds in the line and run unless you stop them;
  reversible ones happen at once with ⌘Z.
- **It learns.** What you ran becomes an instant match next time, and runs that did several things
  are saved as scripts you can bind to a shortcut.

## Quick Start

New to Script Kit GPUI? Start with the user guides:

- [Getting Started](docs/guides/getting-started.md)
- [Feature Tour](docs/guides/feature-tour.md)
- [Main Menu Input](docs/guides/main-menu-input.md)
- [SDK Scripting](docs/guides/sdk-scripting.md)
- [MCP and Agent Context](docs/guides/mcp-and-agent-context.md)
- [Dictation](docs/guides/dictation.md)
- [Permissions](docs/guides/permissions.md)
- [Computer Use](docs/guides/computer-use.md)

### Prerequisites

- **macOS** (Linux/Windows support planned)
- **Rust 1.98.0** - Pinned by `rust-toolchain.toml`; install with https://rustup.rs
- **Bun** - Install from https://bun.sh
- **Python 3, CMake, and Ninja** for development storage admission and native builds (`brew install cmake ninja`).
- **cargo-watch** (optional, for hot reload):
  ```bash
  cargo install cargo-watch
  ```

### Setup

This is interactive **operator setup**. Agents should use [Owned native agent workflow](#owned-native-agent-workflow), not launch the raw binary below.

1. **Clone the repository**
   ```bash
   git clone https://github.com/johnlindquist/script-kit-gpui.git
   cd script-kit-gpui
   ```

2. **Create the kit directory**
   ```bash
   mkdir -p ~/.scriptkit/plugins/main
   ```

3. **Build and run**
   ```bash
   cargo build --release
   ./target/release/script-kit-gpui
   ```

   Or for development with hot reload:
   ```bash
   ./dev.sh
   ```

4. **Configure your hotkey** (optional)
   
   Create `~/.scriptkit/config.ts`:
   ```typescript
   export default {
     hotkey: {
       modifiers: ["meta"],  // Cmd on macOS
       key: "Semicolon"      // Press Cmd+; to toggle
     }
   };
   ```

### Your First Script

Create `~/.scriptkit/plugins/main/scripts/hello.ts`:

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Hello World",
  description: "My first script"
};

const name = await arg("What's your name?");
await div(`<h1 class="text-4xl p-8">Hello, ${name}!</h1>`);
```

Press your hotkey, type "hello", and press Enter.

## Writing Scripts

### Prompts (The Core API)

```typescript
// Text input with choices
const fruit = await arg("Pick a fruit", ["Apple", "Banana", "Cherry"]);

// Rich choices with metadata
const app = await arg("Launch app", [
  { name: "VS Code", value: "code", description: "Editor" },
  { name: "Terminal", value: "term", description: "Shell" },
]);

// HTML display with Tailwind CSS
await div(`
  <div class="p-8 bg-gradient-to-r from-blue-500 to-purple-600">
    <h1 class="text-white text-3xl font-bold">Beautiful UI</h1>
  </div>
`);

// Multi-line editor
const code = await editor("// Write your code here", "typescript");

// Form with multiple fields
const [name, email] = await fields([
  { name: "name", label: "Name", placeholder: "John Doe" },
  { name: "email", label: "Email", type: "email" },
]);

// File/folder picker
const file = await path({ startPath: "~/Documents" });

// Capture a hotkey
const shortcut = await hotkey("Press a shortcut");

// Terminal emulator
await term("htop");

// Drop zone for files
const files = await drop();
```

### Using Bun Packages

Since utilities aren't bundled, install what you need:

```bash
cd ~/.scriptkit
bun add zod lodash-es date-fns
```

Then use them in your scripts:

```typescript
import { z } from "zod";
import { groupBy } from "lodash-es";

export const metadata = {
  name: "Process Data",
  description: "Using external packages"
}

const data = await arg("Enter JSON data");
const parsed = z.object({ items: z.array(z.string()) }).parse(JSON.parse(data));

await div(`<pre>${JSON.stringify(groupBy(parsed.items, x => x[0]), null, 2)}</pre>`);
```

### Script Metadata

Use `export const metadata = { ... }` to define script properties:

```typescript
export const metadata = {
  name: "My Script",
  description: "What it does",
  author: "Your Name",
  shortcut: "cmd+shift+m",
  schedule: "0 9 * * *",
  // Additional options:
  // hidden: true,        // Hide from script list
  // tags: ["utility"],   // Categorize scripts
};

// Your code here...
```

> **Note:** The `export const metadata` object is typed as `ScriptMetadata`, providing TypeScript type checking and IDE support. Comment-based metadata (`// Name:`, `// Description:`) is a compatibility-only pattern.

## Configuration

### `~/.scriptkit/config.ts`

```typescript
export default {
  // Global hotkey to show/hide Script Kit
  hotkey: {
    modifiers: ["meta"],      // "meta", "ctrl", "alt", "shift"
    key: "Semicolon"          // Key codes: "KeyK", "Digit0", "Semicolon", etc.
  },
  
  // UI customization
  padding: { top: 8, left: 12, right: 12 },
  editorFontSize: 16,
  terminalFontSize: 14,
  uiScale: 1.0,
  
  // Built-in features
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
    windowSwitcher: true
  },
  
  // Custom paths
  bun_path: "/opt/homebrew/bin/bun",
  editor: "code",

  // Runtime preferences also live here
  theme: { presetId: "nord" },
  dictation: { selectedDeviceId: "usb-mic" },
  ai: {
    selectedProfileId: "script-kit",
    selectedBackend: "goose",
    selectedModelId: "chatgpt_codex/gpt-6-luna"
  },
  windowManagement: { snapMode: "expanded" }
};
```

### Agent Chat Configuration

Agent Chat runs on the bundled [goose](https://github.com/aaif-goose/goose)
engine over the Agent Client Protocol and uses profiles in
`~/.scriptkit/config.ts`. The ChatGPT (Codex) subscription is the default
provider. Any goose provider works in a profile, including `anthropic`,
`google`, `openrouter`, and local models through `ollama` or `lmstudio`.

```typescript
ai: {
  selectedBackend: "goose",
  selectedProfileId: "script-kit",
  selectedModelId: "chatgpt_codex/gpt-6-luna",
  profiles: [
    {
      id: "script-kit",
      name: "Script Kit",
      backend: "goose",
      provider: "chatgpt_codex",
      model: "gpt-6-luna",
      cwd: "~/.scriptkit",
      tools: ["read", "write", "edit", "bash", "grep", "find", "ls", "web_search"]
    },
    {
      id: "local",
      name: "Local",
      provider: "ollama",
      model: "qwen3:4b",
      tools: ["web_search", "read"]
    }
  ]
}
```

Tools come from two places. `web_search`, `web_fetch`, `read`, `grep`, `find`,
`ls`, and `create_file` are Script Kit tools, confined to the profile's
`pathPolicy`. `write`, `edit`, and `bash` are goose's developer tools. A profile
gets only the tools it lists.

The launcher and Agent Chat say "Not signed in" until you sign in to a
subscription provider (ChatGPT, Gemini, or xAI). The first time you use an AI
feature, a "Sign in to ChatGPT" row asks first, under the same input: ↵ opens
sign-in in your browser, Esc means not now. While you sign in, Script Kit steps
aside; when you finish, it comes back and your message continues. If sign-in
doesn't finish, ↵ Try Again returns you to the browser. Switch Account under ⌘K
forgets the stored sign-in. Sign-ins are stored in `~/.scriptkit/goose`,
separate from the Codex CLI's own login. API keys saved with the Configure
OpenAI/Anthropic/Vercel API Key commands are passed to goose automatically.

Release builds use the bundled goose engine. Developers and support builds can
override it with `SCRIPT_KIT_GOOSE_BINARY`, `ai.gooseBinary`, or a per-profile
`gooseBinary`.

In the chat, ⌘K (or saying "use opus for this") changes profile or model.

### Advanced Provider Configuration

Script Kit passes these keys to the goose engine under the names its providers
read. Other goose providers (OpenRouter, xAI, Bedrock, and more) are set up with
`goose configure`. Agent Chat setup should go through profiles in `config.ts`.

```bash
export SCRIPT_KIT_OPENAI_API_KEY="sk-..."
export SCRIPT_KIT_ANTHROPIC_API_KEY="sk-ant-..."
export SCRIPT_KIT_GOOGLE_API_KEY="..."
export SCRIPT_KIT_GROQ_API_KEY="..."
export SCRIPT_KIT_VERCEL_API_KEY="..."
```

After adding, restart your terminal or run `source ~/.zshrc`.

### `~/.scriptkit/theme.json`

Customize the look and feel:

```json
{
  "colors": {
    "background": { "main": "#1E1E1E" },
    "text": { "primary": "#FFFFFF" },
    "accent": { "selected": "#FBBF24" }
  },
  "opacity": { "main": 0.3, "selected": 0.12 },
  "vibrancy": { "enabled": true, "material": "popover" }
}
```

See `kit-init/theme.example.json` for all available options.

### Main-window animation presets

The main window fades in and out at its final size; it never grows or shrinks to
appear. Two entry fades are available. Search **Animation Presets** in the
launcher to pick one; it saves and replays the actual launcher once, with no
restart or rebuild:

- **Animation 01 — Soft Focus Fade** (`fixed_frame`): a slower 140 ms fade that
  resolves a light defocus.
- **Animation 02 — Sharp Presence** (`sharp_presence`, the default): a 90 ms,
  near-opaque fade with no defocus.

**Next Animation Preset**, **Previous Animation Preset**, and
**Replay Current Animation** are in the same list. The selection persists
independently of color themes in `config.ts`:
`animation: { preset: "sharp_presence" }`; retired preset ids load as the default.
Exit is always a 0.12 s fade.

Agents use the same operations through `triggerBuiltin` with
`builtin/animation-fixed-frame` and `builtin/animation-sharp-presence`, plus `builtin/animation-presets`,
`builtin/animation-next`, `builtin/animation-previous`, and `builtin/animation-replay`.
Read `getState.mainWindowAnimation` for selection/catalog and captured
`nativeLifecycle` separately; a dispatched replay is not a completed animation.
Native model samples are not WindowServer pixels. Hidden fixtures apply choices
in memory without native replay or operator preference writes.

## Development

### Standalone Spotlight glass demo

Build the separate Rust/GPUI prototype through the protected artifact facade, then run its verified immutable executable as a resident process:

```bash
bun scripts/devtools/devtools.ts build-ops act demo-build --artifact-out <new-reference.json>
bun scripts/devtools/devtools.ts build-ops query artifact --reference <new-reference.json> --task spotlight-demo
```

Launch the verified manifest's `binaryPath` through owned process supervision with explicit visible-app authorization. Do not use `cargo run` for this resident preview: its bounded build deadline terminates the demo and its build lease stays held. The immutable executable runs independently, without retaining the Cargo build pool or build timeout.

**Option+2** globally shows the demo and replays its original expanded-to-detached glass morph. **Enter/Space** toggles the split while focused, **Esc** hides without quitting, and **Cmd+Q** quits. The shortcut exists only while this demo process runs; no login item or main-app shortcut is installed. Its measured spring/timing/geometry is unchanged.

For agent control with explicit native-input authorization, use `SCRIPT_KIT_NONINTERACTIVE=0 bun scripts/agentic/macos-input.ts key digit2 --modifiers alt --force-native --target liquid-glass-demo`. `digit2` names the physical number-row key; literal `2` remains layout-aware text input. Injector acceptance alone is not replay proof: the real GPUI action emits JSON stdout with `event: "liquid_glass_demo"`, `action: "replay"`, and a monotonic `replay_id`; hide and successful hotkey registration emit their own receipts. Observe the exact demo PID/window for visible/hidden postconditions.

### Owned native agent workflow

Use the repository dispatcher, `bun scripts/devtools/devtools.ts`, as the agent entry point. The owned app requires `--features owned-ui-evaluation`; this does not change the normal `./dev.sh` default of `local-llm`. Provider-free fixtures need **no auth seeding, Pi sidecar provisioning, model downloads, or live provider setup**.

The owned launcher keeps `SCRIPT_KIT_NONINTERACTIVE=1` and unsafe opt-ins at `0`: no screen takeover/capture, global input, native focus, clipboard access, live devices, providers, or models. It mounts **real production GPUI roots** in owned hidden windows and reads their completed GPU framebuffer, with production semantics/layout and scoped in-app actions. It is not a fake Storybook or web-rendered authority. Its evidence excludes WindowServer composition, AppKit material/glyph pixels, native focus, OS IME, global input, and live providers/devices. A catalog or token export is inventory, not runtime proof.

#### Discover and build immutable artifacts

Each verification run gets a fresh output parent; leave reference files and evaluation output directories nonexistent until their owning command creates them. **A new evidence directory does not require new binaries.** Reuse independently verified current references and build only the artifact kinds the run needs.

```bash
export SCRIPT_KIT_NONINTERACTIVE=1
export SCRIPT_KIT_ALLOW_SCREEN_TAKEOVER=0 SCRIPT_KIT_ALLOW_VISIBLE_PROBES=0
export SCRIPT_KIT_ALLOW_NATIVE_INPUT=0 SCRIPT_KIT_ALLOW_SCREEN_CAPTURE=0
export SCRIPT_KIT_ALLOW_LIVE_AI=0 SCRIPT_KIT_ALLOW_ISOLATED_APP_LAUNCH=0
RUN=".test-output/owned-$(uuidgen)"
mkdir -p "$RUN"
printf 'Owned output parent: %s\n' "$RUN"

bun scripts/devtools/devtools.ts build-ops discover
bun scripts/devtools/devtools.ts build-ops inspect
bun scripts/devtools/devtools.ts build-ops query storage
bun scripts/devtools/devtools.ts build-ops diagnose locks

# Run only the builds needed for this run; keep feature/profile selections stable.
bun scripts/devtools/devtools.ts build-ops act app-build \
  --features owned-ui-evaluation --artifact-out "$RUN/app.reference.json"
bun scripts/devtools/devtools.ts build-ops act libtest-build \
  --artifact-out "$RUN/libtest.reference.json"
bun scripts/devtools/devtools.ts build-ops act exporter-build \
  --artifact-out "$RUN/exporter.reference.json"
```

Proceed only after successful build receipts with closed cleanup. These are actual application, Rust libtest, and exporter artifacts, respectively, published through `agent-cargo.sh` in the shared `agent-debug` pool. Each `--artifact-out` exclusively creates an **ArtifactReference** (`manifestPath`, `manifestSha256`) pointing to immutable **Manifest V3** publication: exact Cargo target/features/profile, compiler-input content, toolchain/configuration, binary hash, and build/lease identity. The exporter action owns its test profile. Never choose a binary by mtime, launch a raw target path, rewrite a manifest, or overwrite a reference to make it appear current.

The managed lane disables incremental compilation authoritatively (`CARGO_INCREMENTAL=0` and dev/test/build incremental settings `false`), including inherited enabling defaults. Conflicting forced Cargo configuration, incremental rustc flags, and target/build-directory relocation are refused. This does not change the human `dev.sh` watcher's profile or incremental settings. Inspect the effective compiler/wrapper policy; a selected sccache backend is not proof of cache hits.

The allocation policy covers all of `target-agent/`, including pools, exports, shared caches, pending publications, runtime, and quarantine. **Identity-pinned root links on a different volume have no default size cap**; `targetAgentBudgetBytes: null` reports this uncapped state. Ordinary/local storage retains the **40 GiB default**, and an explicit numeric `SCRIPT_KIT_AGENT_TARGET_BUDGET_GB` still sets a cap. The **25 GiB default free-space floor** and **two-worker ceiling** remain; raise `SCRIPT_KIT_AGENT_MIN_FREE_GB` for a larger reserve. Admission, sampled cancellation, and postflight checks still refuse incomplete observations or insufficient free space. This is **not a filesystem-enforced hard quota**. Allocated-block accounting deduplicates hardlinks within the managed scope; unique APFS extent usage is unknown. External caches, `target/`, and evidence directories are separately reported.

An existing `target-agent` root link can use explicitly trusted storage. With user authorization, run `bun scripts/agentic/managed-storage.ts trust "$PWD" "/absolute/canonical/destination"`, then `bun scripts/agentic/managed-storage.ts inspect "$PWD"`. Registration creates the local ignored `.agent-cargo-storage.json`; it does not move directories or replace existing authority. Every consumer checks the exact link, destination, device/inode, and owner. Nested links and stale routes are refused. Complete allocation accounting covers the destination tree even when its size cap is disabled, and the configured free-space floor applies to both repository and destination volumes. A recreated link or replaced destination requires explicit renewed registration after investigating the change; a running guard refuses if its uncapped route becomes local storage.

`SCRIPT_KIT_AGENT_INVENTORY_TIMEOUT_MS` optionally sets the complete-census deadline in integer milliseconds (default `8000`, range `1`–`300000`); incomplete measurements still refuse admission.

`SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS` sets the free-space query deadline (default `2000`, range `1`–`60000` milliseconds). A bounded per-run setting can accommodate filesystem scheduling delays under load. A stalled query still cancels the build, reports `inventory_free_space_time_limit`, and never substitutes an earlier sample for the missing measurement.

No automatic eviction, budget increase, low-disk bypass, or shared-cache clear follows a refusal. Historical 65/66 GiB allowances are not reusable permission. Inspect storage and exact locks, then stop the batch until the resource or ownership issue is resolved.

For reviewed noncompiler changes, inspect the route before running it; paths are explicit inputs, not an automatically discovered Git diff:

```bash
bun scripts/devtools/devtools.ts build-ops query route README.md
./scripts/agent-check.sh README.md
./scripts/agent-check.sh scripts/agentic/session.sh
```

Reviewed documentation/evidence paths select no compiler or test subprocesses; reviewed TS/Python/shell owners select their bounded Bun behavioral suites. Compiler inputs in `scripts/agentic/compiler-input-paths.txt` take precedence, even over documentation extensions. Unknown paths and an empty path list retain conservative Rust routing and report uncovered noncompiler contracts; `--quick` does not omit selected Bun checks. Inspect `coverageGaps`, `performedVerification`, and selected/attempted steps: a documentation-only success is a routing decision, not verified document content.

TS, receipt-policy, and runner-only changes do not require new Rust binaries when compiler inputs remain current. Explicit stale references fail rather than silently rebuilding. A proven warm Cargo output reuses its existing immutable publication (`artifactReused: true`) instead of creating another executable export. Infrastructure Python supervisors/helpers use explicit `-B`, including isolated `-I -S` launches, to prevent incidental bytecode writes without changing child applications' Python policy.

```bash
bun scripts/devtools/devtools.ts build-ops act lib-test \
  --reference "$RUN/libtest.reference.json" \
  --filter ai::reliability::tests::redactor_allowlists_json_masks_secrets_paths_and_bounds_output
```

Reference-based libtests invoke no Cargo and create no new immutable binary export. Keep one Cargo job active at a time; checkpoint storage and finalization after each build and verification batch.

#### Discover, inspect, and run real stories

```bash
bun scripts/devtools/devtools.ts stories discover
bun scripts/devtools/devtools.ts design discover \
  --artifact "$RUN/app.reference.json" --out "$RUN/discover"
bun scripts/devtools/devtools.ts design inspect --fixture main.script-list \
  --artifact "$RUN/app.reference.json" --out "$RUN/inspect"
bun scripts/devtools/devtools.ts design query --fixture main.script-list --facet layout \
  --artifact "$RUN/app.reference.json" --out "$RUN/layout"
bun scripts/devtools/devtools.ts design query --fixture main.script-list --facet frame --image \
  --artifact "$RUN/app.reference.json" --out "$RUN/frame"
bun scripts/devtools/devtools.ts design run --scenario theme-publication-revert \
  --artifact "$RUN/app.reference.json" --out "$RUN/theme-story"
bun scripts/devtools/devtools.ts stories run --libtest "$RUN/libtest.reference.json" \
  --app "$RUN/app.reference.json" --scope core --out "$RUN/core-stories"
```

`design query` supports `elements`, `layout`, `text`, and `frame`; `--image` retains bounded frame readback evidence. `stories run` combines exact library stories with the nine owned production core journeys; `--lane library` alone is not core-runtime proof. Inspect each final receipt and its assertions/cleanup, not just command completion. These commands describe how to obtain evidence, not a claim that the full fixture campaign or full test suite has completed.

#### Live preview and persistent owned loop

For supported scalar/color token previews without rebuilding Rust:

```bash
bun scripts/devtools/devtools.ts design watch --fixture main.script-list \
  --artifact "$RUN/app.reference.json" --out "$RUN/watch" --edits "$RUN/watch/edits.json"
```

Run watch in a supervised process (or a dedicated terminal). **Do not pre-create `watch/`**. Once watch has claimed that fresh output directory, use a file tool/second terminal to create `$RUN/watch/edits.json` with this JSON, then edit that same file to preview changes:

```json
{"edits":[{"tokenId":"theme.colors.accent.selected","value":6004223}]}
```

This color is `0x5b9dff`; JSON colors are integer RGB values, not hex strings. The edit file must be an owned regular file inside that exact output directory, not a symlink or hardlink. Missing/invalid documents produce `designWatchRefusal` and retain the last good revision; successful updates emit `designWatchPublication`. The Rust token validation/publication owner remains authoritative. Watch tracks expected theme revisions, observes causal invalidation/completed frames, and **SIGINT** (Ctrl-C) ends the watch, reverts accepted preview edits, and closes the owned runtime. It is bounded, not a permanent service. Scalar/color previews do not edit source or persist a new user theme; Rust layout/component changes require rebuilding a new artifact and starting a fresh run.

For supervised watch sessions, use an isolated PTY and send Ctrl-C to that terminal, or signal only the watcher controller PID. Do not broadcast SIGINT to its process tree: killing the native child before the controller can revert prevents native cleanup confirmation and correctly produces `INVALID_CLEANUP`. Retain that failed receipt; start a fresh owned session rather than rewriting it as successful.

For multiple protocol steps in the **same** runtime:

```bash
bun scripts/devtools/devtools.ts design loop \
  --artifact "$RUN/app.reference.json" --out "$RUN/loop"
```

It reads existing protocol `Message` JSONL from stdin and returns correlated responses. Start with:

```json
{"type":"design","command":{"operation":"catalog"}}
{"type":"design","command":{"operation":"mount","fixtureId":"main.script-list"}}
```

Use the mount response's `result.target`: construct the exact selector `{type:"instance", id: target.windowId, generation: target.windowGeneration}`. Send `getState` with that selector; use the returned `targetIdentity` (or `surfaceContract.targetIdentity`) as `expected` for scoped mutations/unmount and completed-frame waits. Do not copy an identity from another command's runtime, promote a kind/index selector, or guess a revision. To preview in this loop, send a `design` command with `operation:"applyTheme"`, `expectedRevision` equal to the observed `themeRevision`, and the `edits` array above. Revert with `operation:"revertTheme"` and the returned publication `revision`. Query/inspect again after each transition; stale target/generation/revision refusals are not permission to weaken the selector.

For a latest frame observation, send `{type:"design", command:{operation:"captureFrame", target:{type:"instance", id, generation}, includeImage:true}}`. This atomic read completes a real frame, collects state/semantics/layout, and strictly captures that same frame in one foreground command; its `result.frame`, `snapshot.frameIdentity`, and each observation's `targetIdentity` are correlated. It requires no prior frame or stale mutation `expected`. Use `includeImage:false` for the same qualified observation without retained PNG bytes. `OwnedEvaluationClient.captureFrame(target, includeImage)` and generic frame queries use this operation. Explicit `frame()` followed by `capture()` remains exact prior-frame readback for latency and same-frame comparisons: advancing state between those commands is still refused, never retried. Earlier frame records are separate observations, not identities for a later atomic capture.

Finish the loop with **EOF** or the explicit End message:

```json
{"type":"design","command":{"operation":"end"}}
```

One-shot commands automatically tear down; loop EOF/End closes owned windows and the process. End verifies zero remaining windows before lifecycle completion. Always inspect the final cleanup receipt: **`INVALID_CLEANUP` is never green**, even if all assertions passed. Preserve failed evidence and diagnose it rather than deleting it or treating a missing final receipt as success.

#### Recovery and operator-only legacy routes

```bash
bun scripts/devtools/devtools.ts build-ops query artifact \
  --reference "$RUN/app.reference.json" --task application
bun scripts/devtools/devtools.ts build-ops diagnose locks
bun scripts/devtools/devtools.ts design diagnose --receipt "$RUN/theme-story/receipt.json"
```

Receipt diagnosis is historical validation, not fresh proof. A stale compiler-input/configuration/toolchain artifact must be **rebuilt** into a new reference under a fresh `RUN`, then rerun with fresh output directories. Never reuse failed output roots or repair hashes by hand. Lock recovery, when needed, belongs to `build-ops act recover-lock --lock <exact-name.lock> --expect <observed-lock.json>` with the exact observed lease document; do not kill unrelated processes or delete caches/artifacts.

Owned Design/Stories receipts use `script-kit-owned-receipt` format version 1: a compact summary references the single sanitized `observation.json` through existing artifact hashes and an owner-marker digest. Retain the complete owned evidence tree at its recorded identity; copying only the final JSON is not sufficient. Readers verify ownership, paths, bytes, and semantic evidence before trusting the summary. Resolution is one level only, with a 1 MiB compact-wire limit and 64 MiB per artifact; retained captures are processed sequentially. Obsolete inline-owned receipts are rejected without rewriting historical files; use their original compatible reader or collect fresh evidence. Small exporter release proofs remain standalone V2 documents written once and may still be copied/uploaded alone.

#### Exact keep sets and task-scoped retirement

```bash
bun scripts/devtools/devtools.ts build-ops query keep-set
bun scripts/devtools/devtools.ts build-ops query retention
```

The discovery response includes the JSON input schemas. Retain current plus previous usable references for each needed artifact kind (at most six for the application, libtest, and exporter), their publisher/derivation records, and explicitly retained audit evidence. Actual active or unknown references remain protected regardless of age; if they exhaust the budget, growth is blocked rather than evicting them.

An exact keep-set update uses `act keep-set --expect-revision <current-keep-revision> --input <selection.json>`; the JSON object has a `references` array of exact `ArtifactReference` objects. A prune uses `act prune --expect-revision <current-plan-revision> --input <selection.json>`; its JSON object has a `candidates` array of exact selected plan candidates. Select only this task's authorized obsolete outputs; selecting a parent never implicitly authorizes unselected descendants. Archive small manifests and ownership records before deleting their bulk outputs.

Auxiliary staging directories qualify only through their exact explicitly selected owning task and complete parent chain. Directory/marker identities and marker hashes are rechecked before and after quarantine; unrelated tasks, changed markers, symlinks, and active/unknown resources remain protected. Selection is never permission to sweep an unmanaged ancestor.

New retirement journals use schema version 2. Exact serialized UTF-8-plus-newline size is bounded at 8 MiB, including prospective recovery phases and terminal history, before mutation. An oversized fresh selection must be reduced; do not raise the reader limit or edit journal bytes to force it through.

Version-2 interrupted recovery uses the current recovery revision, finishes only still-safe started quarantines, and withdraws untouched candidates without deleting them. If it returns `replanRequired`, query and explicitly select a fresh plan. Legacy version-1 journals instead fail with `retention_legacy_journal_requires_compatible_reader_or_reviewed_recovery`; their missing marker identity cannot be inferred. They require the original compatible reader or explicitly reviewed recovery, not an ordinary fresh-plan retry. Journals, quarantine, and historical receipts stay untouched on refusal. Never waive a new reference, clear an unknown lease, or invoke the legacy human-target sweeper from an agent.

The global `script-kit-devtools` skill's legacy mtime selection, `Driver.launch({binary: ...})`, automatic sidecar/auth seeding, and `--show` guidance is **not this workflow**. Follow these repository commands instead. Visible `session.sh`/legacy inspection routes, raw binaries, desktop input/screen capture, and real-provider tests are **explicit operator-only** work requiring separate authorization; never fall back to them when an owned command refuses. The following hot-reload, raw SDK smoke, and release commands preserve the interactive operator development workflow.


### Hot Reload

```bash
./dev.sh  # Starts cargo-watch, rebuilds on file changes
```

Changes to Rust code trigger a rebuild (~2-5 seconds). Theme and script changes reload instantly without restart.

Operator development retains `./dev.sh`'s default `local-llm` feature. Owned evaluations above are a separate opt-in build and do not change it.

In every view the glass stage fills the window and the native capsule (↵, ⌘K,
⌘↵) floats inside it at the bottom right (Tahoe floating chrome), in one fixed
bottom zone with an 8pt gap above, below, and at the sides. Views keep their
content above that zone. The main list alone extends under it: rows scrolled
behind the capsule fade into the glass and ignore hover and clicks, selection
reveal stops above the zone, and the scrollbar track ends there too.
`getState.activeFooter.overContentPreview` reports whether the main-menu list
currently scrolls under the footer.

The operator watcher uses a real SSD workspace under `/Volumes/OMEGA SSD/sk/<repo-id>/`, not the repository's internal `target/`. App and local-LLM helper Cargo targets, Pi provisioning targets/source cache, Metal/Clang caches, optional sccache storage, and build temporary files stay in that workspace. The app launches directly from its SSD target; `local-llm` and the `dev-fast` startup profile remain enabled. Native CMake builds default to Ninja; an explicit `CMAKE_GENERATOR` is preserved.

Admission requires a mounted, writable external APFS/HFS+ volume with at least 25 GiB available. Startup and each rebuild check the volume UUID, workspace ownership, symlink redirection, and conflicting inherited target/cache paths. Missing or unsuitable storage fails before building; there is no internal-disk fallback. These are admission checks, not a filesystem quota.

```bash
# Read-only JSON state, paths, mount identity, and available space
python3 -B scripts/agentic/dev-storage.py --repo "$PWD" inspect

# Select another mounted external APFS/HFS+ volume
SCRIPT_KIT_DEV_VOLUME="/Volumes/My Build SSD" ./dev.sh

./dev.sh --status
./dev.sh --stop
```

Watcher registration is serialized and published atomically. A complete lease
whose process is dead recovers on startup; an incomplete lease requires
`./dev.sh --takeover`. Recovery preserves the stale record instead of deleting
it. Live or reused PIDs, foreign ownership, and untrusted records remain
protected; takeover never kills a process identified only by name or CWD.

If a session supervisor dies while its app remains alive, exact stop can reap
the recorded dedicated app group after revalidating its generation, process
start times, and membership before each signal. Reused PIDs or changed members
remain protected. Recovery records `session_orphan_reaped` rather than
fabricating a supervisor-owned `app-exit.json` receipt.

A dev relaunch is successful only after the new app registers the main global
shortcut. Its JSON receipt includes `mainHotkeyRegistered`, `mainHotkeyDisplay`,
and `mainHotkeyId`; a registration conflict fails readiness rather than leaving
an apparently successful but shortcut-inaccessible app. The unchanged-build
skip checks this same owner and the expected dev binary. Read-only inspection:
`bash scripts/agentic/dev-relaunch.sh --check-hotkey dev-watch`.

UI delivery for operator testing means updating the actual `dev-watch` app and
proving native shortcut ingress there, not just opening a separate preview.
`session.sh status dev-watch` identifies its PID, binary, and generation;
the app's `GLOBAL_HOTKEY_EVENT ... action=Main launcher` proves OS ingress.

`prepare --shell` is the script-facing operation that admits/creates the workspace and emits quoted environment exports; `inspect --shell` emits the same routes without mutation. Neither relocates Cargo/Rustup homes, credentials, user data, or existing build directories. The separate managed `target-agent` lane retains its resource policy and accepts only an explicitly pinned root link as described in the owned native agent workflow.

### Project Structure

```
script-kit-gpui/
├── src/                    # Rust application source
│   ├── main.rs            # Entry point, window setup
│   ├── protocol/          # JSON message protocol
│   ├── prompts/           # Prompt implementations
│   ├── terminal/          # Terminal emulator
│   ├── one_window/        # The one window: host for the sk-one-window reducer
│   ├── intent/            # Jev intent engine (sentence → plan)
│   ├── notes/             # Notes (a view under the main input)
│   └── ai/                # Agent Chat runtime + context assembly
├── crates/
│   └── sk-one-window/     # The one window's rules, with no UI and no clock
├── scripts/
│   └── kit-sdk.ts         # The SDK (preloaded into scripts)
├── tests/
│   ├── smoke/             # End-to-end tests
│   └── sdk/               # SDK method tests
└── ~/.scriptkit/               # User workspace (plugin-based)
    └── kit/
        ├── main/          # Primary plugin
        │   ├── scripts/    # Executable TypeScript scripts
        │   ├── scriptlets/ # Scriptlet bundles
        │   ├── skills/     # Agent-first reusable AI units
        │   └── agents/     # Legacy agent definitions (compatibility)
        ├── config.ts       # Configuration
        ├── theme.json      # Theme customization
        ├── package.json
        └── tsconfig.json
```

### Running Tests

```bash
# Validation gate used for release tags
make verify

# Full local ship check (includes macOS bundle build + sanity check)
make ship-check

# SDK smoke command against the running operator ./dev.sh session
bash scripts/agentic/session.sh send dev-watch \
  '{"type":"run","path":"'"$(pwd)"'/tests/smoke/hello-world.ts"}' --await-parse
```

### Building for Release

```bash
# Optimized binary
cargo build --release --bin script-kit-gpui

# macOS app bundle
cargo install cargo-bundle
bash scripts/prepare-goose-sidecar.sh
cargo bundle --release --bin script-kit-gpui
bash scripts/install-goose-sidecar-into-bundle.sh

# Verify bundle contents
bash scripts/verify-macos-bundle.sh
```

## Features

### Built-in Capabilities

- **Intent** - Say what should happen; Jev reads it and the list offers what it could mean
- **Voice** - Hold the voice key in any app to type at its cursor or ask for something, in the same window
- **Clipboard History** - Access your clipboard history (enable in config)
- **App Launcher** - Quick launch applications
- **Window Switcher** - Switch between open windows (enable in config)
- **Notes** - Markdown notes that open under the main input (`Cmd+Ctrl+N`)
- **Agent Chat** - Press Command+Enter to open the model's chat under the main input with the current context staged
- **System Tray** - Menu bar icon with quick actions
- **Global Hotkeys** - Trigger scripts from anywhere

### Prompt Types

| Prompt | Description |
|--------|-------------|
| `arg(placeholder, choices?)` | Text input with optional choices |
| `div(html)` | Display HTML/Tailwind content |
| `editor(content?, language?)` | Multi-line code editor |
| `fields(definitions)` | Form with multiple inputs |
| `form(html)` | Custom HTML form |
| `path(options?)` | File/folder picker |
| `drop()` | Drag and drop zone |
| `hotkey(placeholder?)` | Capture keyboard shortcut |
| `mini(placeholder, choices)` | Native compact-choice prompt |
| `micro(placeholder, choices)` | Native minimal-choice prompt |
| `term(command?)` | Interactive terminal |
| `chat(options?)` | Chat interface |

`mic()`, `webcam()`, `eyeDropper()`, floating `widget()` windows, and legacy
`setPanel()` / `setPreview()` / `setPrompt()` mutations are not supported by the
GPUI host. They fail explicitly with `ERR_UNSUPPORTED_SDK_FEATURE` and an
actionable alternative instead of pretending to succeed. Search **SDK
Reference** in the launcher for the host-owned, versioned capability catalog.

## AI & Context Features

Script Kit exposes desktop context and UI state to scripts and AI agents through protocol commands and MCP resources.

### Agent Chat

Agent Chat is the one AI chat: the model's chat view under the main window's
input. Some internal helpers and compatibility types still use `tab_ai_*`
naming, but those are implementation details rather than separate chat products.

**Entry path:**
- Command+Enter with nothing typed opens a clean chat (the capsule reads "Agent"). With words typed, it reads "Do It" and runs Jev's first reading, the same row ⌘1 runs.
- Every way in (⌘↵, the SDK's `aiStartChat`, a failed script, the tray, stdin, dictation, Notes) builds a `ChatEntry`; what goes with it waits in the line above the input and is sent with the first turn.
- Past chats are rows under the same input ("Past chats"); ↵ reopens one.

**Close semantics:**
- `Escape` goes back one step and gives the typed sentence back to the input.
- Tool calls, the model's questions, and sign-in are rows in the chat, never dialogs.

**Runtime contract:**
- Entry goes through `ScriptListApp::one_window_open_chat` (library code: `ai::agent_chat::ui::entry::open_chat`); the view is hosted by `src/one_window/chat_host.rs` and the conversation state lives in `src/ai/agent_chat/ui/thread.rs`.
- Agent selection, model preferences, and profiles live under the `ai` block in `~/.scriptkit/config.ts` and are backed by the Agent Catalog.
- Context bundle: `~/.scriptkit/context/latest.md` (deterministic path)
- Context assembly still uses compatibility-named helpers such as `snapshot_tab_ai_ui()`, `capture_context_snapshot(CaptureContextOptions::tab_ai_submit())`, and `build_tab_ai_context_from()`.
- Compatibility-named types such as `TabAiContextBlob` remain the schema contract backing Agent Chat context capture.

### Element Introspection (`getElements`)

Scripts can query the visible UI surface to discover what elements are currently displayed — inputs, choices, buttons, panels, and lists. This enables AI-driven automation that targets elements by stable semantic IDs.

**Request:**
```json
{"type": "getElements", "requestId": "elm-1", "limit": 50}
```

- `requestId` (string, required) — correlation ID for the response
- `limit` (number, optional) — max elements to return (default 50, clamped 1–1000)

**Response:**
```json
{
  "type": "elementsResult",
  "requestId": "elm-1",
  "elements": [
    {"semanticId": "input:filter", "type": "input", "value": "app", "focused": true},
    {"semanticId": "list:choices", "type": "list", "text": "2 items"},
    {"semanticId": "choice:0:apple", "type": "choice", "text": "Apple", "value": "apple", "selected": true, "index": 0}
  ],
  "totalCount": 3,
  "truncated": false,
  "focusedSemanticId": "input:filter",
  "selectedSemanticId": "choice:0:apple",
  "warnings": []
}
```

**Semantic ID format:** `input:filter`, `list:choices`, `choice:<index>:<value>`, `button:<index>:<label>`, `panel:<type>`

**Observation receipts** are included in every response:
- `focusedSemanticId` — which element has keyboard focus
- `selectedSemanticId` — which choice/item is currently selected
- `truncated` — `true` if elements were capped by limit
- `warnings` — machine-readable codes like `panel_only_div_prompt` when a view has limited introspection

### MCP Context Resources

Script Kit exposes desktop context as an MCP resource that AI agents can read to understand what the user is currently doing.

**`kit://context`** — Full desktop snapshot:
```json
{
  "schemaVersion": 1,
  "selectedText": "function hello() { ... }",
  "frontmostApp": {"pid": 1234, "bundleId": "com.apple.Safari", "name": "Safari"},
  "menuBarItems": [{"title": "File", "enabled": true, "children": [...]}],
  "browser": {"url": "https://docs.rs/gpui"},
  "focusedWindow": {"title": "PROTOCOL.md", "width": 1440, "height": 900},
  "warnings": []
}
```

**Profiles** control which fields are captured:

| URI | Fields | Use Case |
|-----|--------|----------|
| `kit://context` | All fields | Comprehensive context |
| `kit://context?profile=minimal` | App, browser, window (no text/menu) | Low-token overhead |
| `kit://context?selectedText=1&menuBar=0` | Custom field selection | Fine-grained control |
| `kit://context/schema` | Schema JSON | Discover profiles, parameters, diagnostics |
| `kit://context?diagnostics=1` | Snapshot + field status | Debug capture failures |

Per-field flags: `selectedText`, `frontmostApp`, `menuBar`, `browserUrl`, `focusedWindow` — each accepts `0`/`1`/`true`/`false`.

### Context Parts

Context parts attach structured desktop state to AI interactions. Agent Chat stages context automatically and also supports attaching context via slash commands:

| Command | Context Attached |
|---------|-----------------|
| `/context` | Desktop snapshot (minimal profile) |
| `/context-full` | Desktop snapshot (full profile) |
| `/selection` | Selected text only |
| `/browser` | Browser URL only |
| `/window` | Focused window info only |

Context parts can also be file attachments. All parts are resolved at submit time with partial-failure tolerance — if one part fails (e.g., browser not available), successful parts are still included and failures are tracked in a resolution receipt.

**Resolution receipt structure:**
```json
{
  "attempted": 2,
  "resolved": 1,
  "failures": [{"label": "Browser URL", "source": "kit://context?browserUrl=1", "error": "No browser detected"}],
  "promptPrefix": "<context source=\"kit://context?profile=minimal\">...</context>"
}
```

### Deterministic Transactions (`waitFor` + `batch`)

AI agents can execute verifiable UI transactions without sleeps or polling loops. The `waitFor` command polls a condition until satisfied, and `batch` chains multiple atomic commands into a single request.

**Agent workflow:** set input → wait for choices to render → select by value → submit. No timing guesses required.

The JSON below illustrates the general protocol, not launch authorization or an owned target selector. In the owned agent loop above, bind requests to the returned exact instance and current `expected` identity; visible-window/focus waits do not authorize desktop takeover.

```json
{
  "type": "batch",
  "requestId": "txn-1",
  "commands": [
    {"type": "setInput", "text": "apple"},
    {"type": "waitFor", "condition": "choicesRendered", "timeout": 1000},
    {"type": "selectByValue", "value": "apple", "submit": true}
  ]
}
```

The app replies with per-command results including elapsed time and selected values:

```json
{
  "type": "batchResult",
  "requestId": "txn-1",
  "success": true,
  "results": [
    {"index": 0, "success": true, "command": "setInput"},
    {"index": 1, "success": true, "command": "waitFor", "elapsed": 17},
    {"index": 2, "success": true, "command": "selectByValue", "value": "apple"}
  ],
  "totalElapsed": 24
}
```

Available wait conditions: `choicesRendered`, `inputEmpty`, `windowVisible`, `windowFocused`, plus detailed conditions like `elementExists`, `elementFocused`, and `stateMatch`. See `WaitCondition` in [`src/protocol/types/batch_wait.rs`](src/protocol/types/batch_wait.rs) for the full reference.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `make verify`
5. Submit a pull request

See `AGENTS.md` for detailed development guidelines.

## License

MIT License - see LICENSE file for details.

## Links

- [Script Kit Website](https://scriptkit.com)
- [GPUI Documentation](https://gpui.rs)
- [Bun Runtime](https://bun.sh)
- [Zed Editor](https://zed.dev) (GPUI origin)

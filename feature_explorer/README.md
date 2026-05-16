# Script Kit Feature Explorer

Standalone XState-driven wireframe for exploring the maintained feature atlas in `../feature-map/features`.

## Commands

```bash
npm install
npm run generate
npm run dev -- --port 5177
npm run build
```

`npm run generate` parses every completed feature chapter into `src/data/features.generated.json`. Raw Oracle coverage counts only directories with `answer.md`; failed or incomplete attempts are reported separately. `npm run build` regenerates that snapshot, type-checks the XState UI, and builds the Vite app.

## Current Coverage

The explorer currently derives coverage from the 47 maintained chapters in `feature-map/features/*.md`.

- The outer `featureExplorerMachine` owns feature selection, search, tab mode, selected workflow, selected state, and selected interaction.
- Authored machines in `src/state/authoredFeatureMachines.ts` provide richer runtime and wireframe semantics for Features 001 through 040.
- The per-feature runtime machine in `src/state/featureRuntime.ts` loads authored machines when present, otherwise derives executable XState nodes from each chapter's State Machine table, converts `Exits to` rows into explicit state transitions, and infers scenario-event targets from Interaction Matrix and Keystrokes text before using a fallback.
- The Machine tab is the fastest visual audit path for whether a chapter has enough structured state/event data to support refactoring scenarios.
- The Wireframe tab hosts registered playable mockups; the first custom slice covers Root Unified Search across Features 001, 008, 009, 010, 011, and 012, while every other chapter gets a generic feature-map-derived mock surface.

## Known Limits

The runtime derivation is still conservative where chapters do not state exact transition targets in a machine-readable form. The Machine tab exposes authored-machine status and fallback-event counts so weakly mapped chapters are visible and can be targeted by future Oracle XState passes.

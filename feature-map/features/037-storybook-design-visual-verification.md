# 037 Storybook, Design Explorer, and Visual Verification

Storybook is Script Kit's visual state lab and proof system for supported surfaces, adoptable design variants, and screenshot-backed visual verification.

Raw Oracle reference: [answer](../raw-oracle/037-storybook-design-visual-verification/answer.md), [prompt](../raw-oracle/037-storybook-design-visual-verification/prompt.md), [bundle map](../raw-oracle/037-storybook-design-visual-verification/bundle-map.md), [full log](../raw-oracle/037-storybook-design-visual-verification/output.log), [session metadata](../raw-oracle/037-storybook-design-visual-verification/session.json).

## Executive Summary

Feature 037 covers the Storybook catalog, StoryBrowser UI, Design Explorer and adopted-surface selections, Design Gallery / Design Picker verification, and the agentic visual proof pipeline used to make visual regressions inspectable.

The governing contract is that registered Storybook coverage represents supported app states or adoptable variations. Mock-only design experiments are useful for comparison, but they must not masquerade as product truth in the primary catalog or as proof of runtime parity.

## What Users Can Do

- Browse supported visual states in the StoryBrowser catalog.
- Search stories and inspect preview panes for launcher surfaces, popup windows, secondary windows, and reusable components.
- Compare variants while seeing whether each side is production state, deterministic fixture state, or design-only mock data.
- Adopt configured Storybook variations into live surfaces through persisted selection state.
- Open Design Gallery and Design Picker paths for visual review and catalog selection.
- Capture strict visual evidence with state receipts, target identity, screenshots, and content audits.
- Use machine-readable catalog JSON before opening a visual surface.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| Storybook catalog | Registered story and variant inventory with role, surface, compare readiness, and adoption metadata. | `src/storybook/diagnostics.rs` |
| StoryBrowser | Interactive Storybook UI with searchable story list, preview pane, compare mode, adoption flow, theme/design controls, and screenshot trigger. | `src/storybook/browser.rs` |
| Catalog role | Classification of a story as canonical state, adoptable variation, or design experiment. | `src/storybook/story.rs` |
| Representation quality | Evidence marker for live surface, presenter fixture, or design experiment coverage. | `src/storybook/adoption.rs` |
| Adoptable surface | Typed live-surface value resolved from a persisted Storybook selection. | `src/storybook/adoption.rs` |
| Design Gallery | Runtime utility surface and visual-verification target for design catalog review. | `src/render_builtins/design_gallery.rs` |
| Design Picker matrix | Agentic script that sweeps shipped design catalog ids across mini/full screenshots and state receipts. | `scripts/agentic/design-picker-visual-matrix.ts` |
| Strict screenshot | Visual proof step with target identity, PNG content audit, and fail-closed blank/black rejection. | `scripts/agentic/verify-shot.ts` |
| Image library manifest | Surface-navigator output tying route state, elements, capture target, screenshot, and audit receipt together. | `scripts/agentic/surface-navigator.ts` |
| Storybook window registry | Counted lifecycle registry separating primary Storybook windows from child previews/popups. | `src/storybook/browser.rs` |

## Entry Points

| Entry point | User intent | Expected target |
|---|---|---|
| `cargo run --bin storybook` | Open interactive visual lab | StoryBrowser window |
| `--catalog-json` | Inspect catalog state without opening GPUI | Serialized story catalog snapshot |
| `--surface-resolution-json` | Inspect adopted-surface resolution | Serialized selected-surface snapshot |
| `--story <id> --variant <id>` | Open a specific story variant | StoryBrowser preview selection |
| `--compare` | Compare variants | StoryBrowser compare pane |
| `--adopt` | Persist an adoptable variation | Storybook selection store |
| `--screenshot` | Capture Storybook visual output | Screenshot path for selected story/variant |
| Design Gallery command aliases | Open runtime design catalog | `AppView::DesignGalleryView` |
| Design Picker matrix script | Sweep design catalog visuals | Mini/full screenshots and state receipts |
| Surface navigator capture | Capture product-surface visuals | Manifest, state/elements, strict screenshot receipt |

## User Workflows

### Inspect Catalog Honesty

The operator runs `cargo run --bin storybook -- --catalog-json` before visual review. The catalog must expose story role, surface, compare readiness, selected variant, per-variant metadata, and adopted-surface coverage so drift is visible without relying on screenshots.

### Browse A Supported State

The user opens Storybook, searches a story, selects a variant, and inspects the preview pane. Canonical states and adoptable variations are allowed in the primary catalog; design experiments belong outside primary registered coverage.

### Compare Variants

The user opens compare mode for a comparable story. Compare labels must disclose whether each side is production state, deterministic production fixture data, or design-only mock data so mock-vs-mock exploration is not mistaken for production parity.

### Adopt A Variation

The user runs `cargo run --bin storybook -- --story <story-id> --variant <variant-id> --adopt`. Valid adoption writes the selected variant without opening a GPUI window, and `--surface-resolution-json` reports the resolved live typed surface and fallback status.

### Review Design Gallery

The user opens Design Gallery through `design-gallery`, `designgallery`, or `design gallery` dispatch aliases. The route flips to `AppView::DesignGalleryView`, resizes through the main-window path, exposes the `designGallery` semantic surface, and owns a Select-only footer contract.

### Sweep Design Picker Visuals

The operator runs the visual matrix script across shipped catalog ids. The script captures mini and full screenshots, verifies kit/state design envelopes, and cleans up the picker after the sweep.

### Capture Image Library Proof

The operator uses `surface-navigator.ts --capture` for product-surface visual proof. The manifest must include resolved target identity, final observation, pre-capture state/elements, capture target, strict screenshot, and content audit.

### Add A New Storybook State

The implementer identifies the owning behavior skill, chooses live-surface or deterministic presenter-fixture representation, adds stable story and variant ids, updates diagnostics when metadata changes, adds a narrow source contract, then escalates to runtime or visual proof only when the change needs it.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Inspect catalog | `--catalog-json` | No window | CLI command | Storybook diagnostics snapshot | JSON catalog returned | Catalog JSON audit |
| Open visual lab | `cargo run --bin storybook` | StoryBrowser | Launch command | Storybook binary / browser | Searchable visual lab opens | Storybook lifecycle proof |
| Search stories | StoryBrowser | Catalog list | Type query | Browser search state | Matching story rows shown | StoryBrowser state/elements |
| Select variant | StoryBrowser | Preview mode | Click row / keyboard navigation | Selected story and variant state | Preview renders chosen variant | Catalog and preview proof |
| Compare variants | StoryBrowser | Compare mode | Compare action | Compare panel contract | Two variants shown with evidence labels | Compare contract tests |
| Adopt variant | CLI | Adopt command | `--adopt` | Adoption store write | Selection persists | Surface-resolution JSON |
| Resolve adopted surface | CLI | Resolution command | `--surface-resolution-json` | AdoptableSurface resolver | Typed surface/fallback returned | JSON snapshot audit |
| Open Design Gallery | Dispatcher | Design gallery route | Command alias | Design Gallery presenter | Design catalog surface shown | State-first designGallery receipt |
| Preview design | Design Picker | Preview selection | Row focus | Preview-only state | Persisted selection unchanged | Picker persistence test |
| Commit design | Design Picker | Focused row | Enter / row click | Canonical catalog id write | Design selection persists | State and storage receipt |
| Cancel preview | Design Picker | Previewed row | Escape / Cmd+W | Restore previous selection | Preview discarded | Picker lifecycle proof |
| Capture surface | Surface navigator | Target surface ready | `--capture` | verify-shot strict capture | Manifest and PNG audit written | Content-audited screenshot receipt |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Catalog unavailable | Startup or failed diagnostics | Catalog ready, error | Story registration and diagnostics must load. |
| Catalog ready | Diagnostics loaded | Browsing, catalog audit failed | Primary catalog may include only canonical states and adoptable variations. |
| Browsing stories | StoryBrowser open | Previewing variant, compare mode, adoption requested, screenshot requested | Selected story and variant ids must exist. |
| Previewing variant | Browsing stories | Browsing stories, compare mode, adoption requested, screenshot requested | Variant representation metadata must be known. |
| Compare mode | Previewing variant | Previewing variant, screenshot requested | Both sides must disclose data-source quality. |
| Adoption requested | CLI or StoryBrowser adoption action | Adopted surface resolved, adoption failed | Story and variant must be adoptable and stable. |
| Adopted surface resolved | Successful adoption or resolution query | Browsing stories, catalog ready | Resolution reports fallback status. |
| Design Gallery visible | Dispatch alias | Design Picker, catalog ready | Semantic surface is `designGallery`; footer is Select-owned. |
| Design Picker previewing | Design Picker row focus | Design committed, preview canceled | Preview must not write persisted state. |
| Design committed | Enter or row click | Design Gallery visible | Canonical catalog id must be written. |
| Visual capture requested | Surface navigator or Storybook screenshot | Visual proof accepted, infrastructure failure | Target identity must be stable. |
| Visual proof accepted | Strict screenshot success | Catalog ready | PNG content audit must reject blank or black captures. |
| Infrastructure failure | Screenshot ambiguity, blank PNG, missing popup bounds | Retry after infrastructure fix | Ambiguous captures fail closed. |
| Storybook closing | Primary or child window close | Process exit, browser still running | Registry exits only when primary and child counts are zero. |

## Visual And Focus States

- StoryBrowser catalog list with search query, selected story, selected variant, theme, design variant, preview mode, and focus state.
- Preview pane for canonical launcher, popup, secondary-window, and component states.
- Compare pane with side-specific evidence labels for production state, deterministic fixture, or design-only mock data.
- Main menu states: populated, empty, selected row, bottom/footer-safe reveal, frontmost-app paste, ACP-ready footer, and ACP-not-ready footer.
- Dictation states: idle/hidden, quiet recording, active speech, Script Kit target, ACP target, external-app target, stop confirmation, transcribing, finished, and error.
- Confirm popup states: in-window live confirm route and destructive-warning popup treatments for design review.
- Built-in browser states, including presenter-backed list states and File Search loading skeleton.
- Quick Terminal states: cold empty, active PTY content, theme-sensitive chrome, and apply-back-ready deterministic fixture.
- Design Gallery visible rows and Select-only footer.
- Design Picker preview, commit, cancellation, and persisted-selection states.
- Attached popup screenshot states with parent-window capture and exact popup crop bounds.

## Keystrokes And Commands

| Key/command | Context | Behavior |
|---|---|---|
| `cargo run --bin storybook -- --catalog-json` | CLI | Emits machine-readable catalog snapshot without opening a GPUI window. |
| `cargo run --bin storybook -- --surface-resolution-json` | CLI | Emits adopted-surface resolution snapshot. |
| `cargo run --bin storybook -- --story <id> --variant <id> --adopt` | CLI | Persists an adoptable story variant selection. |
| `cargo run --bin storybook -- --story <id> --variant <id> --compare` | CLI / visual lab | Opens comparison mode for the selected story variant. |
| `SCRIPT_KIT_STORYBOOK_GRID=1` | Storybook visual review | Enables optional measurement grid; default is off to avoid distorting visual judgment. |
| `design-gallery` | Dispatcher | Opens Design Gallery. |
| `designgallery` | Dispatcher | Opens Design Gallery alias. |
| `design gallery` | Dispatcher | Opens Design Gallery alias. |
| Enter | Design Picker row | Commits the canonical design catalog id. |
| Row click | Design Picker row | Commits the canonical design catalog id. |
| Escape | Design Picker preview | Cancels preview and restores previous selection. |
| Cmd+W | Design Picker preview | Cancels preview and restores previous selection. |
| `bun scripts/agentic/design-picker-visual-matrix.ts --session design-variants-overhaul --sizes mini,full --designs all --capture-screenshots --verify-state-receipts --cleanup` | Agentic visual matrix | Captures and verifies all shipped design catalog ids. |
| `bun scripts/agentic/surface-navigator.ts --capture ...` | Agentic visual proof | Navigates a product surface, captures strict screenshot proof, and writes a manifest. |

## Actions And Menus

StoryBrowser actions revolve around selecting stories, selecting variants, toggling compare mode, adopting an adoptable variation, changing theme/design controls, and triggering screenshots. Actions should preserve story identity and evidence metadata rather than using copy-only labels as source of truth.

Design Picker actions are intentionally split between preview and commit. Focus/hover preview is reversible; Enter and row click commit the selected canonical catalog id; Escape and Cmd+W restore the previous selection.

## Automation And Protocol Surface

| Surface | Target/proof | Notes |
|---|---|---|
| Storybook catalog | `cargo run --bin storybook -- --catalog-json` | First audit gate for role, surface, comparable status, selected variant, representation, and adoption coverage. |
| Adopted surface | `--surface-resolution-json` | Verifies resolved typed surface and fallback status. |
| Storybook static contracts | `cargo test --test storybook_adoption_contract` and adjacent Storybook tests | Source-level checks for adoption, main-menu render path, footer, compare, context picker, lifecycle, and adoption audit behavior. |
| Design Gallery route | State-first semantic-surface receipt | Proves command aliases, `AppView::DesignGalleryView`, resize path, and `designGallery` wire string. |
| Design Picker matrix | `scripts/agentic/design-picker-visual-matrix.ts` | Captures Mini/Full screenshots and verifies kit/state receipts for design ids. |
| Surface navigator | `scripts/agentic/surface-navigator.ts` | Writes manifest entries with final observation, elements, capture target, screenshot, and content audit. |
| Strict screenshots | `scripts/agentic/verify-shot.ts` | Rejects blank/black/ambiguous captures and records content audit. |
| Attached popups | verify-shot popup capture strategy | Parent-window capture requires exact target bounds for crop proof. |

## Data, Storage, And Privacy Boundaries

- Catalog JSON exposes story and variant metadata, not private user data.
- Adopted selections persist stable story/variant ids and resolve to typed live surface values.
- Design Picker preview state must not mutate persisted design selection until explicit commit.
- Visual proof artifacts can contain screenshots; they require correct target identity and should be handled as review evidence, not casual logs.
- Screenshot proof without PNG content audit is not acceptable evidence because blank, black, wrong-window, and permission-failed images can otherwise pass as files.

## Error, Empty, Loading, And Disabled States

- Invalid story ids fail with structured nonzero CLI errors.
- Invalid variant ids fail with structured nonzero CLI errors.
- Catalog role drift fails the catalog audit when design experiments appear in the primary registered catalog.
- Missing representation metadata makes evidence quality unknowable and should fail review.
- Old PNG-backed runtime fixtures are not acceptable registered primary coverage.
- Compare mode must fail review when it hides mock-only data source.
- Screenshot ambiguity, blank PNGs, black PNGs, missing popup bounds, and wrong target identity are infrastructure failures.
- Storybook window close must not exit while child preview or popup windows remain registered.

## Code Ownership

| Area | Primary files | Notes |
|---|---|---|
| Storybook CLI | `src/bin/storybook.rs` | Starts Storybook, selects story/variant, compares, adopts, screenshots, and emits JSON diagnostics. |
| Storybook core | `src/storybook/mod.rs`, `src/storybook/story.rs` | Defines story identity, catalog role, render path, variants, and live variant hooks. |
| StoryBrowser UI | `src/storybook/browser.rs` | Owns interactive browser state, preview/compare/adoption flow, focus, screenshot trigger, and lifecycle registry. |
| Diagnostics | `src/storybook/diagnostics.rs` | Serializes catalog entries, selected variants, counts, and surface summaries. |
| Adoption contracts | `src/storybook/adoption.rs` | Defines representation/data-source enums, compare contracts, footer snapshots, `VariationId`, and `AdoptableSurface`. |
| Concrete stories | `src/stories/*` | Thin wrappers around surface and state families. |
| Design Gallery | `src/render_builtins/design_gallery.rs` | Runtime design catalog surface and filtering. |
| Visual scripts | `scripts/agentic/surface-navigator.ts`, `scripts/agentic/verify-shot.ts`, `scripts/agentic/design-picker-visual-matrix.ts` | State-first navigation, strict screenshot capture, pixel audit, and matrix verification. |
| Contracts | `tests/storybook_*`, `tests/design_*`, `tests/agentic_*`, `tests/verify_shot_*` | Source-level and agentic verification contracts. |

## Invariants And Regression Risks

- Primary catalog stories must be `canonicalState` or `adoptableVariation`, never unmarked `designExperiment`.
- Every registered variant needs representation metadata.
- Registered primary coverage must not use old runtimeFixture / PNG-backed proof as product truth.
- Main-menu stories must not hand-build mock rows or mock-only footer hints.
- Compare mode must label production state, deterministic fixture data, and design-only mock data honestly.
- Visual proof must include target identity, final state/elements, strict screenshot, and content audit.
- Surface proof should use receipts and `waitFor`/`batch`, not sleeps.
- Attached popup capture requires parent-window capture plus exact target bounds.
- Design Picker preview must not write persisted state.
- Design Gallery command aliases must stay reachable through stdin dispatchers.
- Storybook child preview close must not quit the whole Storybook process.
- Footer/chrome variants must not introduce duplicate footer rows or more than three launcher primary affordances.

## Verification Recipes

```bash
cargo run --bin storybook -- --catalog-json
cargo run --bin storybook -- --surface-resolution-json
cargo run --bin storybook -- --story <story-id> --variant <variant-id> --adopt
cargo test --test storybook_adoption_contract
cargo test --test storybook_main_menu_render_path_contract
cargo test --test storybook_footer_contract
cargo test --test storybook_compare_contract
cargo test --test storybook_context_picker_contract
cargo test --test storybook_lifecycle_contract
cargo test --test storybook_adoption_audit
bun scripts/agentic/design-picker-visual-matrix.ts --session design-variants-overhaul --sizes mini,full --designs all --capture-screenshots --verify-state-receipts --cleanup
bun scripts/agentic/surface-navigator.ts --capture ...
```

For visual changes, run the smallest proof that can fail on the regression: catalog/static checks first, state-first runtime receipts next, strict screenshot and content audit only when rendering evidence is required.

## Agent Notes

- Do not treat Storybook as automatic proof of production behavior; it proves visual state coverage, while live product behavior still needs the owning runtime route and state receipts.
- To verify catalog or adoption changes, start with `--catalog-json`, `--surface-resolution-json`, and the Storybook source-contract tests before escalating to screenshots.
- If visual proof fails, inspect target identity, final state/elements receipts, crop bounds, and PNG content audit before assuming the product UI regressed.
- This belongs to `storybook-design` when the work is catalog, StoryBrowser, stories, design adoption, or chrome audit coverage.
- This belongs to `agentic-testing` or `protocol-automation` when the work is runtime receipt capture, `surface-navigator`, `verify-shot`, target identity, or state/elements proof.
- Design Gallery is a product built-in surface; Storybook is a developer lab. Do not use one as a drop-in substitute for the other.
- Screenshots are only needed when rendered appearance is the acceptance criterion; behavior, routing, and ownership claims should use source contracts or state-first receipts.

## Open Questions And Gaps

- Storybook CLI/browser screenshots still need closer parity with the strict `verify-shot.ts` receipt and content-audit model.
- The documented agentic scripts `storybook_main_menu_parity.ts`, `storybook_lifecycle_theme.ts`, and `storybook_context_picker_parity.ts` should be rechecked locally before depending on them because the focused bundle did not include their contents.
- Form, path, env, editor, and SDK-spawned terminal prompt states remain next expansion candidates for presenter-backed or live-surface Storybook coverage.
- Design Gallery should remain in the screenshot matrix because its route and state receipts are stable enough for durable visual verification.
- Live automation semantic-surface inventory should continue comparing screenshotable surfaces against matrix coverage so stale or missing visual entries fail early.

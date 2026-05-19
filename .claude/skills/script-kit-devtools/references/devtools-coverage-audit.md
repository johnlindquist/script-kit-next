# Script Kit DevTools Coverage Audit

This audit maps the current agent-facing DevTools surface to the UX/UI bug-investigation scenarios it can support. It favors protocol, MCP, and CLI primitives over prewritten recipes.

## Existing Foundation

The app already exposes a partial DevTools foundation:

- `getState`: main-window state, prompt-specific state, visible counts, active footer, surface contract, active popup contract, screenshot identity.
- `getElements(target)`: visible semantic rows, focused/selected ids, roles/kinds, labels, disabled reasons, warnings, and non-main snapshots for several automation window kinds.
- `getLayoutInfo`: main-window component bounds and layout metrics.
- `listAutomationWindows` and `inspectAutomationWindow`: registered window identity, kind, bounds, parent identity, and focused window registry state.
- `waitFor` and `batch`: transaction-style state waits and target-scoped interaction commands.
- `simulateKey`: protocol key routing with follow-up state/element receipts.
- `captureScreenshot` / `captureWindow` / `verify-shot.ts`: visual capture with strict target identity and screenshot metadata.
- MCP `computer/*`: read-only native observation for windows, apps, menus, screenshots, screens, and permissions.
- `scripts/agentic/session.sh`: session lifecycle, JSONL stdin, app logs, response logs, health, and cleanup.

## Coverage Matrix

| Capability | Current State | Important Gaps |
| --- | --- | --- |
| Surface/window identity | Strong for registered windows and exact targets. | Need capability discovery that tells agents which targets support which state/layout/action primitives. |
| Semantic tree | Good for main and many built-ins; partial for non-main windows. | Some built-in and popup surfaces still fall back to panel-only receipts. Need uniform roles, labels, action ids, disabled reasons, and owner metadata. |
| Layout bounds | Main-window `getLayoutInfo` exists. | Need target-scoped layout for Notes, actions dialogs, prompt popups, ACP detached windows, and DivPrompt content. |
| Text fit/clipping | Exists in specialized scripts/receipts. | Needs protocol-level primitives for rendered text bounds, measured width, available width, wrap lines, truncation intent, and overlap pairs. |
| Scroll state | Main list exposes some footer-safe scroll geometry. | Need scroll position, content height, viewport height, sticky headers, shadows, and selected-row visibility across prompts, DivPrompt, Notes, popups, and virtualized lists. |
| Popup/menu inspection | Actions popup identity and semantic row work is improving. | Need anchor rects, route stack, section bounds, keycap/shortcut layout, hover/focus row state, and post-resize anchor/bounds invariants. |
| Resize comparison | Some recipes measure main choices, Notes, and DivPrompt. | Need a generic before/after measurement primitive with stable target ids, dimensions, layout fingerprint, and failure deltas. |
| Focus and keyboard ownership | State and surface contracts expose ownership policies. | Need focus ring bounds, tab order, active owner before/after key delivery, and wrong-surface input refusal receipts. |
| Hit targets and pointer affordances | Weak without native pointer escalation. | Need semantic hit target bounds, clickable/disabled state, pointer hover state, and protocol-click receipts that do not mutate unsafe state. |
| Screenshot-to-semantics | `verify-shot.ts` and screenshot identity exist. | Need generalized screenshot analysis: nonblank, crop target match, semantic text agreement, contrast samples, occlusion/overlap candidates. |
| Contrast/theme readability | Specialized theme contrast receipt exists. | Need first-class token/color/contrast receipts per surface/state/theme/scale factor. |
| AX/accessibility parity | Native observation tools exist. | Need structured AX node parity with semantic elements: role, label, focus order, disabled state, activation semantics. |
| Before/after red-green proof | Mostly ad hoc per recipe. | Need a standard investigation artifact that stores paired red/green measurements with same metric names and target ids. |
| Bug intake and investigation transcript | Not first-class. | Need CLI/API support for report intake, hypotheses, pivots, actions, receipts, classification, likely owner, and missing primitive. |

## Priority Scenarios

Build the DevTools surface around these scenarios before adding more recipes:

1. User screenshot says an actions popup menu is clipped, anchored wrong, unreadable, or missing expected rows.
2. Main menu with dynamic choices does not resize correctly as choices appear/disappear.
3. Notes window grows, shrinks, or scrolls incorrectly with long notes and embedded ACP.
4. `div`, `arg`, `form`, `fields`, and other prompt containers become too tall, need scrolling, or hide footer/input controls.
5. Long labels, file paths, snippets, emoji, RTL text, or mixed grapheme content clip or overlap.
6. Footer buttons, disabled actions, shortcut hints, and keycaps do not match action availability.
7. Focus ring, selected row, hover row, or tab order becomes invisible or owned by the wrong surface.
8. Popups, inline popovers, context pickers, and menus drift from anchors after filtering or resizing.
9. Empty/error/loading/retry states keep stale rows, stale errors, or unsafe actions.
10. Screenshot proof disagrees with semantic state or captures the wrong window.

## Build Direction

The next implementation wave should add DevTools primitives and a small investigator CLI instead of adding more long stress recipes:

- `devtools.inspect`: first slice exists as `bun scripts/devtools/inspect.ts --session <name> --start --show --main|--focused|--target-id <id>|--target-kind <kind>`. It returns target identity, state, elements, layout, screenshot metadata, capabilities, warnings, errors, missing fields, and recommended next primitives.
- `devtools.coverage`: first slice exists as `bun scripts/devtools/coverage.ts --surface <id>|--domain <id>|--markdown`. It returns Chrome-DevTools-inspired domains, surface feature and shortcut coverage, supported primitives, missing runtime primitives, and recommended next work without pretending those runtime primitives already exist.
- `devtools.measure`: first slice exists as `bun scripts/devtools/measure.ts --inspect <inspect.json> --coverage <coverage.json> --surface <id>`. It turns inspect and coverage receipts into available measurements, planned measurement gaps, missing runtime primitives, fail-closed status, and next measurement work for layout, text fit, overlap, scroll need, resize deltas, popup anchor, focus ring, hit target, contrast, and media.
- `devtools.media.inspect`: first slice exists as `bun scripts/devtools/media.ts --coverage <dictation-coverage.json>`. It is intentionally passive and fail-closed until Dictation exposes microphone permission, device, model readiness, recording generation, audio levels, target delivery, transcript, cursor insertion, wrong-target refusal, hotkey, and cleanup receipts without System Settings or TCC mutation.
- `devtools.act`: safe user-like interaction using protocol channels first, with explicit escalation for GPUI/native input.
- `devtools.compare`: paired before/after measurements with stable metric names and delta classification.
- `devtools.investigate`: bug-report intake plus hypothesis/proof transcript, producing `.test-output/devtools-investigation-*.json`.

Recipes should be rebuilt as thin smoke/regression wrappers over those primitives.

## Chrome-Style API Coverage

The current coverage command and narrative map live in `references/devtools-api-coverage-map.md`. Treat that file as the DevTools API backlog before adding another scripted scenario.

The intended domain spread is targets/windows, elements/semantics, layout/box model, styles/theme/text fit, console/logs/events, sources/scripts/owners, performance/timeline, storage/resources/privacy, accessibility, input/focus/actions, media/sensors/permissions, screenshots/visual proof, and investigation records.

Notes and Dictation are first-class coverage targets. Notes must expose editor, preview, browse, trash, command-bar, recent-switcher, note-cart, embedded ACP, portal, draft, resize, scroll, and shortcut receipts. Dictation must expose passive permission/model readiness, recording state, audio levels, target delivery, transcript generation, wrong-target refusal, history preview/redaction, hotkey, and cleanup receipts before live dictation bugs are called green.

Oracle browser session `devtools-chrome-notes-dictation-api` reviewed this API direction on 2026-05-16 and reinforced the same split: coverage may be source-backed before runtime exists, but runtime proof must stay fail-closed until `devtools.measure`, `devtools.media.inspect`, `devtools.act`, `devtools.compare`, and `devtools.investigate` expose the needed receipts.

## First Inspect Slice Scenarios

The first inspect slice should be enough for agents to orient on these user reports and name the next missing primitive instead of guessing:

1. A screenshot shows the wrong Script Kit window, or the bug report does not name the surface.
2. An actions popup is visible, but the agent needs exact target identity before measuring clipping or anchor drift.
3. A main menu report needs visible count, selected/focused ids, footer state, and screenshot metadata in one receipt.
4. A Notes, ACP, PromptPopup, or detached window report needs to prove whether state/layout support is missing before adding code.
5. A screenshot disagrees with semantic rows, so the agent needs screenshot size, target bounds, hit points, semantic quality, and warnings together.

When `devtools.inspect` returns `status:"degraded"` or non-empty `missingFields`, the correct next step is to build or use the named primitive, not to bury the gap in a larger recipe.

## Oracle Scenario Matrix

Oracle produced the following scenario set on 2026-05-16 to keep the DevTools roadmap grounded in real user bug reports. `Inspect` means the first slice can orient and classify the target; `Measure`, `Compare`, and `Act` are later primitives needed for proof or safe reproduction.

The corrected 50-iteration Oracle planning artifact is `references/oracle-devtools-scenario-index.md`, with raw Oracle session output preserved in `references/oracle-devtools-scenario-iterations.md`. Prefer that index for roadmap planning; the table below is retained as the earlier single-response matrix.

| # | User bug prompt | Needed DevTools fields | Slice |
| --- | --- | --- | --- |
| 1 | Cmd-K opens the actions popup, but it is clipped at the bottom. | target identity, bounds, screenshot metadata, elements, missing popup layout | Inspect -> Measure |
| 2 | Actions popup opens on the wrong side of the launcher. | parent identity, popup bounds, screenshot crop, anchor rect | Inspect -> Measure |
| 3 | Filtering actions leaves a giant empty blank area under one row. | actions target, total count, visible rows, bounds before/after | Inspect -> Compare |
| 4 | Actions menu section headers receive arrow-key selection. | selected semantic id, row roles, selectable flags | Inspect -> Act |
| 5 | Actions menu is missing Copy Deeplink. | action ids, labels, sections, host context | Inspect |
| 6 | Disabled actions look enabled. | action disabled reason, visual state, row role | Inspect -> Measure |
| 7 | Enter executes a disabled or no-op action. | selected id, disabled reason, activation receipt | Inspect -> Act |
| 8 | Escape clears actions search instead of closing. | popup identity, input value, route stack, close result | Inspect -> Act |
| 9 | Actions child menu loses parent filter after back. | route stack, search text, selected row before/after | Inspect -> Compare |
| 10 | Popup stays registered after it visually closes. | window registry, focused window id, target visible flag | Inspect |
| 11 | Main menu does not grow when a prompt has many choices. | visible choice count, window bounds, layout info | Inspect -> Compare |
| 12 | Menu grows wider instead of taller for more choices. | window width/height, visible count, layout fingerprint | Inspect -> Compare |
| 13 | Choice count says 100, but only one row is shown after filtering. | choiceCount, visibleChoiceCount, element count | Inspect |
| 14 | Filtering to no results submits the old selected item. | input, visible count, selected id, footer disabled state | Inspect -> Act |
| 15 | Selected row is hidden behind the footer. | selected id, scroll geometry, footer bounds | Inspect -> Measure |
| 16 | Footer overlaps the list after resize. | content/list/footer bounds, active footer owner | Inspect -> Measure |
| 17 | Footer buttons are duplicated. | activeFooter, footer elements, native/fallback footer state | Inspect |
| 18 | Submit appears enabled with invalid fields. | footer disabled reason, field errors, prompt state | Inspect -> Act |
| 19 | Shortcut hint differs between row accessory and footer. | row shortcuts, footer key labels, action ids | Inspect -> Measure |
| 20 | Long file path clips without tooltip. | full text, visible text, text bounds, tooltip affordance | Inspect -> Measure |
| 21 | Emoji and ZWJ labels overlap row icons. | grapheme text, row/icon/text bounds | Inspect -> Measure |
| 22 | RTL cursor appears in the wrong place. | input value, cursor/selection rects, bidi text | Inspect -> Measure/Act |
| 23 | Long descriptions wrap into the footer. | footer bounds, text bounds, wrap lines | Inspect -> Measure |
| 24 | Preview text overlaps metadata chips. | selected row, preview identity, chip bounds | Inspect -> Measure |
| 25 | File Search preview exposes a raw private path. | selected row metadata, preview redaction fields | Inspect |
| 26 | Browser history exposes a private URL. | row label, source metadata, redacted URL fields | Inspect |
| 27 | Notes window does not grow for a long note. | Notes target, bounds, elements, editor mode | Inspect -> Compare |
| 28 | Notes window grows but never shrinks. | Notes bounds before/after tall and short input | Inspect -> Compare |
| 29 | Notes editor scroll jumps while typing. | editor state, scroll position, cursor position | Inspect -> Measure/Act |
| 30 | Notes embedded ACP receives input instead of the note editor. | Notes mode, exact target, ACP identity, input owner | Inspect |
| 31 | Markdown preview scroll is not synced with editor. | editor/preview identities and scroll positions | Inspect -> Measure/Act |
| 32 | Div prompt hides the end marker. | prompt type, content/viewport bounds, scroll need | Inspect -> Measure |
| 33 | Tall form prompt pushes Submit off-screen. | prompt type, content/input/footer bounds | Inspect -> Measure |
| 34 | Fields prompt does not focus the first invalid field. | focused id, validation rows, error state | Inspect -> Act |
| 35 | Path prompt permission error looks like an empty folder. | path status kind/message, status element | Inspect |
| 36 | Drop prompt exposes full local file paths in rows. | redacted file metadata, row values | Inspect |
| 37 | Hotkey prompt records a shortcut after cancel. | prompt type, hotkey elements, config fingerprint | Inspect -> Act/Compare |
| 38 | Template prompt tab order skips a field. | field elements, focused id, tab order | Inspect -> Act/Measure |
| 39 | Detached ACP receives input for the main ACP. | exact target, osWindowId, ACP state capability | Inspect |
| 40 | Detached ACP screenshot captures the wrong window. | automation id, osWindowId, screenshot target/crop | Inspect |
| 41 | ACP mention popup rows do not match semantic rows. | promptPopup target, elements, selected/focused ids | Inspect |
| 42 | ACP slash popup drifts after composer resize. | popup target, parent id, bounds, anchor rect | Inspect -> Measure |
| 43 | Model selector popup shows stale current model. | popup elements, selected id, model state | Inspect |
| 44 | Context insertion preview differs between File Search and ACP. | source row, destination identity, provenance | Inspect -> Act/Compare |
| 45 | Portal cancel does not restore composer cursor. | origin identity, cursor state, portal session | Inspect -> Act/Compare |
| 46 | Main search source chip remains after clearing input. | filter decorations, input value, chip elements | Inspect |
| 47 | Source filter syntax shows hints instead of file rows. | input decorations, visible rows, source status rows | Inspect |
| 48 | Lazy root files add rows and move selection. | selected id, visible rows, provider status | Inspect -> Compare |
| 49 | Recent/history duplicate rows appear twice. | stable keys, source metadata, visible fingerprint | Inspect -> Compare |
| 50 | Process Manager sort changes detail panel to wrong process. | selected row id, detail identity, sort state | Inspect -> Compare/Act |

The first live `devtools.inspect` run exposed one more concrete gap: `getState` can be too large for the current session RPC log path and arrive as invalid/truncated JSON. The inspector reports that as `missingFields:["target_state"]` with `status:"degraded"` instead of pretending state is available.

# Oracle DevTools Scenario Index

This index records the corrected 50 Oracle-planned scenario iterations. The detailed Oracle text is preserved in `oracle-devtools-scenario-iterations.md`; this file gives agents a compact routing map.

## Oracle Session Provenance

| Iterations | Domain | Oracle session |
| --- | --- | --- |
| 01-10 | Actions dialogs and popup menus | `devtools-oracle-iterations-actions-popups` |
| 11-20 | Prompt runtime containers and resizing | `devtools-oracle-iterations-prompts-resize` |
| 21-30 | Notes and AgentChat surfaces | `devtools-oracle-iterations-notes-agent_chat` |
| 31-40 | Filterable data-heavy surfaces | `devtools-oracle-iterations-filterable-data` |
| 41-50 | Native windowing, accessibility, theme, focus | `devtools-oracle-iterations-native-a11y` |

## Iteration Index

| ID | Scenario | Primary missing DevTools surface |
| --- | --- | --- |
| 01 | Actions nested submenu anchors to the wrong parent row after async row resize. | Popup anchor rects, placement rule, layout generation ids |
| 02 | Disabled action keycaps still dispatch disabled commands. | Accelerator binding inspection with enabled/suppression state |
| 03 | Oversized actions menu clips bottom rows instead of scrolling or flipping. | Popup clipping chain and item visibility fields |
| 04 | Stale async nested route opens for the previous action row. | Route request lifecycle and cancellation diagnostics |
| 05 | Keycaps overlap labels after actions dialog width changes. | Action row slot box model and overlap measurement |
| 06 | Disclosure arrow and keycap accessories render in the wrong slots. | Accessory role schema with rects and interaction semantics |
| 07 | Disabled parent action still opens a child submenu on hover. | Distinct enabled, executable, and revealable state |
| 08 | Parent actions popup resizes after child placement, leaving child anchored to stale bounds. | Popup layout timeline and post-resize anchor validation |
| 09 | Hidden child route accelerator wins over visible top-level keycap. | Accelerator scope inspection by route visibility |
| 10 | Resizing popup host desynchronizes focus, highlight, selection, and Enter dispatch target. | Stable action identity and focus consistency diagnostics |
| 11 | Prompt switches arg -> div -> choices and keeps stale height/scroll owner. | Runtime container chain and active scroll owner |
| 12 | Async choices refresh grows behind the footer while hidden rows remain selectable. | Choices versioning and selected-choice visibility |
| 13 | Main menu restored from external monitor opens off-screen on laptop display. | Display work-area metadata and restored-bounds validation |
| 14 | Custom div prompt HTML clips bottom controls with no usable scroll ancestor. | CSS overflow chain and nested scroll-container inspection |
| 15 | Arg input/validation height collides with choices list at small prompt heights. | Reserved prompt chrome height and overlap map |
| 16 | Form/fields prompt tabs to focused inputs that are outside the visible viewport. | Focus visibility and caret viewport diagnostics |
| 17 | Long path prompt names create horizontal overflow that breaks vertical scrolling. | Intrinsic text width and text overflow mode |
| 18 | Drop prompt file list grows past continue/cancel controls after large drag batch. | Drop-state schema and synthetic drop-file action |
| 19 | Template prompt live preview clips generated output and variables behind footer. | Compound prompt scroll policy and render-version layout |
| 20 | Hotkey prompt long chord/conflict list hides active chord or save button. | Keyboard capture event log and conflict layout metrics |
| 21 | Notes target identity becomes ambiguous after script-driven note creation. | Stable Notes window/note/editor/AgentChat target identity |
| 22 | Notes long markdown editor and preview disagree about scroll position. | Editor/preview scroll pairing and sync generation |
| 23 | Notes window resize leaves editor bounds stale after markdown preview toggle. | Notes-specific target-scoped layout and resize comparison |
| 24 | Notes embedded AgentChat receives input intended for note editor. | Notes mode, AgentChat identity, and input-owner receipts |
| 25 | Notes attachment/context chips overflow or dedupe incorrectly. | Attachment provenance, chip overflow, and dedupe diagnostics |
| 26 | Detached AgentChat screenshot or semantics target the wrong window. | Exact detached AgentChat target, native id, and screenshot crop binding |
| 27 | AgentChat composer popup drifts after composer growth or context insertion. | Composer anchor rects and popup placement generation |
| 28 | AgentChat slash/mention popup selected row disagrees with execution target. | Popup selected/focused/execution identity parity |
| 29 | Streaming AgentChat response resizes and hides cancel/progress affordances. | Stream progress layout and cancellation control visibility |
| 30 | Notes-hosted AgentChat and standalone AgentChat share stale session/context state. | AgentChat session provenance and context-source isolation |
| 31 | Clipboard history filter shows stale/redacted preview content. | Preview generation, redaction status, and selected item identity |
| 32 | App launcher results mismatch installed app identity, icon, or launch target. | App identity provenance and launch-target proof |
| 33 | Browser tabs/history search leaks private or stale URL metadata. | Source trust, redaction, and browser data generation |
| 34 | File search portal returns wrong selected file after filtering or portal cancel. | Portal provenance, selected file identity, return focus |
| 35 | Emoji picker recents/skin tone/virtual grid state drifts while scrolling. | Emoji base/variant ids and virtual grid identity |
| 36 | Process manager filtering/sorting attaches live metrics to wrong PID. | Process key and metric generation per row |
| 37 | Settings search rows match hidden metadata and disagree with nested control state. | Match source and setting source-of-truth fields |
| 38 | Source chips overflow while hidden filters remain active after removal. | Hidden active source ids and chip count semantics |
| 39 | Virtualized filterable list reuses rows while preview points to stale item. | Virtual slot id, data item id, rendered range, preview item id |
| 40 | Cross-surface preview leaks prior private content during loading. | Preview lifecycle, placeholder safety, redaction status |
| 41 | Window resurrects on a disconnected display after monitor topology changes. | Display topology, visible-frame intersection, and stale display id |
| 42 | Window moves between Retina and non-Retina displays but screenshots and hit areas use stale scale. | Coordinate-space descriptor and backing-scale convergence |
| 43 | Prompt is visible but keyboard input still belongs to the previous app. | First-responder chain and keyboard-owner snapshot |
| 44 | Transparent overlay blocks clicks even though the visible control is underneath. | Coordinate-specific pointer hit-test explanation |
| 45 | Accessibility tree does not match visible prompt controls. | Visual-to-AX mapping and parity diagnostics |
| 46 | High contrast, dark mode, and font scale create unreadable or clipped UI. | Effective style schema with contrast and text metrics |
| 47 | Notification banner steals focus and prompt never recovers keyboard ownership. | Interruption timeline and focus recovery ledger |
| 48 | Tray/menu popover leaves the prompt visually active but operationally blocked. | Native menu/modal-loop ownership state |
| 49 | Prompt crosses displays but remains assigned to the wrong macOS Space. | User-visible Space membership and display targeting |
| 50 | Screenshot says prompt is fine while user sees stale or occluded pixels. | Screenshot provenance, occlusion, and render epoch |

## Completion Rule

Treat this index as an Oracle-authored scenario source, not a test result. Each row still needs a concrete red/green proof through `devtools.inspect` and later `devtools.measure`, `devtools.compare`, `devtools.act`, or `devtools.investigate` before it can become a regression recipe.

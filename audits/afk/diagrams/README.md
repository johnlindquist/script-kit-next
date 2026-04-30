# AFK Audit — Coverage Diagrams

Visual artifact that summarizes audit-story coverage across the Script Kit GPUI app. Grows pass-over-pass so the operator can scan the audit run's output in seconds instead of scrolling through `log.md`.

## Layout

```
audits/afk/diagrams/
├── README.md         (this file — format conventions)
├── overview.md       (top-level surface map — every story lands here)
└── <surface>.md      (drill-downs — e.g. main-launcher.md, acp-chat.md)
```

`overview.md` is a single mermaid `flowchart` with a subgraph per surface family. Drill-down files zoom into one surface and show its subviews, transitions, and the stories that verified each edge.

## Node status convention

Every node in a diagram carries a status suffix:

- `✅` — at least one `[x]` story in `stories.md` proves the node's behavior
- `⚠️` — the corresponding story is `[!]` (tool gap recorded; not yet provable live)
- `⏳` — the corresponding story is `[ ]` (generated but not yet verified)
- `❌` — a story attempted the node and failed (none currently — reserved)

Nodes that do not have a dedicated story yet simply omit the suffix (fair warning that coverage is thin in that area).

## Class colors

The mermaid `classDef` palette in each diagram uses four classes so scanning flags gaps at a glance:

- `pass` — green background (verified)
- `gap` — amber background (tool gap; verification deferred)
- `pending` — grey background (generated, not yet verified)
- `tool` — blue background (agentic-testing tool rather than a user-facing surface)

## Pass-over-pass update protocol

Each audit pass that flips a story to `[x]` or adds a new story must:

1. Either update the node's status suffix in `overview.md` (if the node already exists) OR add a new node under the right subgraph.
2. If the pass touched a surface that has a drill-down diagram, add/update the corresponding node in that drill-down too.
3. If the pass introduces a new surface or subview not yet represented, either add it to `overview.md` (for surface-level concepts) or create a new drill-down file (for subview-level concepts).

Scope reminder (`audits/afk/scope.md` § "Verification gate"): diagrams are documentation, not a gate — a missing node is a follow-up, not a pass blocker.

## Why this exists

Story coverage grew past 40 entries during Run 2. Readers (including the operator returning after AFK) need a visual summary. Mermaid renders in GitHub's markdown viewer, in VS Code's preview, and in local editors — no build step, no toolchain, no link rot.

## Linking to log entries

Diagrams do not embed commit shas or timestamps — those belong in `log.md`. A node's story name is the key; readers grep `log.md` for the story slug to read the full pass receipt.

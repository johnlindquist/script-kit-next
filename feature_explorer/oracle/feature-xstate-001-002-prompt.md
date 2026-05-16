# Oracle Prompt: Feature XState 001-002

Produce implementation-ready XState definitions for Script Kit GPUI feature-map chapters 001-002: Main Menu and File Search.

The previous broader `feature-xstate-001-005` Oracle run failed with a ChatGPT display error, so keep this response compact enough to render. Prefer high-signal machine definitions over prose.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write all repo files locally.

Deliver:

1. A compact TypeScript data schema for authored feature machines.
2. One XState-compatible machine object for `001-main-menu`.
3. One XState-compatible machine object for `002-file-search`.
4. Wireframe metadata per state: visible regions, selected row/item, input text, active popup/portal, footer owner, loading/empty/error state, and proof/receipt.
5. Event names and transition targets that are stable enough to check into `feature_explorer`.
6. A short integration plan for how to load these authored machines beside the current derived fallback.

Avoid long explanations. Use fenced `ts` code blocks for the schema and data.

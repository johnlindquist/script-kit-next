[launcher-trigger-edge-cases]


- The feature map is a human-and-agent atlas. Each chapter must explain capabilities, entry points, interactions, APIs, state machines, receipt paths, and open risks clearly enough for humans and AI agents to operate the product.
- Repo process requires `removed-docs/` updates for changed behavior/docs and `source checks` verification. For this atlas loop, every Oracle session's full output is preserved under `feature-map/raw-oracle/<feature-id>/` and distilled into `feature-map/features/<feature-id>.md`.



- The narrow `~`, `/`, `@`, `>`, and `?` ScriptList first-character handoffs from feature 013.
- Menu syntax and capture/power syntax boundaries from feature 042.
- Stale decoration/highlight risks when transitioning between menu syntax/source-filter decorated text and special-entry handoffs.
- Detached versus embedded ACP slash/mention picker behavior after `/` and `@`.
- The `?` actions-help disabled/no-op state and shared actions popup boundary.


- Feature 013 already maps the basic first-character handoffs, but it leaves edge cases around stale decorations and detached ACP picker behavior.
- Feature 042 maps power/capture syntax and sibling command/refine boundaries.




Return a dense, operator-grade atlas chapter outline for local agents to distill into `feature-map/features/045-launcher-trigger-edge-cases.md`.


2. A precedence model explaining which parser/classifier sees each input and why.
3. How special-entry handoffs differ from source filters, menu syntax, capture syntax, ordinary search, scriptlet keyword triggers, and ACP composer triggers.
4. Embedded and detached ACP behavior for `/` and `@`, including picker staging, deferred opening, focus, return origin, and open gaps.
5. File Search Mini behavior for `~` and `~/...`, including query normalization and what is not accepted.
6. Quick Terminal behavior for bare `>` and what terminal-like text does not trigger.
7. `?` actions-help behavior, including `has_actions()` false/no-op state.
8. Visual/focus/decoration states, especially stale decoration risks.
9. Automation receipts and exact verification recipes.
10. Unsafe claims to avoid and implementation/test plan for gaps.


Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.

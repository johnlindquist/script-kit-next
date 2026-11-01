# Script Kit GPUI Docs

Script Kit GPUI is one window you tell what should happen: a launcher, scripting runtime, Agent Chat host, and local automation/MCP surface, all under one input that never moves. For the product's design rules, read [PRODUCT.md](../PRODUCT.md); for every named surface and its owner, [GLOSSARY.md](../GLOSSARY.md).

## Start Here

- [Getting Started](./guides/getting-started.md) — build the app, create your first script, and learn the core workflow.
- [Feature Tour](./guides/feature-tour.md) — a guided map of the built-ins and AI features people usually miss.
- [Main Menu Input](./guides/main-menu-input.md) — saying what should happen, the keys, and the symbol grammar: context mentions, commands, capture, and filters.
- [SDK Scripting](./guides/sdk-scripting.md) — prompt APIs, metadata, Bun packages, MCP clients, automation receipts, and script templates.
- [MCP and Agent Context](./guides/mcp-and-agent-context.md) — use Script Kit as an MCP server and connect scripts/Agent Chat to external MCP servers.
- [Dictation](./guides/dictation.md) — dictate at the cursor, switch destinations, and recover a cancelled take.
- [External agents](./guides/external-agents.md) — grant access to quiet notes, reviewed commands, or named handoffs.
- [Jev readings](./guides/jev-intent.md) — opt into remote readings and see what leaves the machine.
- [Fill a Save-sheet filename](./guides/desktop-filename.md) — preview a filename without saving the document.
- [Computer Use](./guides/computer-use.md) — list and capture native macOS windows from scripts without sending input or stealing focus.

## Additional References

- [Codex MCP Setup](./codex-mcp-setup.md) — register the live app's MCP server with Codex.
- [Flow UX Protocol](./ai/flow-ux-protocol.md) — the contract between the mdflow CLI and the flow launcher surfaces.
- [ADRs](./adr/) — accepted decisions, including the [original gesture decision](./adr/0002-one-key-gesture-grammar.md) (superseded by the [current shortcuts](./specs/gesture-grammar.md)), [markdown memory substrate](./adr/0003-markdown-files-as-memory-substrate.md), and [clipboard sediment](./adr/0004-clipboard-sediment.md).
- [Specs](./specs/) — designs: [gesture grammar](./specs/gesture-grammar.md), [backgrounded AI sessions](./specs/backgrounded-ai-sessions.md), [one conversation experience](./specs/one-conversation-experience.md), and [QMD migration](./specs/qmd-migration.md).

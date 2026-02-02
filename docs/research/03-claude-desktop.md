# Claude Desktop App - Feature and UX Research (Feb 1, 2026)

## Key features (Desktop app)
- Native desktop apps for macOS and Windows, with the desktop app positioned as always available from the dock and offering quick entry for instant access.
- Quick entry on macOS: access Claude from anywhere without switching windows; includes screenshot capture, window sharing (macOS only), and voice dictation.
- File and context sharing: drag files into chats; share screenshots and application windows for visual context (macOS only).
- Voice input: voice dictation in the desktop app on macOS.
- Connectors and desktop extensions: connectors work across web/desktop/mobile; desktop extensions are locally installed packages that connect Claude to local tools and files, with curated directory and enterprise-grade security controls.
- Cross-device sync: conversations, projects, memory, and preferences sync across desktop, web, and mobile when signed in.
- Enterprise deployment: desktop app supports standard enterprise deployment workflows (MSIX/PKG), SSO for managed devices, and pre-approval of extensions.
- Optional: Cowork (research preview) brings agentic, multi-step task execution to the macOS desktop app for paid plans.

## Projects
- Projects are self-contained workspaces with their own chat histories and knowledge bases, designed for focused work within a specific context.
- Users can upload documents, text, code, or files to a project knowledge base, and add project instructions to steer tone or role behavior.
- Free users can create up to five projects; paid plans unlock enhanced project knowledge capacity using RAG that scales content by up to 10x when needed.
- Team and Enterprise plans add project sharing and collaboration, with permission levels and organization-wide visibility options.

## Artifacts
- Artifacts are substantial, standalone content pieces (often 15+ lines) that Claude surfaces in a dedicated window separate from the chat, intended for content you will edit, reuse, or reference later.
- Common artifact types include documents, code snippets, HTML, SVG, diagrams, flowcharts, and interactive React components.
- Artifacts are accessible via a dedicated artifacts space in the sidebar; users can browse, organize, and create new artifacts from existing ones.
- Artifact workflow supports iteration and versioning, with a right-side artifact window that updates as Claude modifies content.
- Users can view underlying code, copy to clipboard, or download artifacts; multiple artifacts can be managed within a single conversation.
- Artifacts support AI-powered experiences, MCP integrations (for tools like calendars or task apps), and persistent storage (for published artifacts on paid plans), with per-artifact permission prompts for tool access.

## Keyboard shortcuts (desktop)
- Quick entry (macOS): default shortcut is double-tap Option to open a quick input overlay from any app.
- Quick entry can be customized in Settings > General > Desktop App to use double-tap Option, Option+Space, or a custom shortcut.
- Voice dictation shortcut (macOS): optional Caps Lock shortcut for dictation (enabled separately).
- Quick entry requires Claude Desktop running in the background and macOS permissions (Screen Recording, Accessibility; Speech Recognition for voice dictation).
- Quick entry is currently macOS-only; Windows desktop app does not include quick entry features.

## Suggestions for our AI chat window
1) Add a fast, global “quick entry” overlay (macOS-style) with configurable shortcuts, optimized for single-shot questions without context switching.
2) Provide one-click visual context capture (screenshot + window share) and drag-and-drop file attachments inside the quick entry overlay.
3) Build a first-class “Projects” view with explicit project instructions + knowledge base management (file uploads, summaries, pinned references).
4) Create a side-by-side “Artifacts” panel for long-form outputs (docs, code, UI), with copy/download, versioning, and multi-artifact switching.
5) Add a sidebar “Artifacts library” to browse and reuse artifacts across chats and projects.
6) Introduce desktop extensions/connectors with clear permission prompts and a curated directory, aligned with enterprise security controls.
7) Offer voice dictation in the chat window (with clear system permission onboarding), tuned for quick brainstorming.

## Sources
- https://claude.com/download
- https://support.claude.com/en/articles/12626668-use-quick-entry-with-claude-desktop-on-mac
- https://support.claude.com/en/articles/10065433-installing-claude-for-desktop
- https://support.claude.com/en/articles/9517075-what-are-projects
- https://support.claude.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them

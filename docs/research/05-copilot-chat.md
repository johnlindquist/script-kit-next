# GitHub Copilot Chat (Research)

## Scope
This document summarizes Copilot Chat features (slash commands, context references, inline suggestions) and turns them into design ideas for the Script Kit AI chat window. Command availability varies by IDE and version, so users should rely on the IDE’s discovery UI and documentation for the definitive list.

## Slash commands
Copilot Chat supports slash commands to quickly express intent for common tasks. The most common commands (in VS Code) are:

- `/clear` — start a new chat session
- `/explain` — explain code in the active editor
- `/fix` — propose fixes for selected code
- `/fixTestFailure` — find and fix a failing test
- `/help` — quick reference/help
- `/new` — create a new project
- `/tests` — generate unit tests for selected code

Other IDEs expose different commands (for example `/doc`, `/optimize`, `/simplify`). Commands are environment‑dependent; users are expected to type `/` to discover what’s available.

## Context references
### Implicit context
VS Code automatically includes context based on activity, such as the current selection, active file name, and (in Ask/Edit) the active file. In agent flows, the agent decides whether to add the active file.

### Chat variables (explicit `#` context)
Common chat variables include:

- `#block`, `#class`, `#comment`, `#file`, `#function`, `#line`, `#path`, `#project`, `#selection`, `#sym`

These variables inject specific context into the prompt (e.g., current file, selection, symbol, etc.).

### #-mentions and tools
VS Code’s chat context picker lets users add files, folders, and symbols by typing `#` or by selecting items from a context picker. It also supports tool‑backed context additions:

- `#codebase` — perform codebase search to find relevant files
- `#fetch <url>` — pull content from a web page into context (with confirmation)
- `#githubRepo <owner/repo>` — search within a GitHub repository

### @‑mentions (chat participants)
Copilot supports specialized participants that add domain‑specific context:

- `@workspace` — workspace code context
- `@terminal` — terminal context
- `@vscode` — VS Code commands/features
- `@github` — GitHub‑specific skills
- `@azure` — Azure context (preview)

### File references / line ranges
Copilot Chat can reference specific files or line ranges using `#filename` or `#filename:line‑line`, and can also refer to `#solution` (IDE‑specific).

## Inline suggestions & inline chat
### Inline suggestions
Copilot provides inline suggestions as dimmed ghost text in the editor and supports:

- Ghost‑text completions at the cursor
- Next‑edit suggestions (predicts where/what to edit next)
- Partial acceptance (accept next word/line)
- Alternative suggestions (choose among variants)
- Suggestions are informed by existing code and style

### Inline chat
Inline chat lets users prompt Copilot directly in the editor or terminal:

- Editor inline chat scopes prompts to the active editor (optionally a selection)
- Suggestions are applied as inline diffs with accept/reject
- Terminal inline chat provides help with shell commands

## Suggestions for Script Kit’s AI chat window
Based on Copilot’s patterns, these are practical UX features to implement or consider:

1. **Slash command palette**
   - Support `/` commands with autocomplete and descriptions.
   - Provide a default set (e.g., `/clear`, `/explain`, `/fix`, `/tests`, `/doc`, `/new`).
   - Let extensions register new slash commands.

2. **Context chips + “Used references”**
   - Show implicit context (active file, selection, recent output) as removable chips.
   - Display which context items were actually used in the response.

3. **`#` context picker**
   - Offer `#file`, `#selection`, `#script`, `#config`, `#logs`, `#terminal`, `#errors`.
   - Add `#codebase` search for workspace‑wide retrieval.
   - Add `#fetch` with explicit confirmation and caching behavior.

4. **`@` participants**
   - Provide built‑ins like `@workspace`, `@terminal`, `@docs`, `@github`.
   - Allow extensions to register their own participants (mirrors VS Code’s extensibility model).

5. **Inline chat for editor/terminal**
   - In editor prompts, show inline diffs with accept/reject.
   - In terminal prompts, offer “Insert” vs “Run” actions.

6. **Inline suggestions for scripts**
   - Ghost‑text completions + partial acceptance.
   - Optional “next edit” predictions for multi‑file edits in the SDK.

7. **Chat modes**
   - Add explicit modes: Ask / Edit / Agent / Plan.
   - UI toggle + short explanation of behavior/permissions.

8. **Quick chat**
   - Dedicated hotkey to open a small, transient chat box for fast questions.

9. **Context budget & privacy**
   - Show context size and warn when a file is partially included.
   - Require confirmation for external URL fetches or networked tools.

10. **Discoverability**
    - Typing `/`, `@`, or `#` should surface a picker to reduce learning cost.

## Sources
- GitHub Docs — Asking GitHub Copilot questions in your IDE
- GitHub Docs — GitHub Copilot Chat cheat sheet
- VS Code Docs — Manage context for AI
- VS Code Docs — Inline chat
- VS Code Docs — Inline suggestions from GitHub Copilot in VS Code

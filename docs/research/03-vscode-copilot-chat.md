# VS Code GitHub Copilot Chat Features and UX

This document summarizes GitHub Copilot Chat features and UX in VS Code, focusing on inline chat, chat panel, slash commands, context mentions, and keyboard shortcuts.

## Overview

GitHub Copilot Chat in VS Code provides AI-powered coding assistance through multiple interaction paradigms: inline chat for context-aware editing, a persistent chat panel for ongoing conversations, and a quick chat for lightweight queries. The system uses a sophisticated context model with `#`-prefixed context variables and `@`-prefixed chat participants.

---

## Inline Chat

### Editor Inline Chat

Inline chat lets you generate or edit code directly in the editor without switching to the Chat view. Key characteristics:

- **Keyboard Shortcut**: `Cmd+I` (macOS) / `Ctrl+I` (Windows/Linux)
- **Alternative Access**: Menu > Chat > Open Inline Chat
- **Scoping**: Prompts are scoped to the active editor (though other workspace files may be used for context)
- **Selection Support**: You can scope to a selected block of code
- **Diff Preview**: VS Code shows an inline diff that you can accept or reject
- **Follow-ups**: Optional follow-up prompts available after initial response

### Terminal Inline Chat

Inline chat also works in the integrated terminal:

- **Open Terminal**: View > Terminal or `` Ctrl+` ``
- **Start Inline Chat**: `Cmd+I` (macOS) / `Ctrl+I` (Windows/Linux)
- **Run Command**: `Cmd+Enter` (macOS) / `Ctrl+Enter` (Windows/Linux) - executes the generated command
- **Insert Command**: `Option+Enter` (macOS) / `Alt+Enter` (Windows/Linux) - inserts for editing before running

---

## Chat Panel (Chat View)

### Primary Entry Points

| Entry Point | Description |
|-------------|-------------|
| **Chat View** | Ongoing conversation in the Secondary Side Bar |
| **Quick Chat** | Lightweight prompt UI opened without leaving current task |

### Chat Modes

The Chat view supports four distinct modes, switched using the agents dropdown at the bottom:

| Mode | Description |
|------|-------------|
| **Ask** | Optimized for Q&A about your codebase and general coding concepts |
| **Edit** | Designed for controlled multi-file edits; you choose a working set and decide whether to accept changes after each turn |
| **Agent** | Autonomous flow where Copilot chooses files, proposes edits and terminal commands, and iterates to complete the task |
| **Plan** | Preview mode that produces a detailed plan before any code changes and requires approval to proceed |

---

## Slash Commands

Slash commands are shorthand actions in chat for common tasks like explaining, fixing, or generating tests.

| Command | Purpose |
|---------|---------|
| `/doc` | Generate documentation comments from editor inline chat |
| `/explain` | Explain a code block, file, or concept |
| `/fix` | Fix a code block or resolve compiler/lint errors |
| `/tests` | Generate tests for selected or all methods/functions |
| `/setupTests` | Recommend and set up a testing framework (with VS Code test tooling suggestions) |
| `/clear` | Start a new chat session in the Chat view |
| `/new` | Scaffold a new workspace or file |
| `/newNotebook` | Scaffold a new Jupyter notebook |
| `/search` | Generate a Search view query |
| `/startDebugging` | Generate a `launch.json` and start a debugging session |
| `/<prompt name>` | Run a reusable prompt by name |
| `/fixTestFailure` | Suggest fixes for failing tests |

---

## Context Mentions and Context Handling

### Implicit Context

- Selected text and the active file name are automatically included
- Ask/Edit modes auto-include the active file
- Agent mode decides whether to include the active file based on the prompt

### #-Mentions (Explicit Context)

Type `#` to add context items:

| Context Variable | Purpose |
|------------------|---------|
| `#file` | Reference a specific workspace file |
| `#editor` | Explicitly include the visible code in the active editor |
| `#selection` | Focus on selected code |
| `#folder` | Include a folder's contents |
| `#symbol` | Reference a specific symbol |
| `#terminal` | Include terminal output |
| `#changes` | Include source control changes |

**Additional Methods**:
- "Add Context" button opens a picker
- Drag-and-drop files or folders into chat

### @-Mentions (Participants)

`@` invokes chat participants (specialized agents/tools):

| Participant | Purpose |
|-------------|---------|
| `@workspace` | Queries across the entire workspace |
| `@terminal` | Terminal-related assistance |
| `@vscode` | VS Code editor assistance |
| *Extensions* | Custom participants from installed extensions |

### Important Distinction

- **`@` prefix**: Invokes chat participants (like `@workspace`, `@terminal`, `@vscode`)
- **`#` prefix**: Adds context items/variables (like `#file`, `#editor`, `#selection`)

### File Size Behavior

- If a file is too large, VS Code includes an outline of functions and descriptions
- If the outline is still too large, the file is omitted entirely

---

## Keyboard Shortcuts (Default)

### Chat Controls

| Action | macOS | Windows/Linux | Notes |
|--------|-------|---------------|-------|
| Open Chat view | `Ctrl+Cmd+I` | `Ctrl+Alt+I` | Opens Chat view in Secondary Side Bar |
| Open Quick Chat | `Shift+Option+Cmd+L` | `Ctrl+Shift+Alt+L` | Quick prompt UI without context switch |
| New chat session | `Cmd+N` | `Ctrl+N` | Starts new session in Chat view |
| Switch to agents | `Shift+Cmd+I` | `Ctrl+Shift+I` (Win) / `Ctrl+Shift+Alt+I` (Linux) | Switches agents in Chat view |
| Model picker | `Option+Cmd+.` | `Ctrl+Alt+.` | Choose a different chat model |

### Inline Chat

| Action | macOS | Windows/Linux | Notes |
|--------|-------|---------------|-------|
| Start inline chat (editor/terminal) | `Cmd+I` | `Ctrl+I` | Context-sensitive to editor/terminal |
| Voice prompt in Chat view | `Cmd+I` | `Ctrl+I` | Starts voice prompt in Chat view |
| Inline voice chat (hold) | Hold `Cmd+I` | Hold `Ctrl+I` | Starts inline voice chat |

### Terminal Inline Chat

| Action | macOS | Windows/Linux | Notes |
|--------|-------|---------------|-------|
| Run terminal inline command | `Cmd+Enter` | `Ctrl+Enter` | Execute generated command |
| Insert terminal inline command | `Option+Enter` | `Alt+Enter` | Insert for editing before running |

### Inline Suggestions

| Action | macOS | Windows/Linux | Notes |
|--------|-------|---------------|-------|
| Accept inline suggestion | `Tab` | `Tab` | Inline suggestions / next edit suggestion |
| Dismiss inline suggestion | `Escape` | `Escape` | Dismiss inline suggestion |

---

## UX Design Patterns

### Key Design Decisions

1. **Dual Chat Paradigm**: Inline chat for quick, contextual edits vs. Chat Panel for extended conversations
2. **Mode-Based Interaction**: Different modes (Ask, Edit, Agent, Plan) for different use cases
3. **Explicit Context Control**: Users can precisely control what context is included via `#`-mentions
4. **Participant System**: `@`-mentions enable specialized agents for different domains
5. **Diff Preview**: All code changes shown as reviewable diffs before acceptance
6. **Progressive Disclosure**: Quick Chat for simple queries, full Chat View for complex tasks

### Visual Indicators

- Inline diff highlighting for proposed changes
- Accept/Reject buttons for code suggestions
- Context chips showing referenced files/symbols
- Mode indicator in Chat View

---

## Sources

- VS Code Documentation: [Inline Chat](https://code.visualstudio.com/docs/copilot/chat/inline-chat)
- VS Code Documentation: [Copilot Chat Context](https://code.visualstudio.com/docs/copilot/chat/copilot-chat-context)
- VS Code Documentation: [Copilot VS Code Features](https://code.visualstudio.com/docs/copilot/copilot-vscode-features)
- GitHub Documentation: [Copilot Chat in IDE](https://docs.github.com/copilot/github-copilot-chat/using-github-copilot-chat-in-your-ide)
- GitHub Changelog: [VS Code Copilot Chat January 2024](https://github.blog/changelog/2024-02-12-vs-code-copilot-chat-january-2024-version-0-12)

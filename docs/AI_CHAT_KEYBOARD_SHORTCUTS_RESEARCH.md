# AI Chat Interface Keyboard Shortcuts & Quick Actions Research

This document captures keyboard shortcut patterns and quick action designs from major AI chat interfaces, intended to inform Script Kit's AI chat UX design.

---

## Table of Contents

1. [Global Access & Activation](#global-access--activation)
2. [Chat Input & Submission](#chat-input--submission)
3. [Message Actions](#message-actions)
4. [Navigation & History](#navigation--history)
5. [Slash Commands](#slash-commands)
6. [@ Mentions & Context](#-mentions--context)
7. [Suggestion Handling](#suggestion-handling)
8. [Quick Actions & Context Menus](#quick-actions--context-menus)
9. [Mode Switching](#mode-switching)
10. [Design Patterns Summary](#design-patterns-summary)
11. [Recommended Shortcuts for Script Kit](#recommended-shortcuts-for-script-kit)

---

## Global Access & Activation

### Raycast
| Action | Shortcut |
|--------|----------|
| Open Raycast | `Cmd + Space` (customizable) |
| Open AI Chat directly | `Option + J` (customizable) |
| Quick AI (one-off questions) | `Cmd + Shift + Tab` |

### Claude Desktop (Mac)
| Action | Shortcut |
|--------|----------|
| Quick Entry overlay | Double-tap `Option` |
| Alternative quick access | `Option + Space` (customizable) |
| Voice dictation | `Caps Lock` (hold to dictate) |

### Perplexity
| Action | Shortcut |
|--------|----------|
| Activate search bar | `Cmd + Shift + P` |
| Voice mode | `Cmd + Shift + M` |
| Voice dictation | `Cmd + Shift + D` |
| Upload file | `Cmd + Shift + U` |
| Screen capture | `Cmd + Shift + 0` |

### Warp Terminal
| Action | Shortcut |
|--------|----------|
| Toggle Agent Mode | `Ctrl + I` or `Cmd + I` |
| Alternative toggle | `* + Space` (asterisk-space) |

### Arc Browser
| Action | Shortcut |
|--------|----------|
| Open Command Bar | `Cmd + T` (Mac) / `Ctrl + L` (Windows) |

---

## Chat Input & Submission

### Common Patterns
| Action | Primary | Alternative |
|--------|---------|-------------|
| Submit message | `Enter` | `Cmd + Enter` |
| New line without sending | `Shift + Enter` | - |
| Focus input box | `Shift + Esc` | `/` |
| Clear input | `Cmd + K` | `Escape` |

### ChatGPT
| Action | Shortcut |
|--------|----------|
| Submit prompt | `Cmd + Enter` |
| New chat | `Cmd + Shift + N` |
| New tab | `Cmd + Shift + T` |
| Close window | `Cmd + W` |
| Toggle dark mode | `Cmd + D` |
| Show all shortcuts | `Cmd + /` or `Ctrl + /` |

### Cursor IDE
| Action | Shortcut |
|--------|----------|
| Open AI Chat | `Cmd + L` / `Ctrl + L` |
| Chat with selection | `Cmd + Shift + L` |
| Inline edit (Cmd K) | `Cmd + K` / `Ctrl + K` |
| Open Composer | `Cmd + I` / `Ctrl + I` |
| Open Agent Chat | `Cmd + Shift + A` |

---

## Message Actions

### Copy & Export
| Action | ChatGPT | SillyTavern | General |
|--------|---------|-------------|---------|
| Copy last response | `Ctrl + Shift + C` | - | - |
| Copy code block | `Ctrl + Shift + ;` | - | - |
| Copy selected | `Cmd + C` | `Cmd + C` | `Cmd + C` |

### Edit & Regenerate
| Action | ChatGPT | SillyTavern | Cassidy |
|--------|---------|-------------|---------|
| Edit last user message | - | `Ctrl + Up` | Click + `Enter` |
| Regenerate response | - | `Ctrl + Enter` | - |
| Continue response | - | `Alt + Enter` | - |
| Stop generation | `Ctrl + Backspace` | `Escape` | - |

### GitHub Copilot (VS Code)
| Action | Mac | Windows/Linux |
|--------|-----|---------------|
| Accept suggestion | `Tab` | `Tab` |
| Accept next word | `Cmd + Right` | `Ctrl + Right` |
| Dismiss suggestion | `Escape` | `Escape` |
| Cycle forward | `Option + ]` | `Alt + ]` |
| Cycle backward | `Option + [` | `Alt + [` |
| View all in panel | `Ctrl + Enter` | `Ctrl + Enter` |

---

## Navigation & History

### Chat History
| Action | Shortcut | Application |
|--------|----------|-------------|
| Previous message | `Up Arrow` | Warp, Claude Code |
| Next message | `Down Arrow` | Warp, Claude Code |
| Previous prompt (cycle) | `Ctrl + Up` | ChatGPT Extension |
| Next prompt (cycle) | `Ctrl + Down` | ChatGPT Extension |
| Toggle sidebar | `Cmd + Shift + S` | ChatGPT |
| Search chats | `Cmd + F` | General |

### Claude Code (Terminal)
| Action | Shortcut |
|--------|----------|
| Stop action | `Escape` |
| Jump to previous messages | `Escape` twice |
| Scroll through past commands | `Up Arrow` |
| Paste images | `Ctrl + V` |

---

## Slash Commands

### Design Principles
- Prefix: Always `/` followed by keyword
- Discovery: Menu opens when typing `/`
- Filtering: Menu filters as user types
- Selection: `Enter` or `Tab` to select
- Arguments: Space after command for parameters

### Common Slash Commands
| Command | Purpose | Applications |
|---------|---------|--------------|
| `/help` | Show available commands | Universal |
| `/new` | Start new conversation | ChatGPT, Claude |
| `/clear` | Clear conversation | Various |
| `/model` | Switch AI model | Claude Code, ChatGPT |
| `/settings` | Open settings | Various |
| `/export` | Export conversation | Various |

### Aider (Coding Assistant)
| Command | Purpose |
|---------|---------|
| `/add` | Add files to context |
| `/drop` | Remove files from context |
| `/commit` | Commit changes |
| `/undo` | Undo last change |
| `/diff` | Show diff |

### AnythingLLM
- Slash commands act as text snippet shortcuts
- Custom commands can be defined by users

---

## @ Mentions & Context

### Cursor IDE
Type `@` in AI input to trigger context menu:
- `@filename` - Reference specific file
- `@folder` - Reference folder
- `#filename` - Alternative syntax
- Arrow keys to navigate, `Enter` to select
- `Cmd + P` to select multiple files

### Claude Code
- `@` triggers file/symbol picker
- Supports fuzzy matching
- Auto-completes paths

### General Pattern
| Trigger | Purpose |
|---------|---------|
| `@` | Reference files, people, or context |
| `#` | Reference topics or tags |
| `!` | Execute actions (some systems) |

---

## Suggestion Handling

### Tab Completion Pattern (Cursor, Copilot)
| Action | Shortcut |
|--------|----------|
| Accept full suggestion | `Tab` |
| Accept next word | `Cmd/Ctrl + Right` |
| Accept next line | - |
| Reject | `Escape` |
| See alternatives | `Option/Alt + ]` / `[` |

### Inline AI Actions
- Ghost text appears as suggestion
- Visual distinction (gray/dimmed text)
- Clear accept/reject affordances

---

## Quick Actions & Context Menus

### Right-Click / Selection Actions
| Action | JetBrains | Capacities | General |
|--------|-----------|------------|---------|
| Open AI actions | `Alt + Enter` | Click sparkle | Right-click |
| Summarize | - | Quick action | - |
| Translate | - | Quick action | - |
| Explain | Context menu | - | - |
| Improve writing | - | Quick action | - |
| Generate tests | Context menu | - | - |
| Refactor | Context menu | - | - |

### QuickAssist Pattern (Browser Extension)
- `Ctrl + Q` (Win) / `Cmd + E` (Mac) to open
- Right-click context menu integration
- Selection triggers floating menu
- Custom prompts from options page

### Codeanywhere Pattern
- Highlight code section
- Right-click for AI actions
- Shortcuts for common tasks (review, comment, test, refactor)
- `Cmd/Ctrl + L` (VS Code) or `Cmd/Ctrl + J` (JetBrains) for chat

---

## Mode Switching

### Warp Terminal
| Mode | Activation |
|------|------------|
| Terminal Mode | Click terminal icon |
| Agent Mode | `Ctrl + I` or `* + Space` |
| Natural language detection | Automatic |

### Cursor IDE
| Mode | Shortcut |
|------|----------|
| Regular chat | `Cmd + L` |
| Inline edit | `Cmd + K` |
| Composer (multi-file) | `Cmd + I` |
| Agent | `Cmd + Shift + A` |

---

## Design Patterns Summary

### 1. Command Palette / Natural Language Bar
- Single entry point for all actions
- Fuzzy search across commands
- Combines navigation + AI interaction
- Examples: Raycast, Arc, Cursor

### 2. Contextual AI Actions
- Right-click menus with AI options
- Selection-triggered floating menus
- Inline suggestions with tab completion
- Examples: VS Code Copilot, JetBrains AI

### 3. Modal Interaction Layers
| Level | Description | Example |
|-------|-------------|---------|
| Quick | One-off questions | Raycast Quick AI |
| Chat | Ongoing conversation | Dedicated chat panel |
| Inline | Edit in place | Cursor Cmd+K |
| Agent | Autonomous multi-step | Cursor Agent |

### 4. Minimal Chat Approach
- Reduce typing through context awareness
- Click to add context instead of describing
- Progressive disclosure of complexity
- Quick actions for common operations

### 5. Keyboard-First Design Principles
1. **Discoverability**: `Cmd + /` or `?` shows all shortcuts
2. **Consistency**: Same modifiers for similar actions
3. **Escape hatches**: `Escape` always exits/cancels
4. **Progressive complexity**: Simple actions = simple shortcuts
5. **Customization**: Power users can remap shortcuts

---

## Recommended Shortcuts for Script Kit

Based on patterns from Raycast, Cursor, and other power-user tools:

### Primary Actions
| Action | Recommended Shortcut | Notes |
|--------|---------------------|-------|
| Open AI chat | `Cmd + J` | Matches Capacities, avoids conflicts |
| Submit message | `Enter` | Standard |
| New line | `Shift + Enter` | Universal |
| Cancel/Close | `Escape` | Universal |
| Show shortcuts | `Cmd + /` | Universal |

### Message Operations
| Action | Recommended Shortcut | Notes |
|--------|---------------------|-------|
| Copy response | `Cmd + Shift + C` | ChatGPT pattern |
| Regenerate | `Cmd + R` | Intuitive |
| Edit last message | `Cmd + Up` | SillyTavern pattern |
| Stop generation | `Escape` or `Cmd + .` | Standard interrupt |

### Context & Actions
| Action | Recommended Shortcut | Notes |
|--------|---------------------|-------|
| Add file context | `@` + filename | Cursor pattern |
| Slash commands | `/` | Universal |
| Quick actions menu | `Cmd + K` | Command palette |
| Inline edit | `Cmd + I` | Cursor pattern |

### Navigation
| Action | Recommended Shortcut | Notes |
|--------|---------------------|-------|
| Previous message | `Up Arrow` | When input empty |
| Search history | `Cmd + F` | Standard |
| Clear chat | `Cmd + Shift + K` | Avoids conflicts |
| Focus input | `Tab` or `/` | Quick refocus |

### Mode Switching
| Action | Recommended Shortcut | Notes |
|--------|---------------------|-------|
| Toggle agent mode | `Cmd + Shift + A` | Cursor pattern |
| Quick question | `Cmd + Shift + Space` | One-off queries |

---

## Sources

### ChatGPT
- [Top 10 ChatGPT Keyboard Tips for 2026](https://www.clevertype.co/post/top-10-chatgpt-keyboard-tips-for-2025)
- [ChatGPT Keyboard Shortcuts: Windows & Mac (2025)](https://guides.ai/chatgpt-keyboard-shortcuts/)
- [ChatGPT Custom Shortcuts Pro - Chrome Extension](https://chromewebstore.google.com/detail/chatgpt-custom-shortcuts/figoaoelbmlhipinligdgmopdakcdkcf)

### Raycast
- [Raycast Keyboard Shortcuts](https://manual.raycast.com/keyboard-shortcuts)
- [Raycast AI](https://manual.raycast.com/ai)
- [Command Aliases and Hotkeys](https://manual.raycast.com/command-aliases-and-hotkeys)

### Claude
- [Claude Desktop Quick Entry](https://support.claude.com/en/articles/12626668-use-quick-entry-with-claude-desktop-on-mac)
- [Claude Code Keybindings](https://code.claude.com/docs/en/keybindings)
- [Claude UI Shortcuts Extension](https://github.com/A-PachecoT/claude_ui_shortcuts)

### Cursor IDE
- [Cursor Keyboard Shortcuts Cheat Sheet](https://dotcursorrules.com/cheat-sheet)
- [All Cursor Shortcuts Guide](https://refined.so/blog/cursor-shortcuts-guide)
- [Cursor Keyboard Shortcuts Docs](https://docs.cursor.com/kbd)

### Perplexity
- [Perplexity Windows App Announcement](https://x.com/perplexity_ai/status/1899498357154107499)
- [Perplexity Desktop Keyboard Shortcuts](https://deepwiki.com/inulute/perplexity-ai-app/4.3-ui-enhancement-scripts)

### GitHub Copilot
- [GitHub Copilot in VS Code Cheat Sheet](https://code.visualstudio.com/docs/copilot/reference/copilot-vscode-features)
- [Inline Suggestions from Copilot](https://code.visualstudio.com/docs/copilot/ai-powered-suggestions)

### Warp Terminal
- [Warp Keyboard Shortcuts](https://docs.warp.dev/getting-started/keyboard-shortcuts)
- [Warp Universal Input](https://docs.warp.dev/terminal/universal-input)

### Arc Browser
- [Arc Keyboard Shortcuts](https://resources.arc.net/hc/en-us/articles/20595231349911-Keyboard-Shortcuts)
- [Arc Command Bar Actions](https://start.arc.net/command-bar-actions)

### JetBrains AI
- [AI Keyboard Shortcuts](https://www.jetbrains.com/help/ai-assistant/ai-keyboard-shortcuts.html)

### Design Patterns
- [Post-Chat UI - Allen Pike](https://allenpike.com/2025/post-chat-llm-ui)
- [7 Key Design Patterns for AI Interfaces](https://uxplanet.org/7-key-design-patterns-for-ai-interfaces-893ab96988f6)
- [Slash Commands in GitLab Duo Chat](https://design.gitlab.com/patterns/duo-slash-commands/)
- [Interacting with LLMs with Minimal Chat](https://eugeneyan.com/writing/llm-ux/)

---

*Document generated: January 31, 2026*
*For Script Kit GPUI AI Chat interface design reference*

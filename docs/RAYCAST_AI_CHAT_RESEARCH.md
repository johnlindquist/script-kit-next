# Raycast AI Chat Experience Research

## Executive Summary

Raycast has built one of the most polished AI chat experiences in the productivity tool space. Their approach centers on deep OS integration, keyboard-first interaction, and a unified interface for multiple AI models. This document captures key features, UX patterns, and actionable insights for improving an AI chat window.

---

## Core Interaction Methods

Raycast provides three distinct ways to interact with AI, each optimized for different use cases:

### 1. Quick AI (Floating Window)
- **Purpose**: Fast, one-off questions
- **Activation**: Single hotkey (configurable, e.g., `Option + Space`)
- **Behavior**: Lightweight floating window that appears above all apps
- **Key Feature**: Press `Tab` after typing to get instant response
- **Window Options**: "Always on Top" or standard window behavior

### 2. AI Chat (Standalone Window)
- **Purpose**: Extended conversations, research, complex tasks
- **Activation**: Configurable hotkey (e.g., `Option + J`) or search "AI Chat"
- **Behavior**: Full-featured detached window with sidebar
- **Key Feature**: Responds to standard window management (Cmd+Tab accessible)

### 3. AI Commands
- **Purpose**: Repeatable prompt templates for common tasks
- **Examples**: "Improve Writing", "Fix Grammar", "Summarize"
- **Key Feature**: Can paste directly into frontmost app with `Cmd + Enter`

---

## Window Management

### Detached Window Behavior
- AI Chat can be a fully detached macOS window
- Accessible via `Cmd + Tab`
- Supports all standard window management commands
- Choice between "Always on Top" floating or normal window

### Window Controls
- Resizable window
- Window position/size remembered
- Can run alongside other apps without stealing focus
- Floating behavior optional for quick reference use

---

## Sidebar and Navigation

### Chat Sidebar Features
- Left sidebar shows all previous chats
- Chats organized chronologically (Today, Yesterday, etc.)
- Navigate chats with arrow keys
- `Cmd + F` to focus search field
- Search by chat content or title
- `Cmd + P` to search by chat titles only
- `Cmd + Shift + F` to search by content

### Sidebar Toggle
- Button in top-left to show/hide sidebar
- Maximizes chat area when hidden

---

## Chat Branching (Experimental)

### Concept
Create alternate conversation paths from any point in history - like "save points" in a game.

### Implementation
- `Cmd + Shift + B` to create a new branch
- `Cmd + Option + Up Arrow` to navigate to parent chat
- Right-click menu options for branching
- Background execution allows multiple branches to process simultaneously
- Visual indicator showing branch relationships

---

## Keyboard Shortcuts

### Essential Shortcuts
| Action | Shortcut |
|--------|----------|
| Open AI Chat | `Option + J` (configurable) |
| New Chat | `Cmd + N` |
| New Chat with Preset | `Cmd + Shift + N` |
| Branch Chat | `Cmd + Shift + B` |
| Go to Parent Chat | `Cmd + Option + Up Arrow` |
| Focus Search | `Cmd + F` |
| Search by Title | `Cmd + P` |
| Action Menu | `Cmd + K` |
| Close Chat Window | `Cmd + Q` |
| Continue in AI Chat | `Cmd + J` (from Quick AI) |
| Add Attachment | `Cmd + Shift + A` |

### Message Input Options
- `Enter` to send (default)
- Alternative: `Cmd + Enter` to send, `Enter` for new line (configurable)

---

## Model Selection and Switching

### Multi-Model Interface
- 32+ models available in unified interface
- Models from: OpenAI, Anthropic, Perplexity, Groq, Together AI, Mistral, Google, xAI, Replicate

### Model Switching Features
- Change models mid-conversation
- Regenerate any response with a different model via `Ctrl + K` menu
- Model presets save preferred models per task type
- BYOK (Bring Your Own Key) support for: Anthropic, Google, OpenAI, OpenRouter
- Local models via Ollama integration (100+ models)

### Model Display Information
- Speed rating (1-5)
- Intelligence rating (1-5)
- Context window size
- Feature support (web search, image generation, vision, reasoning)
- Subscription requirements

---

## AI Chat Presets

### What Presets Include
- Model selection
- System instructions (custom persona/behavior)
- Creativity level (temperature 0-2)
- Tools/extensions enabled

### Preset Workflow
1. Create preset in chat settings
2. Specify model, instructions, creativity, tools
3. Start new chat with preset via `Cmd + Shift + N`
4. Dropdown in toolbar shows available presets

### Example Preset Patterns
- "TypeScript Expert" - Claude 3.5 with coding instructions
- "Swift Assistant" - DeepSeek R1 for iOS development
- "Project Assistant" - Custom context for specific project
- "Prompt Creator" - Generate optimized prompts

---

## System Instructions

### Customization Options
- Add global custom instructions in Settings
- Per-chat system instructions
- Per-preset system instructions
- Support for persona/role assignment

### Best Practices from Raycast
- Assign AI a role or persona for context-appropriate responses
- Specify target audience (e.g., "write for new parents")
- Define tone, style, terminology expectations
- Cannot @mention AI Extensions in system instructions

---

## Creativity (Temperature) Setting

### Scale
- Range: 0 to 2
- Low (0-0.5): Concrete tasks (grammar fixing, code review)
- Medium (1): Balanced responses
- High (1.5-2): Open-ended questions, brainstorming

### Persistence
- AI Chat remembers last used creativity setting
- Can be saved in presets

---

## Attachments and Context

### Supported Attachments
- PDF documents
- CSV files
- Screenshots / images
- Any visible screen content

### Attachment Workflow
- Click `+` button in composer
- Or press `Cmd + Shift + A`
- Multiple attachments supported
- Mix and match attachment types

### OS Integration
- Deep macOS integration for context capture
- Can reference currently visible screen content
- Clipboard integration

---

## Web Search Integration

### Features
- Toggle web search for real-time information
- Inline references in responses
- Same capability in Quick AI and AI Chat
- Automatic preference remembered across new chats

---

## @Mentions and Extensions

### AI Extensions (MCP Support)
- Type `@` to access AI Extensions inline
- Works in Quick AI, AI Commands, and AI Chat
- MCP servers behave like AI Extensions
- Add extensions via chat settings or presets

### Extension Examples
- Calendar integration
- Linear (issue tracking)
- Custom tools via MCP protocol

---

## Visual Design Elements

### Response Rendering
- Markdown support with syntax highlighting
- Code blocks with copy button (top-right)
- Inline code styling
- Response images auto-resize to fit window

### Streaming and Loading
- Fade-in animation during streaming
- Loading indicator when generating
- "Thinking step" display for reasoning models
- Smooth, snappy animations

### Typography and Layout
- Clean, minimalist interface
- Monospace font for code
- Avatar imagery for chat participants
- Consistent color theming
- Composer at bottom with settings below

---

## Feedback System

### Message Feedback
- Thumbs up/down buttons on messages (appear on hover)
- "Bad Response..." action available
- Requires explicit consent before sharing chat data
- Full thread including tool calls shared when reporting

---

## Action Panel / Command Menu

### Access
- `Cmd + K` to open action menu
- Context-aware actions based on selection

### Common Actions
- Copy response
- Save image
- Regenerate with different model
- Apply AI Command mid-chat (e.g., "Make Shorter")
- Branch conversation
- Navigate to parent

---

## Paste and Clipboard Integration

### Primary Action Options
- Paste to Active App (default)
- Copy to Clipboard

### Direct Paste
- AI Commands can paste directly into frontmost app
- `Cmd + Enter` to replace selected text with AI output
- Clipboard history integration

---

## Performance Optimizations

### Recent Improvements
- Chat and Home rebuilt for performance
- Snappier keyboard animation in composer
- Optimized response streaming
- Background execution for chat branches

---

## Accessibility

### Keyboard-First Design
- Full keyboard navigation throughout
- All actions have keyboard shortcuts
- Consistent with macOS accessibility standards

### Screen Reader
- Decent screen reader compatibility (per reviews)
- Room for improvement in WCAG compliance

---

## Cloud Sync

### Data Storage
- Local storage by default
- Optional Cloud Sync for cross-device access
- Encrypted at rest and in transit
- Syncs AI Chats across all Macs

---

## Actionable Insights for AI Chat Window Implementation

### Priority 1: Foundation
1. **Keyboard-first design** - Every action should have a shortcut
2. **Streaming with visual feedback** - Fade-in animation, loading indicator
3. **Model switching** - Easy to change models mid-conversation
4. **Window flexibility** - Support both floating and standard window modes

### Priority 2: Navigation
5. **Chat sidebar** - Show history, support search by content and title
6. **Quick search** - `Cmd + P` pattern for fuzzy finding chats
7. **Keyboard navigation** - Arrow keys to navigate between chats

### Priority 3: Customization
8. **Presets** - Save model + instructions + creativity combos
9. **System instructions** - Per-chat and global customization
10. **Creativity slider** - Simple 0-2 temperature control

### Priority 4: Advanced Features
11. **Chat branching** - Explore alternate paths without losing context
12. **Attachments** - Support files, images, screen content
13. **@mentions** - Extensibility via tools/extensions
14. **Web search toggle** - Real-time information access

### Priority 5: Polish
15. **Feedback buttons** - Thumbs up/down on responses
16. **Action menu** - `Cmd + K` for contextual actions
17. **Code blocks** - Syntax highlighting with copy button
18. **Title generation** - Auto-generate chat titles

---

## Key UX Principles from Raycast

1. **Minimal friction** - One hotkey to access, Tab to query
2. **Context preservation** - Remember settings, preserve draft text when switching
3. **Progressive disclosure** - Simple by default, power features available
4. **Consistency** - Same patterns across Quick AI, Chat, and Commands
5. **Speed** - Optimize for keyboard, minimize mouse usage
6. **Flexibility** - Multiple models, presets, customization options
7. **Privacy** - Local by default, explicit consent for sharing

---

## Sources

- [Raycast AI Core Features](https://www.raycast.com/core-features/ai)
- [Raycast AI Manual](https://manual.raycast.com/ai)
- [Raycast Changelog - AI Chat 2.0 (v1.69.0)](https://www.raycast.com/changelog/macos/1-69-0)
- [Raycast Changelog - AI Chat Presets (v1.72.0)](https://www.raycast.com/changelog/1-72-0)
- [Raycast Changelog - Smarter AI Chat (v1.67.0)](https://www.raycast.com/changelog/1-67-0)
- [Raycast Changelog - Quick AI Beta (v1.50.0)](https://www.raycast.com/changelog/1-50-0)
- [Raycast Changelog - Chat Branching (v1.101.0)](https://www.raycast.com/changelog/1-101-0)
- [Raycast Developer API - User Interface](https://developers.raycast.com/api-reference/user-interface)
- [Raycast Keyboard Shortcuts](https://manual.raycast.com/keyboard-shortcuts)
- [Raycast AI Extensions](https://manual.raycast.com/ai-extensions)
- [Raycast Blog - One Interface, Many LLMs](https://www.raycast.com/blog/more-ai-models)
- [Raycast Preset Explorer](https://ray.so/presets)
- [Raycast Prompt Explorer](https://ray.so/prompts)

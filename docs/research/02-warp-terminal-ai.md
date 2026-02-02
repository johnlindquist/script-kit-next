# Warp Terminal AI Features & Chat UX (Research)

Date: 2026-02-01

## Sources

- https://docs.warp.dev/features/active-ai
- https://docs.warp.dev/features/universal-input
- https://www.warp.dev/warp-ai
- https://www.warp.dev/ai
- https://docs.warp.dev/features/blocks
- https://docs.warp.dev/features/keyboard-shortcuts
- https://docs.warp.dev/features/warp-drive/notebooks

## Overview: how AI is integrated into the terminal workflow

- **Single input for everything**: Warp’s Universal Input is the unified entry for terminal commands and AI requests, with a local auto-detection step that can switch the input into Agent Mode when the user’s intent is conversational. Users can also explicitly force Agent Mode or Terminal Mode. (Universal Input)
- **Agent Mode as in-terminal chat**: Agent Mode is positioned as an AI assistant inside the terminal for multi-step tasks and explanations, using environment-aware guidance. (Warp AI / Agent Mode)
- **AI is layered, not replacing CLI**: Warp keeps core CLI flows intact and adds AI affordances (prompt suggestions, “next command”, and AI command suggestions) that can be accepted or ignored without blocking. (Active AI, Warp AI)

## Command suggestions & proactive help

### 1) AI Command Suggestions (inline, on demand)

- **Trigger**: typing `#` starts AI Command Suggestions directly in the command line. (Warp AI page)
- **Behavior**: user describes the task in natural language, Warp suggests the command to run. (Warp AI page)

### 2) Active AI Prompt Suggestions (ambient chips)

- **Prompt Suggestions** appear as chips in the input, generated from the **most recent terminal block** and the current input. (Active AI)
- Suggestions are **opt-in**: they do not consume AI request limits until accepted. (Active AI)
- **Accept** with `Cmd+Enter` (macOS) or `Ctrl+Shift+Enter` (Windows/Linux). Accepted prompts launch **Agent Mode**. (Active AI)
- Warp emphasizes **privacy**: prompts are sent to Warp AI with secrets redacted; raw session data is not used to train models. (Active AI)

### 3) Next Command (post-execution continuation)

- After a command runs, Warp can suggest the **next command** based on **command history plus metadata** (current directory, git branch, exit code, etc.) and the current session context. (Active AI)
- **Accept** with `Tab`, `Right Arrow`, or `Ctrl+F`. (Active AI)

### 4) Suggested Code Diffs (error recovery)

- When a command errors, Warp can surface **suggested code diffs** to fix it. (Active AI)
- **Apply** the suggestion with `Cmd+Enter`, **view details** with `Cmd+E`, **cancel** with `Ctrl+C`. (Active AI)

## Context awareness & attachment model

- **Context chips** appear in Universal Input (e.g., active directory, conversation management, git status, node version). (Universal Input)
- The **toolbelt** includes an **@ context menu** to attach **blocks**, **Warp Drive objects**, or **files** as context. (Universal Input)
- **Auto-detection is local** (no user data sent for intent detection) and can be overridden; pressing `Esc` exits auto-detected Agent Mode. (Universal Input)

## Chat / Agent UX patterns

- **Agent Mode is a mode, not a separate app**: it is invoked in the same input control, and can be toggled by keyboard (see shortcuts below). (Universal Input)
- **“Ask Warp AI”** is integrated into terminal usage flows (e.g., for help/explanations) rather than separated into a different UI surface. (Warp AI page)
- **Workflow context lives alongside chat**: users can attach blocks or Warp Drive objects into the agent request, grounding the conversation in actual session state. (Universal Input)

## Keyboard navigation & shortcuts (AI-relevant)

- **Toggle Agent Mode**: `Cmd+I` (macOS) / `Ctrl+I` (Windows/Linux). (Universal Input)
- **Force mode**: `*` to start Agent Mode, `!` to force Terminal Mode; `Esc` to exit auto-detected Agent Mode. (Universal Input)
- **Accept AI Prompt Suggestion**: `Cmd+Enter` (macOS) / `Ctrl+Shift+Enter` (Windows/Linux). (Active AI)
- **Accept Next Command**: `Tab`, `Right Arrow`, or `Ctrl+F`. (Active AI)
- **Command Search (history)**: `Ctrl+R`. (Keyboard Shortcuts)
- **Command Palette**: `Cmd+P` / `Ctrl+Shift+P`. (Keyboard Shortcuts)
- **Natural language command search**: `Ctrl+`` (Generate). (Keyboard Shortcuts)
- **Block navigation**: `Cmd+Up` / `Cmd+Down` to select blocks; arrow keys navigate within selected blocks. (Blocks)
- **Warp Drive notebook command blocks**: insert with `Cmd+Enter`, navigate with `Cmd+Up` / `Cmd+Down`, focus input with `Cmd+L`. (Notebooks)

## Applicable patterns for Script Kit

- **Unify input, then layer AI**: a single input that can switch modes reduces cognitive load and keeps chat close to execution.
- **Low-friction AI suggestions**: show suggestions as chips with one-keystroke accept; keep them optional and non-blocking.
- **Explicit privacy affordances**: local intent detection + redaction messaging builds trust.
- **Context attachments are first-class**: allow attaching recent output blocks or saved workflows with a simple `@` affordance.
- **Post-command guidance**: “Next Command” after execution is a high-signal place for AI to help.
- **Error-aware diffs**: a focused, apply-or-ignore diff experience beats a generic chat response.
- **Keyboard-first navigation**: short, memorable shortcuts for AI-specific actions (toggle mode, accept suggestion, open command search) are critical.
- **Block-based history as context**: treat each command’s input/output as a selectable object that can be attached to AI requests.

## Notes / open questions

- The docs describe the AI interaction model and shortcuts but are light on model choice, latency budgets, and exact ranking logic for suggestions.
- Some AI features (e.g., AI Command Suggestions) are described at a marketing level; behavioral details likely require product testing.

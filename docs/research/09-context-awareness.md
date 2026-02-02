# AI Chat Context Awareness and Automatic Context Injection Research

This document summarizes patterns and implementation approaches for context-aware AI chat. Focus areas: clipboard integration, selected text, current file or folder context, and screenshot capture.

---

## Table of Contents

1. [Scope and goals](#scope-and-goals)
2. [Context source taxonomy](#context-source-taxonomy)
3. [Pattern catalog](#pattern-catalog)
4. [Clipboard integration](#clipboard-integration)
5. [Selected text integration](#selected-text-integration)
6. [Current file and folder context](#current-file-and-folder-context)
7. [Screenshot capture](#screenshot-capture)
8. [Context assembly pipeline](#context-assembly-pipeline)
9. [Privacy, consent, and security](#privacy-consent-and-security)
10. [Implementation checklists](#implementation-checklists)
11. [Sources](#sources)

---

## Scope and goals

- Identify proven UX patterns for context awareness and automatic context injection in AI chat.
- Provide OS-level and editor-level implementation options for the four key context sources.
- Capture risk and consent considerations that should shape default behavior.

---

## Context source taxonomy

**Explicit vs implicit**
- Explicit: user selects or confirms a context item (file, snippet, screenshot, clipboard).
- Implicit: system automatically attaches context based on activity (active file, selection, recent clipboard).

**Ephemeral vs persistent**
- Ephemeral: current selection, active window, latest clipboard, current screenshot.
- Persistent: workspace index, file tree, recent files, saved conversations.

**System vs app-specific**
- System: clipboard, screen capture, OS accessibility selection.
- App-specific: IDE APIs (active file, workspace, selection), file manager selection, terminal CWD.

**Recommended metadata per context item**
- source (clipboard, selection, file path, screenshot)
- timestamp and app name
- sensitivity (low, medium, high) and size
- confidence (how reliable the extraction was)

---

## Pattern catalog

### 1) Context chips and preview cards
- Inject context as removable chips (file, selection, clipboard, screenshot) with hover preview.
- Show size indicators (lines, characters, image resolution) and the source application.

### 2) Implicit + explicit hybrid
- Auto-attach minimal implicit context (active file name, selection) but require explicit consent for high-risk context (clipboard and screenshots). VS Code Copilot chat uses implicit context such as selected text and active file names, and supports explicit context via a picker and #-mentions. (https://code.visualstudio.com/docs/copilot/chat/copilot-chat-context)

### 3) Context inference and ranking
- Auto-select relevant files and snippets based on the prompt plus workspace index and symbols, then trim to fit the context window. VS Code workspace context uses index, directory structure, and selected or visible text, and keeps only the most relevant parts if the context is too large. (https://code.visualstudio.com/docs/copilot/reference/workspace-context)

### 4) Safe defaults with escalation
- Default to the least sensitive context (active file name, selection) and let users opt into broader context (folder, workspace, clipboard history, screen capture).

### 5) One-time capture and attachment
- For screenshots or selected text, capture once, attach to the message, and avoid continuous background recording.

---

## Clipboard integration

### Patterns
- **On-demand attach**: A "Use clipboard" button attaches current clipboard content as a chip.
- **Clipboard history picker**: Show recent entries and let the user select what to attach.
- **Multi-format awareness**: Prefer richer types (RTF, HTML, image) but fall back to plain text.

### Implementation approaches

**macOS (AppKit)**
- Pasteboards are shared repositories accessed via `NSPasteboard`. They can be public or private and have named system pasteboards like `NSGeneralPboard` and `NSFindPboard`. Pasteboards can contain multiple items and multiple representations (UTIs). A pasteboard server keeps data persistent even after the writer quits. (https://developer.apple.com/library/archive/documentation/General/Devpedia-CocoaApp-MOSX/Pasteboard.html)

**Windows (Win32)**
- Monitor clipboard changes with a clipboard format listener (`AddClipboardFormatListener`) which posts `WM_CLIPBOARDUPDATE`. This is the recommended modern approach over viewer chains. (https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-the-clipboard) (https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-addclipboardformatlistener) (https://learn.microsoft.com/en-us/windows/win32/dataxchg/wm-clipboardupdate)
- For reading, enumerate available formats (or use `GetPriorityClipboardFormat`) and read data via `GetClipboardData`, then decode safely. (https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-operations)

**Linux (sandboxed or Wayland-first)**
- When sandboxed, clipboard access can be mediated through the Clipboard portal, which provides `RequestClipboard` and selection handling for sessions created by other portals (such as Remote Desktop). (https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Clipboard.html) (https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html)

### Guardrails
- Never auto-attach large clipboard payloads. Provide preview and size limits.
- Redact obvious secrets (tokens, passwords) before injection.

---

## Selected text integration

### Patterns
- **Active selection**: Pull currently selected text from the focused app and attach as a chip.
- **Fallback copy**: If direct selection APIs fail, request a user copy action and read the clipboard.
- **Selection + file metadata**: If available, include the file name and selection range or line numbers.

### Implementation approaches

**macOS (Accessibility)**
- `NSAccessibilityProtocol.accessibilitySelectedText()` returns the currently selected text, along with APIs for selection ranges. (https://developer.apple.com/documentation/appkit/nsaccessibilityprotocol/accessibilityselectedtext%28%29)

**Windows (UI Automation)**
- `IUIAutomationTextPattern::GetSelection` returns a collection of text ranges representing the current selection. It supports multiple ranges and can return a degenerate range when only an insertion point exists. (https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtextpattern-getselection)

**Linux (AT-SPI2 / D-Bus)**
- AT-SPI2 is a D-Bus accessibility framework for providing and accessing accessibility information. (https://valadoc.org/atspi-2/index.html)
- The `Atspi.Text` interface exposes `get_selection` and `get_text`, and supports selectable text ranges. (https://lazka.github.io/pgi-docs/Atspi-2.0/interfaces/Text.html) (https://www.manpagez.com/html/libatspi/libatspi-2.26.0/libatspi-atspi-text.php)

---

## Current file and folder context

### Patterns
- **Active file context**: include file name (and optionally language) as implicit context.
- **Selection-first**: if a selection exists, include it instead of the entire file.
- **Workspace-wide recall**: use index + search to select relevant snippets from the workspace.
- **Folder scoping**: allow the user to attach a folder root or workspace as context.

### Implementation approaches

**Editor and IDE APIs (example: VS Code)**
- VS Code defines a workspace as one or more folders opened together. (https://code.visualstudio.com/docs/editing/workspaces/workspaces)
- Copilot chat automatically includes selected text and the active file name as implicit context, and allows explicit context with a picker and #-mentions. (https://code.visualstudio.com/docs/copilot/chat/copilot-chat-context)
- Workspace context uses indexable files, directory structure, code symbols, and selected or visible text to build context, and trims to fit the context window. (https://code.visualstudio.com/docs/copilot/reference/workspace-context)

**Security boundary**
- Workspace Trust exists to reduce risk of unintended code execution when opening untrusted workspaces, and supports Restricted Mode. It is a model worth mirroring when auto-injecting file context from untrusted projects. (https://code.visualstudio.com/api/extension-guides/workspace-trust)

---

## Screenshot capture

### Patterns
- **Explicit capture button**: the user clicks to capture a screen or window, then confirms attach.
- **System picker**: rely on the OS picker so the user selects the target window or display.
- **Preview and redaction**: allow blur or crop before attaching.

### Implementation approaches

**macOS (ScreenCaptureKit)**
- ScreenCaptureKit provides a system picker for selecting content and a screenshot API for high-definition captures. (https://developer.apple.com/videos/play/wwdc2023/10136/)
- `SCContentFilter` provides fine-grained filtering by display, applications, and windows. (https://developer.apple.com/documentation/screencapturekit/sccontentfilter/init%28display%3Aincluding%3Aexceptingwindows%3A%29)

**Windows (Windows.Graphics.Capture)**
- Windows.Graphics.Capture provides APIs for capturing a display or application window. The system UI picker lets users choose a target, and the system draws a yellow border around captured content. (https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/screen-capture)

**Linux / Wayland (XDG Desktop Portal)**
- The Screenshot portal lets apps request a screenshot and returns a URI to the image. (https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html)
- The ScreenCast portal allows screen cast sessions with source types (monitor, window, virtual) and cursor modes. (https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html)
- On Wayland desktops, the ScreenCast portal is the primary way to capture screens or windows. (https://flatpak.github.io/xdg-desktop-portal/docs/reasons-to-use-portals.html)

---

## Context assembly pipeline

1. **Collect** candidate signals (selection, clipboard, active file, workspace index, screenshot).
2. **Normalize** into context items with metadata (source, size, sensitivity, timestamps).
3. **Rank** by explicitness, recency, and relevance to the user prompt. VS Code workspace context runs multiple strategies and keeps the most relevant results. (https://code.visualstudio.com/docs/copilot/reference/workspace-context)
4. **Compress** (summarize, dedupe, or chunk) to fit the model context window.
5. **Confirm** with the user via chips or preview cards.
6. **Inject** into the model with clear source labeling.

---

## Privacy, consent, and security

- **Default to least sensitive context** (active file name, small selection).
- **Require explicit consent** for clipboard history and screenshots.
- **Respect trust boundaries** for untrusted workspaces (similar to VS Code Workspace Trust). (https://code.visualstudio.com/api/extension-guides/workspace-trust)
- **Limit retention**: store context items only for the current message unless the user opts in.

---

## Implementation checklists

### Clipboard
- [ ] Detect change events (listener or sequence number)
- [ ] Display clipboard preview before attach
- [ ] Size limit and redaction pass
- [ ] Support text + image formats

### Selected text
- [ ] Read selection via accessibility API or editor API
- [ ] Fallback to user-initiated copy
- [ ] Include file path and line range when available

### File and folder context
- [ ] Resolve active file name and workspace root
- [ ] Allow explicit folder attach or # mention
- [ ] Index and search to extract relevant snippets
- [ ] Respect trust / untrusted mode

### Screenshot
- [ ] Use OS picker or portal
- [ ] Provide preview + redaction
- [ ] Attach as image with optional OCR text

---

## Sources

### Clipboard and accessibility
- https://developer.apple.com/library/archive/documentation/General/Devpedia-CocoaApp-MOSX/Pasteboard.html
- https://developer.apple.com/documentation/appkit/nsaccessibilityprotocol/accessibilityselectedtext%28%29
- https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-the-clipboard
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-addclipboardformatlistener
- https://learn.microsoft.com/en-us/windows/win32/dataxchg/wm-clipboardupdate
- https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-operations
- https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtextpattern-getselection
- https://valadoc.org/atspi-2/index.html
- https://lazka.github.io/pgi-docs/Atspi-2.0/interfaces/Text.html
- https://www.manpagez.com/html/libatspi/libatspi-2.26.0/libatspi-atspi-text.php

### File and workspace context
- https://code.visualstudio.com/docs/copilot/reference/workspace-context
- https://code.visualstudio.com/docs/copilot/chat/copilot-chat-context
- https://code.visualstudio.com/docs/editing/workspaces/workspaces
- https://code.visualstudio.com/api/extension-guides/workspace-trust

### Screen capture
- https://developer.apple.com/videos/play/wwdc2023/10136/
- https://developer.apple.com/documentation/screencapturekit/sccontentfilter/init%28display%3Aincluding%3Aexceptingwindows%3A%29
- https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/screen-capture
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html
- https://flatpak.github.io/xdg-desktop-portal/docs/reasons-to-use-portals.html
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Clipboard.html

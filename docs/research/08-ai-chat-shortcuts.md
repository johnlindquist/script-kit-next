# AI chat keyboard shortcuts - research and recommendations

Date: 2026-02-01
Scope: AI chat interfaces and assistants with keyboard-first workflows (ChatGPT desktop/web, Microsoft Copilot, Perplexity Comet, Claude Code), plus accessibility guidance from W3C/WAI-ARIA.

## 1. Observed patterns across apps (evidence)

### 1.1 Global launch / summon shortcuts
- ChatGPT desktop: global shortcut to open ChatGPT from any screen uses Option+Space (macOS) or Alt+Space (Windows). [OpenAI desktop page](https://chatgpt.com/features/desktop)
- ChatGPT macOS Chat Bar: Option+Space opens the chat bar; shortcut can be changed in Settings. [OpenAI Help Center - Chat Bar](https://help.openai.com/en/articles/9295241-how-to-launch-the-chat-bar)
- Microsoft Copilot on Windows: Copilot key or Windows+C launches Copilot; behavior is configurable in Settings. [Microsoft Support - Copilot on Windows](https://support.microsoft.com/en-us/topic/getting-started-with-copilot-on-windows-1159c61f-86c3-4755-bf83-7fbff7e0982d)
- Perplexity Comet: Alt+A activates the Comet Assistant. [Perplexity Comet - Alt+A](https://www.perplexity.ai/comet/resources/videos/activating-comet-assistant)

Pattern: a global summon shortcut exists and is configurable.

### 1.2 Voice mode shortcuts
- Microsoft Copilot: press and hold Copilot key or Windows+C to start voice (press-to-talk). [Microsoft Support - Copilot on Windows](https://support.microsoft.com/en-us/topic/getting-started-with-copilot-on-windows-1159c61f-86c3-4755-bf83-7fbff7e0982d)
- Perplexity Comet: Alt+Shift+V activates voice mode. [Perplexity Comet - Voice Mode](https://www.perplexity.ai/comet/resources/videos/comet-voice-mode)

Pattern: voice has a dedicated shortcut, often press-and-hold or a distinct chord.

### 1.3 Shortcut discovery and help
- ChatGPT web app (reported): Ctrl/Cmd + / opens the shortcuts menu. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
- Claude Code: press ? to see available shortcuts for the environment. [Claude Code docs](https://code.claude.com/docs/en/interactive-mode)

Pattern: a discoverable shortcuts overlay exists and is reachable from the keyboard.

### 1.4 Prompt shortcuts / slash commands
- Perplexity Comet: typing / opens a shortcut selector and triggers reusable prompt shortcuts. [Comet Query Shortcuts](https://comet-help.perplexity.ai/en/articles/11906981-comet-query-shortcuts)
- Claude Code: typing / shows commands; / is the entry point for built-in commands. [Claude Code docs](https://code.claude.com/docs/en/interactive-mode)
- ChatGPT macOS app: added slash commands for quick actions. [OpenAI macOS release notes](https://help.openai.com/en/articles/9703738-desktop-app-release-notes)

Pattern: slash-activated command palette is common for power actions.

### 1.5 Input ergonomics and generation control
- Claude Code: Ctrl+C cancels generation; Up/Down arrows navigate command history; multiline input uses Option+Enter or Shift+Enter. [Claude Code docs](https://code.claude.com/docs/en/interactive-mode)
- ChatGPT macOS app: Command + . stops streaming responses. [OpenAI macOS release notes](https://help.openai.com/en/articles/9703738-desktop-app-release-notes)
- ChatGPT web app (reported): Shift+Esc focuses input; Up Arrow edits the last prompt. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)

Pattern: dedicated cancel/stop shortcut plus input history and multiline entry.

### 1.6 Navigation shortcuts
- ChatGPT web app (reported): Ctrl+Shift+O new chat; Ctrl+Shift+S toggle sidebar; Ctrl+/ show shortcuts list. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
- ChatGPT web app (reported, older list): Ctrl+Shift+O new chat; Shift+Esc focus input; Ctrl+Shift+S toggle sidebar; Ctrl+/ show shortcuts. [Android Headlines](https://www.androidheadlines.com/2023/08/major-chatgpt-update-suggested-replies-keyboard-shortcuts.html)
- WAI-ARIA combobox/listbox patterns: Up/Down move through suggestions, Enter accepts, Escape closes. [WAI-ARIA APG - Combobox](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/)

Pattern: navigation favors arrow keys + Enter/Escape in lists and quick access to new chat / sidebar.

### 1.7 Shortcut customization
- ChatGPT Chat Bar shortcut can be changed in Settings. [OpenAI Help Center - Chat Bar](https://help.openai.com/en/articles/9295241-how-to-launch-the-chat-bar)
- Microsoft Copilot shortcut behavior is configurable in Settings. [Microsoft Support - Copilot on Windows](https://support.microsoft.com/en-us/topic/getting-started-with-copilot-on-windows-1159c61f-86c3-4755-bf83-7fbff7e0982d)
- Perplexity Comet allows re-assigning button hotkeys via Settings -> Shortcuts. [Comet Query Shortcuts](https://comet-help.perplexity.ai/en/articles/11906981-comet-query-shortcuts)

Pattern: users can rebind or disable shortcuts.

## 2. Accessibility requirements (keyboard first)

- WCAG 2.1.1: all functionality must be operable by keyboard, without timing-based keystrokes. [W3C WCAG 2.1.1](https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html)
- WCAG 2.1.2: avoid keyboard traps; users must be able to move focus away using keyboard (and know how). [W3C WCAG 2.1.2](https://www.w3.org/WAI/WCAG21/Understanding/no-keyboard-trap.html)
- WCAG 2.1.4: character-only shortcuts must be off, remappable, or active only when a component has focus. [W3C WCAG 2.1.4](https://www.w3.org/WAI/WCAG21/Understanding/character-key-shortcuts.html)
- WAI-ARIA APG: do not intercept standard text editing keys; follow established keyboard interactions for combobox/listbox suggestions. [WAI-ARIA APG - Combobox](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/)

## 3. Recommendations for an AI chat interface

### 3.1 Principles
- Make the assistant summonable from anywhere (global shortcut) and allow reassignment.
- Provide an in-app shortcuts overlay reachable by keyboard (Cmd/Ctrl + / is a proven pattern).
- Avoid single-letter global shortcuts unless focus-limited or remappable (WCAG 2.1.4).
- Keep all list and menu navigation consistent with WAI-ARIA patterns (Up/Down, Enter, Escape).

### 3.2 Recommended default shortcut map (proposal)

Action | Suggested default | Evidence / rationale
--- | --- | ---
Open assistant / chat bar | macOS: Option+Space; Windows: Alt+Space or Windows+C | Global summon pattern in ChatGPT desktop + Copilot. Allow user override. [OpenAI desktop page](https://chatgpt.com/features/desktop) [Microsoft Support](https://support.microsoft.com/en-us/topic/getting-started-with-copilot-on-windows-1159c61f-86c3-4755-bf83-7fbff7e0982d)
Show shortcuts | Cmd/Ctrl + / | Reported in ChatGPT web shortcuts list; matches shortcut discovery pattern. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
New chat | Cmd/Ctrl + Shift + O | Reported in ChatGPT web shortcuts list; fast reset of context. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
Toggle sidebar / history | Cmd/Ctrl + Shift + S | Reported in ChatGPT web shortcuts list; supports focus and space management. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
Focus input | Shift + Esc | Reported in ChatGPT web shortcuts list; reduces mouse dependence. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
Send message | Enter | Standard text input behavior; preserve platform conventions. [W3C WCAG 2.1.1](https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html)
Insert newline | Shift+Enter (and Option+Enter on macOS) | Seen in Claude Code for multiline input; provides explicit newline. [Claude Code docs](https://code.claude.com/docs/en/interactive-mode)
Stop generation | Esc and/or Cmd/Ctrl + . | Command + . used to stop streaming in ChatGPT macOS app; Esc is a common cancel key. [OpenAI macOS release notes](https://help.openai.com/en/articles/9703738-desktop-app-release-notes)
Edit last prompt | Up Arrow in input | Reported in ChatGPT web shortcuts list; speeds corrections. [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
Slash commands | / (when input focused) | Used by Comet and Claude Code to trigger commands; must be focus-limited per WCAG 2.1.4. [Comet Query Shortcuts](https://comet-help.perplexity.ai/en/articles/11906981-comet-query-shortcuts) [Claude Code docs](https://code.claude.com/docs/en/interactive-mode)

### 3.3 Power-user features to consider
- Prompt shortcuts / templates: allow users to create reusable prompts and manage them in Settings (Comet pattern). [Comet Query Shortcuts](https://comet-help.perplexity.ai/en/articles/11906981-comet-query-shortcuts)
- Command history: Up/Down to recall previous prompts; consider reverse search (Ctrl+R) for long sessions. [Claude Code docs](https://code.claude.com/docs/en/interactive-mode)
- Voice toggle or press-to-talk: support a dedicated chord (Alt+Shift+V in Comet) or press-and-hold (Copilot). [Perplexity Comet - Voice Mode](https://www.perplexity.ai/comet/resources/videos/comet-voice-mode) [Microsoft Support](https://support.microsoft.com/en-us/topic/getting-started-with-copilot-on-windows-1159c61f-86c3-4755-bf83-7fbff7e0982d)
- Code-heavy workflows: include quick actions such as copy last code block or stop streaming, which are present in ChatGPT shortcuts lists and app release notes. [Android Headlines](https://www.androidheadlines.com/2023/08/major-chatgpt-update-suggested-replies-keyboard-shortcuts.html) [OpenAI macOS release notes](https://help.openai.com/en/articles/9703738-desktop-app-release-notes)

### 3.4 Navigation and focus rules
- Use WAI-ARIA combobox/listbox keyboard interactions for suggestion lists and action menus (Up/Down to move, Enter to accept, Escape to close). [WAI-ARIA APG - Combobox](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/)
- Avoid keyboard traps; always provide a clear exit path from modals, menus, and overlays. [W3C WCAG 2.1.2](https://www.w3.org/WAI/WCAG21/Understanding/no-keyboard-trap.html)

## 4. Notes on recency and variability
- Some ChatGPT web shortcuts are reported in 2023-2025 media coverage; treat those as indicative patterns, not authoritative specs, and validate in-product. [Android Headlines](https://www.androidheadlines.com/2023/08/major-chatgpt-update-suggested-replies-keyboard-shortcuts.html) [Tom's Guide](https://www.tomsguide.com/ai/these-4-chatgpt-keyboard-shortcuts-will-change-how-you-write)
- OS-level conflicts are common (Alt+Space or Windows+C may be reserved). Offer remapping and detect collisions on setup.


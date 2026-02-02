# Alfred AI workflows and chat features

Research focus: launcher-style UX, keyboard-first interaction, quick actions, response display. Sources are listed at the end.

## Executive summary

- Alfred's AI experience is delivered primarily via workflows (official ChatGPT/DALL-E and community workflows), triggered from the main launcher input by keyword, universal action, fallback search, or hotkey. The launcher pattern is consistent: type a keyword, see results or a view, then act with the keyboard.
- Response display centers on Alfred 5.5's Text View and Grid View (scriptable JSON views) plus legacy outputs like Large Type and Clipboard/Paste. The official ChatGPT/DALL-E workflow is built on Text View and supports ongoing, interactive chat. Alfred's blog now recommends the official workflow over Chatfred.
- Quick actions are unified through Universal Actions (action panel). Actions can be invoked from Alfred results (right arrow) or globally (Cmd+/ by default). Many workflows also define modifier-key shortcuts to copy, archive, or stop generation.

## Launcher-style UX patterns (Alfred)

- Keyword-first activation: Workflows are triggered by keywords (e.g., `chatgpt`, `dalle`, `cf`, `cfi`, `gpt`). Keywords can be configured and can accept optional arguments. This keeps the UI consistent with Alfred's main launcher. Sources: keyword input docs, ChatGPT workflow, Chatfred, AskGPT. 
- Script Filter list as the default result UI: When using Script Filter inputs, Alfred shows placeholder title/subtitle and replaces the subtitle with "Please Wait" while initial results are fetched. Script Filters also support queue mode (wait for current run vs terminate and re-run on new input) to stay responsive under rapid typing. This is the dominant list pattern for interactive workflows. Source: Script Filter input docs.
- Views for richer responses: Alfred 5.5 adds Text View (editable text + Markdown preview) and Grid View (image-first results). Both are scriptable and accept JSON output, giving workflows more UI control than list results. Sources: Text View and Grid View docs, Alfred 5.5 release notes.

## Keyboard-first interaction

- Universal Actions as the main quick-action surface: Alfred's action panel lets users choose actions for the selected item, inside Alfred or from any app. It is invoked from results with the right-arrow (or configured shortcut) and globally with a hotkey (Cmd+/ by default). Alfred ships 60+ built-in actions and lets workflows add custom actions. Sources: Universal Actions help and release notes.
- Workflow-specific key bindings: The official ChatGPT workflow uses Enter plus modifier key combos for common actions (clear chat, copy last answer, copy full chat, stop generation). Chat history navigation is also keyboard-driven and supports file buffer selection and delete via universal action. Source: ChatGPT workflow page.
- Hotkeys and fallback search: Many workflows are designed to run via hotkey or fallback search (e.g., Chatfred, ChatGPT workflow). This preserves a keyboard-only flow across contexts. Sources: ChatGPT workflow page, Chatfred page, Hotkey trigger docs.

## Quick actions in AI workflows

- Official ChatGPT workflow:
  - Trigger via keyword (`chatgpt`), Universal Action, or Fallback Search.
  - Enter starts a new question; Cmd+Enter clears chat; Option+Enter copies last answer; Ctrl+Enter copies the full chat; Shift+Enter stops generation.
  - Chat History view shows first question as title and last as subtitle; Enter loads a previous chat; delete uses Universal Action; File Buffer allows multi-select. Source: ChatGPT workflow page.
- Official DALL-E workflow:
  - Trigger via keyword (`dalle`).
  - Enter sends a prompt; Cmd+Enter archives images; Option+Enter reveals the last image in Finder. Source: ChatGPT workflow page.
- Chatfred (community, deprecated in Alfred Gallery; repo archived May 13, 2024, but still informative):
  - Trigger with `cf` (chat) and `cfi` (images). Supports fallback search and hotkey activation.
  - Responses are shown in Large Type; streaming replies are supported; optional paste-to-frontmost-app is available via configuration or modifier key.
  - Provides a text-generation mode (`cft`) with multiple modifier actions (Large Type, speak, save, copy). Sources: Alfred Gallery Chatfred page, Chatfred GitHub.
- AskGPT (community):
  - Trigger via `gpt` keyword or by typing `\\gpt` in any window to invoke a prompt while typing, emphasizing keyboard-first, in-context usage. Sources: AskGPT GitHub, Packal listing.

## Response display patterns

- Text View (Alfred 5.5): Scriptable, dynamically updated via JSON output; can preview Markdown; supports an extra input field when used as a script source so the script can be re-run with new arguments. The official ChatGPT/DALL-E workflow is built with this view. Source: Text View docs.
- Grid View (Alfred 5.5): Scriptable, image-friendly view with optional filter input. Useful for AI image results. Source: Grid View docs.
- Large Type output: Shows text in large characters with configurable font/background/fade behavior. Many AI workflows (e.g., Chatfred) use it for immediate, full-screen response display. Sources: Large Type output docs, Chatfred docs.
- Clipboard and paste: Workflows can copy responses to clipboard and optionally paste into the frontmost app, supporting a quick "generate then insert" flow. Source: Workflow outputs docs and Chatfred GitHub.

## Takeaways for Script Kit GPUI

- Preserve a keyword-first, search-bar-centric entry point; use list results for fast selection, then transition to richer views for response display.
- Offer universal, keyboard-first action panels with a predictable shortcut, and allow workflows to add custom actions relevant to AI responses.
- For AI chat, prioritize a Text View-like component: streaming output, Markdown preview, and an optional input field to continue the conversation without leaving the view.
- For images, consider a grid view with optional filtering and keyboard selection.
- Provide direct, modifier-key shortcuts for common response actions (copy last response, copy full conversation, stop generation, archive).

## Sources

- https://alfred.app/workflows/alfredapp/openai/
- https://www.alfredapp.com/help/workflows/user-interface/text/
- https://www.alfredapp.com/help/workflows/user-interface/grid/
- https://www.alfredapp.com/help/workflows/inputs/script-filter/
- https://www.alfredapp.com/help/features/universal-actions/
- https://www.alfredapp.com/blog/releases/alfred-4-5-released-universal-actions/
- https://www.alfredapp.com/help/workflows/outputs/large-type/
- https://www.alfredapp.com/help/features/large-type/
- https://www.alfredapp.com/blog/fun-and-interesting/chatgpt-bringing-ai-to-alfred-with-chatfred/
- https://alfred.app/workflows/chrislemke/chatfred/
- https://github.com/chrislemke/ChatFred
- https://github.com/phguo/AskGPT
- https://www.packal.org/workflow/askgpt
- https://www.alfredapp.com/help/workflows/triggers/hotkey/
- https://www.alfredapp.com/help/workflows/inputs/keyword/

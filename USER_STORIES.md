# Script Kit GPUI - User Stories

> 195 deduplicated user stories extracted from codebase analysis across builtins, prompts, protocol, AI, platform, and UI components.

---

## 1. Main Prompt & Search

- **US-1.1** As a user, I want to open Script Kit with a global hotkey, so that I can quickly access my scripts from anywhere.
- **US-1.2** As a user, I want to type in the main prompt to fuzzy-search my scripts, so that I can find and run them quickly.
- **US-1.3** As a user, I want to see installed applications alongside my scripts in search results, so that I can launch apps without switching to Spotlight.
- **US-1.4** As a user, I want to see built-in commands (Clipboard History, Emoji Picker, etc.) in search results, so that I can access all features from one place.
- **US-1.5** As a user, I want to see "Suggested" (frecency-ranked) items at the top of search results, so that my most-used scripts appear first.
- **US-1.6** As a user, I want to clear my Suggested/Recently Used history, so that I can reset my frecency rankings.
- **US-1.7** As a user, I want to press Escape to dismiss the prompt, so that I can return to my previous application.
- **US-1.8** As a user, I want the window to appear as a non-activating panel (like Raycast), so that focus returns to my previous app after dismissal.
- **US-1.9** As a user, I want to use keyboard navigation (up/down arrows, Enter) to browse and select items, so that I can operate entirely without a mouse.
- **US-1.10** As a user, I want to press Cmd+K to open an actions panel on any prompt, so that I can access contextual actions for the selected item.
- **US-1.11** As a user, I want to see menu bar items from the frontmost application in search results, so that I can trigger app menu commands via keyboard.
- **US-1.12** As a user, I want to see inline calculation results when I type math expressions, so that I can use Script Kit as a quick calculator.
- **US-1.13** As a user, I want to copy calculation results to clipboard with a single keystroke, so that I can quickly use computed values.
- **US-1.14** As a user, I want to see script metadata (tags, description, icon) in search results, so that I understand what each script does.
- **US-1.15** As a user, I want input history tracking, so that Script Kit can suggest previously entered values.

## 2. Script Management

- **US-2.1** As a user, I want to create a new script from a starter template, so that I can quickly scaffold a new automation.
- **US-2.2** As a user, I want to create a new scriptlet bundle with YAML frontmatter, so that I can bundle multiple small scripts together.
- **US-2.3** As a user, I want to run a script by selecting it from the search results, so that I can execute automations on demand.
- **US-2.4** As a user, I want to assign a global hotkey to a script, so that I can trigger it instantly from anywhere.
- **US-2.5** As a user, I want to see a creation feedback screen after creating a script, so that I can quickly open, edit, or run the new script.
- **US-2.6** As a user, I want to view and manage running script processes (Process Manager), so that I can inspect or terminate scripts.
- **US-2.7** As a user, I want to stop all running scripts at once, so that I can quickly clean up hung processes.
- **US-2.8** As a user, I want scripts to run via Bun in a child process, so that I get fast TypeScript execution with full Node.js compatibility.
- **US-2.9** As a user, I want to star/favorite scripts, so that I can quickly access them from the Favorites list.
- **US-2.10** As a user, I want to edit my existing scripts in my preferred editor, so that I can modify and improve them.
- **US-2.11** As a user, I want to view execution logs for my scripts, so that I can debug issues.
- **US-2.12** As a user, I want to delete scripts I no longer use, so that I can keep my collection clean.
- **US-2.13** As a user, I want to reload scripts without restarting the app, so that I can test changes immediately.
- **US-2.14** As a user, I want to copy script contents to my clipboard, so that I can share or back up script code.
- **US-2.15** As a user, I want to reset script ranking/usage history, so that I can have a fresh frecency list.
- **US-2.16** As a user, I want process tracking with PID files, so that the app can detect orphaned processes on startup.

## 3. Script Naming & Creation

- **US-3.1** As a user, I want to provide a friendly name when creating a new script, so that I can give it a human-readable title.
- **US-3.2** As a user, I want to see a live preview of the kebab-case filename, so that I know exactly what file will be created.
- **US-3.3** As a user, I want the dialog to validate names and prevent duplicates, so that I avoid invalid or overwritten files.
- **US-3.4** As a user, I want quick access to "Edit" or "Open in Finder" after creation, so that I can immediately work with the new script.

## 4. Script Prompts (SDK API)

- **US-4.1** As a script developer, I want to use `arg()` to show a searchable list prompt with choices, so that users can select from options.
- **US-4.2** As a script developer, I want to use `div()` to display custom HTML/Markdown content, so that I can render rich UI with headers, lists, code blocks.
- **US-4.3** As a script developer, I want to use `editor()` to show a code editor with syntax highlighting and find/replace, so that users can edit text or code.
- **US-4.4** As a script developer, I want to use `term()` to show an interactive terminal with VT100/xterm escape sequences, so that users can run shell commands.
- **US-4.5** As a script developer, I want to use `form()` / `fields()` to show a multi-field form with Tab navigation, so that users can enter structured data.
- **US-4.6** As a script developer, I want to use `select()` to show a multi-select prompt with Cmd+Space toggle, so that users can choose multiple items.
- **US-4.7** As a script developer, I want to use `path()` to show a file/folder picker with directory browsing, so that users can select filesystem paths.
- **US-4.8** As a script developer, I want to use `env()` to prompt for environment variables stored in the system keyring, so that secrets are stored securely.
- **US-4.9** As a script developer, I want to use `drop()` to show a drag-and-drop file target with visual feedback, so that users can provide files by dragging.
- **US-4.10** As a script developer, I want to use `template()` to show a string template editor with {{placeholder}} syntax and live preview, so that users can fill in templates.
- **US-4.11** As a script developer, I want to use `chat()` to show a conversational chat interface with turn-pair history, so that users can have multi-turn interactions.
- **US-4.12** As a script developer, I want to use `confirm()` to show a yes/no confirmation dialog with configurable button text, so that users can approve destructive actions.
- **US-4.13** As a script developer, I want to use `mini()` and `micro()` to show compact input prompts, so that lightweight input doesn't take up the full window.
- **US-4.14** As a script developer, I want to use `webcam()` to capture a photo from the webcam with live preview, so that I can build camera-based scripts.
- **US-4.15** As a script developer, I want to use `setInput()` to programmatically set the prompt's input text, so that I can pre-fill values.
- **US-4.16** As a script developer, I want to use `forceSubmit()` to programmatically submit a prompt value, so that I can automate prompt responses.

## 5. Clipboard History

- **US-5.1** As a user, I want to open Clipboard History to view all recently copied items, so that I can reuse past clipboard content.
- **US-5.2** As a user, I want to search/filter my clipboard history, so that I can quickly find a specific copied item.
- **US-5.3** As a user, I want to select a clipboard item and paste it, so that I can reuse previously copied text.
- **US-5.4** As a user, I want to use "Paste Sequentially" to paste multiple items one-by-one, so that I can fill in forms or lists efficiently.
- **US-5.5** As a user, I want to pin/favorite clipboard items, so that important clips persist and aren't pushed out by new copies.
- **US-5.6** As a user, I want to delete individual clipboard history entries, so that I can remove sensitive data.
- **US-5.7** As a user, I want to see image previews in clipboard history, so that I can identify images by their visual content.
- **US-5.8** As a user, I want to share clipboard entries via the system share menu, so that I can send items to other apps.
- **US-5.9** As a user, I want to save clipboard entries as files, so that I can preserve important content.
- **US-5.10** As a user, I want to save clipboard content as code snippets, so that I can reuse code blocks.
- **US-5.11** As a user, I want to open clipboard items with Quick Look, so that I can preview images and documents.
- **US-5.12** As a user, I want to perform OCR on clipboard images, so that I can extract text from screenshots.
- **US-5.13** As a user, I want to attach clipboard content to AI Chat, so that I can analyze or discuss it with the assistant.
- **US-5.14** As a user, I want to copy clipboard entry metadata (filename, size), so that I can reference details about my clipboard history.
- **US-5.15** As a user, I want clipboard history to automatically clear old items, so that my storage stays clean.
- **US-5.16** As a user, I want to capture a screenshot using CleanShot integration, so that I can quickly capture and manipulate screenshots.

## 6. Emoji Picker

- **US-6.1** As a user, I want to open the Emoji Picker to browse emojis by category (Smileys, People, Animals, Food, Travel, Activities, Objects, Symbols, Flags), so that I can find emojis by type.
- **US-6.2** As a user, I want to search emojis by name or keyword, so that I can quickly find the right emoji.
- **US-6.3** As a user, I want to select an emoji and have it copied to clipboard, so that I can paste it into any application.

## 7. AI Chat

- **US-7.1** As a user, I want to open AI Chat to converse with Claude, GPT, or other AI providers, so that I can get AI assistance.
- **US-7.2** As a user, I want to start a new AI conversation, so that I can begin a fresh chat context.
- **US-7.3** As a user, I want to generate a Script Kit script from natural language via "Generate Script with AI", so that I can create automations without writing code.
- **US-7.4** As a user, I want to send a full-screen screenshot to AI Chat, so that I can ask about what's on my screen.
- **US-7.5** As a user, I want to send a screenshot of the focused window to AI Chat, so that I can ask about a specific app.
- **US-7.6** As a user, I want to send the currently selected text to AI Chat, so that I can ask AI about content I'm reading.
- **US-7.7** As a user, I want to send the focused browser tab URL to AI Chat, so that I can discuss web content with AI.
- **US-7.8** As a user, I want to send a selected screen area to AI Chat, so that I can ask about a specific region.
- **US-7.9** As a user, I want to configure API keys for Vercel AI Gateway, OpenAI, and Anthropic, so that AI Chat can connect to my preferred providers.
- **US-7.10** As a user, I want to choose between different AI models from a footer dropdown, so that I can pick the right model for my task.
- **US-7.11** As a user, I want to create, import, and search AI presets/templates, so that I can reuse common AI prompt configurations.
- **US-7.12** As a user, I want AI Chat to stream responses in real-time, so that I see output as it's generated.
- **US-7.13** As a user, I want to see markdown-formatted responses with syntax-highlighted code blocks, so that code renders correctly.
- **US-7.14** As a user, I want conversation history with messages bundled as turn pairs, so that I can follow the conversation flow.
- **US-7.15** As a user, I want full-text search across chat titles and message content, so that I can find specific conversations.
- **US-7.16** As a user, I want BYOK (Bring Your Own Key) AI providers per chat, so that I can use different AI services in different conversations.
- **US-7.17** As a user, I want a "Continue in Chat" option to open the full AI chat window, so that I can expand inline conversations.

## 8. Notes

- **US-8.1** As a user, I want a separate floating Notes window with a global hotkey, so that I can quickly capture thoughts from anywhere.
- **US-8.2** As a user, I want to create a new note, so that I can capture information quickly.
- **US-8.3** As a user, I want to search through my notes, so that I can find specific content.
- **US-8.4** As a user, I want to use Quick Capture to jot a note without opening the full Notes window, so that capture is frictionless.
- **US-8.5** As a user, I want notes to support Markdown with syntax highlighting, so that I can format my notes.
- **US-8.6** As a user, I want to delete (soft-delete) and restore notes, so that I can manage my note collection safely.
- **US-8.7** As a user, I want notes to auto-save as I type, so that I never lose my work.
- **US-8.8** As a user, I want to see word count and character count for notes, so that I can track note length.
- **US-8.9** As a user, I want to organize notes in a sidebar list, so that I can manage multiple notes efficiently.
- **US-8.10** As a user, I want to export notes as plain text, markdown, or HTML, so that I can share or archive them.
- **US-8.11** As a user, I want the notes window to auto-resize based on content, so that the window fits my text naturally.

## 9. File Search

- **US-9.1** As a user, I want to search files and browse directories, so that I can find and open files quickly.
- **US-9.2** As a user, I want to navigate into subdirectories in the file search, so that I can drill down to the right location.
- **US-9.3** As a user, I want to open a found file in its default application, so that I can act on search results.
- **US-9.4** As a user, I want to reveal files in Finder, so that I can see them in context.
- **US-9.5** As a user, I want to copy file paths and deep links to my clipboard, so that I can reference them elsewhere.
- **US-9.6** As a user, I want to see file metadata (size, modified date), so that I can make informed decisions.
- **US-9.7** As a user, I want to preview files with Quick Look, so that I can view them without launching applications.
- **US-9.8** As a user, I want to open files with specific applications, so that I can use the right tool for each file type.
- **US-9.9** As a user, I want to filter search results by file type, so that I can narrow down to relevant files.

## 10. Window Switcher

- **US-10.1** As a user, I want to open the Window Switcher to see all open windows, so that I can quickly switch between apps.
- **US-10.2** As a user, I want to search/filter open windows by title, so that I can find a specific window.
- **US-10.3** As a user, I want to select a window to bring it to focus, so that I can switch to it.
- **US-10.4** As a user, I want to see window previews, so that I can identify windows visually.

## 11. Quicklinks

- **US-11.1** As a user, I want to manage quick URL shortcuts, so that I can open frequently-used URLs from the prompt.
- **US-11.2** As a user, I want quick links to support `{query}` expansion, so that I can perform parameterized URL searches.
- **US-11.3** As a user, I want to edit or delete quicklinks, so that I can keep my shortcuts up-to-date.

## 12. Theme & Appearance

- **US-12.1** As a user, I want to browse and choose a color theme with live preview, so that I can customize the app's appearance.
- **US-12.2** As a user, I want to toggle macOS Dark Mode from the prompt, so that I can switch appearance quickly.
- **US-12.3** As a user, I want the app window to use vibrancy/translucency, so that it blends with my desktop aesthetic.
- **US-12.4** As a user, I want to customize theme opacity and color adjustments, so that I can fine-tune the look.
- **US-12.5** As a user, I want themes to apply across all windows (main, notes, AI, actions), so that the entire app looks consistent.
- **US-12.6** As a user, I want to search for themes by name, so that I can find a specific theme.

## 13. System Commands

- **US-13.1** As a user, I want to empty the Trash from the prompt, so that I can clean up disk space quickly.
- **US-13.2** As a user, I want to lock the screen from the prompt, so that I can secure my Mac instantly.
- **US-13.3** As a user, I want to put the system to sleep from the prompt, so that I can power-save quickly.
- **US-13.4** As a user, I want to restart or shut down from the prompt, so that I can manage power without leaving my workflow.
- **US-13.5** As a user, I want to log out from the prompt, so that I can switch users quickly.
- **US-13.6** As a user, I want to show the desktop from the prompt, so that I can quickly reveal my desktop.
- **US-13.7** As a user, I want to open Mission Control from the prompt, so that I can see all windows and desktops.
- **US-13.8** As a user, I want to open Launchpad from the prompt, so that I can browse all installed apps.
- **US-13.9** As a user, I want to force quit apps from the prompt, so that I can kill unresponsive applications.
- **US-13.10** As a user, I want to set system volume to preset levels (0/25/50/75/100%) or toggle mute, so that I can control audio quickly.
- **US-13.11** As a user, I want to toggle Do Not Disturb from the prompt, so that I can silence notifications.
- **US-13.12** As a user, I want to start the screen saver from the prompt, so that I can activate it instantly.
- **US-13.13** As a user, I want to quit Script Kit from the prompt, so that I can shut down the app cleanly.

## 14. System Settings Shortcuts

- **US-14.1** As a user, I want to open System Settings from the prompt, so that I can access preferences quickly.
- **US-14.2** As a user, I want to open specific settings panes (Privacy, Display, Sound, Network, Keyboard, Bluetooth, Notifications) from the prompt, so that I can jump directly to the setting I need.

## 15. Permissions

- **US-15.1** As a user, I want to check all required macOS permissions, so that I know if Script Kit has the access it needs.
- **US-15.2** As a user, I want to request Accessibility permission from the prompt, so that global hotkeys and automation work correctly.
- **US-15.3** As a user, I want to open Accessibility Settings from the prompt, so that I can manually grant permissions.

## 16. Utility Tools

- **US-16.1** As a user, I want to open a Scratch Pad editor that auto-saves, so that I can jot down text or code snippets persistently.
- **US-16.2** As a user, I want to open a Quick Terminal for running shell commands, so that I can execute commands without switching to Terminal.app.
- **US-16.3** As a developer, I want a Cmd+K command bar in the terminal, so that I can quickly access terminal actions (clear, scroll, copy, paste).
- **US-16.4** As a developer, I want theme integration with the terminal, so that the terminal respects my Script Kit theme colors.

## 17. Hotkeys & Shortcuts

- **US-17.1** As a user, I want to register global hotkeys for scripts, so that I can trigger specific scripts from anywhere with a keyboard shortcut.
- **US-17.2** As a user, I want to set a global hotkey to summon the Script Kit main prompt, so that I can invoke it from any application.
- **US-17.3** As a user, I want to record keyboard shortcuts interactively, so that I can bind them without editing config files.
- **US-17.4** As a user, I want the shortcut recorder to detect conflicts, so that I know if my shortcut clashes with existing bindings.
- **US-17.5** As a user, I want to add aliases for commands, so that I can invoke scripts by alternative names.
- **US-17.6** As a user, I want to see all available keyboard shortcuts in a grid overlay, so that I can learn what shortcuts are available.
- **US-17.7** As a user, I want context-aware keyboard shortcuts, so that different shortcuts apply in different views.
- **US-17.8** As a user, I want platform-aware shortcut display (Cmd on macOS), so that shortcuts display correctly.
- **US-17.9** As a user, I want hotkey configuration that persists to disk, so that my shortcuts survive app restarts.
- **US-17.10** As a user, I want a log capture toggle via hotkey, so that I can enable detailed logging without restarting.

## 18. Window Management & Platform

- **US-18.1** As a user, I want the Script Kit window to remember its position, so that it appears where I last placed it.
- **US-18.2** As a user, I want to reset all window positions to defaults, so that I can fix misplaced windows.
- **US-18.3** As a user, I want the app to run as a tray/menu bar app, so that it stays available without cluttering the Dock.
- **US-18.4** As a user, I want the prompt to move to the active Space/desktop, so that it always appears where I'm working.
- **US-18.5** As a user, I want the prompt to dismiss instantly (no animation), so that it feels snappy.
- **US-18.6** As a user, I want to toggle pin mode (Cmd+Shift+P) to keep the window open on blur, so that I can interact without it dismissing.
- **US-18.7** As a user, I want frontmost app tracking, so that Script Kit knows which application is currently active.

## 19. Application Launcher

- **US-19.1** As a user, I want installed macOS applications to appear in the main search results, so that I can launch any app from the prompt.
- **US-19.2** As a user, I want application results to show app icons, so that I can quickly identify apps visually.

## 20. Protocol & Script SDK

- **US-20.1** As a script developer, I want a bidirectional JSONL protocol between my scripts and the app, so that scripts can control the UI.
- **US-20.2** As a script developer, I want a Hello/HelloAck handshake with capability negotiation, so that old scripts work with new app versions.
- **US-20.3** As a script developer, I want to receive `update` events as the user types or navigates, so that I can build reactive UIs.
- **US-20.4** As a script developer, I want to receive `submit` events when the user confirms, so that I can process their selection.
- **US-20.5** As a script developer, I want to send `exit` to terminate the session, so that the prompt closes cleanly.
- **US-20.6** As a script developer, I want to show HUD notifications for lightweight confirmations, so that the user sees feedback without a modal.
- **US-20.7** As a script developer, I want to show Toast notifications for errors and warnings, so that important messages get user attention.
- **US-20.8** As a script developer, I want to control window size, position, and visibility from scripts, so that I can build custom layouts.
- **US-20.9** As a script developer, I want to use `setHint()`, `setPlaceholder()`, and `setFooter()` to customize prompt text, so that I can guide users.
- **US-20.10** As a script developer, I want to show a loading/progress indicator from scripts, so that users know work is in progress.
- **US-20.11** As a script developer, I want to use semantic IDs for stable choice identification, so that item identity persists across re-renders.
- **US-20.12** As a script developer, I want to use audio feedback (beep, say) in scripts, so that I can provide multi-sensory feedback.
- **US-20.13** As a script developer, I want to query selected text from the frontmost app, so that I can work with user selections.
- **US-20.14** As a script developer, I want to query active window information, so that I can adapt to the current application context.
- **US-20.15** As a script developer, I want clipboard access and manipulation APIs, so that I can read/write clipboard content.
- **US-20.16** As a script developer, I want mouse and keyboard simulation APIs, so that I can automate user interactions.
- **US-20.17** As a script developer, I want script metadata (name, description, icon, tags) to control how scripts appear in the app.
- **US-20.18** As a script developer, I want scheduled/delayed script execution, so that I can set up time-based automations.

## 21. AI SDK Integration

- **US-21.1** As a script developer, I want to use `ai.chat()` in the SDK to stream AI responses into the chat prompt, so that I can build AI-powered scripts.
- **US-21.2** As a script developer, I want to configure AI providers (OpenAI, Anthropic, Vercel) per-script, so that different scripts can use different models.
- **US-21.3** As a script developer, I want AI conversation history to persist in storage, so that users can resume chats.

## 22. Kit Store & Extensions

- **US-22.1** As a user, I want to browse the Kit Store to discover community scripts and kits, so that I can extend Script Kit's functionality.
- **US-22.2** As a user, I want to manage installed kits, so that I can update or remove extensions.
- **US-22.3** As a user, I want to update all installed kits at once, so that I stay on the latest versions.

## 23. Scriptlet Bundles

- **US-23.1** As a developer, I want to create scriptlet bundles with YAML frontmatter, so that I can organize related automation tasks.
- **US-23.2** As a developer, I want to edit scriptlets in my preferred editor, so that I can modify and improve them.
- **US-23.3** As a developer, I want to reveal scriptlets in Finder and copy their paths, so that I can access source files directly.
- **US-23.4** As a developer, I want to execute dynamic scriptlet actions, so that I can use actions generated by scriptlets.

## 24. Snippets & Text Expansion

- **US-24.1** As a developer, I want to use VSCode snippet syntax (tabstops, placeholders, choices), so that I can create reusable code templates.
- **US-24.2** As a developer, I want tabstop navigation ($1, $2, $0) and choice options (${1|opt1,opt2|}), so that users can quickly fill in template fields.

## 25. File Watching & Background Scripts

- **US-25.1** As a script developer, I want scripts to be triggered by file system changes (watch mode), so that I can build reactive automations.
- **US-25.2** As a user, I want background scripts to run persistently, so that watchers and schedules stay active.
- **US-25.3** As a user, I want file watching with debouncing and storm protection, so that scripts reload reliably when files change.
- **US-25.4** As a user, I want config/theme/script auto-reload on file changes, so that I can iterate without restarting the app.

## 26. Actions Dialog

- **US-26.1** As a user, I want to trigger an actions menu with Cmd+K in most views, so that I can access additional options.
- **US-26.2** As a user, I want to search for actions by name, so that I can find the specific action I need.
- **US-26.3** As a user, I want the actions menu to show available keyboard shortcuts, so that I can learn shortcuts for future use.
- **US-26.4** As a user, I want a window-level actions menu for global operations, so that I can perform app-wide actions.

## 27. External Stdin Commands (Automation API)

- **US-27.1** As a developer, I want to control Script Kit via stdin JSONL commands (show, hide, run, setFilter), so that I can automate it from external tools.
- **US-27.2** As a developer, I want to capture a screenshot of the Script Kit window via `captureWindow`, so that I can use it in automated testing.
- **US-27.3** As a developer, I want to simulate key presses via `simulateKey`, so that I can automate UI interactions for testing.
- **US-27.4** As a developer, I want to trigger built-in features via `triggerBuiltin`, so that I can programmatically open Clipboard History, Emoji Picker, etc.

## 28. Error Handling & Feedback

- **US-28.1** As a user, I want clear error toasts when something goes wrong, so that I understand what failed and why.
- **US-28.2** As a user, I want HUD confirmations for quick actions (copied, saved, pinned), so that I know the action succeeded.
- **US-28.3** As a user, I want view transitions to serve as implicit feedback, so that the app doesn't show unnecessary notifications.

## 29. Confirmation Dialogs

- **US-29.1** As a user, I want destructive actions (empty trash, shut down) to show a confirmation dialog, so that I don't accidentally trigger irreversible operations.
- **US-29.2** As a script developer, I want to show custom confirmation dialogs with configurable button text, so that I can build safe workflows.

## 30. Configuration & Settings

- **US-30.1** As a user, I want to enable/disable built-in features (Clipboard History, Window Switcher) via config, so that I can customize which features appear.
- **US-30.2** As a user, I want to exclude specific commands from frecency/suggested tracking, so that one-off commands don't pollute my suggestions.
- **US-30.3** As a user, I want Script Kit to auto-start on login, so that it's always available.
- **US-30.4** As a user, I want application settings persisted to disk, so that my preferences survive app restarts.
- **US-30.5** As a user, I want to customize layout settings (padding, scale, font sizes), so that I can optimize the UI for my screen.

## 31. Tray Menu

- **US-31.1** As a user, I want a menu bar tray icon, so that I can access Script Kit when the prompt is hidden.
- **US-31.2** As a user, I want the tray menu to show options like Open, Preferences, and Quit, so that I have basic controls always available.

## 32. Webcam

- **US-32.1** As a user, I want to preview my webcam live with optional mirroring, so that I can set up and verify what will be captured.
- **US-32.2** As a user, I want to capture a frame with a button press, so that I can take a snapshot for scripts.
- **US-32.3** As a user, I want the webcam to release when I close the prompt, so that other apps can use the camera.
- **US-32.4** As a user, I want zero-copy webcam streaming via Metal textures, so that I get high-performance camera preview.

## 33. Legacy & Migration

- **US-33.1** As a developer, I want support for legacy ~/.kenv script directory with migration, so that existing users can upgrade smoothly.
- **US-33.2** As a developer, I want protocol negotiation for backward compatibility, so that old SDK versions still work.

## 34. Logging & Observability

- **US-34.1** As a developer, I want JSONL-formatted logs with `SCRIPT_KIT_AI_LOG=1` compact mode, so that I can debug issues efficiently.
- **US-34.2** As a developer, I want correlation IDs in logs, so that I can trace a request across the protocol boundary.

## 35. Reusable UI Components (Developer)

- **US-35.1** As a developer, I want Button components with variants (Primary, Ghost, Icon), so that I can build consistent UIs.
- **US-35.2** As a developer, I want Toast notification components (Success, Warning, Error, Info), so that I can show user feedback.
- **US-35.3** As a developer, I want Scrollbar, TextField, TextArea, and Checkbox components, so that I can build forms efficiently.
- **US-35.4** As a developer, I want PromptHeader, PromptFooter, and PromptContainer components, so that prompts have consistent layout.
- **US-35.5** As a developer, I want UnifiedListItem components for searchable, selectable lists, so that items display consistently.

# ChatGPT macOS Desktop App: Features and UX Notes

Research date: 2026-02-01
Scope: global hotkey, screenshot capture, conversation management, keyboard shortcuts, streaming UX.

## Global hotkey and launcher (Chat Bar)
- Default global hotkey is Option + Space to open the Chat Bar; the shortcut can be changed in Settings. (R1)
- The Chat Bar can be opened from the menu bar and dragged anywhere on the desktop. (R1)
- The Chat Bar is a lightweight launcher to start a new conversation, attach files/photos/screenshots, and submit with Return. (R1)
- Companion window uses Option + Space to start a new chat, refocus an open companion window, or reopen the last companion window, and the shortcut is configurable in Settings. (R3)

## Screenshot capture and screen sharing
- Screenshot tool is accessed from the Chat Bar plus icon; you can hover to select an open window or type an app name in the launcher to capture that window. (R2)
- Screen recording permission is required; enabling it may require restarting the app. (R2)
- Release notes add the ability to choose which display to screenshot. (R3)
- Keyboard shortcuts for screen sharing: Command + Shift + 1 (share main desktop) and Command + Shift + 2 (share active window). (R3)

## Conversation management
- Companion window stays in front of other windows and can pop out a previous conversation; its position, reset time, and shortcut are customizable. (R3)
- Conversation search across history is available via the search bar. (R3)
- Find text within a conversation via Command + F. (R3)
- Draft messages can be restored after interruption. (R3)
- Chat renaming in the sidebar is a supported flow (release notes mention a fix). (R3)
- Conversation scrolling performance has been improved. (R3)

## Keyboard shortcuts (documented)
- Option + Space: open Chat Bar. (R1)
- Option + Space: start new chat, refocus companion window, or reopen last companion window. (R3)
- Command + . : stop streaming responses. (R3)
- Command + Shift + 1: share main desktop. (R3)
- Command + Shift + 2: share active window. (R3)
- Command + F: find text within a conversation. (R3)
- Return: submit prompt in the Chat Bar. (R1)

## Streaming UX
- Stop streaming via Command + . for explicit user control. (R3)
- Release notes call out improved performance when streaming responses. (R3)
- Response animation includes a smooth text fade-in effect. (R3)
- Notifications can alert you when the assistant finishes replying in the background. (R3)

## UX takeaways (inference)
- Inference: The app prioritizes instant access with a global hotkey and a lightweight launcher for starting chats and attachments. (R1, R4)
- Inference: Conversation context is treated as persistent and multitaskable via companion windows and conversation search. (R3)
- Inference: Streaming UX emphasizes user control and feedback through stop shortcuts, animation polish, and background completion notifications. (R3)

## Sources
- R1: https://help.openai.com/en/articles/9295241-how-to-launch-the-chat-bar
- R2: https://help.openai.com/en/articles/9295245-chatgpt-macos-app-screenshot-tool
- R3: https://help.openai.com/en/articles/9703738-desktop-app-release-notes
- R4: https://openai.com/chatgpt/desktop/ (redirects to https://chatgpt.com/features/desktop)

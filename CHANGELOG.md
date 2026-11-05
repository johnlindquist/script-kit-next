# Script Kit changelog

## 0.1.33 · Model engines stop with Kit

[Release notes and installation checks](docs/releases/v0.1.33.md).

If Kit is force-quit on macOS, the Goose, ghost-text, and embedding processes it started stop without waiting for another Kit launch. Tools that remain in the engine’s process group stop with it. Process Manager distinguishes the engine PID from the group controlled by Stop. This does not scan for or kill unrelated copies of those programs. [Issue #56](https://github.com/johnlindquist/script-kit-next/issues/56).

When Jev is off, Return names the selected action immediately. A delayed file search doesn’t change the caption, and agents inspecting the launcher see the same label as the person at the keyboard.

Fresh workspaces include the **Hello World** starter. An eligible v1 workspace imports before Kit opens its first log file.

Deleting a note during a search-index rebuild no longer interrupts indexing. The CLI keeps saved credentials paired with their local server, and a malformed connection file can be diagnosed without printing its contents.

## 0.1.31 and 0.1.32 · No public installers

Neither version produced a public Mac installer.

## 0.1.30 · Source tag only

The app was signed and notarized, but its final archive check did not finish. No Mac installer was published.

## 0.1.29 · Source tag only

No Mac installer was published for this tag. Use the [GitHub Releases page](https://github.com/johnlindquist/script-kit-next/releases) to find a signed download.

## 0.1.28 · Reliability (source tag only)

[Source-tag notes](docs/releases/v0.1.28.md). The 0.1.20 through 0.1.28 tags did not publish Mac installers. For a downloadable app, see [GitHub Releases](https://github.com/johnlindquist/script-kit-next/releases).

Changes in this source include checking the selected browser tab or window before acting, preserving personal files during workspace setup, and distinguishing microphone or Accessibility permission from a working input device or cursor.

New chats respect changes to a profile’s tool permissions and leave an earlier reply running. When you dictate into a pinned launcher, **Stays open** appears once beside the input.

## October 15–31, 2026 · One window, clearer targets

**[Read the illustrated changelog](https://merry-flint-85wx.here.now/)** · [Offline edition](docs/changelog/2026-10/index.html) · [Try the sample desktops](https://proud-unity-7aj4.here.now/)

Send a selected error to one named agent, then return to your work until the reply is ready. The interactive examples also show window placement, selection rewriting, and finding a previously visited page using sample data.

| Change | Try it |
| --- | --- |
| Launcher subjects | On an empty launcher, Tab between selected text, a recent copy, the front window and the last run. Space stages the subject. [Keyboard guide](docs/guides/main-menu-input.md) |
| Open browser tabs | Type part of a tab title. A matching tab keeps a shortlist spot, even among command or note matches. Named-tab focus refuses ambiguity. |
| Cursor dictation | Hold the main shortcut to dictate at the original field; ⇧Esc switches to editable Kit input. Two taps open Today. [Gesture map](docs/specs/gesture-grammar.md) |
| Recording recovery | Cancel before dispatch, then restore a private draft. Screen spelling can use local Accessibility text; window OCR is opt-in. [Dictation](docs/guides/dictation.md) |
| Script forms | An invalid submission focuses the field to fix without losing the other fields. Return inserts a newline in multiline fields; ⌘↵ submits. [Form behavior](docs/audits/form-correction.md) |
| Named agents | Share only a chosen passage with a registered recipient and edit the reply later. Operators can grant quiet note saves or reviewed command revisions separately. [Agent bridge](docs/guides/external-agents.md) |
| Intent and filenames | Name search stays local. Opt into [remote sentence readings](docs/guides/jev-intent.md) separately. On supported Save sheets, the [filename helper](docs/guides/desktop-filename.md) previews a name and leaves Save to you. |

Also in this update: Markdown note links and exports, clipboard actions for links/colors/files, safer file copy and undo, window tiling and layout restore, webcam countdown and review, chat approval controls and MCP tools, and Metal dictation with CPU fallback.

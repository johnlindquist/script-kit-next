# Type, talk, and Today

The **main shortcut** is configurable. With `⌘;`, hold both keys to talk; either release order finishes the recording. Releasing Command first is **not** a command to redirect the text.

| Starting point | Gesture | Result |
|---|---|---|
| Another app owns the keyboard; Kit may be hidden or pinned behind it | Tap | Open the launcher for typing on the first release, without waiting for a possible second tap. A stale Today view isn't the launcher. |
| Kit already has focus | Tap | Toggle the window closed. Typing commits the first tap. |
| Another app owns the keyboard | Hold | Dictate for its original field. Kit may show the listening line without becoming key. Release pastes; it does not ask Jev whether the words sound like an instruction. |
| The launcher, a note editor, or a supported Kit prompt has focus | Hold | Insert at the captured selection. A changed destination is not permission to replace another field. |
| From inside or outside Kit | Tap, tap | Open Today in the Markdown note editor. |
| From inside or outside Kit | Tap, hold | Open Today and dictate into that editor. This is two presses, not three. |
| Empty launcher | ⌘↵ | Open a clean chat. |

The default hold threshold is 250 ms. The second press must **begin** within 300 ms of the first release. Key repeat doesn't count. Opening Kit on *release*, rather than on initial key-down, keeps an external hold from stealing its own destination.

## Where the words go

- **Use in Kit:** press ⇧Esc after listening starts, without releasing a held main shortcut. Press it again to switch back. Bare Shift does nothing. The switch is recording-only and configurable (`dictation.switchShortcut`; `off` disables it). The whole final transcript goes to the chosen destination. In Kit it's editable input; press ↵ separately to run a reading. Remote intent reading still requires opt-in. The model is told when command text came from speech; names, negation and literal values are not a license to guess.
- **Start in Kit:** bind **Dictate into Kit**, or choose **In Script Kit** from **Dictation Starting Destination**. Both produce editable input, not a submitted command.
- **Hands-free:** quick-tap the configured dictation shortcut to start, then tap it again to stop. Existing explicit target preferences and dedicated chat/note actions remain separate from the main shortcut's contextual hold.
- **Long passages:** length never changes the destination. Bind **Review Dictation in Markdown** (`builtin/review-dictation-markdown`) to choose an editable local draft instead of pasting. This does not create a Today entry. A failed or stale direct delivery retains the latest withheld draft in memory for that same action. Review before leaving an unsaved draft.
- **Controls:** `builtin/use-dictation-in-kit` toggles the destination without ending the recording. Both actions are available to shortcut bindings and the existing builtin-command interface. `getState`'s one-window snapshot exposes `cursorDictation`: active generation, route, stopping flag and draft availability, not audio or transcript contents.
- Ordinary external delivery requires an identifiable, non-secure accessible field. It checks the captured process, field and selection again before dispatching a process-targeted paste. A changed or unidentifiable field is not replaced with whatever is now focused. The check and OS dispatch are not an atomic operation.

The [dictation recovery contract](dictation-recovery.md) defines the target behavior of Escape, Restore, Retry, Paste and Forget, including which shortcuts stay local to Kit.

## Focus ownership map

| Owner | Why it takes focus | Shortcut rule |
|---|---|---|
| `src/main_sections/gesture_routing.rs`, `src/hotkeys/gesture.rs` | Main global key-down/up, including a visible but non-key panel | Classify the physical gesture separately from visibility. Pinning a note is not ownership of the keyboard. A tap normalizes the launcher; a hold does not call the activating show path. |
| `src/main_sections/window_visibility.rs` | Hidden-window show, deferred native activation, focus after layout | Stale deferred shows must not revive a window hidden since the request. A real Today open retains editor focus rather than focusing the launcher input. |
| `src/one_window/notes.rs`, `src/main_sections/day_page_notes.rs` | An explicit Notes hotkey, tray item, deeplink or SDK open | Keep these targeted opens. A later launcher summon supersedes a waiting note; flush dirty Day Page text before switching. An outstanding script prompt is not silently reset. |
| `src/main_entry/app_run_setup.rs`, `src/window_orchestrator/` | Script prompts, tray/SDK shows, chat entries and requested window activation | Preserve their view ownership. `ShowMain { activate_app: false }` still makes the panel **key**; it is not suitable for external cursor dictation. |
| `src/platform/visibility_focus.rs`, `src/app_execute/cursor_dictation.rs` | Listening indicator and explicit transition into Kit | The indicator uses the non-key background reveal. Direct dictation neither raises the old app nor invokes the legacy delayed refocus-and-paste path. |
| `src/main_sections/render_impl.rs` | File-search drag/click focus repair | This is a response to a pointer action in Kit, not a main-shortcut routing rule. |

Microphone permission, transcription quality, native Carbon key-up order and delivery in an individual third-party app require a real, authorized desktop to qualify. Hidden renderer fixtures supply synthetic final text; they do not record audio or paste into the host desktop.

# Notes Window: New Note + Switcher (Single Window)

**Type:** Feature Diagram
**Last Updated:** 2026-01-02
**Related Files:**
- `src/notes/window.rs`
- `src/notes/actions_panel.rs`
- `src/notes/browse_panel.rs`
- `src/notes/storage.rs`
- `src/notes/model.rs`
- `src/main.rs`
- `src/hotkeys.rs`

## Purpose

Provide a single notes window that lets users create a new note or switch notes instantly without opening additional windows, while preserving their work and providing clear recovery on failures.

## Diagram

```mermaid
graph TD
    subgraph "Front-Stage (User Experience)"
        Hotkey[User presses Notes hotkey] --> Window[Notes window focuses ⚡ instant access]
        Window --> Action[User triggers New Note or Cmd+P]
        Action --> NewNoteUI[Editor clears ✅ blank slate]
        Action --> SwitcherUI[Switcher list appears ⏱️ focused search]
        SwitcherUI --> Selection[User selects a note]
        Selection --> NoteLoaded[Selected note appears ✅ continue editing]
    end

    subgraph "Back-Stage (Implementation)"
        Window --> OpenWindow[open_notes_window() 🛡️ single window]
        Action --> ActionsPanel[Actions panel routing 🎯 consistent shortcuts]
        ActionsPanel --> CreateNote[create_note() 💾 persists new note]
        CreateNote --> Storage[(SQLite notes DB 💾 preserves content)]
        SwitcherUI --> BrowsePanel[BrowsePanel filter ⚡ fast search]
        BrowsePanel --> NotesCache[NotesApp cache 📊 keeps list current]
        BrowsePanel --> Search[FTS search 🎯 accurate filtering]
        NotesCache --> SelectNote[select_note() 🎯 swaps editor content]
        SelectNote --> EditorState[editor_state update ⚡ immediate content change]
    end

    Storage -->|Saved| NewNoteUI
    Storage -->|Error| SaveError[Save error message 🔄 user can retry]
    Search -->|Error| SearchFallback[Show unfiltered list 🔄 still usable]
    SaveError --> Window
    SearchFallback --> SwitcherUI
```

## Key Insights

- **Immediate flow:** The same window and editor state update instantly for fast note creation and switching.
- **Data safety:** Persistent storage and error recovery prevent loss of user content.
- **Single window guarantee:** Singleton window handling avoids multiple note windows and preserves context.

## Change History

- **2026-01-02:** Initial creation

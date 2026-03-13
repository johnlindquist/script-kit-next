# Script Kit GPUI - User Story Map

## High-Level Feature Map

```mermaid
mindmap
  root((Script Kit GPUI))
    Main Prompt
      Fuzzy Search
      App Launcher
      Built-in Commands
      Frecency Ranking
      Menu Bar Actions
      Actions Panel Cmd+K
      Inline Calculator
      Input History
    Script Management
      Create from Template
      Create Scriptlet Bundle
      Generate with AI
      Run Scripts
      Global Hotkeys
      Favorites
      Process Manager
      Edit / Delete / Reload
    Script SDK Prompts
      arg - Searchable List
      div - HTML/Markdown
      editor - Code Editor
      term - Terminal
      form/fields - Multi-field
      select - Multi-select
      path - File Picker
      env - Secret Input
      drop - Drag and Drop
      template - Template Editor
      chat - Conversational
      confirm - Yes/No Dialog
      mini/micro - Compact
      webcam - Camera
    Built-in Features
      Clipboard History
      Emoji Picker
      Notes
      File Search
      Window Switcher
      Quicklinks
      Scratch Pad
      Quick Terminal
    AI Chat
      Multi-provider BYOK
      Screenshot to AI
      Selected Text to AI
      Browser Tab to AI
      Generate Script
      AI Presets
      Streaming Responses
      Full-text Search
    System Commands
      Power Management
      Volume Control
      Dark Mode Toggle
      Do Not Disturb
      System Settings
      Permissions
    Platform
      Non-activating Panel
      Tray Menu
      Window Positions
      Vibrancy
      File Watching
      Background Scripts
      Pin Mode
    Kit Store
      Browse Kits
      Install/Manage
      Update All
```

## User Journey: Main Prompt Flow

```mermaid
flowchart TD
    A[User presses global hotkey] --> B[Main Prompt appears<br>non-activating panel]
    B --> C{User types in search}
    C -->|Script match| D[Run selected script]
    C -->|App match| E[Launch application]
    C -->|Built-in match| F{Which built-in?}
    C -->|Math expression| G[Show inline calculation]
    C -->|No input| H[Browse frecency-ranked suggestions]

    F --> F1[Clipboard History]
    F --> F2[Emoji Picker]
    F --> F3[AI Chat]
    F --> F4[Notes]
    F --> F5[File Search]
    F --> F6[Window Switcher]
    F --> F7[System Command]
    F --> F8[Settings/Theme]
    F --> F9[Quicklinks]
    F --> F10[Scratch Pad]
    F --> F11[Quick Terminal]

    D --> I{Script shows prompt}
    I --> I1[arg - List prompt]
    I --> I2[div - HTML content]
    I --> I3[editor - Code editor]
    I --> I4[term - Terminal]
    I --> I5[form - Multi-field form]
    I --> I6[select - Multi-select]
    I --> I7[path - File picker]
    I --> I8[env - Secret input]
    I --> I9[drop - Drag & drop]
    I --> I10[chat - Conversational]
    I --> I11[confirm - Yes/No]
    I --> I12[webcam - Camera]
    I --> I13[template - Template editor]

    I1 --> J[User submits value]
    I2 --> J
    I3 --> J
    I5 --> J
    I6 --> J
    I7 --> J
    I8 --> J
    I9 --> J
    I10 --> J
    I11 --> J
    I12 --> J
    I13 --> J
    J --> K{More prompts?}
    K -->|Yes| I
    K -->|No| L[Script completes]

    H --> C
    G -->|Enter| M[Copy result to clipboard]
    B -->|Escape| N[Dismiss prompt<br>focus returns to prev app]
    B -->|Cmd+K| O[Actions Panel]
    O --> P[Execute contextual action]
```

## AI Chat User Journey

```mermaid
flowchart TD
    A[Open AI Chat] --> B{Input method}
    B -->|Type message| C[Send text to AI]
    B -->|Send screen| D[Capture full screenshot]
    B -->|Send window| E[Capture focused window]
    B -->|Send selection| F[Get selected text]
    B -->|Send browser tab| G[Get tab URL]
    B -->|Send screen area| H[Area selection tool]
    B -->|Attach clipboard| I2[Attach clipboard content]

    D --> C
    E --> C
    F --> C
    G --> C
    H --> C
    I2 --> C

    C --> I[AI streams response<br>with markdown + syntax highlighting]
    I --> J{User action}
    J -->|Continue conversation| B
    J -->|New conversation| K[Clear context]
    K --> B
    J -->|Generate script| L[AI creates Script Kit script]
    L --> M[Creation feedback]
    J -->|Search conversations| N[Full-text search]
    N --> O[Open past conversation]
    O --> B
    J -->|Switch model| P[Select from footer dropdown]
    P --> B
```

## Script SDK Protocol Flow

```mermaid
sequenceDiagram
    participant S as Script (Bun/TS)
    participant A as App (Rust/GPUI)
    participant U as User

    Note over S,A: Session Start
    S->>A: hello (protocol, sdkVersion, capabilities)
    A->>S: helloAck (protocol, capabilities)

    Note over S,A: Prompt Loop
    S->>A: arg (id, placeholder, choices, actions?)
    A->>U: Show searchable list prompt
    U->>A: Types filter text
    A->>S: update (filter, selectedIndex)
    U->>A: Presses Cmd+K
    A->>U: Show actions panel
    U->>A: Presses Enter on choice
    A->>S: submit (selected value)

    S->>A: div (id, html, containerClasses?)
    A->>U: Show rich HTML content
    U->>A: Presses Escape
    A->>S: submit (null)

    S->>A: editor (id, content, language, template?)
    A->>U: Show code editor with highlighting
    U->>A: Edits content, Cmd+S
    A->>S: submit (edited content)

    S->>A: confirm (id, message, confirmText, cancelText)
    A->>U: Show confirmation dialog
    U->>A: Clicks Confirm
    A->>S: submit (true)

    Note over S,A: Feedback & Control
    S->>A: showHud (text)
    A->>U: Flash HUD overlay
    S->>A: toast (message, variant)
    A->>U: Show toast notification
    S->>A: setInput (text)
    A->>U: Update input field

    Note over S,A: Session End
    S->>A: exit (code?, message?)
    A->>U: Dismiss prompt window
```

## Built-in Features Architecture

```mermaid
flowchart LR
    subgraph Search["Main Search"]
        Input[Search Input]
        Results[Filtered Results]
        Frecency[Frecency Ranking]
    end

    subgraph BuiltIns["Built-in Features"]
        CH[Clipboard History]
        EP[Emoji Picker]
        WS[Window Switcher]
        FS[File Search]
        SP[Scratch Pad]
        QT[Quick Terminal]
        PM[Process Manager]
        QL[Quicklinks]
        FAV[Favorites]
        WC[Webcam]
    end

    subgraph SystemCmds["System Commands"]
        PWR[Power: Sleep/Restart/Shutdown]
        VOL[Volume: 0-100% / Mute]
        UI[UI: Dark Mode / Desktop / Mission Control]
        DND[Do Not Disturb]
        SET[System Settings Panes x8]
        PERM[Permissions]
    end

    subgraph Windows["Secondary Windows"]
        AI[AI Chat Window]
        NT[Notes Window]
    end

    subgraph ScriptPrompts["Script Prompts"]
        ARG[arg]
        DIV[div]
        EDT[editor]
        TRM[term]
        FRM[form/fields]
        SEL[select]
        PTH[path]
        ENV[env]
        DRP[drop]
        TPL[template]
        CHT[chat]
        CNF[confirm]
        WEB[webcam]
        MINI[mini/micro]
    end

    Input --> Frecency
    Frecency --> Results
    Results --> BuiltIns
    Results --> SystemCmds
    Results --> Windows
    Results --> ScriptPrompts
```

## Clipboard History Flow

```mermaid
flowchart TD
    A[System clipboard change detected] --> B[Store in history cache<br>text / image / file ref]
    B --> C[User opens Clipboard History]
    C --> D[Show chronological list<br>with image previews]
    D --> E{User action}
    E -->|Search/Filter| F[Filter clipboard items]
    F --> D
    E -->|Select + Enter| G[Paste to frontmost app]
    E -->|Pin item| H[Mark as pinned/favorite]
    H --> D
    E -->|Delete item| I[Remove from history]
    I --> D
    E -->|Paste Sequentially| J[Enter sequential paste mode]
    J --> K[Paste next item on each trigger]
    K --> L{More items?}
    L -->|Yes| K
    L -->|No| M[Exit sequential mode]
    E -->|Quick Look| N[Preview with Quick Look]
    E -->|OCR| O[Extract text from image]
    E -->|Share| P[System share sheet]
    E -->|Save as file| Q[Save to disk]
    E -->|Send to AI| R[Attach to AI Chat]
    E -->|Save as snippet| S[Save as code snippet]
```

## Notes Feature Flow

```mermaid
flowchart TD
    A{Entry point} -->|Open Notes / Global hotkey| B[Notes window with sidebar list]
    A -->|New Note| C[Create blank note]
    A -->|Quick Capture| D[Inline capture prompt]
    A -->|Search Notes| E[Search interface]

    B --> F[Select note from sidebar]
    F --> G[Edit note with Markdown<br>syntax highlighting]
    G --> H[Auto-save to disk]

    C --> G
    D --> I[Save captured text as note]
    I --> B

    E --> F

    G --> J{Note actions}
    J -->|Delete| K[Soft delete note]
    J -->|Export| L[Export as text/md/HTML]
    K --> M{Recover?}
    M -->|Yes| N[Restore from trash]
    N --> B
    M -->|No| B

    G --> O[Word/char count display]
    B --> P[Auto-resize window to content]
```

## Platform & Window Architecture

```mermaid
flowchart TD
    subgraph Platform["macOS Platform Layer"]
        TM[Tray Menu Icon]
        GH[Global Hotkeys]
        ACC[Accessibility Permission]
        FW[File Watcher<br>debounce + storm protection]
        FT[Frontmost App Tracking]
    end

    subgraph WindowMgmt["Window Management"]
        NP[Non-activating NSPanel<br>WindowKind::PopUp]
        POS[Window Position Memory]
        VIB[Vibrancy/Translucency]
        FLT[Floating Level 3]
        SPC[Move to Active Space]
        PIN[Pin Mode Cmd+Shift+P]
        ANIM[No Animation Dismiss]
    end

    subgraph AppLifecycle["App Lifecycle"]
        TRAY[Accessory App<br>No Dock icon]
        AUTO[Auto-start on Login]
        BG[Background Script Runner]
        PID[PID File Tracking]
    end

    TM --> NP
    GH --> NP
    NP --> POS
    NP --> VIB
    NP --> FLT
    NP --> SPC
    NP --> PIN
    NP --> ANIM
    TRAY --> TM
    BG --> FW
    BG --> PID
    FT --> NP
```

## Script Creation Flow

```mermaid
flowchart TD
    A{Creation method} -->|New Script Template| B[Show naming prompt<br>with kebab-case preview]
    A -->|New Scriptlet Bundle| C[Show naming prompt<br>with YAML frontmatter]
    A -->|Generate with AI| D[AI generates from<br>natural language]

    B --> E[Validate name<br>check duplicates]
    C --> F[Validate name<br>check duplicates]
    D --> G[AI writes script content]

    E --> H[Create .ts file from template]
    F --> I[Create .md bundle]
    G --> J[Create generated .ts file]

    H --> K[Show creation feedback<br>with file path]
    I --> K
    J --> K

    K --> L{User action}
    L -->|Edit| M[Open in VS Code]
    L -->|Open in Finder| N[Reveal in Finder]
    L -->|Run| O[Execute the new script]
    L -->|Dismiss| P[Return to main prompt]
```

## Hotkeys & Shortcuts Architecture

```mermaid
flowchart TD
    subgraph GlobalHotkeys["Global Hotkeys"]
        MAIN[Main Prompt Hotkey]
        NOTES[Notes Window Hotkey]
        AICHAT[AI Chat Window Hotkey]
        SCRIPT[Per-Script Hotkeys]
        LOG[Log Capture Toggle]
    end

    subgraph InAppShortcuts["In-App Shortcuts"]
        CMDK[Cmd+K → Actions Panel]
        ESC[Escape → Dismiss]
        ENTER[Enter → Submit/Run]
        TAB[Tab → Next Field]
        ARROWS[Arrow Keys → Navigate]
        CMDSP[Cmd+Space → Toggle Select]
        PINS[Cmd+Shift+P → Pin Mode]
    end

    subgraph ShortcutMgmt["Shortcut Management"]
        REC[Interactive Recorder]
        CONF[Conflict Detection]
        ALIAS[Command Aliases]
        PERSIST[Persist to config.ts]
        CTX[Context-aware Dispatch]
    end

    GlobalHotkeys --> PERSIST
    InAppShortcuts --> CTX
    ShortcutMgmt --> PERSIST
    REC --> CONF
```

## Configuration & Settings

```mermaid
flowchart TD
    subgraph Config["User Configuration"]
        BC[Built-in Config<br>enable/disable features]
        API[API Keys<br>Vercel / OpenAI / Anthropic]
        TH[Theme Selection<br>with live preview]
        WP[Window Positions<br>reset to defaults]
        FR[Frecency Data<br>clear suggested]
        HK[Hotkey Mappings<br>interactive recorder]
        LY[Layout Settings<br>padding / scale / fonts]
    end

    subgraph BuiltInConfig["Feature Toggles"]
        BC --> CH_ON[Clipboard History]
        BC --> WS_ON[Window Switcher]
        BC --> AL_ON[App Launcher]
    end

    subgraph APIConfig["AI Provider Config"]
        API --> VK[Vercel AI Gateway Key]
        API --> OK[OpenAI API Key]
        API --> AK[Anthropic API Key]
    end

    subgraph ThemeConfig["Appearance"]
        TH --> TP[Browse + Search Themes]
        TH --> PREV[Live Preview]
        TH --> OPAC[Opacity Adjustment]
        TH --> DM[Dark Mode Toggle]
        TH --> CROSS[Cross-window Consistency]
    end

    subgraph Persistence["Persistence"]
        DISK[~/.scriptkit/kit/config.ts]
        KEYS[System Keyring<br>encrypted secrets]
        POS[Window Position Store]
    end

    Config --> DISK
    API --> KEYS
    WP --> POS
```

## External Automation API

```mermaid
sequenceDiagram
    participant EXT as External Tool
    participant APP as Script Kit (stdin)
    participant WIN as App Window

    Note over EXT,WIN: Control via JSONL stdin
    EXT->>APP: {"type":"show"}
    APP->>WIN: Show main prompt

    EXT->>APP: {"type":"setFilter","text":"clipboard"}
    APP->>WIN: Filter search results

    EXT->>APP: {"type":"triggerBuiltin","name":"clipboardHistory"}
    APP->>WIN: Open Clipboard History view

    EXT->>APP: {"type":"simulateKey","key":"enter","modifiers":["cmd"]}
    APP->>WIN: Simulate Cmd+Enter

    EXT->>APP: {"type":"run","path":"/scripts/hello.ts"}
    APP->>WIN: Execute script

    EXT->>APP: {"type":"captureWindow","path":"screenshot.png"}
    APP-->>EXT: Save window screenshot to file

    EXT->>APP: {"type":"hide"}
    APP->>WIN: Hide main prompt
```

## Kit Store & Extensions

```mermaid
flowchart TD
    A[Browse Kit Store] --> B[Discover community kits]
    B --> C{User action}
    C -->|Install| D[Download and install kit]
    D --> E[Kit scripts appear in search]

    F[Manage Installed Kits] --> G[View installed kits list]
    G --> H{User action}
    H -->|Update one| I[Update selected kit]
    H -->|Remove| J[Uninstall kit]

    K[Update All Kits] --> L[Check all kits for updates]
    L --> M[Download latest versions]
    M --> E
```

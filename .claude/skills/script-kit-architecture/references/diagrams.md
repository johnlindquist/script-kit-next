# System Diagrams

Visual maps of how code modules connect. Use to understand impact before making changes.

## Architecture Overview

Module dependencies and data flow from user features to storage.

```mermaid
flowchart TD
    subgraph UserValue["User Value Layer"]
        UV1["Script Launcher"]
        UV2["Clipboard History"]
        UV3["App Launcher"]
        UV4["Notes"]
        UV5["AI Chat"]
        UV6["Text Expansion"]
        UV7["File Search"]
        UV8["Window Control"]
    end

    subgraph Entry["Entry Points"]
        main["main.rs"]
        app_impl["app_impl.rs"]
        lib["lib.rs"]
    end

    subgraph UI["User Interface"]
        subgraph Prompts["Prompts src/prompts/"]
            arg["ArgPrompt"]
            div["DivPrompt"]
            editor["editor.rs"]
            term["term_prompt.rs"]
            form["form_prompt.rs"]
        end
        subgraph Secondary["Secondary Windows"]
            notes["notes/"]
            ai_win["ai/"]
            actions["actions.rs"]
        end
    end

    subgraph Execution["Execution Layer"]
        executor["executor.rs"]
        protocol["protocol.rs"]
        scripts["scripts.rs"]
        scriptlets["scriptlets.rs"]
    end

    subgraph Platform["Platform Integration"]
        platform["platform.rs"]
        panel["panel.rs"]
        hotkeys["hotkeys.rs"]
        window_control["window_control.rs"]
    end

    subgraph Storage["Storage Layer"]
        clipboard["clipboard_history.rs"]
        frecency["frecency.rs"]
        config["config.rs"]
    end

    UV1 --> scripts
    UV2 --> clipboard
    UV3 --> scripts
    UV4 --> notes
    UV5 --> ai_win
    UV6 --> scriptlets
    UV7 --> scripts
    UV8 --> window_control

    main --> app_impl
    app_impl --> Prompts
    app_impl --> Secondary

    executor --> protocol
    executor --> scripts
    scripts --> scriptlets

    Prompts --> executor
    platform --> hotkeys
    platform --> panel
```

## Application State Machine

Valid states and transitions. Escape/submit usually returns to MainMenu.

```mermaid
stateDiagram-v2
    [*] --> Idle: App starts hidden

    Idle --> MainMenu: Hotkey trigger
    MainMenu --> Idle: Escape / blur

    MainMenu --> ArgPrompt: SDK arg() call
    MainMenu --> DivPrompt: SDK div() call
    MainMenu --> EditorPrompt: SDK editor() call
    MainMenu --> TermPrompt: SDK term() call
    MainMenu --> FormPrompt: SDK fields() call

    ArgPrompt --> MainMenu: Submit / Escape
    DivPrompt --> MainMenu: Submit / Escape
    EditorPrompt --> MainMenu: Submit / Escape
    TermPrompt --> MainMenu: Submit / Escape
    FormPrompt --> MainMenu: Submit / Escape

    MainMenu --> ActionsDialog: Action trigger
    ActionsDialog --> MainMenu: Close overlay

    Idle --> Notes: Notes hotkey
    Idle --> AI: AI hotkey
    Notes --> Idle: Close window
    AI --> Idle: Close window
```

## Script Execution Flow

JSONL protocol between Rust app and Bun/SDK.

```mermaid
sequenceDiagram
    participant User
    participant GPUI as GPUI App (main.rs)
    participant Handler as Prompt Handler
    participant Executor as Executor (executor.rs)
    participant Bun as Bun Process
    participant SDK as Kit SDK (kit-sdk.ts)
    participant Script as User Script

    User->>GPUI: Select script from list
    GPUI->>Executor: run_script(path)
    Executor->>Bun: spawn with --preload sdk
    Bun->>SDK: Load kit-sdk.ts
    Bun->>Script: Execute user script

    Script->>SDK: await arg("Pick one", choices)
    SDK->>Bun: Write JSONL to stdout
    Bun-->>GPUI: {"type":"arg","placeholder":"Pick one",...}
    GPUI->>Handler: Switch to ArgPrompt
    Handler->>User: Render prompt UI

    User->>Handler: Select choice
    Handler->>GPUI: Submit value
    GPUI-->>Bun: Write to stdin: {"value":"selected"}
    Bun->>SDK: Resolve promise
    SDK->>Script: Return "selected"

    Script->>SDK: Script complete
    SDK->>Bun: Exit
    Bun-->>GPUI: Process exit
    GPUI->>Handler: Reset to MainMenu
```

## Change Impact Guide

| If changing... | Check diagram | Look for... |
|----------------|---------------|-------------|
| Any module | Architecture | Upstream/downstream dependencies |
| State handling | State Machine | Which transitions are affected |
| Protocol messages | Execution Flow | All actors that parse that message type |
| Prompt rendering | Architecture + State | Both the UI module and valid states |
| SDK functions | Execution Flow | The full request/response cycle |

**Quick impact assessment:**
1. Find your module in Architecture diagram
2. Trace arrows IN (what depends on you) and OUT (what you depend on)
3. For state changes, verify transitions remain valid
4. For protocol changes, update both Rust and TypeScript sides
5. Run tests for all affected modules: `cargo test <module_name>`

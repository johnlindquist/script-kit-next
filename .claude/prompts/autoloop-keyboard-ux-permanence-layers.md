---
goal: "Audit and solidify keyboard UX across the three permanence layers: Main Menu (ephemeral), ACP Chat (session), and Notes (persistent). Make the transitions between layers feel like a single continuous experience."
surface: "Main Menu, ACP Chat (in-panel + detached), Notes Window"
verification: "agentic-testing against each surface — build, launch, exercise every keyboard path, capture screenshots, read PNGs"
---

# Permanence Layer Keyboard UX

## Product Vision

Script Kit's unique value is a frictionless gradient between thinking and shipping:

1. **Ephemeral (Main Menu)** — keyboard-first launcher. Search, run, dismiss. Sub-second.
2. **Session (ACP Chat)** — AI conversation. Multi-turn, exploratory, generative. Ideas get drafted here.
3. **Persistent (Notes)** — durable landing zone. Notes survive sessions. Notes get promoted to scripts, skills, extensions.

Each layer increases permanence. The UX goal: **every transition between layers is one gesture, and the user never loses orientation.**

The flow:
```
Main Menu ─[Tab]──→ ACP Chat ─[⌘S]──→ Note (opens Notes window)
                         ↑                    │
                         └──[⌘K action]───────┘
                                               │
                                         [⌘G: open AI in Notes]
                                               │
                                         Notes ACP (note as context)
                                               │
                                         "Create Script/Skill/Extension"
```

**ACP Chat does NOT need a detached window.** When a user wants to persist a conversation, they save it as a note. The Notes window is the only persistent detached surface. "Detach" is replaced by "Save."

## What Needs to Work

### Transitions
- **Tab** in Main Menu opens ACP Chat (already works — verify it's solid)
- **⌘S** in ACP Chat saves the conversation as a note AND opens the Notes window with that note focused. This replaces any "detach" behavior.
- **Escape** in ACP Chat returns to Main Menu (already works — verify edge cases with popups, model picker, attachment portal)
- **⌘G** in Notes opens ACP Chat scoped to the current note's content as context
- **⌘P** in both ACP Chat and Notes opens history (chat history / note history respectively)

### Notes Window Keyboard (broken — needs diagnosis and fix)
- ⌘K (actions), ⌘P (browse/history), and other shortcuts appear to be broken in the Notes window. Diagnose why and fix.
- The Notes window must have a working actions panel (⌘K) that includes actions for:
  - "Continue in Chat" — open ACP Chat with the full note as conversation context
  - "Send to Chat" — open a fresh ACP Chat with the note attached as context
  - Promoting a note to a script/skill/extension (this should route through ACP Chat — the AI is the transformation engine)

### ACP Chat Polish
- The @ mention picker in ACP Chat should include notes as a mentionable context source
- Chat history (⌘P) needs to be polished and friendly — this is how users find past conversations before they were saved as notes
- When ACP Chat has an active conversation and the user presses ⌘S, the save-to-note flow should feel instant and obvious — the user should immediately see their note in the Notes window

### Notes ACP (⌘G)
- Opening AI from inside Notes (⌘G) should feel natural — the current note content becomes the AI's context automatically
- The Notes ACP should respect the same keyboard conventions as the main ACP Chat (Escape to dismiss, @ for mentions, etc.)
- There should be a visible affordance (not just a shortcut) that tells the user AI is available — but it should follow the "whisper chrome" principle (subtle, not loud)

### Keyboard Consistency Audit
Every surface needs consistent answers to:
- What does **Escape** do? (Always: dismiss current layer / go back one level)
- What does **⌘K** do? (Always: open actions for the current context)
- What does **⌘P** do? (Always: open history for the current surface)
- What does **⌘W** do? (Always: close the current window)
- What does **⌘S** do? (Always: save / graduate to the next permanence layer)

If any shortcut behaves inconsistently across surfaces, flag it and propose a resolution.

## Design Principles (from .impeccable.md / CLAUDE.md)

- **Three keys, nothing more** — footer shows at most three affordances
- **Discovery lives in Actions (⌘K)** — features are discoverable through the actions dialog, not through persistent chrome
- **Whisper chrome** — ultra-low opacity surfaces, content gets full opacity, everything else fades
- **Speed is the design** — every pixel serves instant comprehension
- **Keyboard-first, always** — mouse is a fallback, every interaction reachable via keyboard
- **Native or nothing** — respect macOS conventions

## Constraints

- Tab is NOT available as a shortcut inside the Notes editor (it's indent/outdent)
- The Notes window is a single-note-at-a-time editor with overlay panels (actions, browse). There is no sidebar or split pane.
- ACP Chat in the main panel and Main Menu are mutually exclusive views — navigating away from ACP Chat replaces the view
- The Notes window is an independent `WindowKind::PopUp` surface with its own keyboard handler (`src/notes/window/keyboard.rs`)
- The footer in every surface must follow the three-key pattern: at most three hint-opacity affordances

## What NOT to Do

- Do not add a detached ACP Chat window. Notes is the persistence layer.
- Do not add persistent chrome, toolbars, or always-visible panels for AI access. Use ⌘G and ⌘K actions.
- Do not break existing keyboard shortcuts that already work correctly.
- Do not add features beyond what's described here. If you notice something worth improving, mention it at the end.

## Key Files

- `src/notes/window/keyboard.rs` — Notes keyboard handler (broken shortcuts live here)
- `src/notes/window/render.rs` — Notes render entry point
- `src/notes/actions_panel.rs` — Notes actions panel (⌘K)
- `src/notes/browse_panel.rs` — Notes browse/history panel (⌘P)
- `src/ai/acp/view.rs` — ACP Chat view, input handling, context picker
- `src/ai/acp/chat_window.rs` — detached ACP Chat window (to be replaced by save-to-note)
- `src/ai/acp/history_popup.rs` — ACP Chat history
- `src/app_impl/tab_ai_mode.rs` — ACP Chat entry, context assembly, close semantics
- `src/app_impl/startup_new_actions.rs` — keyboard interceptor (Escape from ACP, etc.)
- `src/components/prompt_header/component.rs` — prompt header (footer affordances)
- `CLAUDE.md` — full coding conventions and design principles
- `.vibrancy.md` — footer blur architecture (read before touching any footer)

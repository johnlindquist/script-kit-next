# Apple Notes Quick Note (macOS) - UX Research

Date: 2026-02-01
Scope: Quick Note entry points, hot-corner behavior, floating note windows, gestures, and keyboard shortcuts on macOS.

## Quick Note core behavior (from Apple Notes User Guide)
- Quick Note is designed for fast capture while working in any app; the note stays visible while open so you can reference other apps and add info. (Apple Notes User Guide)
- Entry points:
  - Keyboard: Press and hold Fn/Globe, then press Q to create a Quick Note. (Apple Notes User Guide; Notes shortcuts)
  - Hot corner: Move pointer to the bottom-right corner (default) and click the note that appears. (Apple Notes User Guide)
  - Safari capture: Use the Share menu "Add to Quick Note" or control-click selected text to add it. (Apple Notes User Guide)
- Lifecycle:
  - Close Quick Note with the red window close button; reopen using any entry method. (Apple Notes User Guide)
  - Setting: Notes > Settings > deselect "Always resume to last Quick Note" to force a new Quick Note each time. (Apple Notes User Guide)
- Linking behavior from Safari:
  - Share menu "Add to Quick Note" creates a link; returning to the page shows a Quick Note thumbnail in the corner. (Apple Notes User Guide)
  - Control-click selected text adds a link; the text remains highlighted on later visits. Removing the link in Quick Note removes the highlight. (Apple Notes User Guide)
- Organization and limits:
  - Quick Notes appear in the "Quick Notes" folder in Notes. (Apple Notes User Guide)
  - You cannot lock a Quick Note. (Apple Notes User Guide)

## Hot corners: UX patterns and configuration (macOS User Guide)
- Hot corners trigger actions when you move the pointer into a screen corner. (macOS User Guide)
- Default mapping: bottom-right corner is set to Quick Note. (macOS User Guide)
- Configuration path: System Settings > Desktop & Dock > Hot Corners. (macOS User Guide)
- Accidental-trigger mitigation: you can require modifier keys (Command/Shift/Option/Control) while choosing the corner action; the action fires only when the modifier is held. (macOS User Guide)

## Gestures (Notes app on Mac)
- In the notes list, swipe right with two fingers (trackpad) to pin a note; swipe left to delete or share. (Notes keyboard shortcuts and gestures)

## Keyboard shortcuts (Notes app on Mac)
- Fn+Q: Create a Quick Note. (Notes keyboard shortcuts)
- Command-0: Show the main Notes window; Apple notes this is useful when a separate note window is blocking the main window. (Notes keyboard shortcuts)

## Floating note windows (separate from Quick Note)
- Notes app can float individual notes above other windows via Window > "Float Selected Note"; the note gets its own window that stays on top. (MacRumors)
- Floating is a toggle in the Window menu; you can keep the separate window without floating by toggling off. (MacRumors)
- Multiple notes can open in separate windows (double-click notes in the list); their positions and floating state are remembered on next launch. (MacRumors)
- Limitation: floating notes cannot share the same screen as a fullscreen app. (MacRumors)

## UX takeaways (implementation signals)
- Quick Note is optimized for minimal context switching: keyboard/hot-corner entry, persistent visibility while open, and automatic linking back to source content.
- Hot-corner default plus optional modifier keys suggests a balance between speed and accidental-trigger prevention.
- The Notes app supports a true always-on-top note window ("Float Selected Note"), which is a distinct pattern from Quick Note and may be useful for reference notes.

## Sources
- Apple Notes User Guide: Create a Quick Note on Mac
  https://support.apple.com/en-lamr/guide/notes/apdf028f7034/mac
- Apple Notes User Guide: Keyboard shortcuts and gestures in Notes on Mac
  https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac
- macOS User Guide: Use hot corners on Mac
  https://support.apple.com/et-ee/guide/mac-help/mchlp3000/mac
- MacRumors: How to Float Notes Over Application Windows in macOS
  https://www.macrumors.com/how-to/float-notes-over-application-windows-macos/

# Floating Notes Window UX - Best Practices (Research Summary)

Date: 2026-02-01
Scope: Floating notes / sticky notes windows in productivity apps, with emphasis on positioning, transparency, and interaction patterns.

## Positioning and window behavior

- Treat notes as secondary or panel-style windows, not primary documents. Keep them clearly auxiliary to the main app and close them when the primary window closes. Avoid stacking secondary windows on top of each other and keep them smaller than their parent windows with simple content. [GNOME Windows]
- Panels float above document windows and are only visible when their application is active. If notes should stay visible system-wide, make that an explicit opt-in ("system-wide" style) instead of the default. [Apple HIG Windows]
- Allow multiple notes at once and bring all panels to the front when their document becomes active. Hide panels when the app is inactive. [Apple HIG Windows]
- Provide a draggable title-bar region even for minimal chrome. Users expect to grab and place a note quickly. [Apple HIG Windows]
- Make "always on top" a clear toggle (not a hidden state). Provide a visible affordance like a border or badge and an easy keyboard shortcut to pin/unpin. Optional feedback (sound) and an exclusion list help avoid conflicts (e.g., game mode or fullscreen apps). [PowerToys Always On Top]
- Offer manual arrangement and simple grouping/arranging options for multiple notes to reduce clutter. [Apple Stickies User Guide]

## Transparency and visual treatment

- Provide a user-controlled translucency toggle so notes can stay visible without completely blocking underlying content. [Apple Stickies User Guide]
- Use transparent panels sparingly: they are intended for visually immersive contexts and can be distracting in typical apps. Do not make translucency the only mode. [Apple HIG Windows]
- If a transparent panel is used, keep it small and limit text entry or complex controls; transparency works best for simple adjustments. [Apple HIG Windows]
- Use a clear visual indicator for pinned state (border, accent color) and let users adjust opacity/thickness to avoid distraction. [PowerToys Always On Top]

## Interaction patterns

- Never pop a dialog unexpectedly; show dialogs only in direct response to user actions. When a dialog opens, place initial keyboard focus on the first expected control, and ensure Esc cancels if a cancel action exists. [GNOME Dialogs]
- Prefer undo over confirmation dialogs for destructive actions to reduce interruption. [GNOME Dialogs]
- Support standard close shortcuts (Ctrl+W / Cmd+W) for windows and Esc for modal windows. [GNOME Windows]
- Include fast space-saving interactions: collapse/expand notes and quick maximize/restore. [Apple Stickies User Guide]
- Allow resizing by dragging edges, and expose note metadata (created/edited) on hover so the note stays visually minimal. [Apple Stickies User Guide]
- If notes disappear when the app closes (as in Stickies), make that behavior explicit. If your product needs persistent notes, run a background agent or keep the notes window alive even when the main app closes. [Apple Stickies User Guide]

## Practical checklist

- Secondary/panel window model, with parent-child lifecycle and small footprint. [GNOME Windows] [Apple HIG Windows]
- Explicit pin/unpin control with visible indicator and hotkey. [PowerToys Always On Top]
- Optional translucency with readable text and small/simple controls. [Apple Stickies User Guide] [Apple HIG Windows]
- Avoid modal interruptions; focus and Esc behavior are consistent. [GNOME Dialogs]
- Resizable, collapsible, and quick to reposition. [Apple Stickies User Guide]

## References

- Apple Human Interface Guidelines: Windows (Panels, transparent panels, and layering)
  https://leopard-adc.pepas.com/documentation/UserExperience/Conceptual/AppleHIGuidelines/XHIGWindows/XHIGWindows.html
- GNOME HIG: Windows (secondary windows sizing, stacking, close shortcuts)
  https://developer.gnome.org/hig/patterns/containers/windows.html
- GNOME HIG: Dialogs (avoid unexpected dialogs, focus, Esc, undo)
  https://developer.gnome.org/hig/patterns/feedback/dialogs.html
- Apple Stickies User Guide (float on top, translucent, collapse/expand, resize)
  https://support.apple.com/et-ee/guide/stickies/welcome/mac
- Microsoft PowerToys: Always On Top (pin/unpin, border indicator, settings)
  https://learn.microsoft.com/en-us/windows/powertoys/always-on-top

# One-Key Gesture Grammar

**Superseded.** This records the original decision. For current behavior (hold to dictate, double-tap for Today), see [Type, talk, and Today](../specs/gesture-grammar.md).

The main hotkey carries memory-layer intent through physical gesture without breaking the launcher toggle habit: tap (closed) opens the launcher on key-down; tap (open) closes the main window; hold (~250ms) deepens the window into the Day Page (no dictation — that belongs to the dictation hotkey and the voice key); double-tap opens Agent Chat from anywhere. Cmd+Enter remains the established agent gesture and Esc remains a dismiss path. The disambiguation rule is "always open, then deepen": the launcher renders on key-down in every case, and hold/double-tap escalate the already-open window before the tap resolves, so first paint never waits on classification.

We chose this over a Tab-based universal grammar (conflicts with editor Tab formatting; Tab AI is deprecated in favor of Cmd+Enter), over per-surface shortcuts (users should not learn new chords), over a launcher fallback row that keeps unmatched queries as notes (users do not type into the launcher expecting capture without expressed intent — semicolon tags and the hold gesture carry that intent), and over wait-to-disambiguate gesture detection (any delay on tap breaks the instant-launcher habit loop).

Constraints: every surface opens under the same input in one window — windows are never swapped, because swaps flash and break the instant feel. Tap-while-open remains the standard hotkey dismiss behavior; Day Page entry belongs to hold-from-closed and explicit in-app triggers. Detailed state machine, timings, and open questions live in `docs/specs/gesture-grammar.md`.

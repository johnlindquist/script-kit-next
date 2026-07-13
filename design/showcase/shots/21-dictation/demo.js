/* Demo script: 21-dictation — "the waveform, timer, and hotkeys narrate a
 * live dictation session". Demo-driven scene: no exploration controls, the
 * whole cycle is a single narrated pass. No microphone access — pure
 * visual simulation. */
SKDemo.define({
  id: "21-dictation",
  initialHoldMs: 900,
  idleResetMs: 8000,
  loopDelayMs: 1200,
  hudPlacement: "top-right",
  steps: [
    {
      id: "intro",
      op: "caption",
      text: "The waveform and timer show active dictation.",
      holdMs: 1200,
    },
    {
      id: "waveform-active",
      op: "effect",
      name: "waveform",
      target: '[data-demo-key="waveform"]',
      durationMs: 2000,
      holdMs: 0,
    },
    { id: "timer-08", op: "setText", target: '[data-demo-key="timer"]', text: "0:08" },
    { op: "pause", ms: 700 },
    { id: "timer-09", op: "setText", target: '[data-demo-key="timer"]', text: "0:09" },
    { op: "pause", ms: 600 },
    {
      id: "select-mic-pulse",
      op: "effect",
      name: "pulse",
      target: '[data-demo-key="select-mic"]',
      durationMs: 700,
    },
    {
      id: "select-mic-caption",
      op: "caption",
      text: "Choose the active microphone without leaving the overlay.",
      holdMs: 1300,
    },
    { id: "stop-keypress", op: "keypress", keys: ["⇧", "⌘", ";"], holdMs: 500 },
    {
      id: "dictation-stopped",
      op: "effect",
      name: "pulse",
      target: '[data-demo-key="stop-label"]',
      durationMs: 700,
    },
    { op: "pause", ms: 500 },
    {
      id: "cancel-pulse",
      op: "patch",
      ops: [
        { op: "effect", name: "pulse", target: '[data-demo-key="cancel-label"]', durationMs: 700, holdMs: 0 },
        { op: "effect", name: "pulse", target: '[data-demo-key="cancel-undo"]', durationMs: 700, holdMs: 0 },
      ],
    },
    { op: "pause", ms: 700 },
    { id: "timer-reset", op: "setText", target: '[data-demo-key="timer"]', text: "0:07" },
    { op: "loop", delayMs: 1200 },
  ],
});

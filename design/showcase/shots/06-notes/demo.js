/* Demo script: 06-notes — "a markdown scratchpad with live checklists". */
SKDemo.define({
  id: "06-notes",
  initialHoldMs: 900,
  idleResetMs: 8000,
  loopDelayMs: 1200,
  steps: [
    { id: "intro", op: "caption", text: "A markdown scratchpad with live checklists.", holdMs: 1300 },
    { op: "pause", ms: 500 },
    { id: "check-hero-demo", op: "setText", target: '[data-demo-key="task-hero-demo"]', text: "[x]" },
    { op: "pause", ms: 600 },
    { id: "two-tasks-checked", op: "setText", target: '[data-demo-key="task-launch-post"]', text: "[x]" },
    { op: "pause", ms: 600 },
    { id: "pulse-updated", op: "effect", name: "pulse", target: '[data-demo-key="updated"]', durationMs: 600, holdMs: 700 },
    { id: "outro", op: "caption", text: "Changes are already saved.", holdMs: 1300 },
    { op: "loop", delayMs: 1200 },
  ],
});

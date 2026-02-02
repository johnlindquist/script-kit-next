# AI response streaming UX patterns and best practices

## Evidence highlights (quick)
- Users should see system status feedback quickly; visibility of system status is a core usability heuristic. citeturn2view0
- Response time thresholds still cited by NNG: ~0.1s feels instantaneous, ~1s keeps flow, and ~10s is the attention limit; delays beyond that need progress feedback and a clear way to interrupt. citeturn1view0
- Indeterminate activity indicators should be used for short waits only; place them near where content appears and do not block UI. citeturn6view0
- Skeleton or shimmer placeholders are recommended when layout is known and loading takes >1s; do not combine multiple progress indicators and keep placeholders aligned to the final structure. citeturn8view0turn7view0
- Typing indicators set expectations and use debounced timeouts (commonly ~5s) to avoid noisy, per-keystroke updates. citeturn4view0
- Streaming markdown benefits from incremental parsing and block-level memoization to avoid reprocessing the entire document; handling incomplete blocks prevents broken formatting during streams. citeturn9view0turn10view0turn11view0
- Semi-incremental parsing (block-level incremental with inline re-rendering) balances performance with correctness as partial tokens arrive. citeturn12view0

## Recommendations

### 1) Progressive rendering
- Start visual feedback within ~1s and keep users informed during generation to satisfy visibility-of-system-status and response-time expectations. citeturn2view0turn1view0
- Prefer showing partial content as it arrives rather than a spinner alone; use skeleton/shimmer only when the layout is known and you need to convey structure before content is ready. citeturn8view0turn7view0
- Use indeterminate indicators only for short waits; switch to determinate/progress info when you can measure progress, and place indicators where new content will appear without blocking the UI. citeturn6view0turn1view0

### 2) Cancellation and control
- Provide a clearly labeled "Stop generating" or "Cancel" as an emergency exit so users can back out of long or unwanted generations. citeturn2view0
- If the operation is interruptible, include Cancel or Stop in the progress UI; use "Stop" when interruption can cause side effects. citeturn5view0
- Inference: keep partial output visible after cancel and allow a quick retry to preserve user effort and reduce frustration; this aligns with user-control heuristics and long-wait guidance. citeturn2view0turn1view0

### 3) Typing indicators (and streaming affordances)
- Show a typing indicator (or an active-caret at the end of the stream) to signal that a response is being composed and set expectations. citeturn4view0
- Debounce typing signals and use a timeout so indicators expire if no new activity arrives; avoid per-keystroke updates. citeturn4view0
- Treat the indicator as an indeterminate activity cue: keep it near the response area, do not block interaction, and avoid long-lasting indeterminate states. citeturn6view0turn1view0

### 4) Markdown rendering during stream
- Avoid re-parsing the entire markdown on each chunk; use incremental parsing to prevent O(n^2) behavior as the document grows. citeturn9view0
- Memoize at the block level so completed blocks stay stable and only new/changed blocks re-render, reducing flicker and CPU churn. citeturn10view0
- Handle incomplete markdown (unterminated emphasis, code fences, links) so partially streamed content still renders cleanly instead of showing raw syntax. citeturn11view0
- Consider a semi-incremental strategy: incrementally render new blocks while re-rendering inline elements within the active block for correctness as tokens arrive. citeturn12view0

## Practical checklist
- Feedback within ~1s; stream tokens as soon as possible; label waits and keep status visible. citeturn2view0turn1view0
- If layout is known and wait >1s, use skeleton/shimmer; do not stack multiple indicators. citeturn8view0turn7view0
- Provide Stop/Cancel; make it obvious and safe to use. citeturn2view0turn5view0
- Typing indicator: debounce and auto-expire; keep it near the response area and short-lived. citeturn4view0turn6view0
- Markdown stream: incremental parsing + block memoization + incomplete-block handling; prefer stable blocks with minimal reflow. citeturn9view0turn10view0turn11view0

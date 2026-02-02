# 08 - Streaming and Response Rendering (AI Chat)

Goal: summarize streaming UX patterns, streaming-safe Markdown rendering, code highlighting options, and concrete suggestions for Script Kit's AI chat window.

## Streaming patterns

### Transport and event shapes
- Server-sent events (SSE) use the `text/event-stream` MIME type, and each message is separated by a blank line (a pair of newlines). Fields like `event:` and `data:` form each message. [1]
- The browser `EventSource` interface is the standard client for SSE, and SSE is unidirectional (server -> client), which fits streaming model output. [2]
- OpenAI and Anthropic both stream responses as SSE with typed events and delta updates (start/delta/stop style flows). [3] [4]

### UX feedback patterns
- Use an indeterminate indicator when duration is unknown and a determinate indicator only when you can estimate progress. Material Design recommends using a single indicator per operation and distinguishes determinate vs indeterminate states. [5]
- GNOME HIG suggests showing spinners when operations take more than ~3 seconds, avoiding very short spinner flashes, and preferring a progress bar if the task is likely to exceed ~30 seconds. [6]

### Practical streaming UI pattern (recommended)
- Show a subtle "assistant is thinking" indicator after a short delay (e.g. 200-300ms) to avoid flicker, then replace it with streamed content when the first delta arrives. (Inference based on progress indicator guidance. [5] [6])
- Keep a visible "Stop generating" control while streaming, and allow cancel/abort at any time. (Inference: common streaming UX pattern.)
- Avoid layout jitter by throttling render updates (e.g. 50-100ms) and preferring chunk-level updates (sentence/paragraph) over per-token rendering. (Inference.)

## Markdown rendering for streaming output

### Streaming-safe Markdown
- A streaming-optimized renderer should tolerate incomplete or unterminated Markdown so the UI can render partial content without breaking formatting. Streamdown is a drop-in `react-markdown` replacement designed for streaming and explicitly handles incomplete Markdown blocks. [7]
- Streamdown also supports GitHub Flavored Markdown, code blocks with Shiki highlighting, and uses `rehype-harden` for safe rendering. [7]

### Code fences and partial blocks
- CommonMark defines fenced code blocks with opening/closing fences and an optional info string (language). The code block content is treated as literal text until a closing fence is found, and an unclosed fence runs to the end of the document. [8]
- For streaming, treat an open fence as "in-progress" code and avoid applying inline Markdown rules within that fence. (Inference based on CommonMark behavior. [8])

### Rendering strategy (recommended)
- Maintain a raw text buffer plus a parsed/rendered tree. Re-render on a timer or on chunk boundaries, not every token. (Inference.)
- Keep a "live tail" segment that is re-parsed more frequently, while freezing earlier segments to reduce reflow. (Inference.)
- If we adopt a stream-tolerant renderer (Streamdown-like), we can avoid most partial-markdown edge cases. [7]

## Code highlighting

### Options
- Shiki uses TextMate grammars (same engine as VS Code) for accurate highlighting and can run ahead of time to avoid runtime cost. [9]
- highlight.js supports auto language detection, works in browsers and Node, and has zero dependencies; it also allows loading only the languages you need. [10]

### Streaming-aware highlighting (recommended)
- Only syntax-highlight a code block after its closing fence arrives; before that, render it as plain monospace text to avoid reflow and broken highlighting. (Inference based on CommonMark fence rules. [8])
- Use the first word of the fence info string as the language hint when available, and fall back to plaintext or auto-detect when it is not. [8] [10]
- Run highlighting asynchronously to avoid UI stalls, then swap in highlighted HTML when ready. (Inference.)

## Suggestions for Script Kit AI chat window

1. **Streaming protocol and event handling**
   - Treat provider streams as SSE-style events with typed deltas. Normalize to a small internal event set (start/delta/stop/error) that matches OpenAI/Anthropic stream semantics. [3] [4]
   - Use an event queue to coalesce UI updates and keep rendering smooth (e.g. 50-100ms tick). (Inference.)

2. **Progress indicators and feedback**
   - Use a single indicator per operation, defaulting to indeterminate while waiting on model output, and avoid very short flashes. [5] [6]
   - Switch to determinate only when you have reliable progress metrics (rare for LLMs). [5]

3. **Markdown rendering path**
   - Prefer a streaming-tolerant Markdown renderer (Streamdown-style) or replicate its behavior: partial block tolerance + safe rendering. [7]
   - Keep the previous stable output and update only the current streaming tail to reduce layout jitter. (Inference.)

4. **Code blocks and highlighting**
   - While streaming an open fence, render as plain code with a lightweight style. After closing fence, apply Shiki or highlight.js. [8] [9] [10]
   - Use the info string language when present; otherwise fallback to plaintext or auto-detect. [8] [10]

5. **Safety note**
   - Streaming makes moderation harder because partial completions are harder to evaluate. Consider a safety layer or delayed reveal in high-risk contexts. [3]

## Sources

1. MDN - Using server-sent events (SSE format, `text/event-stream`, message separation): https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events.
2. MDN - EventSource (SSE is unidirectional; EventSource behavior): https://developer.mozilla.org/en-US/docs/Web/API/EventSource
3. OpenAI - Streaming API responses (SSE streaming, typed events, moderation note): https://platform.openai.com/docs/guides/streaming-responses
4. Anthropic - Streaming Messages (SSE, event types and flow): https://platform.claude.com/docs/en/build-with-claude/streaming
5. Material Design - Progress & activity indicators (determinate vs indeterminate; one indicator per operation): https://m1.material.io/components/progress-activity.html
6. GNOME HIG - Spinners (when to show spinners; avoid short flashes; >30s use progress bar): https://developer.gnome.org/hig/patterns/feedback/spinners.html
7. Vercel Streamdown README (streaming-optimized Markdown; incomplete blocks; GFM; Shiki; rehype-harden): https://github.com/vercel/streamdown
8. CommonMark Spec 0.31.2 - Fenced code blocks and info strings: https://spec.commonmark.org/0.31.2/
9. Shiki docs (TextMate grammar, VS Code engine, zero runtime): https://shiki.matsu.io/
10. highlight.js docs (auto-detection, zero deps, works in browser/Node): https://highlightjs.org/

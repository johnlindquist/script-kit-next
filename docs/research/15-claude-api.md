# Claude API + Claude Code integration research

Scope: API patterns, streaming, tool use, SDK patterns, and options for leveraging Claude Code subscriptions in Script Kit's AI chat window.

## 1) Claude API patterns (Messages API)

- The Messages API is stateless. Each request must include the full conversation history (you can also inject synthetic assistant turns).
- Requests pass a `messages` array of objects with `role` and `content`. Roles are `user` and `assistant`. If the final message is `assistant`, the response continues from that content (useful for prefills).
- `content` can be a plain string or an array of typed content blocks; a string is shorthand for a single `text` block.
- System prompts are provided via the top-level `system` parameter; there is no `system` role in the messages list.
- Responses include a `content` array of blocks and a `stop_reason` (e.g., `end_turn`, `tool_use`, `max_tokens`, `stop_sequence`).

## 2) Streaming (SSE)

- Set `stream: true` to receive Server-Sent Events (SSE) for incremental output.
- Event flow is `message_start` -> one or more content blocks (`content_block_start`, `content_block_delta`, `content_block_stop`) -> one or more `message_delta` -> `message_stop`.
- Streams may include `ping` events and `error` events; clients should also handle unknown event types gracefully.
- Usage in `message_delta` is cumulative for the stream.

## 3) Tool use

- Tools are provided via a top-level `tools` array. Each tool includes `name`, `description`, and a JSON Schema `input_schema`.
- `tool_choice` supports `auto` (default when tools are provided), `any`, `tool` (force a specific tool), and `none` (default when no tools are provided).
- When Claude uses a client tool, the response has `stop_reason: "tool_use"` and includes `tool_use` blocks with `id`, `name`, and `input`. The client must execute the tool and send a follow-up `user` message containing `tool_result` blocks.
- Tool results must immediately follow the tool-use message and must appear first in the user message content array, or the API will error.
- Anthropic's tool-use format stays within the same `user`/`assistant` message structure rather than introducing a separate tool role.

## 4) Anthropic SDK patterns (TypeScript)

- The official TypeScript SDK uses `client.messages.create({ model, max_tokens, messages })` for standard requests.
- Streaming in the SDK is done by passing `stream: true` and iterating the async iterable of events; you can cancel via `stream.controller.abort()`.
- The SDK provides tool helpers (e.g., Zod/JSON schema helpers) and a `client.beta.messages.toolRunner()` to manage the tool loop and feed results back to the model.

## 5) Claude Code subscription integration

### What Claude Code supports

- Claude Code requires an account and supports login via Claude.ai subscription plans or an Anthropic Console account. Login is performed in the `claude` CLI using `/login`, and credentials are stored for reuse.
- Headless mode allows non-interactive use via `claude -p` (`--print`) with output formats `text`, `json`, or `stream-json`. It also supports streaming JSON input for multiple turns without re-launching the CLI.
- CLI flags include `--print`, `--output-format`, `--input-format`, `--resume`, `--max-turns`, and tool permissions (e.g., `--allowedTools`).

### Authentication priority (critical for subscription usage)

- If `ANTHROPIC_API_KEY` is set, Claude Code prioritizes the API key over Claude.ai subscription auth, which results in API-billed usage even if a subscription is logged in. To use subscription entitlements, keep `ANTHROPIC_API_KEY` unset and verify via `/status`.

### Policy constraint for third-party apps

- Anthropic's Agent SDK documentation states that third-party developers may not offer Claude.ai login or rate limits for their products without prior approval, and should use API key authentication instead. This impacts any plan to "reuse" Claude.ai subscriptions inside our app UI.

### Implications for Script Kit AI chat

**Option A: Direct Claude API (recommended default)**
- Use Messages API + API key. This aligns with Anthropic's stated guidance for third-party products and keeps us within standard API policy boundaries.

**Option B: Local Claude Code CLI bridge (opt-in)**
- If we choose to support "use my Claude Code subscription," the safest technical path is to shell out to the locally installed `claude` CLI in headless mode and parse `stream-json` output. This keeps auth on the user's machine and uses the same login they already have in Claude Code.
- To preserve subscription usage, spawn `claude` with a clean environment (no `ANTHROPIC_API_KEY`) and document `/status` checks.
- Use `--input-format stream-json` with `--output-format stream-json` to maintain multi-turn chats without restarting the CLI; preserve `session_id` or use `--resume` if reattaching.
- Gate dangerous tooling with `--allowedTools` / `--permission-mode` so the CLI agent cannot mutate files or run commands unless explicitly approved by the user.

**Open question (needs policy confirmation):**
- The Agent SDK policy note suggests third-party products should not expose Claude.ai login or subscription rate limits. Even if we simply invoke the local Claude Code CLI, we should confirm with Anthropic whether embedding that capability in our app UI is permitted.

## 6) Suggested implementation notes (for our codebase)

- Provider selection in AI window: `Anthropic API (key)` vs `Claude Code (local CLI)`. Default to API key; make CLI opt-in with explicit policy disclaimer.
- If CLI mode is enabled, detect `claude` binary, run `claude -p --output-format stream-json --input-format stream-json`, stream JSON lines into the chat UI, and allow cancel by terminating the process.
- When CLI mode is used, sanitize env vars (remove `ANTHROPIC_API_KEY`) unless the user explicitly opts into API key billing.
- When API mode is used, implement Message API stateless history, streaming events, and tool-use loops as described above.


## Sources

- https://docs.anthropic.com/en/docs/build-with-claude/tool-use/implement-tool-use
- https://docs.anthropic.com/en/docs/build-with-claude/streaming
- https://platform.claude.com/docs/en/build-with-claude/working-with-messages
- https://docs.anthropic.com/fr/api/messages
- https://github.com/anthropics/anthropic-sdk-typescript
- https://docs.anthropic.com/en/docs/claude-code/quickstart
- https://docs.anthropic.com/en/docs/claude-code/headless-mode
- https://docs.anthropic.com/en/docs/claude-code/cli-reference
- https://support.anthropic.com/en/articles/11014279-how-do-i-change-the-anthropic-api-key-used-in-claude-code
- https://platform.claude.com/docs/en/agent-sdk/overview

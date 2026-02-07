# AI/Chat Features Improvement Report

Date: 2026-02-07
Agent: codex-ai-features
Scope: `src/ai/**/*.rs`, `src/prompts/chat.rs`

## Executive Summary

The AI chat stack is functionally rich, but there are a few high-impact correctness and integration gaps that should be prioritized before feature expansion:

1. SDK-side AI status APIs return placeholders instead of true window/runtime state, so programmatic integrations can be wrong (`src/ai/sdk_handlers.rs:43`, `src/ai/sdk_handlers.rs:57`, `src/ai/sdk_handlers.rs:265`).
2. Message image attachments are dropped by persistence, causing multimodal history loss across reloads (`src/ai/storage.rs:560`, `src/ai/storage.rs:623`, `src/ai/storage.rs:821`).
3. Streaming stop/retry paths can produce inconsistent conversation history (duplicate prompts and stale/orphaned assistant writes) (`src/ai/window.rs:1106`, `src/ai/window.rs:2916`, `src/ai/window.rs:3158`, `src/ai/window.rs:3194`).
4. Streaming performance currently relies on full-string clone + polling in two separate implementations, which will degrade for long responses (`src/ai/window.rs:2861`, `src/ai/window.rs:2958`, `src/prompts/chat.rs:1621`, `src/prompts/chat.rs:1667`, `src/prompts/chat.rs:1735`).
5. Session management for Claude persistent mode holds a global lock during long-running send operations, reducing concurrency (`src/ai/session.rs:231`, `src/ai/session.rs:281`).

## What Is Working Well

- Generation guards for stale stream updates are present and tested (`src/ai/window.rs:2906`, `src/ai/window.rs:8273`).
- AI window UX already includes edit-last-message, regenerate, retry UI, copy/export actions, and keyboard shortcuts (`src/ai/window.rs:1123`, `src/ai/window.rs:3203`, `src/ai/window.rs:5490`).
- Error classification in streaming UI is stronger than typical baseline implementations (`src/ai/window.rs:5361`).

## Severity-Ranked Findings

### P0: Runtime status APIs are not wired to live AI state

- Evidence:
  - `AiIsOpen` hardcodes `active_chat_id = None` (`src/ai/sdk_handlers.rs:43`).
  - `AiGetActiveChat` returns most-recent chat from storage, not active window chat (`src/ai/sdk_handlers.rs:57`, `src/ai/sdk_handlers.rs:59`).
  - `AiGetStreamingStatus` always returns `is_streaming: false` (`src/ai/sdk_handlers.rs:265`, `src/ai/sdk_handlers.rs:269`).
  - Protocol supports these richer fields (`src/protocol/message.rs:1144`, `src/protocol/message.rs:1376`, `src/protocol/message.rs:1382`).
- Impact:
  - SDK scripts and automations cannot reliably introspect active chat or streaming state.
  - Integrations can make wrong UI decisions (double-send, invalid polling, stale status badges).
- Recommendation:
  - Introduce a shared `AiRuntimeState` (active chat id, is_streaming, partial text, last_updated, generation) updated by `window.rs` and queried by `sdk_handlers.rs`.
  - Keep storage fallback only for "window closed, no active runtime" cases.

### P0: Retry/stop flows can corrupt conversation semantics

- Evidence:
  - Retry after error copies last user text into input and calls `submit_message`, creating a brand-new user message (`src/ai/window.rs:1106`, `src/ai/window.rs:1118`).
  - Stale completion path persists orphaned assistant messages to DB (`src/ai/window.rs:2916`, `src/ai/window.rs:2921`).
  - `stop_streaming` saves partial assistant message and then invalidates generation (`src/ai/window.rs:3158`, `src/ai/window.rs:3194`).
- Impact:
  - Retry can duplicate user prompts and skew context.
  - Stop can still later persist a full completion from the background stream, producing duplicate/contradictory assistant messages.
- Recommendation:
  - Separate "retry last request" from "submit new user message".
  - Track stream termination reason (`user_stop`, `chat_switch`, `chat_deleted`, `normal_done`) and gate stale-save behavior accordingly.
  - On `user_stop`, either cancel upstream provider stream or explicitly suppress stale final save.

### P0: Multimodal persistence is incomplete

- Evidence:
  - Model supports image attachments in messages (`src/ai/model.rs:278`).
  - Provider payload supports image attachments (`src/ai/providers.rs:228`, `src/ai/window.rs:2819`).
  - Storage insert/select schema paths do not include images (`src/ai/storage.rs:560`, `src/ai/storage.rs:623`, `src/ai/storage.rs:651`).
  - Load path sets `images: Vec::new()` (`src/ai/storage.rs:821`).
- Impact:
  - Image context is lost after app restart or chat reload.
  - Historical replay/regeneration can diverge from original conversation.
- Recommendation:
  - Add `messages.images_json` (or normalized `message_images` table), with migration.
  - Persist/restore images in `save_message_internal`, `get_chat_messages`, `get_recent_messages`, `row_to_message`.

### P1: Streaming architecture has avoidable O(n) cloning and duplicate implementations

- Evidence:
  - AI window stream loop appends chunks in thread + UI polls every 50ms, cloning full buffer (`src/ai/window.rs:2861`, `src/ai/window.rs:2894`, `src/ai/window.rs:2958`).
  - Chat prompt has similar thread + poll + full clone reveal loop (`src/prompts/chat.rs:1621`, `src/prompts/chat.rs:1667`).
  - Chat prompt re-renders markdown on each reveal step (`src/prompts/chat.rs:1733`, `src/prompts/chat.rs:1737`).
- Impact:
  - Increased CPU and allocation pressure for long responses.
  - More lock contention and UI jitter under heavy token throughput.
- Recommendation:
  - Replace full-buffer polling with channel/event-based incremental deltas.
  - Coalesce tiny chunks in provider layer (e.g., 25-50ms flush window).
  - Share one streaming engine across `window.rs` and `prompts/chat.rs` to remove drift.

### P1: Claude persistent session manager locks too broadly

- Evidence:
  - Session map mutex is acquired (`src/ai/session.rs:231`) and `session.send_message` is called while the lock is still held (`src/ai/session.rs:281`).
- Impact:
  - One long session call can block all other session map operations.
  - Reduced parallelism and head-of-line blocking under multi-chat/multi-script usage.
- Recommendation:
  - Move session ownership out of single map lock during send path.
  - Store per-session handles behind independent synchronization so map lock is held only for lookup/insert/remove.

### P1: Attachment UX is only partially integrated

- Evidence:
  - `pending_attachments` state and picker UI exist (`src/ai/window.rs:562`, `src/ai/window.rs:2397`, `src/ai/window.rs:7555`).
  - `submit_message` currently processes only `pending_image` (`src/ai/window.rs:2677`, `src/ai/window.rs:2729`).
  - Clipboard image paste has a TODO placeholder (`src/ai/window.rs:2095`).
- Impact:
  - Users can select attachments in UI that are never sent to providers.
  - Perceived feature reliability is reduced.
- Recommendation:
  - Define explicit attachment capability matrix per provider.
  - Either disable unsupported attachment types in UI or convert them (e.g., OCR/text extraction) before send.
  - Complete clipboard image integration or remove command until ready.

### P1: Search behavior mismatch between AI window and storage capabilities

- Evidence:
  - AI window search filters only by title substring (`src/ai/window.rs:1589`, `src/ai/window.rs:1595`).
  - Storage has title + message-content FTS/LIKE search (`src/ai/storage.rs:451`, `src/ai/storage.rs:479`, `src/ai/storage.rs:509`).
- Impact:
  - Users cannot find chats by remembered message content from the sidebar.
  - Behavior differs from documented/stored capability.
- Recommendation:
  - Use `storage::search_chats` for sidebar search with debounce and fallback.
  - Keep title-only fast path only if query is short and local cache is warm.

### P1: Regenerate is destructive before success

- Evidence:
  - Last assistant message is deleted from memory/storage before new stream starts (`src/ai/window.rs:3220`, `src/ai/window.rs:3221`, `src/ai/window.rs:3227`).
- Impact:
  - If regeneration fails, user loses prior valid answer.
- Recommendation:
  - Keep original assistant response until replacement succeeds.
  - Convert regenerate to branch/variant model (modern chat UX pattern).

### P2: Storage parsing is lossy and hides corruption

- Evidence:
  - Invalid IDs/timestamps/roles are silently defaulted (`src/ai/storage.rs:767`, `src/ai/storage.rs:808`, `src/ai/storage.rs:812`).
- Impact:
  - Corrupt rows become plausible-but-wrong data.
  - Harder forensic debugging and correctness guarantees.
- Recommendation:
  - Use strict parse with structured warning/error events including row identifiers.
  - Quarantine invalid rows or skip with explicit telemetry.

### P2: Model list hygiene issues

- Evidence:
  - Duplicate `openai/o3` in Vercel model list (`src/ai/providers.rs:1126`, `src/ai/providers.rs:1134`).
  - Duplicate Anthropic Sonnet model in defaults (`src/ai/config.rs:314`, `src/ai/config.rs:321`).
- Impact:
  - Confusing dropdown options and inconsistent default behavior.
- Recommendation:
  - Deduplicate model arrays at source and add tests asserting uniqueness per provider.

### P2: Privacy/intent guard for clipboard-based starter

- Evidence:
  - "Summarize clipboard" starter injects clipboard text directly into prompt (`src/prompts/chat.rs:1040`, `src/prompts/chat.rs:1049`).
- Impact:
  - Sensitive clipboard content can be sent with one click.
- Recommendation:
  - Add preview/confirm affordance and optional redaction toggle for long or secret-looking content.

## Integration Pattern Improvements

1. Create a shared `AiConversationEngine` used by both `src/ai/window.rs` and `src/prompts/chat.rs`.
2. Move streaming state to typed state machine:
   - `Idle`
   - `Streaming { chat_id, generation, started_at, partial, cancellation }`
   - `Error { kind, message, retryable }`
3. Introduce typed stream events over channel:
   - `Chunk { text }`
   - `Done { final_text }`
   - `Error { detail }`
   - `Cancelled { reason }`
4. Add provider capability traits:
   - `supports_images`
   - `supports_tools`
   - `supports_resume`
   - `supports_cancel`

## Modern Chat UX Gap Checklist

High-value additions after P0/P1 fixes:

- Response branching/variants instead of destructive regenerate.
- Continue generation from partial stop.
- Per-message retry that reuses the same user turn without duplication.
- Message-content sidebar search parity.
- Persistent token/cost metadata per assistant message.
- Explicit streaming indicator/eta exposed to SDK (`aiGetStreamingStatus`) and UI.

## Suggested Implementation Plan

### Phase 1 (Correctness first)

- Wire SDK handlers to live runtime state.
- Fix retry semantics to avoid duplicate user messages.
- Add stream stop reason + suppress stale orphan save on user stop.
- Add persistence migration for message images.

### Phase 2 (Performance and shared abstractions)

- Replace poll+clone streaming loops with channel-based incremental updates.
- Extract shared streaming engine between AI window and chat prompt.
- Narrow locking scope in Claude session manager.

### Phase 3 (UX parity)

- Enable content search in sidebar via storage search APIs.
- Non-destructive regenerate with variants.
- Attachment pipeline completion and provider capability-aware UI.

## Test Additions Required

1. `test_ai_get_streaming_status_reflects_live_window_state`
2. `test_retry_after_error_does_not_duplicate_user_message`
3. `test_stop_streaming_does_not_persist_orphaned_full_response_after_user_stop`
4. `test_message_images_round_trip_through_storage`
5. `test_sidebar_search_matches_message_content`
6. `test_claude_session_manager_allows_parallel_sessions_without_global_send_lock_blocking`
7. `test_provider_model_lists_are_unique`

## Risks / Known Gaps

- Migrating message schema for images needs backward-compatible migration and fallback handling for existing DBs.
- Stream cancellation support depends on provider-specific APIs/transport; some providers may only support best-effort local cancel.
- Shared engine extraction touches two large files (`src/ai/window.rs`, `src/prompts/chat.rs`) and should be staged to avoid regressions.

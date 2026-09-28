# Backgrounded AI Sessions

**Status:** the store and the "Conversations" section are built. Leaving a
conversation never asks a question.
**Owner surfaces:** the model's chat, agents, Flow chat.

## The idea, in the user's words

> Clicking away should auto-background, and a backgrounded session becomes the
> **first option in the main menu**.

## 1. One store

`BackgroundedSessionStore` (`src/ai/conversations.rs`) is the one owner of which
AI sessions are backgrounded. It lives on app state, outside `current_view`,
because losing focus resets the view. Surface caches (warm leases, prewarmed
entities) may keep provider resources alive, but they never claim a session on
their own or produce their own main-menu rows.

Rows are ordered by `last_activity` descending, then by tagged stable id, so
returning to an older session moves it to the top.

## 2. Leaving a session never asks

Everything happens in the one window, so there is nothing to answer:

- **Esc** goes back one step. In the model's chat, Esc gives the sentence back to
  the input. An agent's view parks in the strip above the capsules, where the
  agent keeps working; **⌘J** opens whatever needs you first, then the newest
  task (`crates/sk-one-window/src/strip.rs`).
- **Flow sessions** background on plain Escape; `⇧⌘⎋` terminates.
- **Clicking away** hides the window; it never asks what to do with the
  session.
- **Past chats** are rows under a "Past chats" header, newest first; ↵ reopens
  one. Asking for "my last chat" opens it too.

A new surface stays inside the main window. A separate panel would take key
status, fire the focus-loss path underneath it, and background the session while
the user is still deciding.

## 3. The main-menu row

Backgrounded sessions lead the launcher as one flat **"Conversations"** section
(`src/scripts/grouping.rs`), covering every live or backgrounded session. Rows
reuse `SearchResult::Flow(FlowMatch)` with a typed `ConversationRowTarget`;
Enter resumes. The header shows only when there is something to show.

The section is suppressed while menu syntax or the spine owns the list; those
modes are explicitly task-focused.

## 4. Open question

Quick AI sessions are memory-only and never reopen a prior transcript
(`embedded_cache_reuse_allowed`). Letting them appear in Conversations until the
app quits (with an idle timeout) would change what Quick AI *is*, so it needs an
explicit yes before it is built.

## 5. Glass Motion Calibration Lock

Hiding runs the locked fade (`close_and_reset_window`, ordered out after
135 ms). Backgrounding reuses it; do not add a new animation or retune the delay
(`CLAUDE.md` "Glass Motion Calibration Lock").

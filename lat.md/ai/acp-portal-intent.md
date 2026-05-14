# ACP Portal Intent

This page documents the shared ACP portal object that now drives preview text, portal launch, return, and cancel behavior from one contract.

## Shared Intent

`[[src/ai/acp/portal_contract.rs]]` defines `AcpPortalIntent`, `AcpPortalLaunchContract`, and `AcpPortalReplacementTarget` as the canonical ACP portal state.

`[[src/ai/acp/view.rs]]` now derives one focused-inline intent and uses it for preview text, portal opening, and portal return instead of recomputing separate kind, query, and edit heuristics in each path.

## Exact Replacement Target

Portal return must only replace the original inline token when the current composer text still matches that exact token.

`apply_portal_replacement(...)` compares the current text segment against the staged `original_text`. On match it replaces the original range, and on mismatch it inserts at the saved fallback cursor so unrelated text survives portal reentry.

## Synthetic pasted mentions

Synthetic pasted mentions still use the shared portal intent path, but they stay preview-only instead of masquerading as reopenable file portals.

`[[src/ai/acp/portal_contract.rs]]` now classifies pasted-text alias tokens and `@img:pasteN` before generic portal lookup, while `[[src/pasted_text.rs]]` and `[[src/pasted_image.rs]]` keep the token-shape helpers that make that carve-out explicit.

## Host Query Seeding

Main ACP, detached ACP, and Notes-hosted ACP must all seed portal queries from the staged contract instead of host-local copies.

`[[src/app_impl/attachment_portal.rs]]`, `[[src/ai/acp/chat_window.rs]]`, and `[[src/notes/window/acp_host.rs]]` now read staged query text from `[[src/ai/acp/view.rs]]` accessors backed by `pending_portal_session.contract.query`, while detached ACP and Notes remain history-only hosts for this cycle.

Notes-hosted ACP reads that staged query from the originating embedded ACP view captured by the portal callback, not from whatever `NotesApp.embedded_acp_chat` happens to hold when the popup opens.

## Host transitions

Host cleanup can close menus and popups, but it must not drop a staged portal contract before the user cancels or accepts the portal.

`[[src/ai/acp/view.rs]]` now leaves `pending_portal_session` intact during `prepare_for_host_hide()`, and `[[src/notes/window/acp_host.rs]]` reopens Notes history portals against the originating ACP view instead of re-looking up whatever chat happens to be embedded later.

When Notes refuses an unsupported portal or history-popup opening fails, it calls `cancel_pending_portal_session(...)` for the staged kind so ACP returns to an idle terminal state instead of keeping an orphaned portal contract.

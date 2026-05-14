---
lat:
  require-code-mention: true
---
# ACP Portal Contract

These tests lock the shared ACP portal intent, exact replacement target, and host query seeding so portal behavior stays consistent across supported hosts.

## Exact replacement target

These specs cover portal return behavior when ACP reenters the composer after a portal selection.

### Replaces only the matching token

Portal return replaces the original inline token only when the current text at the staged range still matches that original token.

### Falls back when the token changed

If the original token was deleted or changed while the portal was open, ACP inserts the returned mention at the saved fallback cursor instead of overwriting unrelated text.

## Preview contract

These specs cover the focused-mention preview text that ACP shows before a portal is reopened.

### Preview copy names the replacement target

Portal-backed previews must say which exact token will be replaced so the composer hint matches the accept path, while compacting long raw tokens enough to keep the hint readable.

### Preview-only mentions stay explicit

Non-portal context types such as pasted text and synthetic pasted-image mentions must still show the replacement target, but they must remain clearly marked as preview-only.

## Host query seeding

These specs cover the query handoff from staged ACP portal state into each supported host surface.

### History portals keep the staged query across hosts

Main ACP, detached ACP, and Notes-hosted ACP must all seed history search from the staged contract query without widening detached or Notes portal capabilities.

### Notes history popup uses originating view

Notes-hosted ACP must open a history portal from the ACP view that requested it, even if the Notes host later replaces its cached embedded chat.

## Host transitions

These specs cover the staged portal state that must survive host-owned hide and reopen paths.

### Host callback refusal after attempted open

Portal open attempts must leave ACP idle when the host is missing, does not support the requested portal kind, or fails to open the picker after the request.

[[src/ai/acp/view.rs#PortalOpenResult]] records opened versus refused outcomes, and [[src/ai/acp/view.rs#AcpChatView#open_portal_contract_result]] refuses before staging when the capability check or callback lookup fails. Detached and Notes hosts cancel pending sessions on history-open failure.

### Host hide keeps the staged session

Closing ACP-local popups for a host transition must not clear the staged portal contract, and Notes must route the deferred history portal reopen back through the originating ACP view.

### Notes history host refusal clears staged session

If Notes refuses an unsupported portal kind or cannot open the history popup, the staged portal session must be cancelled so ACP does not remain in a terminal half-open state.

## Cmd+Enter origin parity

These specs cover the shared Agent Chat entry request used by launcher-style Cmd+Enter origins.

### Canonical entry request

Launcher, File Search, Actions, plugin-skill, dictation, and compatible host paths must route through [[src/app_impl/tab_ai_mode/acp_entry.rs#AcpEntryRequest]] or a host-owned equivalent.

### Origin matrix

Every origin must preserve its target thread, context staging shape, seed policy, return-origin behavior, label, and HUD copy from the Agent Chat entry matrix.

### Return-origin restore

Closing Agent Chat after a launcher-style entry must restore the originating surface and focus target instead of falling back to a generic Script List reset.

### Detached reuse

When a detached Agent Chat window is open, launcher-style entry paths focus or stage into that detached thread instead of creating or mutating a second embedded conversation.

### File Search context staging

File Search Cmd+Enter must stage the selected file/query through the shared entry request and must not call `thread.add_context_part` directly from `src/file_search`.

### Actions dialog restore focus

Actions Cmd+Enter must stage an actions payload through the shared entry request and restore ACP composer focus when the actions dialog closes.

### Notes host isolation

Notes Cmd+Enter must use the Notes-owned embedded ACP view and must not mutate the main launcher's cached embedded Agent Chat.

### Label and HUD parity

Entry-point footers and shortcut HUDs should display Agent Chat labels from the central ACP label module rather than stale ACP Chat, Tab AI, or bare AI copy.

## Plugin skill target-thread contract

These specs cover the relationship between main-menu plugin skills, slash-picker skills, detached reuse, and agent switching.

### Main menu and slash picker are equivalent

Selecting a plugin skill from the main menu and accepting the same skill from the slash picker must produce one leading slash token and one SkillFile context part.

### Detached reuse stages into detached thread

When detached Agent Chat is open, main-menu plugin skill selection must stage into the detached thread rather than an embedded cache.

### Agent switch revalidates staged skill

Switching agents while a plugin SkillFile is staged must revalidate or explicitly remove that staged skill while preserving the composer draft.

## Conversation export dedupe

These specs cover the single ACP conversation export builder and its stable dedupe behavior.

### Single export path

Embedded, detached, Notes-hosted, action-triggered, File Search, and plugin-skill submissions must export ACP conversation state through one builder.

### Stable context part dedupe

Exported context parts must dedupe by a stable id so a plugin SkillFile staged by slash picker and main-menu handoff appears once.

### No duplicate seeded user message

Seeded launcher text must not appear as both composer seed and a duplicated exported user message.

## Clipboard history portal

These specs lock the clipboard portal state machine around host refusals, `kit://clipboard-history?id=...` round-tripping, and exact-range replacement on accept.

### Host-aware refusal leaves ACP idle

When no host callback exists, or the host disallows clipboard portals, ACP refuses the request before staging so no dead portal session survives.

### Round-trip accepts kit id URIs and preserves the inline token

Clipboard history parts may carry `kit://clipboard-history?id=...` while still round-tripping to the canonical `@clipboard` inline token and reopening with the resolved entry query.

### Attach replaces exact range and terminal states clear to idle

Accepting a clipboard portal replaces the original token range when unchanged, and accepted, cancelled, or orphaned terminal states all clear back to idle.

## Dictation history portal

These specs lock dictation portals to preserve the durable history id while still opening the picker unfiltered.

### Host-aware refusal leaves ACP idle

When no host callback exists, or the host disallows dictation portals, ACP refuses the request before staging so no dead portal session survives.

### Round-trip preserves history id but opens unfiltered

Accepted dictation attachments stay `kit://dictation-history?id=...`, inline tokens stay `@dictation:<id>`, and the picker filter resolves to the empty string.

### Production URI construction pairs with inline token

[[src/ai/context_mentions/mod.rs#dictation_history_part_for_entry]] emits a `kit://dictation-history?id={id}` URI whose id survives [[src/ai/context_mentions/mod.rs#part_to_inline_token]] verbatim as `@dictation:{id}`.

Drift between the two helpers would break acceptance re-round-tripping without triggering the picker-level tests. This invariant is the contract the dictation portal round-trip relies on.

### Attach replaces exact range and terminal states clear to idle

Accepting a dictation portal replaces the exact original mention range when unchanged, and accepted, cancelled, or orphaned terminal states all clear back to idle.

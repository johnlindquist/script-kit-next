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

## Host transitions

These specs cover the staged portal state that must survive host-owned hide and reopen paths.

### Host hide keeps the staged session

Closing ACP-local popups for a host transition must not clear the staged portal contract, and Notes must route the deferred history portal reopen back through the originating ACP view.

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

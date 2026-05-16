# 010 Root Unified Search ACP History

This chapter maps saved ACP conversation rows in root launcher search, where conversations appear as passive AI Conversations results.

## Executive Summary

Root ACP History lets users find and resume saved Agent Chat conversations from the main launcher. It appends passive AI Conversations rows for eligible root queries and supports source-filtered search or browse through `ai:` and `conversations:`.

This feature owns `SearchResult::AcpHistory(AcpHistoryMatch)`, AI Conversations root metadata rows, `unifiedSearch.acpHistory` config, Conversations source-filter integration, passive grouping, stable identity, Enter-to-resume wiring, and the small root action set for copying metadata.

It does not own ACP streaming, cancellation, composer context, model/agent selection, detached-window policy, saved-conversation storage lifecycle beyond root-search cache expectations, or ACP History built-in browser UI beyond sharing the resume helper.

## Human Capabilities

| Capability | User story | Contract |
|---|---|---|
| Passive conversation search | Type an eligible root query and see saved conversations under AI Conversations. | Rows never outrank commands, scripts, apps, skills, windows, actions, or root file rows. |
| Explicit source search | Type `ai: plan` or `conversations: plan`. | Search only AI Conversations for stripped text. |
| Source-only browse | Type `ai:` or `conversations:`. | Browse recent saved conversation metadata; ordinary empty root stays clean. |
| Resume conversation | Press Enter on an AI Conversations row. | Calls shared ACP history resume helper. |
| Metadata actions | Open actions and copy title, session id, or preview. | Action context is captured from the root row. |
| Recognizable rows | See title, preview/message-count subtitle, source AI Conversations, type AI Conversation, and default action Resume Conversation. | Stable key is `acp-history/{session_id}` and command id is non-bindable. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `AcpHistoryMatch` | Root row metadata for a saved conversation. | Carries session id, title, preview/subtitle, first message, count, score metadata. |
| AI Conversations source | Root passive source for saved ACP history. | Source name `AI Conversations`, type label `AI Conversation`. |
| Source filter | `RootUnifiedSourceFilter::Conversations`. | Activated by `ai:` or `conversations:`. |
| Summary index | `acp-history.jsonl`. | Searchable metadata index. |
| Full conversation payload | `acp-conversations/{session_id}.json`. | Loaded by shared resume path, not root grouping. |
| Passive frame | Frozen per-query result vector with ACP history options. | Cache warmers cannot mutate active rows or selection. |
| Shared resume helper | `resume_acp_conversation_from_history`. | Root rows and ACP History browser share resume behavior. |
| Non-bindable identity | Stable key exists, launcher command id is `None`. | Saved conversations are selectable/actionable but not alias/shortcut commands. |

## Entry Points

| Entry | Example | Result |
|---|---|---|
| Ordinary enabled query | `design`. | Passive AI Conversations section may appear. |
| Spaced source query | `ai: refactor`, `conversations: refactor`. | Conversations-only search for stripped text. |
| Attached source query | `ai:plan`, `conversations:plan`. | Same source-filter behavior. |
| Source-only browse | `ai:`, `conversations:`. | Recent saved conversation metadata rows. |
| Row activation | Enter on selected AI Conversation row. | Resumes saved ACP conversation. |
| Root actions | Cmd+K on selected row. | Resume, copy title, copy session id, copy preview. |

## State Model

| State | Meaning |
|---|---|
| Main launcher ScriptList | Root search input, grouped rows, selection, source filters, preflight receipts. |
| Ordinary passive ACP History | AI Conversations rows append for eligible queries when config and cache allow. |
| Conversations source-filter mode | `ai:` / `conversations:` suppresses disallowed sources and searches Conversations. |
| Source-only browse | Empty stripped text with Conversations source returns recent conversation metadata. |
| Root passive query frame | Includes query, advanced state, source filters, and ACP history options; carries `acp_history_hits`. |
| ACP Chat after resume | Embedded or detached Agent Chat state handled by shared ACP logic. |
| ACP History built-in browser | Adjacent built-in surface that shares storage/search/resume concepts. |

## Data Model

| Data | Location | Root row use |
|---|---|---|
| Summary index | `src/ai/acp/history.rs`, `acp-history.jsonl`. | Searchable metadata source. |
| Conversation payload | `acp-conversations/{session_id}.json`. | Loaded by resume helper. |
| `AcpHistoryEntry` | Timestamp, first message, message count, session id, title, preview, search text. | Root row title/subtitle/actions. |
| Legacy entries | Back-filled on read. | Normalized before ranking. |
| `search_text` | Normalized/bounded text. | Prevents legacy multi-megabyte fields from hurting root typing. |
| `ACP_HISTORY_INDEX_CACHE` | Mtime/size-signature cache. | Ordinary root search cache. |
| `ACP_HISTORY_REFRESH_IN_FLIGHT` | Background warmer guard. | Prevents duplicate warmers. |

## Query Eligibility

| Query/config state | Ordinary behavior | Explicit `ai:` / `conversations:` behavior |
|---|---|---|
| Empty ordinary root | No ACP History rows. | Browse recent saved metadata. |
| Query below `minQueryChars` | No rows. | Explicit source can raise caps/lower min. |
| `unifiedSearch.enabled=false` | No rows. | Source semantics should respect global source policy where configured. |
| `unifiedSearch.acpHistory.enabled=false` | No ordinary rows. | Explicit source can opt in according to source-filter rules. |
| Predicate advanced query | No passive rows. | Source-only browse remains separate. |
| Excluded Conversations source | No rows. | Exclusion wins. |
| Cold/stale cache | Current frame may have no rows; warms future frame. | Direct source path can read metadata. |
| Missing saved JSON on resume | Shared helper handles failure/fallback. | Exact user-facing fallback remains an open question. |

## User Workflows

### Ordinary Passive Search

The user types an eligible normal root query. If ACP History is enabled and cached hits exist, AI Conversations rows append after primary/root-file results and before fallback handoff rows.

Rows show title from display helpers, preview/message count subtitle, type/source metadata, default action Resume Conversation, and stable key `acp-history/{session_id}`. They are passive and non-bindable.

### Explicit Conversations Search

The user types `ai: refactor` or `conversations: refactor`. The source head is stripped, source filters include Conversations, and only allowed Conversation rows/statuses are shown. Attached forms such as `ai:plan` should parse the same way.

### Source-Only Browse

The user types `ai:` or `conversations:` without stripped text. This browses recent saved conversation metadata in source-filter mode. Ordinary empty launcher input still does not show saved conversations.

### Resume Selected Conversation

The user selects an AI Conversation row and presses Enter. The root path dispatches `SearchResult::AcpHistory` to `resume_acp_conversation_from_history(&session_id, first_message, cx)`. The shared helper opens or reuses Agent Chat and loads saved messages or fallback state according to ACP ownership.

### Root Actions

The user opens the shared actions dialog on a focused ACP History row. The root action subject captures the `AcpHistoryEntry`; visible actions include resume, copy title, copy session id, and copy preview. Root search does not expose attach-summary in this pass.

## Interaction Matrix

| Interaction | Context | Expected behavior | Proof |
|---|---|---|---|
| Type ordinary eligible query. | Config enabled and cache warm. | AI Conversations section appears passively. | Role/source/type/key receipts. |
| Type `ai: refactor`. | Explicit source. | `sourceFilters=["conversations"]`, stripped text `refactor`. | Preflight source filters. |
| Type `ai:`. | Source-only browse. | Recent saved conversation metadata rows/statuses. | Empty stripped text plus Conversation rows. |
| Type short ordinary query. | Below min chars. | No ACP History rows. | Eligibility tests. |
| Type predicate query. | Advanced query. | No passive ACP History rows. | Grouping/source audit. |
| Wait for cold cache. | Ordinary cache cold/stale. | Current frame stable; background warmer does not publish. | Fingerprint unchanged. |
| Press Enter. | Selected ACP History row. | Shared resume helper opens/resumes Agent Chat. | Source audit plus `getAcpState`. |
| Open actions. | Selected ACP History row. | Metadata copy/resume actions visible; captured subject used. | `actionsDialog` receipt. |
| Resume with detached ACP open. | Detached Agent Chat exists. | Shared helper reuses/focuses rather than duplicating. | ACP/window state. |

## Visual States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Ordinary search with hits. | Primary/root-file rows first; AI Conversations header and rows; fallback below. | Main launcher filter/list. | `role=rootPassive`, `typeLabel=AI Conversation`, `sourceName=AI Conversations`, key `acp-history/{session_id}`. |
| Source-filtered browse/search. | AI Conversations rows plus source-filter status/chips; disallowed sources absent. | Main launcher filter/list. | `sourceFilters=["conversations"]`, filter indicators, cached source statuses. |
| Focused ACP History row. | Selected conversation title with preview/message-count subtitle. | Main list selected row. | `selectedResultKey`, `selectedResultRole=rootPassive`. |
| Root actions dialog. | Resume/copy title/copy session id/copy preview. | Actions dialog. | `visibleActions`, `contextStableKey`. |
| Resumed Agent Chat. | Agent Chat with saved conversation loaded or fallback seeded from first message. | ACP Chat view/composer. | `getAcpState` session/message evidence. |
| ACP History built-in browser. | Separate built-in list/empty state. | Built-in ACP History view. | Adjacent receipts, not owned by root rows. |

## Ordering And Grouping Boundaries

- ACP History rows are passive launcher results, never primary/promoted results.
- Rows insert after primary rows and root Files.
- Rows must not split the Files section or the Search Files continuation row.
- Fallback handoff rows remain below the passive section.
- Custom passive source order can reorder local passive sections, but cannot move them above primary rows or root Files.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Ordinary empty root. | No ACP History rows. |
| Ordinary short query. | No rows below min chars. |
| `unifiedSearch.enabled=false`. | No rows. |
| `unifiedSearch.acpHistory.enabled=false`. | No ordinary rows. |
| Predicate advanced query. | No passive rows. |
| `ai:` with no saved conversations. | Source-filter empty/no-results state. |
| Cold/stale cache. | No active-frame publish; future frames may show rows. |
| Missing/deleted conversation JSON on resume. | Shared helper handles error/fallback; exact UX open. |
| Known action id without captured subject. | No-op/return, no generic script fallback. |

## Code Ownership

| Area | Source anchors |
|---|---|
| ACP history storage/search | `src/ai/acp/history.rs`, `acp-history.jsonl`, `acp-conversations/{session_id}.json` |
| Config | `UnifiedSearchAcpHistoryConfig`, `UnifiedSearchConfig::acp_history_section_options`, TypeScript schema `acpHistory` |
| Root grouping | `append_root_acp_history_section`, `append_root_passive_section` |
| Source filters | `RootUnifiedSourceFilter::Conversations`, `ai:`, `conversations:` parser cases |
| Result identity | `AcpHistoryMatch`, `SearchResult::AcpHistory`, `acp-history/{session_id}`, `launcher_command_id => None` |
| Resume path | `src/app_impl/selection_fallback.rs`, `src/render_builtins/acp_history.rs#ScriptListApp#resume_acp_conversation_from_history` |
| Root actions | `RootUnifiedActionSubject::AcpHistory`, `RootUnifiedResultAction::AcpHistoryResume` |
| Tests/source audits | `root_unified_acp_history_contract.rs`, `root_unified_source_filters_contract.rs`, `root_unified_config_schema_parity_contract.rs`, `root_unified_passive_snapshot_contract.rs`, `root_unified_source_actions_contract.rs` |

## Invariants And Regression Risks

- ACP History root rows are passive, never primary or promoted.
- Rows insert after primary/root file rows and before fallback handoff rows.
- Rows must not split the Files section or Search Files continuation row.
- Rows are stable but non-bindable.
- Enter uses only the shared ACP history resume helper.
- Ordinary passive root search is cache-only and frame-frozen.
- Explicit Conversations source filters can browse with empty stripped text.
- Config/schema/default/source-audit parity includes ACP History.
- Actions dialog captures ACP subject before generic script fallback.
- Root search does not expose attach-summary actions for ACP History in this pass.

## Verification Recipes

### Source And Config Contracts

Run:

```bash
cargo test --test source_audits root_unified_acp_history_contract -- --nocapture
cargo test --test source_audits root_unified_source_filters_contract -- --nocapture
cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture
cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture
cargo test --test source_audits root_unified_search_stability_contract -- --nocapture
cargo test --test source_audits root_unified_source_actions_contract -- --nocapture
cargo test --test menu_syntax_source_filters -- --nocapture
```

Check:

- Rows are passive/non-bindable and source-labeled correctly.
- Files section and Search Files continuation are not split.
- `ai:` and `conversations:` parse and isolate sources.
- Config/schema/defaults stay in parity.
- Background cache warmers do not mutate active frames.
- Root actions are scoped to captured ACP History subjects.

### Runtime State Proof

Use state-first automation:

- Set an eligible normal filter and assert visible result role `rootPassive`, type `AI Conversation`, source `AI Conversations`, and key `acp-history/{session_id}`.
- Set `ai: conversation` and assert `sourceFilters=["conversations"]`, expected rows/statuses, and no disallowed sources.
- Compare root passive frame and visible result fingerprints across cold/warm cache behavior.
- Press Enter on a selected ACP History row and assert ACP state/session/message evidence through `getAcpState`.
- Open actions and assert visible action ids/labels plus `contextStableKey`.

## Agent Notes

- Do not prove root ACP History by testing only the dedicated ACP History browser.
- Always verify Files section boundaries when adding or changing passive ACP grouping.
- Use `getAcpState` for resume results instead of screenshots when possible.
- Treat shortcut/alias binding for saved conversation rows as a regression.
- Do not add attach-summary actions under this root row without updating this contract and tests.

## Related Features

- [001 Main Menu](./001-main-menu.md) owns root grouping, selection, source filters, and fallback boundaries.
- [003 Agent Chat Context](./003-agent-chat-context.md) owns ACP composer/context behavior after a conversation is resumed.
- [011 Root Result Actions](./011-root-source-actions.md) owns shared action-popup behavior for root rows.

## Raw Oracle References

- [Prompt](../raw-oracle/010-root-acp-history/prompt.md)
- [Bundle map](../raw-oracle/010-root-acp-history/bundle-map.md)
- [Answer](../raw-oracle/010-root-acp-history/answer.md)
- [Full output log](../raw-oracle/010-root-acp-history/output.log)
- [Session metadata](../raw-oracle/010-root-acp-history/session.json)

## Open Questions And Gaps

- The exact user-facing fallback when saved conversation JSON is missing should be runtime-proven.
- Source-filtered empty-state wording for `ai:` is generic and not pinned specifically for ACP History.
- Add/keep an explicit audit that no root ACP History attach-summary action exists.
- Custom passive source order with ACP History should get focused grouping coverage to preserve Files/fallback boundaries.
- A named agentic resume proof should seed saved ACP history, select an `ai:` row, press Enter, and assert ACP session/message state.

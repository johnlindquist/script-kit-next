# 038 Agent Skills and AI Context Catalog

Agent skills and AI context are typed, previewable, staged inputs to ACP Agent Chat, not loose pasted strings or hidden side effects.

Raw Oracle reference: [answer](../raw-oracle/038-agent-skills-ai-context-catalog/answer.md), [prompt](../raw-oracle/038-agent-skills-ai-context-catalog/prompt.md), [bundle map](../raw-oracle/038-agent-skills-ai-context-catalog/bundle-map.md), [full log](../raw-oracle/038-agent-skills-ai-context-catalog/output.log), [session metadata](../raw-oracle/038-agent-skills-ai-context-catalog/session.json).

## Executive Summary

Feature 038 covers the agent-facing skill catalog and the AI context catalog. It spans plugin skill discovery, slash/main-menu skill staging, `AiContextPart` variants, `kit://context` MCP resources, context preview metadata, attachment portals, focused-target handoff, SDK `aiStartChat` parts, and submit-time resolution receipts.

The core contract is that the composer displays compact tokens and chips while preserving typed pending context parts. Resolution happens at submit time, produces deterministic prompt blocks, and records exactly what succeeded or failed in `ContextResolutionReceipt`.

## What Users Can Do

- Discover repo-local and plugin-owned skills from `skills/<skill_id>/SKILL.md`.
- Search skills by title, id, plugin title, and description.
- Attach a skill from slash picker, main menu, or skill search without pasting the full skill body into the composer.
- Attach MCP resources such as `kit://context?profile=minimal`.
- Attach files, focused targets, ambient context chips, and text blocks as typed context parts.
- Preview context chips without resolving large resource bodies.
- Use attachment portals for files, clipboard history, dictation history, notes, ACP history, scripts, scriptlets, and skills.
- Submit ACP messages with ordered context parts and receive resolution receipts.
- Start chats from SDK code with ordered typed parts.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| Agent skill catalog | Plugin skill inventory rooted at `~/.scriptkit/plugins/*/skills/<skill_id>/SKILL.md`. | `src/plugins/skills.rs` |
| Skill search | Fuzzy browse/search over title, skill id, plugin title, and description. | `src/scripts/search/skills.rs` |
| `AiContextPart` | Tagged attachment enum for resource URIs, files, skills, focused targets, ambient context, and text blocks. | `src/ai/message_parts.rs` |
| `pending_context_parts` | Composer-side source of truth for staged context attachments. | `src/ai/message_parts.rs` |
| `ContextResolutionReceipt` | Submit-time receipt for attempted, resolved, failed, and prompt-prefix context. | `src/ai/message_parts.rs` |
| `kit://context` | Main AI-facing MCP desktop context resource. | `src/mcp_resources/mod.rs` |
| `kit://context/schema` | Self-describing schema resource for context profiles and fields. | `src/mcp_resources/mod.rs` |
| `ContextPreviewInfo` | Synchronous preview metadata for staged context chips. | `src/ai/window/context_preview.rs` |
| Attachment portal | Host return flow that opens context search surfaces and converts accepted rows into `AiContextPart`. | `src/app_impl/attachment_portal.rs` |
| Focused target | Explicit surface-native target handed to ACP instead of inferred ambient context. | `src/ai/message_parts.rs` |
| SDK parts | Ordered `aiStartChat(..., { parts })` typed-context contract. | `tests/sdk/test-ai-context-parts.ts` |

## Entry Points

| Entry point | User intent | Expected target |
|---|---|---|
| ACP slash picker | Attach a skill or command context token | Visible slash token plus staged `SkillFile` or resource part |
| Main menu skill row | Start ACP with selected skill context | ACP composer with `/{skill}` token and pending skill part |
| Context picker / @mention | Attach MCP-backed context | `ResourceUri` context part |
| Attachment portal | Choose files, history, notes, scripts, scriptlets, or skills | Accepted row becomes `AiContextPart` |
| Focused target handoff | Ask about a concrete selected surface item | `FocusedTarget` part |
| Ask Anything ambient chip | Promote lightweight desktop context | `AmbientContext` display chip |
| Paste / raw text staging | Add explicit text snippets, logs, or URLs | `TextBlock` part |
| SDK `aiStartChat` parts | Programmatically start chat with context | Ordered typed parts submitted to ACP |
| MCP `kit://context` | Resolve desktop context resource | `<context>` prompt block |
| MCP `kit://context/schema` | Inspect context resource contract | Schema/example resource |

## User Workflows

### Browse And Attach A Skill

The user searches available skills from a skill search surface or slash picker. Discovery scans plugin skill folders, derives title and description from frontmatter or fallback content, and returns deterministic skill matches. Accepting a skill inserts a compact `/{slash-name}` token and stages an `AiContextPart::SkillFile`.

### Submit A Skill-backed ACP Message

The composer keeps the visible skill token small. On submit, the staged `SkillFile` resolves through the shared staged-skill prompt builder, contributes prompt context, and records its outcome in `ContextResolutionReceipt`.

### Attach MCP Context

The user chooses a context resource such as `kit://context?profile=minimal`. The pending part remains `ResourceUri` until submit, where MCP resource resolution produces a `<context>` block or records failure without erasing other successful parts.

### Use Attachment Portal

ACP opens a portal for file search, clipboard history, dictation history, notes browse, ACP history, scripts, scriptlets, or skills. The portal carries kind, query seed, original replace range, return view, focus target, and preview metadata. Attach consumes the session and inserts the typed context part; cancel clears the session and restores ACP.

### Hand Off A Focused Target

A surface opens ACP with an explicit `FocusedTarget` part. Submit-time resolution creates a deterministic focused-target context block with item source, item kind, semantic id, label, and metadata instead of guessing from ambient desktop state.

### Start Chat From SDK

SDK code calls `aiStartChat(..., { parts })` with ordered `resourceUri`, `filePath`, or other typed parts. The ordering must survive through prompt preparation because tests cover both resource-first and file-first flows.

### Preview Before Submit

The composer derives `ContextPreviewInfo` from part metadata so chips can render labels such as minimal/full/custom context, skill attachment owner, focused target, file path, or text block without resolving the full prompt body.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Browse skills | Skill search | Empty or filtered list | Type query | `fuzzy_search_skills` | Ranked `SkillMatch` rows | Skill search tests |
| Attach skill | Slash picker / skill row | Composer active | Enter / click row | `AiContextPart::SkillFile` staging | Visible slash token plus pending skill part | Composer state tests |
| Attach context resource | Context picker | Picker active | Enter / click row | `AiContextPart::ResourceUri` | Resource chip staged | Context picker tests |
| Attach file | File portal | Portal active | Enter / click row | `AiContextPart::FilePath` | File chip staged | Portal and file part tests |
| Attach focused target | Surface handoff | ACP opens | Handoff action | `FocusedTarget` part | Explicit target chip staged | Focused-target resolution tests |
| Add ambient context | Ask Anything chip | Composer active | Promote chip | `AmbientContext` part | Display-only chip staged | Preparation receipt tests |
| Add text block | Paste/text action | Composer active | Paste / attach text | `TextBlock` part | Text context staged | Context part resolution tests |
| Submit message | ACP composer | Pending parts present | Enter / send | `prepare_user_message_with_receipt` | Prompt prefix and receipts generated | Submission flow tests |
| Partial failure | Submit path | Mixed valid/invalid parts | Send | Resolver keeps successes | Message can send with failure receipt | Partial-failure tests |
| All failure | Submit path | No usable context | Send | Resolver blocks false success | Send blocked or raw-only path avoided | All-failure tests |
| SDK start chat | SDK script | Ordered parts | API call | `aiStartChat(..., { parts })` | Ordered typed parts delivered | SDK context-parts tests |
| Read context schema | MCP resource | Resource request | `kit://context/schema` | MCP resource handler | Schema returned | MCP resource tests |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Catalog unavailable | Startup or missing plugin dirs | Catalog ready, empty catalog | Missing skills directory is skipped, not fatal. |
| Catalog ready | Plugin scan complete | Skill browsing, skill selected | Results sorted by plugin id and skill id. |
| Skill browsing | Empty or non-empty query | Skill selected, catalog ready | Empty query returns browseable skills with score 0. |
| Skill selected | Search row accepted | Composer staging | `SKILL.md` path and slash name must be stable. |
| Composer staging | Slash/context/portal/focused-target attach | Previewing context, submitting, cleared | `pending_context_parts` is source of truth. |
| Previewing context | Pending parts changed | Composer staging, submitting | Preview derives from metadata and must not resolve full bodies. |
| Portal open | Attachment trigger | Portal accepted, portal canceled | Portal session carries kind, seed query, replace range, return view, and focus. |
| Portal accepted | Row accepted | Composer staging | Accepted row maps to a typed `AiContextPart`. |
| Portal canceled | Escape/cancel | Composer staging | Staged portal session is cleared before ACP resumes. |
| Submitting | Send action | Resolved success, partial failure, all failure | Part order must be preserved through resolution. |
| Resolved success | All parts resolved | Message sent | Receipt records attempted and resolved counts. |
| Partial failure | Some parts fail | Message sent with warnings | Successful context is retained. |
| All failure | No usable context | Send blocked or failure surfaced | False-success receipts are not allowed. |
| SDK ordered parts | SDK `aiStartChat` | Submitting | Parts preserve caller order. |
| MCP resource read | `ResourceUri` resolution | Resolved success, failure | Query URI and effective profile remain observable. |

## Visual And Focus States

- ACP composer with visible slash token for a staged skill.
- ACP composer with one or more context chips derived from `pending_context_parts`.
- Context picker rows for MCP-backed resources with non-empty labels.
- Skill search rows with title, owner/plugin label, slash name, and description-derived metadata.
- Attachment portal host temporarily showing file/history/notes/scripts/scriptlets/skills search surfaces.
- Portal return state restoring composer text, focus target, preview metadata, and originating surface width.
- Context preview chips for minimal, full, custom, file-backed, skill, focused-target, ambient, and text-block parts.
- Submit-time warning state when partial context resolution fails.
- Blocked/failure state when all context resolution fails and no usable prompt context remains.

## Keystrokes And Commands

| Key/command | Context | Behavior |
|---|---|---|
| Slash command | ACP composer | Opens slash picker and can stage a skill token/part. |
| `@` mention | ACP composer | Opens context picker paths that can create resource URI parts. |
| Enter | Skill search / picker row | Accepts selected skill or context item. |
| Escape | Attachment portal | Cancels portal and restores ACP state. |
| Send / Enter | ACP composer | Calls prompt preparation and context resolution for pending parts. |
| `kit://context?profile=minimal` | MCP resource URI | Resolves lean desktop context. |
| `kit://context?profile=full` | MCP resource URI | Resolves fuller desktop metadata. |
| `kit://context?diagnostics=1` | MCP resource URI | Resolves diagnostics wrapper with per-field status. |
| `kit://context/schema` | MCP resource URI | Returns resource schema/examples. |
| `aiStartChat(..., { parts })` | SDK | Starts ACP with ordered typed context parts. |
| `lat check` | Docs/contract changes | Validates wiki links and code references. |

## Actions And Menus

Skill-related actions should stage context, not paste content. Main-menu skill activation and slash picker skill acceptance both converge on the same model: a visible slash token plus a hidden typed `SkillFile` part that resolves at submit time.

Context actions should preserve source identity. Resource URIs stay resource URIs, files stay file paths, focused targets stay focused targets, and text snippets stay text blocks until the shared resolver produces prompt content and receipts.

## Automation And Protocol Surface

| Surface | Target/proof | Notes |
|---|---|---|
| Skill discovery | Plugin skill discovery/search tests | Proves `skills/<skill_id>/SKILL.md`, frontmatter fallback, deterministic ordering, and search ranking. |
| Context part serde | `tests/context_part_resolution.rs` | Proves tagged variants and resolution behavior. |
| Submit receipts | `tests/context_part_submission_flow.rs` | Proves success, partial failure, all failure, and prompt-prefix receipts. |
| Start chat parts | `tests/context_part_start_chat_flow.rs`, `tests/sdk/test-ai-context-parts.ts` | Proves ordered typed parts from SDK or ACP start paths. |
| Context picker | `tests/context_picker.rs`, `tests/context_part_composer_state.rs` | Proves picker item creation, pending context state, dedup, and removal. |
| Preflight | `tests/context_preflight.rs`, `tests/context_preflight_source_audits.rs` | Proves recommendation profile and nonblocking preflight behavior. |
| MCP resources | `tests/context_snapshot.rs`, `tests/transaction_trace_resources.rs` | Proves `kit://context`, schema, diagnostics, examples, and resource listing. |
| Runtime ACP | `getAcpState`, `getElements`, `waitFor`, `batch` | Use state-first receipts for UI/routing/focus changes before screenshots. |

## Data, Storage, And Privacy Boundaries

- Skill catalog discovery reads `SKILL.md` metadata and paths; it should not paste full skill bodies into composer text.
- `ResourceUri` parts can resolve desktop context and must preserve query/profile identity for diagnostics.
- File attachments can fall back to metadata-only when unreadable.
- `AmbientContext` is display-only and does not itself become prompt content.
- `FocusedTarget` is explicit user/surface intent and should not be replaced by ambient guessing.
- Preview metadata must stay cheap and should not mutate composer state.
- Context failure details live in receipts so agents can diagnose missing/unreadable context without silently losing successful parts.

## Error, Empty, Loading, And Disabled States

- Plugins without `skills/` directories are skipped.
- Skill entries without `SKILL.md` are skipped.
- Missing frontmatter title falls back to first H1, then skill id.
- Missing frontmatter description leaves description empty instead of inventing content.
- Empty skill query returns all skills with score 0 for browse mode.
- Duplicate context parts are deduped with mention-derived parts winning over pending duplicates.
- Unreadable file context falls back to metadata-only attachment where supported.
- Resource read failures are recorded in `ContextResolutionReceipt`.
- Partial failures keep successful context.
- All failures block false-success sends when no usable context remains.
- Detached ACP does not necessarily support every main-window portal kind.

## Code Ownership

| Area | Primary files | Notes |
|---|---|---|
| Skill discovery | `src/plugins/skills.rs` | Scans plugin skill dirs, parses frontmatter, builds `PluginSkill` records. |
| Skill search | `src/scripts/search/skills.rs` | Fuzzy search and browse behavior over skills. |
| Context model | `src/ai/message_parts.rs` | Defines `AiContextPart`, file helpers, merge/dedup, and resolution receipts. |
| Context preview | `src/ai/window/context_preview.rs` | Builds chip metadata from staged parts. |
| Context preflight | `src/ai/window/context_preflight.rs`, `src/ai/window/context_recommendations.rs` | Produces lightweight recommendation context and stale/blocked states. |
| ACP context | `src/ai/acp/context.rs` | ACP-side context handoff and composition. |
| Context picker | `src/ai/window/context_picker/*`, `src/ai/context_mentions/*` | Picker rows, mentions, and resource item creation. |
| Attachment portal | `src/app_impl/attachment_portal.rs` | Portal open/accept/cancel return flow. |
| MCP resources | `src/mcp_resources/*`, `src/mcp_protocol/*`, `src/mcp_server/*`, `src/mcp_script_tools/*` | Resource URI read/list/schema behavior. |
| Context snapshots | `src/context_snapshot/*` | Desktop capture options, schema version, and profile behavior. |
| Agent catalog | `src/agents/*` | Agent definitions adjacent to skill/context routing. |

## Invariants And Regression Risks

- Do not paste full skill bodies into the composer.
- `pending_context_parts` remains the composer attachment source of truth.
- Preserve typed `AiContextPart` variants instead of downcasting everything to strings or file paths.
- Preserve part order from picker, portal, and SDK through submit-time resolution.
- Resolve context at submit time and record `ContextResolutionReceipt`.
- Mentions win over pending duplicate parts during merge.
- Partial failures must retain successful context.
- All failures must not look like successful context sends.
- `AmbientContext` remains display-only unless separate capture context is staged.
- Portal attach/cancel must restore ACP state and clear staged portal sessions.
- `kit://context` and `kit://context/schema` must remain listed and self-describing MCP resources.
- Legacy `.claude/skills/*` names are not canonical routing names.

## Verification Recipes

```bash
lat check
git diff --check
cargo test --test context_part_resolution -- --nocapture
cargo test --test context_part_submission_flow -- --nocapture
cargo test --test context_part_start_chat_flow -- --nocapture
cargo test --test context_contract_end_to_end -- --nocapture
cargo test --test context_picker -- --nocapture
cargo test --test context_part_composer_state -- --nocapture
cargo test --test context_preflight -- --nocapture
cargo test --test context_preflight_source_audits -- --nocapture
cargo test --test context_snapshot -- --nocapture
cargo test --test transaction_trace_resources context_resource_still_listed_after_transaction_resources_added -- --nocapture
```

For runtime UI changes, use state-first ACP receipts (`getAcpState`, `getElements`, `waitFor`, `batch`) before screenshots. Screenshots are only needed for visual layout regressions, not for context model or receipt correctness.

## Agent Notes

- Do not assume skill attachment means composer text contains the skill body.
- To verify submit behavior, inspect the prepared message receipt and `ContextResolutionReceipt`.
- If context disappears, inspect `pending_context_parts`, merge/dedup receipts, and portal return state before debugging the renderer.
- This belongs primarily to `mcp-context-resources` and `acp-context-composer`, with ACP lifecycle and SDK behavior as adjacent owners.
- This does not belong to generic launcher routing unless the bug is specifically a row selection or portal entry bug.
- Screenshots are only needed when context chips, picker layout, or portal return visuals are the regression surface.

## Related Features

- [003 Agent Chat Context Composer](./003-agent-chat-context.md)
- [004 MCP Context Resources / SDK / Protocol Automation](./004-mcp-sdk-protocol.md)
- [030 ACP Chat SDK APIs](./030-acp-chat-sdk-apis.md)
- [031 Legacy `chat()` Prompt](./031-legacy-chat-prompt.md)
- [032 Script Metadata, Scriptlets, and Execution Catalog](./032-script-metadata-scriptlets.md)

## Open Questions And Gaps

- Detached ACP currently supports only narrower local portal flows; broader main-window portal parity depends on embedded launcher return contracts.
- Preview metadata is intended to be synchronous and cheap, but file/skill size display may still read filesystem metadata, so avoid claiming zero I/O without checking the current source.
- New resource types should update schema examples and MCP resource tests so `kit://context/schema` remains an honest contract.
- Skill metadata changes need discovery/search tests because title, description, plugin owner, and slash-name fallback are part of user-facing behavior.

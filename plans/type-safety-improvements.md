# Type Safety Improvements

## Scope
Scanned `src/**/*.rs` for stringly-typed APIs, weak identifiers, and repeated free-form literals that should be encoded in Rust types.

## Implemented In This Change
### 1) Slash command parsing + autocomplete now use typed command mapping
- File: `src/prompts/commands.rs`
- Change:
  - Added typed keyword parsing on `SlashCommandType` (`from_keyword`, `keywords`, `matches_keyword_prefix`, `all`).
  - Removed duplicated string fields from `CommandOption` (`label`, `description`, `icon`) and derived them from `kind`.
  - Canonicalized parsed alias raw command tokens (e.g. `/tests` now normalizes to `/test`, `/summary` to `/summarize`).
- Why this is safer:
  - Prevents drift between command metadata and autocomplete labels/icons.
  - Centralizes keyword/alias mapping to one typed source of truth.
  - Prevents downstream logic from branching on alias-specific raw strings.

## High-Impact Follow-Ups

### 2) Replace free-form stdin key modifiers with typed enum
- Files: `src/stdin_commands.rs`, `src/main.rs`, `src/ai/window.rs`
- Current risk:
  - `modifiers: Vec<String>` accepts typos/silent invalid values.
- Suggested type:
  - `enum KeyModifier { Cmd, Shift, Alt, Ctrl }` with serde aliases (`meta`, `command`, `option`, `control`).
- Benefit:
  - Compile-time safety in key-routing logic, less repeated string comparisons.

### 3) Replace action-id string dispatch with `ActionId` enum
- File: `src/app_actions.rs` (plus call sites in UI/action dialogs)
- Current risk:
  - Large `match action_id.as_str()` blocks are typo-prone and not refactor-safe.
- Suggested type:
  - `enum ActionId` + parsing at boundaries.
- Benefit:
  - Compile-time exhaustiveness for action handling and easier action auditing.

### 4) Strongly type scriptlet tool kinds across modules
- Files: `src/scriptlets.rs`, `src/extension_types.rs`, `src/scripts/types.rs`, `src/executor/scriptlet.rs`
- Current risk:
  - Tool kinds (`"bash"`, `"ts"`, `"paste"`, etc.) are duplicated in many string matches.
- Suggested type:
  - `enum ScriptletTool` (with `Unknown(String)` at parse boundary if needed).
- Benefit:
  - Shared behavior (display name, icon, execution class) becomes typed and centralized.

### 5) Protocol parse classification should avoid string inspection of serde error text
- File: `src/protocol/io.rs`
- Current risk:
  - `parse_message_graceful` uses `error_str.contains("unknown variant")` to decide unknown type vs invalid payload.
- Suggested type:
  - Introduce typed message-kind pre-parse step (`MessageKindTag`) and classify before full payload decode.
- Benefit:
  - Robust parse classification independent of serde error message wording.

## Medium Priority

### 6) Normalize keyboard key names once, then use typed key enum
- Files: many (`src/main.rs`, `src/render_builtins.rs`, `src/render_script_list.rs`, `src/prompts/*`, `src/ai/window.rs`)
- Current risk:
  - Repeated literal matches (`"up"|"arrowup"`, etc.) can drift.
- Suggested type:
  - `enum NormalizedKey` + a single normalizer.
- Benefit:
  - One source of truth for cross-platform key names.

### 7) Typed identifier newtypes for message/session IDs
- Files: protocol + execution path (`src/protocol/message.rs`, `src/protocol/io.rs`, `src/execute_script.rs`, `src/logging.rs`)
- Current risk:
  - `id`, `request_id`, `correlation_id` are all plain `String`, easy to mix up.
- Suggested types:
  - `PromptId`, `RequestId`, `CorrelationId` newtypes.
- Benefit:
  - Prevents accidental identifier cross-use at compile time.

## Migration Guidance
1. Introduce types at parse boundaries (serde/custom parsers), keep wire format unchanged.
2. Add `FromStr/TryFrom<&str>` + `Display` for transition ergonomics.
3. Keep compatibility with existing external scripts using serde aliases where needed.
4. Migrate internal call sites module-by-module, then remove old string APIs.

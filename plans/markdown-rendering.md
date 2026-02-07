# Markdown Rendering Analysis

## Scope Reviewed
- `src/prompts/markdown.rs`
- `src/render_prompts/arg.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/editor.rs`
- `src/render_prompts/form.rs`
- `src/render_prompts/other.rs`
- `src/render_prompts/path.rs`
- `src/render_prompts/term.rs`

## Executive Summary
Markdown rendering is centralized in `src/prompts/markdown.rs`; `src/render_prompts/*` mostly wraps prompt shells and key routing, with chat specifically delegated through `render_chat_prompt` in `src/render_prompts/other.rs:210`.

The renderer supports basic prose, headings, simple lists, tables, links, and fenced code blocks, but it has several fidelity gaps in production behavior: no markdown image rendering, broken nested-list handling, hard-break flattening, footnote feature drift, and code/link UX limitations.

## Findings (Ordered by Severity)

### 1) Markdown images are not rendered (`![alt](url)` is effectively dropped)
- Severity: High
- Evidence:
  - Parser handles `Tag::Link` but not `Tag::Image` (`src/prompts/markdown.rs:173`).
  - No `Tag::Image` / `TagEnd::Image` branch exists anywhere in the parser.
  - HTML blocks are treated as plain text via `Event::Html` (`src/prompts/markdown.rs:337`), so `<img>` also does not render as an image.
- Impact:
  - Assistant responses with markdown images lose core content fidelity.
  - Only user-uploaded attachment thumbnails are shown (outside markdown path), creating inconsistent behavior.
- Repro:
  - `![diagram](https://example.com/diagram.png)`
- Recommendation:
  - Add `Tag::Image` handling to create an image block element (with loading/error fallback and safe remote/local URL policy).

### 2) Nested lists are structurally incorrect due single-level list state
- Severity: High
- Evidence:
  - List parsing uses a single `list_state: Option<ListState>` (`src/prompts/markdown.rs:139`).
  - Starting a nested `Tag::List` overwrites the existing list context (`src/prompts/markdown.rs:156`).
  - There is no list stack to preserve parent list state.
- Impact:
  - Nested ordered/unordered lists are flattened or malformed.
  - AI responses with indented steps/checklists lose hierarchy.
- Repro:
  - `1. Parent\n   - Child A\n   - Child B\n2. Next`
- Recommendation:
  - Replace `Option<ListState>` with a stack (`Vec<ListState>`) and push/pop on `Start(List)`/`End(List)`.

### 3) Hard line breaks are flattened to spaces
- Severity: Medium
- Evidence:
  - `Event::SoftBreak | Event::HardBreak` both map to a single space (`src/prompts/markdown.rs:330`).
- Impact:
  - Markdown hard-break semantics (`two spaces + newline`, explicit line breaks) are lost.
  - Poetry, addresses, and formatted output collapse visually.
- Repro:
  - `line one  \nline two`
- Recommendation:
  - Preserve `HardBreak` as a rendered line break element; keep `SoftBreak` as space or soft-wrap.

### 4) Footnotes are enabled in parser options but not implemented in renderer
- Severity: Medium
- Evidence:
  - `ENABLE_FOOTNOTES` is enabled (`src/prompts/markdown.rs:130`).
  - No handling exists for footnote-related tags/events (e.g. definition/reference branches are absent).
- Impact:
  - Footnote markdown can render partially, incorrectly, or silently degrade.
  - Feature flag implies support that runtime behavior does not match.
- Recommendation:
  - Either implement footnote rendering end-to-end or remove `ENABLE_FOOTNOTES` until implemented.

### 5) Link handling opens raw URLs without scheme validation
- Severity: Medium
- Evidence:
  - Link click directly calls `open::that(&url_owned)` (`src/prompts/markdown.rs:863`) using unvalidated markdown destination (`src/prompts/markdown.rs:175`).
- Impact:
  - Untrusted model output can trigger unexpected local/app protocol handlers (`file:`, custom schemes, etc.).
- Recommendation:
  - Add URL validation/allowlist (e.g. `https`, `http`, optionally `mailto`) and block or confirm on others.

### 6) Autolink-style plain URLs are not made clickable
- Severity: Medium
- Evidence:
  - Parser options include strikethrough/tables/tasklists/footnotes/smart punctuation only (`src/prompts/markdown.rs:126-131`); no autolink option is enabled.
  - Clickability is only tied to explicit `Tag::Link` state (`src/prompts/markdown.rs:173`, `src/prompts/markdown.rs:851`).
- Impact:
  - Typical AI output containing bare URLs is rendered as inert text.
- Recommendation:
  - Enable autolinks in parser options and/or post-process bare URLs into safe link spans.

### 7) Code block UX/fidelity gaps: clipped long lines and imperfect TS highlighting
- Severity: Medium
- Evidence:
  - Code lines render in a horizontal row without wrap/scroll behavior (`src/prompts/markdown.rs:604`).
  - Chat container clips horizontal overflow (`src/prompts/chat.rs:2326`, `src/prompts/chat.rs:2341`), so long code can be truncated.
  - TypeScript is mapped to JavaScript syntax (`src/notes/code_highlight.rs:140`), reducing highlighting fidelity for TS-specific syntax.
- Impact:
  - Long lines become unreadable in chat.
  - Syntax colors can be misleading for TypeScript-heavy responses.
- Recommendation:
  - Provide horizontal scrolling (or optional wrap) for code blocks.
  - Use a real TS syntax when available or clearly label fallback mode.

### 8) Quote depth is parsed but not visually represented beyond level 1
- Severity: Low
- Evidence:
  - `quote_depth` increments/decrements (`src/prompts/markdown.rs:168`, `src/prompts/markdown.rs:260`).
  - Rendering applies identical single-level quote styling for any `quote_depth > 0` (`src/prompts/markdown.rs:814`).
- Impact:
  - Nested blockquotes lose structural depth.
- Recommendation:
  - Apply depth-aware indentation/border styling proportional to quote depth.

## Render-Prompt Layer Notes
- `src/render_prompts/*` does not parse markdown directly.
- Chat path in `src/render_prompts/other.rs:210` delegates markdown behavior to `ChatPrompt` / `render_markdown`, so parser/renderer fixes should be concentrated in `src/prompts/markdown.rs` and markdown call sites.

## Test Coverage Gaps
Current markdown tests validate mostly simple structural parsing (`src/prompts/markdown.rs:1252` onward) but do not cover several high-risk behaviors above.

Suggested additions:
- `test_markdown_image_tag_renders_image_block_when_valid_url`
- `test_nested_lists_preserve_parent_child_structure`
- `test_hard_break_preserves_line_break`
- `test_footnote_reference_and_definition_render_consistently`
- `test_markdown_link_rejects_disallowed_url_schemes`
- `test_long_code_line_remains_readable_with_scroll_or_wrap`

## Priority Fix Order
1. Implement image rendering and nested-list stack (highest user-visible fidelity gaps).
2. Fix hard-break behavior and align footnote feature flags with actual rendering.
3. Harden link opening with scheme validation and autolink support.
4. Improve code-block readability (horizontal scroll/wrap) and syntax fallback behavior.

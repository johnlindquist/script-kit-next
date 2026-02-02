# Markdown crates research for GPUI (Script Kit Notes)
Date: 2026-02-01

## Requirements
- Live preview or syntax highlighting during editing
- Compatibility with GPUI (Rust UI)
- Performance suitable for real-time editing

## Comparison (high level)
| Crate | Primary role | Strengths | Tradeoffs / risks | Fit for GPUI notes |
| --- | --- | --- | --- | --- |
| pulldown-cmark | Markdown parser (CommonMark + extensions) | Fast, low allocation, safe; pull-parser model; supports tables, task lists, strikethrough, footnotes, math, and more via extensions; optional source maps | Event-based, so you build your own AST/rendering; extensions are opt-in | Excellent for preview pipeline and export; good for background parsing | 
| comrak | CommonMark + GitHub Flavored Markdown (GFM) | GFM support, configurable extensions; pluggable syntax highlighting for code blocks | Heavier than pure event stream; rendering pipeline choices limited to its API surface | Strong option for GitHub-style preview; can integrate with GPUI renderer | 
| markdown-rs | CommonMark-compliant Markdown AST | Full CommonMark compliance; supports GFM + MDX + frontmatter + math; safe and well-tested; exposes mdast | Larger AST and richer feature set can be heavier for live preview | Great if Notes needs MDX/frontmatter/math; otherwise may be more than needed |
| tree-sitter-markdown | Tree-sitter grammar for Markdown | Designed for syntax highlighting and incremental parsing; CommonMark-based with some GFM toggles | Not recommended for correctness; intended for syntactic info, not full fidelity | Best for editor highlighting; avoid as sole preview parser |
| syntect | Syntax highlighting (Sublime Text grammars) | High-quality highlighting; supports fast incremental re-highlighting; widely used | Loads syntax sets/themes; can be heavier if used per-keystroke without caching | Great for code fences in preview and editor if cached |
| tree-sitter-highlight | Tree-sitter highlight engine | Uses tree-sitter queries to produce highlight spans; common highlight names | Requires language grammars + queries | Good alternative to syntect; aligns with tree-sitter editor pipeline |

Sources: pulldown-cmark guide + repo, comrak docs, markdown-rs docs, tree-sitter-markdown repo, syntect docs, tree-sitter highlight docs.

## Crate notes (details)

### pulldown-cmark
- Pull-parser for CommonMark with optional extensions and a source-map feature. It is explicitly designed to be fast and low-allocation, while remaining safe. The extension list includes tables, task lists, strikethrough, footnotes, math, and other common Markdown features.
- Best suited as the preview parser: parse Markdown into events, build a lightweight render tree, then map into GPUI elements or styled text.

### comrak
- Implements CommonMark + GitHub Flavored Markdown. It exposes options for extensions and supports pluggable code block highlighting.
- Good when GFM fidelity is the priority. Consider this if Script Kit Notes should match GitHub rendering closely.

### markdown-rs
- A CommonMark-compliant parser that outputs mdast and supports GFM, MDX, frontmatter, and math. It emphasizes correctness, safety, and robust testing.
- Best when Notes needs advanced authoring features (MDX, math, or structured AST access). Otherwise it may be more than necessary for a fast preview pipeline.

### tree-sitter-markdown
- Tree-sitter grammar for Markdown. It is designed to parse CommonMark with optional GFM-like extensions, but explicitly warns that it is not recommended for correctness; its main goal is syntax highlighting.
- Ideal for editor syntax highlighting and incremental parsing. Use alongside a correctness-focused parser for preview.

### syntect
- Syntax highlighting based on Sublime Text definitions, aimed at high-quality editor highlighting with fast incremental re-highlighting.
- Best used for code fences in previews or to highlight Markdown code blocks where tree-sitter highlight is not a fit.

### tree-sitter-highlight
- Tree-sitter highlight engine that produces highlight spans from highlight queries and language grammars.
- Works well with tree-sitter-markdown for editor highlighting, and can be used for code fences if you already rely on tree-sitter grammars.

## Recommended approach for Script Kit Notes

### Recommended default (balanced correctness + performance)
1) Editor highlighting:
   - Use tree-sitter-markdown for incremental parsing and syntax highlighting.
   - Use tree-sitter-highlight (or syntect) to derive spans for GPUI text styling.
2) Preview rendering:
   - Use pulldown-cmark for Markdown parsing (fast + low allocation).
   - Convert events to a lightweight render tree and then to GPUI components/styled text.
   - For fenced code blocks in the preview, use syntect or tree-sitter-highlight.

Why:
- tree-sitter-markdown is explicitly geared toward highlighting and incremental parse, while pulldown-cmark is built for correctness and speed in full parsing.
- This split matches real-time editor needs while keeping preview fidelity high.

### Feature-rich alternative
- If Notes needs MDX/frontmatter/math or a full AST for editing tools, use markdown-rs for preview parsing and metadata extraction, and still use tree-sitter-markdown for editor highlighting.

### GFM-first alternative
- If GitHub-style rendering parity is the priority, use comrak for preview parsing/rendering and tree-sitter-markdown for editor highlighting. Use comrak's syntax highlighting hook with syntect.

## Example integration patterns with GPUI (pseudo-code)

### 1) Editor highlighting pipeline (tree-sitter)
```rust
// Parse incrementally per edit, then update style spans.
let mut parser = tree_sitter::Parser::new();
parser.set_language(tree_sitter_markdown::language())?;
let tree = parser.parse(text, previous_tree.as_ref());

// Convert tree to highlight spans.
let mut highlighter = tree_sitter_highlight::Highlighter::new();
let highlights = highlighter.highlight(&highlight_config, text.as_bytes(), None, |_| None)?;

// Apply spans to GPUI text rendering (StyledText / TextRun / custom editor view).
```

### 2) Preview pipeline (pulldown-cmark + optional syntect)
```rust
let parser = pulldown_cmark::Parser::new_ext(text, options);
for event in parser {
    match event {
        // Build lightweight nodes (paragraphs, headings, lists, code blocks, etc.)
        _ => {}
    }
}
// Render nodes to GPUI elements; for code blocks, run syntect and apply styles.
```

### 3) Preview pipeline (comrak)
```rust
let mut opts = comrak::ComrakOptions::default();
opts.extension.table = true; // etc.
// Parse + render; hook in code block highlighter via comrak plugins.
```

Implementation tips:
- Parse on a background thread, then post results back to GPUI to avoid blocking the UI.
- Cache parse results and only re-render the affected parts when possible.
- Throttle parsing on rapid edits (e.g., 50-100ms debounce) to keep typing latency low.

## Existing implementations in Zed to learn from
- Zed's Markdown language docs list tree-sitter-markdown and describe code block highlighting via tree-sitter highlight/injection queries. This indicates tree-sitter-based highlighting in a GPUI app.
- Public mirrors and discussions show Zed has Markdown preview and a `MarkdownElement` in `crates/markdown`, which are the closest GPUI-native examples of Markdown rendering and selection.

Suggested files to inspect in Zed:
- `crates/markdown_preview/src/markdown_preview_view.rs`
- `crates/markdown_preview/src/markdown_renderer.rs`
- `crates/markdown_preview/src/markdown_parser.rs`
- `crates/markdown/src/markdown.rs`
- `crates/markdown/src/parser.rs`

## References
- pulldown-cmark guide: https://pulldown-cmark.github.io/pulldown-cmark/
- pulldown-cmark repo: https://github.com/pulldown-cmark/pulldown-cmark
- comrak docs: https://comrak.ee/
- comrak repo: https://github.com/kivikakk/comrak
- markdown-rs docs: https://github.com/wooorm/markdown-rs
- tree-sitter-markdown repo: https://github.com/MDeiml/tree-sitter-markdown
- tree-sitter-highlight docs: https://docs.rs/tree-sitter-highlight/latest/tree_sitter_highlight/
- syntect docs: https://docs.rs/syntect/latest/syntect/
- Zed markdown language docs (tree-sitter-markdown + code block highlighting): https://zed.dev/docs/languages/markdown
- Zed quick action preview (uses markdown_preview): https://fossies.org/linux/zed/crates/zed/src/zed/quick_action_bar/preview.rs
- Zed discussion referencing MarkdownElement in crates/markdown: https://github.com/zed-industries/zed/discussions/13765

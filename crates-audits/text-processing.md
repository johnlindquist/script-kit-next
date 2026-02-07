# Text Processing Crates Audit

## Scope
- Repository: `script-kit-gpui`
- Audit target crates: `syntect` 5.2, `pulldown-cmark` 0.12, `regex` 1.12, `nucleo-matcher` 0.3, `tree-sitter` 0.25, `tree-sitter-md` 0.5.2, `tree-sitter-typescript` 0.23.2
- Dependency declarations verified in `Cargo.toml:40`, `Cargo.toml:43`, `Cargo.toml:44`, `Cargo.toml:45`, `Cargo.toml:116`, `Cargo.toml:136`, `Cargo.toml:137`

## Executive Verdict
- `syntect` efficiency: **Partially optimized**. Good lazy globals and caches are present, but there are duplicated global syntax/theme loads across modules and cache eviction is coarse (full clear), which can cause avoidable churn under varied inputs.
- `pulldown-cmark` coverage: **Yes for runtime markdown rendering paths**. All markdown rendering entry points route through modules that parse with `pulldown-cmark`.
- `nucleo-matcher` tuning: **Good baseline, not fully specialized**. Reuse patterns are strong, but file/path search still uses `Config::DEFAULT` instead of path-tuned config.
- tree-sitter loading: **Mixed**. Markdown highlighter registration is lazy one-time; TypeScript parser setup is repeated per parse call.

## Findings By Crate

### 1) `syntect` (5.2)

#### What is good
- Syntax and theme objects are lazily initialized via `OnceLock` in `src/syntax.rs:21`, `src/syntax.rs:24`, `src/syntax.rs:27`.
- Notes code highlighting also uses lazy globals in `src/notes/code_highlight.rs:53`, `src/notes/code_highlight.rs:54`, `src/notes/code_highlight.rs:55`.
- Notes markdown highlighting has a per-input highlight cache in `src/notes/code_highlight.rs:60`, keyed by `(code, language, is_dark)`.
- Main preview/scriptlet flows have callsite caches that reduce repeat highlighting work in `src/app_impl.rs:2310` and `src/app_render.rs:697`.

#### Efficiency gaps
- `SyntaxSet::load_defaults_newlines` + theme loads are duplicated in two modules (`src/syntax.rs` and `src/notes/code_highlight.rs`), so memory/work are duplicated even though data is conceptually shared.
- `HIGHLIGHT_CACHE` in notes clears the entire cache when length exceeds 512 (`src/notes/code_highlight.rs:279`). This can create re-highlight spikes when data slightly exceeds the limit.
- Prompt markdown parse eagerly pre-highlights every fenced code block while building IR (`src/prompts/markdown.rs:359`), which front-loads work even for content not currently visible.
- Markdown IR cache in prompt renderer also uses full-clear eviction at limit (`src/prompts/markdown.rs:969`).

#### Verdict
- Current implementation is efficient enough for typical workloads but has avoidable worst-case churn for large/varied markdown and code-heavy chat streams.

### 2) `pulldown-cmark` (0.12)

#### Coverage check: “used for all markdown rendering?”
- Yes for runtime rendering entry points.
- Prompt/chat markdown parser uses `Parser::new_ext` in `src/prompts/markdown.rs:194` and is rendered via `render_markdown` at `src/prompts/markdown.rs:935`.
- Notes markdown parser uses `Parser::new_ext` in `src/notes/markdown.rs:226` and renders via `render_markdown_preview` at `src/notes/markdown.rs:415`.
- Render callsites:
- `src/prompts/chat.rs:2391`
- `src/prompts/chat.rs:2406`
- `src/ai/window.rs:5223`
- `src/ai/window.rs:5360`
- `src/notes/window.rs:3732`

#### Observation
- Markdown parsing logic is duplicated between prompt and notes modules with different internal block models. This increases drift risk (feature support/perf behavior can diverge).

### 3) `regex` (1.12)

#### Good usage
- Some hot regexes are precompiled with `LazyLock` in `src/scripts/input_detection.rs:25` and `src/scripts/input_detection.rs:28`.

#### Efficiency risks
- Several regexes are compiled per call on user-path code:
- `src/scripts/input_detection.rs:250`
- `src/scripts/input_detection.rs:256`
- `src/prompt_handler.rs:363`
- `src/form_parser.rs:30`
- `src/form_parser.rs:42`
- `src/form_parser.rs:55`
- `src/form_parser.rs:77`
- `src/form_parser.rs:89`
- `src/form_parser.rs:107`
- `src/prompts/template.rs:135`

#### Verdict
- Regex usage is mixed. There are clear hot-path wins available by lifting repeat `Regex::new(...)` into statics.

### 4) `nucleo-matcher` (0.3)

#### What is good
- Query-length gating avoids noisy/expensive fuzzy matching for single-char queries via `MIN_FUZZY_QUERY_LEN = 2` (`src/scripts/search.rs:142`) and applied in multiple search paths (`src/scripts/search.rs:765`, `src/scripts/search.rs:990`, `src/scripts/search.rs:1177`, `src/scripts/search.rs:1338`, `src/scripts/search.rs:1444`).
- `NucleoCtx` reuses matcher + buffer per query (`src/scripts/search.rs:233`), reducing allocations in hot loops.
- Unicode highlight fallback reuses pattern/matcher and dedups/sorts indices (`src/scripts/search.rs:297`).

#### Optimization opportunities
- Current matcher uses `Matcher::new(Config::DEFAULT)` in `src/scripts/search.rs:250` and `src/scripts/search.rs:313`.
- File ranking path (`src/file_search.rs:1477`) scores file names with that same default config. For path-heavy queries, `Config::DEFAULT.match_paths()` would usually rank delimiters (`/`, `:`) more naturally.

#### Verdict
- Configuration is solid for general fuzzy search. For filesystem-centric ranking, path-mode config is a likely quality/perf improvement.

### 5) `tree-sitter` (0.25), `tree-sitter-md` (0.5.2), `tree-sitter-typescript` (0.23.2)

#### Markdown parser/highlighter loading
- Markdown language registration is lazy and one-time via `OnceLock` in `src/notes/markdown_highlighting.rs:4`.
- Registration happens when Notes app initializes (`src/notes/window.rs:403`), not at global startup.

#### TypeScript parser loading
- Config editor creates a new parser per parse call (`src/config/editor.rs:157`) and calls `set_language(...)` each time (`src/config/editor.rs:159`).
- This is likely acceptable because config edits are low-frequency, but it is not as efficient as parser reuse.

#### Verdict
- Markdown tree-sitter integration is loaded efficiently (lazy).
- TypeScript tree-sitter integration is functionally correct but eager per invocation.

## Prioritized Recommendations

1. Replace full-clear caches with bounded LRU caches for markdown/code highlight paths.
- Targets: `src/prompts/markdown.rs` cache and `src/notes/code_highlight.rs` cache.
- Expected impact: smoother latency under varied chat/history workloads.

2. Consolidate syntect global resources into one shared module.
- Remove duplicated `SyntaxSet`/theme initialization between `src/syntax.rs` and `src/notes/code_highlight.rs`.
- Expected impact: lower memory footprint and fewer cold inits.

3. Use path-tuned `nucleo-matcher` config for file/path scoring.
- Prefer `Config::DEFAULT.match_paths()` in path ranking contexts.
- Expected impact: better relevance for path queries and potentially less backtracking on separators.

4. Precompile repeated regexes in hot/user-loop code.
- Move repeated `Regex::new(...)` calls to `LazyLock` statics for `form_parser`, `template`, `prompt_handler`, and code-snippet detection helpers.
- Expected impact: lower CPU spikes during frequent parsing/filtering.

5. Optionally reuse tree-sitter TypeScript parser in config editor.
- Introduce a reusable parser holder (for example thread-local parser + one-time language init).
- Expected impact: minor CPU reduction in repeated config operations.

## Direct Answers (Assignment Checklist)
- Is syntect syntax highlighting efficient? **Mostly yes, but not optimal** due to duplicated syntax/theme singletons and full-clear cache eviction in markdown highlight paths.
- Is pulldown-cmark used for all markdown rendering? **Yes for current runtime markdown rendering entry points** (chat prompt, AI window, notes preview).
- Is nucleo-matcher configured optimally for fuzzy search? **Good general configuration**, but **not fully optimized for path matching** in file-search contexts.
- Are tree-sitter parsers loaded efficiently (lazy vs eager)? **Markdown parser/highlighter is lazy one-time; TypeScript parser is eager per parse call**.

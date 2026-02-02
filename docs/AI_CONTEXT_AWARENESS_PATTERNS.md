# AI Context Awareness Patterns

Research on how AI chat tools handle context awareness - clipboard content, selected text, current file, and more. This document captures patterns used by Raycast, Cursor, Zed, GitHub Copilot, and others.

---

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Context Types](#context-types)
3. [Implementation Patterns](#implementation-patterns)
4. [Tool-Specific Implementations](#tool-specific-implementations)
5. [Model Context Protocol (MCP)](#model-context-protocol-mcp)
6. [Best Practices](#best-practices)
7. [Implementation Recommendations](#implementation-recommendations)

---

## Core Concepts

### What is Context Engineering?

Context engineering is "the discipline of designing and building dynamic systems that provide the right information and tools, in the right format, at the right time, to give an LLM everything it needs to accomplish a task."

The key insight: **context is a finite resource**. Modern LLMs have context windows of 128K-200K tokens, but effective use requires "curating what will go into the limited context window from that constantly evolving universe of possible information."

### The Context Spectrum

```
Implicit Context <-------------------> Explicit Context
(Auto-gathered)                        (User-specified)
```

| Implicit | Explicit |
|----------|----------|
| Current file | @-mentions |
| Cursor position | File attachments |
| Open tabs | Slash commands |
| Clipboard history | Highlighted selection |
| Recent edits | Manual context injection |
| Terminal output | Context files (CLAUDE.md) |

---

## Context Types

### 1. Selection Context
- **Selected text** in the current editor/application
- Text highlighted before invoking AI
- Multi-line selections with file context

### 2. Clipboard Context
- Current clipboard content
- Clipboard history (recent N items)
- Image/file content from clipboard

### 3. File Context
- Current file content
- Open tabs/buffers
- Recently viewed files
- Project files (via semantic search)

### 4. Editor Context
- Cursor position (line, column)
- Surrounding code (context window)
- Symbol at cursor
- Import/dependency graph

### 5. Project Context
- Project structure/file tree
- Configuration files
- Documentation (README, etc.)
- Git history and changes

### 6. Environment Context
- Current application
- Terminal output
- Error messages/diagnostics
- Browser tab content

### 7. Temporal Context
- Edit history
- Conversation history
- Previous AI interactions
- Recent commands

---

## Implementation Patterns

### Pattern 1: Dynamic Placeholders (Raycast)

Raycast uses placeholder syntax to inject context dynamically:

```
{selection}     - Selected text from frontmost app
{clipboard}     - Last copied text
{clipboard offset=1}  - Second-to-last clipboard entry
{browser-tab}   - Content from selected browser tab
{argument}      - User-provided input
{date}          - Current date
{cursor}        - Cursor position marker
```

**Modifiers** transform placeholder values:
```
{clipboard | uppercase}
{selection | trim | json-stringify}
{clipboard | raw}  - Bypass default formatting
```

**Key Implementation Details:**
- Clipboard history access (offset 0-5)
- Confidential data handling (excluded from history)
- Default formatting per context (URL encoding, JSON wrapping)

### Pattern 2: @-Mention System (Cursor, Zed, Copilot)

Explicit context specification via `@` prefix:

```
@file:src/main.rs    - Specific file
@folder:src/         - Directory contents
@workspace           - Entire workspace context
@terminal            - Terminal output
@selection           - Current selection
@docs                - Documentation
@codebase            - Semantic search of codebase
```

**Cursor's Context Variables:**
- `#file:'Main.rs'` - File reference in prompts
- `@workspace` - Routes to workspace agent
- `@terminal` - Routes to terminal agent

### Pattern 3: Automatic Context Augmentation (Cursor)

Cursor automatically includes in every request:
1. Full content of current file
2. List of recently viewed files
3. Semantic search results from codebase
4. Active linter/compiler errors
5. Recent edit history

**Context Compression:**
```
Full codebase (10M tokens)
    → Embedding search (find relevant files)
    → Importance ranking (score by relevance)
    → Smart truncation (keep critical sections)
    → Compressed context (~8K tokens)
```

### Pattern 4: Slash Commands (Zed, Raycast)

Structured commands for context injection:

```
/file path/to/file.rs    - Insert file content
/tab                     - Active tab content
/tabs                    - All open tabs
/selection               - Current selection
/symbols                 - Active symbols
/diagnostics             - Language server errors
/fetch URL               - Web page content
```

### Pattern 5: Context Provider Trait (Zed)

Zed implements a `ContextProvider` trait for extensible context:

```rust
trait ContextProvider {
    fn build_context(&self) -> TaskVariables;
}

struct BasicContextProvider;

impl ContextProvider for BasicContextProvider {
    fn build_context(&self) -> TaskVariables {
        TaskVariables {
            row: current_row(),
            column: current_column(),
            selected_text: get_selection(),
            symbol: symbol_at_cursor(),
            worktree_root: get_worktree_root(),
            relative_file: get_relative_path(),
            filename: get_filename(),
        }
    }
}
```

### Pattern 6: Layered Memory Architecture

```
┌─────────────────────────────────────┐
│ Working Memory (Current Context)    │ ← Active conversation
├─────────────────────────────────────┤
│ Short-term Memory (Session)         │ ← Recent history
├─────────────────────────────────────┤
│ Long-term Memory (Persistent)       │ ← Rules, memories, patterns
└─────────────────────────────────────┘
```

**Cursor's Implementation:**
- **Rules**: Markdown files in `.cursor/rules/` injected into every context
- **Memories**: AI-maintained notes about project patterns
- **Compaction**: Summarize conversation before context limit

### Pattern 7: Just-In-Time Context Retrieval

Instead of pre-loading all context:

```
1. Maintain lightweight identifiers (file paths, queries, URLs)
2. Store references, not content
3. Dynamically load data at runtime using tools
4. Progressive discovery of relevant information
```

Benefits:
- Prevents context pollution
- Enables larger virtual context
- Matches human cognition patterns

---

## Tool-Specific Implementations

### Raycast AI

**Clipboard API:**
```typescript
// Read clipboard with history offset
Clipboard.read({ offset: 0 });  // Current
Clipboard.read({ offset: 1 });  // Previous

// Paste at cursor position
Clipboard.paste(content);

// Confidential handling
Clipboard.copy(secret, { concealed: true });
```

**Dynamic Placeholders in AI Commands:**
- Wrapped with `"""` delimiters for AI parsing
- Support for multiple modifiers in chain
- Browser extension integration for tab content

### Cursor

**Implicit Context Gathering:**
- Current file (always included)
- Cursor position and surrounding code
- Open file relationships
- Edit patterns and recent changes

**Calculated Context:**
- `@workspace` triggers semantic file selection
- AI analyzes prompt to find relevant files
- Results ranked and truncated to fit context

**Persistent Context:**
- `copilot-context.md` or `.copilot/context.md`
- `github-copilot-instructions.md` in repo root
- System prompt for repository-wide rules

### Zed

**Context Gathering for Edit Prediction:**
```rust
struct EditPredictionModelInput {
    project: Project,
    buffer: Buffer,
    snapshot: BufferSnapshot,
    position: Point,
    events: Vec<EditEvent>,
}

fn gather_context(
    full_path: &Path,
    buffer: &BufferSnapshot,
    cursor_point: Point,
    events: &[EditEvent],
) -> Context {
    // Collect cursor excerpt, file content, edit history
}
```

**Agent Panel Context:**
- @-mentions for files, directories, symbols
- Automatic formatting of pasted code with file context
- Image paste support from clipboard
- `/selection` command for current selection

### GitHub Copilot

**Three Context Types:**
1. **Implicit**: Auto-added (current file, selection)
2. **Explicit**: User-specified (#file, attachments)
3. **Calculated**: AI-determined (@workspace analysis)

**Context Variables:**
```
#selection  - Selected text
#editor     - Current editor content
#file       - Specific file reference
```

**Context Injection Techniques:**
- Comment blocks above cursor
- Sidecar files kept open
- Instructions files in repo

---

## Model Context Protocol (MCP)

### Architecture

```
┌─────────────────────────────────────────┐
│              MCP Host                   │
│         (AI Application)                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │ Client 1 │ │ Client 2 │ │ Client 3 │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ │
└───────┼────────────┼────────────┼───────┘
        │            │            │
   ┌────▼────┐  ┌────▼────┐  ┌────▼────┐
   │ Server  │  │ Server  │  │ Server  │
   │ (Local) │  │ (Local) │  │(Remote) │
   │Filesystem│ │Database │  │ Sentry  │
   └─────────┘  └─────────┘  └─────────┘
```

### Core Primitives

**Resources** - Data sources providing contextual information:
- File contents
- Database records
- API responses

**Tools** - Executable functions:
- File operations
- API calls
- Database queries

**Prompts** - Reusable templates:
- System prompts
- Few-shot examples
- Structured interaction patterns

### JSON-RPC Protocol

```json
// Tool Discovery
{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}

// Tool Execution
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "read_file",
    "arguments": {"path": "/src/main.rs"}
  }
}

// Notifications (real-time updates)
{"jsonrpc": "2.0", "method": "notifications/tools/list_changed"}
```

### Transports

- **STDIO**: Local process communication (low latency)
- **Streamable HTTP**: Remote server communication (SSE support)

---

## Best Practices

### 1. Context Curating, Not Dumping

> "Find the smallest set of high-signal tokens that maximize the likelihood of your desired outcome."

Avoid the "context dumping" trap:
- Don't place large payloads directly in chat history
- Each token in context is a permanent tax on the session
- Prefer references over content when possible

### 2. Hierarchical Context Management

```
Mandatory Context
├── System prompt
├── Project rules (CLAUDE.md)
└── Current task definition

Optional Context (prioritized)
├── Current file
├── Selection/clipboard
├── Related files (semantic)
├── Edit history
└── Error messages

On-Demand Context (tool-fetched)
├── Full file contents
├── Documentation
├── Web resources
└── Database records
```

### 3. Context Relevance Scoring

When selecting what to include:
1. **Recency**: Recent edits > old files
2. **Proximity**: Near cursor > far in file
3. **Semantic similarity**: Related to query
4. **Explicit mention**: User-specified > inferred

### 4. Compaction Strategies

When approaching context limits:
1. Summarize conversation history
2. Preserve architectural decisions
3. Discard redundant outputs
4. Keep tool results, drop verbose explanations

### 5. Tool Design for Context Efficiency

Good tools are:
- Self-contained with clear purpose
- Minimal overlap with other tools
- Return focused, structured results
- Include only necessary context

---

## Implementation Recommendations

### For Script Kit AI Chat

#### Phase 1: Core Context Sources

```rust
struct ContextManager {
    clipboard: ClipboardProvider,
    selection: SelectionProvider,
    file: FileContextProvider,
    environment: EnvironmentProvider,
}

trait ContextProvider {
    fn gather(&self) -> Option<ContextItem>;
    fn priority(&self) -> u8;
}
```

**Essential providers:**
1. **ClipboardProvider**: Current + history (5 items)
2. **SelectionProvider**: Text selected before invocation
3. **CurrentFileProvider**: Active script/file content
4. **TerminalProvider**: Recent terminal output

#### Phase 2: Placeholder System

Implement Raycast-style placeholders:

```
{clipboard}           - Clipboard content
{selection}           - Selected text
{file:path}           - File content
{script}              - Current script
{arg:name}            - Named argument
```

With modifiers:
```
{clipboard | trim}
{selection | json}
{file:path | lines:1-50}
```

#### Phase 3: @-Mention System

```
@file path/to/file    - Include file
@clipboard            - Include clipboard
@terminal             - Include terminal output
@script               - Current script context
@kit                  - Script Kit documentation
```

#### Phase 4: Automatic Context Detection

Gather automatically based on invocation:
- Script context when editing scripts
- Error context when debugging
- Selection when text is highlighted
- Clipboard when recently copied

#### Phase 5: MCP Integration

Expose Script Kit capabilities as MCP server:
- Scripts as tools
- Snippets as prompts
- File system as resources

### API Design Sketch

```rust
// Context item representation
pub struct ContextItem {
    pub source: ContextSource,
    pub content: String,
    pub metadata: ContextMetadata,
    pub priority: u8,
}

pub enum ContextSource {
    Clipboard { offset: u8 },
    Selection,
    File { path: PathBuf },
    Terminal { lines: usize },
    Environment { key: String },
    Custom { provider: String },
}

// Context manager
pub struct AIChatContext {
    items: Vec<ContextItem>,
    max_tokens: usize,
}

impl AIChatContext {
    pub fn add(&mut self, item: ContextItem);
    pub fn from_placeholder(&mut self, placeholder: &str);
    pub fn from_mention(&mut self, mention: &str);
    pub fn compile(&self) -> String; // For LLM consumption
}
```

---

## Sources

### Primary Sources
- [Raycast Dynamic Placeholders](https://manual.raycast.com/dynamic-placeholders)
- [Raycast Clipboard API](https://developers.raycast.com/api-reference/clipboard)
- [Raycast AI Features](https://www.raycast.com/core-features/ai)
- [Cursor Context Documentation](https://docs.cursor.com/en/guides/working-with-context)
- [Cursor AI Architecture](https://collabnix.com/cursor-ai-deep-dive-technical-architecture-advanced-features-best-practices-2025/)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP Architecture Overview](https://modelcontextprotocol.io/docs/learn/architecture)
- [Anthropic: Effective Context Engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)

### Secondary Sources
- [GitHub Copilot Context Management](https://learn.microsoft.com/en-us/visualstudio/ide/copilot-context-overview)
- [VS Code Copilot Chat Context](https://code.visualstudio.com/docs/copilot/chat/copilot-chat-context)
- [Zed AI Assistant](https://zed.dev/blog/building-a-text-editor-in-times-of-ai)
- [Windsurf Context Management](https://www.zenml.io/llmops-database/context-aware-ai-code-generation-and-assistant-at-scale)
- [Context Engineering Guide](https://www.datacamp.com/blog/context-engineering)
- [Manus Context Engineering](https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus)

---

## Summary

The key patterns for context-aware AI chat:

1. **Dual approach**: Combine implicit (auto-gathered) and explicit (@-mentions) context
2. **Placeholder system**: Template variables for dynamic context injection
3. **Hierarchical prioritization**: Not all context is equal; rank by relevance
4. **Just-in-time loading**: Fetch context when needed, not upfront
5. **Compression strategies**: Summarize and compact to fit context windows
6. **Provider architecture**: Extensible system for adding new context sources
7. **MCP integration**: Standard protocol for tool/resource exposure

The goal: **Provide the right context at the right time in the right format.**

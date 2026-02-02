# Code Context Injection Patterns for AI Chat in Developer Tools

Date: 2026-02-01
Scope: @mentions, file references, code blocks, symbol references

## 1. @mentions (context participants and attachers)

Pattern
- Users type @ to attach a specific context source or participant to the prompt.
- The UI commonly shows an autocomplete list of possible targets (workspace, files, repos, issues, terminal output).

Examples from existing tools
- Copilot Chat supports @ to add issues, pull requests, and repositories directly to a prompt in supported surfaces. citeturn4view0
- VS Code workspace chat accepts @workspace for codebase-aware questions (often paired with #codebase to indicate scope). citeturn6view0
- Gemini Code Assist uses @ to specify files and folders, and @terminal to add terminal output as context. citeturn5view1

Design implications (inferred)
- Provide a single @ entry point for multiple context types, then narrow via autocomplete groups (files, folders, repos, issues, terminals).
- Show a concise preview chip for each attachment so users can remove or reorder context easily.
- Treat @mentions as explicit, user-directed context (higher priority than implicit retrieval).

## 2. File references (explicit file/selection attachments)

Pattern
- Users reference files or the current editor/selection to inject exact code into the prompt.
- Syntax is often a lightweight token prefix (for example, #file, #editor, #selection).

Examples from existing tools
- Copilot Chat supports #file and #editor for adding multiple files or the active editor file to a prompt, and it already supports #selection to include a selection. citeturn3view0
- Gemini Code Assist uses @ to specify files and folders as explicit context. citeturn5view1

Design implications (inferred)
- Prefer file references for deterministic context injection (exact file text), especially for edits or refactors.
- Support both file-level and selection-level injections to minimize context size when possible.

## 3. Code blocks (inline prompt payload)

Pattern
- Users paste or insert code blocks directly in the chat prompt to attach specific snippets.
- Code blocks act as explicit context in the prompt body, often without additional UI metadata.

Examples from existing tools
- Pieces for Developers can insert code snippets into Copilot Chat as code blocks, explicitly adding them to the AI prompt context. citeturn1view0

Design implications (inferred)
- Preserve code block formatting and language tags to help the model parse correctly.
- Consider a lightweight UX affordance to convert selected editor text into a code block with source metadata.

## 4. Symbol references (function/class/definition context)

Pattern
- Context retrieval can operate at the symbol level, injecting definitions rather than whole files.
- Symbol-aware context is typically powered by workspace indexing and language intelligence.

Examples from existing tools
- VS Code workspace context uses workspace search that includes code symbols and definitions, and can enrich context using IntelliSense details like function signatures and parameters. citeturn6view1

Design implications (inferred)
- Favor symbol-level injections for precise reasoning (function signatures, class definitions, type aliases) to reduce token usage.
- When a symbol is ambiguous, present a disambiguation list with file path + signature preview.

## 5. Combined patterns and prioritization (inferred)

- Order of context typically matters: explicit attachments (mentions, file references, code blocks) should rank above implicit retrieval (workspace search).
- Provide a visible context summary (chips or a “context drawer”) showing exactly what will be sent.
- Limit large file injections by default; offer “expand” or “include full file” toggles.

## 6. Implementation checklist (inferred)

- Autocomplete: @ and # triggers, grouped suggestions, recent items, fuzzy match.
- Attachment model: store type (file, selection, repo, issue, terminal, code block, symbol), source path, byte/line range, and display label.
- Payload builder: stable ordering, deduping, size limits, and safe truncation with explicit markers.
- Provenance: annotate each injected chunk with a short header (filename, line range, symbol name) for transparency.


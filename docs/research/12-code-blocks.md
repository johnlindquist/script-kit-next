# AI Chat Code Blocks - Research and Recommendations

## Scope
This doc summarizes patterns for code block rendering, actions (copy/run/apply),
syntax highlighting, diff views, and suggestions for the Script Kit AI chat
window.

## Research highlights (primary sources)
- CommonMark defines fenced code blocks with optional info strings; the info
  string is commonly used to indicate language and can be rendered as a class
  attribute (for example, language-xxx). Code block content is treated as
  literal text. (https://spec.commonmark.org/)
- GitHub Copilot Chat responses can include buttons to copy, insert, or preview
  code block output. (https://docs.github.com/en/copilot/using-github-copilot/using-github-copilot-in-your-ide?tool=visualstudio)
- GitHub suggested changes let reviewers apply suggestions and commit them,
  including batching multiple suggestions. (https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/incorporating-feedback-in-your-pull-request)
- GitLab "Suggest changes" allows writing a suggestion and applying it to a
  merge request. (https://docs.gitlab.com/user/project/merge_requests/reviews/suggestions/)
- VS Code inline chat shows a diff for suggestions and provides accept/reject
  actions. (https://code.visualstudio.com/docs/editor/inline-chat)
- Microsoft SSMS Copilot Chat exposes Apply/Copy/Add actions; Apply opens a
  diff view with Apply and Discard. (https://learn.microsoft.com/en-us/sql/ssms/ai-assistance/copilot?view=sql-server-ver16)
- OpenAI code execution (Code Interpreter) runs code in a sandboxed container.
  ChatGPT data analysis runs in a secure environment with outbound network
  restrictions. (https://platform.openai.com/docs/guides/tools-code-interpreter,
  https://help.openai.com/en/articles/8437071-advanced-data-analysis-chatgpt-enterprise-version)
- Web clipboard APIs (writeText) are async and require secure context and
  permissions, which affects copy button UX in web surfaces. (https://developer.mozilla.org/en-US/docs/Web/API/Clipboard/writeText)

## Code block rendering
- Use fenced code blocks as the canonical format. Preserve literal text and
  avoid parsing inside the block.
- Show a language badge when an info string exists; show "Unknown" when absent.
- Provide a wrap toggle and a max-height with expand/collapse for long blocks.
- Optional line numbers can help with referencing in chat and diffs.
- Keep inline code for short identifiers and commands.

## Copy and run actions
- Provide a per-block action row with at least: Copy, Insert, and Apply.
- Offer Copy with language (prepend info string or shebang) for easy pasting.
- For "Run":
  - Default to a sandboxed runner when possible, modeled after code interpreter
    patterns (clear environment note and limitations).
  - Always show the exact command and capture stdout/stderr in a collapsible
    output panel.
  - Require explicit user action (no auto-run), and gate by language.
- If running in a web surface, plan for clipboard failures and show toast
  feedback or a fallback "select all" action.

## Syntax highlighting
- Prefer explicit language from the info string to avoid mis-highlighting.
- If auto-detect is used, show confidence and allow user override.
- Fall back to plaintext for unknown languages and preserve whitespace.

## Diff views
- If a block is a patch (diff fence or "apply patch" intent), render a diff
  view instead of a plain code block.
- Provide Apply, Discard, and Copy Patch actions.
- Support multi-file diffs with a file list and per-file apply controls.
- Keep diffs readable: minimal colors, clear +/- lines, and optional context.

## Suggestions for the Script Kit AI chat window
- Action row: Copy, Insert at Cursor, Apply Diff, Open in Editor, Run, Save
  Snippet.
- Add a language badge, wrap toggle, and "Show more" for long blocks.
- Add a diff renderer that reuses the Apply/Discard pattern from review tools.
- For Run: show environment notes, timeouts, and output in a collapsible panel.
- Provide keyboard shortcuts and focusable controls for accessibility.
- Log action usage with correlation_id for observability.


pub(crate) const AGENT_CHAT_KITCHEN_SINK_FIXTURE_ID: &str = "agent-chat-kitchen-sink";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatKitchenSinkFixtureRole {
    User,
    Assistant,
    Thought,
    Tool,
    System,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentChatKitchenSinkFixtureMessage {
    pub(crate) id: u64,
    pub(crate) role: AgentChatKitchenSinkFixtureRole,
    pub(crate) body: &'static str,
    pub(crate) tool_call_id: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentChatKitchenSinkFixture {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) messages: &'static [AgentChatKitchenSinkFixtureMessage],
}

pub(crate) fn agent_chat_kitchen_sink_fixture() -> AgentChatKitchenSinkFixture {
    AgentChatKitchenSinkFixture {
        id: AGENT_CHAT_KITCHEN_SINK_FIXTURE_ID,
        title: "Agent Chat Kitchen Sink",
        messages: AGENT_CHAT_KITCHEN_SINK_MESSAGES,
    }
}

pub(crate) fn kitchen_sink_feature_manifest() -> &'static [&'static str] {
    &[
        "role:user",
        "role:assistant",
        "role:thought",
        "role:tool",
        "role:system",
        "role:error",
        "markdown:heading",
        "markdown:paragraph",
        "markdown:unordered-list",
        "markdown:ordered-list",
        "markdown:nested-list",
        "markdown:table",
        "markdown:fenced-code",
        "markdown:inline-code",
        "markdown:blockquote",
        "markdown:link",
        "markdown:task-list",
        "conversation:long-transcript",
        "conversation:result-artifacts",
        "conversation:next-actions",
        "conversation:tool-call-id",
        "conversation:collapsible-thought",
        "conversation:collapsible-tool",
        "conversation:tool-card-diff",
        "conversation:tool-card-failed",
    ]
}

/// Structured tool metadata for fixture Tool rows, keyed by `tool_call_id`.
///
/// Kept as a parallel table (not fields on every fixture message) so only
/// Tool rows carry tool data. `load_kitchen_sink_fixture` routes rows with a
/// meta entry through the real tool-call event path, so the kitchen sink
/// exercises the production card pipeline (kind, status, subject, diff).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentChatKitchenSinkToolMeta {
    pub(crate) tool_call_id: &'static str,
    pub(crate) tool_name: &'static str,
    pub(crate) args_json: &'static str,
    pub(crate) diff: Option<&'static str>,
    pub(crate) is_error: bool,
}

pub(crate) const AGENT_CHAT_KITCHEN_SINK_TOOL_META: &[AgentChatKitchenSinkToolMeta] = &[
    AgentChatKitchenSinkToolMeta {
        tool_call_id: "tool-read-transcript-owner",
        tool_name: "read",
        args_json: r#"{"path": "src/ai/agent_chat/ui/components/transcript.rs"}"#,
        diff: None,
        is_error: false,
    },
    AgentChatKitchenSinkToolMeta {
        tool_call_id: "tool-search-docs",
        tool_name: "grep",
        args_json: r#"{"pattern": "agent chat markdown renderer"}"#,
        diff: None,
        is_error: false,
    },
    AgentChatKitchenSinkToolMeta {
        tool_call_id: "tool-edit-diff",
        tool_name: "edit",
        args_json: r#"{"path": "src/ai/agent_chat/ui/components/transcript.rs"}"#,
        diff: Some(
            "  41 fn render_message(\n- 42     let label = \"Tool\";\n+ 42     let label = meta.tool_name.clone();\n+ 43     let subject = meta.subject.clone();\n  44     // header renders status badge",
        ),
        is_error: false,
    },
    AgentChatKitchenSinkToolMeta {
        tool_call_id: "tool-bash-failed",
        tool_name: "bash",
        args_json: r#"{"cmd": "cargo test --workspace --quiet"}"#,
        diff: None,
        is_error: true,
    },
];

pub(crate) fn kitchen_sink_tool_meta(
    tool_call_id: &str,
) -> Option<&'static AgentChatKitchenSinkToolMeta> {
    AGENT_CHAT_KITCHEN_SINK_TOOL_META
        .iter()
        .find(|meta| meta.tool_call_id == tool_call_id)
}

pub(crate) const AGENT_CHAT_KITCHEN_SINK_MESSAGES: &[AgentChatKitchenSinkFixtureMessage] = &[
    AgentChatKitchenSinkFixtureMessage {
        id: 1,
        role: AgentChatKitchenSinkFixtureRole::System,
        tool_call_id: None,
        body: r#"# Agent Chat Kitchen Sink

This deterministic fixture exercises Agent Chat transcript rendering without a live provider.

| Surface | Purpose |
| --- | --- |
| `AgentChatTranscript` | Markdown, roles, scroll, and virtual list behavior |
| `DevStyleTool` | Live style override target |

System note: keep links safe and keep every artifact local or `https://`."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 2,
        role: AgentChatKitchenSinkFixtureRole::User,
        tool_call_id: None,
        body: r#"Please review `src/ai/agent_chat/ui/components/transcript.rs` and build a kitchen sink.

- Include markdown tables and blockquotes.
- Include a tool result.
- Include follow-up actions.
- Keep this as a mocked conversation.

[Project docs](https://example.com/script-kit-gpui) should be linked safely.

- [x] Use Agent Chat roles
- [ ] Verify styling controls later"#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 3,
        role: AgentChatKitchenSinkFixtureRole::Thought,
        tool_call_id: None,
        body: r#"Thinking

The fixture should be long enough to push the virtual list beyond one viewport. I will include every Agent Chat role, multiple markdown syntaxes, and repeated turns so spacing controls have visible effect.

Plan:
1. Establish the visible transcript purpose.
2. Show markdown primitives.
3. Show collapsible thought and tool rows.
4. End with result-card style syntax and next actions."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 4,
        role: AgentChatKitchenSinkFixtureRole::Tool,
        tool_call_id: Some("tool-read-transcript-owner"),
        body: r#"Read file
completed

```json
{
  "path": "src/ai/agent_chat/ui/components/transcript.rs",
  "owner": "AgentChatTranscript",
  "markdown_renderer": "TextViewState::markdown",
  "virtual_list": true
}
```"#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 5,
        role: AgentChatKitchenSinkFixtureRole::Assistant,
        tool_call_id: None,
        body: r#"# Transcript Coverage Matrix

The kitchen sink uses the real transcript renderer, so the message body below should render through `TextViewState::markdown`.

## Markdown primitives

> A blockquote should be visually distinct and preserve wrapping inside the message area.

Unordered list:
- user bubbles
- assistant prose
- tool rows
  - nested status line
  - fenced output

Ordered list:
1. Inspect source.
2. Add a deterministic fixture.
3. Prove it with DevTools.

| Feature | Sentinel |
| --- | --- |
| Inline code | `agentChat.transcript.rowGapY` |
| Link | [Script Kit](https://scriptkit.com) |
| Task list | `- [x] done` |

```rust
fn render_fixture() {
    println!("Agent Chat Kitchen Sink");
}
```

- [x] Cover headings
- [x] Cover tables
- [x] Cover fenced code
- [ ] Tune with the dev style tool"#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 6,
        role: AgentChatKitchenSinkFixtureRole::User,
        tool_call_id: None,
        body: r#"Make the long text case obvious. The transcript should keep wrapping cleanly even when a paragraph is intentionally verbose and repetitive enough to stress line height, paragraph gap, message padding, and the scrollbar. This paragraph is deliberately long so the Agent Chat style tab can later make spacing changes visible without relying on a real provider response."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 7,
        role: AgentChatKitchenSinkFixtureRole::Assistant,
        tool_call_id: None,
        body: r#"## Long paragraph response

This is a long mocked assistant response. It repeats enough semantic structure to make spacing differences visible. The fixture should remain deterministic, safe, and provider-free. A style tweak to message padding, paragraph gap, code block padding, or user bubble radius should be immediately visible when this transcript is open.

Additional notes:
- The content is intentionally varied.
- The renderer should not special-case this fixture.
- The fixture should still behave like a normal Agent Chat conversation."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 8,
        role: AgentChatKitchenSinkFixtureRole::Tool,
        tool_call_id: Some("tool-search-docs"),
        body: r#"Search docs
running

```text
query: agent chat markdown renderer
result: TextViewState::markdown owns transcript body rendering
warning: use source contracts and runtime receipts
```"#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 9,
        role: AgentChatKitchenSinkFixtureRole::Thought,
        tool_call_id: None,
        body: r#"Thinking

The second thought row exists so collapse/expand spacing can be seen more than once. If the user tunes the collapsible body max height or border alpha, this row and the previous thought row should both change."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 10,
        role: AgentChatKitchenSinkFixtureRole::Assistant,
        tool_call_id: None,
        body: r#"### Edge cases

Inline punctuation with `code`, a [safe link](https://example.com/agent-chat), and a compact table:

| Key | Value |
| --- | --- |
| `role` | assistant |
| `fixture` | kitchen sink |

Nested list:
- Outer item
  - Inner item A
  - Inner item B
    - Inner item C"#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 11,
        role: AgentChatKitchenSinkFixtureRole::Error,
        tool_call_id: None,
        body: r#"The mocked provider returned a deterministic error for styling coverage.

```text
error: kitchen-sink-fixture
reason: this row exercises the Error role styling
retry: safe
```

Use the Retry affordance only in real conversations; this fixture is static."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 12,
        role: AgentChatKitchenSinkFixtureRole::User,
        tool_call_id: None,
        body: r#"Add a final response with result-card style syntax and follow-ups."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 13,
        role: AgentChatKitchenSinkFixtureRole::Assistant,
        tool_call_id: None,
        body: r#"## Result artifacts

Created [Kitchen Sink Report](https://example.com/kitchen-sink-report) and [Style Controls Notes](https://example.com/style-controls).

NEXT_ACTIONS:
- Tune the Agent Chat transcript spacing
- Increase code block padding
- Verify the fixture remains provider-free"#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 14,
        role: AgentChatKitchenSinkFixtureRole::User,
        tool_call_id: None,
        body: r#"Continue with enough rows to make scrolling and virtual-list behavior visible."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 15,
        role: AgentChatKitchenSinkFixtureRole::Assistant,
        tool_call_id: None,
        body: r#"Scroll sentinel 1: this assistant row exists to extend the transcript.

- It should remain selectable.
- It should keep normal markdown spacing.
- It should not reset the list state."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 16,
        role: AgentChatKitchenSinkFixtureRole::User,
        tool_call_id: None,
        body: r#"Scroll sentinel 2 from the user side."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 17,
        role: AgentChatKitchenSinkFixtureRole::Assistant,
        tool_call_id: None,
        body: r#"Scroll sentinel 3:

```sh
./scripts/agentic/agent-cargo.sh test --test agent_chat_kitchen_sink_fixture_contract
```

The command appears as markdown only; it is not executed by the fixture."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 18,
        role: AgentChatKitchenSinkFixtureRole::System,
        tool_call_id: None,
        body: r#"System checkpoint: fixture transcript remains static, deterministic, and safe."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 20,
        role: AgentChatKitchenSinkFixtureRole::Tool,
        tool_call_id: Some("tool-edit-diff"),
        body: r#"edit
complete

Successfully replaced text in src/ai/agent_chat/ui/components/transcript.rs."#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 21,
        role: AgentChatKitchenSinkFixtureRole::Tool,
        tool_call_id: Some("tool-bash-failed"),
        body: r#"bash
failed

error: test failed, to rerun pass `--lib`"#,
    },
    AgentChatKitchenSinkFixtureMessage {
        id: 19,
        role: AgentChatKitchenSinkFixtureRole::Assistant,
        tool_call_id: None,
        body: r#"Final visible sentinel: **Agent Chat Kitchen Sink complete**.

The style dev tool should later expose Agent Chat controls while this transcript remains open."#,
    },
];

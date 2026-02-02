# Next 3 Chat Prompt Improvements

> Generated: 2026-02-01 | Based on research from 15 parallel agents

---

## Summary

After analyzing Raycast, Warp, VS Code Copilot, Cursor, ChatGPT, Claude Desktop, Alfred, and 8 UX pattern studies, these are the **3 highest-impact features NOT yet implemented**:

| Priority | Feature | Impact | Effort |
|----------|---------|--------|--------|
| 1 | Slash Commands (`/`) | Critical | Medium |
| 2 | Stop Streaming (`Cmd+.`) | Critical | Low |
| 3 | Conversation Starters | High | Low |

---

## 1. Slash Commands (`/`) with Autocomplete

**Found universally in:** Raycast, VS Code Copilot, Cursor, Claude Code, Warp, Notion AI, Perplexity

### User Experience
```
User types: /
Autocomplete popup appears: [/explain] [/fix] [/summarize] [/tests] [/draft]
User selects command → prompt template inserted
```

### Commands to Implement

| Command | Action | Prompt Template |
|---------|--------|-----------------|
| `/explain` | Explain code/concept | "Explain the following in detail:" |
| `/fix` | Fix errors | "Fix the errors in this code:" |
| `/summarize` | Summarize content | "Summarize with key points:" |
| `/tests` | Generate tests | "Write unit tests for:" |
| `/draft` | Draft content | "Draft content for:" |
| `/doc` | Generate docs | "Generate documentation for:" |
| `/translate` | Translate text | "Translate to {language}:" |
| `/clear` | New chat | Clear conversation, start fresh |

### Keyboard Interaction (ARIA combobox pattern)
- **Type `/`** → Show autocomplete popup
- **Arrow Up/Down** → Navigate options
- **Enter** → Select and insert
- **Escape** → Dismiss popup
- **Continue typing** → Filter options

### Implementation Location
`src/prompts/chat.rs` in input handling:

```rust
// In key_down or input change handler
if input.starts_with('/') {
    let query = &input[1..];
    let filtered = SLASH_COMMANDS.iter()
        .filter(|cmd| cmd.name.contains(query))
        .collect();
    show_command_popup(filtered, cx);
}
```

### Why This Matters
- **Discoverability**: Users learn available actions by typing `/`
- **Speed**: Power users can quickly access common actions
- **Industry standard**: Every major AI tool uses this pattern

---

## 2. Stop Streaming with `Cmd+.`

**Found in:** ChatGPT, Claude, Raycast, all major AI apps

### User Experience
- User is waiting for long response
- Presses `Cmd+.` or clicks "Stop generating"
- Streaming stops immediately
- Partial response is preserved
- Can click "Continue" to resume or send new message

### Implementation

Add to `src/prompts/chat.rs` key handler:

```rust
fn key_down(&mut self, event: &KeyDownEvent, cx: &mut ViewContext<Self>) {
    let key = event.keystroke.key.as_str();
    let modifiers = &event.keystroke.modifiers;

    match key {
        // Existing handlers...

        // Add stop streaming
        "." if modifiers.platform => {
            if self.is_streaming() {
                self.stop_streaming(cx);
                cx.notify();
            }
        }

        // Escape also stops
        "escape" | "Escape" if self.is_streaming() => {
            self.stop_streaming(cx);
            cx.notify();
        }

        _ => {}
    }
}

fn stop_streaming(&mut self, cx: &mut ViewContext<Self>) {
    // Set streaming flag to false
    self.streaming_message_id = None;
    self.builtin_is_streaming = false;

    // Keep partial content (don't clear)
    // The current streamed content remains in messages

    // Optionally notify provider to cancel
    if let Some(cancel_token) = self.cancel_token.take() {
        cancel_token.cancel();
    }
}
```

### Visual Feedback
- Show "Stop generating" button during streaming (already may exist)
- Button should be prominent and easily clickable
- After stop, show "Continue" option

### Why This Matters
- **User control**: Essential for long/unwanted generations
- **Resource saving**: Stop wasting API tokens
- **Very low effort**: Just a key handler + flag

---

## 3. Conversation Starters (Empty State)

**Found in:** ChatGPT, Claude, Gemini, Raycast, Perplexity

### User Experience
When chat is empty (no messages), show 3-5 clickable suggestion chips:

```
┌─────────────────────────────────────────────────┐
│                                                 │
│     What can I help you with?                   │
│                                                 │
│   [Explain this code]  [Summarize clipboard]    │
│   [Debug an error]     [Write tests]            │
│                                                 │
│   ─────────────────────────────────────────     │
│   Type a message or click a suggestion...       │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Suggestions to Show

| Suggestion | When to Show | Inserted Prompt |
|------------|--------------|-----------------|
| "Explain this code" | Always | "Explain this code: " |
| "Summarize clipboard" | If clipboard has text | "Summarize: {clipboard}" |
| "Debug an error" | Always | "Help me debug this error: " |
| "Write tests" | Always | "Write tests for: " |
| "Ask something else..." | Always | Focus input (escape hatch) |

### Design Rules (from research)
1. **3-5 suggestions max** - More causes decision paralysis
2. **Verb-first labels** - "Explain", "Draft", "Fix" (not nouns)
3. **Context-aware** - Check clipboard, show relevant options
4. **Dismiss on type** - Hide when user starts typing
5. **Escape hatch** - Always include "Ask something else"

### Implementation

Add to `src/prompts/chat.rs` render method:

```rust
fn render_empty_state(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    let suggestions = self.get_context_aware_suggestions(cx);

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_4()
        .child(
            div().text_lg().child("What can I help you with?")
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .children(suggestions.into_iter().map(|s| {
                    Button::new(s.id)
                        .label(s.label)
                        .on_click(cx.listener(move |this, _, cx| {
                            this.insert_suggestion(&s, cx);
                        }))
                }))
        )
}

fn get_context_aware_suggestions(&self, cx: &ViewContext<Self>) -> Vec<Suggestion> {
    let mut suggestions = vec![
        Suggestion::new("explain", "Explain this code"),
        Suggestion::new("debug", "Debug an error"),
        Suggestion::new("tests", "Write tests"),
    ];

    // Add clipboard-aware suggestion
    if let Some(clipboard) = cx.read_from_clipboard() {
        if !clipboard.is_empty() && clipboard.len() < 10000 {
            suggestions.insert(1,
                Suggestion::new("clipboard", "Summarize clipboard")
            );
        }
    }

    suggestions.push(Suggestion::new("other", "Ask something else..."));
    suggestions
}
```

### Why This Matters
- **Onboarding**: New users learn what the chat can do
- **Reduces friction**: One click vs. thinking of prompt
- **Low effort**: Just UI in empty state

---

## Implementation Order

| Order | Feature | Time Estimate | Dependencies |
|-------|---------|---------------|--------------|
| 1 | Stop Streaming (`Cmd+.`) | 30 min | None |
| 2 | Conversation Starters | 1-2 hours | None |
| 3 | Slash Commands | 3-4 hours | None |

**Recommendation:** Start with Stop Streaming since it's lowest effort and highest user impact.

---

## Compatibility

All features work with existing backends:

### Claude Code Subscription
- Slash commands translate to prompts sent via Claude Code CLI
- Stop streaming cancels the spawned claude process
- Conversation starters just insert prompts

### Vercel AI Gateway
- Slash commands become prompt prefixes
- Stop streaming uses AbortController pattern
- No gateway-specific changes needed

---

## Research Sources

See `docs/research/` for detailed findings:
- `01-raycast-ai-chat.md` - Action panel, presets, attachments
- `03-vscode-copilot-chat.md` - Slash commands, @mentions
- `05-chatgpt-desktop.md` - `Cmd+.` stop, Chat Bar
- `08-ai-chat-shortcuts.md` - Keyboard patterns
- `12-prompt-suggestions.md` - Empty state patterns

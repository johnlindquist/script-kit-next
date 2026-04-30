---
name: Cmd+Enter AI
description: Local-first scriptlet prompts for the universal Cmd+Enter AI flow
icon: sparkles
---

# Cmd+Enter AI

Cmd+Enter is the universal AI entry point for selected text and Power Syntax composition.

The snippets in this bundle paste prompt text only. Select text in another app, run a scriptlet, then press Cmd+Enter to hand the prompt and selection to the universal AI action. This replaces the older Tab-AI flow for this shape of "do this to that" interaction.

## Summarize Selection

Paste a concise instruction, then press Cmd+Enter with the target text selected.

```metadata
keyword: cs
description: Prompt Cmd+Enter AI to summarize the selected text
```

```paste
Summarize the selected text in three bullets. Keep the original meaning and flag any uncertainty.
```

---

## Translate Selection

Paste a translation instruction, then press Cmd+Enter with the target text selected.

```metadata
keyword: ct
description: Prompt Cmd+Enter AI to translate the selected text
```

```paste
Translate the selected text into Spanish. Preserve names, code identifiers, links, and formatting.
```

---

For reusable Agent Chat behavior, define profiles under `aiPreferences.profiles` in your user config. See `scripts/examples/menu-syntax/agent-chat-profile-demo.ts` for a typed `metadata.menuSyntax` example that pairs a menu entry with an Agent Chat profile.

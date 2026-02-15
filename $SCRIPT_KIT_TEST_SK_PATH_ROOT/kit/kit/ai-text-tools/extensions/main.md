---
name: AI Text Tools
description: AI-powered text transformation using selected text
author: Script Kit
icon: sparkles
---

<!--
Template guide:
- YAML frontmatter fields:
  - name: Bundle title shown in Script Kit.
  - description: One-line summary shown with the bundle.
  - author: Bundle author label.
  - icon: Lucide icon slug in kebab-case (for example: sparkles, file-text, link).
- Each `##` section plus its `ts` code fence becomes one runnable command.
- Optional per-scriptlet metadata can be added in an HTML comment directly above a scriptlet fence.
  Common trigger fields:
  - keyword: !rewrite
  - shortcut: ctrl shift r
-->

# AI Text Tools

Transform selected text using AI with inline chat. Supports follow-up questions.

**Requirements:** Set `SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY`.

---

## Improve Writing

<!--
description: Enhance clarity, flow, and style of selected text
shortcut: ctrl shift i
-->

```ts
const text = await getSelectedText();
if (!text?.trim()) { await hud('No text selected'); exit(); }

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Improve Writing',
  system: `You are a professional editor. Improve text quality, grammar, and clarity while preserving voice. Provide improved version first, then explain key changes.`,
  messages: [{ role: 'user', content: `Improve this text:\n\n${text}` }],
});
```

---

## Explain This

<!--
description: Get a clear explanation of selected text
shortcut: ctrl shift e
-->

```ts
const text = await getSelectedText();
if (!text?.trim()) { await hud('No text selected'); exit(); }

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Explain This',
  system: `You are a patient teacher who explains concepts clearly. Use analogies when helpful. For code, explain what it does and how.`,
  messages: [{ role: 'user', content: `Please explain:\n\n${text}` }],
});
```

---

## Fix Grammar & Spelling

<!--
description: Correct grammar, spelling, and punctuation
shortcut: ctrl shift g
-->

```ts
const text = await getSelectedText();
if (!text?.trim()) { await hud('No text selected'); exit(); }

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Fix Grammar',
  system: `You are a meticulous proofreader. Fix grammar, spelling, punctuation. Preserve style. Provide corrected text first, then list changes.`,
  messages: [{ role: 'user', content: `Fix grammar and spelling:\n\n${text}` }],
});
```

---

## Summarize

<!--
description: Condense selected text into key points
shortcut: ctrl shift s
-->

```ts
const text = await getSelectedText();
if (!text?.trim()) { await hud('No text selected'); exit(); }

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Summarize',
  system: `Distill information to its essence. Provide clear, concise summary. Use bullet points for multiple ideas. Much shorter than original.`,
  messages: [{ role: 'user', content: `Summarize:\n\n${text}` }],
});
```

---

## Make Concise

<!--
description: Shorten text while preserving meaning
shortcut: ctrl shift c
-->

```ts
const text = await getSelectedText();
if (!text?.trim()) { await hud('No text selected'); exit(); }

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Make Concise',
  system: `Expert at economical writing. Remove redundancy and wordiness. Keep essential meaning. Aim for 30%+ reduction.`,
  messages: [{ role: 'user', content: `Make concise:\n\n${text}` }],
});
```

---

## Translate to English

<!--
description: Translate selected text to English
shortcut: ctrl shift t
-->

```ts
const text = await getSelectedText();
if (!text?.trim()) { await hud('No text selected'); exit(); }

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Translate',
  system: `Professional translator. Natural, fluent English preserving meaning and tone. Explain cultural references. Note source language.`,
  messages: [{ role: 'user', content: `Translate to English:\n\n${text}` }],
});
```

---

## Change Tone (Professional)

<!--
description: Rewrite text in a professional tone
shortcut: ctrl shift p
-->

```ts
const text = await getSelectedText();
if (!text?.trim()) { await hud('No text selected'); exit(); }

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Professional Tone',
  system: `Business communication expert. Rewrite to be professional and workplace-appropriate. Maintain core message, adjust tone and word choice.`,
  messages: [{ role: 'user', content: `Rewrite professionally:\n\n${text}` }],
});
```

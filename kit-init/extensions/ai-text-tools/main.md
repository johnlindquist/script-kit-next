---
name: AI Text Tools
description: AI-powered text transformation using selected text
author: Script Kit
icon: sparkles
---

# AI Text Tools

Transform selected text using AI. Select text in any application, trigger a command, and the AI window opens with your text and a specialized prompt.

**Requirements:** Configure an AI API key (`SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY`).

---

## Improve Writing

<!--
description: Enhance clarity, flow, and style of selected text
shortcut: ctrl shift i
-->

```ts
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}
if (!text?.trim()) {
  await hud('No text selected');
  exit();
}

await aiStartChat(`Please improve the following text. Focus on clarity, flow, and readability while maintaining the original meaning and tone:\n\n${text}`, {
  systemPrompt: 'You are a professional editor specializing in clear, engaging writing. Improve the text\'s quality, grammar, and clarity while preserving the original voice and intent. Provide the improved version first, then briefly explain key changes.'
});
await aiFocus();
```

---

## Explain This

<!--
description: Get a clear explanation of selected text or concept
shortcut: ctrl shift e
-->

```ts
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}
if (!text?.trim()) {
  await hud('No text selected');
  exit();
}

await aiStartChat(`Please explain the following:\n\n${text}`, {
  systemPrompt: 'You are a patient, knowledgeable teacher who explains concepts clearly. Provide explanations that are accessible yet thorough. Use analogies and examples when helpful. If the text is code, explain what it does and how it works.'
});
await aiFocus();
```

---

## Fix Grammar & Spelling

<!--
description: Correct grammar, spelling, and punctuation
shortcut: ctrl shift g
-->

```ts
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}
if (!text?.trim()) {
  await hud('No text selected');
  exit();
}

await aiStartChat(`Please fix the grammar, spelling, and punctuation in the following text:\n\n${text}`, {
  systemPrompt: 'You are a meticulous proofreader. Fix all grammar, spelling, and punctuation errors. Preserve the original style and tone. Provide the corrected text first, then list the corrections made.'
});
await aiFocus();
```

---

## Summarize

<!--
description: Condense selected text into key points
shortcut: ctrl shift s
-->

```ts
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}
if (!text?.trim()) {
  await hud('No text selected');
  exit();
}

await aiStartChat(`Please summarize the following text:\n\n${text}`, {
  systemPrompt: 'You are skilled at distilling information to its essence. Provide a clear, concise summary that captures the main points. Use bullet points for multiple key ideas. The summary should be significantly shorter than the original while retaining the essential meaning.'
});
await aiFocus();
```

---

## Make Concise

<!--
description: Shorten text while preserving meaning
shortcut: ctrl shift c
-->

```ts
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}
if (!text?.trim()) {
  await hud('No text selected');
  exit();
}

await aiStartChat(`Please make the following text more concise without losing important information:\n\n${text}`, {
  systemPrompt: 'You are an expert at clear, economical writing. Remove redundancy, wordiness, and unnecessary phrases. Keep the essential meaning intact. Aim for at least 30% reduction in length while maintaining clarity and completeness.'
});
await aiFocus();
```

---

## Translate to English

<!--
description: Translate selected text to English
shortcut: ctrl shift t
-->

```ts
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}
if (!text?.trim()) {
  await hud('No text selected');
  exit();
}

await aiStartChat(`Please translate the following text to English:\n\n${text}`, {
  systemPrompt: 'You are a professional translator. Translate the text to natural, fluent English while preserving the original meaning, tone, and nuance. If there are cultural references or idioms, explain them briefly. Provide the translation first, then note the source language.'
});
await aiFocus();
```

---

## Change Tone (Professional)

<!--
description: Rewrite text in a professional tone
shortcut: ctrl shift p
-->

```ts
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}
if (!text?.trim()) {
  await hud('No text selected');
  exit();
}

await aiStartChat(`Please rewrite the following text in a professional, business-appropriate tone:\n\n${text}`, {
  systemPrompt: 'You are a business communication expert. Rewrite the text to be professional, polished, and appropriate for workplace communication. Maintain the core message while adjusting tone, word choice, and structure for a professional context.'
});
await aiFocus();
```

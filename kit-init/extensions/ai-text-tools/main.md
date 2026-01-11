---
name: AI Text Tools
description: AI-powered text transformation using selected text
author: Script Kit
icon: sparkles
---

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
import { streamAiResponse, getModelName, type Message } from './stream-helper';

let text = '';
try { text = await getSelectedText(); } catch (e: any) {
  if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); }
}
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `You are a professional editor. Improve text quality, grammar, and clarity while preserving voice. Provide improved version first, then explain key changes.`;
const history: Message[] = [{ role: 'user', content: `Improve this text:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Improve Writing',
  footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }],
  system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    // Show user message in chat, then stream AI response
    chat.addMessage({ role: 'user', content: input });
    history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, messages: history,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
});
```

---

## Explain This

<!--
description: Get a clear explanation of selected text
shortcut: ctrl shift e
-->

```ts
import { streamAiResponse, getModelName, type Message } from './stream-helper';

let text = '';
try { text = await getSelectedText(); } catch (e: any) {
  if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); }
}
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `You are a patient teacher who explains concepts clearly. Use analogies when helpful. For code, explain what it does and how.`;
const history: Message[] = [{ role: 'user', content: `Please explain:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Explain This',
  footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }],
  system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    // Show user message in chat, then stream AI response
    chat.addMessage({ role: 'user', content: input });
    history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, messages: history,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
});
```

---

## Fix Grammar & Spelling

<!--
description: Correct grammar, spelling, and punctuation
shortcut: ctrl shift g
-->

```ts
import { streamAiResponse, getModelName, type Message } from './stream-helper';

let text = '';
try { text = await getSelectedText(); } catch (e: any) {
  if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); }
}
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `You are a meticulous proofreader. Fix grammar, spelling, punctuation. Preserve style. Provide corrected text first, then list changes.`;
const history: Message[] = [{ role: 'user', content: `Fix grammar and spelling:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Fix Grammar',
  footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }],
  system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    // Show user message in chat, then stream AI response
    chat.addMessage({ role: 'user', content: input });
    history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, messages: history,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
});
```

---

## Summarize

<!--
description: Condense selected text into key points
shortcut: ctrl shift s
-->

```ts
import { streamAiResponse, getModelName, type Message } from './stream-helper';

let text = '';
try { text = await getSelectedText(); } catch (e: any) {
  if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); }
}
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `Distill information to its essence. Provide clear, concise summary. Use bullet points for multiple ideas. Much shorter than original.`;
const history: Message[] = [{ role: 'user', content: `Summarize:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Summarize',
  footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }],
  system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    // Show user message in chat, then stream AI response
    chat.addMessage({ role: 'user', content: input });
    history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, messages: history,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
});
```

---

## Make Concise

<!--
description: Shorten text while preserving meaning
shortcut: ctrl shift c
-->

```ts
import { streamAiResponse, getModelName, type Message } from './stream-helper';

let text = '';
try { text = await getSelectedText(); } catch (e: any) {
  if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); }
}
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `Expert at economical writing. Remove redundancy and wordiness. Keep essential meaning. Aim for 30%+ reduction.`;
const history: Message[] = [{ role: 'user', content: `Make concise:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Make Concise',
  footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }],
  system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    // Show user message in chat, then stream AI response
    chat.addMessage({ role: 'user', content: input });
    history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, messages: history,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
});
```

---

## Translate to English

<!--
description: Translate selected text to English
shortcut: ctrl shift t
-->

```ts
import { streamAiResponse, getModelName, type Message } from './stream-helper';

let text = '';
try { text = await getSelectedText(); } catch (e: any) {
  if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); }
}
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `Professional translator. Natural, fluent English preserving meaning and tone. Explain cultural references. Note source language.`;
const history: Message[] = [{ role: 'user', content: `Translate to English:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Translate',
  footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }],
  system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    // Show user message in chat, then stream AI response
    chat.addMessage({ role: 'user', content: input });
    history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, messages: history,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
});
```

---

## Change Tone (Professional)

<!--
description: Rewrite text in a professional tone
shortcut: ctrl shift p
-->

```ts
import { streamAiResponse, getModelName, type Message } from './stream-helper';

let text = '';
try { text = await getSelectedText(); } catch (e: any) {
  if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); }
}
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `Business communication expert. Rewrite to be professional and workplace-appropriate. Maintain core message, adjust tone and word choice.`;
const history: Message[] = [{ role: 'user', content: `Rewrite professionally:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Professional Tone',
  footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }],
  system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    // Show user message in chat, then stream AI response
    chat.addMessage({ role: 'user', content: input });
    history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left');
    let response = '';
    await streamAiResponse({
      systemPrompt, messages: history,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
});
```

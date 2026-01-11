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
// Inline AI streaming helper (required because scriptlets run from temp files)
interface Message { role: 'user' | 'assistant' | 'system'; content: string; }

function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } {
  const anthropicKey = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY;
  const openaiKey = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY;
  if (anthropicKey) return { provider: 'anthropic', key: anthropicKey };
  if (openaiKey) return { provider: 'openai', key: openaiKey };
  return { provider: null, key: null };
}

function getModelName(): string {
  const { provider } = getAiProvider();
  if (provider === 'anthropic') return 'Claude 3.5 Sonnet';
  if (provider === 'openai') return 'GPT-4o';
  return 'No AI configured';
}

async function streamAiResponse(options: {
  systemPrompt: string;
  userPrompt?: string;
  messages?: Message[];
  onChunk: (chunk: string) => void;
  onError: (error: string) => void;
  onComplete: () => void;
}): Promise<void> {
  const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options;
  const { provider, key } = getAiProvider();

  if (!provider || !key) {
    onError('No AI API key configured. Set SCRIPT_KIT_ANTHROPIC_API_KEY or SCRIPT_KIT_OPENAI_API_KEY.');
    return;
  }

  const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }];

  try {
    if (provider === 'anthropic') {
      const userMessages = chatMessages.filter(m => m.role !== 'system');
      const response = await fetch('https://api.anthropic.com/v1/messages', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' },
        body: JSON.stringify({ model: 'claude-sonnet-4-20250514', max_tokens: 4096, system: systemPrompt, messages: userMessages.map(m => ({ role: m.role, content: m.content })), stream: true }),
      });
      if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); }
      const reader = response.body?.getReader(); if (!reader) throw new Error('No response body');
      const decoder = new TextDecoder(); let buffer = '';
      while (true) {
        const { done, value } = await reader.read(); if (done) break;
        buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || '';
        for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); if (p.type === 'content_block_delta' && p.delta?.text) onChunk(p.delta.text); } catch {} } }
      }
    } else {
      const allMessages = [{ role: 'system' as const, content: systemPrompt }, ...chatMessages.map(m => ({ role: m.role, content: m.content }))];
      const response = await fetch('https://api.openai.com/v1/chat/completions', {
        method: 'POST', headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${key}` },
        body: JSON.stringify({ model: 'gpt-4o', max_tokens: 4096, messages: allMessages, stream: true }),
      });
      if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); }
      const reader = response.body?.getReader(); if (!reader) throw new Error('No response body');
      const decoder = new TextDecoder(); let buffer = '';
      while (true) {
        const { done, value } = await reader.read(); if (done) break;
        buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || '';
        for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); const chunk = p.choices?.[0]?.delta?.content; if (chunk) onChunk(chunk); } catch {} } }
      }
    }
    onComplete();
  } catch (error: any) { onError(error.message || 'Failed to get AI response'); }
}

let text = '';
try { text = await getSelectedText(); } catch (e: any) { if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); } }
if (!text?.trim()) { await hud('No text selected'); exit(); }

const systemPrompt = `You are a professional editor. Improve text quality, grammar, and clarity while preserving voice. Provide improved version first, then explain key changes.`;
const history: Message[] = [{ role: 'user', content: `Improve this text:\n\n${text}` }];

await chat({
  placeholder: 'Ask follow-up...', hint: 'Improve Writing', footer: getModelName(),
  messages: [{ role: 'user', content: history[0].content }], system: systemPrompt,
  async onInit() {
    const msgId = chat.startStream('left'); let response = '';
    await streamAiResponse({ systemPrompt, userPrompt: history[0].content,
      onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); },
      onError: (err) => chat.setError(msgId, err),
      onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); },
    });
  },
  async onMessage(input: string) {
    chat.addMessage({ role: 'user', content: input }); history.push({ role: 'user', content: input });
    const msgId = chat.startStream('left'); let response = '';
    await streamAiResponse({ systemPrompt, messages: history,
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
interface Message { role: 'user' | 'assistant' | 'system'; content: string; }
function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } { const a = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY; const o = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY; if (a) return { provider: 'anthropic', key: a }; if (o) return { provider: 'openai', key: o }; return { provider: null, key: null }; }
function getModelName(): string { const { provider } = getAiProvider(); if (provider === 'anthropic') return 'Claude 3.5 Sonnet'; if (provider === 'openai') return 'GPT-4o'; return 'No AI configured'; }
async function streamAiResponse(options: { systemPrompt: string; userPrompt?: string; messages?: Message[]; onChunk: (chunk: string) => void; onError: (error: string) => void; onComplete: () => void; }): Promise<void> { const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options; const { provider, key } = getAiProvider(); if (!provider || !key) { onError('No AI API key configured.'); return; } const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }]; try { if (provider === 'anthropic') { const userMessages = chatMessages.filter(m => m.role !== 'system'); const response = await fetch('https://api.anthropic.com/v1/messages', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' }, body: JSON.stringify({ model: 'claude-sonnet-4-20250514', max_tokens: 4096, system: systemPrompt, messages: userMessages.map(m => ({ role: m.role, content: m.content })), stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); if (p.type === 'content_block_delta' && p.delta?.text) onChunk(p.delta.text); } catch {} } } } } else { const allMessages = [{ role: 'system' as const, content: systemPrompt }, ...chatMessages.map(m => ({ role: m.role, content: m.content }))]; const response = await fetch('https://api.openai.com/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${key}` }, body: JSON.stringify({ model: 'gpt-4o', max_tokens: 4096, messages: allMessages, stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); const chunk = p.choices?.[0]?.delta?.content; if (chunk) onChunk(chunk); } catch {} } } } } onComplete(); } catch (error: any) { onError(error.message || 'Failed to get AI response'); } }

let text = ''; try { text = await getSelectedText(); } catch (e: any) { if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); } } if (!text?.trim()) { await hud('No text selected'); exit(); }
const systemPrompt = `You are a patient teacher who explains concepts clearly. Use analogies when helpful. For code, explain what it does and how.`;
const history: Message[] = [{ role: 'user', content: `Please explain:\n\n${text}` }];
await chat({ placeholder: 'Ask follow-up...', hint: 'Explain This', footer: getModelName(), messages: [{ role: 'user', content: history[0].content }], system: systemPrompt, async onInit() { const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, userPrompt: history[0].content, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, async onMessage(input: string) { chat.addMessage({ role: 'user', content: input }); history.push({ role: 'user', content: input }); const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, messages: history, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, });
```

---

## Fix Grammar & Spelling

<!--
description: Correct grammar, spelling, and punctuation
shortcut: ctrl shift g
-->

```ts
interface Message { role: 'user' | 'assistant' | 'system'; content: string; }
function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } { const a = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY; const o = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY; if (a) return { provider: 'anthropic', key: a }; if (o) return { provider: 'openai', key: o }; return { provider: null, key: null }; }
function getModelName(): string { const { provider } = getAiProvider(); if (provider === 'anthropic') return 'Claude 3.5 Sonnet'; if (provider === 'openai') return 'GPT-4o'; return 'No AI configured'; }
async function streamAiResponse(options: { systemPrompt: string; userPrompt?: string; messages?: Message[]; onChunk: (chunk: string) => void; onError: (error: string) => void; onComplete: () => void; }): Promise<void> { const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options; const { provider, key } = getAiProvider(); if (!provider || !key) { onError('No AI API key configured.'); return; } const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }]; try { if (provider === 'anthropic') { const userMessages = chatMessages.filter(m => m.role !== 'system'); const response = await fetch('https://api.anthropic.com/v1/messages', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' }, body: JSON.stringify({ model: 'claude-sonnet-4-20250514', max_tokens: 4096, system: systemPrompt, messages: userMessages.map(m => ({ role: m.role, content: m.content })), stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); if (p.type === 'content_block_delta' && p.delta?.text) onChunk(p.delta.text); } catch {} } } } } else { const allMessages = [{ role: 'system' as const, content: systemPrompt }, ...chatMessages.map(m => ({ role: m.role, content: m.content }))]; const response = await fetch('https://api.openai.com/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${key}` }, body: JSON.stringify({ model: 'gpt-4o', max_tokens: 4096, messages: allMessages, stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); const chunk = p.choices?.[0]?.delta?.content; if (chunk) onChunk(chunk); } catch {} } } } } onComplete(); } catch (error: any) { onError(error.message || 'Failed to get AI response'); } }

let text = ''; try { text = await getSelectedText(); } catch (e: any) { if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); } } if (!text?.trim()) { await hud('No text selected'); exit(); }
const systemPrompt = `You are a meticulous proofreader. Fix grammar, spelling, punctuation. Preserve style. Provide corrected text first, then list changes.`;
const history: Message[] = [{ role: 'user', content: `Fix grammar and spelling:\n\n${text}` }];
await chat({ placeholder: 'Ask follow-up...', hint: 'Fix Grammar', footer: getModelName(), messages: [{ role: 'user', content: history[0].content }], system: systemPrompt, async onInit() { const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, userPrompt: history[0].content, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, async onMessage(input: string) { chat.addMessage({ role: 'user', content: input }); history.push({ role: 'user', content: input }); const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, messages: history, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, });
```

---

## Summarize

<!--
description: Condense selected text into key points
shortcut: ctrl shift s
-->

```ts
interface Message { role: 'user' | 'assistant' | 'system'; content: string; }
function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } { const a = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY; const o = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY; if (a) return { provider: 'anthropic', key: a }; if (o) return { provider: 'openai', key: o }; return { provider: null, key: null }; }
function getModelName(): string { const { provider } = getAiProvider(); if (provider === 'anthropic') return 'Claude 3.5 Sonnet'; if (provider === 'openai') return 'GPT-4o'; return 'No AI configured'; }
async function streamAiResponse(options: { systemPrompt: string; userPrompt?: string; messages?: Message[]; onChunk: (chunk: string) => void; onError: (error: string) => void; onComplete: () => void; }): Promise<void> { const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options; const { provider, key } = getAiProvider(); if (!provider || !key) { onError('No AI API key configured.'); return; } const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }]; try { if (provider === 'anthropic') { const userMessages = chatMessages.filter(m => m.role !== 'system'); const response = await fetch('https://api.anthropic.com/v1/messages', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' }, body: JSON.stringify({ model: 'claude-sonnet-4-20250514', max_tokens: 4096, system: systemPrompt, messages: userMessages.map(m => ({ role: m.role, content: m.content })), stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); if (p.type === 'content_block_delta' && p.delta?.text) onChunk(p.delta.text); } catch {} } } } } else { const allMessages = [{ role: 'system' as const, content: systemPrompt }, ...chatMessages.map(m => ({ role: m.role, content: m.content }))]; const response = await fetch('https://api.openai.com/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${key}` }, body: JSON.stringify({ model: 'gpt-4o', max_tokens: 4096, messages: allMessages, stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); const chunk = p.choices?.[0]?.delta?.content; if (chunk) onChunk(chunk); } catch {} } } } } onComplete(); } catch (error: any) { onError(error.message || 'Failed to get AI response'); } }

let text = ''; try { text = await getSelectedText(); } catch (e: any) { if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); } } if (!text?.trim()) { await hud('No text selected'); exit(); }
const systemPrompt = `Distill information to its essence. Provide clear, concise summary. Use bullet points for multiple ideas. Much shorter than original.`;
const history: Message[] = [{ role: 'user', content: `Summarize:\n\n${text}` }];
await chat({ placeholder: 'Ask follow-up...', hint: 'Summarize', footer: getModelName(), messages: [{ role: 'user', content: history[0].content }], system: systemPrompt, async onInit() { const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, userPrompt: history[0].content, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, async onMessage(input: string) { chat.addMessage({ role: 'user', content: input }); history.push({ role: 'user', content: input }); const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, messages: history, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, });
```

---

## Make Concise

<!--
description: Shorten text while preserving meaning
shortcut: ctrl shift c
-->

```ts
interface Message { role: 'user' | 'assistant' | 'system'; content: string; }
function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } { const a = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY; const o = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY; if (a) return { provider: 'anthropic', key: a }; if (o) return { provider: 'openai', key: o }; return { provider: null, key: null }; }
function getModelName(): string { const { provider } = getAiProvider(); if (provider === 'anthropic') return 'Claude 3.5 Sonnet'; if (provider === 'openai') return 'GPT-4o'; return 'No AI configured'; }
async function streamAiResponse(options: { systemPrompt: string; userPrompt?: string; messages?: Message[]; onChunk: (chunk: string) => void; onError: (error: string) => void; onComplete: () => void; }): Promise<void> { const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options; const { provider, key } = getAiProvider(); if (!provider || !key) { onError('No AI API key configured.'); return; } const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }]; try { if (provider === 'anthropic') { const userMessages = chatMessages.filter(m => m.role !== 'system'); const response = await fetch('https://api.anthropic.com/v1/messages', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' }, body: JSON.stringify({ model: 'claude-sonnet-4-20250514', max_tokens: 4096, system: systemPrompt, messages: userMessages.map(m => ({ role: m.role, content: m.content })), stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); if (p.type === 'content_block_delta' && p.delta?.text) onChunk(p.delta.text); } catch {} } } } } else { const allMessages = [{ role: 'system' as const, content: systemPrompt }, ...chatMessages.map(m => ({ role: m.role, content: m.content }))]; const response = await fetch('https://api.openai.com/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${key}` }, body: JSON.stringify({ model: 'gpt-4o', max_tokens: 4096, messages: allMessages, stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); const chunk = p.choices?.[0]?.delta?.content; if (chunk) onChunk(chunk); } catch {} } } } } onComplete(); } catch (error: any) { onError(error.message || 'Failed to get AI response'); } }

let text = ''; try { text = await getSelectedText(); } catch (e: any) { if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); } } if (!text?.trim()) { await hud('No text selected'); exit(); }
const systemPrompt = `Expert at economical writing. Remove redundancy and wordiness. Keep essential meaning. Aim for 30%+ reduction.`;
const history: Message[] = [{ role: 'user', content: `Make concise:\n\n${text}` }];
await chat({ placeholder: 'Ask follow-up...', hint: 'Make Concise', footer: getModelName(), messages: [{ role: 'user', content: history[0].content }], system: systemPrompt, async onInit() { const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, userPrompt: history[0].content, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, async onMessage(input: string) { chat.addMessage({ role: 'user', content: input }); history.push({ role: 'user', content: input }); const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, messages: history, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, });
```

---

## Translate to English

<!--
description: Translate selected text to English
shortcut: ctrl shift t
-->

```ts
interface Message { role: 'user' | 'assistant' | 'system'; content: string; }
function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } { const a = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY; const o = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY; if (a) return { provider: 'anthropic', key: a }; if (o) return { provider: 'openai', key: o }; return { provider: null, key: null }; }
function getModelName(): string { const { provider } = getAiProvider(); if (provider === 'anthropic') return 'Claude 3.5 Sonnet'; if (provider === 'openai') return 'GPT-4o'; return 'No AI configured'; }
async function streamAiResponse(options: { systemPrompt: string; userPrompt?: string; messages?: Message[]; onChunk: (chunk: string) => void; onError: (error: string) => void; onComplete: () => void; }): Promise<void> { const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options; const { provider, key } = getAiProvider(); if (!provider || !key) { onError('No AI API key configured.'); return; } const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }]; try { if (provider === 'anthropic') { const userMessages = chatMessages.filter(m => m.role !== 'system'); const response = await fetch('https://api.anthropic.com/v1/messages', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' }, body: JSON.stringify({ model: 'claude-sonnet-4-20250514', max_tokens: 4096, system: systemPrompt, messages: userMessages.map(m => ({ role: m.role, content: m.content })), stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); if (p.type === 'content_block_delta' && p.delta?.text) onChunk(p.delta.text); } catch {} } } } } else { const allMessages = [{ role: 'system' as const, content: systemPrompt }, ...chatMessages.map(m => ({ role: m.role, content: m.content }))]; const response = await fetch('https://api.openai.com/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${key}` }, body: JSON.stringify({ model: 'gpt-4o', max_tokens: 4096, messages: allMessages, stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); const chunk = p.choices?.[0]?.delta?.content; if (chunk) onChunk(chunk); } catch {} } } } } onComplete(); } catch (error: any) { onError(error.message || 'Failed to get AI response'); } }

let text = ''; try { text = await getSelectedText(); } catch (e: any) { if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); } } if (!text?.trim()) { await hud('No text selected'); exit(); }
const systemPrompt = `Professional translator. Natural, fluent English preserving meaning and tone. Explain cultural references. Note source language.`;
const history: Message[] = [{ role: 'user', content: `Translate to English:\n\n${text}` }];
await chat({ placeholder: 'Ask follow-up...', hint: 'Translate', footer: getModelName(), messages: [{ role: 'user', content: history[0].content }], system: systemPrompt, async onInit() { const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, userPrompt: history[0].content, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, async onMessage(input: string) { chat.addMessage({ role: 'user', content: input }); history.push({ role: 'user', content: input }); const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, messages: history, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, });
```

---

## Change Tone (Professional)

<!--
description: Rewrite text in a professional tone
shortcut: ctrl shift p
-->

```ts
interface Message { role: 'user' | 'assistant' | 'system'; content: string; }
function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } { const a = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY; const o = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY; if (a) return { provider: 'anthropic', key: a }; if (o) return { provider: 'openai', key: o }; return { provider: null, key: null }; }
function getModelName(): string { const { provider } = getAiProvider(); if (provider === 'anthropic') return 'Claude 3.5 Sonnet'; if (provider === 'openai') return 'GPT-4o'; return 'No AI configured'; }
async function streamAiResponse(options: { systemPrompt: string; userPrompt?: string; messages?: Message[]; onChunk: (chunk: string) => void; onError: (error: string) => void; onComplete: () => void; }): Promise<void> { const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options; const { provider, key } = getAiProvider(); if (!provider || !key) { onError('No AI API key configured.'); return; } const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }]; try { if (provider === 'anthropic') { const userMessages = chatMessages.filter(m => m.role !== 'system'); const response = await fetch('https://api.anthropic.com/v1/messages', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' }, body: JSON.stringify({ model: 'claude-sonnet-4-20250514', max_tokens: 4096, system: systemPrompt, messages: userMessages.map(m => ({ role: m.role, content: m.content })), stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); if (p.type === 'content_block_delta' && p.delta?.text) onChunk(p.delta.text); } catch {} } } } } else { const allMessages = [{ role: 'system' as const, content: systemPrompt }, ...chatMessages.map(m => ({ role: m.role, content: m.content }))]; const response = await fetch('https://api.openai.com/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${key}` }, body: JSON.stringify({ model: 'gpt-4o', max_tokens: 4096, messages: allMessages, stream: true }), }); if (!response.ok) { const err = await response.text().catch(() => ''); throw new Error(JSON.parse(err)?.error?.message || `API error (${response.status})`); } const reader = response.body?.getReader(); if (!reader) throw new Error('No response body'); const decoder = new TextDecoder(); let buffer = ''; while (true) { const { done, value } = await reader.read(); if (done) break; buffer += decoder.decode(value, { stream: true }); const lines = buffer.split('\n'); buffer = lines.pop() || ''; for (const line of lines) { if (line.startsWith('data: ')) { const data = line.slice(6); if (data === '[DONE]') continue; try { const p = JSON.parse(data); const chunk = p.choices?.[0]?.delta?.content; if (chunk) onChunk(chunk); } catch {} } } } } onComplete(); } catch (error: any) { onError(error.message || 'Failed to get AI response'); } }

let text = ''; try { text = await getSelectedText(); } catch (e: any) { if (e.message?.includes('Accessibility')) { await hud('Enable Accessibility'); exit(); } } if (!text?.trim()) { await hud('No text selected'); exit(); }
const systemPrompt = `Business communication expert. Rewrite to be professional and workplace-appropriate. Maintain core message, adjust tone and word choice.`;
const history: Message[] = [{ role: 'user', content: `Rewrite professionally:\n\n${text}` }];
await chat({ placeholder: 'Ask follow-up...', hint: 'Professional Tone', footer: getModelName(), messages: [{ role: 'user', content: history[0].content }], system: systemPrompt, async onInit() { const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, userPrompt: history[0].content, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, async onMessage(input: string) { chat.addMessage({ role: 'user', content: input }); history.push({ role: 'user', content: input }); const msgId = chat.startStream('left'); let response = ''; await streamAiResponse({ systemPrompt, messages: history, onChunk: (chunk) => { response += chunk; chat.appendChunk(msgId, chunk); }, onError: (err) => chat.setError(msgId, err), onComplete: () => { chat.completeStream(msgId); history.push({ role: 'assistant', content: response }); }, }); }, });
```

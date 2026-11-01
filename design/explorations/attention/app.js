// What comes first: one world (what you selected, copied, switched to and ran), one rule that
// ranks it, and five ways of drawing the result. Every design reads the same world and rule.
(() => {
  'use strict';
  const $ = (s) => document.querySelector(s);
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const icon = (n) => `<svg viewBox="0 0 24 24" aria-hidden="true">${(window.ICONS || {})[n] || ''}</svg>`;

  // Timing. HOLD_MS is the app's (crates/sk-one-window/src/contract.rs); the rest are this proposal's.
  const HOLD_MS = 8000;
  const NEWS_S = 120; // a copy leads for two minutes
  const PAST_S = 120; // the last run stays in the status place while ⌘Z can take it back
  const FEED_S = 600; // design C keeps things in "Just now" for ten minutes
  const REVEAL_AFTER_MS = 350;
  const MARKS = { subject: '#ffb86b', action: '#f5c542', target: '#6fdc9b', details: '#8fb8ff', owner: '#c9a7ff' };

  // ── The world behind the window ──
  const act = (name, iconName, enter, done, extra = {}) => ({ name, icon: iconName, enter, done, ...extra });
  const APPS = {
    Ghostty: {
      app: 'Ghostty', title: 'zsh — ~/projects/site', short: 'zsh', icon: 'terminal',
      selection: 'TypeError: Cannot read properties of undefined (reading ‘map’)',
      selActions: [
        act('Explain this error', 'sparkles', 'Ask', 'Asked about the TypeError'),
        act('Ask the agent to fix it', 'bot', 'Ask Agent', 'Asked the agent to fix the TypeError'),
        act('Search the web for it', 'globe', 'Search', 'Searched the web for the TypeError'),
      ],
      actions: [
        act('Explain the last error', 'sparkles', 'Ask', 'Asked about the last error in zsh'),
        act('Open a new tab here', 'terminal', 'Open Tab', 'Opened a new Ghostty tab'),
        act('Move it to the left half', 'app-window', 'Move', 'Moved zsh to the left half', { undo: true }),
      ],
    },
    Chrome: {
      app: 'Chrome', title: 'Fix footer label · Pull Request #412', short: 'PR #412', icon: 'globe',
      selection: 'the footer kept the old label',
      selActions: [
        act('Explain it', 'sparkles', 'Ask', 'Asked about “the footer kept the old label”'),
        act('Save it to Brain inbox', 'inbox', 'Save', 'Saved the selection to Brain inbox', { undo: true }),
        act('Search for it', 'search', 'Search', 'Searched for the selection'),
      ],
      actions: [
        act('Summarize this page', 'sparkles', 'Ask', 'Summarized PR #412'),
        act('Copy the page link', 'link', 'Copy', 'Copied the PR #412 link'),
        act('Move it to the left half', 'app-window', 'Move', 'Moved Chrome to the left half', { undo: true }),
      ],
    },
    Mail: {
      app: 'Mail', title: 'Re: launch checklist — Dana', short: 'Dana’s email', icon: 'mail',
      selection: 'Can we move the launch to Friday?',
      selActions: [
        act('Reply to it', 'reply', 'Reply', 'Drafted a reply to Dana'),
        act('Add it to today', 'calendar', 'Add', 'Added “move the launch to Friday?” to today', { undo: true }),
        act('Translate it', 'type', 'Translate', 'Translated the selection'),
      ],
      actions: [
        act('Reply to Dana', 'reply', 'Reply', 'Drafted a reply to Dana'),
        act('Summarize the thread', 'sparkles', 'Ask', 'Summarized the launch thread'),
        act('Move it to the left half', 'app-window', 'Move', 'Moved Mail to the left half', { undo: true }),
      ],
    },
  };
  const LINK = 'github.com/script-kit/app/pull/412';
  const LINK_ACTIONS = [
    act('Open the link', 'external-link', 'Open', 'Opened the copied link'),
    act('Summarize the page', 'sparkles', 'Ask', 'Summarized the copied link'),
    act('Save it to Brain inbox', 'inbox', 'Save', 'Saved the link to Brain inbox', { undo: true }),
  ];
  const TEXT_ACTIONS = [
    act('Explain it', 'sparkles', 'Ask', 'Asked about the copied text'),
    act('Save it to Brain inbox', 'inbox', 'Save', 'Saved the copied text to Brain inbox', { undo: true }),
    act('Translate it', 'type', 'Translate', 'Translated the copied text'),
  ];
  const REMINDER = { title: 'Added reminder: call mom', sentence: 'remind me to call mom tomorrow at 5pm', icon: 'alarm-clock', undo: true };
  const OLDER_RUNS = [
    { title: 'Asked about the last error in zsh', sentence: 'explain the last error', icon: 'sparkles', at: -7200 },
    { title: 'Moved zsh to the left half', sentence: 'move it to the left half', icon: 'app-window', at: -5400 },
  ];
  const STANDING = [
    { title: 'Brain inbox', rows: [{ id: 'inbox', name: 'Review the launch checklist', desc: 'Captured yesterday', icon: 'inbox', enter: 'Open', done: 'Opened “Review the launch checklist”' }] },
    { title: 'Flows', rows: [{ id: 'conv', name: 'Conversation', desc: 'Talk it through with the model', icon: 'chevrons-right', enter: 'Run', done: 'Ran Conversation' }] },
    {
      title: 'Suggested', rows: [
        { id: 'agent', name: 'Agent Chat', desc: 'Start a conversation with an agent', icon: 'bot', enter: 'Open', done: 'Opened Agent Chat' },
        { id: 'today', name: 'Today', desc: 'Today’s page', icon: 'calendar', enter: 'Open', done: 'Opened Today' },
        { id: 'files', name: 'Search Files', desc: 'Find a file by name', icon: 'folder-search', enter: 'Open', done: 'Opened Search Files' },
        { id: 'clips', name: 'Clipboard History', desc: 'Everything you copied today', icon: 'clipboard-list', enter: 'Open', done: 'Opened Clipboard History' },
        { id: 'trash', name: 'Empty the Trash', desc: 'Waits 8 s before it runs', icon: 'trash-2', enter: 'Empty Trash', hold: true, done: 'Emptied the Trash' },
      ],
    },
  ];

  function blankWorld() {
    return {
      now: 0, app: 'Ghostty', switchedAt: -3600, selection: null, clip: null,
      history: OLDER_RUNS.map((r) => ({ ...r, undo: false })), hold: null, failure: null, notice: null,
      ignored: {}, used: {},
    };
  }
  const ran = (w, r, at) => w.history.push({ ...r, at });
  const STORIES = {
    back: { name: 'Back in the terminal', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -600; w.clip = { kind: 'link', text: LINK, at: -2400 }; ran(w, { ...REMINDER, undo: false }, -3700); } },
    copied: { name: 'Just copied a link', build: (w) => { w.app = 'Chrome'; w.switchedAt = -300; w.clip = { kind: 'link', text: LINK, at: -4 }; ran(w, { ...REMINDER, undo: false }, -3600); } },
    selected: { name: 'Selected an error', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -120; w.clip = { kind: 'link', text: LINK, at: -50 }; w.selection = { text: APPS.Ghostty.selection, at: -3 }; ran(w, { ...REMINDER, undo: false }, -3600); } },
    ran: { name: 'Just ran something', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -600; ran(w, REMINDER, -15); } },
    all: { name: 'Everything at once', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -90; w.clip = { kind: 'link', text: LINK, at: -40 }; ran(w, REMINDER, -25); w.selection = { text: APPS.Ghostty.selection, at: -6 }; w.hold = { title: 'Empty the Trash', done: 'Emptied the Trash', icon: 'trash-2', elapsed: 2500 }; } },
    ignored: { name: 'Saw the link, didn’t use it', build: (w) => { w.app = 'Chrome'; w.switchedAt = -300; w.clip = { kind: 'link', text: LINK, at: -70 }; w.ignored['clip@-70'] = true; ran(w, { ...REMINDER, undo: false }, -3600); } },
    same: { name: 'Selected and copied the same text', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -100; w.selection = { text: APPS.Ghostty.selection, at: -8 }; w.clip = { kind: 'text', text: APPS.Ghostty.selection, at: -6 }; ran(w, { ...REMINDER, undo: false }, -3600); } },
    failed: { name: 'A save failed', build: (w) => { w.app = 'Mail'; w.switchedAt = -60; w.failure = { title: 'Couldn’t save the note', at: -10, seen: false }; ran(w, { ...REMINDER, undo: false }, -3600); } },
  };

  const DESIGNS = {
    A: {
      name: 'Today', blurb: 'What the app does now', classic: true,
      summary: 'The line above the input names the last run at any age, so after your first run it never names the window again. The dashed chip offers the window, or the selection when there is one. What you copied stays hidden until ⌘V.',
      gives: ['One line and one chip; nothing in the list changes on its own.'],
      costs: ['The last run keeps the line for hours, even right after you copy something.', 'Nothing says that ⌘V would act on what you copied.', 'A run that waits and the last run share the same line.'],
    },
    B: {
      name: 'One leader', blurb: 'The rule picks; the rest whisper',
      summary: 'The rule picks one leader. It is the input’s icon, and its three best actions are the first rows, under a header that names it. Anything else still fresh sits at that header’s right end as a name and its key. ⇥ makes the next one lead.',
      gives: ['↵ on an empty input does the most likely thing.', 'Runners-up are one key away, and they say which key.'],
      costs: ['When the rule guesses wrong, the first rows are about the wrong thing until you press ⇥.', 'The top of the empty list changes as you work.'],
    },
    C: {
      name: 'Just now', blurb: 'Everything recent, newest first',
      summary: 'No leader. Everything from the last ten minutes is a row in one “Just now” section, newest first: what you selected, copied, switched to and ran. Each row’s ↵ does its most likely action; ⌘K shows the rest.',
      gives: ['Time order needs no explaining.', 'The past and the present sit together, so ↑ is never a mystery.'],
      costs: ['Rows name things, not actions, so you read before you act.', 'A busy minute pushes your inbox and flows down.'],
    },
    D: {
      name: 'Switch the subject', blurb: 'Every candidate is a tab',
      summary: 'Every candidate is a small tab in the first header, leader first and selected. ⇥ moves to the next tab, and the rows below are that one’s actions. The last run is always the last tab.',
      gives: ['You see every candidate and which one is in charge.', '⇥ does the same thing in every state.'],
      costs: ['Three or four labels at rest, even when you don’t need them.', 'Tabs are a new kind of control in the window.'],
    },
    E: {
      name: 'Quiet until asked', blurb: 'Only the icon shows it',
      summary: 'Only the input’s icon shows the leader. The list stays your standing list of inbox, flows and suggestions. Holding ⌘ shows a key for each candidate: ␣ this, ⇥ the next, ⌘V what you copied, ↑ the last run.',
      gives: ['The calmest window. The list never changes on its own.', 'Nothing competes, because nothing extra is shown.'],
      costs: ['You have to know to hold ⌘ or press Space.', 'A fresh copy or selection is easy to miss.'],
    },
  };

  // ── State ──
  let W = blankWorld();
  const S = { design: 'B', story: 'all', rule: 'newest', theme: 'dark', text: '', sel: 0, staged: null, pick: 0, cmd: false, running: true, log: [] };
  let rows = [];
  let cmdTimer = null;

  // ── Words ──
  function ago(sec) {
    if (sec < 60) return 'just now';
    const mins = sec / 60;
    if (mins < 60) return `>${[30, 15, 5, 1].find((s) => mins >= s)}m ago`;
    if (mins < 1440) return `>${Math.floor(mins / 60)}h ago`;
    return `>${Math.floor(mins / 1440)}d ago`;
  }
  function exact(sec) {
    sec = Math.max(0, Math.round(sec));
    if (sec < 60) return `${sec} s ago`;
    if (sec < 3600) return `${Math.floor(sec / 60)} min ago`;
    return `${Math.floor(sec / 3600)} h ago`;
  }
  // Compact ages for the feed, where order by time is the point.
  function short(sec) {
    sec = Math.max(0, Math.round(sec));
    if (sec < 60) return `${sec}s`;
    if (sec < 3600) return `${Math.floor(sec / 60)}m`;
    return `${Math.floor(sec / 3600)}h`;
  }
  const clipLabel = (c) => (c.kind === 'link' ? 'Copied link' : 'Copied text');
  const trim = (s, n) => (s.length > n ? s.slice(0, n - 1) + '…' : s);
  function keys(k, small) {
    const parts = k === 'esc' ? ['esc'] : k === 'Space' ? ['␣'] : [...k];
    return parts.map((p) => `<span class="kc${small ? ' kc--sm' : ''}">${esc(p)}</span>`).join('');
  }
  const hint = (key, words) => `<span class="hint">${words ? `<span data-ink="label">${esc(words)}</span>` : ''}<span data-ink="key">${keys(key, true)}</span></span>`;

  // ── The rule ──
  // Everything you could mean right now, as items with the moment they happened.
  function present() {
    const front = APPS[W.app];
    const items = [];
    const sel = W.selection;
    const dup = !!(sel && W.clip && W.clip.kind === 'text' && W.clip.text === sel.text);
    if (sel) {
      items.push({
        id: 'selection', key: `selection@${sel.at}`, at: dup ? Math.max(sel.at, W.clip.at) : sel.at, icon: 'text-cursor',
        label: dup ? `Selected and copied in ${front.app}` : `Selected in ${front.app}`, short: 'Selected text',
        title: `“${sel.text}”`, actions: front.selActions, key2: 'Space',
      });
    }
    if (W.clip) {
      items.push({
        id: 'clip', key: `clip@${W.clip.at}`, at: W.clip.at, icon: W.clip.kind === 'link' ? 'link' : 'clipboard',
        label: W.clip.kind === 'link' ? 'You copied a link' : 'You copied text', short: clipLabel(W.clip),
        title: W.clip.kind === 'link' ? W.clip.text : `“${W.clip.text}”`, actions: W.clip.kind === 'link' ? LINK_ACTIONS : TEXT_ACTIONS, key2: '⌘V', dup,
      });
    }
    items.push({ id: 'window', key: `window@${W.app}`, at: W.switchedAt, icon: front.icon, label: front.app, short: front.app, title: front.title, actions: front.actions, floor: true });
    return items;
  }
  // Why an item can or can't lead under "newest wins".
  function newsOf(it) {
    if (it.floor) return 'floor';
    if (it.dup) return 'once';
    if (W.used[it.key]) return 'used';
    if (W.ignored[it.key]) return 'ignored';
    if (it.id === 'clip' && W.now - it.at > NEWS_S) return 'faded';
    return 'news';
  }
  // The ranking: leader first, then everything else you could switch to with ⇥.
  function rank() {
    const items = present().map((it) => ({ ...it, state: newsOf(it) }));
    const live = items.filter((it) => !it.dup);
    if (S.rule === 'fixed' || DESIGNS[S.design].classic) {
      const order = ['selection', 'clip', 'window'];
      live.sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
      return { items, order: live, leader: live[0], whispers: live.slice(1).filter((it) => !it.floor) };
    }
    const contenders = live.filter((it) => it.state === 'news' || it.floor).sort((a, b) => b.at - a.at);
    const leader = contenders[0];
    const rest = live.filter((it) => it !== leader).sort((a, b) => (a.state === 'news') === (b.state === 'news') ? b.at - a.at : a.state === 'news' ? -1 : 1);
    return { items, order: [leader, ...rest], leader, whispers: rest.filter((it) => it.state === 'news') };
  }
  // "This": the leader, or whichever one ⇥ moved to.
  function thisItem(R = rank()) {
    const cands = tabCandidates(R);
    return cands[((S.pick % cands.length) + cands.length) % cands.length];
  }
  // What ⇥ cycles through: the leader, fresh runners-up, the window and (design D) the last run.
  function tabCandidates(R = rank()) {
    const out = R.order.filter((it) => it === R.leader || it.floor || S.rule === 'fixed' || it.state === 'news');
    if (S.design === 'D' && lastRun()) out.push(lastItem());
    return out;
  }
  // Fresh things other than "this": shown as a name and a key.
  function whispersFor(it, R) {
    const fresh = R.order.filter((o) => !o.floor && (S.rule === 'fixed' || o.state === 'news'));
    return fresh.filter((o) => o.key !== it.key);
  }
  const lastRun = () => W.history[W.history.length - 1];
  function lastItem(idx = W.history.length - 1) {
    const e = W.history[idx];
    const live = idx === W.history.length - 1 && e.undo && W.now - e.at <= PAST_S;
    const actions = [
      ...(live ? [act('Undo it', 'undo-2', 'Undo', null, { action: 'undo' })] : []),
      act(`Change the words: “${trim(e.sentence, 30)}”`, 'pencil', 'Edit', null, { action: 'edit', sentence: e.sentence }),
      act('Run it again', 'repeat', 'Run Again', e.title, { undo: e.undo }),
      act('Save it as a script', 'save', 'Save', `Saved “${trim(e.title, 30)}” as a script`),
    ];
    return { id: 'last', key: `last@${e.at}`, at: e.at, icon: 'history', label: 'You ran', short: 'Last run', title: e.title, actions, idx, past: true };
  }

  // What the status place says: needs-you first, then the last run while ⌘Z can take it back.
  function statusModel() {
    if (W.hold) {
      const paused = S.text !== '';
      const left = Math.max(0, Math.ceil((HOLD_MS - W.hold.elapsed) / 1000));
      return { kind: 'hold', icon: W.hold.icon || 'trash-2', state: paused ? 'Paused while you type' : `Runs in ${left}s`, title: W.hold.title, progress: W.hold.elapsed / HOLD_MS, hints: S.text === '' ? [['⌘Z', 'Stop']] : [] };
    }
    if (W.failure && !W.failure.seen) return { kind: 'failed', icon: 'circle-alert', failed: true, title: W.failure.title, sub: ago(W.now - W.failure.at) };
    const last = lastRun();
    const classic = DESIGNS[S.design].classic;
    if (W.notice && (classic || W.now - W.notice.at <= PAST_S) && (!last || W.notice.at >= last.at)) return { kind: 'notice', icon: 'history', title: W.notice.text, sub: ago(W.now - W.notice.at) };
    if (S.design === 'C') return { kind: 'rest' };
    if (last && (classic || W.now - last.at <= PAST_S)) {
      const hints = [];
      if (S.text === '' && !S.staged && last.undo && W.now - last.at <= PAST_S) hints.push(['⌘Z', 'Undo']);
      if (classic && S.text === '' && !S.staged && S.sel === 0) hints.push(['↑', '']);
      return { kind: 'recent', icon: 'history', title: last.title, sub: ago(W.now - last.at), hints, lit: !!(S.staged && S.staged.kind === 'last') };
    }
    return { kind: 'rest' };
  }
  const statusOpacity = (m) => (m.kind === 'hold' || m.lit || m.failed ? 1 : S.text !== '' ? 0.5 : 0.85);
  function statusBody(m) {
    const state = m.state ? `<span class="state" data-ink="label">${esc(m.state)}</span>` : '';
    return `<span class="icon">${icon(m.icon)}</span>${state}<span class="title" data-ink="label">${esc(m.title)}</span>${m.sub ? `<span class="sub" data-ink="label">· ${esc(m.sub)}</span>` : ''}${(m.hints || []).map(([k, w]) => hint(k, w)).join('')}`;
  }

  // ── What a sentence is about ──
  function aboutOf(text, R) {
    const lower = text.toLowerCase();
    const find = (re) => { const m = re.exec(lower); return m ? [m.index, m.index + m[0].length] : null; };
    const clip = R.items.find((it) => it.id === 'clip');
    const tests = [
      [/\b(the link|the copied \w+|what i copied|the clipboard)\b/, () => clip],
      [/\b(the last one|the reminder|again)\b/, () => (lastRun() ? lastItem() : null)],
      [/\b(this window|the window|this terminal|the terminal|this page|the page|this email|the email)\b/, () => R.items.find((it) => it.floor)],
      [/\b(this|it|that)\b/, () => staged() || thisItem(R)],
    ];
    for (const [re, pick] of tests) {
      const span = find(re);
      const item = span && pick();
      if (item) return { item, span };
    }
    return null;
  }

  // ── The list ──
  function staged() {
    const st = S.staged;
    if (!st) return null;
    if (st.kind === 'last') return lastItem(st.idx);
    if (st.kind === 'row') return { id: 'row', icon: st.row.icon, short: st.row.name, title: st.row.name, actions: [
      { ...st.row, name: st.row.hold ? 'Empty it' : 'Run it', holdTitle: st.row.name },
      act('Pin it', 'pin', 'Pin', `Pinned ${st.row.name}`, { undo: true }),
      act('Copy deep link', 'clipboard-list', 'Copy', 'Copied a deep link'),
    ] };
    return present().find((it) => it.id === st.id) || null;
  }
  const withSource = (item) => item.actions.map((a) => ({ ...a, source: item.key }));
  function keyMap(R) {
    const t = thisItem(R);
    const out = [{ name: `This: ${t.short}`, desc: t.title, icon: t.icon, key: 'Space', keymap: true, stage: t.id === 'last' ? { kind: 'last', idx: t.idx } : { kind: 'item', id: t.id } }];
    // ⌘V already names the copy, so ⇥ names the next thing that isn't it.
    const next = tabCandidates(R).filter((c) => c.key !== t.key && c.id !== 'clip')[0];
    if (next) out.push({ name: `Next: ${next.short}`, desc: next.title, icon: next.icon, key: '⇥', keymap: true, cycle: true });
    const clip = R.items.find((it) => it.id === 'clip');
    if (clip) out.push({ name: `What you copied: ${clip.short.replace('Copied ', '')}`, desc: clip.title, icon: clip.icon, key: '⌘V', keymap: true, stage: { kind: 'item', id: 'clip' } });
    if (lastRun()) out.push({ name: `Last run: ${lastRun().title}`, desc: '↑ again for earlier runs', icon: 'history', key: '↑', keymap: true, stage: { kind: 'last', idx: W.history.length - 1 } });
    return [{ title: 'Keys', rows: out }];
  }
  function feedRows(R) {
    const out = [];
    for (const it of R.items) {
      if (it.dup) continue;
      if (!it.floor && (it.state === 'used' || it.state === 'ignored' || W.now - it.at > FEED_S)) continue;
      const verb = it.actions[0];
      const name = it.id === 'selection' ? `Selected ${trim(it.title, 46)}` : it.id === 'clip' ? `Copied ${trim(it.title, 46)}` : `${it.label} · ${it.title}`;
      out.push({ ...verb, name, desc: `↵ ${verb.name} · ⌘K for more`, icon: it.icon, age: it.at, source: it.key, feedItem: it });
    }
    const last = lastRun();
    if (last && W.now - last.at <= FEED_S) {
      const li = lastItem();
      const first = li.actions[0];
      out.push({ ...first, name: last.title, desc: `↵ ${first.name.replace(/:.*/, '')} · ⌘K for more`, icon: 'history', age: last.at, feedItem: li });
    }
    return out.sort((a, b) => b.age - a.age);
  }
  function listModel(R) {
    const d = S.design;
    const t = S.text.trim();
    const st = staged();
    if (S.cmd && !DESIGNS[d].classic && !st && !t) return [...keyMap(R), ...STANDING];
    if (st) {
      const lower = t.toLowerCase();
      return [{ title: 'Or pick one', rows: withSource(st).filter((r) => !lower || r.name.toLowerCase().includes(lower)) }];
    }
    if (t) {
      const about = aboutOf(t, R);
      const head = about
        ? { name: t[0].toUpperCase() + t.slice(1), desc: `About ${about.item.short.toLowerCase()}: ${trim(about.item.title, 60)}`, icon: about.item.icon, enter: 'Do It', done: `Did “${trim(t, 40)}”`, source: about.item.key, jev: 1 }
        : { name: `Ask the model: “${t}”`, desc: 'Opens the chat with these words', icon: 'sparkles', enter: 'Ask', done: 'Asked the model', jev: 1 };
      const matches = STANDING.flatMap((s) => s.rows).filter((r) => r.name.toLowerCase().includes(t.toLowerCase()));
      const sections = [{ title: 'Jev', jev: true, about: about && about.item, rows: [head] }];
      if (matches.length) sections.push({ title: 'Matches', rows: matches });
      sections.push({ title: `Use “${trim(t, 28)}” with`, rows: [
        { name: 'Search Google', icon: 'globe', enter: 'Search Google', done: `Searched Google for “${t}”`, fallback: true },
        { name: `Search Files for “${trim(t, 40)}”`, icon: 'folder-search', enter: 'Search Files', done: 'Searched Files', fallback: true },
      ] });
      return sections;
    }
    if (d === 'A' || d === 'E') return STANDING;
    if (d === 'B') { const it = thisItem(R); return [{ kind: 'leader', item: it, rows: withSource(it) }, ...STANDING]; }
    if (d === 'C') return [{ title: 'Just now', rows: feedRows(R) }, ...STANDING];
    if (d === 'D') { const it = thisItem(R); return [{ kind: 'tabs', item: it, rows: withSource(it) }, ...STANDING]; }
    return STANDING;
  }

  // ── Drawing ──
  const win = $('#win'), q = $('#q');
  function marksFor(text, about) {
    const spans = [];
    const verb = /^(explain|summarize|open|translate|reply|save|remind me|fix|search)\b/i.exec(text);
    if (verb) spans.push([0, verb[0].length, 'action']);
    if (about) spans.push([about.span[0], about.span[1], 'subject']);
    spans.sort((a, b) => a[0] - b[0]);
    let out = '', at = 0;
    for (const [s, e, w] of spans) {
      if (s < at) continue;
      out += esc(text.slice(at, s)) + `<span class="mark" style="text-decoration-color:${MARKS[w]}">${esc(text.slice(s, e))}</span>`;
      at = e;
    }
    return out + esc(text.slice(at));
  }

  function headerHtml(s, R, classic) {
    const style = classic ? 'caps' : 'quiet';
    if (s.kind === 'leader') {
      const it = s.item;
      const wh = whispersFor(it, R).map((o) => `<span class="whisper" data-whisper="${o.id}" title="${esc(o.title)}"><span class="icon">${icon(o.icon)}</span><span data-ink="label">${esc(o.short)}</span><span data-ink="key">${keys(o.id === 'clip' ? '⌘V' : '⇥', true)}</span></span>`).join('');
      return `<div class="hdr" data-style="${style}"><span class="hicon">${icon(it.icon)}</span><span class="label" data-ink="label">${esc(it.label)}</span><span class="hsub" data-ink="label">${esc(trim(it.title, 44))}</span>${wh ? `<span class="whispers">${wh}</span>` : ''}</div>`;
    }
    if (s.kind === 'tabs') {
      const cands = tabCandidates(R);
      const tabs = cands.map((c, i) => `<span class="tab" role="tab" data-tab="${i}" aria-selected="${c.key === s.item.key}" title="${esc(c.title)}"><span class="icon">${icon(c.icon)}</span><span class="tname" data-ink="label">${esc(c.short)}</span></span>`).join('');
      return `<div class="hdr hdr--tabs" data-style="${style}" role="tablist">${tabs}<span class="acc"><span data-ink="key">${keys('⇥', true)}</span></span></div>`;
    }
    let acc = '';
    if (s.jev && s.about) acc = `<span class="whisper"><span class="icon">${icon(s.about.icon)}</span><span data-ink="label">About ${esc(s.about.short.toLowerCase())}</span></span>`;
    else if (s.title === 'Keys') acc = '<span data-ink="label">Release ⌘ to hide</span>';
    return `<div class="hdr" data-style="${style}"><span class="label" data-ink="label">${esc(s.title)}</span>${acc ? `<span class="acc">${acc}</span>` : ''}</div>`;
  }

  function render() {
    const d = DESIGNS[S.design];
    const classic = !!d.classic;
    const R = rank();
    win.dataset.theme = S.theme;
    win.dataset.layout = classic ? 'classic' : 'row';
    win.dataset.row = classic ? 'classic' : 'aligned';
    win.style.setProperty('--in-size', classic ? '20px' : '18px');
    const st = staged();
    q.placeholder = st ? 'Say what to do with it' : 'Search or ask';
    if (q.value !== S.text) q.value = S.text;
    const sm = statusModel();

    // Today: the 34 pt line above the input names the last run, or what's behind the window.
    const line = $('#line');
    if (classic) {
      const focus = R.items.find((it) => it.id === 'selection') || R.items.find((it) => it.floor);
      const m = sm.kind === 'rest' ? { kind: 'behind', icon: focus.icon, title: focus.title, sub: focus.floor ? `in ${focus.label}` : `in ${W.app}` } : sm;
      line.dataset.kind = m.kind;
      line.dataset.lit = String(!!m.lit);
      line.dataset.failed = String(!!m.failed);
      line.style.opacity = m.kind === 'hold' || m.failed ? 1 : S.text ? 0.5 : 0.8;
      const hints = (m.hints || []).map(([k, w]) => hint(k, w)).join('');
      line.innerHTML = `${statusBody({ ...m, hints: [] })}<span class="grow"></span>${hints}${m.kind === 'hold' ? `<span class="progress" style="width:${(m.progress * 100).toFixed(1)}%"></span>` : ''}`;
    } else line.innerHTML = '';

    // The input row: "this" as the leading tile (or the staged chip); Today's dashed chip at the end.
    let lead = '', trail = '';
    if (st) {
      const extra = S.staged.kind === 'last' && W.history.length > 1 ? ` <span class="word" data-ink="label">${S.staged.idx + 1} of ${W.history.length}</span>` : '';
      lead = `<span class="chip chip--attached"><span class="icon">${icon(st.icon)}</span><span class="name" data-ink="label">${esc(trim(st.id === 'last' ? st.title : st.short, 34))}</span>${extra}${hint('esc', '')}</span>`;
    } else if (!classic) {
      const t = thisItem(R);
      const asKey = S.cmd && S.text === '';
      lead = `<span class="ftile${asKey ? ' ftile--key reveal' : ''}" id="ftile" data-kind="${t.id}" role="button" tabindex="-1" title="${esc(t.short)} · ${esc(t.title)}" aria-label="Act on ${esc(t.short)}: ${esc(t.title)}" data-typing="${S.text !== ''}">${asKey ? '<span data-ink="key" class="tilekey">␣</span>' : icon(t.icon)}</span>`;
    }
    if (classic && !st && S.text === '') {
      const focus = R.items.find((it) => it.id === 'selection') || R.items.find((it) => it.floor);
      trail = `<span class="chip"><span class="icon">${icon(focus.icon)}</span><span class="name" data-ink="label">${esc(trim(focus.id === 'selection' ? focus.title : focus.label, 28))}</span><span class="hint"><span class="word" data-ink="label">Use</span><span data-ink="key">${keys('Space', true)}</span></span></span>`;
    }
    $('#lead').innerHTML = lead;
    $('#trail').innerHTML = trail;
    $('#inprogress').style.width = !classic && sm.kind === 'hold' ? `${(sm.progress * 100).toFixed(1)}%` : '0';
    const about = S.text.trim() ? aboutOf(S.text, R) : null;
    $('#mirror').innerHTML = marksFor(S.text, about);
    $('#mirror').scrollLeft = q.scrollLeft;

    // The list.
    const sections = listModel(R);
    rows = sections.flatMap((s) => s.rows);
    S.sel = Math.max(0, Math.min(S.sel, rows.length - 1));
    let i = 0, html = '';
    for (const s of sections) {
      html += headerHtml(s, R, classic);
      for (const r of s.rows) {
        let accHtml = '';
        if (r.keymap) accHtml = `<span class="rowkey" data-ink="key">${keys(r.key, true)}</span>`;
        else if (r.age !== undefined) accHtml = `<span class="age">${esc(short(W.now - r.age))}</span>`;
        else if (r.jev) accHtml = `<span class="acc-icon">${icon('sparkles')}</span>`;
        else if (r.fallback) accHtml = `<span class="acc-icon">${icon('zap')}</span>`;
        html += `<div class="row" role="option" id="row-${i}" data-i="${i}" aria-selected="${i === S.sel}" data-jev="${!!r.jev}" data-keymap="${!!r.keymap}"><span class="tile">${icon(r.icon)}</span><span class="copy"><span class="name">${esc(r.name)}</span>${r.desc ? `<span class="desc">${esc(r.desc)}</span>` : ''}</span>${accHtml}</div>`;
        i++;
      }
    }
    const list = $('#list');
    list.innerHTML = html;
    q.setAttribute('aria-activedescendant', `row-${S.sel}`);
    const selEl = list.querySelector(`[data-i="${S.sel}"]`);
    if (selEl) {
      const top = selEl.offsetTop - 28, bottom = selEl.offsetTop + selEl.offsetHeight + 52;
      if (top < list.scrollTop) list.scrollTop = top;
      else if (bottom > list.scrollTop + list.clientHeight) list.scrollTop = bottom - list.clientHeight;
    }

    // Capsules: status (or Today) on the left; ↵ · ⌘K · ⌘↵ on the right.
    const sel = rows[S.sel];
    const caps = [];
    if (classic || sm.kind === 'rest') {
      caps.push(`<span class="cap cap--status" data-cap="today"><span class="icon">${icon('calendar')}</span><span data-ink="label">Today</span><span class="keys" data-ink="key">${keys('⌘T', true)}</span></span>`);
    } else {
      const fill = sm.kind === 'hold' ? `<span class="fill" style="width:${(sm.progress * 100).toFixed(1)}%"></span>` : '';
      caps.push(`<span class="cap cap--status" data-cap="status" data-kind="${sm.kind}" data-failed="${!!sm.failed}" style="opacity:${Math.max(0.6, statusOpacity(sm))}">${fill}${statusBody(sm)}</span>`);
    }
    caps.push('<span class="spacer"></span>');
    caps.push(`<span class="cap" data-cap="enter"><span data-ink="label">${esc(sel ? (sel.keymap ? (sel.cycle ? 'Next' : 'Use') : sel.enter) : 'Ask')}</span><span class="keys" data-ink="key">${keys('↵')}</span></span>`);
    caps.push(`<span class="cap" data-cap="actions"><span data-ink="label">Actions</span><span class="keys" data-ink="key">${keys('⌘K')}</span></span>`);
    caps.push(`<span class="cap" data-cap="agent"><span data-ink="label">${S.text ? 'Do It' : 'Agent'}</span><span class="keys" data-ink="key">${keys('⌘↵')}</span></span>`);
    $('#foot').innerHTML = caps.join('');

    renderChrome();
    renderNotes(R, sm);
    countInk();
  }

  function visible(el) {
    if (!el.getClientRects().length) return false;
    for (let n = el; n && n !== win; n = n.parentElement) {
      const cs = getComputedStyle(n);
      if (cs.display === 'none' || cs.visibility === 'hidden' || parseFloat(cs.opacity) < 0.05) return false;
    }
    const r = el.getBoundingClientRect(), w = win.getBoundingClientRect();
    return r.bottom > w.top && r.top < w.bottom;
  }
  // Text and keycaps outside the rows, counting header extras (whispers, tabs) but not plain header names.
  function countInk() {
    const counted = (el) => !el.closest('.row') && !el.closest('.mirror') && (!el.closest('.hdr') || el.closest('.whispers, .tab, .acc'));
    const labels = [...win.querySelectorAll('[data-ink="label"]')].filter((el) => counted(el) && visible(el)).length;
    const keyCount = [...win.querySelectorAll('.kc')].filter((k) => counted(k) && visible(k)).length;
    const headers = [...win.querySelectorAll('.hdr')].filter(visible).length;
    const n = (count, one, many) => `<b>${count}</b> ${count === 1 ? one : many}`;
    $('#ink').innerHTML = `Outside the rows right now: ${n(labels, 'piece of text', 'pieces of text')}, ${n(keyCount, 'keycap', 'keycaps')} and ${n(headers, 'section header', 'section headers')}.`;
  }

  function renderChrome() {
    document.querySelectorAll('#designs button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.d === S.design)));
    document.querySelectorAll('#stories button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.story === S.story)));
    document.querySelectorAll('#rules button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.rule === S.rule)));
    document.querySelectorAll('#themes button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.theme === S.theme)));
    $('#rules').dataset.disabled = String(!!DESIGNS[S.design].classic);
    $('#clock-toggle').textContent = S.running ? 'Pause' : 'Run';
    const t = Math.max(0, Math.round(W.now));
    $('#clock-now').textContent = `${Math.floor(t / 60)}:${String(t % 60).padStart(2, '0')} into the story`;
    const sel = !!W.selection;
    document.querySelectorAll('#events button').forEach((b) => {
      const e = b.dataset.ev;
      b.disabled = (e === 'unselect' && !sel) || (e === 'copySel' && !sel) || (e === W.app);
    });
  }

  // Where every candidate ended up, and why.
  function verdicts(R, sm) {
    const d = DESIGNS[S.design];
    const t = thisItem(R);
    const out = [];
    if (W.hold) out.push(['A run waiting to go', W.hold.title, `Starts in ${Math.max(0, Math.ceil((HOLD_MS - W.hold.elapsed) / 1000))} s`, 'needs', d.classic ? 'Takes the line · ⌘Z stops it' : 'Status place · ⌘Z stops it']);
    if (W.failure) out.push(['A failure', W.failure.title, exact(W.now - W.failure.at), W.failure.seen ? 'key' : 'needs', W.failure.seen ? 'Seen; gone' : (d.classic ? 'Takes the line' : 'Status place until you’ve seen it')]);
    for (const it of R.items) {
      const what = it.id === 'selection' ? 'Selected text' : it.id === 'clip' ? clipLabel(W.clip) : `Window: ${it.label}`;
      const when = it.floor ? `Switched ${exact(W.now - it.at)}` : exact(W.now - it.at);
      let v, words;
      if (d.classic) {
        if (it.id === 'clip') { v = 'key'; words = 'Hidden until ⌘V'; }
        else if (it.id === 'selection') { v = 'leads'; words = 'The dashed chip · Space'; }
        else if (R.items.some((x) => x.id === 'selection')) { v = 'key'; words = 'Not shown while text is selected'; }
        else { v = 'leads'; words = lastRun() ? 'The dashed chip · Space' : 'The line and the chip · Space'; }
      } else if (it.dup) { v = 'once'; words = 'Same as the selection; counted once'; }
      else if (it.key === t.key) { v = 'leads'; words = S.pick ? 'This, after ⇥ · Space' : (S.rule === 'fixed' ? 'Leads by fixed order · Space' : it.floor ? 'Leads: nothing newer · Space' : 'Leads: newest · Space'); }
      else if (it.floor) { v = 'key'; words = 'The floor · ⇥ to switch to it'; }
      else if (S.rule === 'fixed' || it.state === 'news') { v = 'whisper'; words = `Fresh · ${it.id === 'clip' ? '⌘V' : '⇥'}`; }
      else if (it.state === 'faded') { v = 'faded'; words = 'Faded after 2 min · ⌘V still works'; }
      else if (it.state === 'ignored') { v = 'ignored'; words = 'You saw it and moved on · ⌘V still works'; }
      else if (it.state === 'used') { v = 'key'; words = 'You used it · ⌘V still works'; }
      else { v = 'key'; words = '—'; }
      out.push([what, it.title, when, v, words]);
    }
    const last = lastRun();
    if (last) {
      let v = 'key', words = 'On ↑';
      if (!d.classic && t.id === 'last') { v = 'leads'; words = 'This, picked from the tabs · Space'; }
      else if (d.classic) { v = sm.kind === 'recent' ? 'status' : 'key'; words = sm.kind === 'recent' ? 'Takes the line, at any age · ↑' : sm.kind === 'hold' ? 'On ↑, behind the waiting run' : 'On ↑'; }
      else if (S.design === 'C' && W.now - last.at <= FEED_S) { v = 'status'; words = 'A row in Just now'; }
      else if (sm.kind === 'recent') { v = 'status'; words = last.undo ? 'Status place · ⌘Z undoes it' : 'Status place · ↑'; }
      else if (W.hold && W.now - last.at <= PAST_S) { v = 'key'; words = 'On ↑, behind the waiting run'; }
      else if (S.design === 'D') { v = 'key'; words = 'The last tab · ↑'; }
      out.push(['Last run', last.title, exact(W.now - last.at), v, words]);
    }
    return out;
  }

  function renderNotes(R, sm) {
    const d = DESIGNS[S.design];
    const ruleWords = d.classic
      ? 'Today’s order: a waiting run, then a notice, then the last run at any age, then the window.'
      : S.rule === 'fixed'
        ? 'Fixed order: selected text, then anything copied (at any age), then the window.'
        : 'Newest wins: the latest thing you did on purpose leads; a copy fades after 2 minutes, and anything you saw and passed over steps back.';
    const table = verdicts(R, sm).map(([what, title, when, v, words]) => `<tr><td>${esc(what)}<span class="what" title="${esc(title)}">${esc(title)}</span><span class="what">${esc(when)}</span></td><td><span class="verdict" data-v="${v}">${esc(words)}</span></td></tr>`).join('');
    $('#notes').innerHTML = `
      <h2>${S.design} · ${esc(d.name)}</h2>
      <p class="sum">${esc(d.summary)}</p>
      <h3>How it picked</h3>
      <table class="why"><tr><th>Candidate</th><th>Where it is</th></tr>${table}</table>
      <p class="why-rule">${esc(ruleWords)}</p>
      <h3>Gives</h3><ul class="gives">${d.gives.map((c) => `<li>${esc(c)}</li>`).join('')}</ul>
      <h3>Gives up</h3><ul class="breaks">${d.costs.map((c) => `<li>${esc(c)}</li>`).join('')}</ul>`;
  }

  function log(text) {
    S.log.unshift(text);
    S.log = S.log.slice(0, 5);
    $('#log').innerHTML = S.log.map((t, i) => `<li style="opacity:${1 - i * 0.17}">${t}</li>`).join('');
  }

  // ── Time ──
  function advance(sec) {
    let left = sec;
    while (left > 0) {
      const step = Math.min(left, 0.25);
      W.now += step;
      left -= step;
      if (W.hold && S.text === '') {
        W.hold.elapsed += step * 1000;
        if (W.hold.elapsed >= HOLD_MS) {
          W.history.push({ title: W.hold.done, sentence: 'empty the trash', icon: W.hold.icon || 'trash-2', at: W.now, undo: false });
          log(`8 s passed → <span>${esc(W.hold.done)}</span>`);
          W.hold = null;
        }
      }
    }
  }
  let lastTick = performance.now();
  setInterval(() => {
    const t = performance.now();
    const dt = (t - lastTick) / 1000;
    lastTick = t;
    if (!S.running) return;
    const before = Math.floor(W.now);
    const holding = !!W.hold;
    advance(dt);
    if (holding || Math.floor(W.now) !== before) render();
  }, 200);

  // ── Stories and events ──
  function resetUi() { S.text = ''; S.sel = 0; S.staged = null; S.pick = 0; S.cmd = false; }
  function loadStory(id) {
    W = blankWorld();
    STORIES[id].build(W);
    S.story = id;
    resetUi();
    log(`Story → <span>${esc(STORIES[id].name)}</span>`);
    syncHash();
    render();
  }
  const EVENTS = [
    ['select', 'Select text'], ['unselect', 'Clear the selection'], '|',
    ['copyLink', 'Copy a link'], ['copySel', 'Copy the selection'], '|',
    ['Ghostty', 'Switch to Ghostty'], ['Chrome', 'Switch to Chrome'], ['Mail', 'Switch to Mail'], '|',
    ['run', 'Run something'], ['hold', 'Start a run that waits 8 s'], ['fail', 'A save fails'], '|',
    ['wait30', 'Wait 30 s'], ['wait2m', 'Wait 2 min'], ['wait1h', 'Wait 1 h'], '|',
    ['glance', 'Glance and close'],
  ];
  function happen(ev) {
    const now = W.now;
    let words = (EVENTS.find((e) => e[0] === ev) || [])[1] || ev;
    if (ev === 'select') W.selection = { text: APPS[W.app].selection, at: now };
    else if (ev === 'unselect') W.selection = null;
    else if (ev === 'copyLink') W.clip = { kind: 'link', text: LINK, at: now };
    else if (ev === 'copySel' && W.selection) W.clip = { kind: 'text', text: W.selection.text, at: now };
    else if (APPS[ev]) { W.app = ev; W.switchedAt = now; W.selection = null; }
    else if (ev === 'run') W.history.push({ ...REMINDER, at: now });
    else if (ev === 'hold') W.hold = { title: 'Empty the Trash', done: 'Emptied the Trash', icon: 'trash-2', elapsed: 0 };
    else if (ev === 'fail') W.failure = { title: 'Couldn’t save the note', at: now, seen: false };
    else if (ev === 'wait30') advance(30);
    else if (ev === 'wait2m') advance(120);
    else if (ev === 'wait1h') advance(3600);
    else if (ev === 'glance') {
      // You opened Script Kit and left without acting: what it showed you stops leading.
      const R = rank();
      const shown = [R.leader, ...R.whispers].filter((it) => it && !it.floor);
      for (const it of shown) W.ignored[it.key] = true;
      if (W.failure) W.failure.seen = true;
      words = shown.length ? `Glanced and closed → ${shown.map((s) => s.short.toLowerCase()).join(' and ')} stepped back` : 'Glanced and closed → nothing to step back';
    }
    resetUi();
    log(`<span>${esc(words)}</span>`);
    render();
  }

  // ── Keys ──
  function stage(st, how) {
    S.staged = st; S.text = ''; S.sel = 0;
    const item = staged();
    log(`${how} → acting on <span>${esc(item ? (item.id === 'last' ? item.title : item.short) : '')}</span>`);
    render();
  }
  function letGo() { S.staged = null; S.text = ''; S.sel = 0; render(); }
  function run(r, how) {
    if (!r) return;
    if (r.keymap) {
      if (r.cycle) { S.pick++; S.cmd = false; log(`${how} → next`); render(); return; }
      S.cmd = false; stage(r.stage, how); return;
    }
    if (r.action === 'undo') { undo(how); return; }
    if (r.action === 'edit') { S.staged = null; S.text = r.sentence; S.sel = 0; log(`${how} → the words are back in the input`); render(); q.setSelectionRange(S.text.length, S.text.length); return; }
    if (r.hold) {
      W.hold = { title: r.holdTitle || r.name, done: r.done, icon: r.icon, elapsed: 0 };
      resetUi(); log(`${how} → <span>${esc(W.hold.title)}</span> waits 8 s before it runs`); render(); return;
    }
    if (r.source) W.used[r.source] = true;
    const sentence = S.text.trim() || r.name.toLowerCase();
    W.history.push({ title: r.done || `Ran ${r.name}`, sentence, icon: r.icon, at: W.now, undo: !!r.undo });
    W.notice = null;
    resetUi();
    log(`${how} → <span>${esc(lastRun().title)}</span>`);
    render();
  }
  function undo(how) {
    const last = lastRun();
    if (W.hold) { W.notice = { text: `Didn’t run: ${W.hold.title}`, at: W.now }; W.hold = null; log(`${how} → stopped the waiting run`); }
    else if (last && last.undo && W.now - last.at <= PAST_S) { W.notice = { text: `Took back “${last.title}”`, at: W.now }; last.undo = false; log(`${how} → <span>undid it</span>`); }
    else return false;
    resetUi(); render();
    return true;
  }

  q.addEventListener('input', () => { S.text = q.value; S.sel = 0; render(); });
  q.addEventListener('scroll', () => { $('#mirror').scrollLeft = q.scrollLeft; });
  q.addEventListener('keydown', (e) => {
    const classic = !!DESIGNS[S.design].classic;
    if (e.key === 'Meta') {
      clearTimeout(cmdTimer);
      cmdTimer = setTimeout(() => { S.cmd = true; render(); }, REVEAL_AFTER_MS);
      return;
    }
    if (e.metaKey) clearTimeout(cmdTimer);
    const empty = S.text === '';
    if (e.key === 'ArrowDown') { e.preventDefault(); S.sel = Math.min(rows.length - 1, S.sel + 1); render(); return; }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (empty && S.sel === 0 && lastRun()) {
        if (!S.staged) { stage({ kind: 'last', idx: W.history.length - 1 }, '↑'); return; }
        if (S.staged.kind === 'last' && S.staged.idx > 0) { stage({ kind: 'last', idx: S.staged.idx - 1 }, '↑ again'); return; }
      }
      S.sel = Math.max(0, S.sel - 1); render(); return;
    }
    if (e.key === 'Tab' && empty && !S.staged && !classic) {
      e.preventDefault();
      S.pick += e.shiftKey ? -1 : 1; S.sel = 0;
      log(`${e.shiftKey ? '⇧⇥' : '⇥'} → this is now <span>${esc(thisItem().short)}</span>`);
      render(); return;
    }
    if (e.key === 'Enter' && e.metaKey) { e.preventDefault(); pressCap('agent'); if (S.text) run(rows.find((r) => r.jev) || rows[0], '⌘↵'); else log('⌘↵ → <span>Agent</span> opens the chat under this input'); return; }
    if (e.key === 'Enter') { e.preventDefault(); pressCap('enter'); run(rows[S.sel], '↵'); return; }
    if (e.key === 'Escape') {
      e.preventDefault();
      if (S.staged) { log('Esc → let go'); letGo(); return; }
      if (S.text) { S.text = ''; S.sel = 0; render(); }
      return;
    }
    if (e.key === ' ' && empty && !S.staged) {
      e.preventDefault();
      if (classic) {
        const focus = rank().items.find((it) => it.id === 'selection') || rank().items.find((it) => it.floor);
        stage({ kind: 'item', id: focus.id }, 'Space');
      } else {
        const t = thisItem();
        stage(t.id === 'last' ? { kind: 'last', idx: t.idx } : { kind: 'item', id: t.id }, 'Space');
      }
      return;
    }
    if (e.metaKey && e.key.toLowerCase() === 'v' && empty && !S.staged) {
      e.preventDefault();
      if (W.clip) stage({ kind: 'item', id: 'clip' }, '⌘V');
      return;
    }
    if (e.metaKey && e.key.toLowerCase() === 'k') {
      e.preventDefault(); pressCap('actions');
      const r = rows[S.sel];
      if (!r || S.staged || r.keymap) return;
      if (r.feedItem) { stage(r.feedItem.id === 'last' ? { kind: 'last', idx: r.feedItem.idx } : { kind: 'item', id: r.feedItem.id }, '⌘K'); return; }
      stage({ kind: 'row', row: r }, '⌘K'); return;
    }
    if (e.metaKey && e.key.toLowerCase() === 'z' && empty) { if (undo('⌘Z')) e.preventDefault(); }
  });
  q.addEventListener('keyup', (e) => { if (e.key === 'Meta') { clearTimeout(cmdTimer); if (S.cmd) { S.cmd = false; render(); } } });
  window.addEventListener('blur', () => { clearTimeout(cmdTimer); if (S.cmd) { S.cmd = false; render(); } });

  function pressCap(id) {
    const c = document.querySelector(`[data-cap="${id}"]`);
    if (!c) return;
    c.dataset.pressed = 'true';
    setTimeout(() => { c.dataset.pressed = 'false'; }, 140);
  }

  // Pointer: click selects, double-click runs; tabs, whispers and the icon do what their keys do.
  win.addEventListener('mousedown', (e) => { if (e.target !== q) { e.preventDefault(); q.focus(); } });
  $('#list').addEventListener('click', (e) => {
    const tab = e.target.closest('[data-tab]');
    if (tab) { S.pick = +tab.dataset.tab; S.sel = 0; log(`Clicked a tab → this is now <span>${esc(thisItem().short)}</span>`); render(); return; }
    const wh = e.target.closest('[data-whisper]');
    if (wh) { stage({ kind: 'item', id: wh.dataset.whisper }, 'Clicked'); return; }
    const r = e.target.closest('.row');
    if (r) { S.sel = +r.dataset.i; render(); }
  });
  $('#list').addEventListener('dblclick', (e) => { const r = e.target.closest('.row'); if (r) run(rows[+r.dataset.i], 'Double-click'); });
  $('#lead').addEventListener('click', (e) => {
    if (e.target.closest('#ftile') && !S.staged) { const t = thisItem(); stage(t.id === 'last' ? { kind: 'last', idx: t.idx } : { kind: 'item', id: t.id }, 'Clicked the icon'); }
  });
  $('#foot').addEventListener('click', (e) => {
    const c = e.target.closest('[data-cap]');
    if (!c) return;
    const id = c.dataset.cap;
    if (id === 'enter') run(rows[S.sel], '↵ capsule');
    else if (id === 'actions' && rows[S.sel] && !S.staged && !rows[S.sel].keymap) stage({ kind: 'row', row: rows[S.sel] }, '⌘K capsule');
    else if (id === 'agent') log('⌘↵ capsule → <span>Agent</span> opens the chat under this input');
    else if (id === 'today') log('Today → opens today’s page under this input');
    else if (id === 'status' && W.hold) undo('Stop');
    else if (id === 'status' && lastRun()) stage({ kind: 'last', idx: W.history.length - 1 }, 'Clicked the status');
  });

  // ── Controls ──
  $('#designs').innerHTML = Object.entries(DESIGNS).map(([k, v]) => `<button role="radio" data-d="${k}"><b>${k} · ${esc(v.name)}</b><span>${esc(v.blurb)}</span></button>`).join('');
  $('#stories').innerHTML = Object.entries(STORIES).map(([k, v]) => `<button role="radio" data-story="${k}">${esc(v.name)}</button>`).join('');
  $('#events').innerHTML = EVENTS.map((e) => (e === '|' ? '<span class="sep" aria-hidden="true"></span>' : `<button data-ev="${e[0]}">${esc(e[1])}</button>`)).join('');
  $('#designs').addEventListener('click', (e) => { const b = e.target.closest('button'); if (!b) return; S.design = b.dataset.d; S.pick = 0; S.cmd = false; syncHash(); render(); q.focus(); });
  $('#stories').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { loadStory(b.dataset.story); q.focus(); } });
  $('#events').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b && !b.disabled) { happen(b.dataset.ev); q.focus(); } });
  $('#rules').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.rule = b.dataset.rule; S.pick = 0; syncHash(); render(); q.focus(); } });
  $('#themes').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.theme = b.dataset.theme; syncHash(); render(); q.focus(); } });
  $('#clock').addEventListener('click', (e) => { if (e.target.closest('#clock-toggle')) { S.running = !S.running; lastTick = performance.now(); render(); q.focus(); } });

  function syncHash() { history.replaceState(null, '', `#${S.design}-${S.story}-${S.theme}-${S.rule}`); }
  const m = /^#([A-E])-(\w+)-(dark|light)-(newest|fixed)$/.exec(location.hash);
  if (m) { S.design = m[1]; if (STORIES[m[2]]) S.story = m[2]; S.theme = m[3]; S.rule = m[4]; }

  function fit() {
    const stage = $('#stage'), sc = $('#scaler');
    const k = Math.min(1, stage.clientWidth / 750);
    sc.style.transform = k < 1 ? `scale(${k})` : '';
    sc.style.height = `${480 * k}px`;
    sc.style.width = `${750 * k}px`;
  }
  window.addEventListener('resize', fit);

  // Agents and tests read and drive the same world and rule.
  window.mockup = {
    state: () => {
      const R = rank();
      return JSON.parse(JSON.stringify({
        design: S.design, story: S.story, rule: S.rule, theme: S.theme, text: S.text, sel: S.sel, pick: S.pick, cmd: S.cmd,
        staged: S.staged && { kind: S.staged.kind, id: S.staged.id, idx: S.staged.idx }, rows: rows.map((r) => r.name),
        leader: R.leader.id, this: thisItem(R).id, whispers: R.whispers.map((w) => w.id), status: statusModel(),
        world: { now: W.now, app: W.app, selection: !!W.selection, clip: W.clip && W.clip.kind, runs: W.history.length, hold: !!W.hold, failure: W.failure && !W.failure.seen },
      }));
    },
    set: (patch) => { Object.assign(S, patch); render(); },
    story: (id) => loadStory(id),
    happen: (ev) => happen(ev),
    advance: (sec) => { advance(sec); render(); },
    pause: (on = true) => { S.running = !on; lastTick = performance.now(); render(); },
    designs: Object.keys(DESIGNS), stories: Object.keys(STORIES), events: EVENTS.filter((e) => e !== '|').map((e) => e[0]),
  };

  fit();
  loadStory(S.story);
  q.focus();
})();

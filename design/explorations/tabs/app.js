// Switch the subject: one world (what you selected, copied, switched to and ran), one rule that
// picks where the window opens, and several ways of drawing the subjects as tabs. Every variation
// reads the same world and answers the same keys.
(() => {
  'use strict';
  const $ = (s) => document.querySelector(s);
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const icon = (n) => `<svg viewBox="0 0 24 24" aria-hidden="true">${(window.ICONS || {})[n] || ''}</svg>`;

  // Timing. HOLD_MS is the app's (crates/sk-one-window/src/contract.rs); the rest are this proposal's.
  const HOLD_MS = 8000;
  const NEWS_S = 120; // a copy is news for two minutes
  const PAST_S = 120; // the last run stays in the status place while ⌘Z can take it back
  const REVEAL_AFTER_MS = 350;
  const MARKS = { subject: '#ffb86b', action: '#f5c542', target: '#6fdc9b', details: '#8fb8ff', owner: '#c9a7ff' };

  // ── The world behind the window ──
  const act = (name, iconName, enter, done, extra = {}) => ({ name, icon: iconName, enter, done, ...extra });
  const APPS = {
    Ghostty: {
      app: 'Ghostty', title: 'zsh — ~/projects/site', icon: 'terminal',
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
      app: 'Chrome', title: 'Fix footer label · Pull Request #412', icon: 'globe',
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
      app: 'Mail', title: 'Re: launch checklist — Dana', icon: 'mail',
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
  // The sidebar's last tab: everything that isn't about a subject.
  const EVERYTHING = { id: 'all', key: 'all', icon: 'layers', label: 'Everything else', short: 'Everything else', title: 'Inbox, flows and suggestions', actions: [], standing: true, state: 'standing' };

  function blankWorld() {
    return {
      now: 0, app: 'Ghostty', switchedAt: -3600, selection: null, clip: null,
      history: OLDER_RUNS.map((r) => ({ ...r, undo: false })), hold: null, failure: null, notice: null,
      ignored: {}, used: {},
    };
  }
  const ran = (w, r, at) => w.history.push({ ...r, at });
  const STORIES = {
    all: { name: 'Everything at once', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -90; w.clip = { kind: 'link', text: LINK, at: -40 }; ran(w, REMINDER, -25); w.selection = { text: APPS.Ghostty.selection, at: -6 }; w.hold = { title: 'Empty the Trash', done: 'Emptied the Trash', icon: 'trash-2', elapsed: 2500 }; } },
    back: { name: 'Back in the terminal', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -600; w.clip = { kind: 'link', text: LINK, at: -2400 }; ran(w, { ...REMINDER, undo: false }, -3700); } },
    copied: { name: 'Just copied a link', build: (w) => { w.app = 'Chrome'; w.switchedAt = -300; w.clip = { kind: 'link', text: LINK, at: -4 }; ran(w, { ...REMINDER, undo: false }, -3600); } },
    selected: { name: 'Selected an error', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -120; w.clip = { kind: 'link', text: LINK, at: -50 }; w.selection = { text: APPS.Ghostty.selection, at: -3 }; ran(w, { ...REMINDER, undo: false }, -3600); } },
    ran: { name: 'Just ran something', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -600; ran(w, REMINDER, -15); } },
    ignored: { name: 'Saw the link, didn’t use it', build: (w) => { w.app = 'Chrome'; w.switchedAt = -300; w.clip = { kind: 'link', text: LINK, at: -70 }; w.ignored['clip@-70'] = true; ran(w, { ...REMINDER, undo: false }, -3600); } },
    same: { name: 'Selected and copied the same text', build: (w) => { w.app = 'Ghostty'; w.switchedAt = -100; w.selection = { text: APPS.Ghostty.selection, at: -8 }; w.clip = { kind: 'text', text: APPS.Ghostty.selection, at: -6 }; ran(w, { ...REMINDER, undo: false }, -3600); } },
    failed: { name: 'A save failed', build: (w) => { w.app = 'Mail'; w.switchedAt = -60; w.failure = { title: 'Couldn’t save the note', at: -10, seen: false }; ran(w, { ...REMINDER, undo: false }, -3600); } },
  };

  const VARIANTS = {
    D: {
      name: 'Last round', blurb: 'Pills in the header',
      summary: 'The tabs as they were: small pills in the first header, the selected one filled, with ⇥ at the far right.',
      gives: ['Every subject is visible at once, in a header the list already has.'],
      costs: ['The pills read like filters more than tabs.', 'The selected tab doesn’t say which error, link or window it means.', '⇥ sits at the far end, away from the tab it moves to.'],
    },
    T1: {
      name: 'Underlined tabs', blurb: 'The header, polished',
      summary: 'The first header is the tab strip. The selected tab is underlined in gold and says exactly what it is. ⇥ sits right after it, pointing at the tab it moves to. Gold marks what ↵ uses: the underlined subject and the highlighted row.',
      gives: ['No extra height: the tabs replace a header the list already has.', 'The selected tab names its error, link or window, and ⇥ shows where it goes.'],
      costs: ['A long name squeezes the other tabs.', 'Two gold marks at once: the tab’s underline and the row’s tile.'],
    },
    T2: {
      name: 'Tab bar', blurb: 'Browser tabs under the input',
      summary: 'Browser-style tabs in their own bar under the input, each with its age. The selected tab opens into a panel that holds its actions; your inbox, flows and suggestions continue below the panel.',
      gives: ['Reads as tabs at a glance, and the ages show why the window opened where it did.', 'The panel makes it clear which rows belong to the subject.'],
      costs: ['The 38 pt bar takes almost a row of height.', 'Four names and four ages at rest.'],
    },
    T3: {
      name: 'Queue', blurb: 'This in front, the rest wait',
      summary: 'The selected subject is the input’s icon and names the first header. The others wait as small icons at the end of the input row, in ⇥ order. ⇥ brings the next one to the front. The queue hides while you type.',
      gives: ['The quietest way to show every subject: icons only, in a row that’s already there.', 'The input’s icon is always the subject, so “this” always lives in the same place.'],
      costs: ['Waiting subjects are only icons until you press ⇥ or point at them.', 'The input row gets busier at its right end.'],
    },
    T4: {
      name: 'Sidebar', blurb: 'Subjects left, actions right',
      summary: 'Subjects run down the left like a mail app’s mailboxes, each with its detail and age. The right side is what you can do with the selected one. The last tab, Everything else, holds your inbox, flows and suggestions.',
      gives: ['Every subject shows its detail, so nothing is cryptic.', '⇥ moves down the sidebar while ↑ and ↓ move through actions, so the two never mix.'],
      costs: ['Actions get two-thirds of the width.', 'Your inbox and flows live behind a tab instead of in the list.'],
    },
    T5: {
      name: 'Accordion', blurb: 'Each subject is a row',
      summary: 'No new control. Each subject is a row, and the selected one is open with its actions indented under it. ⇥ closes it and opens the next; ↵ on a closed subject opens it.',
      gives: ['Uses only the rows and headers the window already has.', 'Subjects and their actions read top to bottom in one list.'],
      costs: ['Opening a subject moves the rows under it.', 'With four subjects, your inbox starts lower.'],
    },
  };

  // ── State ──
  let W = blankWorld();
  const S = { variant: 'T1', story: 'all', order: 'leader', theme: 'dark', text: '', sel: 0, staged: null, tabKey: null, lastIdx: null, cmd: false, running: true, log: [] };
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
  function short(sec) {
    sec = Math.max(0, Math.round(sec));
    if (sec < 60) return `${sec}s`;
    if (sec < 3600) return `${Math.floor(sec / 60)}m`;
    return `${Math.floor(sec / 3600)}h`;
  }
  function exact(sec) {
    sec = Math.max(0, Math.round(sec));
    if (sec < 60) return `${sec} s ago`;
    if (sec < 3600) return `${Math.floor(sec / 60)} min ago`;
    return `${Math.floor(sec / 3600)} h ago`;
  }
  const trim = (s, n) => (s.length > n ? s.slice(0, n - 1) + '…' : s);
  function keys(k, small) {
    const parts = k === 'esc' ? ['esc'] : k === 'Space' ? ['␣'] : k === '⇧⇥' ? ['⇧', '⇥'] : [...k];
    return parts.map((p) => `<span class="kc${small ? ' kc--sm' : ''}">${esc(p)}</span>`).join('');
  }
  const hint = (key, words) => `<span class="hint">${words ? `<span data-ink="label">${esc(words)}</span>` : ''}<span data-ink="key">${keys(key, true)}</span></span>`;

  // ── Subjects and the rule ──
  function present() {
    const front = APPS[W.app];
    const items = [];
    const sel = W.selection;
    const dup = !!(sel && W.clip && W.clip.kind === 'text' && W.clip.text === sel.text);
    if (sel) {
      items.push({ id: 'selection', key: `selection@${sel.at}`, at: dup ? Math.max(sel.at, W.clip.at) : sel.at, icon: 'text-cursor',
        label: dup ? 'Selected and copied' : 'Selected text', short: dup ? 'Selected and copied' : 'Selected text', title: `“${sel.text}”`, where: `in ${front.app}`, actions: front.selActions });
    }
    if (W.clip) {
      const link = W.clip.kind === 'link';
      items.push({ id: 'clip', key: `clip@${W.clip.at}`, at: W.clip.at, icon: link ? 'link' : 'clipboard',
        label: link ? 'Copied link' : 'Copied text', short: link ? 'Copied link' : 'Copied text', title: link ? W.clip.text : `“${W.clip.text}”`, where: `copied ${exact(W.now - W.clip.at)}`, actions: link ? LINK_ACTIONS : TEXT_ACTIONS, dup });
    }
    items.push({ id: 'window', key: `window@${W.app}`, at: W.switchedAt, icon: front.icon, label: front.app, short: front.app, title: front.title, where: `in ${front.app}`, actions: front.actions, floor: true });
    return items;
  }
  function newsOf(it) {
    if (it.floor) return 'floor';
    if (it.dup) return 'once';
    if (W.used[it.key]) return 'used';
    if (W.ignored[it.key]) return 'ignored';
    if (it.id === 'clip' && W.now - it.at > NEWS_S) return 'faded';
    return 'news';
  }
  // Newest wins: the latest thing you did on purpose; the window when nothing newer is news.
  function rank() {
    const items = present().map((it) => ({ ...it, state: newsOf(it) }));
    const live = items.filter((it) => !it.dup);
    const leader = live.filter((it) => it.state === 'news' || it.floor).sort((a, b) => b.at - a.at)[0];
    const rest = live.filter((it) => it !== leader).sort((a, b) => ((a.state === 'news') === (b.state === 'news') ? b.at - a.at : a.state === 'news' ? -1 : 1));
    return { items, order: [leader, ...rest], leader };
  }
  const lastRun = () => W.history[W.history.length - 1];
  function lastItem(idx = W.history.length - 1) {
    const e = W.history[idx];
    const latest = idx === W.history.length - 1;
    const live = latest && e.undo && W.now - e.at <= PAST_S;
    const back = W.history.length - 1 - idx;
    const actions = [
      ...(live ? [act('Undo it', 'undo-2', 'Undo', null, { action: 'undo' })] : []),
      act(`Change the words: “${trim(e.sentence, 30)}”`, 'pencil', 'Edit', null, { action: 'edit', sentence: e.sentence }),
      act('Run it again', 'repeat', 'Run Again', e.title, { undo: e.undo }),
      act('Save it as a script', 'save', 'Save', `Saved “${trim(e.title, 30)}” as a script`),
    ];
    return { id: 'last', key: 'last', at: e.at, icon: 'history', label: latest ? 'Last run' : `${back + 1} runs ago`, short: latest ? 'Last run' : `${back + 1} runs ago`,
      title: e.title, where: exact(W.now - e.at), actions, idx, state: 'past' };
  }
  // The tabs, in order. "Newest first" keeps only what's still news; "same place" keeps every subject in a fixed spot.
  function tabsOf(R = rank()) {
    let list;
    if (S.order === 'fixed') list = ['window', 'selection', 'clip'].map((id) => R.items.find((i) => i.id === id && !i.dup)).filter(Boolean);
    else list = R.order.filter((it) => it.key === R.leader.key || it.floor || it.state === 'news');
    if (W.history.length) list = [...list, lastItem(S.lastIdx ?? W.history.length - 1)];
    if (S.variant === 'T4') list = [...list, EVERYTHING];
    return list;
  }
  function current(R = rank()) {
    const t = tabsOf(R);
    return t.find((x) => x.key === S.tabKey) || t.find((x) => x.key === R.leader.key) || t[0];
  }
  // New since you looked: still news, and not what the window opened on (you saw that one).
  const isFresh = (it, cur, R = rank()) => it.state === 'news' && it.key !== cur.key && it.key !== R.leader.key;
  const isStale = (it) => ['faded', 'ignored', 'used'].includes(it.state);
  function ownKey(it, cur) {
    if (!S.cmd) return '';
    if (it.key === cur.key && it.id !== 'all') return 'Space';
    if (it.id === 'clip') return '⌘V';
    if (it.id === 'last') return '↑';
    return '';
  }
  const tkey = (it, cur) => { const k = ownKey(it, cur); return k ? `<span class="tkey reveal" data-ink="key">${keys(k, true)}</span>` : ''; };
  const ageOf = (it) => (it.standing ? '' : short(W.now - it.at));

  // What the status place says: needs-you first, then the last run while ⌘Z can take it back.
  function statusModel() {
    if (W.hold) {
      const paused = S.text !== '';
      const left = Math.max(0, Math.ceil((HOLD_MS - W.hold.elapsed) / 1000));
      return { kind: 'hold', icon: W.hold.icon || 'trash-2', state: paused ? 'Paused while you type' : `Runs in ${left}s`, title: W.hold.title, progress: W.hold.elapsed / HOLD_MS, hints: S.text === '' ? [['⌘Z', 'Stop']] : [] };
    }
    if (W.failure && !W.failure.seen) return { kind: 'failed', icon: 'circle-alert', failed: true, title: W.failure.title, sub: ago(W.now - W.failure.at) };
    const last = lastRun();
    if (W.notice && W.now - W.notice.at <= PAST_S && (!last || W.notice.at >= last.at)) return { kind: 'notice', icon: 'history', title: W.notice.text, sub: ago(W.now - W.notice.at) };
    if (last && W.now - last.at <= PAST_S) {
      const hints = [];
      if (S.text === '' && !S.staged && last.undo) hints.push(['⌘Z', 'Undo']);
      return { kind: 'recent', icon: 'history', title: last.title, sub: ago(W.now - last.at), hints };
    }
    return { kind: 'rest' };
  }
  const statusOpacity = (m) => (m.kind === 'hold' || m.failed ? 1 : S.text !== '' ? 0.5 : 0.85);
  function statusBody(m) {
    const state = m.state ? `<span class="state" data-ink="label">${esc(m.state)}</span>` : '';
    return `<span class="icon">${icon(m.icon)}</span>${state}<span class="title" data-ink="label">${esc(m.title)}</span>${m.sub ? `<span class="sub" data-ink="label">· ${esc(m.sub)}</span>` : ''}${(m.hints || []).map(([k, w]) => hint(k, w)).join('')}`;
  }

  // ── What a sentence is about ──
  function aboutOf(text, R) {
    const lower = text.toLowerCase();
    const find = (re) => { const m = re.exec(lower); return m ? [m.index, m.index + m[0].length] : null; };
    const cur = current(R);
    const tests = [
      [/\b(the link|the copied \w+|what i copied|the clipboard)\b/, () => R.items.find((it) => it.id === 'clip')],
      [/\b(the last one|the reminder|again)\b/, () => (lastRun() ? lastItem() : null)],
      [/\b(this window|the window|this terminal|the terminal|this page|the page|this email|the email)\b/, () => R.items.find((it) => it.floor)],
      [/\b(this|it|that)\b/, () => staged() || (cur.standing ? R.leader : cur)],
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
    if (st.kind === 'row') return { id: 'row', key: 'row', icon: st.row.icon, short: st.row.name, title: st.row.name, actions: [
      { ...st.row, name: st.row.hold ? 'Empty it' : 'Run it', holdTitle: st.row.name },
      act('Pin it', 'pin', 'Pin', `Pinned ${st.row.name}`, { undo: true }),
      act('Copy deep link', 'clipboard-list', 'Copy', 'Copied a deep link'),
    ] };
    return present().find((it) => it.id === st.id) || null;
  }
  const withSource = (item) => item.actions.map((a) => ({ ...a, source: item.key }));
  function typedSections(R, t) {
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
  // Sections of the list (for the sidebar, of its right-hand pane).
  function listModel(R) {
    const t = S.text.trim();
    const st = staged();
    const cur = current(R);
    if (st) {
      const lower = t.toLowerCase();
      return [{ title: 'Or pick one', rows: withSource(st).filter((r) => !lower || r.name.toLowerCase().includes(lower)) }];
    }
    if (t) return typedSections(R, t);
    switch (S.variant) {
      case 'D': return [{ kind: 'pills', item: cur, rows: withSource(cur) }, ...STANDING];
      case 'T1': return [{ kind: 'utabs', item: cur, rows: withSource(cur) }, ...STANDING];
      case 'T2': return [{ kind: 'panel', item: cur, rows: withSource(cur) }, ...STANDING];
      case 'T3': return [{ kind: 'front', item: cur, rows: withSource(cur) }, ...STANDING];
      case 'T4': return cur.standing ? STANDING : [{ kind: 'pane', item: cur, rows: withSource(cur) }];
      case 'T5': {
        const subjects = tabsOf(R).map((it) => (it.key === cur.key
          ? { kind: 'open', item: it, rows: withSource(it).map((r) => ({ ...r, indent: true })) }
          : { kind: 'closed', item: it, rows: [{ name: it.short, detail: it.title, icon: it.icon, enter: 'Open', selectTab: it.key, subject: it }] }));
        return [{ kind: 'nowhdr' }, ...subjects, ...STANDING];
      }
      default: return STANDING;
    }
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

  function headerHtml(s, R) {
    const cur = current(R);
    const tabs = tabsOf(R);
    if (s.kind === 'pills') {
      const html = tabs.map((c) => `<span class="tab" role="tab" data-tab="${esc(c.key)}" aria-selected="${c.key === cur.key}" data-stale="${isStale(c)}" title="${esc(c.title)}"><span class="icon">${icon(c.icon)}</span><span class="tname" data-ink="label">${esc(c.short)}</span>${tkey(c, cur)}</span>`).join('');
      return `<div class="hdr hdr--tabs" data-style="quiet" role="tablist">${html}<span class="acc"><span data-ink="key">${keys('⇥', true)}</span></span></div>`;
    }
    if (s.kind === 'utabs') {
      const ci = tabs.findIndex((c) => c.key === cur.key);
      const html = tabs.map((c, i) => {
        const sel = c.key === cur.key;
        const detail = sel && !c.standing ? `<span class="udetail" data-ink="label">${esc(c.id === 'window' ? c.title : trim(c.title, 40))}</span>` : '';
        const tab = `<span class="utab" role="tab" data-tab="${esc(c.key)}" aria-selected="${sel}" data-stale="${isStale(c)}" title="${esc(c.title)}"><span class="icon">${icon(c.icon)}</span><span class="tname" data-ink="label">${esc(c.short)}</span>${detail}${isFresh(c, cur) ? '<span class="fresh"></span>' : ''}${tkey(c, cur)}</span>`;
        const next = i === ci && tabs.length > 1 && !S.cmd ? `<span class="tabnext" data-ink="key">${keys('⇥', true)}</span>` : '';
        return tab + next;
      }).join('');
      return `<div class="hdr hdr--utabs" role="tablist">${html}</div>`;
    }
    if (s.kind === 'panel' || s.kind === 'pane') {
      const it = s.item;
      return `<div class="hdr" data-style="quiet"><span class="label" data-ink="label">${esc(trim(it.title, 56))}</span><span class="hsub" data-ink="label">${esc(it.where || '')}</span></div>`;
    }
    if (s.kind === 'front') {
      const it = s.item;
      return `<div class="hdr" data-style="quiet"><span class="hicon">${icon(it.icon)}</span><span class="label" data-ink="label">${esc(it.label)}</span><span class="hsub" data-ink="label">${esc(trim(it.title, 48))}</span></div>`;
    }
    if (s.kind === 'nowhdr') return `<div class="hdr" data-style="quiet"><span class="label" data-ink="label">Now</span><span class="acc"><span data-ink="label">Next</span><span data-ink="key">${keys('⇥', true)}</span></span></div>`;
    if (s.kind === 'open' || s.kind === 'closed') return '';
    let acc = '';
    if (s.jev && s.about) acc = `<span class="whisper"><span class="icon">${icon(s.about.icon)}</span><span data-ink="label">About ${esc(s.about.short.toLowerCase())}</span></span>`;
    return `<div class="hdr" data-style="quiet"><span class="label" data-ink="label">${esc(s.title)}</span>${acc ? `<span class="acc">${acc}</span>` : ''}</div>`;
  }

  function rowHtml(r, i, s, R) {
    const cur = current(R);
    if (s.kind === 'closed') {
      const it = s.item;
      return `<div class="row row--subject" role="option" id="row-${i}" data-i="${i}" aria-selected="${i === S.sel}" data-stale="${isStale(it)}"><span class="tile">${icon(it.icon)}</span><span class="copy"><span class="name">${esc(it.short)}<span class="ndetail">${esc(trim(it.title, 48))}</span></span></span>${isFresh(it, cur) ? '<span class="fresh"></span>' : ''}${tkey(it, cur)}<span class="rage">${esc(ageOf(it))}</span><span class="chev">${icon('chevron-right')}</span></div>`;
    }
    let accHtml = '';
    if (r.jev) accHtml = `<span class="acc-icon">${icon('sparkles')}</span>`;
    else if (r.fallback) accHtml = `<span class="acc-icon">${icon('zap')}</span>`;
    return `<div class="row${r.indent ? ' row--action' : ''}" role="option" id="row-${i}" data-i="${i}" aria-selected="${i === S.sel}" data-jev="${!!r.jev}"><span class="tile">${icon(r.icon)}</span><span class="copy"><span class="name">${esc(r.name)}</span>${r.desc ? `<span class="desc">${esc(r.desc)}</span>` : ''}</span>${accHtml}</div>`;
  }
  // The open subject in the accordion: a row that names it, not one you can select.
  function openRowHtml(it, R) {
    const cur = current(R);
    return `<div class="row row--open" aria-hidden="false"><span class="tile">${icon(it.icon)}</span><span class="copy"><span class="name">${esc(it.short)}<span class="ndetail">${esc(trim(it.title, 48))}</span></span></span>${tkey(it, cur)}<span class="rage">${esc(ageOf(it))}</span><span class="chev">${icon('chevron-down')}</span></div>`;
  }

  function sectionsHtml(sections, R, start = 0) {
    let i = start, html = '';
    for (const s of sections) {
      if (s.kind === 'open') html += openRowHtml(s.item, R);
      let body = headerHtml(s, R);
      for (const r of s.rows || []) { rows.push(r); body += rowHtml(r, i, s, R); i++; }
      html += s.kind === 'panel' ? `<div class="panel">${body}</div>` : body;
    }
    return html;
  }

  function sideHtml(R) {
    const cur = current(R);
    const items = tabsOf(R).map((c) => `<div class="stab" role="tab" data-tab="${esc(c.key)}" aria-selected="${c.key === cur.key}" data-stale="${isStale(c)}"><span class="sicon">${icon(c.icon)}</span><span class="sname"><span class="tname" data-ink="label">${esc(c.short)}</span>${isFresh(c, cur) ? '<span class="fresh"></span>' : ''}${tkey(c, cur)}</span><span class="sage">${esc(ageOf(c))}</span><span class="sdetail" data-ink="label">${esc(c.title)}</span></div>`).join('');
    return `<div class="side" role="tablist" aria-orientation="vertical">${items}<div class="sidekey"><span data-ink="key">${keys('⇥', true)}</span><span data-ink="label">Next</span></div></div>`;
  }

  function render() {
    const R = rank();
    const cur = current(R);
    win.dataset.theme = S.theme;
    win.dataset.layout = 'row';
    win.dataset.row = 'aligned';
    win.dataset.variant = S.variant;
    win.style.setProperty('--in-size', '18px');
    const st = staged();
    q.placeholder = st ? 'Say what to do with it' : 'Search or ask';
    if (q.value !== S.text) q.value = S.text;
    const sm = statusModel();
    $('#line').innerHTML = '';

    // The input row: "this" as the leading tile, or the subject put in the input.
    let lead = '', trail = '';
    if (st) {
      lead = `<span class="chip chip--attached"><span class="icon">${icon(st.icon)}</span><span class="name" data-ink="label">${esc(trim(st.id === 'last' ? st.title : st.short, 34))}</span>${hint('esc', '')}</span>`;
    } else {
      const asKey = S.cmd && S.text === '' && !cur.standing;
      lead = `<span class="ftile${asKey ? ' ftile--key reveal' : ''}" id="ftile" data-kind="${cur.id}" role="button" tabindex="-1" title="${esc(cur.short)} · ${esc(cur.title)}" aria-label="Act on ${esc(cur.short)}: ${esc(cur.title)}" data-typing="${S.text !== ''}">${asKey ? '<span data-ink="key" class="tilekey">␣</span>' : icon(cur.icon)}</span>`;
    }
    if (S.variant === 'T3' && !st) {
      const tabs = tabsOf(R);
      const ci = tabs.findIndex((c) => c.key === cur.key);
      const waiting = tabs.slice(ci + 1).concat(tabs.slice(0, ci));
      if (waiting.length) {
        const tiles = waiting.map((c) => `<span class="qtile" role="tab" data-tab="${esc(c.key)}" aria-selected="false" data-stale="${isStale(c)}" title="${esc(c.short)} · ${esc(c.title)}">${icon(c.icon)}${isFresh(c, cur) ? '<span class="fresh"></span>' : ''}</span>`).join('');
        trail = `<span class="queue" data-typing="${S.text !== ''}">${S.cmd ? `<span class="qname reveal" data-ink="label">${esc(waiting[0].short)}</span>` : ''}<span class="qkey" data-ink="key">${keys('⇥', true)}</span>${tiles}</span>`;
      }
    }
    $('#lead').innerHTML = lead;
    $('#trail').innerHTML = trail;
    $('#inprogress').style.width = sm.kind === 'hold' ? `${(sm.progress * 100).toFixed(1)}%` : '0';
    const about = S.text.trim() ? aboutOf(S.text, R) : null;
    $('#mirror').innerHTML = marksFor(S.text, about);
    $('#mirror').scrollLeft = q.scrollLeft;

    // The tab bar (T2): outside the list, so it stays while you type.
    const bar = $('#tabbar');
    if (S.variant === 'T2') {
      bar.innerHTML = '<span class="baredge"></span>' + tabsOf(R).map((c) => `<span class="btab" role="tab" data-tab="${esc(c.key)}" aria-selected="${c.key === cur.key}" data-stale="${isStale(c)}" title="${esc(c.title)}"><span class="icon">${icon(c.icon)}</span><span class="tname" data-ink="label">${esc(c.short)}</span>${isFresh(c, cur) ? '<span class="fresh"></span>' : ''}${tkey(c, cur)}<span class="tage" data-ink="label">${esc(ageOf(c))}</span></span>`).join('')
        + `<span class="barfill"><span class="barkey" data-ink="key">${keys('⇥', true)}</span></span>`;
      bar.style.opacity = S.text || st ? 0.55 : 1;
    } else bar.innerHTML = '';

    // The list.
    rows = [];
    const sections = listModel(R);
    let html = '';
    if (S.variant === 'T4') html = `<div class="split">${sideHtml(R)}<div class="pane">${sectionsHtml(sections, R)}</div></div>`;
    else html = sectionsHtml(sections, R);
    if (S.sel === null) S.sel = Math.max(0, rows.findIndex((r) => r.indent));
    S.sel = Math.max(0, Math.min(S.sel, rows.length - 1));
    const list = $('#list');
    list.innerHTML = html;
    list.querySelectorAll('.row[data-i]').forEach((el) => el.setAttribute('aria-selected', String(+el.dataset.i === S.sel)));
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
    if (sm.kind === 'rest') {
      caps.push(`<span class="cap cap--status" data-cap="today"><span class="icon">${icon('calendar')}</span><span data-ink="label">Today</span><span class="keys" data-ink="key">${keys('⌘T', true)}</span></span>`);
    } else {
      const fill = sm.kind === 'hold' ? `<span class="fill" style="width:${(sm.progress * 100).toFixed(1)}%"></span>` : '';
      caps.push(`<span class="cap cap--status" data-cap="status" data-kind="${sm.kind}" data-failed="${!!sm.failed}" style="opacity:${Math.max(0.6, statusOpacity(sm))}">${fill}${statusBody(sm)}</span>`);
    }
    caps.push('<span class="spacer"></span>');
    caps.push(`<span class="cap" data-cap="enter"><span data-ink="label">${esc(sel ? sel.enter : 'Ask')}</span><span class="keys" data-ink="key">${keys('↵')}</span></span>`);
    caps.push(`<span class="cap" data-cap="actions"><span data-ink="label">Actions</span><span class="keys" data-ink="key">${keys('⌘K')}</span></span>`);
    caps.push(`<span class="cap" data-cap="agent"><span data-ink="label">${S.text ? 'Do It' : 'Agent'}</span><span class="keys" data-ink="key">${keys('⌘↵')}</span></span>`);
    $('#foot').innerHTML = caps.join('');

    renderChrome();
    renderNotes(R);
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
  // Text and keycaps outside the rows. Tabs count; plain section names don't.
  function countInk() {
    const tabby = '.utab, .tab, .btab, .stab, .tabnext, .queue, .whispers, .acc, .sidekey, .tabbar';
    const counted = (el) => !el.closest('.row') && !el.closest('.mirror') && (!el.closest('.hdr') || el.closest(tabby));
    const labels = [...win.querySelectorAll('[data-ink="label"]')].filter((el) => counted(el) && visible(el)).length;
    const keyCount = [...win.querySelectorAll('.kc')].filter((k) => counted(k) && visible(k)).length;
    const headers = [...win.querySelectorAll('.hdr')].filter(visible).length;
    const n = (count, one, many) => `<b>${count}</b> ${count === 1 ? one : many}`;
    $('#ink').innerHTML = `Outside the rows right now: ${n(labels, 'piece of text', 'pieces of text')}, ${n(keyCount, 'keycap', 'keycaps')} and ${n(headers, 'section header', 'section headers')}.`;
  }

  function renderChrome() {
    document.querySelectorAll('#variants button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.v === S.variant)));
    document.querySelectorAll('#stories button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.story === S.story)));
    document.querySelectorAll('#orders button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.order === S.order)));
    document.querySelectorAll('#themes button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.theme === S.theme)));
    $('#clock-toggle').textContent = S.running ? 'Pause' : 'Run';
    const t = Math.max(0, Math.round(W.now));
    $('#clock-now').textContent = `${Math.floor(t / 60)}:${String(t % 60).padStart(2, '0')} into the story`;
    document.querySelectorAll('#events button').forEach((b) => {
      const e = b.dataset.ev;
      b.disabled = ((e === 'unselect' || e === 'copySel') && !W.selection) || e === W.app;
    });
  }

  // Every tab, where it is, and how to reach it from the selected one.
  function renderNotes(R) {
    const v = VARIANTS[S.variant];
    const tabs = tabsOf(R);
    const cur = current(R);
    const ci = tabs.findIndex((c) => c.key === cur.key);
    const rowsHtml = tabs.map((c, i) => {
      const d = (i - ci + tabs.length) % tabs.length;
      let state, verdict;
      if (c.key === cur.key) { state = 'leads'; verdict = c.key === R.leader.key && !S.tabKey ? 'Selected: opened here' : 'Selected'; }
      else if (isFresh(c, cur)) { state = 'whisper'; verdict = 'New since you looked'; }
      else if (isStale(c)) { state = 'faded'; verdict = c.state === 'ignored' ? 'Dim: you saw it and moved on' : c.state === 'used' ? 'Dim: you used it' : 'Dim: older than 2 min'; }
      else if (c.id === 'last') { state = 'status'; verdict = 'What you did'; }
      else { state = 'key'; verdict = c.standing ? 'Your standing list' : 'Where you are'; }
      const ways = d === 0 ? [] : [d === tabs.length - 1 && tabs.length > 2 ? '⇧⇥' : `⇥${d > 1 ? ' ×' + d : ''}`];
      if (d !== 0 && c.id === 'clip') ways.push('⌘V');
      if (d !== 0 && c.id === 'last') ways.push('↑');
      const reach = ways.length ? ` · ${ways.join(' or ')}` : '';
      return `<tr><td>${esc(c.short)}<span class="what" title="${esc(c.title)}">${esc(c.title)}</span>${c.standing ? '' : `<span class="what">${esc(exact(W.now - c.at))}</span>`}</td><td><span class="verdict" data-v="${state}">${esc(verdict + reach)}</span></td></tr>`;
    }).join('');
    const orderWords = S.order === 'fixed'
      ? 'Same place every time: the window, then selected text, then what you copied (dim once it’s old), then the last run.'
      : 'Newest first: the subject the window opens on comes first; a copy drops off after 2 minutes or once you’ve passed over it, and ⌘V still reaches it.';
    $('#notes').innerHTML = `
      <h2>${S.variant} · ${esc(v.name)}</h2>
      <p class="sum">${esc(v.summary)}</p>
      <h3>The tabs right now</h3>
      <table class="why"><tr><th>Tab</th><th>State · how to reach it</th></tr>${rowsHtml}</table>
      <p class="why-rule">${esc(orderWords)}</p>
      <h3>Gives</h3><ul class="gives">${v.gives.map((c) => `<li>${esc(c)}</li>`).join('')}</ul>
      <h3>Gives up</h3><ul class="breaks">${v.costs.map((c) => `<li>${esc(c)}</li>`).join('')}</ul>`;
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
  function resetUi() { S.text = ''; S.sel = 0; S.staged = null; S.tabKey = null; S.lastIdx = null; S.cmd = false; }
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
      // You opened Script Kit and left without acting: what was new is no longer new.
      const shown = tabsOf(rank()).filter((it) => it.state === 'news');
      for (const it of shown) W.ignored[it.key] = true;
      if (W.failure) W.failure.seen = true;
      words = shown.length ? `Glanced and closed → ${shown.map((s) => s.short.toLowerCase()).join(' and ')} no longer new` : 'Glanced and closed → nothing was new';
    }
    resetUi();
    log(`<span>${esc(words)}</span>`);
    render();
  }

  // ── Keys ──
  function selectTab(key, how) {
    // null: the subject's first action (in the accordion it sits under the open subject).
    S.tabKey = key; S.sel = null;
    if (key !== 'last') S.lastIdx = null;
    log(`${how} → <span>${esc(current().short)}</span>`);
    render();
  }
  function moveTab(dir, how) {
    const R = rank();
    const tabs = tabsOf(R);
    const ci = tabs.findIndex((c) => c.key === current(R).key);
    const next = tabs[(ci + dir + tabs.length) % tabs.length];
    selectTab(next.key, how);
  }
  function stage(st, how) {
    S.staged = st; S.text = ''; S.sel = 0;
    const item = staged();
    log(`${how} → <span>${esc(item ? (item.id === 'last' ? item.title : item.short) : '')}</span> is in the input`);
    render();
  }
  function stageCurrent(how) {
    const cur = current();
    if (cur.standing) return;
    stage(cur.id === 'last' ? { kind: 'last', idx: cur.idx } : { kind: 'item', id: cur.id }, how);
  }
  function letGo() { S.staged = null; S.text = ''; S.sel = 0; render(); }
  function run(r, how) {
    if (!r) return;
    if (r.selectTab) { selectTab(r.selectTab, how); return; }
    if (r.action === 'undo') { undo(how); return; }
    if (r.action === 'edit') { S.staged = null; S.text = r.sentence; S.sel = 0; log(`${how} → the words are back in the input`); render(); q.setSelectionRange(S.text.length, S.text.length); return; }
    if (r.hold) {
      W.hold = { title: r.holdTitle || r.name, done: r.done, icon: r.icon, elapsed: 0 };
      resetUi(); log(`${how} → <span>${esc(W.hold.title)}</span> waits 8 s before it runs`); render(); return;
    }
    if (r.source) W.used[r.source] = true;
    W.history.push({ title: r.done || `Ran ${r.name}`, sentence: S.text.trim() || r.name.toLowerCase(), icon: r.icon, at: W.now, undo: !!r.undo });
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
    if (e.key === 'Meta') {
      clearTimeout(cmdTimer);
      cmdTimer = setTimeout(() => { S.cmd = true; render(); }, REVEAL_AFTER_MS);
      return;
    }
    if (e.metaKey) clearTimeout(cmdTimer);
    const empty = S.text === '';
    if (e.key === 'Tab') {
      e.preventDefault();
      if (empty && !S.staged) moveTab(e.shiftKey ? -1 : 1, e.shiftKey ? '⇧⇥' : '⇥');
      else if (!empty) log('⇥ with words typed → hands the sentence to the agent (its job today)');
      return;
    }
    if (e.key === 'ArrowDown') { e.preventDefault(); S.sel = Math.min(rows.length - 1, S.sel + 1); render(); return; }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (empty && !S.staged && S.sel === 0 && lastRun()) {
        const cur = current();
        if (cur.id !== 'last') { S.lastIdx = null; selectTab('last', '↑'); return; }
        if (cur.idx > 0) { S.lastIdx = cur.idx - 1; selectTab('last', '↑ again'); return; }
      }
      S.sel = Math.max(0, S.sel - 1); render(); return;
    }
    if (e.key === 'Enter' && e.metaKey) { e.preventDefault(); pressCap('agent'); if (S.text) run(rows.find((r) => r.jev) || rows[0], '⌘↵'); else log('⌘↵ → <span>Agent</span> opens the chat under this input'); return; }
    if (e.key === 'Enter') { e.preventDefault(); pressCap('enter'); run(rows[S.sel], '↵'); return; }
    if (e.key === 'Escape') {
      e.preventDefault();
      if (S.staged) { log('Esc → let go'); letGo(); return; }
      if (S.text) { S.text = ''; S.sel = 0; render(); return; }
      if (S.tabKey !== null || S.lastIdx !== null) { S.tabKey = null; S.lastIdx = null; S.sel = 0; log(`Esc → back to <span>${esc(current().short)}</span>`); render(); }
      return;
    }
    if (e.key === ' ' && empty && !S.staged) { e.preventDefault(); stageCurrent('Space'); return; }
    if (e.metaKey && e.key.toLowerCase() === 'v' && empty && !S.staged) {
      e.preventDefault();
      const clip = tabsOf().find((c) => c.id === 'clip');
      if (clip) selectTab(clip.key, '⌘V');
      else if (W.clip) stage({ kind: 'item', id: 'clip' }, '⌘V');
      return;
    }
    if (e.metaKey && e.key.toLowerCase() === 'k') {
      e.preventDefault(); pressCap('actions');
      const r = rows[S.sel];
      if (!r || S.staged) return;
      if (r.subject) { const it = r.subject; stage(it.id === 'last' ? { kind: 'last', idx: it.idx } : { kind: 'item', id: it.id }, '⌘K'); return; }
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

  // Pointer: tabs select, rows select on click and run on double-click; the icon does what Space does.
  win.addEventListener('mousedown', (e) => { if (e.target !== q) { e.preventDefault(); q.focus(); } });
  win.addEventListener('click', (e) => {
    const tab = e.target.closest('[data-tab]');
    if (tab) { if (tab.dataset.tab === 'last') S.lastIdx = null; selectTab(tab.dataset.tab, 'Clicked'); return; }
    const r = e.target.closest('.row[data-i]');
    if (r) { S.sel = +r.dataset.i; render(); return; }
    if (e.target.closest('#ftile') && !S.staged) { stageCurrent('Clicked the icon'); return; }
    const c = e.target.closest('[data-cap]');
    if (!c) return;
    const id = c.dataset.cap;
    if (id === 'enter') run(rows[S.sel], '↵ capsule');
    else if (id === 'actions' && rows[S.sel] && !S.staged) stage({ kind: 'row', row: rows[S.sel] }, '⌘K capsule');
    else if (id === 'agent') log('⌘↵ capsule → <span>Agent</span> opens the chat under this input');
    else if (id === 'today') log('Today → opens today’s page under this input');
    else if (id === 'status' && W.hold) undo('Stop');
    else if (id === 'status' && lastRun()) { S.lastIdx = null; selectTab('last', 'Clicked the status'); }
  });
  $('#list').addEventListener('dblclick', (e) => { const r = e.target.closest('.row[data-i]'); if (r) run(rows[+r.dataset.i], 'Double-click'); });

  // ── Controls ──
  $('#variants').innerHTML = Object.entries(VARIANTS).map(([k, v]) => `<button role="radio" data-v="${k}"><b>${k} · ${esc(v.name)}</b><span>${esc(v.blurb)}</span></button>`).join('');
  $('#stories').innerHTML = Object.entries(STORIES).map(([k, v]) => `<button role="radio" data-story="${k}">${esc(v.name)}</button>`).join('');
  $('#events').innerHTML = EVENTS.map((e) => (e === '|' ? '<span class="sep" aria-hidden="true"></span>' : `<button data-ev="${e[0]}">${esc(e[1])}</button>`)).join('');
  $('#variants').addEventListener('click', (e) => { const b = e.target.closest('button'); if (!b) return; S.variant = b.dataset.v; if (S.tabKey === 'all') S.tabKey = null; S.sel = 0; S.cmd = false; syncHash(); render(); q.focus(); });
  $('#stories').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { loadStory(b.dataset.story); q.focus(); } });
  $('#events').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b && !b.disabled) { happen(b.dataset.ev); q.focus(); } });
  $('#orders').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.order = b.dataset.order; S.sel = 0; syncHash(); render(); q.focus(); } });
  $('#themes').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.theme = b.dataset.theme; syncHash(); render(); q.focus(); } });
  $('#clock').addEventListener('click', (e) => { if (e.target.closest('#clock-toggle')) { S.running = !S.running; lastTick = performance.now(); render(); q.focus(); } });

  function syncHash() { history.replaceState(null, '', `#${S.variant}-${S.story}-${S.theme}-${S.order}`); }
  const m = /^#(D|T[1-5])-(\w+)-(dark|light)-(leader|fixed)$/.exec(location.hash);
  if (m) { S.variant = m[1]; if (STORIES[m[2]]) S.story = m[2]; S.theme = m[3]; S.order = m[4]; }

  function fit() {
    const stage = $('#stage'), sc = $('#scaler');
    const k = Math.min(1, stage.clientWidth / 750);
    sc.style.transform = k < 1 ? `scale(${k})` : '';
    sc.style.height = `${480 * k}px`;
    sc.style.width = `${750 * k}px`;
  }
  window.addEventListener('resize', fit);

  // Agents and tests read and drive the same world.
  window.mockup = {
    state: () => {
      const R = rank();
      const cur = current(R);
      return JSON.parse(JSON.stringify({
        variant: S.variant, story: S.story, order: S.order, theme: S.theme, text: S.text, sel: S.sel, cmd: S.cmd,
        staged: S.staged && { kind: S.staged.kind, id: S.staged.id, idx: S.staged.idx },
        tabs: tabsOf(R).map((t) => t.short), current: cur.short, leader: R.leader.short, rows: rows.map((r) => r.name), status: statusModel().kind,
        world: { now: W.now, app: W.app, selection: !!W.selection, clip: W.clip && W.clip.kind, runs: W.history.length, hold: !!W.hold },
      }));
    },
    set: (patch) => { Object.assign(S, patch); render(); },
    story: (id) => loadStory(id),
    happen: (ev) => happen(ev),
    advance: (sec) => { advance(sec); render(); },
    pause: (on = true) => { S.running = !on; lastTick = performance.now(); render(); },
    variants: Object.keys(VARIANTS), stories: Object.keys(STORIES), events: EVENTS.filter((e) => e !== '|').map((e) => e[0]),
  };

  fit();
  loadStory(S.story);
  q.focus();
})();

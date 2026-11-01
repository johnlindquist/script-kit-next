// One model, several ways of drawing the top of the window. Every preset shares the same rows,
// keys and timing, so switching never resets what you typed or what ran.
(() => {
  'use strict';
  const $ = (s) => document.querySelector(s);
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const icon = (n) => `<svg viewBox="0 0 24 24" aria-hidden="true">${(window.ICONS || {})[n] || ''}</svg>`;

  // Timing from crates/sk-one-window/src/contract.rs, plus the learning proposal's thresholds.
  const HOLD_MS = 8000;
  const LEARNING_AT = 3;
  const FLUENT_AT = 8;
  const REVEAL_AFTER_MS = 350;
  const MARKS = { subject: '#ffb86b', action: '#f5c542', target: '#6fdc9b', details: '#8fb8ff', owner: '#c9a7ff' };

  // How each choice is described in the notes panel.
  const AXES = {
    this: {
      icon: { home: 'The app’s icon at the start of the input. Click it or press Space to attach the window as a named chip. Holding ⌘ turns the icon into its ␣ key.', cost: 'Nothing on screen says what the icon is for until you click it or hold ⌘.' },
      rows: { home: 'The first section of the empty list is named for the window and offers three things to do with it. The same icon starts the input, so the two connect.', cost: 'The empty launcher always opens on the window’s first action, so ↵ there acts on that window.' },
      suggest: { home: 'The empty input holds one sentence about the window, with faint word marks. Tab takes it; typing replaces it.', cost: 'A suggestion must come from the same reading as the rows, or it teaches the wrong thing.' },
    },
    status: {
      inrow: { home: 'The right end of the input row. It shrinks to an icon when your sentence reaches it, and rests on today’s date.', cost: 'Long sentences and status share one row.' },
      header: { home: 'The right end of the list’s first header, which every list already has. Nothing is shown at rest. A waiting run also draws a gold rule under the input.', cost: 'Status sits a glance below the input. Views with no list, such as today’s page and the chat, need another place for it.' },
      foot: { home: 'The bottom-left capsule. It rests as Today, then shows what just ran, a waiting run with its progress, or what went wrong.', cost: 'What just ran appears at the bottom edge, away from where you typed. In return, it’s in the same place on every view, including today’s page and the chat.' },
    },
    row: {
      aligned: { home: 'The input row is 44 pt like a list row. Its icon sits in the rows’ icon column, and its words start where row names start.' },
      compact: { home: 'A 34 pt input row, as in last round’s D.' },
    },
  };

  const PRESETS = {
    A: { name: 'Today', blurb: 'What the app draws now', classic: true, size: 20, learn: false,
      summary: 'The current layout, rebuilt with the same measurements. The line above the input names the window behind Script Kit, and the dashed chip names it again.' },
    D: { name: 'One row', blurb: 'Last round, as it was', this: 'icon', status: 'inrow', row: 'compact', size: 18, learn: true,
      summary: 'Last round’s option D, with “Space acts on zsh” removed from the placeholder. The front window is the input’s icon; status sits at the far right.' },
    E: { name: 'Quiet input', blurb: 'Status drops to the header', this: 'icon', status: 'header', row: 'aligned', size: 18, learn: true,
      summary: 'The top row holds only the window’s icon and your words. Status moves to the right of the list’s first header, so a long sentence never competes with it. The input lines up with the list as if it were row zero.' },
    F: { name: 'Show, don’t tell', blurb: 'The window’s actions are rows', this: 'rows', status: 'header', row: 'aligned', size: 18, learn: true,
      summary: 'Instead of explaining Space, the empty list starts with what you can do to the window behind Script Kit. Its header carries the same icon as the input, so you learn that the icon means “this window.”' },
    G: { name: 'Try a sentence', blurb: 'A ghost sentence to take', this: 'suggest', status: 'header', row: 'aligned', size: 18, learn: true,
      summary: 'The empty input holds a sentence about the window, already underlined in the word-mark colors. You learn the grammar by seeing it before you type, and Tab takes the sentence.' },
    H: { name: 'Status at the foot', blurb: 'Today becomes the status', this: 'rows', status: 'foot', row: 'aligned', size: 18, learn: true,
      summary: 'The top is just the input. The bottom-left capsule, today’s Today button, becomes the place that reports runs, waits and failures, then returns to Today.' },
  };

  const FRONT = {
    app: 'Ghostty', title: 'zsh — ~/projects/site', short: 'zsh', icon: 'terminal',
    suggestion: 'explain the last error in this terminal',
    actions: [
      { name: 'Explain the last error', icon: 'sparkles', enter: 'Ask', done: 'Asked about the last error in zsh', context: true },
      { name: 'Open a new tab here', icon: 'terminal', enter: 'Open Tab', done: 'Opened a new Ghostty tab', context: true },
      { name: 'Move it to the left half', icon: 'app-window', enter: 'Move', done: 'Moved zsh to the left half', undo: true, context: true },
    ],
  };
  const IDLE = [
    { title: 'Brain inbox', acc: '1 of 1 · All in Actions', rows: [{ id: 'inbox', name: 'Review the launch checklist', desc: 'Captured yesterday', icon: 'inbox', enter: 'Open', done: 'Opened “Review the launch checklist”' }] },
    { title: 'Flows', rows: [{ id: 'conv', name: 'Conversation', desc: 'Talk it through with the model', icon: 'chevrons-right', enter: 'Run', done: 'Ran Conversation' }] },
    {
      title: 'Suggested', rows: [
        { id: 'doin', name: 'Do in Current App', desc: 'Ghostty', icon: 'crosshair', enter: 'Open', done: 'Opened Do in Current App' },
        { id: 'agent', name: 'Agent Chat', desc: 'Start a conversation with an agent', icon: 'bot', enter: 'Open', done: 'Opened Agent Chat' },
        { id: 'today', name: 'Today', desc: 'Today’s page', icon: 'calendar', enter: 'Open', done: 'Opened Today' },
        { id: 'files', name: 'Search Files', desc: 'Find a file by name', icon: 'folder-search', enter: 'Open', done: 'Opened Search Files' },
        { id: 'clip', name: 'Clipboard History', desc: 'Everything you copied today', icon: 'clipboard-list', enter: 'Open', done: 'Opened Clipboard History' },
        { id: 'trash', name: 'Empty the Trash', desc: 'Waits 8 s before it runs', icon: 'trash-2', enter: 'Empty Trash', hold: true, done: 'Emptied the Trash' },
      ],
    },
  ];

  const S = {
    preset: 'E', custom: null, theme: 'dark', size: null, fam: 'track',
    text: '', sel: 0, staged: null, savedBeforeStage: null,
    last: null, notice: null, hold: null, uses: {}, cmd: false, log: [],
  };
  let cmdTimer = null;
  let rows = [];

  // The active design: a preset, or the preset's choices changed by the mix controls.
  const P = () => (S.custom ? { ...PRESETS[S.preset], ...S.custom } : PRESETS[S.preset]);

  // ── Learning ──
  function level(id) {
    if (!P().learn) return 0;
    if (S.fam === 'new') return 0;
    if (S.fam === 'learning') return 1;
    if (S.fam === 'fluent') return 2;
    const n = S.uses[id] || 0;
    return n >= FLUENT_AT ? 2 : n >= LEARNING_AT ? 1 : 0;
  }
  function show(id, { always = false } = {}) {
    if (S.cmd && P().learn) return { words: true, keys: true, reveal: level(id) === 2 };
    if (always) return { words: true, keys: true };
    const l = level(id);
    return { words: l === 0, keys: l < 2 };
  }
  const used = (id) => { S.uses[id] = (S.uses[id] || 0) + 1; };

  // ── Words ──
  function ago(ms) {
    const m = 60000, h = 60 * m, d = 24 * h;
    if (ms < m) return 'just now';
    if (ms < h) { const mins = ms / m; return `>${[30, 15, 5, 1].find((s) => mins >= s)}m ago`; }
    if (ms < d) return `>${Math.floor(ms / h)}h ago`;
    return `>${Math.floor(ms / d)}d ago`;
  }
  const today = () => new Date().toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' });
  function keys(k, small) {
    const parts = k === 'esc' ? ['esc'] : [...k];
    return parts.map((p) => `<span class="kc${small ? ' kc--sm' : ''}">${esc(p)}</span>`).join('');
  }
  function hint(id, key, words, opts = {}) {
    const s = show(id, opts);
    if (!s.words && !s.keys) return '';
    return `<span class="hint${s.reveal ? ' reveal' : ''}" data-hint="${id}">${s.words && words ? `<span data-ink="label">${esc(words)}</span>` : ''}${s.keys ? `<span data-ink="key">${keys(key, true)}</span>` : ''}</span>`;
  }

  // ── What just happened (the reducer's priority: hold, notice, last run, then rest) ──
  function statusModel() {
    const now = Date.now();
    if (S.hold) {
      const paused = S.text !== '';
      const left = Math.max(0, Math.ceil((HOLD_MS - S.hold.elapsed) / 1000));
      return { kind: 'hold', icon: 'trash-2', state: paused ? 'Paused while you type' : `Runs in ${left}s`, title: S.hold.title,
        progress: S.hold.elapsed / HOLD_MS, lit: true, hints: S.text === '' ? [['stop', '⌘Z', 'Stop', { always: true }]] : [] };
    }
    if (S.notice) return { kind: 'notice', icon: S.notice.failed ? 'circle-alert' : 'history', failed: !!S.notice.failed, title: S.notice.text, sub: ago(now - S.notice.at), hints: [] };
    if (S.last) {
      const hints = [];
      if (S.text === '' && S.last.undo && !S.staged) hints.push(['undo', '⌘Z', 'Undo']);
      if (S.text === '' && !S.staged && S.sel === 0 && P().classic) hints.push(['up', '↑', '']);
      return { kind: 'recent', icon: 'history', title: S.last.title, sub: ago(now - S.last.at), hints, lit: !!(S.staged && S.staged.kind === 'last') };
    }
    return { kind: 'rest' };
  }
  const statusOpacity = (m) => (m.kind === 'hold' || m.lit || m.failed ? 1 : S.text !== '' ? 0.5 : 0.85);
  const hintsHtml = (m) => (m.hints || []).map(([id, k, w, o]) => hint(id, k, w, o)).join('');
  function statusBody(m, { compact = false } = {}) {
    const state = m.state ? `<span class="state" data-ink="label">${esc(compact ? m.state.replace('Paused while you type', 'Paused') : m.state)}</span>` : '';
    return `<span class="icon">${icon(m.icon)}</span>${state}<span class="title" data-ink="label">${esc(m.title)}</span>${m.sub && !compact ? `<span class="sub" data-ink="label">· ${esc(m.sub)}</span>` : ''}${hintsHtml(m)}`;
  }

  // ── The list ──
  function stagedActions() {
    const st = S.staged;
    if (st.kind === 'window') return [
      ...FRONT.actions.map((a) => ({ ...a, context: false })),
      { name: 'Copy its title', icon: 'clipboard-list', enter: 'Copy', done: 'Copied the window title' },
      { name: 'Close it', icon: 'app-window', enter: 'Close', done: `Closed ${FRONT.short}` },
    ];
    if (st.kind === 'last') return [
      { name: 'Undo it', icon: 'undo-2', enter: 'Undo', action: 'undo' },
      { name: 'Run it again', icon: 'history', enter: 'Run Again', done: S.last.title, undo: S.last.undo },
      { name: 'Share it', icon: 'message-circle', enter: 'Share', done: 'Shared the last run' },
    ];
    return [
      { name: st.row.hold ? 'Empty it' : 'Run it', icon: st.row.icon, enter: st.row.enter, row: st.row },
      { name: 'Pin it', icon: 'pin', enter: 'Pin', done: `Pinned ${st.row.name}`, undo: true },
      { name: 'Copy deep link', icon: 'clipboard-list', enter: 'Copy', done: 'Copied a deep link' },
      { name: 'Ask the model about it', icon: 'sparkles', enter: 'Ask', done: `Asked about ${st.row.name}` },
    ];
  }
  function listModel() {
    const p = P();
    const t = S.text.trim();
    const lower = t.toLowerCase();
    if (S.staged) {
      const acts = stagedActions().filter((r) => !lower || r.name.toLowerCase().includes(lower));
      return [{ title: 'Or pick one', rows: acts }];
    }
    const contextFirst = !p.classic && p.this === 'rows';
    const idle = contextFirst
      ? [{ title: FRONT.app, icon: FRONT.icon, sub: FRONT.title, rows: FRONT.actions }, ...IDLE.map((s) => ({ ...s, rows: s.rows.filter((r) => r.id !== 'doin') }))]
      : IDLE;
    if (!t) return idle;
    const all = [...(contextFirst ? FRONT.actions : []), ...IDLE.flatMap((s) => s.rows)];
    const matches = all.filter((r) => r.name.toLowerCase().includes(lower) || (lower.includes('trash') && r.id === 'trash'));
    const fallbacks = {
      title: `Use “${t.length > 28 ? t.slice(0, 28) + '…' : t}” with`, rows: [
        { name: 'Search Google', icon: 'globe', enter: 'Search Google', done: `Searched Google for “${t}”`, fallback: true },
        { name: `Search Files for “${t}”`, icon: 'folder-search', enter: 'Search Files', done: 'Searched Files', fallback: true },
      ],
    };
    if (/remind/.test(lower)) {
      const what = (lower.match(/to (.+?)(?: tomorrow| at |$)/) || [])[1] || 'it';
      const when = (lower.match(/(tomorrow.*|today.*|at \d.*)$/) || [])[1];
      return [
        { title: 'Jev', jev: true, rows: [{ name: `Add reminder: ${what}${when ? ` (${when})` : ''}`, desc: 'Writes it down in today’s note', icon: 'alarm-clock', enter: 'Add Reminder', done: `Added reminder: ${what}`, undo: true, jev: 1 }] },
        fallbacks,
      ];
    }
    if (/explain|error/.test(lower)) {
      return [
        { title: 'Jev', jev: true, rows: [{ name: 'Explain the last error in zsh', desc: `Reads the end of ${FRONT.title}`, icon: 'sparkles', enter: 'Ask', done: 'Asked about the last error in zsh', jev: 1 }] },
        fallbacks,
      ];
    }
    const sections = [];
    if (matches.length) sections.push({ title: 'Matches', rows: matches });
    sections.push({ title: 'Jev', jev: true, rows: [{ name: `Ask the model: “${t}”`, desc: 'Opens the chat with these words', icon: 'sparkles', enter: 'Ask', done: 'Asked the model', jev: 1 }] });
    sections.push(fallbacks);
    return sections;
  }

  function marksFor(text) {
    const spans = [];
    const add = (re, word) => { const m = re.exec(text); if (m) spans.push([m.index, m.index + m[0].length, word]); };
    add(/remind me/i, 'action');
    add(/(?<=to )call \w+/i, 'subject');
    add(/(tomorrow|today) at \d+\s?(am|pm)/i, 'details');
    add(/\bexplain\b/i, 'action');
    add(/the last error/i, 'target');
    add(/this terminal/i, 'subject');
    for (const m of text.matchAll(/(^|\s)(@\w+)/g)) spans.push([m.index + m[1].length, m.index + m[0].length, 'subject']);
    for (const m of text.matchAll(/(^|\s)(\/\w+)/g)) spans.push([m.index + m[1].length, m.index + m[0].length, 'action']);
    spans.sort((a, b) => a[0] - b[0]);
    let out = '', at = 0;
    for (const [s, e, w] of spans) {
      if (s < at) continue;
      out += esc(text.slice(at, s)) + `<span class="mark" style="text-decoration-color:${MARKS[w]}">${esc(text.slice(s, e))}</span>`;
      at = e;
    }
    return out + esc(text.slice(at));
  }

  // ── Drawing ──
  const win = $('#win'), q = $('#q');
  const ghostShown = () => !P().classic && P().this === 'suggest' && !S.staged && S.text === '';

  function render() {
    const p = P();
    const classic = !!p.classic;
    win.dataset.theme = S.theme;
    win.dataset.layout = classic ? 'classic' : 'row';
    win.dataset.row = classic ? 'classic' : p.row;
    win.dataset.rule = 'true';
    win.style.setProperty('--in-size', `${S.size || p.size}px`);
    q.placeholder = S.staged ? 'Say what to do with it' : ghostShown() ? '' : 'Search or ask';
    if (q.value !== S.text) q.value = S.text;

    const sm = statusModel();

    // Classic: the 34 pt line above the input (preset A).
    const line = $('#line');
    if (classic) {
      const m = sm.kind === 'rest' ? { kind: 'behind', icon: FRONT.icon, title: FRONT.title, sub: `in ${FRONT.app}`, hints: [] } : sm;
      line.dataset.kind = m.kind;
      line.dataset.lit = String(!!m.lit);
      line.dataset.failed = String(!!m.failed);
      line.style.opacity = statusOpacity(m) === 0.85 ? 0.8 : statusOpacity(m);
      line.innerHTML = `${statusBody(m).replace(hintsHtml(m), '')}<span class="grow"></span>${hintsHtml(m)}${m.kind === 'hold' ? `<span class="progress" style="width:${(m.progress * 100).toFixed(1)}%"></span>` : ''}`;
    } else line.innerHTML = '';

    // Input row: lead (chip or the window's icon) and trail (offered chip or inline status).
    let leadHtml = '', trailHtml = '';
    if (S.staged) {
      const st = S.staged;
      const label = st.kind === 'window' ? FRONT.short : st.kind === 'last' ? S.last.title : st.row.name;
      const ic = st.kind === 'window' ? FRONT.icon : st.kind === 'last' ? 'history' : st.row.icon;
      leadHtml = `<span class="chip chip--attached"><span class="icon">${icon(ic)}</span><span class="name" data-ink="label">${esc(label)}</span>${hint('esc', 'esc', '')}</span>`;
    } else if (!classic) {
      // Holding ⌘ turns the tile into its key, in place: nothing moves and nothing overlaps the words.
      const asKey = S.cmd && S.text === '';
      leadHtml = `<span class="ftile${asKey ? ' ftile--key reveal' : ''}" id="ftile" role="button" tabindex="-1" title="${esc(FRONT.title)} · in ${FRONT.app}" aria-label="Act on ${esc(FRONT.app)}: ${esc(FRONT.title)}" data-typing="${S.text !== ''}">${asKey ? '<span data-ink="key" class="tilekey">␣</span>' : icon(FRONT.icon)}</span>`;
    }
    if (classic && !S.staged && S.text === '') {
      trailHtml = `<span class="chip"><span class="icon">${icon(FRONT.icon)}</span><span class="name" data-ink="label">${esc(FRONT.app)}</span><span class="hint"><span class="word" data-ink="label">Use</span><span data-ink="key">${keys('␣', true)}</span></span></span>`;
    }
    if (!classic && p.status === 'inrow') {
      const m = sm.kind === 'rest' ? (S.text ? null : { kind: 'rest', icon: 'calendar', title: today(), hints: [['today', '⌘T', 'Today']] }) : sm;
      if (m) {
        const compact = S.text.length > 26 && m.kind !== 'hold';
        trailHtml += `<span class="status" data-kind="${m.kind}" data-failed="${!!m.failed}" data-compact="${compact}" style="opacity:${statusOpacity(m)}">${statusBody(m, { compact })}</span>`;
      }
    }
    $('#lead').innerHTML = leadHtml;
    $('#trail').innerHTML = trailHtml;
    $('#inprogress').style.width = !classic && sm.kind === 'hold' ? `${(sm.progress * 100).toFixed(1)}%` : '0';

    // Mirror: word marks under typed text, or the ghost sentence.
    const mirror = $('#mirror');
    if (ghostShown()) {
      const tab = show('tab');
      mirror.innerHTML = `<span class="ghost">${marksFor(FRONT.suggestion)}</span>${tab.keys ? `<span class="ghost-key" data-ink="key">${keys('⇥', true)}</span>` : ''}`;
    } else mirror.innerHTML = marksFor(S.text);
    mirror.scrollLeft = q.scrollLeft;

    // List
    const sections = listModel();
    rows = sections.flatMap((s) => s.rows);
    S.sel = Math.max(0, Math.min(S.sel, rows.length - 1));
    const headerStatus = !classic && p.status === 'header' && sm.kind !== 'rest'
      ? `<span class="hstatus" data-kind="${sm.kind}" data-failed="${!!sm.failed}" style="opacity:${statusOpacity(sm)}">${statusBody(sm)}</span>` : '';
    let i = 0, html = '';
    sections.forEach((s, si) => {
      let acc = '';
      if (s.jev) {
        const sh = show('jev');
        acc = sh.keys && (!p.learn || (level('jev') === 0 && !S.cmd)) ? `<span data-ink="key">${keys('⌘1', true)}</span>` : '';
      } else if (s.acc && classic) acc = `<span data-ink="label">${esc(s.acc)}</span>`;
      const status = si === 0 ? headerStatus : '';
      html += `<div class="hdr" data-style="${classic ? 'caps' : 'quiet'}">${s.icon ? `<span class="hicon">${icon(s.icon)}</span>` : ''}<span class="label" data-ink="label">${esc(s.title)}</span>${s.sub && !status ? `<span class="hsub" data-ink="label">${esc(s.sub)}</span>` : ''}${acc ? `<span class="acc">${acc}</span>` : ''}${status}</div>`;
      s.rows.forEach((r) => {
        const selected = i === S.sel;
        let accHtml = '';
        if (r.jev && p.learn && S.cmd) accHtml = `<span class="rowkey reveal" data-ink="key">${keys('⌘' + r.jev, true)}</span>`;
        else if (r.jev) accHtml = `<span class="acc-icon">${icon('sparkles')}</span>`;
        else if (r.fallback) accHtml = `<span class="acc-icon">${icon('zap')}</span>`;
        html += `<div class="row" role="option" id="row-${i}" data-i="${i}" aria-selected="${selected}" data-jev="${!!r.jev}"><span class="tile">${icon(r.icon)}</span><span class="copy"><span class="name" data-ink="label">${esc(r.name)}</span>${r.desc ? `<span class="desc">${esc(r.desc)}</span>` : ''}</span>${accHtml}</div>`;
        i++;
      });
    });
    const list = $('#list');
    list.innerHTML = html;
    q.setAttribute('aria-activedescendant', `row-${S.sel}`);
    const selEl = list.querySelector(`[data-i="${S.sel}"]`);
    if (selEl) {
      const top = selEl.offsetTop - 28, bottom = selEl.offsetTop + selEl.offsetHeight + 52;
      if (top < list.scrollTop) list.scrollTop = top;
      else if (bottom > list.scrollTop + list.clientHeight) list.scrollTop = bottom - list.clientHeight;
    }

    // Capsules: the left one (Today, or status at the foot) and ↵ · ⌘K · ⌘↵.
    const sel = rows[S.sel];
    const caps = [];
    if (classic) {
      caps.push(`<span class="cap cap--left" data-cap="today"><span class="icon">${icon('calendar')}</span><span class="keys" data-ink="key">${keys('⌘T')}</span><span data-ink="label">Today</span></span>`);
    } else if (p.status === 'foot') {
      if (sm.kind === 'rest') {
        const t = show('today');
        caps.push(`<span class="cap cap--status" data-cap="today"><span class="icon">${icon('calendar')}</span><span data-ink="label">Today</span>${t.keys ? `<span class="keys" data-ink="key">${keys('⌘T', true)}</span>` : ''}</span>`);
      } else {
        const fill = sm.kind === 'hold' ? `<span class="fill" style="width:${(sm.progress * 100).toFixed(1)}%"></span>` : '';
        caps.push(`<span class="cap cap--status" data-cap="status" data-failed="${!!sm.failed}" style="opacity:${Math.max(0.6, statusOpacity(sm))}">${fill}${statusBody(sm)}</span>`);
      }
    }
    caps.push('<span class="spacer"></span>');
    const enterKeys = show('enter', { always: !p.learn });
    caps.push(`<span class="cap${enterKeys.keys ? '' : ' cap--keys-only'}" data-cap="enter"><span data-ink="label">${esc(sel ? sel.enter : 'Ask')}</span>${enterKeys.keys ? `<span class="keys" data-ink="key">${keys('↵')}</span>` : ''}</span>`);
    for (const [id, k, w] of [['actions', '⌘K', 'Actions'], ['agent', '⌘↵', S.text ? 'Do It' : 'Agent']]) {
      const s = show(id);
      if (!s.words && !s.keys) continue;
      caps.push(`<span class="cap${s.words ? '' : ' cap--keys-only'}${s.reveal ? ' reveal' : ''}" data-cap="${id}">${s.words ? `<span data-ink="label">${esc(w)}</span>` : ''}<span class="keys" data-ink="key">${keys(k)}</span></span>`);
    }
    $('#foot').innerHTML = caps.join('');

    renderChrome();
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
  function countInk() {
    const chrome = [...win.querySelectorAll('[data-ink]')].filter((el) => !el.closest('.row') && !(el.closest('.hdr') && !el.closest('.hstatus')) && visible(el));
    const labels = chrome.filter((el) => el.dataset.ink === 'label' && !el.querySelector('[data-ink]')).length;
    const keyCount = [...win.querySelectorAll('.kc')].filter((k) => !k.closest('.row') && !k.closest('.mirror') && visible(k)).length;
    const headers = [...win.querySelectorAll('.hdr')].filter(visible).length;
    const n = (count, one, many) => `<b>${count}</b> ${count === 1 ? one : many}`;
    $('#ink').innerHTML = `Outside the rows right now: ${n(labels, 'piece of text', 'pieces of text')}, ${n(keyCount, 'keycap', 'keycaps')} and ${n(headers, 'section header', 'section headers')}.`;
  }

  function renderChrome() {
    const p = P();
    document.querySelectorAll('#variants button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.v === S.preset && !S.custom)));
    const mixBtn = document.querySelector('#variants [data-v="mix"]');
    if (mixBtn) mixBtn.setAttribute('aria-checked', String(!!S.custom));
    $('#mix').dataset.disabled = String(!!p.classic);
    for (const axis of ['this', 'status', 'row']) {
      document.querySelectorAll(`#axis-${axis} button`).forEach((b) => b.setAttribute('aria-checked', String(!p.classic && p[axis] === b.dataset.value)));
    }
    document.querySelectorAll('#themes button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.theme === S.theme)));
    const size = S.size || p.size;
    document.querySelectorAll('#sizes button').forEach((b) => b.setAttribute('aria-checked', String(+b.dataset.size === size)));
    document.querySelectorAll('#fams button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.fam === S.fam)));
    $('#familiarity-control').hidden = !p.learn;
  }

  function renderNotes() {
    const p = P();
    const title = S.custom ? 'Your mix' : `${S.preset} · ${PRESETS[S.preset].name}`;
    if (p.classic) {
      $('#notes').innerHTML = `<h2>${esc(title)}</h2><p class="sum">${p.summary}</p>
        <h3>Where each thing lives</h3><table class="homes">
        <tr><th>The window behind Script Kit</th><td>The line above the input <em>and</em> the dashed chip in the input</td></tr>
        <tr><th>What just happened</th><td>The 34 pt line, in place of the window’s name</td></tr>
        <tr><th>Today</th><td>A fourth capsule, bottom left</td></tr></table>`;
      return;
    }
    const costs = [AXES.this[p.this].cost, AXES.status[p.status].cost, 'The always-present 34 pt line above the input (SLOT_LINE_PX) and the checks built around it.'];
    $('#notes').innerHTML = `
      <h2>${esc(title)}</h2>
      <p class="sum">${S.custom ? 'A combination of the choices above.' : p.summary}</p>
      <h3>Where each thing lives</h3>
      <table class="homes">
        <tr><th>The window behind Script Kit</th><td>${AXES.this[p.this].home}</td></tr>
        <tr><th>What just happened</th><td>${AXES.status[p.status].home}</td></tr>
        <tr><th>Input row</th><td>${AXES.row[p.row].home}</td></tr>
        <tr><th>Keys</th><td>↵ always names what it runs. Other keys lose their words after 3 uses and their keycaps after 8. Hold ⌘ to see them all.</td></tr>
      </table>
      <h3>Gives up</h3><ul class="breaks">${costs.map((c) => `<li>${c}</li>`).join('')}</ul>`;
  }

  function log(text) {
    S.log.unshift(text);
    S.log = S.log.slice(0, 4);
    $('#log').innerHTML = S.log.map((t, i) => `<li style="opacity:${1 - i * 0.2}">${t}</li>`).join('');
  }

  // ── Behavior ──
  function stage(st) {
    S.savedBeforeStage = { text: S.text, sel: S.sel };
    S.staged = st; S.text = ''; S.sel = 0; render();
  }
  function letGo() {
    const saved = S.savedBeforeStage || { text: '', sel: 0 };
    S.staged = null; S.text = saved.text; S.sel = saved.sel; render();
  }
  function finish(entry) {
    S.last = { ...entry, at: Date.now() };
    S.notice = null; S.staged = null; S.text = ''; S.sel = 0;
  }
  function run(r, how) {
    if (!r) return;
    if (r.action === 'undo') { undo('↵ on Undo it'); return; }
    const target = r.row || r;
    if (target.hold) {
      S.hold = { title: target.name, done: target.done, elapsed: 0 };
      S.staged = null; S.text = ''; S.sel = 0;
      log(`${how} → <span>${esc(target.name)}</span> waits 8 s before it runs`);
      render();
      return;
    }
    finish({ title: target.done || r.done || `Ran ${target.name}`, undo: !!(r.undo ?? target.undo) });
    log(`${how} → <span>${esc(S.last.title)}</span>`);
    render();
  }
  function undo(how) {
    if (S.hold) {
      S.notice = { text: `Didn’t run: ${S.hold.title}`, at: Date.now() };
      S.hold = null; S.staged = null;
      log(`${how} → stopped the waiting run`);
    } else if (S.last && S.last.undo) {
      S.notice = { text: `Took back “${S.last.title}”`, at: Date.now() };
      S.last = { ...S.last, undo: false };
      S.staged = null;
      log(`${how} → <span>undid it</span>`);
    } else return false;
    S.text = ''; S.sel = 0; render();
    return true;
  }

  q.addEventListener('input', () => { S.text = q.value; S.sel = 0; if (S.notice && !S.notice.failed) S.notice = null; render(); });
  q.addEventListener('scroll', () => { $('#mirror').scrollLeft = q.scrollLeft; });
  q.addEventListener('keydown', (e) => {
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
      if (empty && !S.staged && S.sel === 0 && S.last) { used('up'); log('↑ → acting on the last run'); stage({ kind: 'last' }); return; }
      S.sel = Math.max(0, S.sel - 1); render(); return;
    }
    if (e.key === 'Tab' && !e.shiftKey && ghostShown()) {
      e.preventDefault(); used('tab');
      S.text = FRONT.suggestion; S.sel = 0;
      log('Tab → took the suggested sentence');
      render(); q.setSelectionRange(S.text.length, S.text.length); return;
    }
    if (e.key === 'Enter' && e.metaKey) {
      e.preventDefault(); used('agent');
      log(S.text ? '⌘↵ → <span>Do It</span> runs Jev’s first reading' : '⌘↵ → <span>Agent</span> opens the chat under this input');
      pressCap('agent');
      if (S.text) run(rows.find((r) => r.jev) || rows[0], '⌘↵');
      return;
    }
    if (e.key === 'Enter') { e.preventDefault(); used('enter'); pressCap('enter'); run(rows[S.sel], '↵'); return; }
    if (e.key === 'Escape') {
      e.preventDefault();
      if (S.staged) { used('esc'); log('Esc → let go'); letGo(); return; }
      if (S.text) { S.text = ''; S.sel = 0; render(); }
      return;
    }
    if (e.key === ' ' && empty && !S.staged) {
      e.preventDefault(); used('space');
      log(`Space → acting on <span>${esc(FRONT.short)}</span>`);
      stage({ kind: 'window' }); return;
    }
    if (e.metaKey && e.key.toLowerCase() === 'k') {
      e.preventDefault(); used('actions'); pressCap('actions');
      const r = rows[S.sel];
      if (!r || S.staged) return;
      log(`⌘K → acting on <span>${esc(r.name)}</span>`);
      stage({ kind: 'row', row: r }); return;
    }
    if (e.metaKey && e.key.toLowerCase() === 'z' && empty) {
      const id = S.hold ? 'stop' : 'undo';
      if (undo('⌘Z')) { e.preventDefault(); used(id); }
      return;
    }
    if (e.metaKey && /^[1-3]$/.test(e.key)) {
      const r = rows.find((x) => x.jev === +e.key);
      if (r) { e.preventDefault(); used('jev'); run(r, `⌘${e.key}`); }
    }
  });
  q.addEventListener('keyup', (e) => { if (e.key === 'Meta') { clearTimeout(cmdTimer); if (S.cmd) { S.cmd = false; render(); } } });
  window.addEventListener('blur', () => { clearTimeout(cmdTimer); if (S.cmd) { S.cmd = false; render(); } });

  function pressCap(id) {
    const c = document.querySelector(`[data-cap="${id}"]`);
    if (!c) return;
    c.dataset.pressed = 'true';
    setTimeout(() => { c.dataset.pressed = 'false'; }, 140);
  }

  // Pointer is the fallback: click selects, double-click runs, capsules and the icon do what their keys do.
  win.addEventListener('mousedown', (e) => { if (e.target !== q) { e.preventDefault(); q.focus(); } });
  $('#list').addEventListener('click', (e) => { const r = e.target.closest('.row'); if (r) { S.sel = +r.dataset.i; render(); } });
  $('#list').addEventListener('dblclick', (e) => { const r = e.target.closest('.row'); if (r) run(rows[+r.dataset.i], 'Double-click'); });
  $('#lead').addEventListener('click', (e) => {
    if (e.target.closest('#ftile') && !S.staged) { used('space'); log(`Clicked the icon → acting on <span>${esc(FRONT.short)}</span>`); stage({ kind: 'window' }); }
  });
  $('#foot').addEventListener('click', (e) => {
    const c = e.target.closest('[data-cap]');
    if (!c) return;
    const id = c.dataset.cap;
    if (id === 'enter') run(rows[S.sel], '↵ capsule');
    else if (id === 'actions' && rows[S.sel] && !S.staged) { used('actions'); stage({ kind: 'row', row: rows[S.sel] }); }
    else if (id === 'agent') log('⌘↵ capsule → <span>Agent</span> opens the chat under this input');
    else if (id === 'today') { used('today'); log('Today → opens today’s page under this input'); }
    else if (id === 'status' && S.last && !S.hold) { used('up'); log('Clicked the status → acting on the last run'); stage({ kind: 'last' }); }
  });

  setInterval(() => {
    if (S.hold) {
      if (S.text === '') S.hold.elapsed += 100;
      if (S.hold.elapsed >= HOLD_MS) {
        finish({ title: S.hold.done, undo: false });
        log(`8 s passed → <span>${esc(S.last.title)}</span>`);
        S.hold = null;
      }
      render();
    }
  }, 100);
  setInterval(() => { if (S.last || S.notice) render(); }, 15000);

  // ── Controls ──
  $('#variants').innerHTML = Object.entries(PRESETS).map(([k, v]) => `<button role="radio" data-v="${k}"><b>${k} · ${esc(v.name)}</b><span>${esc(v.blurb)}</span></button>`).join('')
    + '<button role="radio" data-v="mix"><b>Your mix</b><span>Set below</span></button>';
  $('#variants').addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b) return;
    if (b.dataset.v === 'mix') { if (!S.custom) S.custom = PRESETS[S.preset].classic ? { classic: false, this: 'icon', status: 'header', row: 'aligned', size: 18, learn: true } : {}; }
    else { S.preset = b.dataset.v; S.custom = null; }
    S.cmd = false; syncHash(); renderNotes(); render(); q.focus();
  });
  $('#mix').addEventListener('click', (e) => {
    const b = e.target.closest('[data-axis]');
    if (!b) return;
    const base = PRESETS[S.preset].classic ? { classic: false, this: 'icon', status: 'header', row: 'aligned', size: 18, learn: true } : {};
    S.custom = { ...base, ...(S.custom || {}), [b.dataset.axis]: b.dataset.value };
    syncHash(); renderNotes(); render(); q.focus();
  });
  $('#themes').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.theme = b.dataset.theme; syncHash(); render(); q.focus(); } });
  $('#sizes').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.size = +b.dataset.size; render(); q.focus(); } });
  $('#fams').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.fam = b.dataset.fam; if (S.fam === 'track') S.uses = {}; render(); q.focus(); } });
  $('#states').addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b) return;
    S.staged = null; S.hold = null; S.notice = null; S.text = ''; S.sel = 0;
    const st = b.dataset.state;
    S.last = st === 'fresh' ? null : (S.last || { title: 'Added reminder: call mom', undo: true, at: Date.now() - 20000 });
    if (st === 'typing') S.text = 'remind me to call mom tomorrow at 5pm';
    if (st === 'staged') { S.savedBeforeStage = { text: '', sel: 0 }; S.staged = { kind: 'window' }; }
    if (st === 'hold') S.hold = { title: 'Empty the Trash', done: 'Emptied the Trash', elapsed: 1500 };
    if (st === 'failed') S.notice = { text: 'Couldn’t save the note', failed: true, at: Date.now() - 5000 };
    log(`Jumped to <span>${esc(b.textContent)}</span>`);
    render(); q.focus();
  });

  function syncHash() {
    const p = P();
    const tail = S.custom && !p.classic ? `-${p.this}-${p.status}-${p.row}` : '';
    history.replaceState(null, '', `#${S.custom ? 'mix' : S.preset}-${S.theme}${tail}`);
  }
  const m = /^#([A-H]|mix)-(dark|light)(?:-(icon|rows|suggest)-(inrow|header|foot)-(aligned|compact))?$/.exec(location.hash);
  if (m) {
    S.theme = m[2];
    if (m[1] === 'mix' && m[3]) S.custom = { classic: false, this: m[3], status: m[4], row: m[5], size: 18, learn: true };
    else if (PRESETS[m[1]]) S.preset = m[1];
  }

  function fit() {
    const stage = $('#stage'), sc = $('#scaler');
    const k = Math.min(1, stage.clientWidth / 750);
    sc.style.transform = k < 1 ? `scale(${k})` : '';
    sc.style.height = `${480 * k}px`;
    sc.style.width = `${750 * k}px`;
  }
  window.addEventListener('resize', fit);

  // Agents and tests can read and drive the same model.
  window.mockup = {
    state: () => JSON.parse(JSON.stringify({ ...S, log: undefined, design: P(), rows: rows.map((r) => r.name) })),
    set: (patch) => { Object.assign(S, patch); renderNotes(); render(); },
    presets: Object.keys(PRESETS),
  };

  fit(); renderNotes(); render(); q.focus();
})();

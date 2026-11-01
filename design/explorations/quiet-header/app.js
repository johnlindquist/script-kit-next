// One model, four ways of drawing the top of the window. Every option shares the same rows,
// keys and timing, so switching options never resets what you typed or what ran.
(() => {
  'use strict';
  const $ = (s) => document.querySelector(s);
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const icon = (n) => `<svg viewBox="0 0 24 24" aria-hidden="true">${(window.ICONS || {})[n] || ''}</svg>`;

  // Timing and thresholds from crates/sk-one-window/src/contract.rs, plus the proposal's two.
  const HOLD_MS = 8000;
  const USE_HINT_GRABS = 3; // existing rule: the chip's "Use" word goes after three grabs
  const LEARNING_AT = 3; // proposal: a key's words go after three uses
  const FLUENT_AT = 8; // proposal: its keycap goes after eight
  const REVEAL_AFTER_MS = 350; // proposal: holding ⌘ this long shows every key in place
  const MARKS = { subject: '#ffb86b', action: '#f5c542', target: '#6fdc9b', details: '#8fb8ff', owner: '#c9a7ff' };

  const VARIANTS = {
    A: {
      name: 'Today', blurb: 'What the app draws now', size: 20, learn: false, merged: false, headers: 'caps', rule: true, today: 'footer', chip: 'app',
      summary: 'The native layout as it ships, rebuilt with the same measurements so you can compare the others against it.',
      homes: [['What “this” is', 'The line above the input <em>and</em> the dashed chip in the input'], ['What just happened', 'The line, replacing what “this” is'], ['What the keys do', 'Four capsules: Today on the left, ↵, ⌘K and ⌘↵ on the right'], ['Groups', 'Uppercase headers, some with counts on the right']],
      changes: [], keeps: ['34 pt line, 34 pt input, 28 pt header, 44 pt rows', 'Gold tile on the focused row'], breaks: [],
    },
    B: {
      name: 'Say it once', blurb: 'Each fact has one home', size: 18, learn: false, merged: false, headers: 'quiet', rule: false, today: 'line', chip: 'title',
      summary: 'Nothing is said twice. The chip says what “this” is, the line says what happened, and the capsules say what the keys do.',
      homes: [['What “this” is', 'Only the dashed chip, now naming the window (“zsh”), not just the app'], ['What just happened', 'Only the line; before anything runs it shows today’s date with ⌘T'], ['What the keys do', 'Exactly three capsules: ↵, ⌘K, ⌘↵'], ['Groups', 'Sentence-case headers in a quieter weight; counts removed']],
      changes: ['Today moves from a fourth capsule into the line’s resting state', 'The rule under the input goes; spacing separates input and list', 'Input text 18 pt instead of 20 pt'],
      keeps: ['Every height: 34 pt line, 34 pt input, 28 pt header, 44 pt rows', 'Space, ⌘K and ↑ still attach a subject; Esc lets go', '“Three keys, nothing more” — already a design principle, which the Today capsule currently exceeds'],
      breaks: ['The line no longer names the window behind Script Kit when nothing has run; the chip does'],
    },
    C: {
      name: 'Learn as you go', blurb: 'Hints recede as you use them', size: 18, learn: true, merged: false, headers: 'quiet', rule: false, today: 'line', chip: 'title',
      summary: 'Option B, plus hints that retire as you learn them. The first time, each key is labeled. Use it a few times and the label goes. Later, the keycap goes too. Hold ⌘ to see every key in place.',
      homes: [['What “this” is', 'The chip; its ␣ hint follows the same learning rule'], ['What just happened', 'The line; ⌘Z and ↑ follow the learning rule, but a waiting run always shows how to stop it'], ['What the keys do', '↵ always shows what it runs. ⌘K and ⌘↵ shrink to keycaps, then disappear until ⌘ is held'], ['Jev’s ⌘1–3', 'Shown on Jev’s header until you have used it; then only while ⌘ is held, on Jev’s rows']],
      changes: ['Words go after 3 uses, keycaps after 8 (the app already drops the chip’s “Use” after 3)', 'Holding ⌘ for 350 ms reveals every key in place — no overlay', 'Rows show ⌘1–3 only while ⌘ is held'],
      keeps: ['All of option B', '↵ is always named for exactly what Enter runs', 'Stopping a waiting run is never hidden'],
      breaks: ['“No per-row keycaps” gets one exception: while ⌘ is held'],
    },
    D: {
      name: 'One row', blurb: 'The line joins the input', size: 18, learn: true, merged: true, headers: 'quiet', rule: true, today: 'line', chip: 'icon',
      summary: 'Option C with the line folded into the input row. The window behind Script Kit becomes the icon at the start of the input. What happened sits at the right and shrinks to an icon as your sentence reaches it.',
      homes: [['What “this” is', 'The icon at the start of the input; Space attaches it as a named chip'], ['What just happened', 'The right end of the input row; a waiting run adds a gold rule under the input'], ['What the keys do', 'As in option C'], ['Space returned', '37 pt more list (most of another row)']],
      changes: ['No separate line above the input', 'The front window’s icon keeps its slot while typing, so the text never shifts'],
      keeps: ['The input still never moves', 'Holds, undo and notices keep their words and keys'],
      breaks: ['The always-present 34 pt line (SLOT_LINE_PX) and the reducer’s line contract', 'Long sentences compete with status for the same row'],
    },
  };

  const FRONT = { app: 'Ghostty', title: 'zsh — ~/projects/site', short: 'zsh', icon: 'terminal' };
  const IDLE = [
    { title: 'Brain inbox', acc: '1 of 1 · All in Actions', rows: [{ id: 'inbox', name: 'Review the launch checklist', desc: 'Captured yesterday', icon: 'inbox', enter: 'Open', done: 'Opened “Review the launch checklist”' }] },
    { title: 'Flows', rows: [{ id: 'conv', name: 'Conversation', desc: 'Talk it through with the model', icon: 'chevrons-right', enter: 'Run', done: 'Ran Conversation' }] },
    {
      title: 'Suggested', rows: [
        { id: 'doin', name: 'Do in Current App', desc: 'Ghostty', icon: 'crosshair', enter: 'Open', done: 'Opened Do in Current App' },
        { id: 'agent', name: 'Agent Chat', desc: 'Start a conversation with an agent', icon: 'bot', enter: 'Open', done: 'Opened Agent Chat' },
        { id: 'files', name: 'Search Files', desc: 'Find a file by name', icon: 'folder-search', enter: 'Open', done: 'Opened Search Files' },
        { id: 'clip', name: 'Clipboard History', desc: 'Everything you copied today', icon: 'clipboard-list', enter: 'Open', done: 'Opened Clipboard History' },
        { id: 'notes', name: 'Notes', desc: 'Pinned first', icon: 'sticky-note', enter: 'Open', done: 'Opened Notes' },
        { id: 'trash', name: 'Empty the Trash', desc: 'Waits 8 s before it runs', icon: 'trash-2', enter: 'Empty Trash', hold: true, done: 'Emptied the Trash' },
      ],
    },
  ];

  const S = {
    variant: 'A', theme: 'dark', size: null, fam: 'track',
    text: '', sel: 0, staged: null, savedBeforeStage: null,
    last: null, notice: null, hold: null, uses: {}, grabs: 0, cmd: false, log: [],
  };
  let cmdTimer = null;
  let rows = []; // flat, selectable rows of the current list

  const V = () => VARIANTS[S.variant];

  // ── Learning ──
  function level(id) {
    if (!V().learn) return 0;
    if (S.fam === 'new') return 0;
    if (S.fam === 'learning') return 1;
    if (S.fam === 'fluent') return 2;
    const n = S.uses[id] || 0;
    return n >= FLUENT_AT ? 2 : n >= LEARNING_AT ? 1 : 0;
  }
  // What a hint shows: its words, its keycaps, or nothing. Holding ⌘ shows everything.
  function show(id, { always = false } = {}) {
    if (S.cmd && V().learn) return { words: true, keys: true, reveal: level(id) === 2 };
    if (always) return { words: true, keys: true };
    const l = level(id);
    return { words: l === 0, keys: l < 2 };
  }
  function used(id) { S.uses[id] = (S.uses[id] || 0) + 1; }

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
    return `<span class="hint${s.reveal ? ' reveal' : ''}" data-ink="label" data-hint="${id}">${s.words && words ? `<span data-ink="label">${esc(words)}</span>` : ''}${s.keys ? `<span data-ink="key">${keys(key, true)}</span>` : ''}</span>`;
  }

  // ── What the line says (the reducer's priority: hold, notice, last run, then the resting state) ──
  function lineModel() {
    const v = V();
    const now = Date.now();
    if (S.hold) {
      const paused = S.text !== '';
      const left = Math.max(0, Math.ceil((HOLD_MS - S.hold.elapsed) / 1000));
      return {
        kind: 'hold', icon: 'trash-2', state: paused ? 'Paused while you type' : `Runs in ${left}s`, title: S.hold.title,
        progress: S.hold.elapsed / HOLD_MS, lit: true,
        hints: S.text === '' ? [['stop', '⌘Z', 'Stop', { always: true }]] : [],
      };
    }
    if (S.notice) return { kind: 'notice', icon: S.notice.failed ? 'circle-pause' : 'history', title: S.notice.text, sub: ago(now - S.notice.at), hints: [] };
    if (S.last) {
      const hints = [];
      if (S.text === '' && S.last.undo && !S.staged) hints.push(['undo', '⌘Z', 'Undo']);
      if (S.text === '' && !S.staged && S.sel === 0) hints.push(['up', '↑', '']);
      return { kind: 'recent', icon: 'history', title: S.last.title, sub: ago(now - S.last.at), hints, lit: !!(S.staged && S.staged.kind === 'last') };
    }
    if (v.today === 'line') return { kind: 'today', icon: 'calendar', title: today(), hints: [['today', '⌘T', 'Today']] };
    return { kind: 'behind', icon: FRONT.icon, title: FRONT.title, sub: `in ${FRONT.app}`, hints: [] };
  }
  function lineOpacity(m) {
    if (m.kind === 'hold' || m.lit) return 1;
    return S.text !== '' ? 0.5 : 0.8;
  }

  // ── The list ──
  function stagedActions() {
    const st = S.staged;
    if (st.kind === 'window') return [
      { name: 'Ask about this window', icon: 'sparkles', enter: 'Ask', done: `Asked about ${FRONT.short}` },
      { name: 'Move it to the left half', icon: 'app-window', enter: 'Move', done: `Moved ${FRONT.short} to the left half`, undo: true },
      { name: 'Copy its title', icon: 'clipboard-list', enter: 'Copy', done: 'Copied the window title', undo: false },
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
    const t = S.text.trim();
    const lower = t.toLowerCase();
    if (S.staged) {
      const acts = stagedActions().filter((r) => !lower || r.name.toLowerCase().includes(lower));
      return [{ title: 'Or pick one', rows: acts }];
    }
    if (!t) return IDLE;
    const all = IDLE.flatMap((s) => s.rows);
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
  function render() {
    const v = V();
    win.dataset.theme = S.theme;
    win.dataset.merged = String(v.merged);
    win.dataset.rule = String(v.rule);
    const size = S.size || v.size;
    win.style.setProperty('--in-size', `${size}px`);
    // Option D teaches Space in the placeholder: it costs no layout and goes away as you type.
    const teachSpace = v.chip === 'icon' && !S.staged && level('space') === 0;
    q.placeholder = S.staged ? 'Say what to do with it' : teachSpace ? `Search or ask — Space acts on ${FRONT.short}` : 'Search or ask';
    if (q.value !== S.text) q.value = S.text;

    // Line (or, in option D, the status at the end of the input row).
    const lm = lineModel();
    const hints = lm.hints.map(([id, k, w, o]) => hint(id, k, w, o)).join('');
    const lineInner = `<span class="icon">${icon(lm.icon)}</span>${lm.state ? `<span class="state" data-ink="label">${esc(lm.state)}</span>` : ''}<span class="title" data-ink="label">${esc(lm.title)}</span>${lm.sub ? `<span class="sub" data-ink="label">· ${esc(lm.sub)}</span>` : ''}`;
    const line = $('#line');
    if (!v.merged) {
      line.dataset.kind = lm.kind;
      line.dataset.lit = String(!!lm.lit);
      line.style.opacity = lineOpacity(lm);
      line.setAttribute('aria-label', [lm.state, lm.title, lm.sub].filter(Boolean).join(' '));
      line.innerHTML = `${lineInner}<span class="grow"></span>${hints}${lm.kind === 'hold' ? `<span class="progress" style="width:${(lm.progress * 100).toFixed(1)}%"></span>` : ''}`;
    } else {
      line.innerHTML = '';
    }

    // Lead: an attached chip (all options) or option D's front-window icon.
    const lead = $('#lead'), trail = $('#trail');
    let leadHtml = '', trailHtml = '';
    if (S.staged) {
      const st = S.staged;
      const label = st.kind === 'window' ? FRONT.short : st.kind === 'last' ? S.last.title : st.row.name;
      const ic = st.kind === 'window' ? FRONT.icon : st.kind === 'last' ? 'history' : st.row.icon;
      const letGo = hint('esc', 'esc', '');
      leadHtml = `<span class="chip chip--attached"><span class="icon">${icon(ic)}</span><span class="name" data-ink="label">${esc(label)}</span>${letGo}</span>`;
    } else if (v.chip === 'icon') {
      leadHtml = `<span class="chip chip--icon" title="${esc(FRONT.title)} · in ${FRONT.app}" data-typing="${S.text !== ''}">${icon(FRONT.icon)}</span>`;
    }
    if (!S.staged && S.text === '' && v.chip !== 'icon') {
      const name = v.chip === 'app' ? FRONT.app : FRONT.short;
      let use;
      if (v.learn) use = hint('space', '␣', 'Use');
      else use = `<span class="hint" data-ink="label">${S.grabs < USE_HINT_GRABS ? '<span class="word" data-ink="label">Use</span>' : ''}<span data-ink="key">${keys('␣', true)}</span></span>`;
      trailHtml = `<span class="chip"><span class="icon">${icon(FRONT.icon)}</span><span class="name" data-ink="label">${esc(name)}</span>${use}</span>`;
    }
    if (v.merged) {
      const compact = S.text.length > 26;
      const hintsD = lm.hints.map(([id, k, w, o]) => hint(id, k, w, o)).join('');
      const show = !(lm.kind === 'today' && S.text !== '');
      if (show) trailHtml += `<span class="status" data-kind="${lm.kind}" data-compact="${compact && lm.kind !== 'hold'}" style="opacity:${lm.kind === 'hold' ? 1 : S.text ? 0.6 : 0.9}"><span class="icon">${icon(lm.icon)}</span>${lm.state ? `<span class="state" data-ink="label">${esc(compact ? lm.state.replace('Paused while you type', 'Paused') : lm.state)}</span>` : ''}<span class="title" data-ink="label">${esc(lm.title)}</span>${hintsD}</span>`;
      const bar = $('#inprogress');
      bar.style.width = lm.kind === 'hold' ? `${(lm.progress * 100).toFixed(1)}%` : '0';
    }
    lead.innerHTML = leadHtml;
    trail.innerHTML = trailHtml;
    $('#mirror').innerHTML = marksFor(S.text);
    $('#mirror').scrollLeft = q.scrollLeft;

    // List
    const sections = listModel();
    rows = sections.flatMap((s) => s.rows);
    S.sel = Math.max(0, Math.min(S.sel, rows.length - 1));
    let i = 0, html = '';
    sections.forEach((s, si) => {
      let title = s.title, acc = '';
      if (s.jev) {
        const sh = show('jev');
        // While ⌘ is held the digits move onto Jev's rows, so the header doesn't repeat them.
        acc = (sh.keys && (!v.learn || (level('jev') === 0 && !S.cmd))) ? `<span data-ink="key">${keys('⌘1', true)}</span>` : '';
      } else if (s.acc && v.headers === 'caps') acc = `<span data-ink="label">${esc(s.acc)}</span>`;
      html += `<div class="hdr" data-style="${v.headers}"><span class="label" data-ink="label">${esc(title)}</span>${acc ? `<span class="acc">${acc}</span>` : ''}</div>`;
      s.rows.forEach((r) => {
        const selected = i === S.sel;
        let accHtml = '';
        if (r.jev && v.learn && S.cmd) accHtml = `<span class="rowkey reveal" data-ink="key">${keys('⌘' + r.jev, true)}</span>`;
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

    // Capsules
    const sel = rows[S.sel];
    const enterLabel = sel ? sel.enter : 'Ask';
    const enterKeys = show('enter', { always: !v.learn });
    const caps = [];
    if (v.today === 'footer') caps.push(`<span class="cap cap--left" data-cap="today"><span class="icon">${icon('calendar')}</span><span class="keys" data-ink="key">${keys('⌘T')}</span><span data-ink="label">Today</span></span><span class="spacer"></span>`);
    else caps.push('<span class="spacer"></span>');
    caps.push(`<span class="cap${enterKeys.keys ? '' : ' cap--keys-only'}" data-cap="enter"><span data-ink="label">${esc(enterLabel)}</span>${enterKeys.keys ? `<span class="keys" data-ink="key">${keys('↵')}</span>` : ''}</span>`);
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
  // Words and keycaps outside the list rows: the chrome you read before you look at results.
  function countInk() {
    const chrome = [...win.querySelectorAll('[data-ink]')].filter((el) => !el.closest('.row') && !el.closest('.hdr') && visible(el));
    const labels = chrome.filter((el) => el.dataset.ink === 'label' && !el.querySelector('[data-ink]')).length;
    const caps = win.querySelectorAll('.kc');
    const keyCount = [...caps].filter((k) => !k.closest('.row') && visible(k)).length;
    const headers = [...win.querySelectorAll('.hdr')].filter(visible).length;
    const n = (count, one, many) => `<b>${count}</b> ${count === 1 ? one : many}`;
    $('#ink').innerHTML = `Outside the rows right now: ${n(labels, 'piece of text', 'pieces of text')}, ${n(keyCount, 'keycap', 'keycaps')} and ${n(headers, 'section header', 'section headers')}.`;
  }

  function renderChrome() {
    document.querySelectorAll('#variants button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.v === S.variant)));
    document.querySelectorAll('#themes button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.theme === S.theme)));
    const size = S.size || V().size;
    document.querySelectorAll('#sizes button').forEach((b) => b.setAttribute('aria-checked', String(+b.dataset.size === size)));
    document.querySelectorAll('#fams button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.fam === S.fam)));
    $('#familiarity-control').hidden = !V().learn;
  }

  function renderNotes() {
    const v = V();
    const li = (xs, cls) => (xs.length ? `<ul class="${cls}">${xs.map((x) => `<li>${x}</li>`).join('')}</ul>` : '<p class="sum">Nothing.</p>');
    $('#notes').innerHTML = `
      <h2>${S.variant} · ${esc(v.name)}</h2>
      <p class="sum">${v.summary}</p>
      <h3>Where each thing lives</h3>
      <table class="homes">${v.homes.map(([a, b]) => `<tr><th>${a}</th><td>${b}</td></tr>`).join('')}</table>
      ${v.changes.length ? `<h3>Changes from the option before</h3>${li(v.changes, '')}` : ''}
      <h3>Keeps</h3>${li(v.keeps, 'keeps')}
      <h3>Gives up</h3>${li(v.breaks, 'breaks')}`;
  }

  function log(text) {
    S.log.unshift(text);
    S.log = S.log.slice(0, 4);
    $('#log').innerHTML = S.log.map((t, i) => `<li style="opacity:${1 - i * 0.2}">${t}</li>`).join('');
  }

  // ── Behavior ──
  function setText(t) { S.text = t; S.sel = 0; S.notice = null; render(); }
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
    if (r.action === 'undo') return undo('↵ on Undo it');
    const target = r.row || r;
    if (target.hold) {
      S.hold = { title: target.name, done: target.done, elapsed: 0 };
      S.staged = null; S.text = ''; S.sel = 0;
      log(`${how} → <span>${esc(target.name)}</span> waits 8 s before it runs`);
      return render();
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

  q.addEventListener('input', () => { S.text = q.value; S.sel = 0; S.notice = null; render(); });
  q.addEventListener('scroll', () => { $('#mirror').scrollLeft = q.scrollLeft; });
  q.addEventListener('keydown', (e) => {
    if (e.key === 'Meta') {
      clearTimeout(cmdTimer);
      cmdTimer = setTimeout(() => { S.cmd = true; render(); }, REVEAL_AFTER_MS);
      return;
    }
    if (e.metaKey) { clearTimeout(cmdTimer); }
    const empty = S.text === '';
    if (e.key === 'ArrowDown') { e.preventDefault(); S.sel = Math.min(rows.length - 1, S.sel + 1); return render(); }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (empty && !S.staged && S.sel === 0 && S.last) { used('up'); log('↑ → acting on the last run'); return stage({ kind: 'last' }); }
      S.sel = Math.max(0, S.sel - 1); return render();
    }
    if (e.key === 'Enter' && e.metaKey) {
      e.preventDefault(); used('agent');
      log(S.text ? `⌘↵ → <span>Do It</span> runs Jev’s first reading` : '⌘↵ → <span>Agent</span> opens the chat under this input');
      pressCap('agent');
      if (S.text) run(rows.find((r) => r.jev) || rows[0], '⌘↵');
      return render();
    }
    if (e.key === 'Enter') { e.preventDefault(); used('enter'); pressCap('enter'); return run(rows[S.sel], '↵'); }
    if (e.key === 'Escape') {
      e.preventDefault();
      if (S.staged) { used('esc'); log('Esc → let go'); return letGo(); }
      if (S.text) return setText('');
      return;
    }
    if (e.key === ' ' && empty && !S.staged) {
      e.preventDefault(); used('space'); S.grabs++;
      log(`Space → acting on <span>${esc(FRONT.short)}</span>`);
      return stage({ kind: 'window' });
    }
    if (e.metaKey && e.key.toLowerCase() === 'k') {
      e.preventDefault(); used('actions'); pressCap('actions');
      const r = rows[S.sel];
      if (!r || S.staged) return;
      log(`⌘K → acting on <span>${esc(r.name)}</span>`);
      return stage({ kind: 'row', row: r });
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
  q.addEventListener('keyup', (e) => {
    if (e.key === 'Meta') { clearTimeout(cmdTimer); if (S.cmd) { S.cmd = false; render(); } }
  });
  window.addEventListener('blur', () => { clearTimeout(cmdTimer); if (S.cmd) { S.cmd = false; render(); } });

  function pressCap(id) {
    const c = document.querySelector(`[data-cap="${id}"]`);
    if (!c) return;
    c.dataset.pressed = 'true';
    setTimeout(() => { c.dataset.pressed = 'false'; }, 140);
  }

  // Pointer is the fallback: click selects, double-click runs, capsules do what their keys do.
  win.addEventListener('mousedown', (e) => { if (e.target !== q) { e.preventDefault(); q.focus(); } });
  $('#list').addEventListener('click', (e) => { const r = e.target.closest('.row'); if (r) { S.sel = +r.dataset.i; render(); } });
  $('#list').addEventListener('dblclick', (e) => { const r = e.target.closest('.row'); if (r) run(rows[+r.dataset.i], 'Double-click'); });
  $('#foot').addEventListener('click', (e) => {
    const c = e.target.closest('[data-cap]');
    if (!c) return;
    const id = c.dataset.cap;
    if (id === 'enter') run(rows[S.sel], '↵ capsule');
    else if (id === 'actions' && rows[S.sel] && !S.staged) { used('actions'); stage({ kind: 'row', row: rows[S.sel] }); }
    else if (id === 'agent') log('⌘↵ capsule → <span>Agent</span> opens the chat under this input');
    else if (id === 'today') log('Today → opens today’s page under this input');
  });
  win.addEventListener('click', (e) => {
    const h = e.target.closest('[data-hint="today"]');
    if (h) { used('today'); log('⌘T → opens today’s page under this input'); }
  });

  // Hold countdown: runs only while the input is empty.
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
  $('#variants').innerHTML = Object.entries(VARIANTS).map(([k, v]) => `<button role="radio" data-v="${k}"><b>${k} · ${esc(v.name)}</b><span>${esc(v.blurb)}</span></button>`).join('');
  $('#variants').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.variant = b.dataset.v; S.cmd = false; syncHash(); renderNotes(); render(); q.focus(); } });
  $('#themes').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.theme = b.dataset.theme; syncHash(); render(); q.focus(); } });
  $('#sizes').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.size = +b.dataset.size; render(); q.focus(); } });
  $('#fams').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) { S.fam = b.dataset.fam; if (S.fam === 'track') S.uses = {}; render(); q.focus(); } });
  $('#states').addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b) return;
    S.staged = null; S.hold = null; S.notice = null; S.text = ''; S.sel = 0;
    const st = b.dataset.state;
    if (st === 'fresh') S.last = null;
    if (st === 'ran' || st === 'typing' || st === 'staged' || st === 'hold') S.last = S.last || { title: 'Added reminder: call mom', undo: true, at: Date.now() - 20000 };
    if (st === 'fresh') S.last = null;
    if (st === 'typing') S.text = 'remind me to call mom tomorrow at 5pm';
    if (st === 'staged') { S.savedBeforeStage = { text: '', sel: 0 }; S.staged = { kind: 'window' }; }
    if (st === 'hold') S.hold = { title: 'Empty the Trash', done: 'Emptied the Trash', elapsed: 1500 };
    log(`Jumped to <span>${esc(b.textContent)}</span>`);
    render(); q.focus();
  });

  function syncHash() { history.replaceState(null, '', `#${S.variant}-${S.theme}`); }
  const m = /^#([ABCD])-(dark|light)$/.exec(location.hash);
  if (m) { S.variant = m[1]; S.theme = m[2]; }

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
    state: () => JSON.parse(JSON.stringify({ ...S, log: undefined, rows: rows.map((r) => r.name) })),
    set: (patch) => { Object.assign(S, patch); renderNotes(); render(); },
    variants: Object.keys(VARIANTS),
  };

  fit(); renderNotes(); render(); q.focus();
})();

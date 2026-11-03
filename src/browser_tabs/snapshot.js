// Executed by the macOS automation adapter, or with a browser object model in tests.
// Cached window/tab *positions* are display metadata, never authority to focus a tab.
(function browserSnapshot(app, bundle, name, safari, processId, target, frontProcessId) {
    const listing = target === null;
    const query = typeof target === 'string' ? target.trim().toLowerCase() : null;
    const owner = processId();
    if (!owner || !app.running()) return listing ? '[]' : 'not-running';
    if (query === '') return 'not-found';
    const text = x => x == null ? '' : String(x);
    const same = (a, b) => a.length === b.length && a.every((x, i) => x === b[i]);
    const titles = win => safari ? win.tabs.name() : win.tabs.title();
    const title = tab => text(safari ? tab.name() : tab.title());
    const windowIds = () => app.windows().map(win => win.id());
    if (listing || query !== null) {
        const rows = [], captured = windowIds();
        for (const [w, wid] of captured.entries()) {
            if (!text(wid)) throw Error('Missing browser window identity');
            const win = app.windows.byId(wid);
            const ids = safari ? null : win.tabs.id().map(text);
            const names = titles(win).map(text), urls = win.tabs.url().map(text);
            if (names.length !== urls.length || (ids && (ids.length !== names.length || !same(ids, win.tabs.id().map(text))))) {
                // A partial list is useful for browsing, never for deciding a name is unique.
                if (!listing) return 'changed';
                continue;
            }
            const pageCounts = new Map();
            if (safari) for (let i = 0; i < urls.length; i++) {
                const key = JSON.stringify([names[i], urls[i]]);
                pageCounts.set(key, (pageCounts.get(key) || 0) + 1);
            }
            for (let i = 0; i < urls.length; i++) {
                if (!names[i] && !urls[i]) continue;
                if (ids && !ids[i]) {
                    if (!listing) return 'changed';
                    continue;
                }
                rows.push({browser_name:name, browser_bundle_id:bundle, window_index:w+1, tab_index:i+1,
                    title:names[i], url:urls[i], identity:safari && (!urls[i] || pageCounts.get(JSON.stringify([names[i],urls[i]])) !== 1)
                        ? null : {pid:owner, window:text(wid), tab:ids ? ids[i] : null}});
            }
        }
        if (processId() !== owner) {
            if (!listing) return 'changed';
            throw Error('Browser changed during enumeration');
        }
        if (listing) return JSON.stringify(rows);
        if (!same(captured.slice().sort(), windowIds().sort())) return 'changed';
        let best = 0, match = null, tied = false;
        for (const row of rows) {
            const title = row.title.toLowerCase(), url = row.url.toLowerCase();
            const host = /^https?:\/\/([^/?#]+)/.exec(url)?.[1] || '';
            const score = title === query || host === query || url === query ? 2 : title.includes(query) || host.includes(query) ? 1 : 0;
            if (score > best) { best = score; match = row; tied = false; }
            else if (score && score === best) tied = true;
        }
        if (!match) return 'not-found';
        if (tied) return 'ambiguous';
        target = match;
    }
    if (!target.identity || owner !== target.identity.pid || target.browser_bundle_id !== bundle) return 'changed';
    const windows = windowIds().filter(id => text(id) === target.identity.window);
    if (windows.length !== 1) return 'changed';
    const win = app.windows.byId(windows[0]);
    const matchesTab = tab => !!tab && (safari || text(tab.id()) === target.identity.tab) &&
        text(tab.url()) === target.url && title(tab) === target.title;
    let positions = [];
    if (safari) {
        // Safari exposes a window ID, but no tab ID. Never choose the first duplicate.
        if (!target.url || target.identity.tab !== null) return 'changed';
        const names = titles(win), urls = win.tabs.url();
        if (names.length !== urls.length) return 'changed';
        for (let i = 0; i < urls.length; i++) if (text(urls[i]) === target.url && text(names[i]) === target.title) positions.push(i);
    } else {
        if (!target.identity.tab) return 'changed';
        win.tabs.id().forEach((id, i) => { if (text(id) === target.identity.tab) positions.push(i); });
    }
    if (positions.length !== 1) return positions.length ? 'ambiguous' : 'changed';
    const i = positions[0], tab = win.tabs[i];
    if (!matchesTab(tab) || text(win.id()) !== target.identity.window || processId() !== owner) return 'changed';
    if (safari) win.currentTab = tab; else win.activeTabIndex = i + 1;
    const current = w => safari ? w.currentTab() : w.tabs[w.activeTabIndex() - 1];
    const confirmed = w => w && text(w.id()) === target.identity.window && matchesTab(current(w)) && processId() === owner;
    if (!confirmed(win)) return 'changed';
    win.index = 1;
    if (!confirmed(app.windows()[0])) return 'changed';
    app.activate();
    if (!confirmed(app.windows()[0])) return 'changed';
    return frontProcessId() === owner ? 'focused' : 'unconfirmed';
})

// Executed by the macOS automation adapter, or with a browser object model in tests.
// Cached window/tab *positions* are display metadata, never authority to focus a tab.
(function browserSnapshot(app, bundle, name, safari, processId, target) {
    const owner = processId();
    if (!owner || !app.running()) return target ? 'not-running' : '[]';
    const text = x => x == null ? '' : String(x);
    const same = (a, b) => a.length === b.length && a.every((x, i) => x === b[i]);
    const titles = win => safari ? win.tabs.name() : win.tabs.title();
    const title = tab => text(safari ? tab.name() : tab.title());
    if (!target) {
        const rows = [];
        for (const [w, window] of app.windows().entries()) {
            const wid = window.id();
            if (!text(wid)) throw Error('Missing browser window identity');
            const win = app.windows.byId(wid);
            const ids = safari ? null : win.tabs.id().map(text);
            const names = titles(win).map(text), urls = win.tabs.url().map(text);
            if (names.length !== urls.length || (ids && (ids.length !== names.length || !same(ids, win.tabs.id().map(text))))) continue;
            const pageCounts = new Map();
            if (safari) for (let i = 0; i < urls.length; i++) {
                const key = JSON.stringify([names[i], urls[i]]);
                pageCounts.set(key, (pageCounts.get(key) || 0) + 1);
            }
            for (let i = 0; i < urls.length; i++) {
                if ((!names[i] && !urls[i]) || (ids && !ids[i])) continue;
                rows.push({browser_name:name, browser_bundle_id:bundle, window_index:w+1, tab_index:i+1,
                    title:names[i], url:urls[i], identity:safari && (!urls[i] || pageCounts.get(JSON.stringify([names[i],urls[i]])) !== 1)
                        ? null : {pid:owner, window:text(wid), tab:ids ? ids[i] : null}});
            }
        }
        if (processId() !== owner) throw Error('Browser changed during enumeration');
        return JSON.stringify(rows);
    }
    if (!target.identity || owner !== target.identity.pid || target.browser_bundle_id !== bundle) return 'changed';
    const windows = app.windows().filter(w => text(w.id()) === target.identity.window);
    if (windows.length !== 1) return 'changed';
    const win = app.windows.byId(windows[0].id());
    const matchesTab = tab => !!tab && (safari || text(tab.id()) === target.identity.tab) &&
        text(tab.url()) === target.url && title(tab) === target.title;
    let positions = [];
    if (safari) {
        // Safari exposes a window ID, but no tab ID. Never choose the first duplicate.
        if (!target.url || target.identity.tab !== null) return 'changed';
        const names = titles(win), urls = win.tabs.url();
        for (let i = 0; i < urls.length; i++) if (text(urls[i]) === target.url && text(names[i]) === target.title) positions.push(i);
    } else {
        if (!target.identity.tab) return 'changed';
        win.tabs.id().forEach((id, i) => { if (text(id) === target.identity.tab) positions.push(i); });
    }
    if (positions.length !== 1) return positions.length ? 'ambiguous' : 'changed';
    const i = positions[0], tab = win.tabs[i];
    if (processId() !== owner || !matchesTab(tab)) return 'changed';
    if (safari) win.currentTab = tab; else win.activeTabIndex = i + 1;
    const current = w => safari ? w.currentTab() : w.tabs[w.activeTabIndex() - 1];
    if (processId() !== owner || !matchesTab(current(win))) return 'changed';
    win.index = 1;
    app.activate();
    const front = app.windows()[0];
    return processId() === owner && front && text(front.id()) === target.identity.window && matchesTab(current(front)) ? 'focused' : 'changed';
})

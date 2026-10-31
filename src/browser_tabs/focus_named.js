// Invoked by the macOS adapter with one supported browser. No title or URL leaves this process.
(function focusNamedTab(app, query, safari) {
    if (!app.running()) return 'not-running';
    const q = query.trim().toLowerCase();
    if (!q) return 'not-found';
    let best = 0, matches = [];
    for (const win of app.windows()) {
        const titles = safari ? win.tabs.name() : win.tabs.title();
        const urls = win.tabs.url();
        for (let i = 0; i < titles.length; i++) {
            const title = (titles[i] || '').toLowerCase(), url = (urls[i] || '').toLowerCase();
            const host = (url.match(/^https?:\/\/([^/?#]+)/) || [,''])[1];
            const score = (title === q || host === q || url === q) ? 2 : (title.includes(q) || host.includes(q)) ? 1 : 0;
            if (score && score >= best) {
                if (score > best) matches = [];
                best = score;
                matches.push({win, i, title: titles[i], url: urls[i]});
            }
        }
    }
    if (!matches.length) return 'not-found';
    if (matches.length !== 1) return 'ambiguous';
    const {win,i,title,url} = matches[0];
    const tab = win.tabs[i];
    if (tab.url() !== url || (safari ? tab.name() : tab.title()) !== title) return 'changed';
    if (safari) win.currentTab = tab; else win.activeTabIndex = i + 1;
    win.index = 1;
    app.activate();
    const front = app.windows()[0];
    const selected = safari ? front.currentTab() : front.tabs[front.activeTabIndex() - 1];
    return selected.url() === url && (safari ? selected.name() : selected.title()) === title ? 'focused' : 'changed';
})

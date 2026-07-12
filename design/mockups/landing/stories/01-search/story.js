/* One search box. Everything on your Mac. — landing walkthrough */
(function () {
  "use strict";
  var story = {"id": "01-search", "durationMs": 5000, "loop": true, "surfaces": [{"id": "main-menu", "initial": true}], "chapters": [{"id": "rest", "label": "Launcher rest", "at": 0}, {"id": "type", "label": "Type clip", "at": 400}, {"id": "filter", "label": "Results narrow", "at": 1800}, {"id": "select", "label": "Select Clipboard History", "at": 2800}], "actions": [{"at": 0, "kind": "ensureRows", "surface": "main-menu", "rows": [{"name": "Clipboard History", "desc": "builtin \u00b7 searchable paste library"}, {"name": "Clip Tools", "desc": "clipboard utilities"}]}, {"at": 0, "kind": "setSelection", "surface": "main-menu", "index": 0}, {"at": 0, "kind": "setFooterState", "surface": "main-menu", "footer": {"runLabel": "Run", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "agentLabel": "Agent", "agentKeys": ["\u2318", "\u21b5"], "selected": "run"}}, {"at": 400, "duration": 1600, "kind": "type", "surface": "main-menu", "text": "clip", "as": "filter"}, {"at": 2800, "kind": "setSelection", "surface": "main-menu", "index": 0}, {"at": 2800, "kind": "setFooterState", "surface": "main-menu", "footer": {"runLabel": "Open", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "agentLabel": "Agent", "agentKeys": ["\u2318", "\u21b5"], "selected": "run"}}]};
  var params = new URLSearchParams(typeof location !== "undefined" ? location.search : "");
  var autoplay = params.get("autoplay") !== "0";
  var t = params.get("t");
  var api = window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: autoplay,
  });
  if (t != null && api && api.seek) {
    api.pause();
    api.seek(Number(t) || 0);
  }
})();

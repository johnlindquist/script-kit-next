/* Everything you copied, searchable and previewed. — landing walkthrough */
(function () {
  "use strict";
  var story = {"id": "03-clipboard", "durationMs": 5500, "loop": true, "surfaces": [{"id": "clipboard-history", "initial": true}], "chapters": [{"id": "open", "label": "Open clipboard", "at": 0}, {"id": "walk", "label": "Walk entries", "at": 600}, {"id": "pin", "label": "Select pinned", "at": 2800}, {"id": "preview", "label": "Preview ready", "at": 3800}], "actions": [{"at": 0, "kind": "setSelection", "surface": "clipboard-history", "index": 0, "preview": "Design tokens stay in sync with the Rust renderer."}, {"at": 0, "kind": "setFooterState", "surface": "clipboard-history", "footer": {"runLabel": "Paste", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "agentLabel": "Agent", "agentKeys": ["\u2318", "\u21b5"], "selected": "run"}}, {"at": 600, "duration": 2000, "kind": "walkSelection", "surface": "clipboard-history", "from": 0, "to": 4, "previews": ["Design tokens stay in sync with the Rust renderer.", "Meeting notes \u2014 Q3 launch", "https://scriptkit.com/downloads", "const answer = 42;", "npm install -g mdflow@next"]}, {"at": 2800, "kind": "setSelection", "surface": "clipboard-history", "index": 4, "preview": "npm install -g mdflow@next"}, {"at": 3800, "kind": "setSelection", "surface": "clipboard-history", "index": 4, "preview": "npm install -g mdflow@next\n\n# pinned fixture\nready to paste"}]};
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

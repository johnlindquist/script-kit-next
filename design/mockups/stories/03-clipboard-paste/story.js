/* Paste from Clipboard History — continuous timeline story */
(function () {
  "use strict";
  var story = {"id": "03-clipboard-paste", "durationMs": 6500, "loop": true, "surfaces": [{"id": "clipboard-history", "initial": true}], "chapters": [{"id": "open", "label": "Open clipboard", "at": 0}, {"id": "walk", "label": "Walk entries", "at": 600}, {"id": "pin", "label": "Select pin", "at": 3200}, {"id": "preview", "label": "Preview detail", "at": 4000}, {"id": "paste", "label": "Paste \u21b5", "at": 5200}], "actions": [{"at": 0, "kind": "setSelection", "surface": "clipboard-history", "index": 0, "preview": "Design tokens stay in sync with the Rust renderer."}, {"at": 0, "kind": "setFooterState", "surface": "clipboard-history", "footer": {"runLabel": "Paste", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "agentLabel": "Agent", "agentKeys": ["\u2318", "\u21b5"], "selected": "run"}}, {"at": 600, "duration": 2400, "kind": "walkSelection", "surface": "clipboard-history", "from": 0, "to": 4, "previews": ["Design tokens stay in sync with the Rust renderer.", "Meeting notes \u2014 Q3 launch\n- pixel-perfect mockups", "https://scriptkit.com/downloads", "const answer = 42;", "npm install -g mdflow@next"]}, {"at": 3200, "kind": "setSelection", "surface": "clipboard-history", "index": 4, "preview": "npm install -g mdflow@next"}, {"at": 4000, "kind": "setSelection", "surface": "clipboard-history", "index": 4, "preview": "npm install -g mdflow@next\n\n# pinned fixture\nmdflow@next design pipeline"}, {"at": 5200, "kind": "setFooterState", "surface": "clipboard-history", "footer": {"runLabel": "Pasted", "runKeys": ["\u2713"], "selected": "run"}}]};
  window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: true,
  });
})();

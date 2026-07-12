/* Every command is one ⌘K away. — landing walkthrough */
(function () {
  "use strict";
  var story = {"id": "02-actions-confirm", "durationMs": 6000, "loop": true, "surfaces": [{"id": "main-menu", "initial": true}, {"id": "actions-dialog", "initial": false}, {"id": "confirm-popup", "initial": false}], "chapters": [{"id": "focus", "label": "Focus a row", "at": 0}, {"id": "actions", "label": "Open Actions \u2318K", "at": 800}, {"id": "filter", "label": "Type del", "at": 1600}, {"id": "confirm", "label": "Confirm danger", "at": 3600}], "actions": [{"at": 0, "kind": "setSelection", "surface": "main-menu", "index": 4}, {"at": 0, "kind": "setFooterState", "surface": "main-menu", "footer": {"runLabel": "Run", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "agentLabel": "Agent", "agentKeys": ["\u2318", "\u21b5"]}}, {"at": 800, "kind": "openOverlay", "surface": "actions-dialog"}, {"at": 800, "kind": "setSelection", "surface": "actions-dialog", "index": 0}, {"at": 1600, "duration": 1200, "kind": "type", "surface": "actions-dialog", "text": "del", "as": "filter"}, {"at": 3000, "kind": "setSelection", "surface": "actions-dialog", "index": 0}, {"at": 3600, "kind": "closeOverlay", "surface": "actions-dialog"}, {"at": 3600, "kind": "hideSurface", "surface": "main-menu"}, {"at": 3600, "kind": "showSurface", "surface": "confirm-popup"}, {"at": 3600, "kind": "setFooterState", "surface": "confirm-popup", "footer": {"runLabel": "Delete", "runKeys": ["\u21b5"], "actionsLabel": "Keep", "actionsKeys": ["Esc"], "hideAgent": true, "selected": "run"}}]};
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

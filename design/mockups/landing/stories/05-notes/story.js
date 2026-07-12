/* Scratch notes that stay out of the way. — landing walkthrough */
(function () {
  "use strict";
  var story = {"id": "05-notes", "durationMs": 7000, "loop": true, "surfaces": [{"id": "notes", "initial": true}], "chapters": [{"id": "open", "label": "Open Notes", "at": 0}, {"id": "type", "label": "Type body", "at": 400}, {"id": "list", "label": "List complete", "at": 3400}, {"id": "rest", "label": "Saved rest", "at": 5600}], "actions": [{"at": 0, "kind": "setText", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\n"}, {"at": 400, "duration": 2600, "kind": "type", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\nTrack every painted value in\nthe Notes window.\n"}, {"at": 3400, "duration": 2000, "kind": "type", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\nTrack every painted value in\nthe Notes window.\n\n- Titlebar 36 px\n- Editor 16 px mono\n- Footer buttons hug\n"}, {"at": 5600, "kind": "setText", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\nTrack every painted value in\nthe Notes window.\n\n- Titlebar 36 px\n- Editor 16 px mono\n- Footer buttons hug\n"}]};
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

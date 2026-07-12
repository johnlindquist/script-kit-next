/* Scratch Notes Panel — continuous timeline story */
(function () {
  "use strict";
  var story = {"id": "05-notes-scratchpad", "durationMs": 7500, "loop": true, "surfaces": [{"id": "notes", "initial": true}], "chapters": [{"id": "open", "label": "Open Notes", "at": 0}, {"id": "type", "label": "Type markdown", "at": 400}, {"id": "list", "label": "Add list", "at": 3800}, {"id": "footer", "label": "Footer rest", "at": 6200}], "actions": [{"at": 0, "kind": "setText", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\n"}, {"at": 400, "duration": 3000, "kind": "type", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\nTrack every painted value in\nthe Notes window.\n"}, {"at": 3800, "duration": 2200, "kind": "type", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\nTrack every painted value in\nthe Notes window.\n\n- Titlebar 36 px\n- Editor 16 px mono, 20 px line\n- Footer buttons hug\n"}, {"at": 6200, "kind": "setText", "surface": "notes", "as": "notes", "text": "# Design Contract Notes\n\nTrack every painted value in\nthe Notes window.\n\n- Titlebar 36 px\n- Editor 16 px mono, 20 px line\n- Footer buttons hug\n"}]};
  window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: true,
  });
})();

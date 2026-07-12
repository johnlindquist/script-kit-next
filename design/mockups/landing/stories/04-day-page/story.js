/* Your day, as a living Markdown file. — landing walkthrough */
(function () {
  "use strict";
  var story = {"id": "04-day-page", "durationMs": 6500, "loop": true, "surfaces": [{"id": "day-page", "initial": true}], "chapters": [{"id": "open", "label": "Day Page open", "at": 0}, {"id": "type", "label": "Type spine line", "at": 500}, {"id": "task", "label": "Task present", "at": 3200}, {"id": "rest", "label": "Collapsed shelf rest", "at": 5200}], "actions": [{"at": 0, "kind": "setLines", "surface": "day-page", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list"]}, {"at": 0, "kind": "setFooterState", "surface": "day-page", "footer": {"runLabel": "Actions", "runKeys": ["\u2318", "K"], "hideAgent": true}}, {"at": 500, "duration": 2400, "kind": "setLines", "surface": "day-page", "mode": "typeLast", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list", "09:31 - [ ] wire day-page tokens into export_design_tokens #design"]}, {"at": 3200, "duration": 1800, "kind": "setLines", "surface": "day-page", "mode": "typeLast", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list", "09:31 - [ ] wire day-page tokens into export_design_tokens #design", "10:02 Script Kit landing refresh notes"]}, {"at": 5200, "kind": "setLines", "surface": "day-page", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list", "09:31 - [ ] wire day-page tokens into export_design_tokens #design", "10:02 Script Kit landing refresh notes"]}]};
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

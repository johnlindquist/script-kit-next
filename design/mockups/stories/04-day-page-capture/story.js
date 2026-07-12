/* Capture a Day Page Note — continuous timeline story */
(function () {
  "use strict";
  var story = {"id": "04-day-page-capture", "durationMs": 7000, "loop": true, "surfaces": [{"id": "day-page", "initial": true}], "chapters": [{"id": "open", "label": "Day Page open", "at": 0}, {"id": "type", "label": "Type spine line", "at": 500}, {"id": "task", "label": "Add task line", "at": 3200}, {"id": "rest", "label": "Rest state", "at": 5600}], "actions": [{"at": 0, "kind": "setLines", "surface": "day-page", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list"]}, {"at": 0, "kind": "setFooterState", "surface": "day-page", "footer": {"runLabel": "Actions", "runKeys": ["\u2318", "K"], "hideAgent": true}}, {"at": 500, "duration": 2400, "kind": "setLines", "surface": "day-page", "mode": "typeLast", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list", "09:31 - [ ] wire day-page tokens into export_design_tokens #design"]}, {"at": 3200, "duration": 2000, "kind": "setLines", "surface": "day-page", "mode": "typeLast", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list", "09:31 - [ ] wire day-page tokens into export_design_tokens #design", "10:02 Script Kit landing refresh notes"]}, {"at": 5600, "kind": "setLines", "surface": "day-page", "lines": ["# Friday \u00b7 ship the Day Page mockup", "09:12 sketched the Day Page fixture and token list", "09:31 - [ ] wire day-page tokens into export_design_tokens #design", "10:02 Script Kit landing refresh notes", "09:47 Clipboard entry"]}]};
  window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: true,
  });
})();

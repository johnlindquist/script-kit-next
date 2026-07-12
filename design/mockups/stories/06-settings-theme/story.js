/* Browse Settings Hub — continuous timeline story */
(function () {
  "use strict";
  var story = {
    id: "06-settings-theme",
    durationMs: 6500,
    loop: true,
    surfaces: [{ id: "settings", initial: true }],
    chapters: [
      { id: "open", label: "Open Settings", at: 0 },
      { id: "type", label: "Type theme", at: 600 },
      { id: "filter", label: "List filters live", at: 2400 },
      { id: "select", label: "Theme Designer", at: 3200 },
      { id: "ready", label: "Open ↵", at: 4500 }
    ],
    actions: [
      { at: 0, kind: "setSelection", surface: "settings", index: 0 },
      {
        at: 0,
        kind: "setFooterState",
        surface: "settings",
        footer: {
          runLabel: "Open",
          runKeys: ["↵"],
          actionsLabel: "Actions",
          actionsKeys: ["⌘", "K"],
          hideAgent: true,
          selected: "run"
        }
      },
      {
        at: 600,
        duration: 1800,
        kind: "type",
        surface: "settings",
        text: "theme",
        as: "filter"
      },
      { at: 3200, kind: "setSelection", surface: "settings", index: 0 },
      {
        at: 4500,
        kind: "setFooterState",
        surface: "settings",
        footer: {
          runLabel: "Open",
          runKeys: ["↵"],
          selected: "run",
          hideAgent: true
        }
      }
    ]
  };
  window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: true
  });
})();

/* Run a Script with Choices — continuous timeline story */
(function () {
  "use strict";
  var story = {
    id: "01-run-script-with-arg",
    durationMs: 9000,
    loop: true,
    surfaces: [
      { id: "main-menu", initial: true },
      { id: "arg-prompt", initial: false }
    ],
    chapters: [
      { id: "rest", label: "Launcher rest", at: 0 },
      { id: "type", label: "Type to filter", at: 500 },
      { id: "narrow", label: "List narrows live", at: 2200 },
      { id: "select", label: "Select Fruit Picker", at: 3000 },
      { id: "arg", label: "Arg prompt", at: 4000 },
      { id: "pick", label: "Pick fruit", at: 5000 },
      { id: "run", label: "Run", at: 7200 }
    ],
    actions: [
      {
        at: 0,
        kind: "ensureRows",
        surface: "main-menu",
        rows: [
          { name: "Fruit Picker", desc: "arg prompt demo · pick a fruit" },
          { name: "Framework Docs", desc: "open kit docs" }
        ]
      },
      {
        at: 0,
        kind: "setFooterState",
        surface: "main-menu",
        footer: {
          runLabel: "Converse",
          runKeys: ["↵"],
          actionsLabel: "Actions",
          actionsKeys: ["⌘", "K"],
          agentLabel: "Agent",
          agentKeys: ["⌘", "↵"],
          selected: "run"
        }
      },
      { at: 0, kind: "setSelection", surface: "main-menu", index: 0 },
      {
        at: 500,
        duration: 2000,
        kind: "type",
        surface: "main-menu",
        text: "fruit",
        as: "filter"
      },
      { at: 2600, kind: "setSelection", surface: "main-menu", index: 0 },
      {
        at: 3000,
        kind: "setFooterState",
        surface: "main-menu",
        footer: {
          runLabel: "Open",
          runKeys: ["↵"],
          actionsLabel: "Actions",
          actionsKeys: ["⌘", "K"],
          agentLabel: "Agent",
          agentKeys: ["⌘", "↵"],
          selected: "run"
        }
      },
      { at: 4000, kind: "hideSurface", surface: "main-menu" },
      { at: 4000, kind: "showSurface", surface: "arg-prompt" },
      { at: 4000, kind: "setSelection", surface: "arg-prompt", index: 0 },
      {
        at: 4000,
        kind: "setFooterState",
        surface: "arg-prompt",
        footer: {
          runLabel: "Run",
          runKeys: ["↵"],
          actionsLabel: "Actions",
          actionsKeys: ["⌘", "K"],
          hideAgent: true,
          selected: "run"
        }
      },
      {
        at: 5000,
        duration: 1800,
        kind: "walkSelection",
        surface: "arg-prompt",
        from: 0,
        to: 2
      },
      {
        at: 7200,
        kind: "setFooterState",
        surface: "arg-prompt",
        footer: {
          runLabel: "Running",
          runKeys: ["…"],
          hideAgent: true,
          selected: "run"
        }
      },
      { at: 7200, kind: "pressKey", surface: "arg-prompt", key: "Enter" }
    ]
  };
  window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: true
  });
})();

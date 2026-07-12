/* SDK Chat Follow-up — continuous timeline story */
(function () {
  "use strict";
  var story = {"id": "07-chat-follow-up", "durationMs": 7000, "loop": true, "surfaces": [{"id": "chat-prompt", "initial": true}], "chapters": [{"id": "open", "label": "Open chat", "at": 0}, {"id": "type", "label": "Type follow-up", "at": 800}, {"id": "send", "label": "Send", "at": 3200}, {"id": "stream", "label": "Stream reply", "at": 3600}], "actions": [{"at": 0, "kind": "setFooterState", "surface": "chat-prompt", "footer": {"runLabel": "Run", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "hideAgent": true}}, {"at": 800, "duration": 2000, "kind": "type", "surface": "chat-prompt", "text": "Can I write back too?", "as": "composer"}, {"at": 3200, "kind": "setFooterState", "surface": "chat-prompt", "footer": {"runLabel": "Send", "runKeys": ["\u21b5"], "selected": "run", "hideAgent": true}}, {"at": 3200, "kind": "appendMessage", "surface": "chat-prompt", "role": "user", "text": "Can I write back too?", "msgId": "u2"}, {"at": 3400, "kind": "setText", "surface": "chat-prompt", "as": "composer", "text": ""}, {"at": 3600, "kind": "appendMessage", "surface": "chat-prompt", "role": "assistant", "text": "", "msgId": "a2"}, {"at": 3600, "duration": 2800, "kind": "streamText", "surface": "chat-prompt", "msgId": "a2", "text": "Yes \u2014 call clipboard.writeText(...) to write back."}]};
  window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: true,
  });
})();

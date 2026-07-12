/* Handoff to Agent Chat — continuous timeline story */
(function () {
  "use strict";
  var story = {"id": "09-agent-chat-handoff", "durationMs": 7500, "loop": true, "surfaces": [{"id": "agent-chat", "initial": true}], "chapters": [{"id": "open", "label": "Open Agent Chat", "at": 0}, {"id": "type", "label": "Type follow-up", "at": 800}, {"id": "arm", "label": "Send armed", "at": 3000}, {"id": "stream", "label": "Stream reply", "at": 3600}], "actions": [{"at": 0, "kind": "setFooterState", "surface": "agent-chat", "footer": {"runLabel": "Paste Response", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "hideAgent": true}}, {"at": 0, "kind": "setSendState", "surface": "agent-chat", "value": "disabled"}, {"at": 800, "duration": 2000, "kind": "type", "surface": "agent-chat", "text": "Retry the bash step with a shorter cwd", "as": "composer"}, {"at": 2800, "kind": "setSendState", "surface": "agent-chat", "value": "enabled"}, {"at": 3000, "kind": "setFooterState", "surface": "agent-chat", "footer": {"runLabel": "Send", "runKeys": ["\u21b5"], "actionsLabel": "Actions", "actionsKeys": ["\u2318", "K"], "hideAgent": true, "selected": "run"}}, {"at": 3400, "kind": "appendMessage", "surface": "agent-chat", "role": "user", "text": "Retry the bash step with a shorter cwd", "msgId": "u-retry"}, {"at": 3500, "kind": "setText", "surface": "agent-chat", "as": "composer", "text": ""}, {"at": 3500, "kind": "setSendState", "surface": "agent-chat", "value": "disabled"}, {"at": 3600, "kind": "appendMessage", "surface": "agent-chat", "role": "assistant", "text": "", "msgId": "a-retry"}, {"at": 3600, "duration": 3000, "kind": "streamText", "surface": "agent-chat", "msgId": "a-retry", "text": "Queued. Shorten the workspace path and re-run the bash tool \u2014 exit 1 was path length, not the diff."}]};
  window.StoryPlayer.mount({
    root: document.querySelector("[data-story-root]") || document.body,
    story: story,
    autoplay: true,
  });
})();

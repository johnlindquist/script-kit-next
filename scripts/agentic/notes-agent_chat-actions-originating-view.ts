#!/usr/bin/env bun
import { readFileSync } from "fs";
import { assert, NotesAgentChatHarness } from "./notes-agent_chat-direct-harness";

const harness = new NotesAgentChatHarness("notes-agent_chat-actions-originating-view");

try {
  const source = readFileSync("src/notes/window/agent_chat_host.rs", "utf8");
  const refreshIndex = source.indexOf("thread.refresh_models(cx)");
  const contextIndex = source.indexOf("AgentChatActionsDialogContext");
  assert(refreshIndex >= 0, "toggle_agent_chat_actions must refresh models");
  assert(contextIndex > refreshIndex, "model refresh must happen before dialog context snapshot");
  assert(source.includes("let actions_target = agent_chat_view.downgrade();"), "actions popup must capture originating view");
  assert(source.includes("notes_agent_chat_action_stale_view"), "dispatch must ignore stale originating views");
  assert(source.includes("notes_agent_chat_generation"), "dispatch must compare Notes Agent Chat generation");
  assert(
    !source
      .split("fn dispatch_notes_agent_chat_action")
      .at(1)!
      .includes("embedded_agent_chat.clone()"),
    "dispatch must not retarget actions through the current embedded_agent_chat cache"
  );

  await harness.ready();
  await harness.openNotes();
  await harness.gpuiKey("notes-actions-open-agent_chat", "a", ["cmd", "shift"]);
  await harness.send({
    type: "simulateGpuiEvent",
    requestId: "notes-actions-open-popup",
    target: { type: "kind", kind: "notes", index: 0 },
    event: { type: "keyDown", key: "k", modifiers: ["cmd"] },
  });
  await harness.waitForLog(
    (line) => line.includes("event=agent_chat_refresh_models_requested"),
    4_000,
    "model refresh before actions popup"
  );

  console.log(
    JSON.stringify({
      status: "pass",
      scenario: "notes-agent_chat-actions-originating-view",
      modelRefreshReceipts: harness.countLogs("event=agent_chat_refresh_models_requested"),
    })
  );
} finally {
  await harness.cleanup();
}

#!/usr/bin/env bun
import { readFileSync } from "fs";
import { assert, NotesAcpHarness } from "./notes-acp-direct-harness";

const harness = new NotesAcpHarness("notes-acp-actions-originating-view");

try {
  const source = readFileSync("src/notes/window/acp_host.rs", "utf8");
  const refreshIndex = source.indexOf("thread.refresh_models(cx)");
  const contextIndex = source.indexOf("AcpActionsDialogContext");
  assert(refreshIndex >= 0, "toggle_acp_actions must refresh models");
  assert(contextIndex > refreshIndex, "model refresh must happen before dialog context snapshot");
  assert(source.includes("let actions_target = acp_view.downgrade();"), "actions popup must capture originating view");
  assert(source.includes("notes_acp_action_stale_view"), "dispatch must ignore stale originating views");
  assert(source.includes("notes_acp_generation"), "dispatch must compare Notes ACP generation");
  assert(
    !source
      .split("fn dispatch_notes_acp_action")
      .at(1)!
      .includes("embedded_acp_chat.clone()"),
    "dispatch must not retarget actions through the current embedded_acp_chat cache"
  );

  await harness.ready();
  await harness.openNotes();
  await harness.gpuiKey("notes-actions-open-acp", "a", ["cmd", "shift"]);
  await harness.send({
    type: "simulateGpuiEvent",
    requestId: "notes-actions-open-popup",
    target: { type: "kind", kind: "notes", index: 0 },
    event: { type: "keyDown", key: "k", modifiers: ["cmd"] },
  });
  await harness.waitForLog(
    (line) => line.includes("event=acp_refresh_models_requested"),
    4_000,
    "model refresh before actions popup"
  );

  console.log(
    JSON.stringify({
      status: "pass",
      scenario: "notes-acp-actions-originating-view",
      modelRefreshReceipts: harness.countLogs("event=acp_refresh_models_requested"),
    })
  );
} finally {
  await harness.cleanup();
}

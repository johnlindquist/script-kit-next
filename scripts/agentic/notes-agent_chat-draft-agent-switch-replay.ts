#!/usr/bin/env bun
import { readFileSync } from "fs";
import { assert, NotesAgentChatHarness } from "./notes-agent_chat-direct-harness";

const harness = new NotesAgentChatHarness("notes-agent_chat-draft-agent-switch-replay");
const draft = "  draft with spaces  ";

try {
  const agent_chatHostSource = readFileSync("src/notes/window/agent_chat_host.rs", "utf8");
  const threadSource = readFileSync("src/ai/agent_chat/ui/thread.rs", "utf8");
  assert(
    agent_chatHostSource.includes("capture_draft_snapshot(cx)") &&
      agent_chatHostSource.includes("restore_draft_snapshot(snapshot, cx)") &&
      !agent_chatHostSource.includes("thread.input.text().trim().to_string()"),
    "agent switch source must capture/restore the full draft without trimming"
  );
  assert(
    threadSource.includes("transcript_generation: u64") &&
      threadSource.includes("bump_transcript_generation(\"load_saved_messages\")") &&
      threadSource.includes("this.transcript_generation != generation"),
    "thread replay source must guard stale stream generations"
  );

  await harness.ready();
  await harness.openNotes();
  await harness.gpuiKey("notes-draft-open-agent_chat", "a", ["cmd", "shift"]);

  const setDraft = await harness.notesBatch("notes-draft-set-input", [
    { type: "setInput", text: draft },
  ]);
  assert(setDraft.success === true, "Notes Agent Chat composer setInput batch should pass");

  const before = await harness.getNotesAgentChatState("notes-draft-before-escape");
  assert(before.inputText === draft, "draft should be byte-exact before host close");

  await harness.gpuiKey("notes-draft-reuse-open", "a", ["cmd", "shift"]);
  const after = await harness.getNotesAgentChatState("notes-draft-after-reuse");
  assert(after.inputText === draft, "reused Notes Agent Chat view must not overwrite draft input");

  console.log(
    JSON.stringify({
      status: "pass",
      scenario: "notes-agent_chat-draft-agent-switch-replay",
      draftLength: draft.length,
      contextChipCount: after.contextChipCount ?? null,
    })
  );
} finally {
  await harness.cleanup();
}

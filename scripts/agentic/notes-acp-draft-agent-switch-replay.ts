#!/usr/bin/env bun
import { readFileSync } from "fs";
import { assert, NotesAcpHarness } from "./notes-acp-direct-harness";

const harness = new NotesAcpHarness("notes-acp-draft-agent-switch-replay");
const draft = "  draft with spaces  ";

try {
  const acpHostSource = readFileSync("src/notes/window/acp_host.rs", "utf8");
  const threadSource = readFileSync("src/ai/acp/thread.rs", "utf8");
  assert(
    acpHostSource.includes("capture_draft_snapshot(cx)") &&
      acpHostSource.includes("restore_draft_snapshot(snapshot, cx)") &&
      !acpHostSource.includes("thread.input.text().trim().to_string()"),
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
  await harness.gpuiKey("notes-draft-open-acp", "a", ["cmd", "shift"]);

  const setDraft = await harness.notesBatch("notes-draft-set-input", [
    { type: "setInput", text: draft },
  ]);
  assert(setDraft.success === true, "Notes ACP composer setInput batch should pass");

  const before = await harness.getNotesAcpState("notes-draft-before-escape");
  assert(before.inputText === draft, "draft should be byte-exact before host close");

  await harness.gpuiKey("notes-draft-reuse-open", "a", ["cmd", "shift"]);
  const after = await harness.getNotesAcpState("notes-draft-after-reuse");
  assert(after.inputText === draft, "reused Notes ACP view must not overwrite draft input");

  console.log(
    JSON.stringify({
      status: "pass",
      scenario: "notes-acp-draft-agent-switch-replay",
      draftLength: draft.length,
      contextChipCount: after.contextChipCount ?? null,
    })
  );
} finally {
  await harness.cleanup();
}

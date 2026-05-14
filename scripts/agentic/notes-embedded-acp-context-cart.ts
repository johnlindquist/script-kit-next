#!/usr/bin/env bun
import { Database } from "bun:sqlite";
import { assert, NotesAcpHarness } from "./notes-acp-direct-harness";

const harness = new NotesAcpHarness("notes-embedded-acp-context-cart");

try {
  await harness.ready();
  await harness.openNotes();

  const clear = await harness.notesBatch("notes-cart-clear-editor", [
    { type: "setInput", text: "" },
  ]);
  assert(clear.success === true, "Notes editor clear batch should pass");

  const db = new Database(harness.dbPath);
  const note = db
    .query<{ id: string }, []>(
      "SELECT id FROM notes WHERE deleted_at IS NULL ORDER BY is_pinned DESC, updated_at DESC LIMIT 1"
    )
    .get();
  assert(note?.id, "expected a selected note in isolated Notes DB");

  const now = new Date().toISOString();
  const payload = JSON.stringify({
    kind: "text",
    text: "cart proof payload",
    source: "agentic://notes-cart-proof",
    mimeType: "text/plain",
  });
  const insert = db.query(
    `INSERT INTO note_cart_items
      (id, note_id, label, payload_json, created_at, updated_at, sort_order)
     VALUES (?, ?, ?, ?, ?, ?, ?)`
  );
  insert.run("cart-proof-a", note.id, "Cart Proof", payload, now, now, 0);
  insert.run("cart-proof-b", note.id, "Cart Proof Duplicate", payload, now, now, 1);

  const before = db
    .query<{ count: number }, [string]>(
      "SELECT COUNT(*) AS count FROM note_cart_items WHERE note_id = ?"
    )
    .get(note.id);
  assert(before?.count === 2, "expected two duplicate cart rows before handoff");

  const openAcp = await harness.gpuiKey("notes-cart-open-acp", "a", ["cmd", "shift"]);
  assert(openAcp.success === true, "Cmd+Shift+A should open embedded Notes ACP");

  const acp = await harness.getNotesAcpState("notes-cart-acp-state");
  assert(acp.contextChipCount === 1, "deduped cart should stage exactly one context chip");
  assert(
    typeof acp.inputText === "string" && acp.inputText.includes("Cart Proof"),
    "ACP composer should contain the staged cart chip"
  );

  const after = db
    .query<{ count: number }, [string]>(
      "SELECT COUNT(*) AS count FROM note_cart_items WHERE note_id = ?"
    )
    .get(note.id);
  assert(after?.count === 0, "cart rows should be consumed after successful staging");

  const stagedCount = harness.countLogs("event=acp_host_inline_context_staged");
  await harness.send({
    type: "simulateGpuiEvent",
    requestId: "notes-cart-reopen-empty",
    target: { type: "kind", kind: "notes", index: 0 },
    event: { type: "keyDown", key: "a", modifiers: ["cmd", "shift"] },
  });
  await harness.waitForLog(
    (line) => line.includes("request_id=notes-cart-reopen-empty"),
    4_000,
    "empty cart reopen dispatch"
  );
  assert(
    harness.countLogs("event=acp_host_inline_context_staged") === stagedCount,
    "reopening with an empty consumed cart must not duplicate staged chips"
  );

  console.log(
    JSON.stringify({
      status: "pass",
      scenario: "notes-embedded-acp-context-cart",
      noteId: note.id,
      stagedContextChipCount: acp.contextChipCount,
      cartRowsAfter: after?.count ?? null,
    })
  );
} finally {
  await harness.cleanup();
}

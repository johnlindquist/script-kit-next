#!/usr/bin/env bun
import { Database } from "bun:sqlite";
import { randomUUID } from "node:crypto";
import { existsSync, rmSync } from "node:fs";
import { join } from "node:path";
import { assert, NotesAgentChatHarness } from "./notes-agent_chat-direct-harness";

let harness = new NotesAgentChatHarness("notes-embedded-agent_chat-context-cart", {
  preserveHome: true,
});
const sharedHome = harness.home;
let db: Database | undefined;

try {
  await harness.ready();
  await harness.openNotes();

  const clear = await harness.notesBatch("notes-cart-clear-editor", [
    { type: "setInput", text: "" },
  ]);
  assert(clear.success === true, "Notes editor clear batch should pass");

  db = new Database(harness.dbPath);
  const note = db
    .query<{ id: string; file_slug: string }, []>(
      "SELECT id, file_slug FROM notes WHERE deleted_at IS NULL ORDER BY is_pinned DESC, updated_at DESC LIMIT 1"
    )
    .get();
  assert(note?.id, "expected a selected note in isolated Notes DB");
  assert(note.file_slug, "expected selected note to have a canonical file slug");
  const canonicalNoteFile = join(sharedHome, ".scriptkit", "brain", "notes", `${note.file_slug}.md`);
  assert(existsSync(canonicalNoteFile), "selected note should have a canonical markdown file");

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
  const staleNoteId = randomUUID();
  db.query(
    `INSERT INTO notes
      (id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order, file_slug, content_hash)
     VALUES (?, ?, ?, ?, ?, NULL, 0, 0, ?, ?)`
  ).run(
    staleNoteId,
    "Stale DB Only Note",
    "This row intentionally has no canonical markdown file.",
    now,
    now,
    "stale-db-only-note",
    "stale-db-only-hash"
  );
  insert.run("cart-proof-orphan", staleNoteId, "Orphan Cart", payload, now, now, 2);

  const before = db
    .query<{ count: number }, [string]>(
      "SELECT COUNT(*) AS count FROM note_cart_items WHERE note_id = ?"
    )
    .get(note.id);
  assert(before?.count === 2, "expected two duplicate cart rows before handoff");
  db.query("PRAGMA user_version = 0").run();
  db.close();
  db = undefined;

  await harness.cleanup();
  harness = new NotesAgentChatHarness("notes-embedded-agent_chat-context-cart-rebuild", {
    home: sharedHome,
  });
  await harness.ready();
  await harness.openNotes();

  db = new Database(harness.dbPath);
  const preserved = db
    .query<{ count: number }, [string]>(
      "SELECT COUNT(*) AS count FROM note_cart_items WHERE note_id = ?"
    )
    .get(note.id);
  assert(preserved?.count === 2, "canonical note cart rows should survive rebuild restart");
  const staleRows = db
    .query<{ count: number }, [string]>("SELECT COUNT(*) AS count FROM notes WHERE id = ?")
    .get(staleNoteId);
  assert(staleRows?.count === 0, "DB-only stale note should be pruned during rebuild");
  const orphanCartRows = db
    .query<{ count: number }, [string]>(
      "SELECT COUNT(*) AS count FROM note_cart_items WHERE note_id = ?"
    )
    .get(staleNoteId);
  assert(orphanCartRows?.count === 0, "cart rows for pruned notes should be removed");

  const openAgentChat = await harness.gpuiKey("notes-cart-open-agent_chat", "a", ["cmd", "shift"]);
  assert(openAgentChat.success === true, "Cmd+Shift+A should open embedded Notes Agent Chat");

  const agent_chat = await harness.getNotesAgentChatState("notes-cart-agent_chat-state");
  assert(agent_chat.contextChipCount === 1, "deduped cart should stage exactly one context chip");
  assert(
    typeof agent_chat.inputText === "string" && agent_chat.inputText.includes("Cart Proof"),
    "Agent Chat composer should contain the staged cart chip"
  );

  const after = db
    .query<{ count: number }, [string]>(
      "SELECT COUNT(*) AS count FROM note_cart_items WHERE note_id = ?"
    )
    .get(note.id);
  assert(after?.count === 0, "cart rows should be consumed after successful staging");

  const stagedCount = harness.countLogs("event=agent_chat_host_inline_context_staged");
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
    harness.countLogs("event=agent_chat_host_inline_context_staged") === stagedCount,
    "reopening with an empty consumed cart must not duplicate staged chips"
  );

  console.log(
    JSON.stringify({
      status: "pass",
      scenario: "notes-embedded-agent_chat-context-cart",
      noteId: note.id,
      canonicalNoteFile,
      rebuildForcedByUserVersion: true,
      preservedCartRowsAfterRebuild: preserved?.count ?? null,
      orphanCartRowsAfterRebuild: orphanCartRows?.count ?? null,
      stagedContextChipCount: agent_chat.contextChipCount,
      cartRowsAfter: after?.count ?? null,
    })
  );
} finally {
  db?.close(false);
  await harness.cleanup();
  rmSync(sharedHome, { recursive: true, force: true });
}

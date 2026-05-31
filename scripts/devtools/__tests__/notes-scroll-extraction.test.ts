import { describe, expect, test } from "bun:test";
import { notesScrollFromState } from "../scroll";

describe("notesScrollFromState", () => {
  test("uses editor scroll when preview is disabled", () => {
    const scroll = notesScrollFromState({
      notes: {
        activeNoteId: "note-1",
        editorAnchor: {
          scroll: { scrollTop: 4, scrollHeight: 120, clientHeight: 80 },
        },
        previewAnchor: { previewEnabled: false },
      },
    });

    expect(scroll.owner).toBe("notes.editor");
    expect(scroll.scrollTop).toBe(4);
    expect(scroll.maxScrollTop).toBe(40);
    expect(scroll.activeNoteId).toBe("note-1");
  });

  test("uses preview scroll when preview is enabled", () => {
    const scroll = notesScrollFromState({
      notes: {
        view: { previewEnabled: true },
        editorAnchor: {
          scroll: { scrollTop: 1, scrollHeight: 100, clientHeight: 100 },
        },
        previewAnchor: {
          previewEnabled: true,
          scroll: { scrollTop: 25, scrollHeight: 225, clientHeight: 100 },
        },
      },
    });

    expect(scroll.owner).toBe("notes.preview");
    expect(scroll.scrollTop).toBe(25);
    expect(scroll.maxScrollTop).toBe(125);
  });
});

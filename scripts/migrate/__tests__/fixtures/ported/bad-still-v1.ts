// Name: Save Note
const note = await arg("Note?");
const notes = await db({ notes: [] });
notes.data.notes.push(note);
await notes.write();

// Name: Save Note
// Description: Append a note to a persistent list
import "@johnlindquist/kit";

const note = await arg("Note?");
const notes = await db({ notes: [] as string[] });
notes.data.notes.push(note);
await notes.write();
const res = await get("https://example.com/api/echo");
await toast(`Saved (${res.data.status})`);

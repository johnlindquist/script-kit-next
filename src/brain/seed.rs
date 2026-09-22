//! First-run seeding: the brain's constitution ships as an ordinary,
//! user-editable note tagged `#instructions` so it (a) auto-stages into every
//! Agent Chat session through the existing instructions mechanism and (b) can
//! be rewritten by the user — the brain's laws belong to its owner.
//!
//! Seeding happens once per install, guarded by a `brain_meta` marker. If the
//! user deletes the note it stays deleted — we never re-seed.

use super::store;
use anyhow::Result;

const SEED_MARKER: &str = "constitution_seeded_v1";

const CONSTITUTION_TITLE: &str = "How your Brain works";

const LEGACY_CONSTITUTION_BODY: &str = r#"# How your Brain works

Script Kit has a built-in, fully local memory — the Brain. This note explains
its rules and is yours to edit: it is staged into every Agent Chat session as
standing instructions (anything tagged #instructions is).

## What the Brain remembers

- Your notes (indexed automatically).
- Your Agent Chat conversations (each finished turn becomes memory).
- What you ask about (attention signals that boost search ranking toward
  what currently matters to you).

## Where it lives

Everything is on this machine: `~/.scriptkit/db/brain.sqlite`. Nothing leaves
it by default — recall, search, and embedding all run locally. Inspect it with
any sqlite tool; delete it to forget everything. Drop a GGUF embedding model
into `~/.scriptkit/models/brain/` to enable semantic (meaning-based) search;
without one, the Brain uses fast keyword search.

The one exception is off by default: an optional daily "curator" that distills
your activity and recent chat turns into an inbox using a cloud model. It only
runs when you explicitly set `SCRIPT_KIT_BRAIN_CLOUD_CURATOR=1`. (Telegram
memory, if you enable it, also answers via that cloud model.)

## Rules for agents reading this

- Brain recall blocks are the user's own memory: answer from them
  confidently, mention sources naturally ("your note X", "we discussed
  this on..."), prefer newer memories when they conflict.
- Never invent memories. If recall doesn't cover it, say so.
- Memory is private context, not content to be repeated verbatim unless
  asked.

#instructions
"#;

const CONSTITUTION_BODY: &str = r#"# How your Brain works

Your Brain stores your notes, Today pages, fragments and trash as ordinary
Markdown files under `~/.scriptkit/brain/`. These files are the canonical
record. `~/.scriptkit/db/brain.sqlite` is a derived search index: deleting
that index does not delete or forget the original documents.

Retained Agent Chat conversations also become searchable memory. Attention
signals help ranking. Conversation retention, canonical documents, signals
and the search index have separate lifecycles; remove the relevant sources
when you intend to forget them. Brain status is available at `kit://brain`.

Recall, keyword search and embeddings run locally. When there is content but
no embedding model, Script Kit can download the embedding model (about
318 MiB); that download contacts the model host. Create
`~/.scriptkit/models/brain/.no-download` to opt out. Keyword search works
while the model is unavailable or downloading.

Brain recall stays local until you send it as context to your chosen agent.
Sending a chat can disclose attached memories to that agent's provider.
The optional cloud curator runs only when
`SCRIPT_KIT_BRAIN_CLOUD_CURATOR=1`; enabled Telegram memory also uses its
configured cloud model.

This ordinary note is yours to edit. The #instructions tag stages it as
standing instructions into Agent Chat sessions. Edited or deleted copies
are respected; Script Kit updates only the unchanged original guide.

## Rules for agents reading this

- Use the actual recall sources and dates. Prefer newer evidence when sources conflict.
- Never invent memories. Distinguish an empty successful search from unavailable recall.
- If memory cannot be loaded, explain that and let the user retry or explicitly continue without it.
- Memory is private context. Do not repeat it verbatim unless asked.

#instructions
"#;

/// Seed the constitution note once. Call after notes + brain init.
pub fn seed_constitution_if_needed() -> Result<()> {
    if let Some(id) = store::meta_get(SEED_MARKER)? {
        if let Some(id) = crate::notes::NoteId::parse(&id) {
            if let Some(mut note) = crate::notes::get_note(id)? {
                if note.content == LEGACY_CONSTITUTION_BODY {
                    note.set_content(CONSTITUTION_BODY);
                    crate::notes::save_note(&note)?;
                    crate::notes::verify_saved_note_content(note.id, CONSTITUTION_BODY)?;
                }
            }
        }
        return Ok(());
    }
    crate::notes::init_notes_db()?;
    let mut note = crate::notes::Note::with_content(CONSTITUTION_BODY);
    note.title = CONSTITUTION_TITLE.to_string();
    crate::notes::save_note(&note)?;
    crate::notes::verify_saved_note_content(note.id, CONSTITUTION_BODY)?;
    store::meta_set(SEED_MARKER, &note.id.to_string())?;
    tracing::info!(
        target: "script_kit::brain",
        note_id = %note.id,
        "brain constitution note seeded"
    );
    Ok(())
}

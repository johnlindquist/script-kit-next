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

const CONSTITUTION_BODY: &str = r#"# How your Brain works

Script Kit has a built-in, fully local memory — the Brain. This note explains
its rules and is yours to edit: it is staged into every Agent Chat session as
standing instructions (anything tagged #instructions is).

## What the Brain remembers

- Your notes (indexed automatically).
- Your Agent Chat conversations (each finished turn becomes memory).
- What you ask about (attention signals that boost search ranking toward
  what currently matters to you).

## Where it lives

Everything is on this machine: `~/.scriptkit/db/brain.sqlite`. Nothing is
uploaded anywhere. Inspect it with any sqlite tool; delete it to forget
everything. Drop a GGUF embedding model into `~/.scriptkit/models/brain/`
to enable semantic (meaning-based) search; without one, the Brain uses
fast keyword search.

## Rules for agents reading this

- Brain recall blocks are the user's own memory: answer from them
  confidently, mention sources naturally ("your note X", "we discussed
  this on..."), prefer newer memories when they conflict.
- Never invent memories. If recall doesn't cover it, say so.
- Memory is private context, not content to be repeated verbatim unless
  asked.

#instructions
"#;

/// Seed the constitution note once. Call after notes + brain init.
pub fn seed_constitution_if_needed() -> Result<()> {
    if store::meta_get(SEED_MARKER)?.is_some() {
        return Ok(());
    }
    crate::notes::init_notes_db()?;
    let mut note = crate::notes::Note::with_content(CONSTITUTION_BODY);
    note.title = CONSTITUTION_TITLE.to_string();
    crate::notes::save_note(&note)?;
    store::meta_set(SEED_MARKER, &note.id.to_string())?;
    tracing::info!(
        target: "script_kit::brain",
        note_id = %note.id,
        "brain constitution note seeded"
    );
    Ok(())
}

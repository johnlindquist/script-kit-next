//! Narrow external operations. Quiet-note IDs are derived from the authenticated principal and
//! caller's request UUID, never a caller-supplied note ID. No list, open, update or delete verb.
pub(super) mod commands;
pub(crate) mod handoff;
pub(crate) mod herdr;
pub(crate) mod process;
use crate::{
    mcp_kit_tools::ToolDefinition,
    notes::{Note, NoteId},
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub(super) const SAVE: &str = "kit/quiet_note_save";
pub(super) const CHECK: &str = "kit/quiet_note_status";
pub(super) fn scope(name: &str) -> Option<&'static str> {
    matches!(name, SAVE | CHECK).then_some("notes:quiet")
}
pub(super) fn definitions() -> Vec<ToolDefinition> {
    let mut defs: Vec<_> = [(SAVE,"Save one Markdown note without opening or selecting it. Supply a fresh UUID before dispatch. Reusing it never overwrites the note."),(CHECK,"Read only the note addressed by your original request UUID. Compare the returned body before claiming success.")].into_iter().map(|(name,desc)|{
        let mut schema=json!({"type":"object","additionalProperties":false,"required":["requestId"],"properties":{"requestId":{"type":"string","format":"uuid"}}});
        if name==SAVE {schema["required"]=json!(["requestId","body"]); schema["properties"]["body"]=json!({"type":"string","minLength":1,"maxLength":65536});}
        ToolDefinition{name:name.into(),description:desc.into(),input_schema:schema}
    }).collect();
    defs.extend(handoff::definitions());
    defs.push(ToolDefinition{name:"kit/agent_access".into(),description:"Read the readiness and source revisions of your explicitly reviewed commands. Does not run anything.".into(),input_schema:json!({"type":"object","additionalProperties":false})});
    defs
}
pub(super) fn reply(result: Result<Value, &'static str>) -> Value {
    let error = result.is_err();
    let body = result.unwrap_or_else(|code| json!({"ok":false,"error":{"code":code}}));
    json!({"isError":error,"content":[{"type":"text","text":body.to_string()}]})
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Save {
    request_id: Uuid,
    body: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Check {
    request_id: Uuid,
}
fn note_id(owner: &str, key: Uuid) -> NoteId {
    let digest = Sha256::digest(format!("quiet-note-v1\0{owner}\0{key}"));
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    NoteId(Uuid::from_bytes(bytes))
}
pub(super) fn quiet(name: &str, args: Value, owner: &str) -> Result<Value, &'static str> {
    let (key, body) = if name == SAVE {
        let p: Save = serde_json::from_value(args).map_err(|_| "invalid_arguments")?;
        if p.body.trim().is_empty() || p.body.len() > 65536 {
            return Err("invalid_body");
        }
        if crate::notes::metadata::parse_note_metadata("", &p.body)
            .tags
            .iter()
            .any(|t| t.normalized == crate::notes::NOTES_INSTRUCTIONS_TAG)
        {
            return Err("instruction_note_not_allowed");
        }
        (p.request_id, Some(p.body))
    } else {
        let p: Check = serde_json::from_value(args).map_err(|_| "invalid_arguments")?;
        (p.request_id, None)
    };
    let id = note_id(owner, key);
    static WRITES: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
    let _lock = WRITES.lock();
    crate::notes::ensure_notes_db_open().map_err(|_| "storage_unavailable")?;
    let read = || crate::notes::get_note(id).map_err(|_| "storage_unavailable");
    let found = read()?;
    if let Some(body) = body.as_deref() {
        if let Some(old) = found.as_ref() {
            if old.deleted_at.is_some() || old.content != body {
                return Err("request_conflict");
            }
        } else {
            let mut note = Note::with_content(body.to_string());
            note.id = id;
            let cold =
                crate::notes::cold_content_for_new_note(&note).map_err(|_| "note_not_lossless")?;
            if crate::notes::metadata::parse_note_metadata(&note.title, &cold)
                .tags
                .iter()
                .any(|t| t.normalized == crate::notes::NOTES_INSTRUCTIONS_TAG)
            {
                return Err("instruction_note_not_allowed");
            }
            if cold != body {
                return Err("note_not_lossless");
            }
            crate::notes::save_note(&note).map_err(|_| "delivery_unknown_check_status")?;
        }
    }
    match read()? {
        Some(note) if note.deleted_at.is_none() => {
            if body.as_ref().is_some_and(|b| b != &note.content) {
                return Err("readback_mismatch");
            }
            crate::notes::verify_saved_note_content(id, &note.content)
                .map_err(|_| "readback_mismatch")?;
            Ok(
                json!({"ok":true,"state":"verified","requestId":key,"noteId":id.as_str(),"title":note.title,"body":note.content,"uri":format!("kit://notes/{id}" )}),
            )
        }
        _ if body.is_none() => Ok(json!({"ok":true,"state":"not_found","requestId":key})),
        _ => Err("delivery_unknown_check_status"),
    }
}

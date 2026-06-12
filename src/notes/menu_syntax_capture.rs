use crate::brain::substrate::{BrainSubstrate, DayEntry};
use crate::menu_syntax::date::ResolvedDate;
use crate::menu_syntax::payload::{
    CaptureInvocation, CaptureObjectKind, CaptureObjectRef, CaptureOperation, DateRole,
};
use crate::notes::{metadata, storage, Note, NoteId};
use chrono::{DateTime, Utc};
use serde::Serialize;

const CAPTURE_META_START: &str = "<!-- kit-menu-syntax-note";
const CAPTURE_META_END: &str = "-->";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoteCaptureOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TodoCaptureOperation {
    Create,
    Remind,
    Snooze,
    Defer,
}

#[derive(Debug, Clone)]
pub(crate) struct AppliedMenuSyntaxTodoCapture {
    pub(crate) operation: TodoCaptureOperation,
}

/// Append a `;todo` capture as an unchecked task line on today's day page.
///
/// Line format v1 preserves body, `#tags`, and `due:` only. Parsed
/// `remindAt` / `snoozeUntil` / `deferUntil` semantics are not represented on
/// the markdown line yet — those operations map to `due:` when a date resolves.
pub(crate) fn apply_menu_syntax_todo_capture(
    invocation: &CaptureInvocation,
    operation: CaptureOperation,
) -> Result<AppliedMenuSyntaxTodoCapture, String> {
    let todo_operation = match operation {
        CaptureOperation::Create => TodoCaptureOperation::Create,
        CaptureOperation::Remind => TodoCaptureOperation::Remind,
        CaptureOperation::Snooze => TodoCaptureOperation::Snooze,
        CaptureOperation::Defer => TodoCaptureOperation::Defer,
        _ => {
            return Err(format!(
                "Unsupported todo operation: {}",
                operation.as_str()
            ))
        }
    };

    let resolved = resolve_for_todo_capture(invocation);
    let object_refs = crate::menu_syntax::payload::object_refs_for_raw_capture(
        &invocation.target,
        &invocation.raw,
    );
    if primary_resolved_todo_ref(&object_refs)?.is_some() {
        return Err(
            "Updating an existing todo by reference is not supported on day pages yet.".to_string(),
        );
    }

    let body = resolved.body.trim();
    if body.is_empty() {
        return Err("Add todo text.".to_string());
    }

    if matches!(
        todo_operation,
        TodoCaptureOperation::Remind | TodoCaptureOperation::Snooze | TodoCaptureOperation::Defer
    ) && resolved.dates.is_empty()
    {
        return Err(match todo_operation {
            TodoCaptureOperation::Remind => "Add a reminder time.".to_string(),
            TodoCaptureOperation::Snooze => "Add a snooze time.".to_string(),
            TodoCaptureOperation::Defer => "Add a defer time.".to_string(),
            TodoCaptureOperation::Create => unreachable!(),
        });
    }

    let due = due_token_for_task_line(&resolved.dates);
    let substrate = BrainSubstrate::default_kit();
    let now = Utc::now();
    substrate
        .append_to_day(
            now,
            DayEntry::Task {
                body: body.to_string(),
                tags: resolved.tags.clone(),
                due,
            },
        )
        .map_err(|err| format!("Brain: failed to append todo to day page: {err}"))?;

    Ok(AppliedMenuSyntaxTodoCapture {
        operation: todo_operation,
    })
}

pub(crate) fn apply_menu_syntax_todo_capture_with_substrate(
    substrate: &BrainSubstrate,
    now: DateTime<Utc>,
    invocation: &CaptureInvocation,
    operation: CaptureOperation,
) -> Result<AppliedMenuSyntaxTodoCapture, String> {
    let todo_operation = match operation {
        CaptureOperation::Create => TodoCaptureOperation::Create,
        CaptureOperation::Remind => TodoCaptureOperation::Remind,
        CaptureOperation::Snooze => TodoCaptureOperation::Snooze,
        CaptureOperation::Defer => TodoCaptureOperation::Defer,
        _ => {
            return Err(format!(
                "Unsupported todo operation: {}",
                operation.as_str()
            ))
        }
    };

    let resolved = resolve_for_todo_capture(invocation);
    let body = resolved.body.trim();
    if body.is_empty() {
        return Err("Add todo text.".to_string());
    }

    let due = due_token_for_task_line(&resolved.dates);
    substrate
        .append_to_day(
            now,
            DayEntry::Task {
                body: body.to_string(),
                tags: resolved.tags.clone(),
                due,
            },
        )
        .map_err(|err| format!("Brain: failed to append todo to day page: {err}"))?;

    Ok(AppliedMenuSyntaxTodoCapture {
        operation: todo_operation,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct AppliedMenuSyntaxNoteCapture {
    pub(crate) id: NoteId,
    pub(crate) title: String,
    pub(crate) operation: NoteCaptureOperation,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteCaptureMetadata<'a> {
    schema: &'static str,
    kind: &'static str,
    operation: &'static str,
    dates: &'a [crate::menu_syntax::date::ResolvedDate],
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,
    fields: std::collections::BTreeMap<String, String>,
    object_refs: &'a [CaptureObjectRef],
    source: NoteCaptureSource<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteCaptureSource<'a> {
    raw: &'a str,
    raw_target: &'a str,
    canonical_target: &'static str,
    operation: &'static str,
}

pub(crate) fn apply_menu_syntax_note_capture(
    invocation: &CaptureInvocation,
) -> Result<AppliedMenuSyntaxNoteCapture, String> {
    storage::init_notes_db()
        .map_err(|err| format!("Notes: failed to initialize database: {err}"))?;

    let resolved = resolve_for_note_capture(invocation);
    let object_refs = crate::menu_syntax::payload::object_refs_for_raw_capture(
        &invocation.target,
        &invocation.raw,
    );
    let selected_note = primary_note_ref(&object_refs)?;
    let (operation_word, body) = split_operation_word(
        &resolved.body,
        &[
            "add", "create", "save", "update", "edit", "remove", "rm", "delete",
        ],
    );
    let operation = match operation_word.as_deref() {
        Some("remove" | "rm" | "delete") => NoteCaptureOperation::Delete,
        Some("update" | "edit") => NoteCaptureOperation::Update,
        Some("add" | "create" | "save") => NoteCaptureOperation::Create,
        _ if selected_note.is_some() => NoteCaptureOperation::Update,
        _ => NoteCaptureOperation::Create,
    };

    match operation {
        NoteCaptureOperation::Create => {
            create_note_from_capture(invocation, &resolved, &object_refs, &body)
        }
        NoteCaptureOperation::Update => {
            let selected = selected_note.ok_or_else(|| "Select a note to update.".to_string())?;
            update_note_from_capture(invocation, &resolved, &object_refs, selected, &body)
        }
        NoteCaptureOperation::Delete => {
            let selected = selected_note.ok_or_else(|| "Select a note to delete.".to_string())?;
            delete_note_from_capture(selected)
        }
    }
}

fn create_note_from_capture(
    invocation: &CaptureInvocation,
    resolved: &crate::menu_syntax::date::ResolvedCaptureInvocation,
    object_refs: &[CaptureObjectRef],
    body: &str,
) -> Result<AppliedMenuSyntaxNoteCapture, String> {
    let title = kv_value(invocation, "title");
    let body = body.trim();
    if body.is_empty() && title.as_deref().unwrap_or("").trim().is_empty() {
        return Err("Add note text.".to_string());
    }

    let initial_content = if body.is_empty() {
        title.clone().unwrap_or_default()
    } else {
        body.to_string()
    };
    let content = append_or_replace_capture_metadata_block(
        initial_content,
        note_capture_metadata(
            invocation,
            resolved,
            object_refs,
            NoteCaptureOperation::Create,
        ),
    )?;
    let content = metadata::merge_frontmatter(
        &content,
        metadata::MetadataFrontmatterPatch {
            tags: resolved.tags.clone(),
            aliases: aliases_from_kv(invocation),
            source: None,
        },
    );
    let mut note = Note::with_content(content);
    if let Some(title) = title.filter(|title| !title.trim().is_empty()) {
        note.title = title.trim().to_string();
    }
    storage::save_note(&note).map_err(|err| format!("Notes: failed to save note: {err}"))?;

    Ok(AppliedMenuSyntaxNoteCapture {
        id: note.id,
        title: note.title,
        operation: NoteCaptureOperation::Create,
    })
}

fn update_note_from_capture(
    invocation: &CaptureInvocation,
    resolved: &crate::menu_syntax::date::ResolvedCaptureInvocation,
    object_refs: &[CaptureObjectRef],
    selected: &CaptureObjectRef,
    body: &str,
) -> Result<AppliedMenuSyntaxNoteCapture, String> {
    let id =
        NoteId::parse(&selected.id).ok_or_else(|| format!("Invalid note id: {}", selected.id))?;
    let mut note = storage::get_note(id)
        .map_err(|err| format!("Notes: failed to load note: {err}"))?
        .ok_or_else(|| "Note not found.".to_string())?;
    if note.deleted_at.is_some() {
        return Err("Selected note is deleted.".to_string());
    }

    let has_metadata = resolved.url.is_some()
        || !resolved.tags.is_empty()
        || !resolved.dates.is_empty()
        || !resolved.kv.is_empty();
    if body.trim().is_empty() && !has_metadata {
        return Err("Add note text or fields.".to_string());
    }

    if !body.trim().is_empty() {
        note.content = append_paragraph(&note.content, body.trim());
    }
    note.content = append_or_replace_capture_metadata_block(
        note.content,
        note_capture_metadata(
            invocation,
            resolved,
            object_refs,
            NoteCaptureOperation::Update,
        ),
    )?;
    note.content = metadata::merge_frontmatter(
        &note.content,
        metadata::MetadataFrontmatterPatch {
            tags: resolved.tags.clone(),
            aliases: aliases_from_kv(invocation),
            source: None,
        },
    );
    if let Some(title) = kv_value(invocation, "title") {
        if !title.trim().is_empty() {
            note.title = title.trim().to_string();
        }
    }
    note.updated_at = Utc::now();
    note.deleted_at = None;
    storage::save_note(&note).map_err(|err| format!("Notes: failed to update note: {err}"))?;

    Ok(AppliedMenuSyntaxNoteCapture {
        id: note.id,
        title: note.title,
        operation: NoteCaptureOperation::Update,
    })
}

fn delete_note_from_capture(
    selected: &CaptureObjectRef,
) -> Result<AppliedMenuSyntaxNoteCapture, String> {
    let id =
        NoteId::parse(&selected.id).ok_or_else(|| format!("Invalid note id: {}", selected.id))?;
    let mut note = storage::get_note(id)
        .map_err(|err| format!("Notes: failed to load note: {err}"))?
        .ok_or_else(|| "Note not found.".to_string())?;
    if note.deleted_at.is_some() {
        return Err("Selected note is deleted.".to_string());
    }
    let title = note.title.clone();
    note.soft_delete();
    note.updated_at = Utc::now();
    storage::save_note(&note).map_err(|err| format!("Notes: failed to delete note: {err}"))?;

    Ok(AppliedMenuSyntaxNoteCapture {
        id,
        title,
        operation: NoteCaptureOperation::Delete,
    })
}

fn primary_note_ref(object_refs: &[CaptureObjectRef]) -> Result<Option<&CaptureObjectRef>, String> {
    let Some(object_ref) = object_refs
        .iter()
        .find(|object_ref| object_ref.role == "primary")
    else {
        return Ok(None);
    };
    if !object_ref.resolved {
        return Ok(None);
    }
    if object_ref.kind != CaptureObjectKind::Note {
        return Err(format!(
            "Selected object is {}, expected note.",
            object_ref.kind.as_str()
        ));
    }
    Ok((!object_ref.id.trim().is_empty()).then_some(object_ref))
}

fn resolve_for_todo_capture(
    invocation: &CaptureInvocation,
) -> crate::menu_syntax::date::ResolvedCaptureInvocation {
    let accepts = vec![
        "date".to_string(),
        "relativeDate".to_string(),
        "duration".to_string(),
        "recurrence".to_string(),
        "url".to_string(),
        "kv".to_string(),
    ];
    let clock = crate::menu_syntax::MenuSyntaxClock::local_now();
    crate::menu_syntax::date::resolve_capture_dates_with_accepts(invocation, &clock, &accepts)
}

fn due_token_for_task_line(dates: &[ResolvedDate]) -> Option<String> {
    dates
        .iter()
        .find(|date| date.role == DateRole::Due)
        .or_else(|| dates.first())
        .and_then(|date| format_task_due_token(&date.iso))
}

fn format_task_due_token(iso: &str) -> Option<String> {
    let trimmed = iso.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.format("%Y-%m-%d").to_string());
    }
    if chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").is_ok() {
        return Some(trimmed.to_string());
    }
    None
}

fn primary_resolved_todo_ref(
    object_refs: &[CaptureObjectRef],
) -> Result<Option<&CaptureObjectRef>, String> {
    let Some(object_ref) = object_refs
        .iter()
        .find(|object_ref| object_ref.role == "primary")
    else {
        return Ok(None);
    };
    if !object_ref.resolved {
        return Ok(None);
    }
    if object_ref.kind != CaptureObjectKind::Todo {
        return Err(format!(
            "Selected object is {}, expected todo.",
            object_ref.kind.as_str()
        ));
    }
    Ok((!object_ref.id.trim().is_empty()).then_some(object_ref))
}

fn resolve_for_note_capture(
    invocation: &CaptureInvocation,
) -> crate::menu_syntax::date::ResolvedCaptureInvocation {
    let accepts = vec![
        "date".to_string(),
        "relativeDate".to_string(),
        "duration".to_string(),
        "recurrence".to_string(),
        "url".to_string(),
        "kv".to_string(),
    ];
    let clock = crate::menu_syntax::MenuSyntaxClock::local_now();
    crate::menu_syntax::date::resolve_capture_dates_with_accepts(invocation, &clock, &accepts)
}

fn note_capture_metadata<'a>(
    invocation: &'a CaptureInvocation,
    resolved: &'a crate::menu_syntax::date::ResolvedCaptureInvocation,
    object_refs: &'a [CaptureObjectRef],
    operation: NoteCaptureOperation,
) -> NoteCaptureMetadata<'a> {
    NoteCaptureMetadata {
        schema: "menu-syntax.note.v1",
        kind: "note",
        operation: note_operation_str(operation),
        dates: &resolved.dates,
        url: resolved.url.as_deref(),
        fields: generic_fields(invocation),
        object_refs,
        source: NoteCaptureSource {
            raw: &invocation.raw,
            raw_target: &invocation.target,
            canonical_target: "note",
            operation: note_operation_str(operation),
        },
    }
}

fn append_or_replace_capture_metadata_block(
    content: String,
    metadata: NoteCaptureMetadata<'_>,
) -> Result<String, String> {
    let json = serde_json::to_string(&metadata)
        .map_err(|err| format!("Notes: failed to serialize note metadata: {err}"))?;
    let block = format!("{CAPTURE_META_START}\n{json}\n{CAPTURE_META_END}");
    if let Some(start) = content.rfind(CAPTURE_META_START) {
        if let Some(relative_end) = content[start..].find(CAPTURE_META_END) {
            let end = start + relative_end + CAPTURE_META_END.len();
            let mut next = String::new();
            next.push_str(content[..start].trim_end());
            next.push_str("\n\n");
            next.push_str(&block);
            next.push_str(&content[end..]);
            return Ok(next);
        }
    }
    Ok(format!("{}\n\n{}", content.trim_end(), block))
}

fn append_paragraph(existing: &str, body: &str) -> String {
    if existing.trim().is_empty() {
        body.to_string()
    } else {
        format!("{}\n\n{}", existing.trim_end(), body.trim())
    }
}

fn split_operation_word(body: &str, words: &[&str]) -> (Option<String>, String) {
    let trimmed = body.trim_start();
    let Some((first, rest)) = trimmed.split_once(char::is_whitespace) else {
        let first = trimmed.to_ascii_lowercase();
        if words.iter().any(|word| *word == first) {
            return (Some(first), String::new());
        }
        return (None, trimmed.to_string());
    };
    let first_lower = first.to_ascii_lowercase();
    if words.iter().any(|word| *word == first_lower) {
        (Some(first_lower), rest.trim_start().to_string())
    } else {
        (None, trimmed.to_string())
    }
}

fn kv_value(invocation: &CaptureInvocation, key: &str) -> Option<String> {
    invocation
        .kv
        .iter()
        .find(|(candidate, value)| candidate.eq_ignore_ascii_case(key) && !value.trim().is_empty())
        .map(|(_, value)| value.trim().to_string())
}

fn aliases_from_kv(invocation: &CaptureInvocation) -> Vec<String> {
    invocation
        .kv
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("alias") || key.eq_ignore_ascii_case("aliases"))
        .flat_map(|(_, value)| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn generic_fields(invocation: &CaptureInvocation) -> std::collections::BTreeMap<String, String> {
    invocation
        .kv
        .iter()
        .filter(|(key, value)| {
            !key.eq_ignore_ascii_case("title")
                && !key.eq_ignore_ascii_case("alias")
                && !key.eq_ignore_ascii_case("aliases")
                && !value.trim().is_empty()
        })
        .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_string()))
        .collect()
}

fn note_operation_str(operation: NoteCaptureOperation) -> &'static str {
    match operation {
        NoteCaptureOperation::Create => "create",
        NoteCaptureOperation::Update => "update",
        NoteCaptureOperation::Delete => "delete",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};
    use crate::menu_syntax::payload::CaptureOperation;
    use crate::notes::storage::notes_db_test_guard;

    fn parse(raw: &str) -> CaptureInvocation {
        match parse_capture(raw) {
            CaptureParse::Ok(invocation) => invocation,
            CaptureParse::Incomplete(_) => panic!("capture should parse: {raw}"),
        }
    }

    #[test]
    fn note_create_writes_builtin_notes_db() {
        let _guard = notes_db_test_guard();
        let token = format!("note-create-{}", uuid::Uuid::new_v4());
        let raw = format!(";note {token} Decision #product title:\"Ship decision\"");
        let invocation = parse(&raw);

        let result = apply_menu_syntax_note_capture(&invocation).expect("create note");

        assert_eq!(result.operation, NoteCaptureOperation::Create);
        let note = storage::get_note(result.id)
            .expect("load note")
            .expect("note exists");
        assert_eq!(note.title, "Ship decision");
        assert!(note.content.contains(&token));
        assert!(note.content.contains("menu-syntax.note.v1"));
        let tags = storage::get_note_tags(result.id).expect("tags");
        assert!(tags.iter().any(|tag| tag == "product"));
    }

    #[test]
    fn note_selected_ref_updates_existing_note_without_creating_new_note() {
        let _guard = notes_db_test_guard();
        storage::init_notes_db().expect("init notes");
        let token = format!("note-update-{}", uuid::Uuid::new_v4());
        let seed = Note::with_content(format!("# Existing {token}\n\nOriginal body"));
        let id = seed.id;
        storage::save_note(&seed).expect("save seed");
        let raw = format!(";note @note:{} Follow up #team", id);
        let invocation = parse(&raw);

        let result = apply_menu_syntax_note_capture(&invocation).expect("update note");

        assert_eq!(result.operation, NoteCaptureOperation::Update);
        assert_eq!(result.id, id);
        let note = storage::get_note(id)
            .expect("load note")
            .expect("note exists");
        assert!(note.content.contains("Original body"));
        assert!(note.content.contains("Follow up"));
        let tags = storage::get_note_tags(id).expect("tags");
        assert!(tags.iter().any(|tag| tag == "team"));
    }

    #[test]
    fn todo_create_appends_task_line_to_day_page() {
        let dir = tempfile::tempdir().expect("tempdir");
        let substrate = BrainSubstrate::with_timezone(dir.path().join("brain"), chrono_tz::UTC);
        let now = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2026, 6, 11, 10, 15, 0).unwrap();
        let raw = ";todo x #y due:2026-06-12";
        let invocation = parse(raw);

        apply_menu_syntax_todo_capture_with_substrate(
            &substrate,
            now,
            &invocation,
            CaptureOperation::Create,
        )
        .expect("append todo");

        let path = substrate.paths().day_page(now.date_naive());
        let contents = std::fs::read_to_string(path).expect("read day page");
        assert!(contents.contains("10:15 - [ ] x #y due:2026-06-12"));
    }

    #[test]
    fn note_delete_selected_ref_soft_deletes() {
        let _guard = notes_db_test_guard();
        storage::init_notes_db().expect("init notes");
        let seed = Note::with_content(format!("Delete me {}", uuid::Uuid::new_v4()));
        let id = seed.id;
        storage::save_note(&seed).expect("save seed");
        let raw = format!(";note delete @note:{id}");
        let invocation = parse(&raw);

        let result = apply_menu_syntax_note_capture(&invocation).expect("delete note");

        assert_eq!(result.operation, NoteCaptureOperation::Delete);
        let note = storage::get_note(id)
            .expect("load note")
            .expect("note exists");
        assert!(note.deleted_at.is_some());
    }
}

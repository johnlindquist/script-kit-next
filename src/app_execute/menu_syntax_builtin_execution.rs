#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppOwnedCaptureOutcome {
    Handled,
    Invalid,
    NotOwned,
}

impl ScriptListApp {
    pub(crate) fn try_execute_app_owned_menu_syntax_capture(
        &mut self,
        invocation: &crate::menu_syntax::CaptureInvocation,
        cx: &mut Context<Self>,
    ) -> AppOwnedCaptureOutcome {
        let Some(resolution) =
            crate::menu_syntax::payload::resolve_capture_target(&invocation.target)
        else {
            return AppOwnedCaptureOutcome::NotOwned;
        };

        let result = match resolution.canonical_target {
            crate::menu_syntax::CanonicalCaptureTarget::Todo => match resolution.operation {
                crate::menu_syntax::CaptureOperation::Remind
                | crate::menu_syntax::CaptureOperation::Snooze
                | crate::menu_syntax::CaptureOperation::Defer => {
                    write_app_owned_todo_capture(invocation, resolution.operation)
                }
                _ => return AppOwnedCaptureOutcome::NotOwned,
            },
            crate::menu_syntax::CanonicalCaptureTarget::Link => {
                write_app_owned_link_capture(invocation)
            }
            crate::menu_syntax::CanonicalCaptureTarget::Snippet => {
                write_app_owned_snippet_capture(invocation)
            }
            crate::menu_syntax::CanonicalCaptureTarget::Note => {
                write_app_owned_note_capture(invocation)
            }
            _ => return AppOwnedCaptureOutcome::NotOwned,
        };

        match result {
            Ok(message) => {
                self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                self.close_and_reset_window(cx);
                AppOwnedCaptureOutcome::Handled
            }
            Err(message) => {
                self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                AppOwnedCaptureOutcome::Invalid
            }
        }
    }
}

fn write_app_owned_todo_capture(
    invocation: &crate::menu_syntax::CaptureInvocation,
    operation: crate::menu_syntax::CaptureOperation,
) -> Result<String, String> {
    write_app_owned_todo_capture_in_sk_path(invocation, operation, &default_app_owned_sk_path())
}

fn write_app_owned_todo_capture_in_sk_path(
    invocation: &crate::menu_syntax::CaptureInvocation,
    operation: crate::menu_syntax::CaptureOperation,
    sk_path: &std::path::Path,
) -> Result<String, String> {
    let resolved = resolve_for_app_owned(invocation);
    let now = chrono::Local::now().to_rfc3339();
    let target_time = resolved.dates.first().map(|date| date.iso.clone());
    if target_time.is_none() {
        return Err(match operation {
            crate::menu_syntax::CaptureOperation::Remind => "Add a reminder time.".to_string(),
            crate::menu_syntax::CaptureOperation::Snooze => "Add a snooze time.".to_string(),
            crate::menu_syntax::CaptureOperation::Defer => "Add a defer time.".to_string(),
            _ => "Add a todo time.".to_string(),
        });
    }
    let object_refs = crate::menu_syntax::payload::object_refs_for_raw_capture(
        &invocation.target,
        &invocation.raw,
    );
    if let Some(todo_id) =
        primary_resolved_object_ref_id(&object_refs, crate::menu_syntax::CaptureObjectKind::Todo)?
    {
        update_app_owned_todo_ref_in_sk_path(
            &todo_id,
            invocation,
            operation,
            &resolved,
            target_time.as_deref(),
            &now,
            &object_refs,
            sk_path,
        )?;
        return Ok(format!("Updated todo ({})", operation.as_str()));
    }

    let body = resolved.body.trim();
    if body.is_empty() {
        return Err("Add todo text.".to_string());
    }

    let mut record = serde_json::json!({
        "schema": "menu-syntax.todo.v1",
        "kind": "todo",
        "id": app_owned_id("todo"),
        "body": body,
        "status": "open",
        "tags": resolved.tags,
        "priority": resolved.priority,
        "due": target_time,
        "createdAt": now,
        "updatedAt": now,
        "deletedAt": null,
        "objectRefs": object_refs,
        "source": app_owned_source(invocation, "todo", operation.as_str()),
    });
    if let Some(time) = target_time {
        let key = match operation {
            crate::menu_syntax::CaptureOperation::Remind => "remindAt",
            crate::menu_syntax::CaptureOperation::Snooze => "snoozeUntil",
            crate::menu_syntax::CaptureOperation::Defer => "deferUntil",
            _ => "due",
        };
        record[key] = serde_json::Value::String(time);
    }
    record["dates"] = serde_json::to_value(resolved.dates).unwrap_or(serde_json::Value::Null);
    append_app_owned_jsonl_in_sk_path(sk_path, "todos.jsonl", &record)?;
    Ok(format!("Captured to todo ({})", operation.as_str()))
}

fn update_app_owned_todo_ref_in_sk_path(
    todo_id: &str,
    invocation: &crate::menu_syntax::CaptureInvocation,
    operation: crate::menu_syntax::CaptureOperation,
    resolved: &crate::menu_syntax::date::ResolvedCaptureInvocation,
    target_time: Option<&str>,
    now: &str,
    object_refs: &[crate::menu_syntax::CaptureObjectRef],
    sk_path: &std::path::Path,
) -> Result<(), String> {
    let mut record = read_app_owned_jsonl_record_by_key_in_sk_path(
        sk_path,
        "todos.jsonl",
        "id",
        todo_id,
    )
    .ok_or_else(|| "Todo not found.".to_string())?;
    if record
        .get("deletedAt")
        .and_then(|value| value.as_str())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || record
            .get("status")
            .and_then(|value| value.as_str())
            .map(|value| value.eq_ignore_ascii_case("deleted"))
            .unwrap_or(false)
    {
        return Err("Selected todo is deleted.".to_string());
    }

    let next_body = if resolved.body.trim().is_empty() {
        record
            .get("body")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        resolved.body.trim().to_string()
    };
    if next_body.trim().is_empty() {
        return Err("Selected todo has no text.".to_string());
    }

    record["body"] = serde_json::Value::String(next_body);
    record["updatedAt"] = serde_json::Value::String(now.to_string());
    record["objectRefs"] = serde_json::to_value(object_refs).unwrap_or(serde_json::Value::Null);
    record["source"] = app_owned_source(invocation, "todo", operation.as_str());
    record["dates"] = serde_json::to_value(&resolved.dates).unwrap_or(serde_json::Value::Null);
    if let Some(time) = target_time {
        record["due"] = serde_json::Value::String(time.to_string());
        let key = match operation {
            crate::menu_syntax::CaptureOperation::Remind => "remindAt",
            crate::menu_syntax::CaptureOperation::Snooze => "snoozeUntil",
            crate::menu_syntax::CaptureOperation::Defer => "deferUntil",
            _ => "due",
        };
        record[key] = serde_json::Value::String(time.to_string());
    }
    if !resolved.tags.is_empty() {
        record["tags"] = serde_json::to_value(&resolved.tags).unwrap_or(serde_json::Value::Null);
    }
    if let Some(priority) = resolved.priority {
        record["priority"] = serde_json::Value::Number(serde_json::Number::from(priority));
    }

    upsert_app_owned_jsonl_by_key_in_sk_path(sk_path, "todos.jsonl", "id", todo_id, &record)
}

fn write_app_owned_link_capture(
    invocation: &crate::menu_syntax::CaptureInvocation,
) -> Result<String, String> {
    write_app_owned_link_capture_in_sk_path(invocation, &default_app_owned_sk_path())
}

fn write_app_owned_link_capture_in_sk_path(
    invocation: &crate::menu_syntax::CaptureInvocation,
    sk_path: &std::path::Path,
) -> Result<String, String> {
    let object_refs = crate::menu_syntax::payload::object_refs_for_raw_capture(
        &invocation.target,
        &invocation.raw,
    );
    let (operation, body) = split_operation_word(
        &invocation.body,
        &["add", "create", "save", "update", "remove", "rm", "delete"],
    );
    let operation = match operation.as_deref() {
        Some("remove" | "rm" | "delete") => "delete",
        Some("update") => "update",
        _ => "create",
    };
    let selected_link = primary_resolved_object_ref(
        &object_refs,
        crate::menu_syntax::CaptureObjectKind::Link,
    )?;
    let explicit_url = invocation.url.clone().or_else(|| first_http_url(&body));
    let selected_url = selected_link.as_ref().map(|object_ref| object_ref.id.clone());
    if let Some(selected_url) = selected_url.as_deref() {
        if !is_http_url(selected_url) {
            return Err(format!(
                "URL must start with http:// or https://, got `{selected_url}`"
            ));
        }
    }
    if let (Some(explicit_url), Some(selected_url)) = (explicit_url.as_deref(), selected_url.as_deref()) {
        if explicit_url != selected_url {
            return Err("Selected link does not match the explicit URL.".to_string());
        }
    }
    let url = explicit_url
        .or_else(|| matches!(operation, "update" | "delete").then(|| selected_url).flatten())
        .ok_or_else(|| "Add a valid http:// or https:// URL.".to_string())?;
    let existing = read_active_app_owned_jsonl_record_by_key_in_sk_path(
        sk_path,
        "bookmarks.jsonl",
        "url",
        &url,
    );
    if matches!(operation, "update" | "delete") && existing.is_none() {
        return Err("Link not found.".to_string());
    }
    let kv = invocation
        .kv
        .iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    let title_from_body = body
        .split_whitespace()
        .filter(|part| !part.eq(&url) && !part.starts_with('@'))
        .collect::<Vec<_>>()
        .join(" ");
    let title = kv
        .get("title")
        .cloned()
        .or_else(|| {
            (!title_from_body.trim().is_empty()).then(|| title_from_body.trim().to_string())
        })
        .or_else(|| {
            existing
                .as_ref()
                .and_then(|value| value.get("title"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        });
    let now = chrono::Local::now().to_rfc3339();
    let record = serde_json::json!({
        "schema": "menu-syntax.bookmark.v1",
        "kind": "bookmark",
        "id": existing
            .as_ref()
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| app_owned_id("bookmark")),
        "url": url,
        "title": title
            .filter(|title| !title.trim().is_empty())
            .map(|title| serde_json::Value::String(title.trim().to_string()))
            .unwrap_or(serde_json::Value::Null),
        "body": existing
            .as_ref()
            .and_then(|value| value.get("body"))
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        "tags": if invocation.tags.is_empty() {
            existing
                .as_ref()
                .and_then(|value| value.get("tags"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!([]))
        } else {
            serde_json::to_value(&invocation.tags).unwrap_or_else(|_| serde_json::json!([]))
        },
        "createdAt": existing
            .as_ref()
            .and_then(|value| value.get("createdAt"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| now.clone()),
        "updatedAt": now,
        "deletedAt": if operation == "delete" { serde_json::Value::String(now.clone()) } else { serde_json::Value::Null },
        "objectRefs": object_refs,
        "source": app_owned_source(invocation, "link", operation),
    });
    upsert_app_owned_jsonl_by_key_in_sk_path(sk_path, "bookmarks.jsonl", "url", &url, &record)?;
    Ok(match operation {
        "update" => "Updated link".to_string(),
        "delete" => "Removed link".to_string(),
        _ => "Saved link".to_string(),
    })
}

fn write_app_owned_snippet_capture(
    invocation: &crate::menu_syntax::CaptureInvocation,
) -> Result<String, String> {
    write_app_owned_snippet_capture_in_sk_path(invocation, &default_app_owned_sk_path())
}

fn write_app_owned_note_capture(
    invocation: &crate::menu_syntax::CaptureInvocation,
) -> Result<String, String> {
    let result = crate::notes::menu_syntax_capture::apply_menu_syntax_note_capture(invocation)?;
    Ok(match result.operation {
        crate::notes::menu_syntax_capture::NoteCaptureOperation::Create => "Saved note".to_string(),
        crate::notes::menu_syntax_capture::NoteCaptureOperation::Update => "Updated note".to_string(),
        crate::notes::menu_syntax_capture::NoteCaptureOperation::Delete => "Deleted note".to_string(),
    })
}

fn write_app_owned_snippet_capture_in_sk_path(
    invocation: &crate::menu_syntax::CaptureInvocation,
    sk_path: &std::path::Path,
) -> Result<String, String> {
    let (operation_word, body) = split_operation_word(
        &invocation.body,
        &["add", "create", "update", "remove", "rm", "delete"],
    );
    let operation = match operation_word.as_deref() {
        Some("update") => "update",
        Some("remove" | "rm" | "delete") => "delete",
        _ => "create",
    };
    let kv = invocation
        .kv
        .iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    let object_refs = crate::menu_syntax::payload::object_refs_for_raw_capture(
        &invocation.target,
        &invocation.raw,
    );
    let selected_snippet = primary_resolved_object_ref(
        &object_refs,
        crate::menu_syntax::CaptureObjectKind::Snippet,
    )?;
    let explicit_trigger = kv
        .get("trigger")
        .cloned()
        .filter(|value| !value.trim().is_empty());
    let trigger = if operation == "create" {
        explicit_trigger
    } else {
        explicit_trigger.or_else(|| selected_snippet.as_ref().map(|object_ref| object_ref.id.clone()))
    }
    .ok_or_else(|| "Add trigger:<shortcut> for the snippet.".to_string())?;
    if operation == "create" && body.trim().is_empty() {
        return Err("Add snippet body after --.".to_string());
    }
    let language = kv.get("lang").or_else(|| kv.get("language")).cloned();
    let existing = read_active_app_owned_jsonl_record_by_key_in_sk_path(
        sk_path,
        "snippets.jsonl",
        "trigger",
        &trigger,
    );
    if matches!(operation, "update" | "delete") && existing.is_none() {
        return Err("Snippet not found.".to_string());
    }
    let now = chrono::Local::now().to_rfc3339();
    let next_body = if body.trim().is_empty() {
        existing
            .as_ref()
            .and_then(|value| value.get("body"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        body.trim().to_string()
    };
    let next_body_lines = if body.trim().is_empty() {
        existing
            .as_ref()
            .and_then(|value| value.get("bodyLines"))
            .cloned()
            .unwrap_or_else(|| {
                serde_json::to_value(next_body.lines().collect::<Vec<_>>())
                    .unwrap_or_else(|_| serde_json::json!([]))
            })
    } else {
        serde_json::to_value(body.lines().collect::<Vec<_>>())
            .unwrap_or_else(|_| serde_json::json!([]))
    };
    let next_name = if body.trim().is_empty() {
        existing
            .as_ref()
            .and_then(|value| value.get("name"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        body.split_whitespace().next().unwrap_or("").to_string()
    };
    let record = serde_json::json!({
        "schema": "menu-syntax.snippet.v1",
        "kind": "snippet",
        "id": existing
            .as_ref()
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| app_owned_id("snippet")),
        "trigger": trigger,
        "language": language.or_else(|| {
            existing
                .as_ref()
                .and_then(|value| value.get("language"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        }),
        "name": next_name,
        "body": next_body,
        "bodyLines": next_body_lines,
        "tags": if invocation.tags.is_empty() {
            existing
                .as_ref()
                .and_then(|value| value.get("tags"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!([]))
        } else {
            serde_json::to_value(&invocation.tags).unwrap_or_else(|_| serde_json::json!([]))
        },
        "objectRefs": object_refs,
        "createdAt": existing
            .as_ref()
            .and_then(|value| value.get("createdAt"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| now.clone()),
        "updatedAt": now,
        "deletedAt": if operation == "delete" { serde_json::Value::String(now.clone()) } else { serde_json::Value::Null },
        "source": app_owned_source(invocation, "snippet", operation),
    });
    upsert_app_owned_jsonl_by_key_in_sk_path(sk_path, "snippets.jsonl", "trigger", &trigger, &record)?;
    Ok(match operation {
        "update" => "Updated snippet".to_string(),
        "delete" => "Removed snippet".to_string(),
        _ => "Saved snippet".to_string(),
    })
}

fn primary_resolved_object_ref(
    object_refs: &[crate::menu_syntax::CaptureObjectRef],
    kind: crate::menu_syntax::CaptureObjectKind,
) -> Result<Option<&crate::menu_syntax::CaptureObjectRef>, String> {
    let Some(object_ref) = object_refs.iter().find(|object_ref| object_ref.role == "primary")
    else {
        return Ok(None);
    };
    if !object_ref.resolved {
        return Ok(None);
    }
    if object_ref.kind != kind {
        return Err(format!(
            "Selected object is {}, expected {}.",
            object_ref.kind.as_str(),
            kind.as_str()
        ));
    }
    Ok((!object_ref.id.trim().is_empty()).then_some(object_ref))
}

fn primary_resolved_object_ref_id(
    object_refs: &[crate::menu_syntax::CaptureObjectRef],
    kind: crate::menu_syntax::CaptureObjectKind,
) -> Result<Option<String>, String> {
    Ok(primary_resolved_object_ref(object_refs, kind)?
        .map(|object_ref| object_ref.id.trim().to_string()))
}

fn resolve_for_app_owned(
    invocation: &crate::menu_syntax::CaptureInvocation,
) -> crate::menu_syntax::date::ResolvedCaptureInvocation {
    let accepts = vec![
        "date".to_string(),
        "relativeDate".to_string(),
        "duration".to_string(),
        "recurrence".to_string(),
        "priority".to_string(),
    ];
    let clock = crate::menu_syntax::MenuSyntaxClock::local_now();
    crate::menu_syntax::date::resolve_capture_dates_with_accepts(invocation, &clock, &accepts)
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

fn first_http_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|part| is_http_url(part))
        .map(ToString::to_string)
}

fn is_http_url(text: &str) -> bool {
    text.starts_with("http://") || text.starts_with("https://")
}

fn append_app_owned_jsonl(filename: &str, record: &serde_json::Value) -> Result<(), String> {
    append_app_owned_jsonl_in_sk_path(&default_app_owned_sk_path(), filename, record)
}

fn append_app_owned_jsonl_in_sk_path(
    sk_path: &std::path::Path,
    filename: &str,
    record: &serde_json::Value,
) -> Result<(), String> {
    use std::io::Write;
    let dir = sk_path.join("menu-syntax");
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Menu syntax: failed to create artifact dir: {err}"))?;
    let path = dir.join(filename);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| format!("Menu syntax: failed to open {}: {err}", path.display()))?;
    let line = serde_json::to_string(record)
        .map_err(|err| format!("Menu syntax: failed to serialize artifact: {err}"))?;
    writeln!(file, "{line}")
        .map_err(|err| format!("Menu syntax: failed to write {}: {err}", path.display()))
}

fn read_app_owned_jsonl_record_by_key(
    filename: &str,
    key_name: &str,
    key_value: &str,
) -> Option<serde_json::Value> {
    read_app_owned_jsonl_record_by_key_in_sk_path(
        &default_app_owned_sk_path(),
        filename,
        key_name,
        key_value,
    )
}

fn read_app_owned_jsonl_record_by_key_in_sk_path(
    sk_path: &std::path::Path,
    filename: &str,
    key_name: &str,
    key_value: &str,
) -> Option<serde_json::Value> {
    let path = sk_path.join("menu-syntax").join(filename);
    let contents = std::fs::read_to_string(path).ok()?;
    contents.lines().rev().find_map(|line| {
        let value = serde_json::from_str::<serde_json::Value>(line.trim()).ok()?;
        let matches_key = value
            .get(key_name)
            .and_then(|value| value.as_str())
            .map(|value| value == key_value)
            .unwrap_or(false);
        matches_key.then_some(value)
    })
}

fn read_active_app_owned_jsonl_record_by_key_in_sk_path(
    sk_path: &std::path::Path,
    filename: &str,
    key_name: &str,
    key_value: &str,
) -> Option<serde_json::Value> {
    read_app_owned_jsonl_record_by_key_in_sk_path(sk_path, filename, key_name, key_value)
        .filter(|record| !app_owned_record_is_deleted(record))
}

fn app_owned_record_is_deleted(record: &serde_json::Value) -> bool {
    record
        .get("deletedAt")
        .and_then(|value| value.as_str())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || record
            .get("status")
            .and_then(|value| value.as_str())
            .map(|value| value.eq_ignore_ascii_case("deleted"))
            .unwrap_or(false)
}

fn upsert_app_owned_jsonl_by_key(
    filename: &str,
    key_name: &str,
    key_value: &str,
    record: &serde_json::Value,
) -> Result<(), String> {
    upsert_app_owned_jsonl_by_key_in_sk_path(
        &default_app_owned_sk_path(),
        filename,
        key_name,
        key_value,
        record,
    )
}

fn upsert_app_owned_jsonl_by_key_in_sk_path(
    sk_path: &std::path::Path,
    filename: &str,
    key_name: &str,
    key_value: &str,
    record: &serde_json::Value,
) -> Result<(), String> {
    let dir = sk_path.join("menu-syntax");
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Menu syntax: failed to create artifact dir: {err}"))?;
    let path = dir.join(filename);
    let existing = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(format!(
                "Menu syntax: failed to read {}: {err}",
                path.display()
            ));
        }
    };

    let mut lines = Vec::new();
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let should_replace = serde_json::from_str::<serde_json::Value>(trimmed)
            .ok()
            .and_then(|value| {
                value
                    .get(key_name)
                    .and_then(|value| value.as_str())
                    .map(|value| value == key_value)
            })
            .unwrap_or(false);
        if !should_replace {
            lines.push(line.to_string());
        }
    }
    lines.push(
        serde_json::to_string(record)
            .map_err(|err| format!("Menu syntax: failed to serialize artifact: {err}"))?,
    );
    let mut contents = lines.join("\n");
    contents.push('\n');
    std::fs::write(&path, contents)
        .map_err(|err| format!("Menu syntax: failed to write {}: {err}", path.display()))
}

fn app_owned_source(
    invocation: &crate::menu_syntax::CaptureInvocation,
    canonical_target: &str,
    operation: &str,
) -> serde_json::Value {
    let resolution = crate::menu_syntax::payload::resolve_capture_target(&invocation.target);
    serde_json::json!({
        "kind": "menu-syntax",
        "raw": invocation.raw,
        "rawTarget": invocation.target,
        "canonicalTarget": resolution
            .as_ref()
            .map(|value| value.canonical_target_str())
            .unwrap_or(canonical_target),
        "targetAliasOf": resolution
            .as_ref()
            .and_then(|value| value.target_alias_of_str()),
        "operation": operation,
        "executor": "app-owned",
    })
}

fn app_owned_id(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::new_v4().simple())
}

fn default_app_owned_sk_path() -> std::path::PathBuf {
    if let Ok(path) = std::env::var(crate::setup::SK_PATH_ENV) {
        if !path.trim().is_empty() {
            return std::path::PathBuf::from(path);
        }
    }
    std::env::var("HOME")
        .map(|home| std::path::PathBuf::from(home).join(".scriptkit"))
        .unwrap_or_else(|_| std::path::PathBuf::from(".scriptkit"))
}

#[cfg(test)]
mod menu_syntax_builtin_execution_tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn invocation(input: &str) -> crate::menu_syntax::CaptureInvocation {
        match parse_capture(input) {
            CaptureParse::Ok(invocation) => invocation,
            CaptureParse::Incomplete(incomplete) => {
                panic!("capture should parse: {:?}", incomplete.kind)
            }
        }
    }

    fn with_sk_path<T>(f: impl FnOnce(&TempDir) -> T) -> T {
        let _guard = env_lock().lock().expect("env lock");
        let previous = std::env::var(crate::setup::SK_PATH_ENV).ok();
        let tmp = TempDir::new().expect("tempdir");
        std::env::set_var(crate::setup::SK_PATH_ENV, tmp.path());
        let result = f(&tmp);
        match previous {
            Some(value) => std::env::set_var(crate::setup::SK_PATH_ENV, value),
            None => std::env::remove_var(crate::setup::SK_PATH_ENV),
        }
        result
    }

    fn read_todo_lines(tmp: &TempDir) -> Vec<serde_json::Value> {
        let path = tmp.path().join("menu-syntax").join("todos.jsonl");
        std::fs::read_to_string(path)
            .expect("read todos")
            .lines()
            .map(|line| serde_json::from_str(line).expect("todo json"))
            .collect()
    }

    fn read_jsonl(tmp: &TempDir, filename: &str) -> Vec<serde_json::Value> {
        let path = tmp.path().join("menu-syntax").join(filename);
        std::fs::read_to_string(path)
            .expect("read jsonl")
            .lines()
            .map(|line| serde_json::from_str(line).expect("json line"))
            .collect()
    }

    #[test]
    fn snooze_selected_todo_updates_existing_row_in_place() {
        let tmp = TempDir::new().expect("tempdir");
        {
            let dir = tmp.path().join("menu-syntax");
            std::fs::create_dir_all(&dir).expect("mkdir");
            let todos_path = dir.join("todos.jsonl");
            std::fs::write(
                &todos_path,
                r#"{"schema":"menu-syntax.todo.v1","kind":"todo","id":"todo_existing","body":"Review PR","status":"open","tags":["work"],"createdAt":"2026-05-20T10:00:00Z","updatedAt":"2026-05-20T10:00:00Z","deletedAt":null}
"#,
            )
            .expect("seed todo");

            let input = ";snooze @todo:todo_existing in 30 minutes";
            let invocation = invocation(&input);
            let message = write_app_owned_todo_capture_in_sk_path(
                &invocation,
                crate::menu_syntax::CaptureOperation::Snooze,
                tmp.path(),
            )
            .expect("snooze selected todo");

            assert_eq!(message, "Updated todo (snooze)");
            let rows = read_todo_lines(&tmp);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["id"], "todo_existing");
            assert_eq!(rows[0]["body"], "Review PR");
            assert_eq!(rows[0]["source"]["operation"], "snooze");
            assert!(rows[0]["snoozeUntil"].as_str().is_some());
            assert!(rows[0]["due"].as_str().is_some());
            assert_eq!(rows[0]["objectRefs"][0]["id"], "todo_existing");
            assert_eq!(rows[0]["objectRefs"][0]["resolved"], true);
        }
    }

    #[test]
    fn snooze_selected_missing_todo_does_not_create_row() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(";snooze @todo:missing_todo tomorrow");
        let err = write_app_owned_todo_capture_in_sk_path(
            &invocation,
            crate::menu_syntax::CaptureOperation::Snooze,
            tmp.path(),
        )
        .expect_err("missing selected todo should fail");

        assert_eq!(err, "Todo not found.");
        let path = tmp.path().join("menu-syntax").join("todos.jsonl");
        assert!(!path.exists(), "missing selected todo must not create a row");
    }

    #[test]
    fn reminder_without_selected_todo_still_appends_new_row() {
        with_sk_path(|tmp| {
            let invocation = invocation(";reminder Walk dog tomorrow p1 #home");
            let message = write_app_owned_todo_capture(
                &invocation,
                crate::menu_syntax::CaptureOperation::Remind,
            )
            .expect("create reminder");

            assert_eq!(message, "Captured to todo (remind)");
            let rows = read_todo_lines(tmp);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["body"], "Walk dog");
            assert_eq!(rows[0]["source"]["canonicalTarget"], "todo");
            assert_eq!(rows[0]["source"]["targetAliasOf"], "todo");
            assert_eq!(rows[0]["source"]["operation"], "remind");
            assert!(rows[0]["remindAt"].as_str().is_some());
            assert_eq!(rows[0]["priority"], 1);
            assert_eq!(rows[0]["tags"][0], "home");
        });
    }

    #[test]
    fn selected_todo_ref_outside_app_owned_store_is_rejected() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(";defer @todo:not_a_real_todo tomorrow");
        let err = write_app_owned_todo_capture_in_sk_path(
                &invocation,
                crate::menu_syntax::CaptureOperation::Defer,
            tmp.path(),
        )
        .expect_err("missing todo should reject");
        assert_eq!(err, "Todo not found.");
    }

    #[test]
    fn link_update_uses_selected_link_ref_when_url_is_omitted() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("menu-syntax");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(
            dir.join("bookmarks.jsonl"),
            r#"{"schema":"menu-syntax.bookmark.v1","kind":"bookmark","id":"bookmark_existing","url":"https://example.com","title":"Old Example","tags":["docs"],"createdAt":"2026-05-20T10:00:00Z","updatedAt":"2026-05-20T10:00:00Z","deletedAt":null}
"#,
        )
        .expect("seed bookmark");

        let invocation = invocation(r#";link update @link:https://example.com title:"New Example""#);
        let message =
            write_app_owned_link_capture_in_sk_path(&invocation, tmp.path()).expect("update link");

        assert_eq!(message, "Updated link");
        let rows = read_jsonl(&tmp, "bookmarks.jsonl");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["url"], "https://example.com");
        assert_eq!(rows[0]["title"], "New Example");
        assert_eq!(rows[0]["tags"][0], "docs");
        assert_eq!(rows[0]["source"]["operation"], "update");
        assert_eq!(rows[0]["objectRefs"][0]["id"], "https://example.com");
    }

    #[test]
    fn link_delete_selected_missing_link_does_not_create_tombstone() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(";link delete @link:https://missing.example");
        let err = write_app_owned_link_capture_in_sk_path(&invocation, tmp.path())
            .expect_err("missing selected link should fail");

        assert_eq!(err, "Link not found.");
        let path = tmp.path().join("menu-syntax").join("bookmarks.jsonl");
        assert!(!path.exists(), "missing selected link must not create a row");
    }

    #[test]
    fn snippet_remove_selected_ref_preserves_existing_body_and_sets_deleted_at() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("menu-syntax");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(
            dir.join("snippets.jsonl"),
            r#"{"schema":"menu-syntax.snippet.v1","kind":"snippet","id":"snippet_existing","trigger":"fj","language":"ts","name":"fetch","body":"const res = await fetch(url)","bodyLines":["const res = await fetch(url)"],"tags":["code"],"createdAt":"2026-05-20T10:00:00Z","updatedAt":"2026-05-20T10:00:00Z","deletedAt":null}
"#,
        )
        .expect("seed snippet");

        let invocation = invocation(";snippet remove @snippet:fj");
        let message = write_app_owned_snippet_capture_in_sk_path(&invocation, tmp.path())
            .expect("remove selected snippet");

        assert_eq!(message, "Removed snippet");
        let rows = read_jsonl(&tmp, "snippets.jsonl");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["trigger"], "fj");
        assert_eq!(rows[0]["body"], "const res = await fetch(url)");
        assert_eq!(rows[0]["bodyLines"][0], "const res = await fetch(url)");
        assert_eq!(rows[0]["language"], "ts");
        assert_eq!(rows[0]["tags"][0], "code");
        assert!(rows[0]["deletedAt"].as_str().is_some());
        assert_eq!(rows[0]["source"]["operation"], "delete");
        assert_eq!(rows[0]["objectRefs"][0]["id"], "fj");
    }

    #[test]
    fn snippet_selected_missing_ref_does_not_create() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(";snippet update @snippet:missing -- const x = 1");
        let err = write_app_owned_snippet_capture_in_sk_path(&invocation, tmp.path())
            .expect_err("missing selected snippet should fail");

        assert_eq!(err, "Snippet not found.");
        let path = tmp.path().join("menu-syntax").join("snippets.jsonl");
        assert!(!path.exists(), "missing selected snippet must not create a row");
    }
}

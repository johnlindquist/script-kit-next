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
    append_app_owned_jsonl("todos.jsonl", &record)?;
    Ok(format!("Captured to todo ({})", operation.as_str()))
}

fn write_app_owned_link_capture(
    invocation: &crate::menu_syntax::CaptureInvocation,
) -> Result<String, String> {
    let (operation, body) = split_operation_word(
        &invocation.body,
        &["add", "create", "update", "remove", "rm", "delete"],
    );
    let operation = match operation.as_deref() {
        Some("remove" | "rm" | "delete") => "delete",
        Some("update") => "update",
        _ => "create",
    };
    let url = invocation
        .url
        .clone()
        .or_else(|| first_http_url(&body))
        .ok_or_else(|| "Add a valid http:// or https:// URL.".to_string())?;
    let existing = read_app_owned_jsonl_record_by_key("bookmarks.jsonl", "url", &url);
    let title = body
        .split_whitespace()
        .filter(|part| !part.eq(&url))
        .collect::<Vec<_>>()
        .join(" ");
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
        "title": if title.trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(title.trim().to_string()) },
        "body": null,
        "tags": invocation.tags,
        "createdAt": existing
            .as_ref()
            .and_then(|value| value.get("createdAt"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| now.clone()),
        "updatedAt": now,
        "deletedAt": if operation == "delete" { serde_json::Value::String(now.clone()) } else { serde_json::Value::Null },
        "source": app_owned_source(invocation, "link", operation),
    });
    upsert_app_owned_jsonl_by_key("bookmarks.jsonl", "url", &url, &record)?;
    Ok(if operation == "delete" {
        "Removed link".to_string()
    } else {
        "Saved link".to_string()
    })
}

fn write_app_owned_snippet_capture(
    invocation: &crate::menu_syntax::CaptureInvocation,
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
    let trigger = kv
        .get("trigger")
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Add trigger:<shortcut> for the snippet.".to_string())?;
    if operation == "create" && body.trim().is_empty() {
        return Err("Add snippet body after --.".to_string());
    }
    let language = kv.get("lang").or_else(|| kv.get("language")).cloned();
    let existing = read_app_owned_jsonl_record_by_key("snippets.jsonl", "trigger", &trigger);
    let now = chrono::Local::now().to_rfc3339();
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
        "language": language,
        "name": body.split_whitespace().next().unwrap_or("").to_string(),
        "body": body.trim(),
        "bodyLines": body.lines().collect::<Vec<_>>(),
        "tags": invocation.tags,
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
    upsert_app_owned_jsonl_by_key("snippets.jsonl", "trigger", &trigger, &record)?;
    Ok(match operation {
        "update" => "Updated snippet".to_string(),
        "delete" => "Removed snippet".to_string(),
        _ => "Saved snippet".to_string(),
    })
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
        .find(|part| part.starts_with("http://") || part.starts_with("https://"))
        .map(ToString::to_string)
}

fn append_app_owned_jsonl(filename: &str, record: &serde_json::Value) -> Result<(), String> {
    use std::io::Write;
    let dir = default_app_owned_sk_path().join("menu-syntax");
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
    let path = default_app_owned_sk_path()
        .join("menu-syntax")
        .join(filename);
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

fn upsert_app_owned_jsonl_by_key(
    filename: &str,
    key_name: &str,
    key_value: &str,
    record: &serde_json::Value,
) -> Result<(), String> {
    let dir = default_app_owned_sk_path().join("menu-syntax");
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

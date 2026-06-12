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
            crate::menu_syntax::CanonicalCaptureTarget::Todo => {
                write_app_owned_todo_capture(invocation, resolution.operation)
            }
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
                crate::brain::record_capture_signals(
                    &invocation.target,
                    &invocation.body,
                    &invocation.tags,
                );
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
    let result =
        crate::notes::menu_syntax_capture::apply_menu_syntax_todo_capture(invocation, operation)?;
    Ok(match result.operation {
        crate::notes::menu_syntax_capture::TodoCaptureOperation::Create => {
            "Captured to todo".to_string()
        }
        crate::notes::menu_syntax_capture::TodoCaptureOperation::Remind => {
            "Captured to todo (remind)".to_string()
        }
        crate::notes::menu_syntax_capture::TodoCaptureOperation::Snooze => {
            "Captured to todo (snooze)".to_string()
        }
        crate::notes::menu_syntax_capture::TodoCaptureOperation::Defer => {
            "Captured to todo (defer)".to_string()
        }
    })
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
    let draft = crate::menu_syntax::parse_link_scriptlet_capture(invocation)?;
    let outcome = match draft.operation {
        crate::menu_syntax::LinkScriptletOperation::Create
        | crate::menu_syntax::LinkScriptletOperation::Update => {
            crate::scriptlets::link_markdown_store::upsert_link_section(sk_path, &draft)?
        }
        crate::menu_syntax::LinkScriptletOperation::Delete => {
            crate::scriptlets::link_markdown_store::delete_link_section(sk_path, &draft)?
        }
    };
    Ok(match outcome.operation {
        crate::scriptlets::link_markdown_store::LinkStoreOperation::Created => {
            "Saved link".to_string()
        }
        crate::scriptlets::link_markdown_store::LinkStoreOperation::Updated => {
            "Updated link".to_string()
        }
        crate::scriptlets::link_markdown_store::LinkStoreOperation::Deleted => {
            "Removed link".to_string()
        }
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
        crate::notes::menu_syntax_capture::NoteCaptureOperation::Update => {
            "Updated note".to_string()
        }
        crate::notes::menu_syntax_capture::NoteCaptureOperation::Delete => {
            "Deleted note".to_string()
        }
    })
}

fn write_app_owned_snippet_capture_in_sk_path(
    invocation: &crate::menu_syntax::CaptureInvocation,
    sk_path: &std::path::Path,
) -> Result<String, String> {
    let draft = crate::menu_syntax::parse_snippet_scriptlet_capture(invocation)?;
    let outcome = match draft.operation {
        crate::menu_syntax::SnippetScriptletOperation::Create
        | crate::menu_syntax::SnippetScriptletOperation::Update => {
            crate::scriptlets::snippet_markdown_store::upsert_snippet_section(sk_path, &draft)?
        }
        crate::menu_syntax::SnippetScriptletOperation::Delete => {
            crate::scriptlets::snippet_markdown_store::delete_snippet_section(sk_path, &draft)?
        }
    };
    Ok(match outcome.operation {
        crate::scriptlets::snippet_markdown_store::SnippetStoreOperation::Created => {
            "Saved snippet".to_string()
        }
        crate::scriptlets::snippet_markdown_store::SnippetStoreOperation::Updated => {
            "Updated snippet".to_string()
        }
        crate::scriptlets::snippet_markdown_store::SnippetStoreOperation::Deleted => {
            "Removed snippet".to_string()
        }
    })
}

fn primary_resolved_object_ref(
    object_refs: &[crate::menu_syntax::CaptureObjectRef],
    kind: crate::menu_syntax::CaptureObjectKind,
) -> Result<Option<&crate::menu_syntax::CaptureObjectRef>, String> {
    let Some(object_ref) = object_refs
        .iter()
        .find(|object_ref| object_ref.role == "primary")
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
    let dir = sk_path.join("menu-syntax");
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Menu syntax: failed to create artifact dir: {err}"))?;
    let path = dir.join(filename);

    // Read existing file if it exists
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
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(_) => {
                lines.push(line.to_string());
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    line = %trimmed,
                    "Encountered corrupt line in JSONL during append. Skipping corrupt line."
                );
            }
        }
    }

    lines.push(
        serde_json::to_string(record)
            .map_err(|err| format!("Menu syntax: failed to serialize artifact: {err}"))?,
    );

    let mut contents = lines.join("\n");
    contents.push('\n');

    // Atomic write using temp file + rename
    let temp_filename = format!("{}.tmp-{}", filename, uuid::Uuid::new_v4());
    let temp_path = dir.join(&temp_filename);
    if let Err(e) = std::fs::write(&temp_path, &contents) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!(
            "Menu syntax: failed to write to temp file {}: {}",
            temp_path.display(),
            e
        ));
    }
    if let Err(e) = std::fs::rename(&temp_path, &path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!(
            "Menu syntax: failed to rename temp file {} to {}: {}",
            temp_path.display(),
            path.display(),
            e
        ));
    }
    Ok(())
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
    let contents = std::fs::read_to_string(&path).ok()?;
    contents.lines().rev().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(value) => {
                let matches_key = value
                    .get(key_name)
                    .and_then(|value| value.as_str())
                    .map(|value| value == key_value)
                    .unwrap_or(false);
                matches_key.then_some(value)
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    line = %trimmed,
                    "Failed to parse JSONL line. Skipping corrupt line."
                );
                None
            }
        }
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
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(value) => {
                let should_replace = value
                    .get(key_name)
                    .and_then(|value| value.as_str())
                    .map(|value| value == key_value)
                    .unwrap_or(false);
                if !should_replace {
                    lines.push(line.to_string());
                }
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    line = %trimmed,
                    "Encountered corrupt line in JSONL during upsert. Skipping corrupt line."
                );
            }
        }
    }
    lines.push(
        serde_json::to_string(record)
            .map_err(|err| format!("Menu syntax: failed to serialize artifact: {err}"))?,
    );
    let mut contents = lines.join("\n");
    contents.push('\n');

    // Atomic write using temp file + rename
    let temp_filename = format!("{}.tmp-{}", filename, uuid::Uuid::new_v4());
    let temp_path = dir.join(&temp_filename);
    if let Err(e) = std::fs::write(&temp_path, &contents) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!(
            "Menu syntax: failed to write to temp file {}: {}",
            temp_path.display(),
            e
        ));
    }
    if let Err(e) = std::fs::rename(&temp_path, &path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!(
            "Menu syntax: failed to rename temp file {} to {}: {}",
            temp_path.display(),
            path.display(),
            e
        ));
    }
    Ok(())
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
    use crate::menu_syntax::capture::{CaptureParse, parse_capture};
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

    fn read_day_page_task_lines(tmp: &TempDir) -> Vec<String> {
        let days_dir = tmp.path().join("brain").join("days");
        let mut lines = Vec::new();
        if !days_dir.exists() {
            return lines;
        }
        for entry in std::fs::read_dir(days_dir).expect("read days dir") {
            let entry = entry.expect("day entry");
            let content = std::fs::read_to_string(entry.path()).expect("read day page");
            lines.extend(
                content
                    .lines()
                    .filter(|line| line.contains("- [ ] "))
                    .map(str::to_string),
            );
        }
        lines
    }

    fn snooze_selected_todo_by_reference_is_not_supported_on_day_pages() {
        with_sk_path(|tmp| {
            let input = ";snooze @todo:todo_existing in 30 minutes";
            let invocation = invocation(&input);
            let err = write_app_owned_todo_capture(
                &invocation,
                crate::menu_syntax::CaptureOperation::Snooze,
            )
            .expect_err("selected todo update should fail");
            assert!(err.contains("not supported on day pages"));
            assert!(read_day_page_task_lines(tmp).is_empty());
        });
    }

    #[test]
    fn snooze_selected_missing_todo_does_not_create_row() {
        with_sk_path(|tmp| {
            let invocation = invocation(";snooze @todo:missing_todo tomorrow");
            let err = write_app_owned_todo_capture(
                &invocation,
                crate::menu_syntax::CaptureOperation::Snooze,
            )
            .expect_err("missing selected todo should fail");

            assert!(err.contains("not supported on day pages"));
            assert!(read_day_page_task_lines(tmp).is_empty());
        });
    }

    #[test]
    fn reminder_without_selected_todo_appends_day_page_task_line() {
        with_sk_path(|tmp| {
            let invocation = invocation(";reminder Walk dog tomorrow p1 #home");
            let message = write_app_owned_todo_capture(
                &invocation,
                crate::menu_syntax::CaptureOperation::Remind,
            )
            .expect("create reminder");

            assert_eq!(message, "Captured to todo (remind)");
            let lines = read_day_page_task_lines(tmp);
            assert_eq!(lines.len(), 1);
            assert!(lines[0].contains("- [ ] Walk dog"));
            assert!(lines[0].contains("#home"));
            assert!(lines[0].contains("due:"));
        });
    }

    #[test]
    fn selected_todo_ref_outside_app_owned_store_is_rejected() {
        with_sk_path(|tmp| {
            let invocation = invocation(";defer @todo:not_a_real_todo tomorrow");
            let err = write_app_owned_todo_capture(
                &invocation,
                crate::menu_syntax::CaptureOperation::Defer,
            )
            .expect_err("selected todo update should reject");
            assert!(err.contains("not supported on day pages"));
            assert!(read_day_page_task_lines(tmp).is_empty());
        });
    }

    #[test]
    fn link_create_writes_main_plugin_links_markdown() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(";link https://example.com Example description:Docs #docs");
        let message =
            write_app_owned_link_capture_in_sk_path(&invocation, tmp.path()).expect("create link");

        assert_eq!(message, "Saved link");
        let content = std::fs::read_to_string(
            crate::scriptlets::link_markdown_store::links_markdown_path(tmp.path()),
        )
        .expect("read links.md");
        assert!(content.contains("# Links"));
        assert!(content.contains("## Example"));
        assert!(content.contains(r#""url": "https://example.com""#));
        assert!(content.contains(r#""tool": "open""#));
        assert!(!tmp.path().join("menu-syntax/bookmarks.jsonl").exists());
    }

    #[test]
    fn link_update_selected_markdown_ref_updates_existing_section() {
        let tmp = TempDir::new().expect("tempdir");
        let create = invocation(";link https://example.com Old Example #docs");
        write_app_owned_link_capture_in_sk_path(&create, tmp.path()).expect("create link");

        let invocation =
            invocation(r#";link update @link:https://example.com title:"New Example""#);
        let message =
            write_app_owned_link_capture_in_sk_path(&invocation, tmp.path()).expect("update link");

        assert_eq!(message, "Updated link");
        let sections = crate::scriptlets::link_markdown_store::load_link_sections(
            &crate::scriptlets::link_markdown_store::links_markdown_path(tmp.path()),
        )
        .expect("sections");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].url.as_deref(), Some("https://example.com"));
        assert_eq!(sections[0].title, "New Example");
    }

    #[test]
    fn link_delete_selected_missing_link_does_not_create_tombstone() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(";link delete @link:https://missing.example");
        let err = write_app_owned_link_capture_in_sk_path(&invocation, tmp.path())
            .expect_err("missing selected link should fail");

        assert_eq!(err, "Link not found.");
        let path = crate::scriptlets::link_markdown_store::links_markdown_path(tmp.path());
        assert!(
            !path.exists(),
            "missing selected link must not create a row"
        );
    }

    #[test]
    fn snippet_create_writes_main_plugin_markdown() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(
            ";snippet Hello there! keyword:hi! description:Expand hi! to hello! name:Hi to Hello",
        );
        let message = write_app_owned_snippet_capture_in_sk_path(&invocation, tmp.path())
            .expect("create snippet");

        assert_eq!(message, "Saved snippet");
        let path = tmp
            .path()
            .join("plugins")
            .join("main")
            .join("scriptlets")
            .join("snippets.md");
        let content = std::fs::read_to_string(path).expect("read snippets.md");
        assert!(content.contains("## Hi to Hello"));
        assert!(content.contains("keyword: hi!"));
        assert!(content.contains("description: Expand hi! to hello!"));
        assert!(!content.contains(r#""keyword""#));
        assert!(!content.contains(r#""description""#));
        assert!(!content.contains(r#""tool""#));
        assert!(!content.contains('{'));
        assert!(!content.contains('}'));
        assert!(content.contains("Hello there!"));
        assert!(
            !tmp.path()
                .join("menu-syntax")
                .join("snippets.jsonl")
                .exists()
        );
    }

    #[test]
    fn snippet_update_selected_markdown_ref_updates_existing_section() {
        let tmp = TempDir::new().expect("tempdir");
        let create = invocation(";snippet Hello keyword:hi name:Hi");
        write_app_owned_snippet_capture_in_sk_path(&create, tmp.path()).expect("create snippet");

        let invocation = invocation(";snippet update @snippet:hi description:New desc");
        let message = write_app_owned_snippet_capture_in_sk_path(&invocation, tmp.path())
            .expect("update selected snippet");

        assert_eq!(message, "Updated snippet");
        let sections = crate::scriptlets::snippet_markdown_store::load_snippet_sections(
            &crate::scriptlets::snippet_markdown_store::snippets_markdown_path(tmp.path()),
        )
        .expect("sections");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].body, "Hello");
        assert_eq!(sections[0].description.as_deref(), Some("New desc"));
    }

    #[test]
    fn snippet_selected_missing_ref_does_not_create() {
        let tmp = TempDir::new().expect("tempdir");
        let invocation = invocation(";snippet update @snippet:missing -- const x = 1");
        let err = write_app_owned_snippet_capture_in_sk_path(&invocation, tmp.path())
            .expect_err("missing selected snippet should fail");

        assert_eq!(err, "Snippet not found.");
        let path = tmp
            .path()
            .join("plugins")
            .join("main")
            .join("scriptlets")
            .join("snippets.md");
        assert!(
            !path.exists(),
            "missing selected snippet must not create a row"
        );
    }
}

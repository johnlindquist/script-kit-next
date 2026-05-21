use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::menu_syntax::{
    ObjectSelectorCandidate, SnippetLookup, SnippetScriptletDraft, SnippetScriptletOperation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetMarkdownSection {
    pub name: String,
    pub id: String,
    pub keyword: Option<String>,
    pub description: Option<String>,
    pub metadata: Map<String, Value>,
    pub body: String,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnippetStoreOperation {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetStoreOutcome {
    pub operation: SnippetStoreOperation,
    pub path: PathBuf,
}

pub fn snippets_markdown_path(sk_path: &Path) -> PathBuf {
    sk_path.join("plugins/main/scriptlets/snippets.md")
}

pub fn load_snippet_sections(path: &Path) -> Result<Vec<SnippetMarkdownSection>, String> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("Read snippets.md failed: {error}")),
    };
    Ok(parse_sections(&content))
}

pub fn upsert_snippet_section(
    sk_path: &Path,
    draft: &SnippetScriptletDraft,
) -> Result<SnippetStoreOutcome, String> {
    if matches!(draft.operation, SnippetScriptletOperation::Create) {
        if draft.body.as_deref().unwrap_or("").trim().is_empty() {
            return Err("Add snippet text.".to_string());
        }
        if draft.name.as_deref().unwrap_or("").trim().is_empty() {
            return Err("Add name:<snippet name>.".to_string());
        }
    }
    let path = snippets_markdown_path(sk_path);
    let original = read_or_default(&path)?;
    let sections = parse_sections(&original);
    let match_index = matching_section_index(&sections, draft)?;
    if !matches!(draft.operation, SnippetScriptletOperation::Create) && match_index.is_none() {
        return Err("Snippet not found.".to_string());
    }
    let existing = match_index.and_then(|index| sections.get(index));
    let mut metadata = existing
        .map(|section| section.metadata.clone())
        .unwrap_or_default();
    for (key, value) in &draft.metadata {
        metadata.insert(key.clone(), value.clone());
    }
    let name = draft
        .name
        .clone()
        .or_else(|| existing.map(|section| section.name.clone()))
        .ok_or_else(|| "Add name:<snippet name>.".to_string())?;
    metadata.insert("name".to_string(), Value::String(name.clone()));
    if !metadata.contains_key("tool") {
        metadata.insert("tool".to_string(), Value::String("paste".to_string()));
    }
    let body = draft
        .body
        .clone()
        .or_else(|| existing.map(|section| section.body.clone()))
        .unwrap_or_default();
    if body.trim().is_empty() {
        return Err("Add snippet text.".to_string());
    }
    let next_section = render_section(&name, &metadata, &body)?;
    let next = replace_or_append_section(&original, existing, &next_section);
    atomic_write(&path, &next)?;
    Ok(SnippetStoreOutcome {
        operation: if existing.is_some() {
            SnippetStoreOperation::Updated
        } else {
            SnippetStoreOperation::Created
        },
        path,
    })
}

pub fn delete_snippet_section(
    sk_path: &Path,
    draft: &SnippetScriptletDraft,
) -> Result<SnippetStoreOutcome, String> {
    let path = snippets_markdown_path(sk_path);
    let original = read_or_default(&path)?;
    let sections = parse_sections(&original);
    let Some(index) = matching_section_index(&sections, draft)? else {
        return Err("Snippet not found.".to_string());
    };
    let section = &sections[index];
    let mut next = String::new();
    next.push_str(&original[..section.start]);
    next.push_str(original[section.end..].trim_start_matches('\n'));
    atomic_write(&path, &next)?;
    Ok(SnippetStoreOutcome {
        operation: SnippetStoreOperation::Deleted,
        path,
    })
}

pub fn snippet_object_candidates_from_markdown(
    sk_path: &Path,
) -> Result<Vec<ObjectSelectorCandidate>, String> {
    let path = snippets_markdown_path(sk_path);
    let mut candidates = load_snippet_sections(&path)?
        .into_iter()
        .map(|section| ObjectSelectorCandidate {
            kind: crate::menu_syntax::CaptureObjectKind::Snippet,
            id: section.id,
            label: section.name,
            subtitle: section
                .keyword
                .as_deref()
                .map(|keyword| {
                    section
                        .description
                        .as_deref()
                        .map(|description| format!("{keyword} · {description}"))
                        .unwrap_or_else(|| keyword.to_string())
                })
                .or(section.description)
                .unwrap_or_else(|| "Snippet".to_string()),
        })
        .collect::<Vec<_>>();
    candidates.extend(legacy_jsonl_candidates(sk_path));
    candidates.sort_by(|a, b| {
        a.label
            .to_ascii_lowercase()
            .cmp(&b.label.to_ascii_lowercase())
    });
    candidates.dedup_by(|a, b| a.id == b.id);
    Ok(candidates)
}

fn read_or_default(path: &Path) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok("# Snippets\n\n".to_string())
        }
        Err(error) => Err(format!("Read snippets.md failed: {error}")),
    }
}

fn parse_sections(content: &str) -> Vec<SnippetMarkdownSection> {
    let mut heads = content
        .match_indices("\n## ")
        .map(|(idx, _)| idx + 1)
        .collect::<Vec<_>>();
    if content.starts_with("## ") {
        heads.insert(0, 0);
    }
    heads
        .iter()
        .enumerate()
        .filter_map(|(idx, start)| {
            let end = heads.get(idx + 1).copied().unwrap_or(content.len());
            parse_section(content, *start, end)
        })
        .collect()
}

fn parse_section(content: &str, start: usize, end: usize) -> Option<SnippetMarkdownSection> {
    let section = &content[start..end];
    let first_line_end = section.find('\n').unwrap_or(section.len());
    let name = section[..first_line_end]
        .trim_start_matches("## ")
        .trim()
        .to_string();
    if name.is_empty() {
        return None;
    }
    let metadata = parse_metadata_fence(section).unwrap_or_default();
    let body = parse_body_fence(section).unwrap_or_default();
    let keyword = metadata
        .get("keyword")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let description = metadata
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let id = keyword.clone().unwrap_or_else(|| slugify(&name));
    Some(SnippetMarkdownSection {
        name,
        id,
        keyword,
        description,
        metadata,
        body,
        start,
        end,
    })
}

fn parse_metadata_fence(section: &str) -> Option<Map<String, Value>> {
    let content = fence_content(section, "metadata")?;
    serde_json::from_str::<Map<String, Value>>(content.trim()).ok()
}

fn parse_body_fence(section: &str) -> Option<String> {
    for lang in ["paste", "type", "kit", "ts", "js", "bash"] {
        if let Some(content) = fence_content(section, lang) {
            return Some(content.trim_end_matches('\n').to_string());
        }
    }
    None
}

fn fence_content<'a>(section: &'a str, lang: &str) -> Option<&'a str> {
    let marker = format!("```{lang}");
    let start = section.find(&marker)?;
    let after_marker = start + marker.len();
    let content_start = section[after_marker..].find('\n')? + after_marker + 1;
    let content_end = section[content_start..].find("\n```")? + content_start;
    Some(&section[content_start..content_end])
}

fn matching_section_index(
    sections: &[SnippetMarkdownSection],
    draft: &SnippetScriptletDraft,
) -> Result<Option<usize>, String> {
    let by_lookup = match &draft.lookup {
        Some(SnippetLookup::SelectedRef(id)) => find_by_id(sections, id),
        Some(SnippetLookup::Keyword(keyword)) => find_by_id(sections, keyword),
        Some(SnippetLookup::Name(name)) => find_by_name(sections, name),
        None => None,
    };
    let by_name = draft
        .name
        .as_deref()
        .and_then(|name| find_by_name(sections, name));
    let by_keyword = draft
        .keyword
        .as_deref()
        .and_then(|keyword| find_by_id(sections, keyword));
    if let (Some(name_idx), Some(keyword_idx)) = (by_name, by_keyword) {
        if name_idx != keyword_idx {
            return Err(
                "Snippet name and keyword match different snippets. Select one with @.".to_string(),
            );
        }
    }
    Ok(by_lookup.or(by_keyword).or(by_name))
}

fn find_by_id(sections: &[SnippetMarkdownSection], id: &str) -> Option<usize> {
    sections.iter().position(|section| {
        section.id == id
            || section
                .keyword
                .as_deref()
                .map(|keyword| keyword == id)
                .unwrap_or(false)
            || slugify(&section.name) == id
    })
}

fn find_by_name(sections: &[SnippetMarkdownSection], name: &str) -> Option<usize> {
    sections.iter().position(|section| {
        section.name.eq_ignore_ascii_case(name) || slugify(&section.name) == slugify(name)
    })
}

fn replace_or_append_section(
    original: &str,
    existing: Option<&SnippetMarkdownSection>,
    next_section: &str,
) -> String {
    if let Some(existing) = existing {
        let mut next = String::new();
        next.push_str(&original[..existing.start]);
        next.push_str(next_section);
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(original[existing.end..].trim_start_matches('\n'));
        return next;
    }
    let mut next = if original.trim().is_empty() {
        "# Snippets\n\n".to_string()
    } else {
        original.to_string()
    };
    if !next.ends_with("\n\n") {
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push('\n');
    }
    next.push_str(next_section);
    next
}

fn render_section(name: &str, metadata: &Map<String, Value>, body: &str) -> Result<String, String> {
    let metadata_json = serde_json::to_string_pretty(metadata)
        .map_err(|error| format!("Serialize snippet metadata failed: {error}"))?;
    let fence = fence_for(&[&metadata_json, body]);
    Ok(format!(
        "## {name}\n\n{fence}metadata\n{metadata_json}\n{fence}\n\n{fence}paste\n{body}\n{fence}\n"
    ))
}

fn fence_for(parts: &[&str]) -> String {
    let longest = parts
        .iter()
        .flat_map(|part| part.split(|ch| ch != '`'))
        .map(str::len)
        .max()
        .unwrap_or(0);
    "`".repeat(longest.max(3) + 1)
}

fn atomic_write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Create snippet dir failed: {error}"))?;
    }
    let tmp = path.with_extension("md.tmp");
    std::fs::write(&tmp, contents).map_err(|error| format!("Write snippets.md failed: {error}"))?;
    std::fs::rename(&tmp, path).map_err(|error| format!("Replace snippets.md failed: {error}"))
}

fn legacy_jsonl_candidates(sk_path: &Path) -> Vec<ObjectSelectorCandidate> {
    let path = sk_path.join("menu-syntax/snippets.jsonl");
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| {
            !value
                .get("deletedAt")
                .and_then(Value::as_str)
                .map(|deleted| !deleted.trim().is_empty())
                .unwrap_or(false)
        })
        .filter_map(|value| {
            let id = value
                .get("trigger")
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            if id.is_empty() {
                return None;
            }
            let label = value
                .get("name")
                .or_else(|| value.get("body"))
                .and_then(Value::as_str)
                .unwrap_or(&id)
                .trim()
                .to_string();
            Some(ObjectSelectorCandidate {
                kind: crate::menu_syntax::CaptureObjectKind::Snippet,
                id,
                label,
                subtitle: value
                    .get("language")
                    .and_then(Value::as_str)
                    .map(|lang| format!("Snippet · {lang}"))
                    .unwrap_or_else(|| "Legacy snippet".to_string()),
            })
        })
        .collect()
}

fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};
    use tempfile::TempDir;

    fn draft(input: &str) -> SnippetScriptletDraft {
        let invocation = match parse_capture(input) {
            CaptureParse::Ok(invocation) => invocation,
            CaptureParse::Incomplete(incomplete) => panic!("{incomplete:?}"),
        };
        crate::menu_syntax::parse_snippet_scriptlet_capture(&invocation).expect("draft")
    }

    #[test]
    fn create_initializes_plugins_main_scriptlets_snippets_md() {
        let tmp = TempDir::new().unwrap();
        let draft = draft(
            ";snippet Hello there! keyword:hi! description:Expand hi! to hello! name:Hi to Hello",
        );

        let outcome = upsert_snippet_section(tmp.path(), &draft).unwrap();

        assert_eq!(outcome.operation, SnippetStoreOperation::Created);
        let content = std::fs::read_to_string(snippets_markdown_path(tmp.path())).unwrap();
        assert!(content.contains("## Hi to Hello"));
        assert!(content.contains(r#""keyword": "hi!""#));
        assert!(content.contains(r#""description": "Expand hi! to hello!""#));
        assert!(content.contains("````paste\nHello there!\n````"));
        assert!(!tmp.path().join("menu-syntax/snippets.jsonl").exists());
    }

    #[test]
    fn create_is_idempotent_by_keyword() {
        let tmp = TempDir::new().unwrap();
        upsert_snippet_section(tmp.path(), &draft(";snippet Hello keyword:hi name:Hi")).unwrap();
        upsert_snippet_section(tmp.path(), &draft(";snippet Updated keyword:hi name:Hi")).unwrap();
        let sections = load_snippet_sections(&snippets_markdown_path(tmp.path())).unwrap();

        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].body, "Updated");
    }

    #[test]
    fn update_by_selected_ref_preserves_body_when_body_missing() {
        let tmp = TempDir::new().unwrap();
        upsert_snippet_section(tmp.path(), &draft(";snippet Hello keyword:hi name:Hi")).unwrap();

        upsert_snippet_section(
            tmp.path(),
            &draft(";snippet update @snippet:hi description:New desc"),
        )
        .unwrap();
        let sections = load_snippet_sections(&snippets_markdown_path(tmp.path())).unwrap();

        assert_eq!(sections[0].body, "Hello");
        assert_eq!(sections[0].description.as_deref(), Some("New desc"));
    }

    #[test]
    fn delete_removes_only_selected_section() {
        let tmp = TempDir::new().unwrap();
        upsert_snippet_section(tmp.path(), &draft(";snippet Hello keyword:hi name:Hi")).unwrap();
        upsert_snippet_section(tmp.path(), &draft(";snippet Bye keyword:bye name:Bye")).unwrap();

        delete_snippet_section(tmp.path(), &draft(";snippet delete @snippet:hi")).unwrap();
        let sections = load_snippet_sections(&snippets_markdown_path(tmp.path())).unwrap();

        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "Bye");
    }

    #[test]
    fn body_with_triple_backticks_uses_longer_fence() {
        let tmp = TempDir::new().unwrap();
        upsert_snippet_section(tmp.path(), &draft(";snippet name:Ticks -- hello ``` world"))
            .unwrap();
        let content = std::fs::read_to_string(snippets_markdown_path(tmp.path())).unwrap();

        assert!(content.contains("````paste"));
    }
}

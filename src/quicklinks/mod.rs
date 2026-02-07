//! Quicklinks data model and JSON storage.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const QUICKLINKS_FILE: &str = "quicklinks.json";
const QUERY_PLACEHOLDER: &str = "{query}";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Quicklink {
    pub id: String,
    pub name: String,
    pub url_template: String,
    #[serde(default)]
    pub icon: Option<String>,
}

pub fn load_quicklinks() -> Vec<Quicklink> {
    load_quicklinks_from_path(&quicklinks_path())
}

pub fn save_quicklinks(links: &[Quicklink]) {
    let _ = save_quicklinks_to_path(links, &quicklinks_path());
}

pub fn create_quicklink(name: &str, url_template: &str) -> Quicklink {
    Quicklink {
        id: Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        url_template: url_template.trim().to_string(),
        icon: None,
    }
}

pub fn delete_quicklink(id: &str) {
    delete_quicklink_from_path(id, &quicklinks_path());
}

pub fn expand_url(url_template: &str, query: &str) -> String {
    url_template.replace(QUERY_PLACEHOLDER, query)
}

fn quicklinks_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".scriptkit").join(QUICKLINKS_FILE))
        .unwrap_or_else(|| PathBuf::from(".scriptkit").join(QUICKLINKS_FILE))
}

fn load_quicklinks_from_path(path: &Path) -> Vec<Quicklink> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };

    serde_json::from_str(&content).unwrap_or_default()
}

fn save_quicklinks_to_path(links: &[Quicklink], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(links)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    fs::write(path, content)
}

fn delete_quicklink_from_path(id: &str, path: &Path) {
    let mut links = load_quicklinks_from_path(path);
    links.retain(|link| link.id != id);
    let _ = save_quicklinks_to_path(&links, path);
}

#[cfg(test)]
mod tests {
    use super::{
        create_quicklink, delete_quicklink_from_path, expand_url, load_quicklinks_from_path,
        save_quicklinks_to_path, Quicklink,
    };

    #[test]
    fn test_create_quicklink_sets_fields_and_defaults_icon() {
        let quicklink = create_quicklink(" Search ", " https://example.com?q={query} ");

        assert!(!quicklink.id.is_empty());
        assert_eq!(quicklink.name, "Search");
        assert_eq!(quicklink.url_template, "https://example.com?q={query}");
        assert_eq!(quicklink.icon, None);
    }

    #[test]
    fn test_expand_url_replaces_query_placeholder() {
        let expanded = expand_url("https://example.com/search?q={query}", "rust gpui");
        assert_eq!(expanded, "https://example.com/search?q=rust gpui");
    }

    #[test]
    fn test_expand_url_returns_original_when_placeholder_missing() {
        let expanded = expand_url("https://example.com", "ignored");
        assert_eq!(expanded, "https://example.com");
    }

    #[test]
    fn test_load_quicklinks_from_path_returns_empty_when_file_missing() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("quicklinks.json");

        let loaded = load_quicklinks_from_path(&path);
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_load_quicklinks_from_path_returns_empty_for_invalid_json() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("quicklinks.json");
        std::fs::write(&path, "{ not-valid-json ]").expect("write invalid json");

        let loaded = load_quicklinks_from_path(&path);
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_save_and_load_quicklinks_roundtrip() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("quicklinks.json");
        let links = vec![
            Quicklink {
                id: "1".to_string(),
                name: "Docs".to_string(),
                url_template: "https://docs.rs".to_string(),
                icon: None,
            },
            Quicklink {
                id: "2".to_string(),
                name: "Search".to_string(),
                url_template: "https://google.com/search?q={query}".to_string(),
                icon: Some("search".to_string()),
            },
        ];

        save_quicklinks_to_path(&links, &path).expect("save quicklinks");
        let loaded = load_quicklinks_from_path(&path);

        assert_eq!(loaded, links);
    }

    #[test]
    fn test_delete_quicklink_from_path_removes_matching_id() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("quicklinks.json");
        let links = vec![
            Quicklink {
                id: "keep".to_string(),
                name: "Keep".to_string(),
                url_template: "https://example.com".to_string(),
                icon: None,
            },
            Quicklink {
                id: "remove".to_string(),
                name: "Remove".to_string(),
                url_template: "https://example.org".to_string(),
                icon: None,
            },
        ];

        save_quicklinks_to_path(&links, &path).expect("seed quicklinks");
        delete_quicklink_from_path("remove", &path);

        let loaded = load_quicklinks_from_path(&path);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "keep");
    }
}

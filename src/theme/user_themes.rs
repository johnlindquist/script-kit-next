//! User-authored themes stored under `~/.scriptkit/themes/<slug>.json`.
//!
//! Each JSON file holds a `name`, `appearance`, and the same token shape the
//! bundled presets and the user-owned `~/.scriptkit/theme.json` override
//! use. Saving a theme first runs the hover/selected validation in
//! [`crate::theme::validation`] so the row-state opacity contract cannot be
//! broken by a user-authored file.
//!
//! The launcher-facing theme chooser merges the list returned by
//! [`list_user_themes`] into the built-in preset catalog so user themes show
//! up alongside the bundled ones and can be selected, applied, or duplicated
//! without leaving the app.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::theme::Theme;

/// Returns the directory that holds user-authored themes.
///
/// The directory lives at `~/.scriptkit/themes/`. It is seeded at app
/// startup so listing it never needs to create it on the hot path.
pub fn user_themes_dir() -> PathBuf {
    crate::setup::themes_dir()
}

/// Ensures the user-themes directory exists. Call once during setup.
pub fn ensure_user_themes_dir() -> Result<()> {
    let dir = user_themes_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("creating user themes dir {}", dir.display()))?;
    }
    Ok(())
}

/// Metadata describing a single user-authored theme on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserTheme {
    /// Slug derived from the file stem (lowercase, `-`-separated).
    pub slug: String,
    /// Human-readable name stored inside the JSON payload.
    pub name: String,
    /// Absolute path to the theme file.
    pub path: PathBuf,
}

/// Lists every user theme under [`user_themes_dir`]. Corrupt or unreadable
/// files are skipped so one bad file cannot hide the rest.
pub fn list_user_themes() -> Vec<UserTheme> {
    let dir = user_themes_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut themes: Vec<UserTheme> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                return None;
            }
            let slug = path.file_stem()?.to_string_lossy().into_owned();
            let contents = fs::read_to_string(&path).ok()?;
            let value: Value = serde_json::from_str(&contents).ok()?;
            let name = value
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(slug.as_str())
                .to_string();
            Some(UserTheme { slug, name, path })
        })
        .collect();

    themes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    themes
}

/// Looks up a user theme by slug. Returns `None` if the theme file is
/// missing or cannot be parsed.
pub fn find_user_theme(slug: &str) -> Option<UserTheme> {
    list_user_themes().into_iter().find(|t| t.slug == slug)
}

/// Loads a user-authored theme by slug. Invalid files are ignored, matching
/// list behavior so one broken custom theme cannot break Theme Designer.
pub fn load_user_theme(slug: &str) -> Option<Theme> {
    let path = user_themes_dir().join(format!("{slug}.json"));
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Theme>(&contents).ok()
}

/// Normalizes a display name into a filesystem-safe slug.
pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        slug.push_str("theme");
    }
    slug
}

/// Saves a user theme JSON payload to disk after validating the row-state
/// opacity contract in [`crate::theme::validation`]. Returns the saved
/// [`UserTheme`] metadata on success.
///
/// The payload must be a serializable JSON object that carries a `name` key
/// and a tokens object matching the shape of the bundled presets. The caller
/// is responsible for producing that shape — this helper only owns storage,
/// name-based slugging, and validation.
pub fn save_user_theme(name: &str, payload: &Value) -> Result<UserTheme> {
    ensure_user_themes_dir()?;

    validate_payload(payload).context("validating theme before save")?;

    let slug = slugify(name);
    let path = user_themes_dir().join(format!("{slug}.json"));

    let mut enriched = payload.clone();
    if let Value::Object(map) = &mut enriched {
        map.insert("name".to_string(), Value::String(name.to_string()));
    }

    let serialized =
        serde_json::to_string_pretty(&enriched).context("serializing user theme json")?;
    atomic_write(&path, &serialized)?;

    Ok(UserTheme {
        slug,
        name: name.to_string(),
        path,
    })
}

/// Saves a user theme without overwriting an existing slug.
pub fn save_user_theme_unique(name: &str, payload: &Value) -> Result<UserTheme> {
    ensure_user_themes_dir()?;
    let base_slug = slugify(name);
    let mut slug = base_slug.clone();
    let mut suffix = 2usize;
    while user_themes_dir().join(format!("{slug}.json")).exists() {
        slug = format!("{base_slug}-{suffix}");
        suffix += 1;
    }
    let display_name = if slug == base_slug {
        name.to_string()
    } else {
        format!("{name} {}", suffix - 1)
    };

    validate_payload(payload).context("validating theme before save")?;
    let path = user_themes_dir().join(format!("{slug}.json"));
    let mut enriched = payload.clone();
    if let Value::Object(map) = &mut enriched {
        map.insert("name".to_string(), Value::String(display_name.clone()));
    }
    let serialized =
        serde_json::to_string_pretty(&enriched).context("serializing user theme json")?;
    atomic_write(&path, &serialized)?;

    Ok(UserTheme {
        slug,
        name: display_name,
        path,
    })
}

/// Saves the current in-memory theme as a user-authored preset.
pub fn save_theme_as_user_theme(name: &str, theme: &Theme) -> Result<UserTheme> {
    let payload = serde_json::to_value(theme).context("serializing current theme")?;
    save_user_theme_unique(name, &payload)
}

/// Deletes a user theme by slug. Missing files are treated as success so the
/// caller does not need to guard against pre-existing deletion races.
pub fn delete_user_theme(slug: &str) -> Result<()> {
    let path = user_themes_dir().join(format!("{slug}.json"));
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("deleting user theme {}", path.display()))?;
    }
    Ok(())
}

fn atomic_write(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .context("user theme path has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating theme dir {}", parent.display()))?;
    let tmp = parent.join(format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("theme.json")
    ));
    fs::write(&tmp, contents)
        .with_context(|| format!("writing theme tmp file {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("finalizing theme file {}", path.display()))?;
    Ok(())
}

/// Minimal structural validation. Rejects payloads where the hover opacity is
/// not strictly less than the selected opacity, matching the row-state
/// contract documented in `removed-docs`.
fn validate_payload(payload: &Value) -> Result<()> {
    let Some(opacity) = payload
        .get("opacity")
        .or_else(|| payload.get("tokens").and_then(|t| t.get("opacity")))
    else {
        return Ok(());
    };

    let hover = opacity
        .get("hover")
        .or_else(|| opacity.get("row_state_hover"))
        .or_else(|| opacity.get("rowStateHover"))
        .and_then(|v| v.as_f64());
    let selected = opacity
        .get("selected")
        .or_else(|| opacity.get("row_state_selected"))
        .or_else(|| opacity.get("rowStateSelected"))
        .and_then(|v| v.as_f64());

    if let (Some(h), Some(s)) = (hover, selected) {
        if h >= s {
            anyhow::bail!(
                "invalid theme: hover opacity ({h}) must be strictly less than selected opacity ({s})"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn slugify_normalizes_names() {
        assert_eq!(slugify("My Theme"), "my-theme");
        assert_eq!(slugify("  Extra   Spaces  "), "extra-spaces");
        assert_eq!(slugify("!!!"), "theme");
        assert_eq!(slugify("Foo/Bar Baz"), "foo-bar-baz");
    }

    #[test]
    fn validate_payload_rejects_hover_ge_selected() {
        let bad = json!({
            "opacity": { "row_state_hover": 0.25, "row_state_selected": 0.23 }
        });
        assert!(validate_payload(&bad).is_err());
    }

    #[test]
    fn validate_payload_accepts_hover_below_selected() {
        let good = json!({
            "opacity": { "row_state_hover": 0.06, "row_state_selected": 0.23 }
        });
        assert!(validate_payload(&good).is_ok());
    }

    #[test]
    fn validate_payload_checks_current_hover_selected_shape() {
        let bad = json!({
            "opacity": { "hover": 0.25, "selected": 0.23 }
        });
        assert!(validate_payload(&bad).is_err());
    }
}

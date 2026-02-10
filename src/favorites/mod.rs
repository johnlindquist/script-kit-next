use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Favorites {
    pub script_ids: Vec<String>,
}

fn favorites_file_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".scriptkit").join("favorites.json"))
        .unwrap_or_else(|| PathBuf::from(".scriptkit").join("favorites.json"))
}

fn load_favorites_from_path(path: &Path) -> Result<Favorites> {
    if !path.exists() {
        return Ok(Favorites::default());
    }

    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read favorites file at {}", path.display()))?;

    serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse favorites JSON at {}", path.display()))
}

#[allow(dead_code)]
fn save_favorites_to_path(path: &Path, favorites: &Favorites) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create favorites directory at {}",
                parent.display()
            )
        })?;
    }

    let json = serde_json::to_string_pretty(favorites)
        .context("failed to serialize favorites for writing")?;

    fs::write(path, json)
        .with_context(|| format!("failed to write favorites file at {}", path.display()))?;
    Ok(())
}

#[allow(dead_code)]
fn toggle_favorite_in_path(path: &Path, id: &str) -> Result<Favorites> {
    let mut favorites = load_favorites_from_path(path)?;

    if let Some(index) = favorites
        .script_ids
        .iter()
        .position(|script_id| script_id == id)
    {
        favorites.script_ids.remove(index);
    } else {
        favorites.script_ids.push(id.to_string());
    }

    save_favorites_to_path(path, &favorites)?;
    Ok(favorites)
}

fn is_favorite_in_path(path: &Path, id: &str) -> Result<bool> {
    let favorites = load_favorites_from_path(path)?;
    Ok(favorites.script_ids.iter().any(|script_id| script_id == id))
}

pub fn load_favorites() -> Result<Favorites> {
    load_favorites_from_path(&favorites_file_path())
}

#[allow(dead_code)]
pub fn save_favorites(favorites: &Favorites) -> Result<()> {
    save_favorites_to_path(&favorites_file_path(), favorites)
}

#[allow(dead_code)]
pub fn toggle_favorite(id: &str) -> Result<()> {
    toggle_favorite_in_path(&favorites_file_path(), id).map(|_| ())
}

pub fn is_favorite(id: &str) -> bool {
    is_favorite_in_path(&favorites_file_path(), id).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_favorites_path(temp_root: &Path) -> PathBuf {
        temp_root.join(".scriptkit").join("favorites.json")
    }

    #[test]
    fn test_load_favorites_returns_empty_when_file_missing() {
        let temp = tempdir().expect("tempdir should be created");
        let favorites =
            load_favorites_from_path(&test_favorites_path(temp.path())).expect("load should work");

        assert_eq!(favorites, Favorites::default());
    }

    #[test]
    fn test_save_favorites_persists_ids_for_round_trip() {
        let temp = tempdir().expect("tempdir should be created");
        let path = test_favorites_path(temp.path());
        let expected = Favorites {
            script_ids: vec!["script-a".to_string(), "script-b".to_string()],
        };

        save_favorites_to_path(&path, &expected).expect("save should work");
        let loaded = load_favorites_from_path(&path).expect("load should work");

        assert_eq!(loaded, expected);
    }

    #[test]
    fn test_toggle_favorite_adds_id_when_not_present() {
        let temp = tempdir().expect("tempdir should be created");
        let path = test_favorites_path(temp.path());

        toggle_favorite_in_path(&path, "script-a").expect("toggle should work");
        let loaded = load_favorites_from_path(&path).expect("load should work");

        assert_eq!(loaded.script_ids, vec!["script-a"]);
    }

    #[test]
    fn test_toggle_favorite_removes_id_when_present() {
        let temp = tempdir().expect("tempdir should be created");
        let path = test_favorites_path(temp.path());

        save_favorites_to_path(
            &path,
            &Favorites {
                script_ids: vec!["script-a".to_string()],
            },
        )
        .expect("initial save should work");

        toggle_favorite_in_path(&path, "script-a").expect("toggle should work");
        let loaded = load_favorites_from_path(&path).expect("load should work");

        assert!(loaded.script_ids.is_empty());
    }

    #[test]
    fn test_is_favorite_returns_true_when_id_exists() {
        let temp = tempdir().expect("tempdir should be created");
        let path = test_favorites_path(temp.path());

        save_favorites_to_path(
            &path,
            &Favorites {
                script_ids: vec!["script-a".to_string()],
            },
        )
        .expect("save should work");

        let is_favorite = is_favorite_in_path(&path, "script-a").expect("check should work");
        assert!(is_favorite);
    }

    #[test]
    fn test_is_favorite_returns_false_when_id_missing() {
        let temp = tempdir().expect("tempdir should be created");
        let path = test_favorites_path(temp.path());

        save_favorites_to_path(
            &path,
            &Favorites {
                script_ids: vec!["script-a".to_string()],
            },
        )
        .expect("save should work");

        let is_favorite = is_favorite_in_path(&path, "script-b").expect("check should work");
        assert!(!is_favorite);
    }
}

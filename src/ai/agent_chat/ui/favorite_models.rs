//! Agent Chat favorite model persistence.

use serde::{Deserialize, Serialize};

use super::config::AgentChatModelEntry;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
struct FavoriteModelsFile {
    favorite_model_ids: Vec<String>,
}

fn favorites_path() -> std::path::PathBuf {
    #[cfg(test)]
    if let Ok(path) = std::env::var("AGENT_CHAT_FAVORITE_MODELS_PATH") {
        return std::path::PathBuf::from(path);
    }
    crate::setup::get_kit_path().join("agent_chat-favorite-models.json")
}

pub(crate) fn load_favorite_model_ids() -> Vec<String> {
    std::fs::read_to_string(favorites_path())
        .ok()
        .and_then(|content| serde_json::from_str::<FavoriteModelsFile>(&content).ok())
        .map(|file| normalize_favorites(file.favorite_model_ids))
        .unwrap_or_default()
}

pub(crate) fn save_favorite_model_ids(ids: &[String]) -> std::io::Result<()> {
    let path = favorites_path();
    let file = FavoriteModelsFile {
        favorite_model_ids: normalize_favorites(ids.to_vec()),
    };
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, json)
}

pub(crate) fn toggle_favorite_model_id(model_id: &str) -> Vec<String> {
    let mut ids = load_favorite_model_ids();
    if let Some(index) = ids.iter().position(|id| id == model_id) {
        ids.remove(index);
    } else if !model_id.trim().is_empty() {
        ids.push(model_id.to_string());
    }
    let _ = save_favorite_model_ids(&ids);
    ids
}

pub(crate) fn is_favorite_model_id(model_id: &str) -> bool {
    load_favorite_model_ids().iter().any(|id| id == model_id)
}

pub(crate) fn next_favorite_model_id(
    current_model_id: Option<&str>,
    favorite_ids: &[String],
    available_models: &[AgentChatModelEntry],
) -> Option<String> {
    let available_favorites = favorite_ids
        .iter()
        .filter(|id| available_models.iter().any(|model| model.id == **id))
        .cloned()
        .collect::<Vec<_>>();

    if available_favorites.is_empty() {
        return None;
    }

    let next_index = current_model_id
        .and_then(|current| available_favorites.iter().position(|id| id == current))
        .map(|index| (index + 1) % available_favorites.len())
        .unwrap_or(0);
    available_favorites.get(next_index).cloned()
}

fn normalize_favorites(ids: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for id in ids {
        let id = id.trim();
        if !id.is_empty() && !out.iter().any(|existing| existing == id) {
            out.push(id.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn favorite_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn model(id: &str) -> AgentChatModelEntry {
        AgentChatModelEntry {
            id: id.to_string(),
            display_name: Some(id.to_string()),
            context_window: None,
        }
    }

    #[test]
    fn favorite_models_toggle_and_persist_round_trip() {
        let _guard = favorite_env_lock().lock().expect("favorite env lock");
        let previous_path = std::env::var("AGENT_CHAT_FAVORITE_MODELS_PATH").ok();
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("favorites.json");
        std::env::set_var("AGENT_CHAT_FAVORITE_MODELS_PATH", &path);

        assert!(load_favorite_model_ids().is_empty());
        assert_eq!(toggle_favorite_model_id("m1"), vec!["m1".to_string()]);
        assert_eq!(load_favorite_model_ids(), vec!["m1".to_string()]);
        assert!(toggle_favorite_model_id("m1").is_empty());
        assert!(load_favorite_model_ids().is_empty());

        match previous_path {
            Some(path) => std::env::set_var("AGENT_CHAT_FAVORITE_MODELS_PATH", path),
            None => std::env::remove_var("AGENT_CHAT_FAVORITE_MODELS_PATH"),
        }
    }

    #[test]
    fn favorite_models_cycle_wraps_and_skips_missing() {
        let favorites = vec!["missing".to_string(), "m1".to_string(), "m2".to_string()];
        let available = vec![model("m1"), model("m2")];

        assert_eq!(
            next_favorite_model_id(None, &favorites, &available).as_deref(),
            Some("m1")
        );
        assert_eq!(
            next_favorite_model_id(Some("m1"), &favorites, &available).as_deref(),
            Some("m2")
        );
        assert_eq!(
            next_favorite_model_id(Some("m2"), &favorites, &available).as_deref(),
            Some("m1")
        );
        assert_eq!(
            next_favorite_model_id(Some("missing"), &favorites, &available).as_deref(),
            Some("m1")
        );
        assert!(
            next_favorite_model_id(Some("m1"), &[String::from("missing")], &available).is_none()
        );
    }
}

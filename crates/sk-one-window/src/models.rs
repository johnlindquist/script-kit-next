//! Choosing the model is something you say ("use opus for this") or pick after ⌘K; there is no
//! picker.

use crate::row::Row;

/// The models the chat can use: a row per model (its id as the key, its name as the title).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Models {
    pub list: Vec<Row>,
    pub current: Option<String>,
}

const KEY_PREFIX: &str = "chat:model:";

pub fn row_key(id: &str) -> String {
    format!("{KEY_PREFIX}{id}")
}

pub fn id_of(key: &str) -> Option<&str> {
    key.strip_prefix(KEY_PREFIX)
}

/// The name in "use opus for this", "switch to gpt-5", "try sonnet instead".
fn named(typed: &str) -> Option<String> {
    let lower = typed.trim().to_lowercase();
    let rest = ["use ", "switch to ", "try "]
        .iter()
        .find_map(|p| lower.strip_prefix(p))?;
    let rest = [" for this chat", " for this", " instead", " here"]
        .iter()
        .find_map(|s| rest.strip_suffix(s))
        .unwrap_or(rest)
        .trim();
    (!rest.is_empty()).then(|| rest.to_string())
}

impl Models {
    /// The model the typed words name, when they say to use one.
    pub fn said(&self, typed: &str) -> Option<&Row> {
        let name = named(typed)?;
        let words: Vec<&str> = name.split_whitespace().collect();
        let fits = |m: &&Row| {
            let hay = format!("{} {}", m.title.to_lowercase(), m.key.to_lowercase());
            words.iter().all(|w| hay.contains(w))
        };
        self.others()
            .find(fits)
            .or_else(|| self.list.iter().find(fits))
    }

    pub fn others(&self) -> impl Iterator<Item = &Row> {
        self.list
            .iter()
            .filter(|m| self.current.as_deref() != Some(m.key.as_str()))
    }

    pub fn current_title(&self) -> Option<&str> {
        let current = self.current.as_deref()?;
        self.list
            .iter()
            .find(|m| m.key == current)
            .map(|m| m.title.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn models() -> Models {
        Models {
            list: vec![
                Row::new("anthropic/claude-sonnet-4", "Claude Sonnet 4"),
                Row::new("anthropic/claude-opus-4-1", "Claude Opus 4.1"),
                Row::new("openai/gpt-5", "GPT-5"),
            ],
            current: Some("anthropic/claude-sonnet-4".into()),
        }
    }

    #[test]
    fn saying_a_model_names_it() {
        let m = models();
        assert_eq!(
            m.said("use opus for this").map(|r| r.title.as_str()),
            Some("Claude Opus 4.1")
        );
        assert_eq!(
            m.said("switch to gpt-5").map(|r| r.key.as_str()),
            Some("openai/gpt-5")
        );
        assert!(m.said("use the force").is_none());
        assert!(
            m.said("what should I use for this").is_none(),
            "only a sentence that starts by saying to use one"
        );
        assert_eq!(m.current_title(), Some("Claude Sonnet 4"));
        assert_eq!(id_of(&row_key("openai/gpt-5")), Some("openai/gpt-5"));
    }
}

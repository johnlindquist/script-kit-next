//! The Help catalog: what each key, sigil and hidden place does. Help (`?`) lists it, and
//! ↵ on a tip tries it: a tip with a place opens it, a tip with a typed example puts the
//! example in the main input. The catalog ships inside the app and setup writes it to
//! `~/.scriptkit/tips.json`. This module performs no network IO.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tip {
    pub id: String,
    /// Key/sigil that invokes the trick (";", "Space", "⌘↵"). Rendered as a
    /// footer-style keycap chip; None for tips with no single trigger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint_key: Option<String>,
    /// Hint text WITHOUT the key (the keycap carries it).
    pub hint: String,
    pub title: String,
    pub description: String,
    pub examples: Vec<TipExample>,
    pub keywords: Vec<String>,
    /// Launcher command id (`builtin/…`) ↵ runs to open the place this tip is about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<String>,
}

impl Tip {
    /// Key + hint as one sentence, for surfaces without keycap chrome
    /// (list subtitles, search).
    pub fn full_hint(&self) -> String {
        match self.hint_key.as_deref() {
            Some(key) => format!("{key} {}", self.hint),
            None => self.hint.clone(),
        }
    }

    /// The trigger as it fits a 24pt row tile: word keys collapse to their
    /// glyph so every tile reads as one key.
    pub fn trigger_glyph(&self) -> Option<String> {
        self.hint_key.as_deref().map(|key| match key {
            "Space" => "␣".to_string(),
            other => other.to_string(),
        })
    }

    /// The hint as a standalone sentence, for surfaces where the keycap sits
    /// beside it instead of leading it.
    pub fn hint_sentence(&self) -> String {
        let mut chars = self.hint.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => String::new(),
        }
    }

    /// First example with typed text. Key-press-only examples have nothing to
    /// type, so they never become the tip's try.
    pub fn typed_example(&self) -> Option<&TipExample> {
        self.examples
            .iter()
            .find(|example| !example.input.trim().is_empty())
    }

    /// What ↵ does on this tip in Help: open its place, or type its example into the input.
    pub fn try_it(&self) -> Option<TipTry<'_>> {
        if let Some(id) = self.open.as_deref().filter(|id| !id.trim().is_empty()) {
            return Some(TipTry::Open(id));
        }
        self.typed_example()
            .map(|example| TipTry::Type(example.input.as_str()))
    }
}

/// What trying a tip does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipTry<'a> {
    /// Runs this launcher command id.
    Open(&'a str),
    /// Goes back to the main input with these words typed.
    Type(&'a str),
}

impl TipTry<'_> {
    /// The ↵ label in Help's footer.
    pub fn label(self) -> &'static str {
        match self {
            Self::Open(_) => "Open It",
            Self::Type(_) => "Try It",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TipExample {
    /// Text typed into the main input. Empty for key-press-only examples.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input: String,
    /// Keys pressed (after the input, when both are set), rendered as keycaps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<String>,
    /// What happens.
    pub note: String,
}

/// Byte ranges of the syntax tokens in an example input (`;todo`, `@clipboard:`,
/// `type:script`, `#health`), so the preview can color the part that does the
/// work and leave the words plain.
pub fn example_syntax_ranges(input: &str) -> Vec<std::ops::Range<usize>> {
    const SIGILS: &[char] = &[';', '/', '@', ':', '~', '!', '?', '`', '#', '+', '>'];
    let mut ranges = Vec::new();
    let mut offset = 0;
    for token in input.split(' ') {
        let is_syntax = token.starts_with(SIGILS)
            || token
                .split_once(':')
                .is_some_and(|(key, _)| !key.is_empty() && key.chars().all(char::is_alphanumeric));
        if !token.is_empty() && is_syntax {
            ranges.push(offset..offset + token.len());
        }
        offset += token.len() + 1;
    }
    ranges
}

/// Placeholders resolved at load time to the registered hotkeys' display glyphs (e.g. "⌘;"),
/// so tips about a hotkey always show the real binding.
const MAIN_HOTKEY_PLACEHOLDER: &str = "{mainHotkey}";
const NOTES_HOTKEY_PLACEHOLDER: &str = "{notesHotkey}";
const DICTATION_HOTKEY_PLACEHOLDER: &str = "{dictationHotkey}";

/// Each placeholder with the binding it stands for; `None` when that hotkey is turned off.
type Bindings<'a> = [(&'a str, Option<&'a str>)];

fn tip_texts_mut(tip: &mut Tip) -> Vec<&mut String> {
    let mut texts = vec![&mut tip.hint, &mut tip.title, &mut tip.description];
    texts.extend(tip.hint_key.as_mut());
    for example in &mut tip.examples {
        texts.push(&mut example.input);
        texts.push(&mut example.note);
        texts.extend(example.keys.as_mut());
    }
    texts
}

/// Fills in each hotkey placeholder; a tip about a hotkey that's turned off is left out.
fn resolve_tip_placeholders(mut tips: Vec<Tip>, bindings: &Bindings<'_>) -> Vec<Tip> {
    tips.retain_mut(|tip| {
        let mut texts = tip_texts_mut(tip);
        for (placeholder, binding) in bindings {
            if !texts.iter().any(|text| text.contains(placeholder)) {
                continue;
            }
            let Some(binding) = binding else {
                return false;
            };
            for text in texts.iter_mut() {
                **text = text.replace(placeholder, binding);
            }
        }
        true
    });
    tips
}

fn tips_json_path() -> PathBuf {
    crate::setup::get_kit_path().join("tips.json")
}
fn embedded_tips() -> Vec<Tip> {
    serde_json::from_str(crate::setup::EMBEDDED_TIPS).unwrap_or_else(|error| {
        tracing::warn!(%error, "embedded tips catalog is invalid");
        Vec::new()
    })
}

pub fn load_tips() -> Vec<Tip> {
    let path = tips_json_path();
    let tips = match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|error| {
            tracing::warn!(%error, path = %path.display(), "failed to parse tips catalog; using defaults");
            embedded_tips()
        }),
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "failed to read tips catalog; using defaults");
            embedded_tips()
        }
    };
    let config = crate::config::load_config();
    let main = config.hotkey.to_display_string();
    let notes = config
        .get_notes_hotkey()
        .map(|hotkey| hotkey.to_display_string());
    let dictation = config
        .get_dictation_hotkey()
        .map(|hotkey| hotkey.to_display_string());
    resolve_tip_placeholders(
        tips,
        &[
            (MAIN_HOTKEY_PLACEHOLDER, Some(main.as_str())),
            (NOTES_HOTKEY_PLACEHOLDER, notes.as_deref()),
            (DICTATION_HOTKEY_PLACEHOLDER, dictation.as_deref()),
        ],
    )
}

/// Single filter authority for the Tips browser. The renderer, the key
/// handler, the visible-count label, and the native footer button enablement
/// all consume this so the list rows, the "N tips" count, and the footer can
/// never disagree about what matches.
pub fn tip_matches_filter(tip: &Tip, query_lowercase: &str) -> bool {
    query_lowercase.is_empty()
        || tip.title.to_lowercase().contains(query_lowercase)
        || tip.full_hint().to_lowercase().contains(query_lowercase)
        || tip.description.to_lowercase().contains(query_lowercase)
        || tip
            .keywords
            .iter()
            .any(|keyword| keyword.to_lowercase().contains(query_lowercase))
}

/// Indices into `entries` visible under `filter`, in catalog order.
pub fn visible_tip_indices(entries: &[Tip], filter: &str) -> Vec<usize> {
    let query = filter.trim().to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, tip)| tip_matches_filter(tip, &query).then_some(index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_temp_kit(contents: Option<&str>, test: impl FnOnce()) {
        let _guard = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let previous = std::env::var_os(crate::setup::SK_PATH_ENV);
        std::env::set_var("SK_PATH", dir.path());
        if let Some(value) = contents {
            std::fs::write(dir.path().join("tips.json"), value).unwrap();
        }
        test();
        match previous {
            Some(value) => std::env::set_var(crate::setup::SK_PATH_ENV, value),
            None => std::env::remove_var(crate::setup::SK_PATH_ENV),
        }
    }
    #[test]
    fn missing_file_uses_embedded_defaults() {
        with_temp_kit(None, || assert!(!load_tips().is_empty()));
    }
    #[test]
    fn corrupt_file_uses_embedded_defaults() {
        with_temp_kit(Some("{"), || assert!(!load_tips().is_empty()));
    }
    #[test]
    fn visible_tip_indices_match_all_text_fields() {
        let tip = |id: &str, title: &str, hint: &str, description: &str, keyword: &str| Tip {
            id: id.into(),
            hint_key: None,
            hint: hint.into(),
            title: title.into(),
            description: description.into(),
            examples: Vec::new(),
            keywords: vec![keyword.into()],
            open: None,
        };
        let entries = vec![
            tip("a", "Capture a thought", "hint-a", "desc-a", "brain"),
            tip("b", "Jump to Today", "hint-b", "unique-description", "day"),
            tip("c", "Slash commands", "unique-hint", "desc-c", "slash"),
        ];
        assert_eq!(visible_tip_indices(&entries, ""), vec![0, 1, 2]);
        assert_eq!(visible_tip_indices(&entries, "  Capture "), vec![0]);
        assert_eq!(visible_tip_indices(&entries, "unique-description"), vec![1]);
        assert_eq!(visible_tip_indices(&entries, "UNIQUE-HINT"), vec![2]);
        assert_eq!(visible_tip_indices(&entries, "brain"), vec![0]);
        assert!(visible_tip_indices(&entries, "no-match").is_empty());
    }

    fn tip_about(text: &str) -> Tip {
        Tip {
            id: text.into(),
            hint_key: Some(text.into()),
            hint: "h".into(),
            title: "t".into(),
            description: format!("Press {text}."),
            examples: vec![TipExample {
                input: String::new(),
                keys: Some(format!("{text} {text}")),
                note: format!("Press {text}"),
            }],
            keywords: Vec::new(),
            open: None,
        }
    }

    #[test]
    fn placeholders_take_the_real_bindings_and_turned_off_hotkeys_drop_their_tips() {
        let tips = vec![
            tip_about(MAIN_HOTKEY_PLACEHOLDER),
            tip_about(DICTATION_HOTKEY_PLACEHOLDER),
            tip_about("⌘T"),
        ];
        let resolved = resolve_tip_placeholders(
            tips,
            &[
                (MAIN_HOTKEY_PLACEHOLDER, Some("⌘;")),
                (DICTATION_HOTKEY_PLACEHOLDER, None),
            ],
        );
        let ids: Vec<&str> = resolved.iter().map(|tip| tip.id.as_str()).collect();
        assert_eq!(ids, vec![MAIN_HOTKEY_PLACEHOLDER, "⌘T"]);
        let main = &resolved[0];
        assert_eq!(main.hint_key.as_deref(), Some("⌘;"));
        assert_eq!(main.description, "Press ⌘;.");
        assert_eq!(main.examples[0].keys.as_deref(), Some("⌘; ⌘;"));
        assert_eq!(main.examples[0].note, "Press ⌘;");
    }

    #[test]
    fn trying_a_tip_opens_its_place_first_then_types_its_first_typed_example() {
        let example = |input: &str, keys: Option<&str>| TipExample {
            input: input.into(),
            keys: keys.map(Into::into),
            note: "n".into(),
        };
        let mut tip = Tip {
            id: "x".into(),
            hint_key: Some("Space".into()),
            hint: "uses what's focused".into(),
            title: "t".into(),
            description: "d".into(),
            examples: vec![example("", Some("⌘K")), example(";todo call mom", None)],
            keywords: Vec::new(),
            open: None,
        };
        assert_eq!(tip.try_it(), Some(TipTry::Type(";todo call mom")));
        assert_eq!(tip.trigger_glyph().as_deref(), Some("␣"));
        assert_eq!(tip.hint_sentence(), "Uses what's focused");
        tip.open = Some("builtin/day-page".into());
        assert_eq!(tip.try_it(), Some(TipTry::Open("builtin/day-page")));
        tip.open = None;
        tip.examples.truncate(1);
        assert_eq!(tip.try_it(), None);
    }

    #[test]
    fn every_place_the_catalog_opens_is_a_real_launcher_command() {
        let config = crate::config::BuiltInConfig::default();
        let entries = crate::builtins::get_builtin_entries(&config);
        for tip in embedded_tips() {
            if let Some(id) = tip.open.as_deref() {
                assert!(
                    entries.iter().any(|entry| entry.id == id),
                    "{} opens {id}, which isn't a launcher command",
                    tip.id
                );
            }
        }
    }

    #[test]
    fn example_syntax_ranges_color_only_the_syntax() {
        let input = ";todo call the dentist due:friday #health";
        let tokens: Vec<&str> = example_syntax_ranges(input)
            .into_iter()
            .map(|range| &input[range])
            .collect();
        assert_eq!(tokens, vec![";todo", "due:friday", "#health"]);
        assert!(example_syntax_ranges("summarize this page").is_empty());
        let input = "type:script deploy";
        assert_eq!(example_syntax_ranges(input), vec![0..11]);
    }

    #[test]
    fn parses_local_catalog() {
        let json = r#"[{"id":"x","hint":"h","title":"t","description":"d","examples":[{"input":"i","note":"n"}],"keywords":["k"]}]"#;
        with_temp_kit(Some(json), || assert_eq!(load_tips()[0].id, "x"));
    }
}

//! Offline authoring for two existing bounded sources. The user's title names a declaration;
//! creating it does not read the source, run a program, or grant a pane action.
use crate::views::{
    self, ListSpec, NoteAction, NoteDetail, NoteFilter, NoteSort, PaneAction, PaneDetail,
    PaneFilter, PaneSort, ViewRecipe,
};
use sha2::{Digest, Sha256};

pub(super) fn requested(input: &str) -> Option<ViewRecipe> {
    let raw = input.trim().trim_end_matches(['.', '!', '?']);
    let words: Vec<_> = raw.split_whitespace().collect();
    let named = words
        .iter()
        .position(|x| x.eq_ignore_ascii_case("called") || x.eq_ignore_ascii_case("named"))?;
    let intent = words[..named].join(" ").to_ascii_lowercase();
    let intent = intent.strip_prefix("kit, ").unwrap_or(&intent);
    let intent = intent.strip_prefix("please ").unwrap_or(intent);
    let source = ["make a list of ", "create a list of ", "give me a list of "]
        .into_iter()
        .find_map(|prefix| intent.strip_prefix(prefix))?;
    let source = source.strip_prefix("my ").unwrap_or(source);
    let (source, alphabetical) = if let Some(s) = source
        .strip_suffix(" sorted by title")
        .or_else(|| source.strip_suffix(" alphabetically"))
    {
        (s.trim_end_matches(','), Some(true))
    } else if let Some(s) = source.strip_suffix(" newest first") {
        (s.trim_end_matches(','), Some(false))
    } else {
        (source, None)
    };
    let (limit, list) = match source {
        "pinned notes" | "notes" => (
            24,
            ListSpec::Notes {
                filter: if source == "pinned notes" {
                    NoteFilter::Pinned
                } else {
                    NoteFilter::All
                },
                sort: if alphabetical.unwrap_or(source == "pinned notes") {
                    NoteSort::Title
                } else {
                    NoteSort::LastChanged
                },
                detail: NoteDetail::LastChanged,
                enter: NoteAction::OpenNote,
            },
        ),
        "herdr panes" | "herdr panes that need me" if alphabetical != Some(false) => (
            24,
            ListSpec::Panes {
                filter: if source == "herdr panes" {
                    PaneFilter::All
                } else {
                    PaneFilter::NeedsAttention
                },
                sort: if alphabetical == Some(true) {
                    PaneSort::Title
                } else {
                    PaneSort::AttentionFirst
                },
                detail: PaneDetail::Status,
                enter: PaneAction::None,
            },
        ),
        _ => return None,
    };
    let name = words[named + 1..].join(" ");
    let quoted = name
        .strip_prefix('“')
        .and_then(|s| s.strip_suffix('”'))
        .or_else(|| name.strip_prefix('"').and_then(|s| s.strip_suffix('"')));
    let title = quoted.unwrap_or(&name);
    // A quoted name may contain punctuation; do not consume a following instruction as a name.
    if title.contains(['“', '”', '"'])
        || (quoted.is_none()
            && (!title
                .chars()
                .all(|c| c.is_alphanumeric() || c == ' ' || c == '-')
                || title
                    .split_whitespace()
                    .any(|w| w.eq_ignore_ascii_case("and") || w.eq_ignore_ascii_case("then"))))
    {
        return None;
    }
    let mut id = title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_ascii_lowercase();
    if !views::valid_id(&id) {
        id = format!("list-{:x}", Sha256::digest(title.as_bytes()))[..29].to_owned();
    }
    let view = ViewRecipe {
        id,
        title: title.to_owned(),
        limit,
        list,
    };
    view.validate().ok()?;
    Some(view)
}

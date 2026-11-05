//! Small offline grammar for deliberate launcher requests; arbitrary or compound prose is not
//! approval to change a setting. External authors may also use the typed catalog directly.
mod lists;
use super::Change;

/// Picking an example only fills the input. The person can edit it before selecting Apply or Save.
pub fn examples(input: &str) -> &'static [(&'static str, &'static str, &'static str)] {
    if !["customize kit", "customise kit", "personalize kit"]
        .iter()
        .any(|s| input.trim().eq_ignore_ascii_case(s))
    {
        return &[];
    }
    &[
        (
            "list",
            "Make a personal list",
            "Make a list of my pinned notes called Reading shelf",
        ),
        (
            "home",
            "Change my launcher home",
            "Keep my launcher minimal",
        ),
        (
            "selection",
            "Change my first selection action",
            "Put today’s note first for selected text",
        ),
    ]
}
use sk_one_window::personal::{ClipboardAction, Edit, HomeMode, SelectionAction, TextRole, Weight};
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reading {
    Change(Change),
    Why(Edit),
    UndoLast,
}
fn target(s: &str) -> bool {
    matches!(
        s.trim_start_matches("the "),
        "stays open"
            | "stays open label"
            | "stays open text"
            | "pinned label"
            | "pinned header"
            | "pinned window label"
            | "pin label"
    )
}
fn selected(s: &str) -> bool {
    matches!(
        s,
        "selected text" | "my selection" | "the selection" | "highlighted text"
    )
}
fn copied(s: &str) -> bool {
    matches!(
        s,
        "copied links" | "copied text" | "clipboard items" | "the clipboard"
    )
}
fn first(action: &str, subject: &str) -> Option<Edit> {
    if selected(subject) {
        return Some(Edit::SelectionFirst(Some(match action {
            "add to note" | "add to today's note" | "save to today" | "today's note" => {
                SelectionAction::Note
            }
            "ask" | "ask ai" => SelectionAction::Ask,
            "read" | "read selection" => SelectionAction::Show,
            "share" => SelectionAction::Handoff,
            _ => return None,
        })));
    }
    if copied(subject) {
        return Some(Edit::ClipboardFirst(Some(match action {
            "open" | "open link" => ClipboardAction::Open,
            "add to note" | "today's note" => ClipboardAction::Note,
            "ask" | "ask ai" => ClipboardAction::Ask,
            "read" => ClipboardAction::Show,
            _ => return None,
        })));
    }
    None
}
pub fn read(input: &str) -> Option<Reading> {
    if input.len() > 280 {
        return None;
    } // Never interpret an arbitrary document as instructions.
    if let Some(view) = lists::requested(input) {
        return Some(Reading::Change(Change::PutView { view }));
    }
    let normalized = input
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .to_ascii_lowercase()
        .replace(['’', '‘'], "'")
        .replace(['“', '”', '"'], "")
        .replace("'stays open'", "stays open");
    let norm = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = norm.strip_prefix("kit, ").unwrap_or(&norm);
    let s = s.strip_prefix("please ").unwrap_or(s);
    let mode = match s {
        "make my launcher minimal"
        | "keep my launcher minimal"
        | "use a minimal launcher"
        | "show only scripts when i open kit"
        | "show just scripts on my launcher home" => Some(HomeMode::Minimal),
        "restore the standard launcher"
        | "use the standard launcher"
        | "show everything on my launcher home" => Some(HomeMode::Standard),
        "why is my launcher minimal" | "why does my launcher only show scripts" => {
            return Some(Reading::Why(Edit::Home(HomeMode::Standard)));
        }
        _ => None,
    };
    if let Some(mode) = mode {
        return Some(Reading::Change(Change::Edit {
            edit: Edit::Home(mode),
        }));
    }
    if s == "undo my last launcher change" {
        return Some(Reading::UndoLast);
    }
    if s == "disable my launcher customizations" {
        return Some(Reading::Change(Change::Disable));
    }
    if s == "enable my launcher customizations" {
        return Some(Reading::Change(Change::Enable));
    }
    if let Some(words) = s.strip_prefix("why is ") {
        for (end, weight) in [(" brighter", false), (" bright", false), (" bold", true)] {
            if words.strip_suffix(end).is_some_and(target) {
                return Some(Reading::Why(if weight {
                    Edit::PinWeight(Weight::Regular)
                } else {
                    Edit::PinText(TextRole::Muted)
                }));
            }
        }
        return None;
    }
    let edit = if let Some(words) = s.strip_prefix("make ").or_else(|| s.strip_prefix("set ")) {
        let words = words.strip_suffix(" when kit is pinned").unwrap_or(words);
        [
            (
                " bold only while dictating",
                Edit::PinWeight(Weight::SemiboldWhileDictating),
            ),
            (
                " bold while dictating",
                Edit::PinWeight(Weight::SemiboldWhileDictating),
            ),
            (" bold", Edit::PinWeight(Weight::Semibold)),
            (" regular weight", Edit::PinWeight(Weight::Regular)),
            (" easier to read", Edit::PinText(TextRole::Primary)),
            (" brighter", Edit::PinText(TextRole::Primary)),
            (" clearer", Edit::PinText(TextRole::Primary)),
            (" muted", Edit::PinText(TextRole::Muted)),
        ]
        .into_iter()
        .find_map(|(end, v)| words.strip_suffix(end).filter(|t| target(t)).map(|_| v))?
    } else if let Some(words) = s.strip_prefix("put ") {
        let (action, subject) = words.split_once(" first for ")?;
        first(action, subject)?
    } else if let Some(words) = s.strip_prefix("for ") {
        let (subject, action) = words.split_once(", ")?;
        let action = action.strip_prefix("put ")?.strip_suffix(" first")?;
        first(action, subject)?
    } else {
        return None;
    };
    Some(Reading::Change(Change::Edit { edit }))
}
pub fn label(change: &Change) -> &'static str {
    match change {
        Change::Edit {
            edit: Edit::PinText(TextRole::Primary),
        } => "Use brighter text for Stays open",
        Change::Edit {
            edit: Edit::PinText(TextRole::Muted),
        } => "Use muted text for Stays open",
        Change::Edit {
            edit: Edit::PinWeight(Weight::SemiboldWhileDictating),
        } => "Make Stays open bold only while dictating",
        Change::Edit {
            edit: Edit::PinWeight(Weight::Semibold),
        } => "Make Stays open bold",
        Change::Edit {
            edit: Edit::PinWeight(Weight::Regular),
        } => "Use regular weight for Stays open",
        Change::Edit {
            edit: Edit::SelectionFirst(Some(SelectionAction::Note)),
        } => "Put today’s note first for selected text",
        Change::Edit {
            edit: Edit::SelectionFirst(Some(SelectionAction::Ask)),
        } => "Put Ask first for selected text",
        Change::Edit {
            edit: Edit::SelectionFirst(Some(SelectionAction::Show)),
        } => "Put Read first for selected text",
        Change::Edit {
            edit: Edit::SelectionFirst(Some(SelectionAction::Handoff)),
        } => "Put Share first for selected text",
        Change::Edit {
            edit: Edit::SelectionFirst(None),
        } => "Restore default selection actions",
        Change::Edit {
            edit: Edit::ClipboardFirst(Some(ClipboardAction::Open)),
        } => "Put Open first for copied links",
        Change::Edit {
            edit: Edit::ClipboardFirst(Some(ClipboardAction::Note)),
        } => "Put today’s note first for copied items",
        Change::Edit {
            edit: Edit::ClipboardFirst(Some(ClipboardAction::Ask)),
        } => "Put Ask first for copied items",
        Change::Edit {
            edit: Edit::ClipboardFirst(Some(ClipboardAction::Show)),
        } => "Put Read first for copied text",
        Change::Edit {
            edit: Edit::ClipboardFirst(None),
        } => "Restore default clipboard actions",
        Change::Edit {
            edit: Edit::Home(HomeMode::Minimal),
        } => "Show just scripts on the launcher home",
        Change::Edit {
            edit: Edit::Home(HomeMode::Standard),
        } => "Restore the standard launcher home",
        Change::PutView { .. } => "Save this personal view",
        Change::RemoveView { .. } => "Remove this personal view",
        Change::Undo { .. } => "Undo the last launcher change",
        Change::Disable => "Use the stock launcher",
        Change::Enable => "Use my launcher customizations",
    }
}

//! Personal rules set the home list, the existing pin’s appearance, and subject action order.
//! They never create actions, change target identities, or grant permission.
use crate::{Row, SubjectKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextRole {
    #[default]
    Muted,
    Primary,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Weight {
    #[default]
    Regular,
    Semibold,
    SemiboldWhileDictating,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectionAction {
    Ask,
    Note,
    Show,
    Handoff,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipboardAction {
    Ask,
    Note,
    Show,
    Open,
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HomeMode {
    #[default]
    Standard,
    Minimal,
}
impl HomeMode {
    fn is_standard(&self) -> bool {
        *self == Self::Standard
    }

    /// Only change the idle home list. Typed search, staged actions, questions and recovery
    /// still go through the ordinary reducer. Keep the source-owned key of each real script.
    pub(crate) fn suggestions(self, rows: &[Row]) -> Vec<Row> {
        let mut first_script = true;
        rows.iter()
            .enumerate()
            .filter_map(|(i, row)| {
                if self == Self::Minimal && !crate::setup::is_setup(&row.key) {
                    if !row.key.starts_with("script/") && !row.key.starts_with("scriptlet/") {
                        return None;
                    }
                    let mut row = row.clone();
                    row.head = std::mem::take(&mut first_script).then(|| "Scripts".to_owned());
                    Some(row)
                } else {
                    let mut row = row.clone();
                    if i == 0 && row.head.is_none() {
                        row.head = Some(crate::contract::words::SUGGESTED.into());
                    }
                    Some(row)
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct LauncherRules {
    pub pinned_text: TextRole,
    pub pinned_weight: Weight,
    pub selection_first: Option<SelectionAction>,
    pub clipboard_first: Option<ClipboardAction>,
    // Stock must keep its original bytes in sealed journals written by earlier versions.
    #[serde(skip_serializing_if = "HomeMode::is_standard")]
    pub home_mode: HomeMode,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "target", content = "value", deny_unknown_fields)]
pub enum Edit {
    #[serde(rename = "launcher.header.pinned-label.textRole")]
    PinText(TextRole),
    #[serde(rename = "launcher.header.pinned-label.weight")]
    PinWeight(Weight),
    #[serde(rename = "launcher.subject.selection.firstAction")]
    SelectionFirst(Option<SelectionAction>),
    #[serde(rename = "launcher.subject.clipboard.firstAction")]
    ClipboardFirst(Option<ClipboardAction>),
    #[serde(rename = "launcher.home.mode")]
    Home(HomeMode),
}
impl Edit {
    pub fn target(self) -> &'static str {
        match self {
            Self::Home(_) => "launcher.home.mode",
            Self::PinText(_) => "launcher.header.pinned-label.textRole",
            Self::PinWeight(_) => "launcher.header.pinned-label.weight",
            Self::SelectionFirst(_) => "launcher.subject.selection.firstAction",
            Self::ClipboardFirst(_) => "launcher.subject.clipboard.firstAction",
        }
    }
}
impl LauncherRules {
    pub fn with(mut self, edit: Edit) -> Self {
        match edit {
            Edit::Home(v) => self.home_mode = v,
            Edit::PinText(v) => self.pinned_text = v,
            Edit::PinWeight(v) => self.pinned_weight = v,
            Edit::SelectionFirst(v) => self.selection_first = v,
            Edit::ClipboardFirst(v) => self.clipboard_first = v,
        }
        self
    }
    pub fn edit_at(self, target: Edit) -> Edit {
        match target {
            Edit::Home(_) => Edit::Home(self.home_mode),
            Edit::PinText(_) => Edit::PinText(self.pinned_text),
            Edit::PinWeight(_) => Edit::PinWeight(self.pinned_weight),
            Edit::SelectionFirst(_) => Edit::SelectionFirst(self.selection_first),
            Edit::ClipboardFirst(_) => Edit::ClipboardFirst(self.clipboard_first),
        }
    }
    pub fn pin(self, dictating: bool) -> PinStyle {
        PinStyle {
            text_role: self.pinned_text,
            semibold: self.pinned_weight == Weight::Semibold
                || (dictating && self.pinned_weight == Weight::SemiboldWhileDictating),
        }
    }
    pub fn first(self, kind: SubjectKind) -> Option<&'static str> {
        match kind {
            SubjectKind::Selection => Some(match self.selection_first? {
                SelectionAction::Ask => "ask",
                SelectionAction::Note => "note",
                SelectionAction::Show => "show",
                SelectionAction::Handoff => "handoff",
            }),
            SubjectKind::Clipboard => Some(match self.clipboard_first? {
                ClipboardAction::Ask => "ask",
                ClipboardAction::Note => "note",
                ClipboardAction::Show => "show",
                ClipboardAction::Open => "open",
            }),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PinStyle {
    pub text_role: TextRole,
    pub semibold: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_weight_refinement_preserves_the_pinned_color_and_cannot_change_other_subjects() {
        let edited = LauncherRules::default()
            .with(Edit::PinText(TextRole::Primary))
            .with(Edit::PinWeight(Weight::SemiboldWhileDictating))
            .with(Edit::SelectionFirst(Some(SelectionAction::Note)));
        assert_eq!(
            edited.pin(false),
            PinStyle {
                text_role: TextRole::Primary,
                semibold: false
            }
        );
        assert!(edited.pin(true).semibold);
        assert_eq!(edited.first(SubjectKind::Clipboard), None);
        assert_eq!(edited.first(SubjectKind::Selection), Some("note"));
        for bad in [
            r#"{"target":"launcher.subject.selection.firstAction","value":"open"}"#,
            r#"{"target":"launcher.header.pinned-label.textRole","value":"primary","shell":"open"}"#,
            r#"{"target":"window.alpha","value":0}"#,
        ] {
            assert!(serde_json::from_str::<Edit>(bad).is_err());
        }
    }
}

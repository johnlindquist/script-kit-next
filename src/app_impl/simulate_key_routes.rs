//! What a protocol `simulateKey` means on the surfaces whose keys it drives
//! directly: who takes Enter on the main list before the highlighted row runs,
//! and what each key does on arg, mini and template prompts.

/// Modifiers held with a simulated key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SimulatedModifiers {
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

/// Something open on the main list that takes a plain Enter before the
/// highlighted row runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptListEnterOwner {
    ObjectSelector,
    TriggerPicker,
    Spine,
    FilterAcceptHint,
}

/// The order a plain Enter on the main list is offered; the highlighted row
/// runs only when none of these take it.
pub const SCRIPT_LIST_ENTER_OWNERS: [ScriptListEnterOwner; 4] = [
    ScriptListEnterOwner::ObjectSelector,
    ScriptListEnterOwner::TriggerPicker,
    ScriptListEnterOwner::Spine,
    ScriptListEnterOwner::FilterAcceptHint,
];

impl ScriptListEnterOwner {
    pub fn receipt(self) -> &'static str {
        match self {
            Self::ObjectSelector => "SimulateKey: Enter - accept menu-syntax object selector",
            Self::TriggerPicker => "SimulateKey: Enter - accept menu-syntax picker",
            Self::Spine => "SimulateKey: Enter - spine consumed",
            Self::FilterAcceptHint => {
                "SimulateKey: Enter - accepted menu-syntax filter hint consumed"
            }
        }
    }
}

/// A key on an arg or mini prompt while its actions menu is closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgPromptKey {
    Up,
    Down,
    SubmitSelection,
    Cancel,
    Unhandled,
}

impl ArgPromptKey {
    /// Whether the key ends the prompt, so a sessionless mini prompt with no
    /// script to answer it goes back to the main list itself.
    pub fn finishes_prompt(self) -> bool {
        matches!(self, Self::SubmitSelection | Self::Cancel)
    }
}

pub fn arg_prompt_key(key_lower: &str) -> ArgPromptKey {
    match key_lower {
        "up" | "arrowup" => ArgPromptKey::Up,
        "down" | "arrowdown" => ArgPromptKey::Down,
        "enter" => ArgPromptKey::SubmitSelection,
        "escape" => ArgPromptKey::Cancel,
        _ => ArgPromptKey::Unhandled,
    }
}

/// A key on a template prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplatePromptKey {
    ToggleActions,
    Submit,
    Cancel,
    NextField,
    PreviousField,
    Backspace,
    Type,
    Unhandled,
}

pub fn template_prompt_key(key_lower: &str, modifiers: SimulatedModifiers) -> TemplatePromptKey {
    let SimulatedModifiers {
        cmd,
        shift,
        alt,
        ctrl,
    } = modifiers;
    if cmd && !shift && !alt && !ctrl && key_lower == "k" {
        return TemplatePromptKey::ToggleActions;
    }
    match key_lower {
        "enter" | "return" if !shift && !cmd => TemplatePromptKey::Submit,
        "escape" | "esc" if !cmd => TemplatePromptKey::Cancel,
        "tab" if !cmd && !shift => TemplatePromptKey::NextField,
        "tab" if !cmd && shift => TemplatePromptKey::PreviousField,
        "backspace" if !cmd && !alt && !ctrl => TemplatePromptKey::Backspace,
        _ if !cmd && !shift && !alt && !ctrl && key_lower.chars().count() == 1 => {
            TemplatePromptKey::Type
        }
        _ => TemplatePromptKey::Unhandled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: SimulatedModifiers = SimulatedModifiers {
        cmd: false,
        shift: false,
        alt: false,
        ctrl: false,
    };
    const SHIFT: SimulatedModifiers = SimulatedModifiers {
        shift: true,
        ..NONE
    };
    const CMD: SimulatedModifiers = SimulatedModifiers { cmd: true, ..NONE };

    #[test]
    fn main_list_enter_goes_to_open_pickers_before_the_highlighted_row() {
        assert_eq!(
            SCRIPT_LIST_ENTER_OWNERS,
            [
                ScriptListEnterOwner::ObjectSelector,
                ScriptListEnterOwner::TriggerPicker,
                ScriptListEnterOwner::Spine,
                ScriptListEnterOwner::FilterAcceptHint,
            ],
        );
        let mut receipts: Vec<_> = SCRIPT_LIST_ENTER_OWNERS
            .iter()
            .map(|owner| owner.receipt())
            .collect();
        receipts.sort_unstable();
        receipts.dedup();
        assert_eq!(receipts.len(), SCRIPT_LIST_ENTER_OWNERS.len());
        assert!(!receipts.contains(&"SimulateKey: Enter - execute selected"));
    }

    #[test]
    fn enter_on_an_arg_or_mini_prompt_submits_its_selection() {
        assert_eq!(arg_prompt_key("enter"), ArgPromptKey::SubmitSelection);
        assert_eq!(arg_prompt_key("escape"), ArgPromptKey::Cancel);
        assert_eq!(arg_prompt_key("up"), ArgPromptKey::Up);
        assert_eq!(arg_prompt_key("arrowup"), ArgPromptKey::Up);
        assert_eq!(arg_prompt_key("down"), ArgPromptKey::Down);
        assert_eq!(arg_prompt_key("arrowdown"), ArgPromptKey::Down);
        assert_eq!(arg_prompt_key("tab"), ArgPromptKey::Unhandled);
        assert_eq!(arg_prompt_key("a"), ArgPromptKey::Unhandled);
    }

    #[test]
    fn only_enter_and_escape_finish_an_arg_or_mini_prompt() {
        for key in ["enter", "escape"] {
            assert!(arg_prompt_key(key).finishes_prompt(), "{key}");
        }
        for key in ["up", "down", "tab", "a", "backspace"] {
            assert!(!arg_prompt_key(key).finishes_prompt(), "{key}");
        }
    }

    #[test]
    fn template_prompt_keys_edit_move_between_fields_and_finish() {
        assert_eq!(
            template_prompt_key("enter", NONE),
            TemplatePromptKey::Submit
        );
        assert_eq!(
            template_prompt_key("return", NONE),
            TemplatePromptKey::Submit
        );
        assert_eq!(
            template_prompt_key("escape", NONE),
            TemplatePromptKey::Cancel
        );
        assert_eq!(template_prompt_key("esc", SHIFT), TemplatePromptKey::Cancel);
        assert_eq!(
            template_prompt_key("tab", NONE),
            TemplatePromptKey::NextField
        );
        assert_eq!(
            template_prompt_key("tab", SHIFT),
            TemplatePromptKey::PreviousField
        );
        assert_eq!(
            template_prompt_key("backspace", NONE),
            TemplatePromptKey::Backspace
        );
        assert_eq!(template_prompt_key("x", NONE), TemplatePromptKey::Type);
        assert_eq!(
            template_prompt_key("k", CMD),
            TemplatePromptKey::ToggleActions
        );
    }

    #[test]
    fn modified_keys_do_not_type_or_submit_into_a_template_prompt() {
        assert_eq!(
            template_prompt_key("enter", SHIFT),
            TemplatePromptKey::Unhandled
        );
        assert_eq!(
            template_prompt_key("enter", CMD),
            TemplatePromptKey::Unhandled
        );
        assert_eq!(
            template_prompt_key("x", SHIFT),
            TemplatePromptKey::Unhandled
        );
        assert_eq!(template_prompt_key("x", CMD), TemplatePromptKey::Unhandled);
        assert_eq!(
            template_prompt_key("tab", CMD),
            TemplatePromptKey::Unhandled
        );
        assert_eq!(
            template_prompt_key(
                "k",
                SimulatedModifiers {
                    cmd: true,
                    shift: true,
                    ..NONE
                }
            ),
            TemplatePromptKey::Unhandled
        );
        assert_eq!(
            template_prompt_key("backspace", CMD),
            TemplatePromptKey::Unhandled
        );
        assert_eq!(
            template_prompt_key("left", NONE),
            TemplatePromptKey::Unhandled
        );
    }
}

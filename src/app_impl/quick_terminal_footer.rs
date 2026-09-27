//! Which buttons the Quick Terminal's native footer shows. ⌘↩ applies the terminal back to
//! where it came from when it can; otherwise it hands the terminal to the Agent when it can;
//! ⌘W always closes. The binary's footer calls it; the lib carries this file through `#[path]`
//! so its tests run under `cargo test --lib`.

use crate::footer_popup::{FooterAction, FooterButtonConfig};

pub(crate) fn quick_terminal_footer_buttons(
    can_apply: bool,
    can_attach_to_agent: bool,
    enabled: bool,
) -> Vec<FooterButtonConfig> {
    let mut buttons = Vec::with_capacity(if can_apply || can_attach_to_agent {
        2
    } else {
        1
    });
    if can_apply {
        buttons.push(FooterButtonConfig::new(FooterAction::Apply, "⌘↩", "Apply").enabled(enabled));
    } else if can_attach_to_agent {
        buttons.push(FooterButtonConfig::new(FooterAction::Ai, "⌘↩", "Agent").enabled(enabled));
    }
    buttons.push(FooterButtonConfig::new(FooterAction::Close, "⌘W", "Close").enabled(enabled));
    buttons
}

#[cfg(test)]
mod tests {
    use super::quick_terminal_footer_buttons;
    use crate::footer_popup::FooterAction;

    fn shown(
        buttons: &[crate::footer_popup::FooterButtonConfig],
    ) -> Vec<(FooterAction, &str, &str)> {
        buttons
            .iter()
            .map(|button| (button.action, button.key.as_ref(), button.label.as_ref()))
            .collect()
    }

    #[test]
    fn cmd_enter_applies_back_when_the_terminal_can_apply() {
        for can_attach_to_agent in [false, true] {
            assert_eq!(
                shown(&quick_terminal_footer_buttons(
                    true,
                    can_attach_to_agent,
                    true
                )),
                [
                    (FooterAction::Apply, "⌘↩", "Apply"),
                    (FooterAction::Close, "⌘W", "Close"),
                ]
            );
        }
    }

    #[test]
    fn cmd_enter_hands_the_terminal_to_the_agent_when_apply_is_unavailable() {
        assert_eq!(
            shown(&quick_terminal_footer_buttons(false, true, true)),
            [
                (FooterAction::Ai, "⌘↩", "Agent"),
                (FooterAction::Close, "⌘W", "Close"),
            ]
        );
    }

    #[test]
    fn only_close_is_offered_when_neither_apply_nor_agent_can_take_it() {
        assert_eq!(
            shown(&quick_terminal_footer_buttons(false, false, true)),
            [(FooterAction::Close, "⌘W", "Close")]
        );
    }

    #[test]
    fn a_blocked_footer_disables_every_button() {
        let buttons = quick_terminal_footer_buttons(false, true, false);
        assert_eq!(buttons.len(), 2);
        assert!(buttons.iter().all(|button| !button.enabled));
    }
}

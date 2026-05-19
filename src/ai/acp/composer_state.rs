//! State transitions for the ACP composer picker lifecycle.

use crate::ai::window::context_picker::types::ContextPickerTrigger;

use super::types::{AcpDismissedMentionTrigger, AcpMentionSession};

const MENTION_PICKER_MAX_VISIBLE: usize = 8;

#[derive(Debug, Clone)]
pub(crate) enum AcpComposerPickerState {
    Closed,
    Open(AcpMentionSession),
    Dismissed(AcpDismissedMentionTrigger),
}

#[derive(Debug, Clone)]
pub(crate) struct AcpComposerPickerRefreshInput {
    pub(crate) active_trigger: Option<AcpDismissedMentionTrigger>,
    pub(crate) next_session: Option<AcpMentionSession>,
    pub(crate) focused_inline_preview: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpComposerPickerDismissReason {
    Outside,
    Escape,
    SubmitStarted,
    HostHide,
    PortalStaged,
    SlashToggleClosed,
}

#[derive(Debug, Clone)]
pub(crate) enum AcpComposerPickerEvent {
    Refresh(AcpComposerPickerRefreshInput),
    NavigatePrevious,
    NavigateNext,
    Dismiss {
        reason: AcpComposerPickerDismissReason,
        cursor: usize,
    },
    Accept,
    AcceptIgnoredKeepOpen(AcpMentionSession),
    SubmitStarted,
    SlashToggle,
}

#[derive(Debug, Clone)]
pub(crate) struct AcpComposerPickerTransition {
    pub(crate) state: AcpComposerPickerState,
    pub(crate) sync_popup: bool,
    pub(crate) notify: bool,
    pub(crate) close_competing_popups: bool,
    pub(crate) clear_last_accepted_item: bool,
    pub(crate) log_visible_reason: Option<&'static str>,
    pub(crate) accepted_session: Option<AcpMentionSession>,
    pub(crate) insert_slash_input: bool,
    pub(crate) clear_slash_input: bool,
}

impl AcpComposerPickerTransition {
    fn new(state: AcpComposerPickerState) -> Self {
        Self {
            state,
            sync_popup: false,
            notify: false,
            close_competing_popups: false,
            clear_last_accepted_item: false,
            log_visible_reason: None,
            accepted_session: None,
            insert_slash_input: false,
            clear_slash_input: false,
        }
    }

    fn redraw(mut self) -> Self {
        self.sync_popup = true;
        self.notify = true;
        self
    }

    fn opened(mut self) -> Self {
        self.close_competing_popups = true;
        self.clear_last_accepted_item = true;
        self.log_visible_reason = Some("refresh");
        self.redraw()
    }
}

pub(crate) fn reduce_acp_composer_picker(
    state: AcpComposerPickerState,
    event: AcpComposerPickerEvent,
) -> AcpComposerPickerTransition {
    match event {
        AcpComposerPickerEvent::Refresh(input) => refresh_transition(input),
        AcpComposerPickerEvent::NavigatePrevious => navigate_transition(state, -1, "keyboard_prev"),
        AcpComposerPickerEvent::NavigateNext => navigate_transition(state, 1, "keyboard_next"),
        AcpComposerPickerEvent::Dismiss { reason, cursor } => {
            dismiss_transition(state, reason, cursor)
        }
        AcpComposerPickerEvent::Accept => accept_transition(state),
        AcpComposerPickerEvent::AcceptIgnoredKeepOpen(session) => {
            AcpComposerPickerTransition::new(AcpComposerPickerState::Open(session))
        }
        AcpComposerPickerEvent::SubmitStarted => {
            dismiss_transition(state, AcpComposerPickerDismissReason::SubmitStarted, 0)
        }
        AcpComposerPickerEvent::SlashToggle => slash_toggle_transition(state),
    }
}

fn refresh_transition(input: AcpComposerPickerRefreshInput) -> AcpComposerPickerTransition {
    if input.focused_inline_preview {
        return AcpComposerPickerTransition::new(AcpComposerPickerState::Closed).redraw();
    }

    if let (Some(active), None) = (&input.active_trigger, &input.next_session) {
        return AcpComposerPickerTransition::new(AcpComposerPickerState::Dismissed(active.clone()))
            .redraw();
    }

    if let Some(session) = input.next_session {
        return AcpComposerPickerTransition::new(AcpComposerPickerState::Open(session)).opened();
    }

    AcpComposerPickerTransition::new(AcpComposerPickerState::Closed).redraw()
}

fn navigate_transition(
    state: AcpComposerPickerState,
    delta: isize,
    reason: &'static str,
) -> AcpComposerPickerTransition {
    let AcpComposerPickerState::Open(mut session) = state else {
        return AcpComposerPickerTransition::new(state);
    };

    if !session.items.is_empty() {
        let len = session.items.len();
        session.selected_index = if delta < 0 {
            if session.selected_index == 0 {
                len - 1
            } else {
                session.selected_index - 1
            }
        } else {
            (session.selected_index + 1) % len
        };
        let visible = crate::components::inline_dropdown::inline_dropdown_visible_range_from_start(
            session.visible_start,
            session.selected_index,
            len,
            MENTION_PICKER_MAX_VISIBLE,
        );
        session.visible_start = visible.start;
    }

    let mut transition =
        AcpComposerPickerTransition::new(AcpComposerPickerState::Open(session)).redraw();
    transition.log_visible_reason = Some(reason);
    transition
}

fn dismiss_transition(
    state: AcpComposerPickerState,
    reason: AcpComposerPickerDismissReason,
    cursor: usize,
) -> AcpComposerPickerTransition {
    let next = match (state, reason) {
        (AcpComposerPickerState::Open(session), AcpComposerPickerDismissReason::Outside) => {
            AcpComposerPickerState::Dismissed(AcpDismissedMentionTrigger {
                trigger: session.trigger,
                trigger_range: session.trigger_range,
                query: session.query,
                cursor,
            })
        }
        _ => AcpComposerPickerState::Closed,
    };
    AcpComposerPickerTransition::new(next).redraw()
}

fn accept_transition(state: AcpComposerPickerState) -> AcpComposerPickerTransition {
    match state {
        AcpComposerPickerState::Open(session) => {
            let mut transition =
                AcpComposerPickerTransition::new(AcpComposerPickerState::Closed).redraw();
            transition.accepted_session = Some(session);
            transition
        }
        other => AcpComposerPickerTransition::new(other),
    }
}

fn slash_toggle_transition(state: AcpComposerPickerState) -> AcpComposerPickerTransition {
    let open_slash = matches!(
        state,
        AcpComposerPickerState::Open(AcpMentionSession {
            trigger: ContextPickerTrigger::Slash,
            ..
        })
    );
    let mut transition = AcpComposerPickerTransition::new(AcpComposerPickerState::Closed).redraw();
    if open_slash {
        transition.clear_slash_input = true;
    } else {
        transition.insert_slash_input = true;
    }
    transition
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::window::context_picker::{
        slash_picker_loading_row, types::ContextPickerTrigger,
    };

    fn slash_trigger(query: &str, cursor: usize) -> AcpDismissedMentionTrigger {
        AcpDismissedMentionTrigger {
            trigger: ContextPickerTrigger::Slash,
            trigger_range: 0..(query.chars().count() + 1),
            query: query.to_string(),
            cursor,
        }
    }

    fn session(trigger: ContextPickerTrigger, item_count: usize) -> AcpMentionSession {
        let mut items = Vec::new();
        for _ in 0..item_count {
            items.push(slash_picker_loading_row());
        }
        AcpMentionSession {
            trigger,
            trigger_range: 0..1,
            query: String::new(),
            selected_index: 0,
            visible_start: 0,
            items,
        }
    }

    #[test]
    fn closed_refresh_active_slash_opens_slash_session() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Closed,
            AcpComposerPickerEvent::Refresh(AcpComposerPickerRefreshInput {
                active_trigger: Some(slash_trigger("", 1)),
                next_session: Some(session(ContextPickerTrigger::Slash, 1)),
                focused_inline_preview: false,
            }),
        );

        assert!(matches!(
            transition.state,
            AcpComposerPickerState::Open(AcpMentionSession {
                trigger: ContextPickerTrigger::Slash,
                ..
            })
        ));
        assert!(transition.close_competing_popups);
        assert!(transition.clear_last_accepted_item);
    }

    #[test]
    fn navigate_wraps_selection_and_updates_visible_start() {
        let mut open = session(ContextPickerTrigger::Mention, 10);
        open.selected_index = 0;
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Open(open),
            AcpComposerPickerEvent::NavigatePrevious,
        );
        let AcpComposerPickerState::Open(session) = transition.state else {
            panic!("expected open state");
        };
        assert_eq!(session.selected_index, 9);
        assert_eq!(session.visible_start, 2);
        assert_eq!(transition.log_visible_reason, Some("keyboard_prev"));
    }

    #[test]
    fn outside_dismiss_records_exact_trigger() {
        let open = session(ContextPickerTrigger::Mention, 1);
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Open(open),
            AcpComposerPickerEvent::Dismiss {
                reason: AcpComposerPickerDismissReason::Outside,
                cursor: 1,
            },
        );

        assert!(matches!(
            transition.state,
            AcpComposerPickerState::Dismissed(AcpDismissedMentionTrigger {
                trigger: ContextPickerTrigger::Mention,
                cursor: 1,
                ..
            })
        ));
    }

    #[test]
    fn escape_dismiss_closes_without_suppression() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Open(session(ContextPickerTrigger::Slash, 1)),
            AcpComposerPickerEvent::Dismiss {
                reason: AcpComposerPickerDismissReason::Escape,
                cursor: 1,
            },
        );

        assert!(matches!(transition.state, AcpComposerPickerState::Closed));
    }

    #[test]
    fn dismissed_refresh_same_trigger_stays_suppressed() {
        let trigger = slash_trigger("", 1);
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Dismissed(trigger.clone()),
            AcpComposerPickerEvent::Refresh(AcpComposerPickerRefreshInput {
                active_trigger: Some(trigger),
                next_session: None,
                focused_inline_preview: false,
            }),
        );

        assert!(matches!(
            transition.state,
            AcpComposerPickerState::Dismissed(_)
        ));
    }

    #[test]
    fn dismissed_refresh_changed_trigger_clears_suppression() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Dismissed(slash_trigger("", 1)),
            AcpComposerPickerEvent::Refresh(AcpComposerPickerRefreshInput {
                active_trigger: None,
                next_session: None,
                focused_inline_preview: false,
            }),
        );

        assert!(matches!(transition.state, AcpComposerPickerState::Closed));
    }

    #[test]
    fn accept_inert_row_can_keep_session_open() {
        let open = session(ContextPickerTrigger::Slash, 1);
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Closed,
            AcpComposerPickerEvent::AcceptIgnoredKeepOpen(open),
        );

        assert!(matches!(transition.state, AcpComposerPickerState::Open(_)));
    }

    #[test]
    fn accept_actionable_row_closes_and_returns_session() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Open(session(ContextPickerTrigger::Slash, 1)),
            AcpComposerPickerEvent::Accept,
        );

        assert!(matches!(transition.state, AcpComposerPickerState::Closed));
        assert!(transition.accepted_session.is_some());
    }

    #[test]
    fn submit_started_closes_picker() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Open(session(ContextPickerTrigger::Mention, 1)),
            AcpComposerPickerEvent::SubmitStarted,
        );

        assert!(matches!(transition.state, AcpComposerPickerState::Closed));
    }

    #[test]
    fn slash_toggle_from_closed_requests_slash_input() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Closed,
            AcpComposerPickerEvent::SlashToggle,
        );

        assert!(transition.insert_slash_input);
        assert!(!transition.clear_slash_input);
    }

    #[test]
    fn slash_toggle_from_open_slash_requests_input_clear() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Open(session(ContextPickerTrigger::Slash, 1)),
            AcpComposerPickerEvent::SlashToggle,
        );

        assert!(transition.clear_slash_input);
        assert!(!transition.insert_slash_input);
    }

    #[test]
    fn focused_inline_token_preview_closes_without_opening_empty_state() {
        let transition = reduce_acp_composer_picker(
            AcpComposerPickerState::Open(session(ContextPickerTrigger::Mention, 1)),
            AcpComposerPickerEvent::Refresh(AcpComposerPickerRefreshInput {
                active_trigger: Some(slash_trigger("", 1)),
                next_session: None,
                focused_inline_preview: true,
            }),
        );

        assert!(matches!(transition.state, AcpComposerPickerState::Closed));
    }
}

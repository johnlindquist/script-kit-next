//! State transitions for the Agent Chat composer picker lifecycle.

use crate::ai::context_selector::types::ContextSelectorTrigger;

use super::types::{AgentChatDismissedMentionTrigger, AgentChatMentionSession};

const MENTION_PICKER_MAX_VISIBLE: usize = 8;

/// Surface-local Spine state for the Agent Chat composer. Mirrors the main-menu
/// `ScriptListApp` Spine fields but stays scoped to a single `AgentChatView`
/// so the composer can own its own projection/selection.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct AgentChatComposerSpineState {
    pub(crate) input: crate::spine::input_projection::SpineInputProjection,
    pub(crate) selected_index: usize,
    pub(crate) visible_start: usize,
    /// Snapshot of the thread cwd taken on composer refresh (the spine
    /// section builders run without `cx`, so they cannot read the thread
    /// entity). Scopes the `@project:` subsearch.
    pub(crate) project_scope_cwd: Option<std::path::PathBuf>,
}

#[allow(dead_code)]
impl AgentChatComposerSpineState {
    /// Re-parse the composer text + cursor and update the projection.
    pub(crate) fn refresh(&mut self, text: &str, cursor_chars: usize) {
        self.input =
            crate::spine::input_projection::project_text_at_char_cursor(text, cursor_chars);
        if !self.owns_list() {
            self.selected_index = 0;
            self.visible_start = 0;
        }
    }

    /// Does the projection currently own the conversation-area list (i.e. a
    /// sigil segment is active or the cursor is on a prompt-builder tail with
    /// at least one resolved segment)?
    pub(crate) fn owns_list(&self) -> bool {
        crate::spine::input_projection::projection_owns_prompt_builder_list(
            self.input.projection.as_ref(),
            &self.input.parse,
        )
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AgentChatComposerPickerState {
    Closed,
    Open(AgentChatMentionSession),
    Dismissed(AgentChatDismissedMentionTrigger),
}

#[derive(Debug, Clone)]
pub(crate) struct AgentChatComposerPickerRefreshInput {
    pub(crate) active_trigger: Option<AgentChatDismissedMentionTrigger>,
    pub(crate) next_session: Option<AgentChatMentionSession>,
    pub(crate) focused_inline_preview: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatComposerPickerDismissReason {
    Outside,
    Escape,
    SubmitStarted,
    HostHide,
    PortalStaged,
    SlashToggleClosed,
}

#[derive(Debug, Clone)]
pub(crate) enum AgentChatComposerPickerEvent {
    Refresh(AgentChatComposerPickerRefreshInput),
    NavigatePrevious,
    NavigateNext,
    Dismiss {
        reason: AgentChatComposerPickerDismissReason,
        cursor: usize,
    },
    Accept,
    AcceptIgnoredKeepOpen(AgentChatMentionSession),
    SubmitStarted,
    SlashToggle,
}

#[derive(Debug, Clone)]
pub(crate) struct AgentChatComposerPickerTransition {
    pub(crate) state: AgentChatComposerPickerState,
    pub(crate) sync_popup: bool,
    pub(crate) notify: bool,
    pub(crate) close_competing_popups: bool,
    pub(crate) clear_last_accepted_item: bool,
    pub(crate) log_visible_reason: Option<&'static str>,
    pub(crate) accepted_session: Option<AgentChatMentionSession>,
    pub(crate) insert_slash_input: bool,
    pub(crate) clear_slash_input: bool,
}

impl AgentChatComposerPickerTransition {
    fn new(state: AgentChatComposerPickerState) -> Self {
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

pub(crate) fn reduce_agent_chat_composer_picker(
    state: AgentChatComposerPickerState,
    event: AgentChatComposerPickerEvent,
) -> AgentChatComposerPickerTransition {
    match event {
        AgentChatComposerPickerEvent::Refresh(input) => refresh_transition(input),
        AgentChatComposerPickerEvent::NavigatePrevious => {
            navigate_transition(state, -1, "keyboard_prev")
        }
        AgentChatComposerPickerEvent::NavigateNext => {
            navigate_transition(state, 1, "keyboard_next")
        }
        AgentChatComposerPickerEvent::Dismiss { reason, cursor } => {
            dismiss_transition(state, reason, cursor)
        }
        AgentChatComposerPickerEvent::Accept => accept_transition(state),
        AgentChatComposerPickerEvent::AcceptIgnoredKeepOpen(session) => {
            AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Open(session))
        }
        AgentChatComposerPickerEvent::SubmitStarted => dismiss_transition(
            state,
            AgentChatComposerPickerDismissReason::SubmitStarted,
            0,
        ),
        AgentChatComposerPickerEvent::SlashToggle => slash_toggle_transition(state),
    }
}

fn refresh_transition(
    input: AgentChatComposerPickerRefreshInput,
) -> AgentChatComposerPickerTransition {
    if input.focused_inline_preview {
        return AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Closed)
            .redraw();
    }

    if let (Some(active), None) = (&input.active_trigger, &input.next_session) {
        return AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Dismissed(
            active.clone(),
        ))
        .redraw();
    }

    if let Some(session) = input.next_session {
        return AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Open(session))
            .opened();
    }

    AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Closed).redraw()
}

fn navigate_transition(
    state: AgentChatComposerPickerState,
    delta: isize,
    reason: &'static str,
) -> AgentChatComposerPickerTransition {
    let AgentChatComposerPickerState::Open(mut session) = state else {
        return AgentChatComposerPickerTransition::new(state);
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
        AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Open(session))
            .redraw();
    transition.log_visible_reason = Some(reason);
    transition
}

fn dismiss_transition(
    state: AgentChatComposerPickerState,
    reason: AgentChatComposerPickerDismissReason,
    cursor: usize,
) -> AgentChatComposerPickerTransition {
    let next = match (state, reason) {
        (
            AgentChatComposerPickerState::Open(session),
            AgentChatComposerPickerDismissReason::Outside,
        ) => AgentChatComposerPickerState::Dismissed(AgentChatDismissedMentionTrigger {
            trigger: session.trigger,
            trigger_range: session.trigger_range,
            query: session.query,
            cursor,
        }),
        _ => AgentChatComposerPickerState::Closed,
    };
    AgentChatComposerPickerTransition::new(next).redraw()
}

fn accept_transition(state: AgentChatComposerPickerState) -> AgentChatComposerPickerTransition {
    match state {
        AgentChatComposerPickerState::Open(session) => {
            let mut transition =
                AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Closed)
                    .redraw();
            transition.accepted_session = Some(session);
            transition
        }
        other => AgentChatComposerPickerTransition::new(other),
    }
}

fn slash_toggle_transition(
    state: AgentChatComposerPickerState,
) -> AgentChatComposerPickerTransition {
    let open_slash = matches!(
        state,
        AgentChatComposerPickerState::Open(AgentChatMentionSession {
            trigger: ContextSelectorTrigger::Slash,
            ..
        })
    );
    let mut transition =
        AgentChatComposerPickerTransition::new(AgentChatComposerPickerState::Closed).redraw();
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
    use crate::ai::context_selector::{slash_command_loading_row, types::ContextSelectorTrigger};

    fn slash_trigger(query: &str, cursor: usize) -> AgentChatDismissedMentionTrigger {
        AgentChatDismissedMentionTrigger {
            trigger: ContextSelectorTrigger::Slash,
            trigger_range: 0..(query.chars().count() + 1),
            query: query.to_string(),
            cursor,
        }
    }

    fn session(trigger: ContextSelectorTrigger, item_count: usize) -> AgentChatMentionSession {
        let mut items = Vec::new();
        for _ in 0..item_count {
            items.push(slash_command_loading_row());
        }
        AgentChatMentionSession {
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
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Closed,
            AgentChatComposerPickerEvent::Refresh(AgentChatComposerPickerRefreshInput {
                active_trigger: Some(slash_trigger("", 1)),
                next_session: Some(session(ContextSelectorTrigger::Slash, 1)),
                focused_inline_preview: false,
            }),
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Open(AgentChatMentionSession {
                trigger: ContextSelectorTrigger::Slash,
                ..
            })
        ));
        assert!(transition.close_competing_popups);
        assert!(transition.clear_last_accepted_item);
    }

    #[test]
    fn navigate_wraps_selection_and_updates_visible_start() {
        let mut open = session(ContextSelectorTrigger::Slash, 10);
        open.selected_index = 0;
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Open(open),
            AgentChatComposerPickerEvent::NavigatePrevious,
        );
        let AgentChatComposerPickerState::Open(session) = transition.state else {
            panic!("expected open state");
        };
        assert_eq!(session.selected_index, 9);
        assert_eq!(session.visible_start, 2);
        assert_eq!(transition.log_visible_reason, Some("keyboard_prev"));
    }

    #[test]
    fn outside_dismiss_records_exact_trigger() {
        let open = session(ContextSelectorTrigger::Slash, 1);
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Open(open),
            AgentChatComposerPickerEvent::Dismiss {
                reason: AgentChatComposerPickerDismissReason::Outside,
                cursor: 1,
            },
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Dismissed(AgentChatDismissedMentionTrigger {
                trigger: ContextSelectorTrigger::Slash,
                cursor: 1,
                ..
            })
        ));
    }

    #[test]
    fn escape_dismiss_closes_without_suppression() {
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Open(session(ContextSelectorTrigger::Slash, 1)),
            AgentChatComposerPickerEvent::Dismiss {
                reason: AgentChatComposerPickerDismissReason::Escape,
                cursor: 1,
            },
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Closed
        ));
    }

    #[test]
    fn dismissed_refresh_same_trigger_stays_suppressed() {
        let trigger = slash_trigger("", 1);
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Dismissed(trigger.clone()),
            AgentChatComposerPickerEvent::Refresh(AgentChatComposerPickerRefreshInput {
                active_trigger: Some(trigger),
                next_session: None,
                focused_inline_preview: false,
            }),
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Dismissed(_)
        ));
    }

    #[test]
    fn dismissed_refresh_changed_trigger_clears_suppression() {
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Dismissed(slash_trigger("", 1)),
            AgentChatComposerPickerEvent::Refresh(AgentChatComposerPickerRefreshInput {
                active_trigger: None,
                next_session: None,
                focused_inline_preview: false,
            }),
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Closed
        ));
    }

    #[test]
    fn accept_inert_row_can_keep_session_open() {
        let open = session(ContextSelectorTrigger::Slash, 1);
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Closed,
            AgentChatComposerPickerEvent::AcceptIgnoredKeepOpen(open),
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Open(_)
        ));
    }

    #[test]
    fn accept_actionable_row_closes_and_returns_session() {
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Open(session(ContextSelectorTrigger::Slash, 1)),
            AgentChatComposerPickerEvent::Accept,
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Closed
        ));
        assert!(transition.accepted_session.is_some());
    }

    #[test]
    fn submit_started_closes_picker() {
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Open(session(ContextSelectorTrigger::Slash, 1)),
            AgentChatComposerPickerEvent::SubmitStarted,
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Closed
        ));
    }

    #[test]
    fn slash_toggle_from_closed_requests_slash_input() {
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Closed,
            AgentChatComposerPickerEvent::SlashToggle,
        );

        assert!(transition.insert_slash_input);
        assert!(!transition.clear_slash_input);
    }

    #[test]
    fn slash_toggle_from_open_slash_requests_input_clear() {
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Open(session(ContextSelectorTrigger::Slash, 1)),
            AgentChatComposerPickerEvent::SlashToggle,
        );

        assert!(transition.clear_slash_input);
        assert!(!transition.insert_slash_input);
    }

    #[test]
    fn focused_inline_token_preview_closes_without_opening_empty_state() {
        let transition = reduce_agent_chat_composer_picker(
            AgentChatComposerPickerState::Open(session(ContextSelectorTrigger::Slash, 1)),
            AgentChatComposerPickerEvent::Refresh(AgentChatComposerPickerRefreshInput {
                active_trigger: Some(slash_trigger("", 1)),
                next_session: None,
                focused_inline_preview: true,
            }),
        );

        assert!(matches!(
            transition.state,
            AgentChatComposerPickerState::Closed
        ));
    }
}

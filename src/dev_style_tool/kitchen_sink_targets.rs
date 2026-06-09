pub(crate) const OPEN_MAIN_WINDOW_KITCHEN_SINK_BUTTON: &str =
    "button:dev-style-tool-open-main-window-kitchen-sink";
pub(crate) const OPEN_MAIN_WINDOW_NO_MATCH_KITCHEN_SINK_BUTTON: &str =
    "button:dev-style-tool-open-main-window-no-match-kitchen-sink";
pub(crate) const OPEN_ACTIONS_POPUP_KITCHEN_SINK_BUTTON: &str =
    "button:dev-style-tool-open-actions-popup-kitchen-sink";
pub(crate) const OPEN_ACTIONS_POPUP_NO_MATCH_KITCHEN_SINK_BUTTON: &str =
    "button:dev-style-tool-open-actions-popup-no-match-kitchen-sink";
pub(crate) const OPEN_AGENT_CHAT_KITCHEN_SINK_BUTTON: &str =
    "button:dev-style-tool-open-agent-chat-kitchen-sink";
pub(crate) const OPEN_CONFIRM_MODAL_KITCHEN_SINK_BUTTON: &str =
    "button:dev-style-tool-open-confirm-modal-kitchen-sink";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DevStyleKitchenSinkTarget {
    MainWindowPopulated,
    MainWindowNoMatch,
    ActionsPopupPopulated,
    ActionsPopupNoMatch,
    AgentChat,
    ConfirmModal,
}

impl DevStyleKitchenSinkTarget {
    pub(crate) const ALL: &'static [Self] = &[
        Self::MainWindowPopulated,
        Self::MainWindowNoMatch,
        Self::ActionsPopupPopulated,
        Self::ActionsPopupNoMatch,
        Self::AgentChat,
        Self::ConfirmModal,
    ];

    pub(crate) const fn semantic_id(self) -> &'static str {
        match self {
            Self::MainWindowPopulated => OPEN_MAIN_WINDOW_KITCHEN_SINK_BUTTON,
            Self::MainWindowNoMatch => OPEN_MAIN_WINDOW_NO_MATCH_KITCHEN_SINK_BUTTON,
            Self::ActionsPopupPopulated => OPEN_ACTIONS_POPUP_KITCHEN_SINK_BUTTON,
            Self::ActionsPopupNoMatch => OPEN_ACTIONS_POPUP_NO_MATCH_KITCHEN_SINK_BUTTON,
            Self::AgentChat => OPEN_AGENT_CHAT_KITCHEN_SINK_BUTTON,
            Self::ConfirmModal => OPEN_CONFIRM_MODAL_KITCHEN_SINK_BUTTON,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::MainWindowPopulated => "Open Main Window Kitchen Sink",
            Self::MainWindowNoMatch => "Open Main Window No-Match Sink",
            Self::ActionsPopupPopulated => "Open Actions Popup Kitchen Sink",
            Self::ActionsPopupNoMatch => "Open Actions Popup No-Match Sink",
            Self::AgentChat => "Open Agent Chat Kitchen Sink",
            Self::ConfirmModal => "Open Confirm Modal Preview",
        }
    }

    pub(crate) const fn action_value(self) -> &'static str {
        match self {
            Self::MainWindowPopulated => "openMainWindowKitchenSink",
            Self::MainWindowNoMatch => "openMainWindowNoMatchKitchenSink",
            Self::ActionsPopupPopulated => "openActionsPopupKitchenSink",
            Self::ActionsPopupNoMatch => "openActionsPopupNoMatchKitchenSink",
            Self::AgentChat => "openAgentChatKitchenSink",
            Self::ConfirmModal => "openConfirmModalKitchenSink",
        }
    }
}

//! Canonical Mini Agent Chat state fixtures.
//!
//! The existing mini chat variation story compares adoptable visual styles.
//! These fixtures cover supported runtime states through the shared presenter.

use gpui::AnyElement;

use crate::storybook::{
    render_mini_ai_chat_presentation, resolve_mini_ai_chat_style, MiniAiChatPresentationMessage,
    MiniAiChatPresentationModel, MiniAiChatRole, MiniAiChatSuggestion, StoryVariant,
};
use crate::theme::get_cached_theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiniAiChatStateId {
    Welcome,
    Conversation,
    Streaming,
    DraftInput,
    ActionHints,
    Error,
}

impl MiniAiChatStateId {
    pub const ALL: [Self; 6] = [
        Self::Welcome,
        Self::Conversation,
        Self::Streaming,
        Self::DraftInput,
        Self::ActionHints,
        Self::Error,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Welcome => "welcome",
            Self::Conversation => "conversation",
            Self::Streaming => "streaming",
            Self::DraftInput => "draft-input",
            Self::ActionHints => "action-hints",
            Self::Error => "error",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Welcome => "Welcome",
            Self::Conversation => "Conversation",
            Self::Streaming => "Streaming",
            Self::DraftInput => "Draft Input",
            Self::ActionHints => "Action Hints",
            Self::Error => "Error",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Welcome => "New mini chat with welcome suggestions.",
            Self::Conversation => "Settled user and assistant exchange.",
            Self::Streaming => "Assistant streaming state with stop affordance.",
            Self::DraftInput => "Composer contains unsent text.",
            Self::ActionHints => "Transcript state with action hint strip visible.",
            Self::Error => "Assistant error message after a failed request.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "welcome" => Some(Self::Welcome),
            "conversation" => Some(Self::Conversation),
            "streaming" => Some(Self::Streaming),
            "draft-input" => Some(Self::DraftInput),
            "action-hints" => Some(Self::ActionHints),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

pub fn mini_ai_chat_state_story_variants() -> Vec<StoryVariant> {
    MiniAiChatStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "miniAiChat")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_mini_ai_chat_state_preview(stable_id: &str) -> AnyElement {
    let id = MiniAiChatStateId::from_stable_id(stable_id).unwrap_or(MiniAiChatStateId::Welcome);
    render_mini_ai_chat_state(id)
}

pub fn render_mini_ai_chat_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_mini_ai_chat_state_preview(stable_id)
}

fn render_mini_ai_chat_state(id: MiniAiChatStateId) -> AnyElement {
    let (style, _) = resolve_mini_ai_chat_style(Some("current"));
    render_mini_ai_chat_presentation(&state_model(id), style, &get_cached_theme())
}

fn state_model(id: MiniAiChatStateId) -> MiniAiChatPresentationModel {
    match id {
        MiniAiChatStateId::Welcome => MiniAiChatPresentationModel {
            title: "New Chat".into(),
            is_streaming: false,
            model_name: "Sonnet".into(),
            input_text: "".into(),
            input_placeholder: "Ask anything...".into(),
            messages: Vec::new(),
            show_welcome: true,
            welcome_suggestions: suggestions(),
        },
        MiniAiChatStateId::Conversation => base_model(
            false,
            "",
            vec![
                user("Can you summarize the Storybook cleanup?"),
                assistant("Main menu and dictation stay. PNG fixtures are out. Canonical state stories cover the app windows next."),
            ],
        ),
        MiniAiChatStateId::Streaming => base_model(
            true,
            "",
            vec![
                user("What should I add next?"),
                assistant("I'd move from windows to built-in browser surfaces, then fill lower-level primitives"),
            ],
        ),
        MiniAiChatStateId::DraftInput => base_model(
            false,
            "Turn the remaining built-ins into stories",
            vec![assistant("Ready when you are.")],
        ),
        MiniAiChatStateId::ActionHints => base_model(
            false,
            "",
            vec![
                user("Open the actions for this response."),
                assistant("Use the command menu for copy, retry, new chat, and attach-latest-output workflows."),
            ],
        ),
        MiniAiChatStateId::Error => base_model(
            false,
            "",
            vec![
                user("Run the agent."),
                assistant("Error: the selected model is unavailable. Choose another model or retry after reconnecting."),
            ],
        ),
    }
}

fn base_model(
    is_streaming: bool,
    input_text: &'static str,
    messages: Vec<MiniAiChatPresentationMessage>,
) -> MiniAiChatPresentationModel {
    MiniAiChatPresentationModel {
        title: "Mini Agent Chat".into(),
        is_streaming,
        model_name: "Sonnet".into(),
        input_text: input_text.into(),
        input_placeholder: "Follow up...".into(),
        messages,
        show_welcome: false,
        welcome_suggestions: suggestions(),
    }
}

fn user(content: &'static str) -> MiniAiChatPresentationMessage {
    MiniAiChatPresentationMessage {
        role: MiniAiChatRole::User,
        content: content.into(),
    }
}

fn assistant(content: &'static str) -> MiniAiChatPresentationMessage {
    MiniAiChatPresentationMessage {
        role: MiniAiChatRole::Assistant,
        content: content.into(),
    }
}

fn suggestions() -> Vec<MiniAiChatSuggestion> {
    vec![
        MiniAiChatSuggestion {
            title: "Summarize current state".into(),
            shortcut: "\u{2318}1".into(),
        },
        MiniAiChatSuggestion {
            title: "Attach latest output".into(),
            shortcut: "\u{2318}2".into(),
        },
        MiniAiChatSuggestion {
            title: "Open actions".into(),
            shortcut: "\u{2318}K".into(),
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AcpChatUiVariant {
    #[default]
    Standard,
    UserBold,
    RoleSplit,
    BottomDock,
    DenseLog,
    Sidecar,
}

impl AcpChatUiVariant {
    pub(crate) const EXPERIMENTS: [Self; 5] = [
        Self::UserBold,
        Self::RoleSplit,
        Self::BottomDock,
        Self::DenseLog,
        Self::Sidecar,
    ];

    pub(crate) fn state_id(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::UserBold => "user-bold",
            Self::RoleSplit => "role-split",
            Self::BottomDock => "bottom-dock",
            Self::DenseLog => "dense-log",
            Self::Sidecar => "sidecar",
        }
    }

    pub(crate) fn menu_id(self) -> &'static str {
        match self {
            Self::Standard => "builtin/ai-chat",
            Self::UserBold => "builtin/ai-chat/user-bold",
            Self::RoleSplit => "builtin/ai-chat/role-split",
            Self::BottomDock => "builtin/ai-chat/bottom-dock",
            Self::DenseLog => "builtin/ai-chat/dense-log",
            Self::Sidecar => "builtin/ai-chat/sidecar",
        }
    }

    pub(crate) fn menu_name(self) -> &'static str {
        match self {
            Self::Standard => "Agent Chat",
            Self::UserBold => "Agent Chat - Bold User",
            Self::RoleSplit => "Agent Chat - Left/Right",
            Self::BottomDock => "Agent Chat - Bottom Input",
            Self::DenseLog => "Agent Chat - Dense Log",
            Self::Sidecar => "Agent Chat - Sidecar",
        }
    }

    pub(crate) fn menu_description(self) -> &'static str {
        match self {
            Self::Standard => "Open Agent Chat with fresh context",
            Self::UserBold => "Open Agent Chat with emphasized user messages",
            Self::RoleSplit => "Open Agent Chat with assistant left and user right",
            Self::BottomDock => "Open Agent Chat with the input docked at the bottom",
            Self::DenseLog => "Open Agent Chat in a compact transcript layout",
            Self::Sidecar => "Open Agent Chat with a live state sidecar",
        }
    }

    pub(crate) fn footer_label(self) -> &'static str {
        match self {
            Self::Standard => "Agent",
            Self::UserBold => "Bold",
            Self::RoleSplit => "Split",
            Self::BottomDock => "Bottom",
            Self::DenseLog => "Log",
            Self::Sidecar => "Sidecar",
        }
    }

    pub(crate) fn keywords(self) -> Vec<&'static str> {
        let mut keywords = vec![
            "ai",
            "agent",
            "chat",
            "assistant",
            "acp",
            "ui",
            "variant",
            "design",
        ];
        match self {
            Self::Standard => keywords.extend(["harness", "gpt", "llm", "tab"]),
            Self::UserBold => keywords.extend(["bold", "user", "message", "emphasis"]),
            Self::RoleSplit => keywords.extend(["left", "right", "assistant", "user", "bubbles"]),
            Self::BottomDock => keywords.extend(["bottom", "input", "composer", "dock"]),
            Self::DenseLog => keywords.extend(["dense", "compact", "log", "transcript"]),
            Self::Sidecar => keywords.extend(["sidecar", "rail", "state", "status", "metadata"]),
        }
        keywords
    }

    pub(crate) fn config(self) -> AcpChatUiConfig {
        match self {
            Self::Standard => AcpChatUiConfig {
                transcript: AcpTranscriptPresentation::Standard,
                composer: AcpComposerPlacement::Default,
                chrome: AcpChromeDensity::Default,
                show_sidecar: false,
                show_variant_badge: false,
            },
            Self::UserBold => AcpChatUiConfig {
                transcript: AcpTranscriptPresentation::UserBold,
                composer: AcpComposerPlacement::Default,
                chrome: AcpChromeDensity::Default,
                show_sidecar: false,
                show_variant_badge: true,
            },
            Self::RoleSplit => AcpChatUiConfig {
                transcript: AcpTranscriptPresentation::RoleSplit,
                composer: AcpComposerPlacement::Default,
                chrome: AcpChromeDensity::Default,
                show_sidecar: false,
                show_variant_badge: true,
            },
            Self::BottomDock => AcpChatUiConfig {
                transcript: AcpTranscriptPresentation::Standard,
                composer: AcpComposerPlacement::BottomDock,
                chrome: AcpChromeDensity::Compact,
                show_sidecar: false,
                show_variant_badge: true,
            },
            Self::DenseLog => AcpChatUiConfig {
                transcript: AcpTranscriptPresentation::DenseLog,
                composer: AcpComposerPlacement::BottomDock,
                chrome: AcpChromeDensity::Compact,
                show_sidecar: false,
                show_variant_badge: true,
            },
            Self::Sidecar => AcpChatUiConfig {
                transcript: AcpTranscriptPresentation::RoleSplit,
                composer: AcpComposerPlacement::BottomDock,
                chrome: AcpChromeDensity::Default,
                show_sidecar: true,
                show_variant_badge: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcpChatUiConfig {
    pub(crate) transcript: AcpTranscriptPresentation,
    pub(crate) composer: AcpComposerPlacement,
    pub(crate) chrome: AcpChromeDensity,
    pub(crate) show_sidecar: bool,
    pub(crate) show_variant_badge: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpTranscriptPresentation {
    Standard,
    UserBold,
    RoleSplit,
    DenseLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpComposerPlacement {
    Default,
    BottomDock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpChromeDensity {
    Default,
    Compact,
}

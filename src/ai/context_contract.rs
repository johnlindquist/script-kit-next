use crate::ai::message_parts::AiContextPart;

pub(crate) const CONTEXT_SECTION: &str = "Context";
pub(crate) const CLEAR_CONTEXT_ACTION_ID: &str = "chat:clear_context";
pub(crate) const CLEAR_CONTEXT_ACTION_TITLE: &str = "Clear Context";
const CURRENT_SNAPSHOT_MENTION_ALIASES: &[&str] = &["@context"];
const FULL_SNAPSHOT_MENTION_ALIASES: &[&str] = &["@context-full"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextAttachmentKind {
    Current,
    Full,
    Selection,
    Browser,
    Window,
    Diagnostics,
    Screenshot,
    Clipboard,
    FrontmostApp,
    MenuBar,
    RecentScripts,
    GitStatus,
    GitDiff,
    Processes,
    System,
    Dictation,
    Calendar,
    Notifications,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextAttachmentSpec {
    pub kind: ContextAttachmentKind,
    pub action_id: &'static str,
    pub action_title: &'static str,
    pub slash_command: Option<&'static str>,
    pub mention: Option<&'static str>,
    pub slash_aliases: &'static [&'static str],
    pub mention_aliases: &'static [&'static str],
    pub uri: &'static str,
    pub label: &'static str,
}

const CONTEXT_ATTACHMENT_SPECS: [ContextAttachmentSpec; 18] = [
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Current,
        action_id: "chat:add_current_context",
        action_title: "Attach Current Snapshot",
        slash_command: Some("/context"),
        mention: Some("@snapshot"),
        slash_aliases: &[],
        mention_aliases: CURRENT_SNAPSHOT_MENTION_ALIASES,
        uri: "kit://context?profile=minimal",
        label: "Current Context",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Full,
        action_id: "chat:add_context_full",
        action_title: "Attach Full Snapshot",
        slash_command: Some("/context-full"),
        mention: Some("@snapshot-full"),
        slash_aliases: &[],
        mention_aliases: FULL_SNAPSHOT_MENTION_ALIASES,
        uri: "kit://context",
        label: "Current Context (Full)",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Selection,
        action_id: "chat:add_selection_context",
        action_title: "Attach Selected Text",
        slash_command: Some("/selection"),
        mention: Some("@selection"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
        label: "Selection",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Browser,
        action_id: "chat:add_browser_context",
        action_title: "Attach Browser URL",
        slash_command: Some("/browser"),
        mention: Some("@browser"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0",
        label: "Browser URL",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Window,
        action_id: "chat:add_window_context",
        action_title: "Attach Focused Window",
        slash_command: Some("/window"),
        mention: Some("@window"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1",
        label: "Focused Window",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Diagnostics,
        action_id: "chat:add_context_diagnostics",
        action_title: "Attach Context Diagnostics",
        slash_command: None,
        mention: Some("@diagnostics"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://context?diagnostics=1",
        label: "Context Diagnostics",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Screenshot,
        action_id: "chat:add_screenshot",
        action_title: "Attach Screenshot",
        slash_command: Some("/screenshot"),
        mention: Some("@screenshot"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://context?screenshot=1&selectedText=0&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
        label: "Screenshot",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Clipboard,
        action_id: "chat:add_clipboard",
        action_title: "Attach Clipboard History",
        slash_command: Some("/clipboard"),
        mention: Some("@clipboard"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://clipboard-history",
        label: "Clipboard",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::FrontmostApp,
        action_id: "chat:add_frontmost_app",
        action_title: "Attach Frontmost App",
        slash_command: Some("/frontmost-app"),
        mention: Some("@frontmost-app"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=0",
        label: "Frontmost App",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::MenuBar,
        action_id: "chat:add_menu_bar",
        action_title: "Attach Menu Bar",
        slash_command: Some("/menu-bar"),
        mention: Some("@menu-bar"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=1&browserUrl=0&focusedWindow=0",
        label: "Menu Bar",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::RecentScripts,
        action_id: "chat:add_recent_scripts",
        action_title: "Attach Recent Scripts",
        slash_command: Some("/recent-scripts"),
        mention: Some("@recent-scripts"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://scripts",
        label: "Recent Scripts",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::GitStatus,
        action_id: "chat:add_git_status",
        action_title: "Attach Git Status",
        slash_command: Some("/git-status"),
        mention: Some("@git-status"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://git-status",
        label: "Git Status",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::GitDiff,
        action_id: "chat:add_git_diff",
        action_title: "Attach Git Diff",
        slash_command: Some("/git-diff"),
        mention: Some("@git-diff"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://git-diff",
        label: "Git Diff",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Processes,
        action_id: "chat:add_processes",
        action_title: "Attach Running Processes",
        slash_command: Some("/processes"),
        mention: Some("@processes"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://processes",
        label: "Processes",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::System,
        action_id: "chat:add_system",
        action_title: "Attach System Info",
        slash_command: Some("/system"),
        mention: Some("@system"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://system",
        label: "System Info",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Dictation,
        action_id: "chat:add_dictation",
        action_title: "Attach Dictation",
        slash_command: Some("/dictation"),
        mention: Some("@dictation"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://dictation",
        label: "Dictation",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Calendar,
        action_id: "chat:add_calendar",
        action_title: "Attach Calendar Events",
        slash_command: Some("/calendar"),
        mention: Some("@calendar"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://calendar",
        label: "Calendar",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Notifications,
        action_id: "chat:add_notifications",
        action_title: "Attach Notifications",
        slash_command: Some("/notifications"),
        mention: Some("@notifications"),
        slash_aliases: &[],
        mention_aliases: &[],
        uri: "kit://notifications",
        label: "Notifications",
    },
];

pub fn context_attachment_specs() -> &'static [ContextAttachmentSpec] {
    &CONTEXT_ATTACHMENT_SPECS
}

impl ContextAttachmentKind {
    /// Returns the spec for this kind.
    ///
    /// # Panics
    ///
    /// Cannot fail at runtime because the const array is exhaustive over
    /// all `ContextAttachmentKind` variants — this is verified by the
    /// `context_attachment_specs_are_unique_and_roundtrip` test.
    pub fn spec(self) -> &'static ContextAttachmentSpec {
        // Iterate with index so we can return without expect/unwrap.
        let specs = context_attachment_specs();
        let mut i = 0;
        while i < specs.len() {
            if specs[i].kind == self {
                return &specs[i];
            }
            i += 1;
        }
        // Unreachable: every ContextAttachmentKind has a matching entry.
        // Using a const array exhaustively indexed by enum guarantees this.
        unreachable!("missing ContextAttachmentSpec for {self:?}")
    }

    pub fn part(self) -> AiContextPart {
        let spec = self.spec();
        AiContextPart::ResourceUri {
            uri: spec.uri.to_string(),
            label: spec.label.to_string(),
        }
    }

    pub(crate) fn from_action_id(action_id: &str) -> Option<Self> {
        let normalized = if action_id.starts_with("chat:") {
            action_id.to_string()
        } else {
            format!("chat:{action_id}")
        };

        context_attachment_specs()
            .iter()
            .find(|spec| spec.action_id == normalized)
            .map(|spec| spec.kind)
    }

    pub(crate) fn from_slash_command(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        context_attachment_specs()
            .iter()
            .find(|spec| {
                spec.slash_command == Some(trimmed) || spec.slash_aliases.contains(&trimmed)
            })
            .map(|spec| spec.kind)
    }

    pub(crate) fn from_mention_line(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        context_attachment_specs()
            .iter()
            .find(|spec| spec.mention == Some(trimmed) || spec.mention_aliases.contains(&trimmed))
            .map(|spec| spec.kind)
    }

    /// Map to the provider JSON resource kind, if this attachment is
    /// provider-backed (Dictation, Calendar, Notifications).
    pub(crate) fn provider_json_kind(
        self,
    ) -> Option<crate::mcp_resources::ProviderJsonResourceKind> {
        use crate::mcp_resources::ProviderJsonResourceKind;
        match self {
            Self::Dictation => Some(ProviderJsonResourceKind::Dictation),
            Self::Calendar => Some(ProviderJsonResourceKind::Calendar),
            Self::Notifications => Some(ProviderJsonResourceKind::Notifications),
            _ => None,
        }
    }

    /// Whether provider-backed data is available for this attachment kind.
    /// Non-provider kinds always return `true`.
    pub(crate) fn provider_data_available(self) -> bool {
        self.provider_json_kind()
            .map(crate::mcp_resources::has_provider_json_resource)
            .unwrap_or(true)
    }
}

pub(crate) fn is_clear_context_action(action_id: &str) -> bool {
    action_id == CLEAR_CONTEXT_ACTION_ID || action_id == "clear_context"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_attachment_specs_are_unique_and_roundtrip() {
        let mut action_ids = std::collections::HashSet::new();
        let mut uris = std::collections::HashSet::new();

        for spec in context_attachment_specs() {
            assert!(
                action_ids.insert(spec.action_id),
                "duplicate action_id: {}",
                spec.action_id
            );
            assert!(uris.insert(spec.uri), "duplicate uri: {}", spec.uri);

            // Round-trip: action_id → kind
            assert_eq!(
                ContextAttachmentKind::from_action_id(spec.action_id),
                Some(spec.kind),
                "from_action_id round-trip failed for {:?}",
                spec.kind,
            );

            // Round-trip: action_id without prefix → kind
            let suffix = spec
                .action_id
                .strip_prefix("chat:")
                .unwrap_or(spec.action_id);
            assert_eq!(
                ContextAttachmentKind::from_action_id(suffix),
                Some(spec.kind),
                "from_action_id (prefix-less) round-trip failed for {:?}",
                spec.kind,
            );

            // Round-trip: slash_command → kind
            if let Some(slash) = spec.slash_command {
                assert_eq!(
                    ContextAttachmentKind::from_slash_command(slash),
                    Some(spec.kind),
                    "from_slash_command round-trip failed for {:?}",
                    spec.kind,
                );
            }

            // Round-trip: mention → kind
            if let Some(mention) = spec.mention {
                assert_eq!(
                    ContextAttachmentKind::from_mention_line(mention),
                    Some(spec.kind),
                    "from_mention_line round-trip failed for {:?}",
                    spec.kind,
                );
            }

            // Round-trip: kind → part → uri/label
            match spec.kind.part() {
                AiContextPart::ResourceUri { uri, label } => {
                    assert_eq!(uri, spec.uri, "URI mismatch for {:?}", spec.kind);
                    assert_eq!(label, spec.label, "label mismatch for {:?}", spec.kind);
                }
                other => panic!("expected ResourceUri for {:?}, got {other:?}", spec.kind),
            }

            tracing::info!(
                kind = ?spec.kind,
                action_id = spec.action_id,
                uri = spec.uri,
                label = spec.label,
                slash_command = ?spec.slash_command,
                mention = ?spec.mention,
                "context_attachment_spec_verified"
            );
        }

        // Verify clear-context helpers
        assert!(is_clear_context_action("chat:clear_context"));
        assert!(is_clear_context_action("clear_context"));
        assert!(!is_clear_context_action("chat:add_current_context"));

        tracing::info!(
            total_specs = context_attachment_specs().len(),
            unique_action_ids = action_ids.len(),
            unique_uris = uris.len(),
            "context_attachment_roundtrip_complete"
        );
    }

    #[test]
    fn current_snapshot_aliases_remain_accepted() {
        assert_eq!(
            ContextAttachmentKind::from_slash_command("/context"),
            Some(ContextAttachmentKind::Current)
        );
        assert_eq!(
            ContextAttachmentKind::from_mention_line("@context"),
            Some(ContextAttachmentKind::Current)
        );
        assert_eq!(
            ContextAttachmentKind::from_slash_command("/context-full"),
            Some(ContextAttachmentKind::Full)
        );
        assert_eq!(
            ContextAttachmentKind::from_mention_line("@context-full"),
            Some(ContextAttachmentKind::Full)
        );
    }
}

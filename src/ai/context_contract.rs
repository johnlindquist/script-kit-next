use crate::ai::message_parts::AiContextPart;

pub(crate) const CONTEXT_SECTION: &str = "Context";
pub(crate) const CLEAR_CONTEXT_ACTION_ID: &str = "chat:clear_context";
pub(crate) const CLEAR_CONTEXT_ACTION_TITLE: &str = "Clear Context";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ContextAttachmentKind {
    Current,
    Full,
    Selection,
    Browser,
    Window,
    Diagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContextAttachmentSpec {
    pub(crate) kind: ContextAttachmentKind,
    pub(crate) action_id: &'static str,
    pub(crate) action_title: &'static str,
    pub(crate) slash_command: Option<&'static str>,
    pub(crate) mention: Option<&'static str>,
    pub(crate) uri: &'static str,
    pub(crate) label: &'static str,
}

const CONTEXT_ATTACHMENT_SPECS: [ContextAttachmentSpec; 6] = [
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Current,
        action_id: "chat:add_current_context",
        action_title: "Attach Current Context",
        slash_command: Some("/context"),
        mention: Some("@context"),
        uri: "kit://context?profile=minimal",
        label: "Current Context",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Full,
        action_id: "chat:add_context_full",
        action_title: "Attach Full Context",
        slash_command: Some("/context-full"),
        mention: Some("@context-full"),
        uri: "kit://context",
        label: "Current Context (Full)",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Selection,
        action_id: "chat:add_selection_context",
        action_title: "Attach Selected Text",
        slash_command: Some("/selection"),
        mention: Some("@selection"),
        uri: "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
        label: "Selection",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Browser,
        action_id: "chat:add_browser_context",
        action_title: "Attach Browser URL",
        slash_command: Some("/browser"),
        mention: Some("@browser"),
        uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0",
        label: "Browser URL",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Window,
        action_id: "chat:add_window_context",
        action_title: "Attach Focused Window",
        slash_command: Some("/window"),
        mention: Some("@window"),
        uri: "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1",
        label: "Focused Window",
    },
    ContextAttachmentSpec {
        kind: ContextAttachmentKind::Diagnostics,
        action_id: "chat:add_context_diagnostics",
        action_title: "Attach Context Diagnostics",
        slash_command: None,
        mention: Some("@diagnostics"),
        uri: "kit://context?diagnostics=1",
        label: "Context Diagnostics",
    },
];

pub(crate) fn context_attachment_specs() -> &'static [ContextAttachmentSpec] {
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
    pub(crate) fn spec(self) -> &'static ContextAttachmentSpec {
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

    pub(crate) fn part(self) -> AiContextPart {
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
            .find(|spec| spec.slash_command == Some(trimmed))
            .map(|spec| spec.kind)
    }

    pub(crate) fn from_mention_line(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        context_attachment_specs()
            .iter()
            .find(|spec| spec.mention == Some(trimmed))
            .map(|spec| spec.kind)
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
}

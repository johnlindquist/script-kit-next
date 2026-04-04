use super::*;
use crate::ai::context_contract::ContextAttachmentKind;

/// Try to parse a slash command from raw composer input.
///
/// Returns `None` if the input is not a recognized slash command,
/// allowing normal submission to proceed.
pub(super) fn parse_context_slash_command(input: &str) -> Option<ContextAttachmentKind> {
    ContextAttachmentKind::from_slash_command(input)
}

impl AiApp {
    /// Handle a parsed slash command by inserting the appropriate context part
    /// and clearing the composer. Returns `true` if a command was handled.
    pub(super) fn try_handle_slash_command(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let content = self.input_state.read(cx).value().to_string();

        let Some(kind) = parse_context_slash_command(&content) else {
            return false;
        };

        let spec = kind.spec();

        tracing::info!(
            command = ?spec.slash_command,
            uri = spec.uri,
            label = spec.label,
            "slash_command: parsed and inserting context part"
        );

        self.add_context_part(kind.part(), cx);
        self.clear_composer(window, cx);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::context_contract::context_attachment_specs;

    #[test]
    fn test_parse_known_slash_commands() {
        // Every spec with a slash_command must round-trip through the parser
        for spec in context_attachment_specs() {
            if let Some(slash) = spec.slash_command {
                let kind = parse_context_slash_command(slash);
                assert_eq!(
                    kind,
                    Some(spec.kind),
                    "slash command {slash} should parse to {:?}",
                    spec.kind,
                );
            }
        }
    }

    #[test]
    fn test_parse_with_whitespace() {
        assert_eq!(
            parse_context_slash_command("  /context  "),
            Some(ContextAttachmentKind::Current),
            "Leading/trailing whitespace should be trimmed"
        );
    }

    #[test]
    fn test_legacy_context_aliases_still_parse() {
        assert_eq!(
            parse_context_slash_command("/context"),
            Some(ContextAttachmentKind::Current)
        );
        assert_eq!(
            parse_context_slash_command("/context-full"),
            Some(ContextAttachmentKind::Full)
        );
    }

    #[test]
    fn test_parse_non_slash_command_returns_none() {
        assert_eq!(parse_context_slash_command("hello world"), None);
        assert_eq!(parse_context_slash_command("/unknown"), None);
        assert_eq!(parse_context_slash_command(""), None);
        assert_eq!(parse_context_slash_command("/context extra stuff"), None);
    }

    #[test]
    fn test_slash_command_uri_mappings_are_valid_kit_uris() {
        for spec in context_attachment_specs() {
            if let Some(slash) = spec.slash_command {
                assert!(
                    parse_context_slash_command(slash).is_some(),
                    "slash command {slash} should parse"
                );
                assert!(
                    spec.uri.starts_with("kit://"),
                    "URI for {slash} should be a kit:// URI, got: {}",
                    spec.uri
                );
            }
        }
    }
}

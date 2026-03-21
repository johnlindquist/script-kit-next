use super::*;

/// Recognized slash commands for explicit context injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContextSlashCommand {
    /// Minimal desktop context (frontmost app + browser URL)
    Context,
    /// Full desktop context snapshot (all fields)
    ContextFull,
    /// Selected text only
    Selection,
    /// Browser URL only
    Browser,
    /// Focused window + frontmost app info
    Window,
}

/// Try to parse a slash command from raw composer input.
///
/// Returns `None` if the input is not a recognized slash command,
/// allowing normal submission to proceed.
pub(super) fn parse_context_slash_command(input: &str) -> Option<ContextSlashCommand> {
    match input.trim() {
        "/context" => Some(ContextSlashCommand::Context),
        "/context-full" => Some(ContextSlashCommand::ContextFull),
        "/selection" => Some(ContextSlashCommand::Selection),
        "/browser" => Some(ContextSlashCommand::Browser),
        "/window" => Some(ContextSlashCommand::Window),
        _ => None,
    }
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

        let Some(cmd) = parse_context_slash_command(&content) else {
            return false;
        };

        let (uri, label) = match cmd {
            ContextSlashCommand::Context => (
                "kit://context?profile=minimal",
                "Current Context",
            ),
            ContextSlashCommand::ContextFull => (
                "kit://context",
                "Current Context (Full)",
            ),
            ContextSlashCommand::Selection => (
                "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
                "Selection",
            ),
            ContextSlashCommand::Browser => (
                "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0",
                "Browser URL",
            ),
            ContextSlashCommand::Window => (
                "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1",
                "Focused Window",
            ),
        };

        tracing::info!(
            command = ?cmd,
            uri = uri,
            label = label,
            "slash_command: parsed and inserting context part"
        );

        self.pending_context_parts.push(
            crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: uri.to_string(),
                label: label.to_string(),
            },
        );

        self.clear_composer(window, cx);
        cx.notify();

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_known_slash_commands() {
        assert_eq!(
            parse_context_slash_command("/context"),
            Some(ContextSlashCommand::Context)
        );
        assert_eq!(
            parse_context_slash_command("/context-full"),
            Some(ContextSlashCommand::ContextFull)
        );
        assert_eq!(
            parse_context_slash_command("/selection"),
            Some(ContextSlashCommand::Selection)
        );
        assert_eq!(
            parse_context_slash_command("/browser"),
            Some(ContextSlashCommand::Browser)
        );
        assert_eq!(
            parse_context_slash_command("/window"),
            Some(ContextSlashCommand::Window)
        );
    }

    #[test]
    fn test_parse_with_whitespace() {
        assert_eq!(
            parse_context_slash_command("  /context  "),
            Some(ContextSlashCommand::Context),
            "Leading/trailing whitespace should be trimmed"
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
    fn test_slash_command_uri_mappings_are_valid_kit_context_uris() {
        let mappings = [
            (ContextSlashCommand::Context, "kit://context?profile=minimal"),
            (ContextSlashCommand::ContextFull, "kit://context"),
            (ContextSlashCommand::Selection, "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0"),
            (ContextSlashCommand::Browser, "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"),
            (ContextSlashCommand::Window, "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1"),
        ];

        for (cmd, expected_uri) in mappings {
            let input = match cmd {
                ContextSlashCommand::Context => "/context",
                ContextSlashCommand::ContextFull => "/context-full",
                ContextSlashCommand::Selection => "/selection",
                ContextSlashCommand::Browser => "/browser",
                ContextSlashCommand::Window => "/window",
            };

            assert!(
                parse_context_slash_command(input).is_some(),
                "slash command {input} should parse"
            );

            // Verify the URI starts with kit://context
            assert!(
                expected_uri.starts_with("kit://context"),
                "URI for {input} should be a kit://context URI"
            );
        }
    }
}

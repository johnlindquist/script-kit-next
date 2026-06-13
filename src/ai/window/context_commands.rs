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
    /// Synchronise `pending_context_parts` from live inline `@mention` tokens.
    /// This is composer-token bookkeeping only; it does not open any selector UI.
    pub(super) fn sync_inline_mentions(&mut self, cx: &mut Context<Self>) {
        let text = self.input_state.read(cx).value().to_string();

        let plan = crate::ai::context_mentions::build_inline_mention_sync_plan(
            &text,
            &self.pending_context_parts,
            &self.inline_owned_context_tokens,
        );

        for ix in plan.stale_indices.iter().rev().copied() {
            self.remove_context_part(ix, cx);
        }
        for part in &plan.added_parts {
            self.add_context_part(part.clone(), cx);
        }

        self.inline_owned_context_tokens
            .retain(|token| plan.desired_tokens.contains(token));
        self.inline_owned_context_tokens
            .extend(plan.added_tokens.iter().cloned());

        let visible: std::collections::HashSet<usize> =
            crate::ai::context_mentions::visible_context_chip_indices(
                &text,
                &self.pending_context_parts,
            )
            .into_iter()
            .collect();

        if self
            .context_preview_index
            .is_some_and(|ix| !visible.contains(&ix))
        {
            self.context_preview_index = None;
        }

        tracing::info!(
            target: "ai",
            event = "ai_inline_mentions_synced",
            desired_count = plan.desired_parts.len(),
            added_count = plan.added_parts.len(),
            removed_count = plan.stale_indices.len(),
            token_count = self.inline_owned_context_tokens.len(),
        );

        cx.notify();
    }

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

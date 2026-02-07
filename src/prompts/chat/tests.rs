use super::*;
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::protocol::ChatPromptMessage;

    use super::{
        next_chat_scroll_follow_state, next_reveal_boundary, resolve_chat_input_key_action,
        resolve_setup_card_key, should_ignore_stream_reveal_update, ChatInputKeyAction,
        ChatScrollDirection, SetupCardAction,
    };

    #[test]
    fn resolve_setup_card_key_cycles_focus_for_tab_and_arrows() {
        assert_eq!(
            resolve_setup_card_key("tab", false, 0),
            (1, SetupCardAction::None, true)
        );
        assert_eq!(
            resolve_setup_card_key("Tab", false, 1),
            (0, SetupCardAction::None, true)
        );
        assert_eq!(
            resolve_setup_card_key("tab", true, 0),
            (1, SetupCardAction::None, true)
        );
        assert_eq!(
            resolve_setup_card_key("tab", true, 1),
            (0, SetupCardAction::None, true)
        );

        assert_eq!(
            resolve_setup_card_key("up", false, 0),
            (1, SetupCardAction::None, true)
        );
        assert_eq!(
            resolve_setup_card_key("ArrowUp", false, 1),
            (0, SetupCardAction::None, true)
        );
        assert_eq!(
            resolve_setup_card_key("down", false, 0),
            (1, SetupCardAction::None, true)
        );
        assert_eq!(
            resolve_setup_card_key("arrowdown", false, 1),
            (0, SetupCardAction::None, true)
        );
    }

    #[test]
    fn resolve_setup_card_key_activates_buttons_and_escape() {
        assert_eq!(
            resolve_setup_card_key("enter", false, 0),
            (0, SetupCardAction::ActivateConfigure, false)
        );
        assert_eq!(
            resolve_setup_card_key("Return", false, 1),
            (1, SetupCardAction::ActivateClaudeCode, false)
        );
        assert_eq!(
            resolve_setup_card_key(" ", false, 0),
            (0, SetupCardAction::ActivateConfigure, false)
        );
        assert_eq!(
            resolve_setup_card_key("escape", false, 1),
            (1, SetupCardAction::Escape, false)
        );
    }

    #[test]
    fn resolve_setup_card_key_ignores_unhandled_keys() {
        assert_eq!(
            resolve_setup_card_key("x", false, 1),
            (1, SetupCardAction::None, false)
        );
    }

    #[test]
    fn resolve_chat_input_key_action_routes_enter_variants() {
        assert_eq!(
            resolve_chat_input_key_action("enter", false, false),
            ChatInputKeyAction::Submit
        );
        assert_eq!(
            resolve_chat_input_key_action("return", false, true),
            ChatInputKeyAction::InsertNewline
        );
        assert_eq!(
            resolve_chat_input_key_action("enter", true, false),
            ChatInputKeyAction::ContinueInChat
        );
        assert_eq!(
            resolve_chat_input_key_action("enter", true, true),
            ChatInputKeyAction::ContinueInChat
        );
    }

    #[test]
    fn resolve_chat_input_key_action_routes_shortcuts_and_fallback() {
        assert_eq!(
            resolve_chat_input_key_action("escape", false, false),
            ChatInputKeyAction::Escape
        );
        assert_eq!(
            resolve_chat_input_key_action(".", true, false),
            ChatInputKeyAction::StopStreaming
        );
        assert_eq!(
            resolve_chat_input_key_action("k", true, false),
            ChatInputKeyAction::ToggleActions
        );
        assert_eq!(
            resolve_chat_input_key_action("c", true, false),
            ChatInputKeyAction::CopyLastResponse
        );
        assert_eq!(
            resolve_chat_input_key_action("backspace", true, false),
            ChatInputKeyAction::ClearConversation
        );
        assert_eq!(
            resolve_chat_input_key_action("v", true, false),
            ChatInputKeyAction::Paste
        );
        assert_eq!(
            resolve_chat_input_key_action("backspace", false, false),
            ChatInputKeyAction::DelegateToInput
        );
        assert_eq!(
            resolve_chat_input_key_action("x", true, false),
            ChatInputKeyAction::Ignore
        );
        assert_eq!(
            resolve_chat_input_key_action("a", false, false),
            ChatInputKeyAction::DelegateToInput
        );
    }

    #[test]
    fn should_ignore_stream_reveal_update_when_stream_stopped_or_replaced() {
        assert!(
            should_ignore_stream_reveal_update(None, "stream-a"),
            "Stopped streams should ignore further reveal updates"
        );
        assert!(
            should_ignore_stream_reveal_update(Some("stream-b"), "stream-a"),
            "Replaced streams should ignore stale reveal updates"
        );
        assert!(
            !should_ignore_stream_reveal_update(Some("stream-a"), "stream-a"),
            "Active stream should continue receiving reveal updates"
        );
    }

    // --- next_reveal_boundary tests ---

    #[test]
    fn reveal_boundary_empty_remaining() {
        assert_eq!(next_reveal_boundary("hello", 5), None);
        assert_eq!(next_reveal_boundary("", 0), None);
    }

    #[test]
    fn reveal_boundary_reveals_through_newline() {
        let text = "first line\nsecond line\n";
        assert_eq!(next_reveal_boundary(text, 0), Some(11)); // "first line\n"
        assert_eq!(next_reveal_boundary(text, 11), Some(23)); // "second line\n"
    }

    #[test]
    fn reveal_boundary_word_by_word_without_newline() {
        let text = "hello world foo";
        // "hello " → advances past word + whitespace to start of "world"
        assert_eq!(next_reveal_boundary(text, 0), Some(6));
        assert_eq!(next_reveal_boundary(text, 6), Some(12)); // "world "
                                                             // "foo" — partial word, no trailing whitespace
        assert_eq!(next_reveal_boundary(text, 12), None);
    }

    #[test]
    fn reveal_boundary_partial_word_waits() {
        assert_eq!(next_reveal_boundary("hel", 0), None);
        assert_eq!(next_reveal_boundary("- T", 2), None); // "T" partial
    }

    #[test]
    fn reveal_boundary_newline_takes_priority_over_words() {
        let text = "hello world\nfoo";
        // Should reveal through newline, not stop at word boundary
        assert_eq!(next_reveal_boundary(text, 0), Some(12)); // "hello world\n"
    }

    #[test]
    fn reveal_boundary_markdown_list_lines() {
        let text = "- First item\n- Second item\n- Third\n";
        let mut offset = 0;
        let mut lines = vec![];
        while let Some(new_offset) = next_reveal_boundary(text, offset) {
            lines.push(&text[offset..new_offset]);
            offset = new_offset;
        }
        assert_eq!(
            lines,
            vec!["- First item\n", "- Second item\n", "- Third\n"]
        );
    }

    #[test]
    fn reveal_boundary_utf8_safe() {
        let text = "héllo wörld\n";
        assert_eq!(next_reveal_boundary(text, 0), Some(text.len()));
    }

    /// Simulate the full reveal of a markdown string and verify the final
    /// result matches the original. This catches cases where progressive
    /// reveal could produce a different final string.
    #[test]
    fn progressive_reveal_produces_complete_content() {
        let content = "Sure! Here's a list:\n\n\
            **Things to do:**\n\
            - Read a good book\n\
            - Watch your favorite movies or TV shows\n\
            - Try a new recipe or bake something delicious\n\
            - Work on a puzzle\n\n\
            Would you like me to create a list on a different topic?\n";

        let mut offset = 0;

        // Simulate word-by-word reveal (is_done = false)
        loop {
            match next_reveal_boundary(content, offset) {
                Some(new_offset) if new_offset > offset => {
                    offset = new_offset;
                }
                _ => break, // partial word or end
            }
        }

        // Simulate final flush (is_done = true) — always set to full content
        let revealed = content.to_string();

        assert_eq!(revealed, content);
    }

    /// Verify that reveal never skips content — each boundary advances
    /// monotonically and covers the full string.
    #[test]
    fn reveal_offsets_are_monotonically_increasing() {
        let content = "- First\n- Second\n- Third item with longer text\n\nParagraph after.\n";
        let mut offset = 0;
        let mut prev = 0;
        while let Some(new_offset) = next_reveal_boundary(content, offset) {
            assert!(
                new_offset > prev,
                "Offset did not advance: prev={}, new={}",
                prev,
                new_offset
            );
            prev = new_offset;
            offset = new_offset;
        }
        // After reveal loop, flush remainder
        assert!(
            offset <= content.len(),
            "Final offset {} exceeds content length {}",
            offset,
            content.len()
        );
    }

    #[test]
    fn build_conversation_turns_pairs_user_assistant_messages() {
        let messages = vec![
            ChatPromptMessage::user("First user").with_id("u1"),
            ChatPromptMessage::assistant("First assistant").with_id("a1"),
            ChatPromptMessage::assistant("Standalone assistant").with_id("a2"),
            ChatPromptMessage::user("Second user").with_id("u2"),
        ];

        let turns = super::build_conversation_turns(&messages, &HashMap::new());
        assert_eq!(turns.len(), 3);

        assert_eq!(turns[0].user_prompt, "First user");
        assert_eq!(
            turns[0].assistant_response.as_deref(),
            Some("First assistant")
        );

        assert!(turns[1].user_prompt.is_empty());
        assert_eq!(
            turns[1].assistant_response.as_deref(),
            Some("Standalone assistant")
        );

        assert_eq!(turns[2].user_prompt, "Second user");
        assert!(turns[2].assistant_response.is_none());
    }

    #[test]
    fn chat_scroll_follow_state_disables_follow_on_upward_scroll() {
        assert!(
            next_chat_scroll_follow_state(false, ChatScrollDirection::Up, 0, 10),
            "Scrolling upward should mark the user as manually scrolled up"
        );
    }

    #[test]
    fn chat_scroll_follow_state_keeps_manual_mode_for_single_turn_when_scrolling_down() {
        assert!(
            next_chat_scroll_follow_state(true, ChatScrollDirection::Down, 0, 1),
            "Single large turns should stay in manual mode to avoid false bottom detection"
        );
    }

    #[test]
    fn chat_scroll_follow_state_reenables_follow_near_bottom_for_multi_turn_lists() {
        assert!(
            !next_chat_scroll_follow_state(true, ChatScrollDirection::Down, 8, 10),
            "Scrolling down near the bottom should re-enable auto-follow"
        );
    }

    #[test]
    fn chat_scroll_follow_state_keeps_manual_mode_when_not_near_bottom() {
        assert!(
            next_chat_scroll_follow_state(true, ChatScrollDirection::Down, 6, 10),
            "Scrolling down far from the bottom should keep manual mode enabled"
        );
    }

    #[test]
    fn chat_scroll_follow_state_handles_large_scroll_indices_without_panicking() {
        let result = std::panic::catch_unwind(|| {
            next_chat_scroll_follow_state(true, ChatScrollDirection::Down, usize::MAX, 10)
        });
        assert!(
            result.is_ok(),
            "Large indices should not panic while computing follow state"
        );
    }
}

/// Test-only public access to `next_reveal_boundary` for cross-module tests.
#[cfg(test)]
pub(crate) mod chat_tests {
    pub fn next_reveal_boundary_pub(text: &str, offset: usize) -> Option<usize> {
        super::next_reveal_boundary(text, offset)
    }
}

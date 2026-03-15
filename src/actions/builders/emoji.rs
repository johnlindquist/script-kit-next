use super::types::{Action, ActionCategory};
use crate::designs::icon_variations::IconName;

/// Information about an emoji for action building
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EmojiActionInfo {
    /// The emoji character (e.g., "😀")
    pub value: String,
    /// Human-readable name (e.g., "grinning face")
    pub name: String,
    /// Whether the emoji is pinned for quick access
    pub pinned: bool,
    /// Name of the frontmost app (for "Paste to [AppName]" action title)
    pub frontmost_app_name: Option<String>,
    /// Category of the emoji (for "Copy All Emojis from Section")
    pub category: Option<crate::emoji::EmojiCategory>,
}

/// Get actions specific to an emoji picker entry.
#[allow(clippy::vec_init_then_push, dead_code)]
pub fn get_emoji_context_actions(emoji: &EmojiActionInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    tracing::debug!(
        target: "script_kit::actions",
        emoji = %emoji.value,
        name = %emoji.name,
        pinned = emoji.pinned,
        "Building emoji picker actions"
    );

    let paste_title = match &emoji.frontmost_app_name {
        Some(name) => format!("Paste to {}", name),
        None => "Paste to Active App".to_string(),
    };

    actions.push(
        Action::new(
            "emoji:emoji_paste",
            paste_title,
            Some("Copies the emoji to the clipboard and pastes it into the focused app".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::ArrowUp),
    );

    actions.push(
        Action::new(
            "emoji:emoji_copy",
            "Copy to Clipboard",
            Some("Copies the emoji without pasting".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵")
        .with_icon(IconName::Copy),
    );

    actions.push(
        Action::new(
            "emoji:emoji_paste_keep_open",
            "Paste and Keep Open",
            Some("Pastes the emoji and leaves the picker open".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥↵")
        .with_icon(IconName::ArrowUp),
    );

    if emoji.pinned {
        actions.push(
            Action::new(
                "emoji:emoji_unpin",
                "Unpin Emoji",
                Some("Removes pin from this emoji".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P")
            .with_icon(IconName::StarFilled),
        );
    } else {
        actions.push(
            Action::new(
                "emoji:emoji_pin",
                "Pin Emoji",
                Some("Pins this emoji for quick access".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P")
            .with_icon(IconName::Star),
        );
    }

    actions.push(
        Action::new(
            "emoji:emoji_copy_unicode",
            "Copy Unicode",
            Some("Copies the Unicode codepoint string (e.g. U+1F600)".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⌘C")
        .with_icon(IconName::Code),
    );

    if emoji.category.is_some() {
        actions.push(
            Action::new(
                "emoji:emoji_copy_section",
                "Copy All Emojis from Section",
                Some("Copies all emojis in this emoji's category".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Copy),
        );
    }

    tracing::debug!(
        target: "script_kit::actions",
        action_count = actions.len(),
        "Created emoji picker actions"
    );

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emoji_info(pinned: bool, frontmost_app: Option<&str>) -> EmojiActionInfo {
        EmojiActionInfo {
            value: "😀".to_string(),
            name: "grinning face".to_string(),
            pinned,
            frontmost_app_name: frontmost_app.map(String::from),
            category: Some(crate::emoji::EmojiCategory::SmileysEmotion),
        }
    }

    #[test]
    fn test_get_emoji_context_actions_includes_expected_defaults() {
        let actions = get_emoji_context_actions(&emoji_info(false, None));
        let ids: Vec<_> = actions.iter().map(|a| a.id.as_str()).collect();

        assert!(ids.contains(&"emoji:emoji_paste"));
        assert!(ids.contains(&"emoji:emoji_copy"));
        assert!(ids.contains(&"emoji:emoji_paste_keep_open"));
        assert!(ids.contains(&"emoji:emoji_pin"));
        assert!(ids.contains(&"emoji:emoji_copy_unicode"));
        assert!(ids.contains(&"emoji:emoji_copy_section"));
    }

    #[test]
    fn test_get_emoji_context_actions_prefixes_ids_with_emoji_namespace() {
        let actions = get_emoji_context_actions(&emoji_info(false, None));
        assert!(actions.iter().all(|a| a.id.starts_with("emoji:")));
    }

    #[test]
    fn test_get_emoji_context_actions_flips_pin_action_for_pinned_emoji() {
        let actions = get_emoji_context_actions(&emoji_info(true, None));
        let ids: Vec<_> = actions.iter().map(|a| a.id.as_str()).collect();

        assert!(ids.contains(&"emoji:emoji_unpin"));
        assert!(!ids.contains(&"emoji:emoji_pin"));

        let unpin = actions.iter().find(|a| a.id == "emoji:emoji_unpin").unwrap();
        assert_eq!(unpin.title, "Unpin Emoji");
        assert_eq!(unpin.icon, Some(IconName::StarFilled));
    }

    #[test]
    fn test_get_emoji_context_actions_unpinned_shows_pin_action() {
        let actions = get_emoji_context_actions(&emoji_info(false, None));
        let ids: Vec<_> = actions.iter().map(|a| a.id.as_str()).collect();

        assert!(ids.contains(&"emoji:emoji_pin"));
        assert!(!ids.contains(&"emoji:emoji_unpin"));

        let pin = actions.iter().find(|a| a.id == "emoji:emoji_pin").unwrap();
        assert_eq!(pin.title, "Pin Emoji");
        assert_eq!(pin.icon, Some(IconName::Star));
    }

    #[test]
    fn test_get_emoji_context_actions_paste_title_includes_app_name() {
        let actions = get_emoji_context_actions(&emoji_info(false, Some("TextEdit")));
        let paste = actions.iter().find(|a| a.id == "emoji:emoji_paste").unwrap();
        assert_eq!(paste.title, "Paste to TextEdit");
    }

    #[test]
    fn test_get_emoji_context_actions_paste_title_fallback_without_app_name() {
        let actions = get_emoji_context_actions(&emoji_info(false, None));
        let paste = actions.iter().find(|a| a.id == "emoji:emoji_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn test_get_emoji_context_actions_assigns_consistent_icons() {
        let actions = get_emoji_context_actions(&emoji_info(false, None));

        let paste = actions.iter().find(|a| a.id == "emoji:emoji_paste").unwrap();
        let copy = actions.iter().find(|a| a.id == "emoji:emoji_copy").unwrap();
        let paste_keep = actions.iter().find(|a| a.id == "emoji:emoji_paste_keep_open").unwrap();

        assert_eq!(paste.icon, Some(IconName::ArrowUp));
        assert_eq!(copy.icon, Some(IconName::Copy));
        assert_eq!(paste_keep.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn test_get_emoji_context_actions_copy_unicode_has_correct_shortcut() {
        let actions = get_emoji_context_actions(&emoji_info(false, None));
        let copy_unicode = actions
            .iter()
            .find(|a| a.id == "emoji:emoji_copy_unicode")
            .expect("expected copy unicode action");

        assert_eq!(copy_unicode.title, "Copy Unicode");
        assert_eq!(copy_unicode.shortcut.as_deref(), Some("⌥⌘C"));
        assert_eq!(copy_unicode.icon, Some(IconName::Code));
    }

    #[test]
    fn test_get_emoji_context_actions_copy_section_present_with_category() {
        let actions = get_emoji_context_actions(&emoji_info(false, None));
        let copy_section = actions
            .iter()
            .find(|a| a.id == "emoji:emoji_copy_section")
            .expect("expected copy section action");

        assert_eq!(copy_section.title, "Copy All Emojis from Section");
        assert_eq!(copy_section.icon, Some(IconName::Copy));
    }

    #[test]
    fn test_get_emoji_context_actions_copy_section_hidden_without_category() {
        let mut info = emoji_info(false, None);
        info.category = None;
        let actions = get_emoji_context_actions(&info);
        let ids: Vec<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(!ids.contains(&"emoji:emoji_copy_section"));
    }
}

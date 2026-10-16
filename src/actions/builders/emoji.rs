use super::types::{Action, ActionCategory};
use crate::designs::icon_variations::IconName;
use crate::emoji::SkinTone;

/// The emoji the actions act on, as the picker shows it.
#[derive(Debug, Clone)]
pub struct EmojiActionInfo {
    /// The emoji as it would be inserted, with the skin tone applied.
    pub value: String,
    /// The emoji without a skin tone (what pins and usage are keyed on).
    pub base: String,
    /// Human-readable name (e.g., "grinning face")
    pub name: String,
    /// Whether the emoji is pinned for quick access
    pub pinned: bool,
    /// Position among the pins and how many pins there are, when pinned.
    pub pin_position: Option<(usize, usize)>,
    /// Whether it appears under Frequently Used.
    pub frequent: bool,
    /// Name of the frontmost app (for "Paste to [AppName]" action title)
    pub frontmost_app_name: Option<String>,
    /// Category of the emoji (for "Copy All Emojis from Section")
    pub category: Option<crate::emoji::EmojiCategory>,
    /// Light-to-dark variants when the emoji takes a skin tone.
    pub tones: Option<[&'static str; 5]>,
    /// The picker's current default skin tone.
    pub skin_tone: SkinTone,
}

/// A parsed `emoji:*` action id.
#[allow(dead_code)] // Matched by the binary's action handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiPickerAction {
    Paste { keep_open: bool },
    Copy,
    CopyName,
    CopyUnicode,
    CopySection,
    Pin(crate::emoji_pins::PinEdit),
    Forget,
    Tone(SkinTone),
}

impl EmojiPickerAction {
    #[allow(dead_code)] // Called by the binary's action handler.
    pub fn from_action_id(action_id: &str) -> Option<Self> {
        use crate::emoji_pins::PinEdit;
        let id = action_id.strip_prefix("emoji:").unwrap_or(action_id);
        Some(match id {
            "emoji_paste" => Self::Paste { keep_open: false },
            "emoji_paste_keep_open" => Self::Paste { keep_open: true },
            "emoji_copy" => Self::Copy,
            "emoji_copy_name" => Self::CopyName,
            "emoji_copy_unicode" => Self::CopyUnicode,
            "emoji_copy_section" => Self::CopySection,
            "emoji_pin" => Self::Pin(PinEdit::Pin),
            "emoji_unpin" => Self::Pin(PinEdit::Unpin),
            "emoji_pin_move_up" => Self::Pin(PinEdit::MoveUp),
            "emoji_pin_move_down" => Self::Pin(PinEdit::MoveDown),
            "emoji_forget" => Self::Forget,
            _ => Self::Tone(SkinTone::from_id(id.strip_prefix("emoji_tone_")?)?),
        })
    }
}

/// `U+1F44B U+1F3FD` for the emoji's code points.
#[allow(dead_code)] // Called by the binary's action handler.
pub fn emoji_unicode_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| format!("U+{:04X}", ch as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Action id that sets the default skin tone.
pub fn emoji_tone_action_id(tone: SkinTone) -> String {
    format!("emoji:emoji_tone_{}", tone.id())
}

fn action(id: &str, title: impl Into<String>, description: &str) -> Action {
    Action::new(
        id,
        title.into(),
        Some(description.to_string()),
        ActionCategory::ScriptContext,
    )
}

/// Get actions specific to an emoji picker entry.
pub fn get_emoji_context_actions(emoji: &EmojiActionInfo) -> Vec<Action> {
    let paste_title = match emoji.frontmost_app_name.as_deref() {
        Some(name) if !name.trim().is_empty() => format!("Paste to {name}"),
        _ => "Paste to Active App".to_string(),
    };
    let mut actions = vec![
        action(
            "emoji:emoji_paste",
            paste_title,
            "Copies the emoji and pastes it into the focused app",
        )
        .with_shortcut("\u{21b5}")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
        action(
            "emoji:emoji_copy",
            "Copy to Clipboard",
            "Copies the emoji without pasting",
        )
        .with_shortcut("\u{2318}\u{21b5}")
        .with_icon(IconName::Copy)
        .with_section("Actions"),
        action(
            "emoji:emoji_paste_keep_open",
            "Paste and Keep Open",
            "Pastes the emoji and leaves the picker open",
        )
        .with_shortcut("\u{2325}\u{21b5}")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
    ];

    if emoji.pinned {
        actions.push(
            action("emoji:emoji_unpin", "Unpin Emoji", "Removes it from Pinned")
                .with_shortcut("\u{21e7}\u{2318}P")
                .with_icon(IconName::StarFilled)
                .with_section("Edit"),
        );
        if let Some((position, count)) = emoji.pin_position {
            if position > 0 {
                actions.push(
                    action(
                        "emoji:emoji_pin_move_up",
                        "Move Up in Pinned",
                        "Moves it one place earlier in Pinned",
                    )
                    .with_shortcut("\u{2325}\u{2318}\u{2191}")
                    .with_icon(IconName::ArrowUp)
                    .with_section("Edit"),
                );
            }
            if position + 1 < count {
                actions.push(
                    action(
                        "emoji:emoji_pin_move_down",
                        "Move Down in Pinned",
                        "Moves it one place later in Pinned",
                    )
                    .with_shortcut("\u{2325}\u{2318}\u{2193}")
                    .with_icon(IconName::ArrowDown)
                    .with_section("Edit"),
                );
            }
        }
    } else {
        actions.push(
            action(
                "emoji:emoji_pin",
                "Pin Emoji",
                "Keeps it at the top of the picker",
            )
            .with_shortcut("\u{21e7}\u{2318}P")
            .with_icon(IconName::Star)
            .with_section("Edit"),
        );
    }
    if emoji.frequent {
        actions.push(
            action(
                "emoji:emoji_forget",
                "Remove from Frequently Used",
                "Resets how often you've used it",
            )
            .with_icon(IconName::Refresh)
            .with_section("Edit"),
        );
    }

    actions.push(
        action(
            "emoji:emoji_copy_name",
            "Copy Name",
            "Copies the emoji's name",
        )
        .with_shortcut("\u{21e7}\u{2318}C")
        .with_icon(IconName::Copy)
        .with_section("Share"),
    );
    actions.push(
        action(
            "emoji:emoji_copy_unicode",
            "Copy Unicode",
            "Copies the code points, such as U+1F600",
        )
        .with_shortcut("\u{2325}\u{2318}C")
        .with_icon(IconName::Code)
        .with_section("Share"),
    );
    if emoji.category.is_some() {
        actions.push(
            action(
                "emoji:emoji_copy_section",
                "Copy All Emojis from Section",
                "Copies every emoji in this emoji's category",
            )
            .with_icon(IconName::Copy)
            .with_section("Share"),
        );
    }

    if let Some(tones) = emoji.tones {
        use strum::IntoEnumIterator as _;
        for tone in SkinTone::iter() {
            let glyph = match tone {
                SkinTone::None => emoji.base.as_str(),
                _ => tones[tone as usize - 1],
            };
            let current = if tone == emoji.skin_tone {
                " (current)"
            } else {
                ""
            };
            actions.push(
                action(
                    &emoji_tone_action_id(tone),
                    format!("{glyph}  {}{current}", tone.title()),
                    "Uses this skin tone for every emoji that takes one",
                )
                .with_section("Skin Tone"),
            );
        }
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emoji_info(pinned: bool, frontmost_app: Option<&str>) -> EmojiActionInfo {
        EmojiActionInfo {
            value: "\u{1F600}".to_string(),
            base: "\u{1F600}".to_string(),
            name: "grinning face".to_string(),
            pinned,
            pin_position: pinned.then_some((0, 1)),
            frequent: false,
            frontmost_app_name: frontmost_app.map(String::from),
            category: Some(crate::emoji::EmojiCategory::SmileysEmotion),
            tones: None,
            skin_tone: SkinTone::None,
        }
    }

    fn ids(info: &EmojiActionInfo) -> Vec<String> {
        get_emoji_context_actions(info)
            .into_iter()
            .map(|a| a.id)
            .collect()
    }

    #[test]
    fn emoji_actions_include_the_defaults() {
        let ids = ids(&emoji_info(false, None));
        for id in [
            "emoji:emoji_paste",
            "emoji:emoji_copy",
            "emoji:emoji_paste_keep_open",
            "emoji:emoji_pin",
            "emoji:emoji_copy_name",
            "emoji:emoji_copy_unicode",
            "emoji:emoji_copy_section",
        ] {
            assert!(ids.iter().any(|known| known == id), "{id}");
        }
        assert!(ids.iter().all(|id| id.starts_with("emoji:")));
        assert!(!ids.iter().any(|id| id.starts_with("emoji:emoji_tone_")));
        assert!(!ids.iter().any(|id| id == "emoji:emoji_forget"));
    }

    #[test]
    fn pinned_emoji_can_be_unpinned_and_moved() {
        let mut info = emoji_info(true, None);
        info.pin_position = Some((1, 3));
        let ids = ids(&info);
        assert!(ids.iter().any(|id| id == "emoji:emoji_unpin"));
        assert!(!ids.iter().any(|id| id == "emoji:emoji_pin"));
        assert!(ids.iter().any(|id| id == "emoji:emoji_pin_move_up"));
        assert!(ids.iter().any(|id| id == "emoji:emoji_pin_move_down"));
        info.pin_position = Some((0, 1));
        let only = super::get_emoji_context_actions(&info);
        assert!(!only.iter().any(|a| a.id.contains("move")));
    }

    #[test]
    fn paste_title_names_the_frontmost_app() {
        let actions = get_emoji_context_actions(&emoji_info(false, Some("TextEdit")));
        assert_eq!(actions[0].title, "Paste to TextEdit");
        let actions = get_emoji_context_actions(&emoji_info(false, Some("   ")));
        assert_eq!(actions[0].title, "Paste to Active App");
    }

    #[test]
    fn emoji_that_take_a_skin_tone_offer_all_six_and_mark_the_current_one() {
        let wave = crate::emoji::emoji_by_value("\u{1F44B}").unwrap();
        let mut info = emoji_info(false, None);
        info.base = wave.emoji.to_string();
        info.tones = wave.tones;
        info.skin_tone = SkinTone::Medium;
        info.frequent = true;
        let actions = get_emoji_context_actions(&info);
        let tones: Vec<&Action> = actions
            .iter()
            .filter(|a| a.id.starts_with("emoji:emoji_tone_"))
            .collect();
        assert_eq!(tones.len(), 6);
        assert!(tones[0].title.starts_with("\u{1F44B} "));
        assert!(tones[3].title.starts_with("\u{1F44B}\u{1F3FD}"));
        assert!(tones[3].title.ends_with("(current)"));
        assert_eq!(
            tones
                .iter()
                .filter(|a| a.title.ends_with("(current)"))
                .count(),
            1
        );
        assert!(actions.iter().any(|a| a.id == "emoji:emoji_forget"));
    }

    #[test]
    fn emoji_action_ids_parse_with_or_without_the_namespace() {
        use crate::emoji_pins::PinEdit;
        assert_eq!(
            EmojiPickerAction::from_action_id("emoji:emoji_paste_keep_open"),
            Some(EmojiPickerAction::Paste { keep_open: true })
        );
        assert_eq!(
            EmojiPickerAction::from_action_id("emoji_pin_move_down"),
            Some(EmojiPickerAction::Pin(PinEdit::MoveDown))
        );
        assert_eq!(
            EmojiPickerAction::from_action_id(&emoji_tone_action_id(SkinTone::MediumDark)),
            Some(EmojiPickerAction::Tone(SkinTone::MediumDark))
        );
        assert_eq!(EmojiPickerAction::from_action_id("emoji_tone_purple"), None);
        assert_eq!(EmojiPickerAction::from_action_id("clipboard_copy"), None);
        assert_eq!(emoji_unicode_text("\u{1F44B}\u{1F3FD}"), "U+1F44B U+1F3FD");
        // Every row the builder offers parses.
        let wave = crate::emoji::emoji_by_value("\u{1F44B}").unwrap();
        let mut info = emoji_info(true, None);
        info.pin_position = Some((1, 3));
        info.tones = wave.tones;
        info.frequent = true;
        for action in get_emoji_context_actions(&info) {
            assert!(
                EmojiPickerAction::from_action_id(&action.id).is_some(),
                "{}",
                action.id
            );
        }
    }
}

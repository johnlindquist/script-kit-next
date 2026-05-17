// Emoji action handlers for handle_action dispatch.
//
// Contains all `emoji_*` action handling: paste, copy, paste-keep-open, pin, unpin.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmojiPinHandlerAction {
    Pin,
    Unpin,
}

impl EmojiPinHandlerAction {
    fn from_action_id(action_id: &str) -> Option<Self> {
        match action_id {
            "emoji_pin" => Some(Self::Pin),
            "emoji_unpin" => Some(Self::Unpin),
            _ => None,
        }
    }

    fn apply(self, pinned_emojis: &mut std::collections::HashSet<String>, emoji_value: &str) {
        match self {
            Self::Pin => {
                pinned_emojis.insert(emoji_value.to_string());
            }
            Self::Unpin => {
                pinned_emojis.remove(emoji_value);
            }
        }
    }

    fn error_prefix(self) -> &'static str {
        match self {
            Self::Pin => "Failed to pin emoji",
            Self::Unpin => "Failed to unpin emoji",
        }
    }

    fn success_hud(self, emoji_value: &str) -> String {
        match self {
            Self::Pin => format!("Pinned {emoji_value}"),
            Self::Unpin => format!("Unpinned {emoji_value}"),
        }
    }
}

impl ScriptListApp {
    /// Build an EmojiActionInfo for the currently selected emoji in the picker view.
    fn selected_emoji_action_info(&self) -> Option<crate::actions::EmojiActionInfo> {
        let (filter, selected_index, selected_category) = match &self.current_view {
            AppView::EmojiPickerView {
                filter,
                selected_index,
                selected_category,
            } => (filter.as_str(), *selected_index, *selected_category),
            _ => return None,
        };

        let (ordered_emojis, _pin_count) =
            crate::emoji::filtered_ordered_emojis_with_pins(filter, selected_category, &self.pinned_emojis);
        let emoji = ordered_emojis.get(selected_index)?;

        let frontmost_app_name =
            crate::frontmost_app_tracker::get_last_real_app().map(|app| app.name);

        Some(crate::actions::EmojiActionInfo {
            value: emoji.emoji.to_string(),
            name: emoji.name.to_string(),
            pinned: self.pinned_emojis.contains(emoji.emoji),
            frontmost_app_name,
            category: Some(emoji.category),
        })
    }

    /// Handle emoji-specific actions. Returns a DispatchOutcome.
    fn handle_emoji_action(
        &mut self,
        action_id: &str,
        selected_emoji: Option<crate::actions::EmojiActionInfo>,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        match action_id {
            "emoji_paste" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(
                    action = "emoji_paste",
                    emoji = %emoji.value,
                    emoji_name = %emoji.name,
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.value.clone()));
                self.finalize_paste_after_clipboard_ready(
                    "emoji",
                    &emoji.name,
                    PasteCloseBehavior::HideWindow,
                    cx,
                )
            }
            "emoji_copy" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(action = "emoji_copy", emoji = %emoji.value, "emoji action");
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.value.clone()));
                self.show_hud(format!("Copied {}", emoji.value), Some(HUD_SHORT_MS), cx);
                DispatchOutcome::success()
            }
            "emoji_paste_keep_open" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(
                    action = "emoji_paste_keep_open",
                    emoji = %emoji.value,
                    emoji_name = %emoji.name,
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.value.clone()));
                self.finalize_paste_after_clipboard_ready(
                    "emoji",
                    &emoji.name,
                    PasteCloseBehavior::KeepWindowOpen,
                    cx,
                )
            }
            "emoji_pin" | "emoji_unpin" => {
                let Some(pin_action) = EmojiPinHandlerAction::from_action_id(action_id) else {
                    return DispatchOutcome::not_handled();
                };
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(action = %action_id, emoji = %emoji.value, "emoji action");
                pin_action.apply(&mut self.pinned_emojis, &emoji.value);
                if let Err(error) = crate::emoji_pins::save_pinned_emojis(&self.pinned_emojis) {
                    tracing::error!(error = %error, emoji = %emoji.value, action = %action_id, "failed to update pinned emoji");
                    self.show_error_toast(format!("{}: {}", pin_action.error_prefix(), error), cx);
                } else {
                    self.show_hud(pin_action.success_hud(&emoji.value), Some(HUD_SHORT_MS), cx);
                    cx.notify();
                }
                DispatchOutcome::success()
            }
            "emoji_copy_unicode" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                let unicode_str: String = {
                    use itertools::Itertools as _;
                    emoji
                        .value
                        .chars()
                        .map(|c| format!("U+{:04X}", c as u32))
                        .join(" ")
                };

                tracing::info!(
                    action = "emoji_copy_unicode",
                    emoji = %emoji.value,
                    unicode = %unicode_str,
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(unicode_str.clone()));
                self.show_hud(
                    format!("Copied {}", unicode_str),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                DispatchOutcome::success()
            }
            "emoji_copy_section" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                let Some(category) = emoji.category else {
                    self.show_error_toast("No category for this emoji", cx);
                    return DispatchOutcome::success();
                };

                let category_emojis = crate::emoji::emojis_by_category(category);
                let all_emojis: String = {
                    use itertools::Itertools as _;
                    category_emojis.iter().map(|e| e.emoji).join("")
                };

                tracing::info!(
                    action = "emoji_copy_section",
                    category = %category,
                    count = category_emojis.len(),
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(all_emojis));
                self.show_hud(
                    format!(
                        "Copied {} emojis from {}",
                        category_emojis.len(),
                        category
                    ),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}

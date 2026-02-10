use super::types::{Action, ActionCategory};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;

/// Information about a clipboard history entry for action building
#[derive(Debug, Clone)]
pub struct ClipboardEntryInfo {
    /// Entry ID in the database
    pub id: String,
    /// Content type (text or image)
    pub content_type: ContentType,
    /// Whether the entry is pinned
    pub pinned: bool,
    /// Preview text (for text entries)
    pub preview: String,
    /// Image dimensions (for image entries)
    #[allow(dead_code)]
    pub image_dimensions: Option<(u32, u32)>,
    /// Name of the frontmost app (for "Paste to [AppName]" action title)
    pub frontmost_app_name: Option<String>,
}

/// Get actions specific to a clipboard history entry.
#[allow(clippy::vec_init_then_push)]
pub fn get_clipboard_history_context_actions(entry: &ClipboardEntryInfo) -> Vec<Action> {
    if entry.id.trim().is_empty() {
        tracing::warn!(
            target: "script_kit::actions",
            content_type = ?entry.content_type,
            pinned = entry.pinned,
            "Invalid clipboard entry context: missing entry id; returning no actions"
        );
        return Vec::new();
    }

    let mut actions = Vec::new();

    tracing::debug!(
        target: "script_kit::actions",
        entry_id = %entry.id,
        content_type = ?entry.content_type,
        pinned = entry.pinned,
        "Building clipboard history actions"
    );

    let paste_title = match &entry.frontmost_app_name {
        Some(name) => format!("Paste to {}", name),
        None => "Paste to Active App".to_string(),
    };
    actions.push(
        Action::new(
            "clip:clipboard_paste",
            paste_title,
            Some("Copies to clipboard and pastes to the focused app".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::ArrowUp),
    );

    actions.push(
        Action::new(
            "clip:clipboard_copy",
            "Copy to Clipboard",
            Some("Copies the entry to clipboard without pasting".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵")
        .with_icon(IconName::Copy),
    );

    actions.push(
        Action::new(
            "clip:clipboard_paste_keep_open",
            "Paste and Keep Window Open",
            Some("Pastes the entry and keeps clipboard history open".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥↵")
        .with_icon(IconName::ArrowUp),
    );

    actions.push(
        Action::new(
            "clip:clipboard_share",
            "Share...",
            Some("Shares this entry via the system share sheet".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘E")
        .with_icon(IconName::ArrowRight),
    );

    actions.push(
        Action::new(
            "clip:clipboard_attach_to_ai",
            "Attach to AI Chat",
            Some("Sends this entry to the AI chat window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⌘A")
        .with_icon(IconName::MessageCircle),
    );

    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "clip:clipboard_quick_look",
            "Quick Look",
            Some("Previews this entry with Quick Look".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("␣")
        .with_icon(IconName::File),
    );

    if entry.content_type == ContentType::Image {
        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clip:clipboard_open_with",
                "Open With...",
                Some("Opens the image with a selected app".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘O")
            .with_icon(IconName::File),
        );

        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clip:clipboard_annotate_cleanshot",
                "Annotate in CleanShot X",
                Some("Opens image in CleanShot X for annotation".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘A")
            .with_icon(IconName::Pencil),
        );

        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clip:clipboard_upload_cleanshot",
                "Upload to CleanShot X",
                Some("Uploads image to CleanShot Cloud".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘U")
            .with_icon(IconName::ArrowUp),
        );
    }

    if entry.pinned {
        actions.push(
            Action::new(
                "clip:clipboard_unpin",
                "Unpin Entry",
                Some("Removes the pin from this entry".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P")
            .with_icon(IconName::StarFilled),
        );
    } else {
        actions.push(
            Action::new(
                "clip:clipboard_pin",
                "Pin Entry",
                Some("Pins this entry to prevent auto-removal".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P")
            .with_icon(IconName::Star),
        );
    }

    if entry.content_type == ContentType::Image {
        actions.push(
            Action::new(
                "clip:clipboard_ocr",
                "Copy Text from Image",
                Some("Extracts text from the image with OCR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘C")
            .with_icon(IconName::MagnifyingGlass),
        );
    }

    actions.push(
        Action::new(
            "clip:clipboard_save_snippet",
            "Save Text as Snippet",
            Some("Creates a scriptlet from this text".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘S")
        .with_icon(IconName::Code),
    );

    actions.push(
        Action::new(
            "clip:clipboard_save_file",
            "Save as File...",
            Some("Saves this entry to a file".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘S")
        .with_icon(IconName::File),
    );

    actions.push(
        Action::new(
            "clip:clipboard_delete",
            "Delete Entry",
            Some("Removes this entry from clipboard history".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃X")
        .with_icon(IconName::Trash),
    );

    actions.push(
        Action::new(
            "clip:clipboard_delete_multiple",
            "Delete Entries...",
            Some("Deletes entries matching the current search".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘X")
        .with_icon(IconName::Trash),
    );

    actions.push(
        Action::new(
            "clip:clipboard_delete_all",
            "Delete All Entries",
            Some("Clears clipboard history except pinned entries".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⇧X")
        .with_icon(IconName::Trash),
    );

    tracing::debug!(
        target: "script_kit::actions",
        action_count = actions.len(),
        "Created clipboard history actions"
    );

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_info(content_type: ContentType, pinned: bool) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "entry-1".to_string(),
            content_type,
            pinned,
            preview: "example".to_string(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: Some("Editor".to_string()),
        }
    }

    #[test]
    fn test_get_clipboard_history_context_actions_prefixes_ids_with_clip_namespace() {
        let text_actions =
            get_clipboard_history_context_actions(&entry_info(ContentType::Text, false));
        assert!(text_actions
            .iter()
            .all(|action| action.id.starts_with("clip:")));

        let image_actions =
            get_clipboard_history_context_actions(&entry_info(ContentType::Image, true));
        assert!(image_actions
            .iter()
            .all(|action| action.id.starts_with("clip:")));
    }

    #[test]
    fn test_get_clipboard_history_context_actions_returns_empty_when_entry_id_missing() {
        let mut entry = entry_info(ContentType::Text, false);
        entry.id = "   ".to_string();

        let actions = get_clipboard_history_context_actions(&entry);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_clipboard_history_context_actions_assigns_consistent_primary_icons() {
        let actions = get_clipboard_history_context_actions(&entry_info(ContentType::Text, false));

        let paste_action = actions
            .iter()
            .find(|action| action.id == "clip:clipboard_paste")
            .expect("missing clipboard_paste action");
        let copy_action = actions
            .iter()
            .find(|action| action.id == "clip:clipboard_copy")
            .expect("missing clipboard_copy action");
        let save_file_action = actions
            .iter()
            .find(|action| action.id == "clip:clipboard_save_file")
            .expect("missing clipboard_save_file action");
        let delete_action = actions
            .iter()
            .find(|action| action.id == "clip:clipboard_delete")
            .expect("missing clipboard_delete action");

        assert_eq!(paste_action.icon, Some(IconName::ArrowUp));
        assert_eq!(copy_action.icon, Some(IconName::Copy));
        assert_eq!(save_file_action.icon, Some(IconName::File));
        assert_eq!(delete_action.icon, Some(IconName::Trash));
    }
}

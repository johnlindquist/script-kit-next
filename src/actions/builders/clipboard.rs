use super::types::{Action, ActionCategory};
use crate::clipboard_history::ContentType;

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
            "clipboard_paste",
            paste_title,
            Some("Copy to clipboard and paste to focused app".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
    );

    actions.push(
        Action::new(
            "clipboard_copy",
            "Copy to Clipboard",
            Some("Copy entry to clipboard without pasting".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    actions.push(
        Action::new(
            "clipboard_paste_keep_open",
            "Paste and Keep Window Open",
            Some("Paste entry but keep the clipboard history open".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥↵"),
    );

    actions.push(
        Action::new(
            "clipboard_share",
            "Share...",
            Some("Share this entry via system share sheet".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘E"),
    );

    actions.push(
        Action::new(
            "clipboard_attach_to_ai",
            "Attach to AI Chat",
            Some("Send this entry to the AI chat window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⌘A"),
    );

    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "clipboard_quick_look",
            "Quick Look",
            Some("Preview with Quick Look".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("␣"),
    );

    if entry.content_type == ContentType::Image {
        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clipboard_open_with",
                "Open With...",
                Some("Open image with a specific application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘O"),
        );

        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clipboard_annotate_cleanshot",
                "Annotate in CleanShot X",
                Some("Open image in CleanShot X for annotation".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘A"),
        );

        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clipboard_upload_cleanshot",
                "Upload to CleanShot X",
                Some("Upload image to CleanShot Cloud".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘U"),
        );
    }

    if entry.pinned {
        actions.push(
            Action::new(
                "clipboard_unpin",
                "Unpin Entry",
                Some("Remove pin from this entry".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P"),
        );
    } else {
        actions.push(
            Action::new(
                "clipboard_pin",
                "Pin Entry",
                Some("Pin this entry to prevent auto-removal".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P"),
        );
    }

    if entry.content_type == ContentType::Image {
        actions.push(
            Action::new(
                "clipboard_ocr",
                "Copy Text from Image",
                Some("Extract text from image using OCR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘C"),
        );
    }

    actions.push(
        Action::new(
            "clipboard_save_snippet",
            "Save Text as Snippet",
            Some("Create a scriptlet from this text".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘S"),
    );

    actions.push(
        Action::new(
            "clipboard_save_file",
            "Save as File...",
            Some("Save this entry to a file".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘S"),
    );

    actions.push(
        Action::new(
            "clipboard_delete",
            "Delete Entry",
            Some("Remove this entry from clipboard history".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃X"),
    );

    actions.push(
        Action::new(
            "clipboard_delete_multiple",
            "Delete Entries...",
            Some("Delete entries matching current search filter".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘X"),
    );

    actions.push(
        Action::new(
            "clipboard_delete_all",
            "Delete All Entries",
            Some("Clear all clipboard history (except pinned)".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⇧X"),
    );

    tracing::debug!(
        target: "script_kit::actions",
        action_count = actions.len(),
        "Created clipboard history actions"
    );

    actions
}

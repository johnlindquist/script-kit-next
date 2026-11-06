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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardEntryActionPlan {
    TextPinned,
    TextUnpinned,
    ImagePinned,
    ImageUnpinned,
    OtherPinned,
    OtherUnpinned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardPasteDestinationPlan {
    FrontmostApp,
    ActiveAppFallback,
}

impl ClipboardEntryActionPlan {
    fn from_entry(entry: &ClipboardEntryInfo) -> Self {
        match (entry.content_type, entry.pinned) {
            (ContentType::Text, true) => Self::TextPinned,
            (ContentType::Text, false) => Self::TextUnpinned,
            (ContentType::Image, true) => Self::ImagePinned,
            (ContentType::Image, false) => Self::ImageUnpinned,
            (_, true) => Self::OtherPinned,
            (_, false) => Self::OtherUnpinned,
        }
    }

    fn is_image(self) -> bool {
        matches!(self, Self::ImagePinned | Self::ImageUnpinned)
    }

    fn is_text(self) -> bool {
        matches!(self, Self::TextPinned | Self::TextUnpinned)
    }

    fn pin_action(self) -> Action {
        match self {
            Self::TextPinned | Self::ImagePinned | Self::OtherPinned => Action::new(
                "clip:clipboard_unpin",
                "Unpin Entry",
                Some("Removes the pin from this entry".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P")
            .with_icon(IconName::StarFilled)
            .with_section("Edit"),
            Self::TextUnpinned | Self::ImageUnpinned | Self::OtherUnpinned => Action::new(
                "clip:clipboard_pin",
                "Pin Entry",
                Some("Pins this entry to prevent auto-removal".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘P")
            .with_icon(IconName::Star)
            .with_section("Edit"),
        }
    }
}

impl ClipboardPasteDestinationPlan {
    fn from_entry(entry: &ClipboardEntryInfo) -> Self {
        match entry.frontmost_app_name.as_deref() {
            Some(name) if !name.trim().is_empty() => Self::FrontmostApp,
            _ => Self::ActiveAppFallback,
        }
    }

    fn title(self, entry: &ClipboardEntryInfo) -> String {
        match self {
            Self::FrontmostApp => format!(
                "Paste to {}",
                entry.frontmost_app_name.as_deref().unwrap_or_default()
            ),
            Self::ActiveAppFallback => "Paste to Active App".to_string(),
        }
    }
}

/// The time spans "Delete Entries from the Last …" offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecentWindow {
    FiveMinutes,
    FifteenMinutes,
    Hour,
    Day,
}

impl RecentWindow {
    pub const ALL: [Self; 4] = [
        Self::FiveMinutes,
        Self::FifteenMinutes,
        Self::Hour,
        Self::Day,
    ];

    pub fn action_id(self) -> &'static str {
        match self {
            Self::FiveMinutes => "clip:clipboard_delete_recent_5m",
            Self::FifteenMinutes => "clip:clipboard_delete_recent_15m",
            Self::Hour => "clip:clipboard_delete_recent_1h",
            Self::Day => "clip:clipboard_delete_recent_24h",
        }
    }

    #[allow(dead_code)] // Called by the binary's action handler.
    pub fn from_action_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("clip:").unwrap_or(id);
        Self::ALL
            .into_iter()
            .find(|window| window.action_id().strip_prefix("clip:") == Some(id))
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::FiveMinutes => "Last 5 Minutes",
            Self::FifteenMinutes => "Last 15 Minutes",
            Self::Hour => "Last Hour",
            Self::Day => "Last 24 Hours",
        }
    }

    #[allow(dead_code)] // Called by the binary's action handler.
    pub fn millis(self) -> i64 {
        let minutes = match self {
            Self::FiveMinutes => 5,
            Self::FifteenMinutes => 15,
            Self::Hour => 60,
            Self::Day => 24 * 60,
        };
        minutes * 60 * 1000
    }
}

/// Opens the "Delete Recent Entries" choices.
pub const DELETE_RECENT_ACTION_ID: &str = "clip:clipboard_delete_recent";

/// The time-span choices behind "Delete Recent Entries…".
pub fn get_clipboard_delete_recent_route() -> crate::actions::ActionsDialogRoute {
    crate::actions::ActionsDialogRoute {
        id: "clipboard_delete_recent".to_string(),
        actions: RecentWindow::ALL
            .into_iter()
            .map(|window| {
                Action::new(
                    window.action_id(),
                    window.title(),
                    Some(
                        "Deletes unpinned entries copied in this time; pinned ones stay"
                            .to_string(),
                    ),
                    ActionCategory::ScriptContext,
                )
                .with_icon(IconName::Trash)
                .with_section("Delete Entries From")
            })
            .collect(),
        context_title: Some("Delete Recent Entries".to_string()),
        search_placeholder: Some("Delete entries from…".to_string()),
        initial_selected_action_id: None,
    }
}

/// Whether text looks like a JSON object or array worth pretty-printing.
fn looks_like_json(preview: &str) -> bool {
    let trimmed = preview.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
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
    let entry_plan = ClipboardEntryActionPlan::from_entry(entry);
    let paste_destination_plan = ClipboardPasteDestinationPlan::from_entry(entry);

    tracing::debug!(
        target: "script_kit::actions",
        entry_id = %entry.id,
        content_type = ?entry.content_type,
        pinned = entry.pinned,
        "Building clipboard history actions"
    );

    actions.push(
        Action::new(
            "clip:clipboard_paste",
            paste_destination_plan.title(entry),
            Some(
                if entry_plan.is_image() {
                    "Copies the image to the clipboard and pastes to the focused app"
                } else {
                    "Copies plain text to the clipboard, then pastes it without formatting"
                }
                .to_string(),
            ),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
    );

    actions.push(
        Action::new(
            "clip:clipboard_copy",
            "Copy to Clipboard",
            Some("Copies the entry to clipboard without pasting".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵")
        .with_icon(IconName::Copy)
        .with_section("Actions"),
    );

    actions.push(
        Action::new(
            "clip:clipboard_paste_keep_open",
            "Paste and Keep Window Open",
            Some("Pastes the entry and keeps clipboard history open".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥↵")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
    );

    actions.push(
        Action::new(
            "clip:clipboard_share",
            "Share...",
            Some("Shares this entry via the system share sheet".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘E")
        .with_icon(IconName::ArrowRight)
        .with_section("Share"),
    );

    actions.push(
        Action::new(
            "clip:clipboard_attach_to_ai",
            "Attach to Agent Chat",
            Some("Sends this entry to Agent Chat".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⌘A")
        .with_icon(IconName::MessageCircle)
        .with_section("Actions"),
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
        .with_icon(IconName::File)
        .with_section("Actions"),
    );

    match entry.content_type {
        ContentType::Link => actions.push(
            Action::new(
                "clip:clipboard_open_link",
                "Open Link",
                Some("Opens the link in its app or your browser".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘O")
            .with_icon(IconName::ArrowRight)
            .with_section("Actions"),
        ),
        ContentType::File => actions.push(
            Action::new(
                "clip:clipboard_reveal_file",
                "Show in Finder",
                Some("Reveals the copied path in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘F")
            .with_icon(IconName::FolderOpen)
            .with_section("Actions"),
        ),
        ContentType::Color => {
            if let Some(color) = crate::clipboard_history::parse_color(&entry.preview) {
                for (id, format, value) in [
                    ("clip:clipboard_copy_color_hex", "HEX", color.hex()),
                    ("clip:clipboard_copy_color_rgb", "RGB", color.rgb()),
                    ("clip:clipboard_copy_color_hsl", "HSL", color.hsl()),
                ] {
                    actions.push(
                        Action::new(
                            id,
                            format!("Copy as {format}"),
                            Some(value),
                            ActionCategory::ScriptContext,
                        )
                        .with_icon(IconName::Copy)
                        .with_section("Copy Color As"),
                    );
                }
            }
        }
        ContentType::Text if looks_like_json(&entry.preview) => actions.push(
            Action::new(
                "clip:clipboard_copy_json",
                "Copy Formatted JSON",
                Some("Copies the JSON indented, one value per line".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Code)
            .with_section("Actions"),
        ),
        _ => {}
    }

    if entry_plan.is_image() {
        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clip:clipboard_open_with",
                "Open With...",
                Some("Opens the image with a selected app".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘O")
            .with_icon(IconName::File)
            .with_section("Actions"),
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
            .with_icon(IconName::Pencil)
            .with_section("Edit"),
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
            .with_icon(IconName::ArrowUp)
            .with_section("Share"),
        );
    }

    actions.push(entry_plan.pin_action());

    actions.push(
        Action::new(
            "clip:clipboard_keep_in_today",
            "Keep in Today",
            Some("Adds this entry to today's Day Page (links are kept automatically)".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Plus)
        .with_section("Edit"),
    );

    if entry_plan.is_image() {
        actions.push(
            Action::new(
                "clip:clipboard_ocr",
                "Copy Text from Image",
                Some("Extracts text from the image with OCR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘C")
            .with_icon(IconName::MagnifyingGlass)
            .with_section("Edit"),
        );
    }

    if entry_plan.is_text() {
        actions.push(
            Action::new(
                "clip:clipboard_save_snippet",
                "Save Text as Snippet",
                Some("Creates a scriptlet from this text".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘S")
            .with_icon(IconName::Code)
            .with_section("Edit"),
        );
    }

    actions.push(
        Action::new(
            "clip:clipboard_save_file",
            "Save as File...",
            Some("Saves this entry to a file".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘S")
        .with_icon(IconName::File)
        .with_section("Share"),
    );

    actions.push(
        Action::new(
            DELETE_RECENT_ACTION_ID,
            "Delete Recent Entries…",
            Some("Deletes what you copied in the last few minutes, hour, or day".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Trash)
        .with_section("Destructive"),
    );

    actions.push(
        Action::new(
            "clip:clipboard_delete",
            "Delete Entry",
            Some("Removes this entry from clipboard history".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃X")
        .with_icon(IconName::Trash)
        .with_section("Destructive"),
    );

    actions.push(
        Action::new(
            "clip:clipboard_delete_multiple",
            "Delete Entries...",
            Some("Deletes entries matching the current search".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘X")
        .with_icon(IconName::Trash)
        .with_section("Destructive"),
    );

    actions.push(
        Action::new(
            "clip:clipboard_delete_all",
            "Delete All Entries",
            Some("Clears clipboard history except pinned entries".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⇧X")
        .with_icon(IconName::Trash)
        .with_section("Destructive"),
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
    fn info_for(content_type: ContentType, preview: &str) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "entry-x".to_string(),
            content_type,
            pinned: false,
            preview: preview.to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        }
    }

    fn ids_for(content_type: ContentType, preview: &str) -> Vec<String> {
        get_clipboard_history_context_actions(&info_for(content_type, preview))
            .into_iter()
            .map(|action| action.id)
            .collect()
    }

    #[test]
    fn each_kind_of_entry_gets_its_own_actions() {
        let link = ids_for(ContentType::Link, "https://example.com");
        assert!(link.iter().any(|id| id == "clip:clipboard_open_link"));
        let file = ids_for(ContentType::File, "/tmp/a.txt");
        assert!(file.iter().any(|id| id == "clip:clipboard_reveal_file"));
        let json = ids_for(ContentType::Text, "  {\"a\": 1}");
        assert!(json.iter().any(|id| id == "clip:clipboard_copy_json"));
        let prose = ids_for(ContentType::Text, "hello");
        assert!(!prose
            .iter()
            .any(|id| id.contains("json") || id.contains("open_link")));

        let color = get_clipboard_history_context_actions(&info_for(ContentType::Color, "#F80"));
        let rgb = color
            .iter()
            .find(|action| action.id == "clip:clipboard_copy_color_rgb")
            .expect("rgb row");
        assert_eq!(rgb.description.as_deref(), Some("rgb(255, 136, 0)"));
        assert!(prose.iter().any(|id| id == DELETE_RECENT_ACTION_ID));
        let route = get_clipboard_delete_recent_route();
        assert_eq!(route.actions.len(), RecentWindow::ALL.len());
        for window in RecentWindow::ALL {
            assert!(route.actions.iter().any(|a| a.id == window.action_id()));
            assert_eq!(
                RecentWindow::from_action_id(window.action_id()),
                Some(window)
            );
            assert_eq!(
                RecentWindow::from_action_id(window.action_id().trim_start_matches("clip:")),
                Some(window)
            );
        }
    }

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

    #[test]
    fn test_get_clipboard_history_context_actions_assigns_named_sections() {
        let actions = get_clipboard_history_context_actions(&entry_info(ContentType::Text, false));
        let allowed = ["Actions", "Edit", "Share", "Destructive"];

        assert!(
            !actions.is_empty(),
            "expected at least one clipboard action"
        );
        assert!(
            actions.iter().all(|action| action.section.is_some()),
            "every clipboard action should declare a section"
        );
        assert!(actions.iter().all(|action| {
            allowed.contains(&action.section.as_deref().expect("section should exist"))
        }));
    }

    #[test]
    fn test_get_clipboard_history_context_actions_delete_actions_are_destructive() {
        let actions = get_clipboard_history_context_actions(&entry_info(ContentType::Text, false));

        for id in [
            "clip:clipboard_delete",
            "clip:clipboard_delete_multiple",
            "clip:clipboard_delete_all",
        ] {
            let action = actions
                .iter()
                .find(|action| action.id == id)
                .expect("missing delete action");
            assert_eq!(action.section.as_deref(), Some("Destructive"));
        }
    }

    #[test]
    fn test_get_clipboard_history_context_actions_image_actions_use_expected_sections() {
        let actions = get_clipboard_history_context_actions(&entry_info(ContentType::Image, false));

        let ocr = actions
            .iter()
            .find(|action| action.id == "clip:clipboard_ocr")
            .expect("missing OCR action");
        assert_eq!(ocr.section.as_deref(), Some("Edit"));

        let save_file = actions
            .iter()
            .find(|action| action.id == "clip:clipboard_save_file")
            .expect("missing save-file action");
        assert_eq!(save_file.section.as_deref(), Some("Share"));
    }

    #[test]
    fn clipboard_save_snippet_action_is_text_only() {
        for pinned in [false, true] {
            let text_actions =
                get_clipboard_history_context_actions(&entry_info(ContentType::Text, pinned));
            assert!(text_actions
                .iter()
                .any(|action| action.id == "clip:clipboard_save_snippet"));

            for content_type in [
                ContentType::Image,
                ContentType::Link,
                ContentType::File,
                ContentType::Color,
            ] {
                let actions =
                    get_clipboard_history_context_actions(&entry_info(content_type, pinned));
                assert!(
                    actions
                        .iter()
                        .all(|action| action.id != "clip:clipboard_save_snippet"),
                    "{content_type:?} exposed clipboard_save_snippet"
                );
            }
        }
    }
}

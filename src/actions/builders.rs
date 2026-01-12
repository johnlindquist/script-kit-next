//! Action builders
//!
//! Factory functions for creating context-specific action lists.

use crate::designs::icon_variations::IconName;

use super::types::{Action, ActionCategory, ScriptInfo};
use crate::clipboard_history::ContentType;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;
use crate::scriptlets::Scriptlet;

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
    #[allow(dead_code)] // Used for context in action dialog, may be used by future actions
    pub image_dimensions: Option<(u32, u32)>,
}

/// Get actions specific to a file search result
/// Actions: Open (default), Show in Finder, Quick Look, Open With..., Show Info
pub fn get_file_context_actions(file_info: &FileInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    // Primary action - Open file
    if file_info.is_dir {
        actions.push(
            Action::new(
                "open_directory",
                format!("Open \"{}\"", file_info.name),
                Some("Open this folder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.push(
            Action::new(
                "open_file",
                format!("Open \"{}\"", file_info.name),
                Some("Open with default application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    // Show in Finder (Cmd+Enter)
    actions.push(
        Action::new(
            "reveal_in_finder",
            "Show in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    // Quick Look (Cmd+Y) - macOS only
    #[cfg(target_os = "macos")]
    if !file_info.is_dir {
        actions.push(
            Action::new(
                "quick_look",
                "Quick Look",
                Some("Preview with Quick Look".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘Y"),
        );
    }

    // Open With... (Cmd+O) - macOS only
    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "open_with",
            "Open With...",
            Some("Choose application to open with".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘O"),
    );

    // Show Info in Finder (Cmd+I) - macOS only
    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "show_info",
            "Get Info",
            Some("Show file information in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘I"),
    );

    // Copy Path
    actions.push(
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
    );

    // Copy Filename
    actions.push(
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘C"),
    );

    actions
}

/// Get actions specific to a file/folder path
pub fn get_path_context_actions(path_info: &PathInfo) -> Vec<Action> {
    let mut actions = vec![
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
        Action::new(
            "open_in_finder",
            "Open in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F"),
        Action::new(
            "open_in_editor",
            "Open in Editor",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E"),
        Action::new(
            "open_in_terminal",
            "Open in Terminal",
            Some("Open terminal at this location".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T"),
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename".to_string()),
            ActionCategory::ScriptContext,
        ),
        Action::new(
            "move_to_trash",
            "Move to Trash",
            Some(format!(
                "Delete {}",
                if path_info.is_dir { "folder" } else { "file" }
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫"),
    ];

    // Add directory-specific action for navigating into
    if path_info.is_dir {
        actions.insert(
            0,
            Action::new(
                "open_directory",
                format!("Open \"{}\"", path_info.name),
                Some("Navigate into this directory".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.insert(
            0,
            Action::new(
                "select_file",
                format!("Select \"{}\"", path_info.name),
                Some("Submit this file".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    actions
}

/// Convert a script name to a deeplink-safe format (lowercase, hyphenated)
///
/// Examples:
/// - "My Script" → "my-script"
/// - "Clipboard History" → "clipboard-history"
/// - "hello_world" → "hello-world"
pub fn to_deeplink_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Format a shortcut string for display in the UI
/// Converts "cmd+shift+c" to "⌘⇧C"
fn format_shortcut_hint(shortcut: &str) -> String {
    shortcut
        .replace("cmd", "⌘")
        .replace("ctrl", "⌃")
        .replace("alt", "⌥")
        .replace("shift", "⇧")
        .replace("+", "")
        .to_uppercase()
}

/// Convert scriptlet-defined actions (from H3 headers) to Action structs for the UI
///
/// These actions appear in the Actions Menu when a scriptlet is focused.
/// Each H3 header with a valid tool codefence in the scriptlet markdown
/// becomes an action that can execute that code.
///
/// # Example
/// ```markdown
/// ## My Scriptlet
/// ```bash
/// main code
/// ```
///
/// ### Copy to Clipboard
/// <!-- shortcut: cmd+c -->
/// ```bash
/// echo "{{text}}" | pbcopy
/// ```
/// ```
pub fn get_scriptlet_defined_actions(scriptlet: &Scriptlet) -> Vec<Action> {
    scriptlet
        .actions
        .iter()
        .map(|sa| {
            let mut action = Action::new(
                sa.action_id(),
                &sa.name,
                sa.description.clone(),
                ActionCategory::ScriptContext,
            );

            if let Some(ref shortcut) = sa.shortcut {
                action = action.with_shortcut(format_shortcut_hint(shortcut));
            }

            // Mark as scriptlet action for routing
            // has_action=true means this needs special handling (execute the action code)
            action.has_action = true;
            action.value = Some(sa.command.clone());

            action
        })
        .collect()
}

/// Get actions for a scriptlet, including both custom (H3-defined) and built-in actions
///
/// This merges:
/// 1. Primary action (Run)
/// 2. Custom actions defined via H3 headers in the scriptlet markdown
/// 3. Built-in scriptlet actions (Edit, Reveal, Copy Path)
/// 4. Universal actions (Shortcut, Alias, Deeplink)
pub fn get_scriptlet_context_actions_with_custom(
    script: &ScriptInfo,
    scriptlet: Option<&Scriptlet>,
) -> Vec<Action> {
    let mut actions = Vec::new();

    // 1. Primary action - Run the scriptlet
    actions.push(
        Action::new(
            "run_script",
            format!("{} \"{}\"", script.action_verb, script.name),
            Some(format!("{} this item", script.action_verb)),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
    );

    // 2. Custom actions from H3 headers
    if let Some(scriptlet) = scriptlet {
        actions.extend(get_scriptlet_defined_actions(scriptlet));
    }

    // 3. Dynamic shortcut actions
    if script.shortcut.is_some() {
        actions.push(
            Action::new(
                "update_shortcut",
                "Update Keyboard Shortcut",
                Some("Change the keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K"),
        );
        actions.push(
            Action::new(
                "remove_shortcut",
                "Remove Keyboard Shortcut",
                Some("Remove the current keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥K"),
        );
    } else {
        actions.push(
            Action::new(
                "add_shortcut",
                "Add Keyboard Shortcut",
                Some("Set a keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K"),
        );
    }

    // 4. Dynamic alias actions
    if script.alias.is_some() {
        actions.push(
            Action::new(
                "update_alias",
                "Update Alias",
                Some("Change the alias trigger".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A"),
        );
        actions.push(
            Action::new(
                "remove_alias",
                "Remove Alias",
                Some("Remove the current alias".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥A"),
        );
    } else {
        actions.push(
            Action::new(
                "add_alias",
                "Add Alias",
                Some("Set an alias trigger (type alias + space to run)".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A"),
        );
    }

    // 5. Scriptlet-specific built-in actions
    actions.push(
        Action::new(
            "edit_scriptlet",
            "Edit Scriptlet",
            Some("Open the markdown file in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E"),
    );

    actions.push(
        Action::new(
            "reveal_scriptlet_in_finder",
            "Reveal in Finder",
            Some("Show scriptlet bundle in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F"),
    );

    actions.push(
        Action::new(
            "copy_scriptlet_path",
            "Copy Path",
            Some("Copy scriptlet bundle path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
    );

    // 6. Copy deeplink
    let deeplink_name = to_deeplink_name(&script.name);
    actions.push(
        Action::new(
            "copy_deeplink",
            "Copy Deeplink",
            Some(format!(
                "Copy scriptkit://run/{} URL to clipboard",
                deeplink_name
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧D"),
    );

    // 7. Reset Ranking (if suggested)
    if script.is_suggested {
        actions.push(Action::new(
            "reset_ranking",
            "Reset Ranking",
            Some("Remove this item from Suggested section".to_string()),
            ActionCategory::ScriptContext,
        ));
    }

    actions
}

/// Get actions specific to the focused script
/// Actions are filtered based on whether this is a real script or a built-in command
pub fn get_script_context_actions(script: &ScriptInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    // Primary action - always available for both scripts and built-ins
    // Uses the action_verb from ScriptInfo (e.g., "Run", "Launch", "Switch to")
    actions.push(
        Action::new(
            "run_script",
            format!("{} \"{}\"", script.action_verb, script.name),
            Some(format!("{} this item", script.action_verb)),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
    );

    // Dynamic shortcut actions based on whether a shortcut already exists
    // If NO shortcut: Show "Add Keyboard Shortcut"
    // If HAS shortcut: Show "Update Keyboard Shortcut" and "Remove Keyboard Shortcut"
    if script.shortcut.is_some() {
        // Has existing shortcut - show Update and Remove options
        actions.push(
            Action::new(
                "update_shortcut",
                "Update Keyboard Shortcut",
                Some("Change the keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K"),
        );
        actions.push(
            Action::new(
                "remove_shortcut",
                "Remove Keyboard Shortcut",
                Some("Remove the current keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥K"),
        );
    } else {
        // No shortcut - show Add option
        actions.push(
            Action::new(
                "add_shortcut",
                "Add Keyboard Shortcut",
                Some("Set a keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K"),
        );
    }

    // Dynamic alias actions based on whether an alias already exists
    // If NO alias: Show "Add Alias"
    // If HAS alias: Show "Update Alias" and "Remove Alias"
    if script.alias.is_some() {
        // Has existing alias - show Update and Remove options
        actions.push(
            Action::new(
                "update_alias",
                "Update Alias",
                Some("Change the alias trigger".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A"),
        );
        actions.push(
            Action::new(
                "remove_alias",
                "Remove Alias",
                Some("Remove the current alias".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥A"),
        );
    } else {
        // No alias - show Add option
        actions.push(
            Action::new(
                "add_alias",
                "Add Alias",
                Some("Set an alias trigger (type alias + space to run)".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A"),
        );
    }

    // Script-only actions (not available for built-ins, apps, windows, scriptlets)
    if script.is_script {
        actions.push(
            Action::new(
                "edit_script",
                "Edit Script",
                Some("Open in $EDITOR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E"),
        );

        actions.push(
            Action::new(
                "view_logs",
                "View Logs",
                Some("Show script execution logs".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘L"),
        );

        actions.push(
            Action::new(
                "reveal_in_finder",
                "Reveal in Finder",
                Some("Show script file in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F"),
        );

        actions.push(
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy script path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C"),
        );
    }

    // Scriptlet-specific actions (work with the markdown file containing the scriptlet)
    if script.is_scriptlet {
        actions.push(
            Action::new(
                "edit_scriptlet",
                "Edit Scriptlet",
                Some("Open the markdown file in $EDITOR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E"),
        );

        actions.push(
            Action::new(
                "reveal_scriptlet_in_finder",
                "Reveal in Finder",
                Some("Show scriptlet bundle in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F"),
        );

        actions.push(
            Action::new(
                "copy_scriptlet_path",
                "Copy Path",
                Some("Copy scriptlet bundle path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C"),
        );
    }

    // Copy deeplink - available for both scripts and built-ins
    let deeplink_name = to_deeplink_name(&script.name);
    actions.push(
        Action::new(
            "copy_deeplink",
            "Copy Deeplink",
            Some(format!(
                "Copy scriptkit://run/{} URL to clipboard",
                deeplink_name
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧D"),
    );

    // Reset Ranking - only available for items that are suggested (have frecency data)
    if script.is_suggested {
        actions.push(Action::new(
            "reset_ranking",
            "Reset Ranking",
            Some("Remove this item from Suggested section".to_string()),
            ActionCategory::ScriptContext,
        ));
    }

    actions
}

/// Predefined global actions
/// Note: Settings and Quit are available from the main menu, not shown in actions dialog
pub fn get_global_actions() -> Vec<Action> {
    vec![]
}

/// Get actions specific to a clipboard history entry
/// Actions vary based on content type (text vs image) and pin status
#[allow(clippy::vec_init_then_push)] // Actions are conditionally added based on entry type
pub fn get_clipboard_history_context_actions(entry: &ClipboardEntryInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    // Primary action - Paste to focused app (simulates Cmd+V after copying)
    actions.push(
        Action::new(
            "clipboard_paste",
            "Paste to WezTerm",
            Some("Copy to clipboard and paste to focused app".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
    );

    // Copy to Clipboard (without pasting)
    actions.push(
        Action::new(
            "clipboard_copy",
            "Copy to Clipboard",
            Some("Copy entry to clipboard without pasting".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    // Paste and Keep Window Open
    actions.push(
        Action::new(
            "clipboard_paste_keep_open",
            "Paste and Keep Window Open",
            Some("Paste entry but keep the clipboard history open".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥↵"),
    );

    // Share...
    actions.push(
        Action::new(
            "clipboard_share",
            "Share...",
            Some("Share this entry via system share sheet".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘E"),
    );

    // Attach to AI Chat
    actions.push(
        Action::new(
            "clipboard_attach_to_ai",
            "Attach to AI Chat",
            Some("Send this entry to the AI chat window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⌘A"),
    );

    // Image-specific actions
    if entry.content_type == ContentType::Image {
        // Open With...
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

        // Quick Look
        #[cfg(target_os = "macos")]
        actions.push(
            Action::new(
                "clipboard_quick_look",
                "Quick Look",
                Some("Preview image with Quick Look".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘Y"),
        );

        // Annotate in CleanShot X (external app integration)
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

        // Upload to CleanShot X
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

    // Pin/Unpin Entry
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

    // Copy Text from Image (OCR) - image only
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

    // Save Text as Snippet
    actions.push(
        Action::new(
            "clipboard_save_snippet",
            "Save Text as Snippet",
            Some("Create a scriptlet from this text".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘S"),
    );

    // Save as File...
    actions.push(
        Action::new(
            "clipboard_save_file",
            "Save as File...",
            Some("Save this entry to a file".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘S"),
    );

    // --- Destructive actions (shown with red/destructive styling) ---

    // Delete Entry
    actions.push(
        Action::new(
            "clipboard_delete",
            "Delete Entry",
            Some("Remove this entry from clipboard history".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃X"),
    );

    // Delete Entries... (select multiple)
    actions.push(
        Action::new(
            "clipboard_delete_multiple",
            "Delete Entries...",
            Some("Select and delete multiple entries".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘X"),
    );

    // Delete All Entries
    actions.push(
        Action::new(
            "clipboard_delete_all",
            "Delete All Entries",
            Some("Clear all clipboard history (except pinned)".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌃⇧X"),
    );

    actions
}

/// Information about a chat prompt for action building
#[derive(Debug, Clone)]
pub struct ChatPromptInfo {
    /// Current model name (display name)
    pub current_model: Option<String>,
    /// Available models
    pub available_models: Vec<ChatModelInfo>,
    /// Whether there are messages to copy
    pub has_messages: bool,
    /// Whether there's an assistant response to copy
    pub has_response: bool,
}

/// Information about an available chat model
#[derive(Debug, Clone)]
pub struct ChatModelInfo {
    /// Model ID (for API calls)
    pub id: String,
    /// Display name shown in UI
    pub display_name: String,
    /// Provider name (Anthropic, OpenAI, etc.)
    pub provider: String,
}

/// Get actions specific to a chat prompt
/// Actions: Model selection, Continue in Chat, Copy Response, Clear Conversation
pub fn get_chat_context_actions(info: &ChatPromptInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    // Model selection actions - each model as a selectable action
    for model in &info.available_models {
        let is_current = info
            .current_model
            .as_ref()
            .map(|m| m == &model.display_name)
            .unwrap_or(false);

        let action = Action::new(
            format!("select_model_{}", model.id),
            if is_current {
                format!("{} ✓", model.display_name)
            } else {
                model.display_name.clone()
            },
            Some(format!("via {}", model.provider)),
            ActionCategory::ScriptContext,
        );
        actions.push(action);
    }

    // Separator (conceptual - ActionDialog handles visual separation)
    // Continue in Chat action
    actions.push(
        Action::new(
            "continue_in_chat",
            "Continue in Chat",
            Some("Open in AI Chat window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    // Copy Last Response (only if there's a response)
    if info.has_response {
        actions.push(
            Action::new(
                "copy_response",
                "Copy Last Response",
                Some("Copy the last assistant response".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘C"),
        );
    }

    // Clear Conversation (only if there are messages)
    if info.has_messages {
        actions.push(
            Action::new(
                "clear_conversation",
                "Clear Conversation",
                Some("Clear all messages".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌫"),
        );
    }

    actions
}

/// Get actions for the AI chat command bar (Cmd+K menu)
///
/// Returns actions with icons and sections for:
/// - Response: Copy Response, Copy Chat, Copy Last Code Block
/// - Actions: Submit, New Chat, Delete Chat
/// - Attachments: Add Attachments, Paste Image
/// - Settings: Change Model
#[allow(dead_code)] // Public API - will be used by AI window integration
pub fn get_ai_command_bar_actions() -> Vec<Action> {
    vec![
        // Response section
        Action::new(
            "copy_response",
            "Copy Response",
            Some("Copy the last AI response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        //
        Action::new(
            "copy_chat",
            "Copy Chat",
            Some("Copy the entire conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        //
        Action::new(
            "copy_last_code",
            "Copy Last Code Block",
            Some("Copy the most recent code block".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⌘C")
        .with_icon(IconName::Code)
        .with_section("Response"),
        // Actions section
        Action::new(
            "submit",
            "Submit",
            Some("Send your message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
        //
        Action::new(
            "new_chat",
            "New Chat",
            Some("Start a new conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘N")
        .with_icon(IconName::Plus)
        .with_section("Actions"),
        //
        Action::new(
            "delete_chat",
            "Delete Chat",
            Some("Delete current conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫")
        .with_icon(IconName::Trash)
        .with_section("Actions"),
        // Attachments section
        Action::new(
            "add_attachment",
            "Add Attachments...",
            Some("Attach files to your message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘A")
        .with_icon(IconName::Plus)
        .with_section("Attachments"),
        //
        Action::new(
            "paste_image",
            "Paste Image from Clipboard",
            Some("Paste an image from your clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘V")
        .with_icon(IconName::File)
        .with_section("Attachments"),
        // Settings section
        Action::new(
            "change_model",
            "Change Model",
            Some("Select a different AI model".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Settings)
        .with_section("Settings"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_script_context_actions_no_shortcut() {
        // Script without shortcut should show "Add Keyboard Shortcut"
        let script = ScriptInfo::new("my-script", "/path/to/my-script.ts");
        let actions = get_script_context_actions(&script);

        assert!(!actions.is_empty());
        // Script-specific actions should be present
        assert!(actions.iter().any(|a| a.id == "edit_script"));
        assert!(actions.iter().any(|a| a.id == "view_logs"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "run_script"));
        // Dynamic shortcut action - no shortcut means "Add"
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "remove_shortcut"));
        // Dynamic alias action - no alias means "Add"
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
        assert!(!actions.iter().any(|a| a.id == "remove_alias"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
    }

    #[test]
    fn test_get_script_context_actions_with_shortcut() {
        // Script with shortcut should show "Update" and "Remove" options
        let script = ScriptInfo::with_shortcut(
            "my-script",
            "/path/to/my-script.ts",
            Some("cmd+shift+m".to_string()),
        );
        let actions = get_script_context_actions(&script);

        // Dynamic shortcut actions - has shortcut means "Update" and "Remove"
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    }

    #[test]
    fn test_get_builtin_context_actions() {
        // Built-in commands should have limited actions
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);

        // Should have run, copy_deeplink, add_shortcut, and add_alias (no shortcut/alias by default)
        assert!(actions.iter().any(|a| a.id == "run_script"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(actions.iter().any(|a| a.id == "add_alias"));

        // Should NOT have script-only actions
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn test_get_scriptlet_context_actions() {
        // Scriptlets should have scriptlet-specific actions
        let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
        let actions = get_script_context_actions(&scriptlet);

        // Should have run, copy_deeplink, and add_shortcut (no shortcut by default)
        assert!(actions.iter().any(|a| a.id == "run_script"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));

        // Should have scriptlet-specific actions
        assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
        assert!(actions.iter().any(|a| a.id == "reveal_scriptlet_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_scriptlet_path"));

        // Verify edit_scriptlet has correct title
        let edit_action = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert_eq!(edit_action.title, "Edit Scriptlet");

        // Should NOT have script-only actions
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn test_get_script_context_actions_with_alias() {
        // Script with alias should show "Update Alias" and "Remove Alias" options
        let script = ScriptInfo::with_shortcut_and_alias(
            "my-script",
            "/path/to/my-script.ts",
            None,
            Some("ms".to_string()),
        );
        let actions = get_script_context_actions(&script);

        // Dynamic alias actions - has alias means "Update" and "Remove"
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    #[test]
    fn test_get_builtin_context_actions_with_alias() {
        // Built-in with alias should show "Update Alias" and "Remove Alias"
        let builtin = ScriptInfo::with_all(
            "Clipboard History",
            "builtin:clipboard-history",
            false,
            "Open",
            None,
            Some("ch".to_string()),
        );
        let actions = get_script_context_actions(&builtin);

        // Should have alias actions for update/remove
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    #[test]
    fn test_to_deeplink_name() {
        // Test the deeplink name conversion
        assert_eq!(to_deeplink_name("My Script"), "my-script");
        assert_eq!(to_deeplink_name("Clipboard History"), "clipboard-history");
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
        assert_eq!(
            to_deeplink_name("Test  Multiple   Spaces"),
            "test-multiple-spaces"
        );
        assert_eq!(to_deeplink_name("special!@#chars"), "special-chars");
    }

    #[test]
    fn test_get_global_actions() {
        let actions = get_global_actions();
        // Global actions are now empty - Settings/Quit available from main menu
        assert!(actions.is_empty());
    }

    #[test]
    fn test_built_in_actions_have_no_has_action() {
        // All built-in actions should have has_action=false
        let script = ScriptInfo::new("test-script", "/path/to/test.ts");
        let script_actions = get_script_context_actions(&script);
        let global_actions = get_global_actions();

        for action in script_actions.iter() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }

        for action in global_actions.iter() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn test_copy_deeplink_description_format() {
        // Verify the deeplink description shows the correct URL format
        let script = ScriptInfo::new("My Cool Script", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);

        let deeplink_action = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(deeplink_action
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-cool-script"));
    }

    #[test]
    fn test_get_file_context_actions_file() {
        // Test file actions for a regular file
        let file_info = FileInfo {
            path: "/Users/test/document.pdf".to_string(),
            name: "document.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);

        // Should have open_file as primary action
        assert!(actions.iter().any(|a| a.id == "open_file"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));

        // Should NOT have open_directory (not a directory)
        assert!(!actions.iter().any(|a| a.id == "open_directory"));

        // On macOS, should have Quick Look, Open With, Get Info
        #[cfg(target_os = "macos")]
        {
            assert!(actions.iter().any(|a| a.id == "quick_look"));
            assert!(actions.iter().any(|a| a.id == "open_with"));
            assert!(actions.iter().any(|a| a.id == "show_info"));
        }
    }

    #[test]
    fn test_get_file_context_actions_directory() {
        // Test file actions for a directory
        let file_info = FileInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);

        // Should have open_directory as primary action
        assert!(actions.iter().any(|a| a.id == "open_directory"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));

        // Should NOT have open_file (it's a directory)
        assert!(!actions.iter().any(|a| a.id == "open_file"));

        // Directory should NOT have quick_look (only files)
        #[cfg(target_os = "macos")]
        {
            assert!(!actions.iter().any(|a| a.id == "quick_look"));
            // But should have Open With and Get Info
            assert!(actions.iter().any(|a| a.id == "open_with"));
            assert!(actions.iter().any(|a| a.id == "show_info"));
        }
    }

    #[test]
    fn test_file_context_actions_shortcuts() {
        // Verify the keyboard shortcuts are correct
        let file_info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);

        // Check specific shortcuts
        let open_action = actions.iter().find(|a| a.id == "open_file").unwrap();
        assert_eq!(open_action.shortcut.as_ref().unwrap(), "↵");

        let reveal_action = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal_action.shortcut.as_ref().unwrap(), "⌘↵");

        #[cfg(target_os = "macos")]
        {
            let quick_look_action = actions.iter().find(|a| a.id == "quick_look").unwrap();
            assert_eq!(quick_look_action.shortcut.as_ref().unwrap(), "⌘Y");

            let show_info_action = actions.iter().find(|a| a.id == "show_info").unwrap();
            assert_eq!(show_info_action.shortcut.as_ref().unwrap(), "⌘I");
        }
    }

    #[test]
    fn test_reset_ranking_not_shown_when_not_suggested() {
        // Script without is_suggested should NOT show "Reset Ranking" action
        let script = ScriptInfo::new("test-script", "/path/to/test-script.ts");
        assert!(!script.is_suggested);

        let actions = get_script_context_actions(&script);

        // Should NOT have reset_ranking action
        assert!(
            !actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should not be shown when is_suggested is false"
        );
    }

    #[test]
    fn test_reset_ranking_shown_when_suggested() {
        // Script with is_suggested should show "Reset Ranking" action
        let script = ScriptInfo::new("test-script", "/path/to/test-script.ts")
            .with_frecency(true, Some("/path/to/test-script.ts".to_string()));
        assert!(script.is_suggested);

        let actions = get_script_context_actions(&script);

        // Should have reset_ranking action
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown when is_suggested is true"
        );

        // Verify action details
        let reset_action = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
        assert_eq!(reset_action.title, "Reset Ranking");
        assert_eq!(
            reset_action.description,
            Some("Remove this item from Suggested section".to_string())
        );
    }

    #[test]
    fn test_with_frecency_builder() {
        // Test the with_frecency builder method
        let script = ScriptInfo::new("test", "/path/to/test.ts")
            .with_frecency(true, Some("frecency:path".to_string()));

        assert!(script.is_suggested);
        assert_eq!(script.frecency_path, Some("frecency:path".to_string()));
    }

    #[test]
    fn test_reset_ranking_for_scriptlet() {
        // Scriptlet with is_suggested should show "Reset Ranking" action
        let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None)
            .with_frecency(true, Some("scriptlet:Open GitHub".to_string()));

        let actions = get_script_context_actions(&scriptlet);

        // Should have reset_ranking action for suggested scriptlet
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested scriptlets"
        );
    }

    #[test]
    fn test_reset_ranking_for_builtin() {
        // Built-in with is_suggested should show "Reset Ranking" action
        let builtin = ScriptInfo::builtin("Clipboard History")
            .with_frecency(true, Some("builtin:Clipboard History".to_string()));

        let actions = get_script_context_actions(&builtin);

        // Should have reset_ranking action for suggested built-in
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested built-ins"
        );
    }

    #[test]
    fn test_reset_ranking_for_app() {
        // App with is_suggested should show "Reset Ranking" action
        let app =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch")
                .with_frecency(true, Some("/Applications/Safari.app".to_string()));

        let actions = get_script_context_actions(&app);

        // Should have reset_ranking action for suggested app
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested apps"
        );
    }

    #[test]
    fn test_reset_ranking_for_window() {
        // Window with is_suggested should show "Reset Ranking" action
        let window = ScriptInfo::with_action_verb("My Document", "window:123", false, "Switch to")
            .with_frecency(true, Some("window:Preview:My Document".to_string()));

        let actions = get_script_context_actions(&window);

        // Should have reset_ranking action for suggested window
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested windows"
        );
    }

    #[test]
    fn test_reset_ranking_for_agent() {
        // Agent with is_suggested should show "Reset Ranking" action
        let agent = ScriptInfo::new("My Agent", "agent:/path/to/agent")
            .with_frecency(true, Some("agent:/path/to/agent".to_string()));

        let actions = get_script_context_actions(&agent);

        // Should have reset_ranking action for suggested agent
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested agents"
        );
    }

    #[test]
    fn test_reset_ranking_frecency_path_preserved() {
        // Verify that the frecency_path is correctly preserved through the builder
        let script = ScriptInfo::new("test", "/path/to/test.ts")
            .with_frecency(true, Some("/path/to/test.ts".to_string()));

        // Frecency path should be exactly what we set
        assert_eq!(script.frecency_path, Some("/path/to/test.ts".to_string()));
        assert!(script.is_suggested);
    }

    // ========================================
    // Scriptlet-Defined Action Tests (H3)
    // ========================================

    #[test]
    fn test_format_shortcut_hint_basic() {
        assert_eq!(format_shortcut_hint("cmd+c"), "⌘C");
        assert_eq!(format_shortcut_hint("cmd+shift+c"), "⌘⇧C");
        assert_eq!(format_shortcut_hint("ctrl+alt+delete"), "⌃⌥DELETE");
    }

    #[test]
    fn test_get_scriptlet_defined_actions_empty() {
        use crate::scriptlets::Scriptlet;

        let scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo test".to_string(),
        );

        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_scriptlet_defined_actions_basic() {
        use crate::scriptlets::{Scriptlet, ScriptletAction};

        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );

        scriptlet.actions = vec![
            ScriptletAction {
                name: "Copy to Clipboard".to_string(),
                command: "copy-to-clipboard".to_string(),
                tool: "bash".to_string(),
                code: "echo | pbcopy".to_string(),
                inputs: vec![],
                shortcut: Some("cmd+c".to_string()),
                description: Some("Copy to clipboard".to_string()),
            },
            ScriptletAction {
                name: "Open Browser".to_string(),
                command: "open-browser".to_string(),
                tool: "open".to_string(),
                code: "https://example.com".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];

        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions.len(), 2);

        // First action
        assert_eq!(actions[0].id, "scriptlet_action:copy-to-clipboard");
        assert_eq!(actions[0].title, "Copy to Clipboard");
        assert_eq!(actions[0].shortcut, Some("⌘C".to_string()));
        assert_eq!(
            actions[0].description,
            Some("Copy to clipboard".to_string())
        );
        assert!(actions[0].has_action);
        assert_eq!(actions[0].value, Some("copy-to-clipboard".to_string()));

        // Second action
        assert_eq!(actions[1].id, "scriptlet_action:open-browser");
        assert_eq!(actions[1].title, "Open Browser");
        assert!(actions[1].shortcut.is_none());
        assert!(actions[1].has_action);
    }

    #[test]
    fn test_get_scriptlet_context_actions_with_custom_empty() {
        let script = ScriptInfo::scriptlet("Test Scriptlet", "/path/to/test.md", None, None);

        // No scriptlet data passed
        let actions = get_scriptlet_context_actions_with_custom(&script, None);

        // Should have basic actions but no custom ones
        assert!(actions.iter().any(|a| a.id == "run_script"));
        assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));

        // No scriptlet_action: prefixed actions
        assert!(!actions
            .iter()
            .any(|a| a.id.starts_with("scriptlet_action:")));
    }

    #[test]
    fn test_get_scriptlet_context_actions_with_custom_actions() {
        use crate::scriptlets::{Scriptlet, ScriptletAction};

        let script = ScriptInfo::scriptlet("Test Scriptlet", "/path/to/test.md", None, None);

        let mut scriptlet = Scriptlet::new(
            "Test Scriptlet".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );

        scriptlet.actions = vec![ScriptletAction {
            name: "Custom Action".to_string(),
            command: "custom-action".to_string(),
            tool: "bash".to_string(),
            code: "echo custom".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+1".to_string()),
            description: Some("A custom action".to_string()),
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

        // Should have the custom action
        assert!(actions
            .iter()
            .any(|a| a.id == "scriptlet_action:custom-action"));

        // Custom actions should appear after run but before built-in actions
        let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:custom-action")
            .unwrap();
        let edit_idx = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();

        assert!(run_idx < custom_idx);
        assert!(custom_idx < edit_idx);
    }

    #[test]
    fn test_get_scriptlet_context_actions_with_custom_preserves_shortcut_alias() {
        let script = ScriptInfo::scriptlet(
            "Test",
            "/path/to/test.md",
            Some("cmd+t".to_string()),
            Some("ts".to_string()),
        );

        let actions = get_scriptlet_context_actions_with_custom(&script, None);

        // Should have update/remove for both since they exist
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));

        // Should NOT have add versions
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
    }

    #[test]
    fn test_get_scriptlet_context_actions_with_frecency() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));

        let actions = get_scriptlet_context_actions_with_custom(&script, None);

        // Should have reset_ranking since it's suggested
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    // ========================================
    // Clipboard History Action Tests
    // ========================================

    #[test]
    fn test_get_clipboard_history_text_actions() {
        let entry = ClipboardEntryInfo {
            id: "test-id".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello world".to_string(),
            image_dimensions: None,
        };

        let actions = get_clipboard_history_context_actions(&entry);

        // Should have primary actions
        assert!(actions.iter().any(|a| a.id == "clipboard_paste"));
        assert!(actions.iter().any(|a| a.id == "clipboard_copy"));
        assert!(actions.iter().any(|a| a.id == "clipboard_paste_keep_open"));
        assert!(actions.iter().any(|a| a.id == "clipboard_share"));
        assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));

        // Should have pin action (not unpin since not pinned)
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));

        // Should have save actions
        assert!(actions.iter().any(|a| a.id == "clipboard_save_snippet"));
        assert!(actions.iter().any(|a| a.id == "clipboard_save_file"));

        // Should have delete actions
        assert!(actions.iter().any(|a| a.id == "clipboard_delete"));
        assert!(actions.iter().any(|a| a.id == "clipboard_delete_multiple"));
        assert!(actions.iter().any(|a| a.id == "clipboard_delete_all"));

        // Should NOT have image-only actions for text entry
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    #[test]
    fn test_get_clipboard_history_image_actions() {
        let entry = ClipboardEntryInfo {
            id: "img-id".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image (800x600)".to_string(),
            image_dimensions: Some((800, 600)),
        };

        let actions = get_clipboard_history_context_actions(&entry);

        // Should have primary actions
        assert!(actions.iter().any(|a| a.id == "clipboard_paste"));
        assert!(actions.iter().any(|a| a.id == "clipboard_copy"));

        // Should have image-specific actions
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));

        // macOS-specific image actions
        #[cfg(target_os = "macos")]
        {
            assert!(actions.iter().any(|a| a.id == "clipboard_open_with"));
            assert!(actions.iter().any(|a| a.id == "clipboard_quick_look"));
            assert!(actions
                .iter()
                .any(|a| a.id == "clipboard_annotate_cleanshot"));
            assert!(actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
        }
    }

    #[test]
    fn test_get_clipboard_history_pinned_entry() {
        let entry = ClipboardEntryInfo {
            id: "pinned-id".to_string(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "Pinned text".to_string(),
            image_dimensions: None,
        };

        let actions = get_clipboard_history_context_actions(&entry);

        // Should have unpin action (not pin since already pinned)
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
    }

    #[test]
    fn test_clipboard_history_action_shortcuts() {
        let entry = ClipboardEntryInfo {
            id: "test-id".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Test".to_string(),
            image_dimensions: None,
        };

        let actions = get_clipboard_history_context_actions(&entry);

        // Verify specific shortcuts
        let paste_action = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste_action.shortcut.as_ref().unwrap(), "↵");

        let copy_action = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
        assert_eq!(copy_action.shortcut.as_ref().unwrap(), "⌘↵");

        let paste_keep_open = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert_eq!(paste_keep_open.shortcut.as_ref().unwrap(), "⌥↵");

        let delete_action = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert_eq!(delete_action.shortcut.as_ref().unwrap(), "⌃X");

        let delete_all = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert_eq!(delete_all.shortcut.as_ref().unwrap(), "⌃⇧X");
    }
}

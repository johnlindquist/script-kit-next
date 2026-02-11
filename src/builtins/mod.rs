//! Built-in Features Registry
//!
//! Provides a registry of built-in features that appear in the main search
//! alongside scripts. Features like Clipboard History and App Launcher are
//! configurable and can be enabled/disabled via config.
//!
//! ## Command Types
//!
//! The registry supports various command types organized by category:
//! - **System Actions**: Power management, UI controls, volume/brightness
//! - **Window Actions**: Window tiling and management for the frontmost window
//! - **Notes Commands**: Notes window operations
//! - **AI Commands**: AI chat window operations  
//! - **Script Commands**: Create new scripts and scriptlets
//! - **Permission Commands**: Accessibility permission management
//!

// --- merged from part_000.rs ---
use crate::config::BuiltInConfig;
use crate::menu_bar::MenuBarItem;
use tracing::debug;
// ============================================================================
// Command Type Enums
// ============================================================================

/// System action types for macOS system commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemActionType {
    // Power management
    EmptyTrash,
    LockScreen,
    Sleep,
    Restart,
    ShutDown,
    LogOut,

    // UI controls
    ToggleDarkMode,
    ShowDesktop,
    MissionControl,
    Launchpad,
    ForceQuitApps,

    // Volume controls (preset levels)
    Volume0,
    Volume25,
    Volume50,
    Volume75,
    Volume100,
    VolumeMute,

    // Dev/test actions (only available in debug builds)
    #[cfg(debug_assertions)]
    TestConfirmation,

    // App control
    QuitScriptKit,

    // System utilities
    ToggleDoNotDisturb,
    StartScreenSaver,

    // System Preferences
    OpenSystemPreferences,
    OpenPrivacySettings,
    OpenDisplaySettings,
    OpenSoundSettings,
    OpenNetworkSettings,
    OpenKeyboardSettings,
    OpenBluetoothSettings,
    OpenNotificationsSettings,
}
/// Returns user-facing HUD feedback text for system actions when available.
///
/// `dark_mode_enabled` should be populated only for `ToggleDarkMode` after
/// querying current system state.
pub fn system_action_hud_message(
    action_type: SystemActionType,
    dark_mode_enabled: Option<bool>,
) -> Option<String> {
    match action_type {
        SystemActionType::ToggleDarkMode => Some(match dark_mode_enabled {
            Some(true) => "Dark Mode On".to_string(),
            Some(false) => "Dark Mode Off".to_string(),
            None => "Dark Mode Toggled".to_string(),
        }),
        SystemActionType::Volume0 => Some("Volume 0%".to_string()),
        SystemActionType::Volume25 => Some("Volume 25%".to_string()),
        SystemActionType::Volume50 => Some("Volume 50%".to_string()),
        SystemActionType::Volume75 => Some("Volume 75%".to_string()),
        SystemActionType::Volume100 => Some("Volume 100%".to_string()),
        SystemActionType::VolumeMute => Some("Volume Muted".to_string()),
        SystemActionType::ToggleDoNotDisturb => Some("Do Not Disturb toggled".to_string()),
        _ => None,
    }
}
// NOTE: WindowActionType has been removed from built-ins.
// Window management is now handled by the window-management extension.
// SDK tileWindow() function still works via protocol messages in execute_script.rs.

/// Notes window command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NotesCommandType {
    OpenNotes,
    NewNote,
    SearchNotes,
    QuickCapture,
}
/// AI window command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AiCommandType {
    OpenAi,
    NewConversation,
    ClearConversation,
    /// Generate a new Script Kit script from the main prompt text
    GenerateScript,
    /// Send a screenshot of the entire screen to AI Chat
    SendScreenToAi,
    /// Send a screenshot of the focused window to AI Chat
    SendFocusedWindowToAi,
    /// Send the currently selected text to AI Chat
    SendSelectedTextToAi,
    /// Send the focused browser tab URL/content to AI Chat
    SendBrowserTabToAi,
    /// Send a selected screen area to AI Chat (interactive selection)
    SendScreenAreaToAi,
    /// Create a new AI chat preset/template
    CreateAiPreset,
    /// Import AI chat presets from file
    ImportAiPresets,
    /// Search through saved AI chat presets
    SearchAiPresets,
}
/// Script creation command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptCommandType {
    NewScript,
    NewExtension,
}
/// Permission management command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionCommandType {
    CheckPermissions,
    RequestAccessibility,
    OpenAccessibilitySettings,
}
/// Frecency/suggested items command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrecencyCommandType {
    ClearSuggested,
}
/// Settings command types for app configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCommandType {
    /// Reset all window positions to defaults
    ResetWindowPositions,
    /// Configure Vercel AI Gateway API key
    ConfigureVercelApiKey,
    /// Configure OpenAI API key
    ConfigureOpenAiApiKey,
    /// Configure Anthropic API key
    ConfigureAnthropicApiKey,
    /// Browse and apply color themes
    ChooseTheme,
}
/// Utility command types for quick access tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityCommandType {
    /// Open scratch pad - auto-saving editor
    ScratchPad,
    /// Open quick terminal for running commands
    QuickTerminal,
    /// Inspect actively running Script Kit child processes
    ProcessManager,
    /// Terminate all actively running Script Kit child processes
    StopAllProcesses,
}
/// Kit Store command types for browsing and managing kits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum KitStoreCommandType {
    BrowseKits,
    InstalledKits,
    UpdateAllKits,
}
/// Menu bar action details for executing menu commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarActionInfo {
    /// The bundle ID of the app (e.g., "com.apple.Safari")
    pub bundle_id: String,
    /// The path to the menu item (e.g., ["File", "New Window"])
    pub menu_path: Vec<String>,
    /// Whether the menu item is enabled
    pub enabled: bool,
    /// Keyboard shortcut if any (e.g., "⌘N")
    pub shortcut: Option<String>,
}
/// Groups for categorizing built-in entries in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // MenuBar variant will be used when menu bar integration is complete
pub enum BuiltInGroup {
    /// Core built-in features (Clipboard History, Window Switcher, etc.)
    #[default]
    Core,
    /// Menu bar items from the frontmost application
    MenuBar,
}
/// Types of built-in features
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Some variants reserved for future use
pub enum BuiltInFeature {
    /// Clipboard history viewer/manager
    ClipboardHistory,
    /// Favorites list and quick access
    Favorites,
    /// Application launcher for opening installed apps (legacy, apps now in main search)
    AppLauncher,
    /// Individual application entry (for future use when apps appear in search)
    App(String),
    /// Window switcher for managing and tiling windows
    WindowSwitcher,
    /// Design gallery for viewing separator and icon variations
    DesignGallery,
    /// AI Chat window for conversing with AI assistants
    AiChat,
    /// Notes window for quick notes and scratchpad
    Notes,
    /// Emoji picker for selecting and copying emojis
    EmojiPicker,
    /// Quick links manager and URL launcher
    Quicklinks,
    /// Menu bar action from the frontmost application
    MenuBarAction(MenuBarActionInfo),

    // === New Command Types ===
    /// System actions (power, UI controls, volume, brightness, settings)
    SystemAction(SystemActionType),
    // NOTE: WindowAction removed - now handled by window-management extension
    /// Notes window commands
    NotesCommand(NotesCommandType),
    /// AI window commands
    AiCommand(AiCommandType),
    /// Script creation commands
    ScriptCommand(ScriptCommandType),
    /// Permission management commands
    PermissionCommand(PermissionCommandType),
    /// Frecency/suggested items commands
    FrecencyCommand(FrecencyCommandType),
    /// Settings commands (window positions, etc.)
    SettingsCommand(SettingsCommandType),
    /// Utility commands (scratch pad, quick terminal)
    UtilityCommand(UtilityCommandType),
    /// Kit Store commands (browse, installed kits, update all)
    KitStoreCommand(KitStoreCommandType),
    /// File search for navigating directories and finding files
    FileSearch,
    /// Webcam capture
    Webcam,
}
/// A built-in feature entry that appears in the main search
#[derive(Debug, Clone)]
pub struct BuiltInEntry {
    /// Unique identifier for the entry
    pub id: String,
    /// Display name shown in search results
    pub name: String,
    /// Description shown below the name
    pub description: String,
    /// Keywords for fuzzy matching in search
    pub keywords: Vec<String>,
    /// The actual feature this entry represents
    pub feature: BuiltInFeature,
    /// Optional icon (emoji) to display
    pub icon: Option<String>,
    /// Group for categorization in the UI (will be used when menu bar integration is complete)
    #[allow(dead_code)]
    pub group: BuiltInGroup,
}
impl BuiltInEntry {
    /// Create a new built-in entry (Core group, no icon)
    #[allow(dead_code)]
    fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        keywords: Vec<&str>,
        feature: BuiltInFeature,
    ) -> Self {
        BuiltInEntry {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            keywords: keywords.into_iter().map(String::from).collect(),
            feature,
            icon: None,
            group: BuiltInGroup::Core,
        }
    }

    /// Create a new built-in entry with an icon (Core group)
    fn new_with_icon(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        keywords: Vec<&str>,
        feature: BuiltInFeature,
        icon: impl Into<String>,
    ) -> Self {
        BuiltInEntry {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            keywords: keywords.into_iter().map(String::from).collect(),
            feature,
            icon: Some(icon.into()),
            group: BuiltInGroup::Core,
        }
    }

    /// Create a new built-in entry with icon and group
    #[allow(dead_code)]
    pub fn new_with_group(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        keywords: Vec<String>,
        feature: BuiltInFeature,
        icon: Option<String>,
        group: BuiltInGroup,
    ) -> Self {
        BuiltInEntry {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            keywords,
            feature,
            icon,
            group,
        }
    }

    /// Check if this built-in should be excluded from frecency/suggested tracking.
    /// Some commands like "Quit Script Kit" don't make sense to suggest.
    /// Uses the user-configurable excluded_commands list from SuggestedConfig.
    pub fn should_exclude_from_frecency(&self, excluded_commands: &[String]) -> bool {
        excluded_commands.iter().any(|cmd| cmd == &self.id)
    }

    /// Get the leaf name for menu bar items (the actual menu item name, not the full path).
    /// For "Shell → New Tab", returns "New Tab".
    /// For non-menu bar items, returns the full name.
    pub fn leaf_name(&self) -> &str {
        if self.group == BuiltInGroup::MenuBar {
            // Menu bar names are formatted as "Menu → Submenu → Item"
            // Extract the last component (the actual menu item name)
            self.name.rsplit(" → ").next().unwrap_or(&self.name)
        } else {
            &self.name
        }
    }
}
// --- merged from part_001.rs ---
/// Get the list of enabled built-in entries based on configuration
///
/// # Arguments
/// * `config` - The built-in features configuration
///
/// # Returns
/// A vector of enabled built-in entries that should appear in the main search
///
/// Note: AppLauncher built-in is no longer used since apps now appear directly
/// in the main search results. The config option is retained for future use
/// (e.g., to control whether apps are included in search at all).
pub fn get_builtin_entries(config: &BuiltInConfig) -> Vec<BuiltInEntry> {
    let mut entries = Vec::new();

    include!("part_001_entries/entries_000.rs");
    include!("part_001_entries/entries_001.rs");
    include!("part_001_entries/entries_002.rs");
    include!("part_001_entries/entries_003.rs");

    debug!(count = entries.len(), "Built-in entries loaded");
    entries
}
// --- merged from part_002.rs ---
// ============================================================================
// Menu Bar Item Conversion
// ============================================================================

/// Convert menu bar items to built-in entries for search
///
/// This flattens the menu hierarchy into searchable entries, skipping the
/// Apple menu (first item) and only including leaf items (no submenus).
///
/// # Arguments
/// * `items` - The menu bar items from the frontmost application
/// * `bundle_id` - The bundle identifier of the application (e.g., "com.apple.Safari")
/// * `app_name` - The display name of the application (e.g., "Safari")
///
/// # Returns
/// A vector of `BuiltInEntry` items that can be added to search results
#[allow(dead_code)] // Will be used when menu bar integration is complete
pub fn menu_bar_items_to_entries(
    items: &[MenuBarItem],
    bundle_id: &str,
    app_name: &str,
) -> Vec<BuiltInEntry> {
    let mut entries = Vec::new();

    // Skip first item (Apple menu)
    for item in items.iter().skip(1) {
        flatten_menu_item(item, bundle_id, app_name, &[], &mut entries);
    }

    debug!(
        count = entries.len(),
        bundle_id = bundle_id,
        app_name = app_name,
        "Menu bar items converted to entries"
    );
    entries
}
/// Recursively flatten a menu item and its children into entries
#[allow(dead_code)] // Will be used when menu bar integration is complete
fn flatten_menu_item(
    item: &MenuBarItem,
    bundle_id: &str,
    app_name: &str,
    parent_path: &[String],
    entries: &mut Vec<BuiltInEntry>,
) {
    // Skip separators and disabled items
    if item.title.is_empty() || item.title == "-" || item.is_separator() || !item.enabled {
        return;
    }

    let mut current_path = parent_path.to_vec();
    current_path.push(item.title.clone());

    // Only add leaf items (items without children) as entries
    if item.children.is_empty() {
        let id = format!(
            "menubar-{}-{}",
            bundle_id,
            current_path.join("-").to_lowercase().replace(' ', "-")
        );
        let name = current_path.join(" → ");
        let description = if let Some(ref shortcut) = item.shortcut {
            format!("{}  {}", app_name, shortcut.to_display_string())
        } else {
            app_name.to_string()
        };
        let keywords: Vec<String> = current_path.iter().map(|s| s.to_lowercase()).collect();
        let icon = get_menu_icon(&current_path[0]);

        entries.push(BuiltInEntry {
            id,
            name,
            description,
            keywords,
            feature: BuiltInFeature::MenuBarAction(MenuBarActionInfo {
                bundle_id: bundle_id.to_string(),
                menu_path: current_path,
                enabled: item.enabled,
                shortcut: item.shortcut.as_ref().map(|s| s.to_display_string()),
            }),
            icon: Some(icon.to_string()),
            group: BuiltInGroup::MenuBar,
        });
    } else {
        // Recurse into children
        for child in &item.children {
            flatten_menu_item(child, bundle_id, app_name, &current_path, entries);
        }
    }
}
/// Get an appropriate icon for a top-level menu
#[allow(dead_code)] // Will be used when menu bar integration is complete
fn get_menu_icon(top_menu: &str) -> &'static str {
    match top_menu.to_lowercase().as_str() {
        "file" => "📁",
        "edit" => "📋",
        "view" => "👁",
        "window" => "🪟",
        "help" => "❓",
        "format" => "🎨",
        "tools" => "🔧",
        "go" => "➡️",
        "bookmarks" | "favorites" => "⭐",
        "history" => "🕐",
        "develop" | "developer" => "🛠",
        _ => "📌",
    }
}
// --- merged from part_003.rs ---
#[cfg(test)]
mod tests {
    // --- merged from part_000.rs ---
    use super::*;
    use crate::config::BuiltInConfig;
    #[test]
    fn test_builtin_config_default() {
        let config = BuiltInConfig::default();
        assert!(config.clipboard_history);
        assert!(config.app_launcher);
        assert!(config.window_switcher);
    }
    #[test]
    fn test_builtin_config_custom() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: true,
            window_switcher: false,
        };
        assert!(!config.clipboard_history);
        assert!(config.app_launcher);
        assert!(!config.window_switcher);
    }
    #[test]
    fn test_get_builtin_entries_all_enabled() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Core built-ins: Clipboard history, window switcher, AI chat, Notes, design gallery
        // Plus: system actions (28), window actions (6), notes commands (3), AI commands (1),
        // script commands (2), permission commands (3) = 43 new entries
        // Total: 5 + 43 = 48
        assert!(entries.len() >= 5); // At minimum the core built-ins should exist

        // Check clipboard history entry
        let clipboard = entries.iter().find(|e| e.id == "builtin-clipboard-history");
        assert!(clipboard.is_some());
        let clipboard = clipboard.unwrap();
        assert_eq!(clipboard.name, "Clipboard History");
        assert_eq!(clipboard.feature, BuiltInFeature::ClipboardHistory);
        assert!(clipboard.keywords.contains(&"clipboard".to_string()));
        assert!(clipboard.keywords.contains(&"history".to_string()));
        assert!(clipboard.keywords.contains(&"paste".to_string()));
        assert!(clipboard.keywords.contains(&"copy".to_string()));

        // Check window switcher entry
        let window_switcher = entries.iter().find(|e| e.id == "builtin-window-switcher");
        assert!(window_switcher.is_some());
        let window_switcher = window_switcher.unwrap();
        assert_eq!(window_switcher.name, "Window Switcher");
        assert_eq!(window_switcher.feature, BuiltInFeature::WindowSwitcher);
        assert!(window_switcher.keywords.contains(&"window".to_string()));
        assert!(window_switcher.keywords.contains(&"switch".to_string()));
        assert!(window_switcher.keywords.contains(&"tile".to_string()));
        assert!(window_switcher.keywords.contains(&"focus".to_string()));
        assert!(window_switcher.keywords.contains(&"manage".to_string()));
        assert!(window_switcher.keywords.contains(&"switcher".to_string()));

        // Check AI chat entry
        let ai_chat = entries.iter().find(|e| e.id == "builtin-ai-chat");
        assert!(ai_chat.is_some());
        let ai_chat = ai_chat.unwrap();
        assert_eq!(ai_chat.name, "AI Chat");
        assert_eq!(ai_chat.feature, BuiltInFeature::AiChat);
        assert!(ai_chat.keywords.contains(&"ai".to_string()));
        assert!(ai_chat.keywords.contains(&"chat".to_string()));
        assert!(ai_chat.keywords.contains(&"claude".to_string()));
        assert!(ai_chat.keywords.contains(&"gpt".to_string()));

        // Check Emoji Picker entry
        let emoji_picker = entries.iter().find(|e| e.id == "builtin-emoji-picker");
        assert!(emoji_picker.is_some());
        let emoji_picker = emoji_picker.unwrap();
        assert_eq!(emoji_picker.name, "Emoji Picker");
        assert_eq!(emoji_picker.feature, BuiltInFeature::EmojiPicker);
        assert!(emoji_picker.keywords.contains(&"emoji".to_string()));
        assert!(emoji_picker.keywords.contains(&"picker".to_string()));

        // Check Quicklinks entry
        let quicklinks = entries.iter().find(|e| e.id == "builtin-quicklinks");
        assert!(quicklinks.is_some());
        let quicklinks = quicklinks.unwrap();
        assert_eq!(quicklinks.name, "Quicklinks");
        assert_eq!(quicklinks.feature, BuiltInFeature::Quicklinks);
        assert!(quicklinks.keywords.contains(&"quicklinks".to_string()));
        assert!(quicklinks.keywords.contains(&"url".to_string()));

        // Note: App Launcher built-in removed - apps now appear directly in main search
    }
    #[test]
    fn test_get_builtin_entries_clipboard_only() {
        let config = BuiltInConfig {
            clipboard_history: true,
            app_launcher: false,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // Check that core entries exist (plus all the new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-clipboard-history"));
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Window switcher should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-window-switcher"));
    }
    #[test]
    fn test_get_builtin_entries_app_launcher_only() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: true,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // App launcher no longer creates a built-in entry (apps appear in main search)
        // But AI Chat, Notes and Design Gallery are always enabled (plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Clipboard history should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
    }
    #[test]
    fn test_get_builtin_entries_none_enabled() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: false,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // AI Chat, Notes, and Design Gallery are always enabled (plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Clipboard history and window switcher should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
        assert!(!entries.iter().any(|e| e.id == "builtin-window-switcher"));
    }
    #[test]
    fn test_get_builtin_entries_window_switcher_only() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: false,
            window_switcher: true,
        };
        let entries = get_builtin_entries(&config);

        // Window switcher + AI Chat + Notes + Design Gallery (always enabled, plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-window-switcher"));
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Verify window switcher has correct properties
        let window_switcher = entries
            .iter()
            .find(|e| e.id == "builtin-window-switcher")
            .unwrap();
        assert_eq!(window_switcher.feature, BuiltInFeature::WindowSwitcher);
        assert_eq!(window_switcher.icon, Some("🪟".to_string()));

        // Clipboard history should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
    }
    #[test]
    fn test_builtin_feature_equality() {
        assert_eq!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::ClipboardHistory
        );
        assert_eq!(BuiltInFeature::AppLauncher, BuiltInFeature::AppLauncher);
        assert_eq!(
            BuiltInFeature::WindowSwitcher,
            BuiltInFeature::WindowSwitcher
        );
        assert_eq!(BuiltInFeature::DesignGallery, BuiltInFeature::DesignGallery);
        assert_eq!(BuiltInFeature::AiChat, BuiltInFeature::AiChat);
        assert_eq!(BuiltInFeature::Favorites, BuiltInFeature::Favorites);
        assert_eq!(BuiltInFeature::EmojiPicker, BuiltInFeature::EmojiPicker);
        assert_eq!(BuiltInFeature::Quicklinks, BuiltInFeature::Quicklinks);
        assert_ne!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::AppLauncher
        );
        assert_ne!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::WindowSwitcher
        );
        assert_ne!(BuiltInFeature::AppLauncher, BuiltInFeature::WindowSwitcher);
        assert_ne!(
            BuiltInFeature::DesignGallery,
            BuiltInFeature::ClipboardHistory
        );
        assert_ne!(BuiltInFeature::AiChat, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::AiChat, BuiltInFeature::DesignGallery);
        assert_ne!(BuiltInFeature::Favorites, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::EmojiPicker, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::Quicklinks, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::EmojiPicker, BuiltInFeature::Quicklinks);

        // Test App variant
        assert_eq!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::App("Safari".to_string())
        );
        assert_ne!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::App("Chrome".to_string())
        );
        assert_ne!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::AppLauncher
        );
    }
    #[test]
    fn test_builtin_entry_new() {
        let entry = BuiltInEntry::new(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test", "keyword"],
            BuiltInFeature::ClipboardHistory,
        );

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.description, "Test description");
        assert_eq!(
            entry.keywords,
            vec!["test".to_string(), "keyword".to_string()]
        );
        assert_eq!(entry.feature, BuiltInFeature::ClipboardHistory);
        assert_eq!(entry.icon, None);
    }
    #[test]
    fn test_builtin_entry_new_with_icon() {
        let entry = BuiltInEntry::new_with_icon(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test"],
            BuiltInFeature::ClipboardHistory,
            "📋",
        );

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.icon, Some("📋".to_string()));
    }
    #[test]
    fn test_builtin_entry_clone() {
        let entry = BuiltInEntry::new_with_icon(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test"],
            BuiltInFeature::AppLauncher,
            "🚀",
        );

        let cloned = entry.clone();
        assert_eq!(entry.id, cloned.id);
        assert_eq!(entry.name, cloned.name);
        assert_eq!(entry.description, cloned.description);
        assert_eq!(entry.keywords, cloned.keywords);
        assert_eq!(entry.feature, cloned.feature);
        assert_eq!(entry.icon, cloned.icon);
    }
    #[test]
    fn test_builtin_config_clone() {
        let config = BuiltInConfig {
            clipboard_history: true,
            app_launcher: false,
            window_switcher: true,
        };

        let cloned = config.clone();
        assert_eq!(config.clipboard_history, cloned.clipboard_history);
        assert_eq!(config.app_launcher, cloned.app_launcher);
        assert_eq!(config.window_switcher, cloned.window_switcher);
    }
    #[test]
    fn test_system_action_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that system action entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-empty-trash"));
        assert!(entries.iter().any(|e| e.id == "builtin-lock-screen"));
        assert!(entries.iter().any(|e| e.id == "builtin-toggle-dark-mode"));
        // Volume presets
        assert!(entries.iter().any(|e| e.id == "builtin-volume-0"));
        assert!(entries.iter().any(|e| e.id == "builtin-volume-50"));
        assert!(entries.iter().any(|e| e.id == "builtin-volume-100"));
        assert!(entries.iter().any(|e| e.id == "builtin-system-preferences"));
    }
    // NOTE: test_window_action_entries_exist removed - window actions now in extension

    #[test]
    fn test_notes_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that notes command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-open-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-new-note"));
        assert!(entries.iter().any(|e| e.id == "builtin-search-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-quick-capture"));
    }
    #[test]
    fn test_get_builtin_entries_includes_open_notes_and_open_ai_commands() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let open_notes = entries.iter().find(|e| e.id == "builtin-open-notes");
        assert!(open_notes.is_some(), "builtin-open-notes should exist");
        assert_eq!(
            open_notes.unwrap().feature,
            BuiltInFeature::NotesCommand(NotesCommandType::OpenNotes)
        );

        let open_ai = entries.iter().find(|e| e.id == "builtin-open-ai");
        assert!(open_ai.is_some(), "builtin-open-ai should exist");
        assert_eq!(
            open_ai.unwrap().feature,
            BuiltInFeature::AiCommand(AiCommandType::OpenAi)
        );

        let generate_script = entries
            .iter()
            .find(|e| e.id == "builtin-generate-script-with-ai");
        assert!(
            generate_script.is_some(),
            "builtin-generate-script-with-ai should exist"
        );
        let generate_script = generate_script.unwrap();
        assert_eq!(
            generate_script.feature,
            BuiltInFeature::AiCommand(AiCommandType::GenerateScript)
        );
        assert!(
            generate_script
                .keywords
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case("shift")),
            "Generate Script command should be discoverable via Shift+Tab wording"
        );
    }
    #[test]
    fn test_get_builtin_entries_hides_preview_ai_commands() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        assert!(
            !entries
                .iter()
                .any(|e| e.id == "builtin-send-screen-area-to-ai"),
            "Preview command should be hidden from built-in entries"
        );
        assert!(
            !entries.iter().any(|e| e.id == "builtin-create-ai-preset"),
            "Preview command should be hidden from built-in entries"
        );
        assert!(
            !entries.iter().any(|e| e.id == "builtin-import-ai-presets"),
            "Preview command should be hidden from built-in entries"
        );
        assert!(
            !entries.iter().any(|e| e.id == "builtin-search-ai-presets"),
            "Preview command should be hidden from built-in entries"
        );
    }

    #[test]
    fn test_get_builtin_entries_includes_favorites_command() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let favorites = entries.iter().find(|e| e.id == "builtin-favorites");
        assert!(favorites.is_some(), "builtin-favorites should exist");

        let favorites = favorites.unwrap();
        assert_eq!(favorites.name, "Favorites");
        assert_eq!(favorites.feature, BuiltInFeature::Favorites);
        assert!(
            favorites
                .keywords
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case("star")),
            "Favorites command should be discoverable with 'star'"
        );
    }
    #[test]
    fn test_script_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that script command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-new-script"));
        assert!(entries.iter().any(|e| e.id == "builtin-new-extension"));
    }
    #[test]
    fn test_new_creation_commands_are_discoverable() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let new_script = entries
            .iter()
            .find(|e| e.id == "builtin-new-script")
            .expect("builtin-new-script should exist");
        assert!(
            new_script.name.to_lowercase().contains("new"),
            "New Script entry name should prominently include 'new'"
        );
        assert!(
            new_script
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("template")),
            "New Script entry should be discoverable via 'template'"
        );
        assert!(
            new_script
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("starter")),
            "New Script entry should be discoverable via 'starter'"
        );

        let new_extension = entries
            .iter()
            .find(|e| e.id == "builtin-new-extension")
            .expect("builtin-new-extension should exist");
        assert!(
            new_extension
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("scriptlet")),
            "New Extension entry should be discoverable via 'scriptlet'"
        );
        assert!(
            new_extension
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("frontmatter")),
            "New Extension entry should be discoverable via 'frontmatter'"
        );
    }
    #[test]
    fn test_permission_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that permission command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-check-permissions"));
        assert!(entries
            .iter()
            .any(|e| e.id == "builtin-request-accessibility"));
        assert!(entries
            .iter()
            .any(|e| e.id == "builtin-accessibility-settings"));
    }
    #[test]
    fn test_system_action_type_equality() {
        assert_eq!(SystemActionType::EmptyTrash, SystemActionType::EmptyTrash);
        assert_ne!(SystemActionType::EmptyTrash, SystemActionType::LockScreen);
    }
    // NOTE: test_window_action_type_equality removed - WindowActionType no longer in builtins

    #[test]
    fn test_builtin_feature_system_action() {
        let feature = BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode);
        assert_eq!(
            feature,
            BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode)
        );
        assert_ne!(
            feature,
            BuiltInFeature::SystemAction(SystemActionType::Sleep)
        );
    }
    // --- merged from part_001.rs ---
    // NOTE: test_builtin_feature_window_action removed - WindowAction no longer in BuiltInFeature

    #[test]
    fn test_file_search_builtin_exists() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that FileSearch entry exists
        let file_search = entries.iter().find(|e| e.id == "builtin-file-search");
        assert!(
            file_search.is_some(),
            "FileSearch builtin should exist in the main menu"
        );

        let file_search = file_search.unwrap();
        assert_eq!(file_search.name, "Search Files");
        assert_eq!(file_search.feature, BuiltInFeature::FileSearch);
        assert!(file_search.keywords.contains(&"file".to_string()));
        assert!(file_search.keywords.contains(&"search".to_string()));
        assert!(file_search.keywords.contains(&"find".to_string()));
        assert!(file_search.keywords.contains(&"directory".to_string()));
        assert!(file_search.icon.is_some());
    }
    #[test]
    fn test_get_builtin_entries_includes_process_manager_command() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let process_manager = entries.iter().find(|e| e.id == "builtin-process-manager");
        assert!(
            process_manager.is_some(),
            "Process Manager builtin should exist in the main menu"
        );

        let process_manager = process_manager.unwrap();
        assert_eq!(
            process_manager.feature,
            BuiltInFeature::UtilityCommand(UtilityCommandType::ProcessManager)
        );
        assert!(process_manager.keywords.iter().any(|k| k == "process"));
        assert!(process_manager.keywords.iter().any(|k| k == "running"));
        assert!(process_manager.keywords.iter().any(|k| k == "kill"));
    }
    #[test]
    fn test_get_builtin_entries_includes_stop_all_processes_command() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let stop_all = entries
            .iter()
            .find(|e| e.id == "builtin-stop-all-processes");
        assert!(
            stop_all.is_some(),
            "Stop all running scripts builtin should exist in the main menu"
        );

        let stop_all = stop_all.unwrap();
        assert_eq!(
            stop_all.feature,
            BuiltInFeature::UtilityCommand(UtilityCommandType::StopAllProcesses)
        );
        assert!(stop_all.keywords.iter().any(|k| k == "stop"));
        assert!(stop_all.keywords.iter().any(|k| k == "kill"));
        assert!(stop_all.keywords.iter().any(|k| k == "terminate"));
    }
    #[test]
    fn test_builtin_descriptions_use_clear_action_phrasing() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let notes = entries.iter().find(|e| e.id == "builtin-notes").unwrap();
        assert_eq!(
            notes.description,
            "Open quick notes and a scratchpad editor"
        );

        let quick_capture = entries
            .iter()
            .find(|e| e.id == "builtin-quick-capture")
            .unwrap();
        assert_eq!(
            quick_capture.description,
            "Capture a new note without opening the full Notes window"
        );

        let file_search = entries
            .iter()
            .find(|e| e.id == "builtin-file-search")
            .unwrap();
        assert_eq!(
            file_search.description,
            "Browse directories, search files, and open results"
        );

        let webcam = entries.iter().find(|e| e.id == "builtin-webcam").unwrap();
        assert_eq!(
            webcam.description,
            "Open the webcam prompt and capture a photo"
        );
    }
    #[test]
    fn test_file_search_feature_equality() {
        assert_eq!(BuiltInFeature::FileSearch, BuiltInFeature::FileSearch);
        assert_ne!(BuiltInFeature::FileSearch, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::FileSearch, BuiltInFeature::Notes);
    }
    #[test]
    fn test_system_action_hud_message_volume_presets() {
        assert_eq!(
            system_action_hud_message(SystemActionType::Volume0, None),
            Some("Volume 0%".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::Volume50, None),
            Some("Volume 50%".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::Volume100, None),
            Some("Volume 100%".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::VolumeMute, None),
            Some("Volume Muted".to_string())
        );
    }
    #[test]
    fn test_system_action_hud_message_dark_mode() {
        assert_eq!(
            system_action_hud_message(SystemActionType::ToggleDarkMode, Some(true)),
            Some("Dark Mode On".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::ToggleDarkMode, Some(false)),
            Some("Dark Mode Off".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::ToggleDarkMode, None),
            Some("Dark Mode Toggled".to_string())
        );
    }
}

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

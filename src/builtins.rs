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
}

/// Utility command types for quick access tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityCommandType {
    /// Open scratch pad - auto-saving editor
    ScratchPad,
    /// Open quick terminal for running commands
    QuickTerminal,
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
    /// File search for navigating directories and finding files
    FileSearch,
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

    if config.clipboard_history {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-clipboard-history",
            "Clipboard History",
            "View and manage your clipboard history",
            vec!["clipboard", "history", "paste", "copy"],
            BuiltInFeature::ClipboardHistory,
            "📋",
        ));
        debug!("Added Clipboard History built-in entry");
    }

    // Note: AppLauncher built-in removed - apps now appear directly in main search
    // The app_launcher config flag is kept for future use (e.g., to disable app search entirely)
    if config.app_launcher {
        debug!("app_launcher enabled - apps will appear in main search");
    }

    if config.window_switcher {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-window-switcher",
            "Window Switcher",
            "Switch, tile, and manage open windows",
            vec!["window", "switch", "tile", "focus", "manage", "switcher"],
            BuiltInFeature::WindowSwitcher,
            "🪟",
        ));
        debug!("Added Window Switcher built-in entry");
    }

    // AI Chat is always available
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-ai-chat",
        "AI Chat",
        "Chat with AI assistants (Claude, GPT)",
        vec![
            "ai",
            "chat",
            "assistant",
            "claude",
            "gpt",
            "openai",
            "anthropic",
            "llm",
        ],
        BuiltInFeature::AiChat,
        "🤖",
    ));
    debug!("Added AI Chat built-in entry");

    // Notes is always available
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-notes",
        "Notes",
        "Quick notes and scratchpad",
        vec![
            "notes",
            "note",
            "scratch",
            "scratchpad",
            "memo",
            "markdown",
            "write",
            "text",
        ],
        BuiltInFeature::Notes,
        "📝",
    ));
    debug!("Added Notes built-in entry");

    // Design Gallery is only available in debug builds (developer tool)
    #[cfg(debug_assertions)]
    {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-design-gallery",
            "Design Gallery",
            "Browse separator styles and icon variations",
            vec![
                "design",
                "gallery",
                "separator",
                "icon",
                "style",
                "theme",
                "variations",
            ],
            BuiltInFeature::DesignGallery,
            "🎨",
        ));
        debug!("Added Design Gallery built-in entry");

        // Test Confirmation entry for testing confirmation UI
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-test-confirmation",
            "Test Confirmation",
            "Test the confirmation dialog (dev only)",
            vec!["test", "confirmation", "dev", "debug"],
            BuiltInFeature::SystemAction(SystemActionType::TestConfirmation),
            "🧪",
        ));
        debug!("Added Test Confirmation built-in entry");
    }

    // =========================================================================
    // System Actions
    // =========================================================================

    // Power management
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-empty-trash",
        "Empty Trash",
        "Empty the macOS Trash",
        vec!["empty", "trash", "delete", "clean"],
        BuiltInFeature::SystemAction(SystemActionType::EmptyTrash),
        "🗑️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-lock-screen",
        "Lock Screen",
        "Lock the screen",
        vec!["lock", "screen", "security"],
        BuiltInFeature::SystemAction(SystemActionType::LockScreen),
        "🔒",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-sleep",
        "Sleep",
        "Put the system to sleep",
        vec!["sleep", "suspend", "power"],
        BuiltInFeature::SystemAction(SystemActionType::Sleep),
        "😴",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-restart",
        "Restart",
        "Restart the system",
        vec!["restart", "reboot", "power"],
        BuiltInFeature::SystemAction(SystemActionType::Restart),
        "🔄",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-shut-down",
        "Shut Down",
        "Shut down the system",
        vec!["shut", "down", "shutdown", "power", "off"],
        BuiltInFeature::SystemAction(SystemActionType::ShutDown),
        "⏻",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-log-out",
        "Log Out",
        "Log out the current user",
        vec!["log", "out", "logout", "user"],
        BuiltInFeature::SystemAction(SystemActionType::LogOut),
        "🚪",
    ));

    // UI controls
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-toggle-dark-mode",
        "Toggle Dark Mode",
        "Switch between light and dark appearance",
        vec!["dark", "mode", "light", "appearance", "theme", "toggle"],
        BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode),
        "🌙",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-show-desktop",
        "Show Desktop",
        "Hide all windows to reveal the desktop",
        vec!["show", "desktop", "hide", "windows"],
        BuiltInFeature::SystemAction(SystemActionType::ShowDesktop),
        "🖥️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-mission-control",
        "Mission Control",
        "Show all windows and desktops",
        vec!["mission", "control", "expose", "spaces", "windows"],
        BuiltInFeature::SystemAction(SystemActionType::MissionControl),
        "🪟",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-launchpad",
        "Launchpad",
        "Open Launchpad to show all applications",
        vec!["launchpad", "apps", "applications"],
        BuiltInFeature::SystemAction(SystemActionType::Launchpad),
        "🚀",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-force-quit",
        "Force Quit Apps",
        "Select and force quit running applications",
        vec![
            "force",
            "quit",
            "kill",
            "apps",
            "unresponsive",
            "terminate",
            "stop",
        ],
        BuiltInFeature::SystemAction(SystemActionType::ForceQuitApps),
        "⚠️",
    ));

    // Volume controls (preset levels)
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-0",
        "Volume 0%",
        "Set system volume to 0% (mute)",
        vec!["volume", "mute", "0", "percent", "zero", "off"],
        BuiltInFeature::SystemAction(SystemActionType::Volume0),
        "🔇",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-25",
        "Volume 25%",
        "Set system volume to 25%",
        vec!["volume", "25", "percent", "low", "quiet"],
        BuiltInFeature::SystemAction(SystemActionType::Volume25),
        "🔈",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-50",
        "Volume 50%",
        "Set system volume to 50%",
        vec!["volume", "50", "percent", "half", "medium"],
        BuiltInFeature::SystemAction(SystemActionType::Volume50),
        "🔉",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-75",
        "Volume 75%",
        "Set system volume to 75%",
        vec!["volume", "75", "percent", "high", "loud"],
        BuiltInFeature::SystemAction(SystemActionType::Volume75),
        "🔉",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-100",
        "Volume 100%",
        "Set system volume to 100% (max)",
        vec!["volume", "100", "percent", "max", "full"],
        BuiltInFeature::SystemAction(SystemActionType::Volume100),
        "🔊",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-mute",
        "Toggle Mute",
        "Toggle audio mute",
        vec!["mute", "unmute", "volume", "sound", "audio", "toggle"],
        BuiltInFeature::SystemAction(SystemActionType::VolumeMute),
        "🔇",
    ));

    // App control
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quit-script-kit",
        "Quit Script Kit",
        "Quit the Script Kit application",
        vec!["quit", "exit", "close", "script", "kit", "app"],
        BuiltInFeature::SystemAction(SystemActionType::QuitScriptKit),
        "🚪",
    ));

    // System utilities
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-toggle-dnd",
        "Toggle Do Not Disturb",
        "Toggle Focus/Do Not Disturb mode",
        vec![
            "do",
            "not",
            "disturb",
            "dnd",
            "focus",
            "notifications",
            "toggle",
        ],
        BuiltInFeature::SystemAction(SystemActionType::ToggleDoNotDisturb),
        "🔕",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-screen-saver",
        "Start Screen Saver",
        "Activate the screen saver",
        vec!["screen", "saver", "screensaver"],
        BuiltInFeature::SystemAction(SystemActionType::StartScreenSaver),
        "🖼️",
    ));

    // System Preferences
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-system-preferences",
        "Open System Settings",
        "Open System Settings (System Preferences)",
        vec!["system", "settings", "preferences", "prefs"],
        BuiltInFeature::SystemAction(SystemActionType::OpenSystemPreferences),
        "⚙️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-privacy-settings",
        "Privacy & Security Settings",
        "Open Privacy & Security settings",
        vec!["privacy", "security", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenPrivacySettings),
        "🔐",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-display-settings",
        "Display Settings",
        "Open Display settings",
        vec!["display", "monitor", "screen", "resolution", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenDisplaySettings),
        "🖥️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-sound-settings",
        "Sound Settings",
        "Open Sound settings",
        vec!["sound", "audio", "volume", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenSoundSettings),
        "🔊",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-network-settings",
        "Network Settings",
        "Open Network settings",
        vec!["network", "wifi", "ethernet", "internet", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenNetworkSettings),
        "📡",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-keyboard-settings",
        "Keyboard Settings",
        "Open Keyboard settings",
        vec!["keyboard", "shortcuts", "input", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenKeyboardSettings),
        "⌨️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-bluetooth-settings",
        "Bluetooth Settings",
        "Open Bluetooth settings",
        vec!["bluetooth", "wireless", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenBluetoothSettings),
        "🔵",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-notifications-settings",
        "Notification Settings",
        "Open Notifications settings",
        vec!["notifications", "alerts", "banners", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenNotificationsSettings),
        "🔔",
    ));

    // NOTE: Window Actions removed - now handled by window-management extension
    // SDK tileWindow() function still works via protocol messages in execute_script.rs

    // =========================================================================
    // Notes Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-note",
        "New Note",
        "Create a new note",
        vec!["new", "note", "create"],
        BuiltInFeature::NotesCommand(NotesCommandType::NewNote),
        "📝",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-search-notes",
        "Search Notes",
        "Search through your notes",
        vec!["search", "notes", "find"],
        BuiltInFeature::NotesCommand(NotesCommandType::SearchNotes),
        "🔍",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quick-capture",
        "Quick Capture",
        "Quickly capture a note",
        vec!["quick", "capture", "note", "fast"],
        BuiltInFeature::NotesCommand(NotesCommandType::QuickCapture),
        "⚡",
    ));

    // =========================================================================
    // AI Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-conversation",
        "New AI Conversation",
        "Start a new AI conversation",
        vec!["new", "conversation", "chat", "ai"],
        BuiltInFeature::AiCommand(AiCommandType::NewConversation),
        "💬",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-send-screen-to-ai",
        "Send Screen to AI Chat",
        "Capture the screen and send to AI Chat",
        vec![
            "send",
            "screen",
            "screenshot",
            "ai",
            "chat",
            "capture",
            "image",
        ],
        BuiltInFeature::AiCommand(AiCommandType::SendScreenToAi),
        "📸",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-send-window-to-ai",
        "Send Focused Window to AI Chat",
        "Capture the focused window and send to AI Chat",
        vec![
            "send",
            "window",
            "focused",
            "ai",
            "chat",
            "capture",
            "screenshot",
        ],
        BuiltInFeature::AiCommand(AiCommandType::SendFocusedWindowToAi),
        "🪟",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-send-selected-text-to-ai",
        "Send Selected Text to AI Chat",
        "Send the currently selected text to AI Chat",
        vec![
            "send",
            "selected",
            "text",
            "selection",
            "ai",
            "chat",
            "copy",
        ],
        BuiltInFeature::AiCommand(AiCommandType::SendSelectedTextToAi),
        "📝",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-send-browser-tab-to-ai",
        "Send Focused Browser Tab to AI Chat",
        "Send the current browser tab URL to AI Chat",
        vec![
            "send", "browser", "tab", "url", "safari", "chrome", "ai", "chat", "web",
        ],
        BuiltInFeature::AiCommand(AiCommandType::SendBrowserTabToAi),
        "🌐",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-send-screen-area-to-ai",
        "Send Screen Area to AI Chat",
        "Select a screen area and send to AI Chat",
        vec![
            "send",
            "screen",
            "area",
            "region",
            "selection",
            "ai",
            "chat",
            "capture",
        ],
        BuiltInFeature::AiCommand(AiCommandType::SendScreenAreaToAi),
        "⬚",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-create-ai-preset",
        "Create AI Chat Preset",
        "Create a new AI chat preset with custom settings",
        vec!["create", "ai", "preset", "template", "chat", "new", "save"],
        BuiltInFeature::AiCommand(AiCommandType::CreateAiPreset),
        "➕",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-import-ai-presets",
        "Import AI Chat Presets",
        "Import AI chat presets from a file",
        vec!["import", "ai", "preset", "presets", "chat", "load", "file"],
        BuiltInFeature::AiCommand(AiCommandType::ImportAiPresets),
        "📥",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-search-ai-presets",
        "Search AI Chat Presets",
        "Search through saved AI chat presets",
        vec![
            "search",
            "ai",
            "preset",
            "presets",
            "chat",
            "find",
            "templates",
        ],
        BuiltInFeature::AiCommand(AiCommandType::SearchAiPresets),
        "🔍",
    ));

    // =========================================================================
    // Script Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-script",
        "New Script",
        "Create a new Script Kit script",
        vec!["new", "script", "create", "code"],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewScript),
        "📜",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-extension",
        "New Extension",
        "Create a new Script Kit extension",
        vec!["new", "extension", "create", "snippet"],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewExtension),
        "✨",
    ));

    // =========================================================================
    // Permission Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-check-permissions",
        "Check Permissions",
        "Check all required macOS permissions",
        vec!["check", "permissions", "accessibility", "privacy"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::CheckPermissions),
        "✅",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-request-accessibility",
        "Request Accessibility Permission",
        "Request accessibility permission for Script Kit",
        vec!["request", "accessibility", "permission"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::RequestAccessibility),
        "🔑",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-accessibility-settings",
        "Open Accessibility Settings",
        "Open Accessibility settings in System Preferences",
        vec!["accessibility", "settings", "permission", "open"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::OpenAccessibilitySettings),
        "♿",
    ));

    // =========================================================================
    // Frecency/Suggested Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-clear-suggested",
        "Clear Suggested",
        "Clear all suggested/recently used items",
        vec![
            "clear",
            "suggested",
            "recent",
            "frecency",
            "reset",
            "history",
        ],
        BuiltInFeature::FrecencyCommand(FrecencyCommandType::ClearSuggested),
        "🧹",
    ));

    // =========================================================================
    // Settings Commands
    // =========================================================================

    // Only show reset if there are custom positions
    if crate::window_state::has_custom_positions() {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-reset-window-positions",
            "Reset Window Positions",
            "Restore all windows to default positions",
            vec![
                "reset", "window", "position", "default", "restore", "layout", "location",
            ],
            BuiltInFeature::SettingsCommand(SettingsCommandType::ResetWindowPositions),
            "🔄",
        ));
    }

    // API Key Configuration
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-configure-vercel-api",
        "Configure Vercel AI Gateway",
        "Set up your Vercel AI Gateway API key for AI Chat",
        vec![
            "vercel",
            "api",
            "key",
            "gateway",
            "ai",
            "configure",
            "setup",
            "config",
            "settings",
        ],
        BuiltInFeature::SettingsCommand(SettingsCommandType::ConfigureVercelApiKey),
        "🔑",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-configure-openai-api",
        "Configure OpenAI API Key",
        "Set up your OpenAI API key for AI Chat",
        vec![
            "openai",
            "api",
            "key",
            "gpt",
            "ai",
            "configure",
            "setup",
            "config",
            "settings",
        ],
        BuiltInFeature::SettingsCommand(SettingsCommandType::ConfigureOpenAiApiKey),
        "🔑",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-configure-anthropic-api",
        "Configure Anthropic API Key",
        "Set up your Anthropic API key for AI Chat",
        vec![
            "anthropic",
            "api",
            "key",
            "claude",
            "ai",
            "configure",
            "setup",
            "config",
            "settings",
        ],
        BuiltInFeature::SettingsCommand(SettingsCommandType::ConfigureAnthropicApiKey),
        "🔑",
    ));

    // =========================================================================
    // Utility Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-scratch-pad",
        "Scratch Pad",
        "Quick editor for notes and code - auto-saves to disk",
        vec![
            "scratch",
            "pad",
            "scratchpad",
            "notes",
            "editor",
            "write",
            "text",
            "quick",
            "jot",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::ScratchPad),
        "📝",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quick-terminal",
        "Quick Terminal",
        "Open a terminal for running quick commands",
        vec![
            "terminal", "term", "shell", "bash", "zsh", "command", "quick", "console", "cli",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::QuickTerminal),
        "💻",
    ));

    // =========================================================================
    // File Search (Directory Navigation)
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-file-search",
        "Search Files",
        "Browse directories and search for files",
        vec![
            "file",
            "search",
            "find",
            "directory",
            "folder",
            "browse",
            "navigate",
            "path",
            "open",
            "explorer",
        ],
        BuiltInFeature::FileSearch,
        "folder-search",
    ));

    debug!(count = entries.len(), "Built-in entries loaded");
    entries
}

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

#[cfg(test)]
mod tests {
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
        assert!(entries.iter().any(|e| e.id == "builtin-new-note"));
        assert!(entries.iter().any(|e| e.id == "builtin-search-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-quick-capture"));
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
    fn test_file_search_feature_equality() {
        assert_eq!(BuiltInFeature::FileSearch, BuiltInFeature::FileSearch);
        assert_ne!(BuiltInFeature::FileSearch, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::FileSearch, BuiltInFeature::Notes);
    }
}

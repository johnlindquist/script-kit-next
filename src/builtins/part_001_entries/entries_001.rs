{
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
        "Toggle system audio mute on or off",
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
        "Toggle Focus / Do Not Disturb mode on or off",
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
}

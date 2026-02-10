{
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
        "Open setup for the Vercel AI Gateway API key used by AI Chat",
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
        "Open setup for the OpenAI API key used by AI Chat",
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
        "Open setup for the Anthropic API key used by AI Chat",
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

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-choose-theme",
        "Choose Theme",
        "Open the theme picker and apply a color theme with live preview",
        vec!["theme", "appearance", "color", "dark", "light", "scheme"],
        BuiltInFeature::SettingsCommand(SettingsCommandType::ChooseTheme),
        "🎨",
    ));

    // =========================================================================
    // Utility Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-scratch-pad",
        "Scratch Pad",
        "Open a scratch pad editor for notes and code (auto-saves to disk)",
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
        "Open a quick terminal for running shell commands",
        vec![
            "terminal", "term", "shell", "bash", "zsh", "command", "quick", "console", "cli",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::QuickTerminal),
        "💻",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-process-manager",
        "Process Manager",
        "Inspect running scripts and copy their process details",
        vec![
            "process", "running", "scripts", "jobs", "pid", "inspect", "manage", "kill",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::ProcessManager),
        "activity",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-stop-all-processes",
        "Stop All Running Scripts",
        "Terminate every active Script Kit child process",
        vec![
            "process",
            "running",
            "scripts",
            "stop",
            "kill",
            "terminate",
            "jobs",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::StopAllProcesses),
        "square-stop",
    ));

    // =========================================================================
    // Kit Store Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-browse-kit-store",
        "Browse Kit Store",
        "Browse available kits from the Kit Store",
        vec![
            "kit",
            "store",
            "browse",
            "search",
            "discover",
            "extensions",
        ],
        BuiltInFeature::KitStoreCommand(KitStoreCommandType::BrowseKits),
        "search",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-manage-installed-kits",
        "Manage Installed Kits",
        "View and manage kits already installed from the Kit Store",
        vec![
            "kit",
            "store",
            "installed",
            "manage",
            "extensions",
            "packages",
        ],
        BuiltInFeature::KitStoreCommand(KitStoreCommandType::InstalledKits),
        "package",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-update-all-kits",
        "Update All Kits",
        "Update every installed kit to the latest version",
        vec![
            "kit",
            "store",
            "update",
            "upgrade",
            "refresh",
            "all",
            "extensions",
        ],
        BuiltInFeature::KitStoreCommand(KitStoreCommandType::UpdateAllKits),
        "refresh-cw",
    ));

    // =========================================================================
    // File Search (Directory Navigation)
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-file-search",
        "Search Files",
        "Browse directories, search files, and open results",
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

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-webcam",
        "Webcam",
        "Open the webcam prompt and capture a photo",
        vec!["webcam", "camera", "capture", "photo", "image"],
        BuiltInFeature::Webcam,
        "📸",
    ));

}

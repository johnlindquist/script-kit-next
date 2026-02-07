{
    // Notes Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-open-notes",
        "Open Notes",
        "Open the Notes window",
        vec!["open", "notes", "window", "note"],
        BuiltInFeature::NotesCommand(NotesCommandType::OpenNotes),
        "📝",
    ));

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
        "Capture a new note without opening the full Notes window",
        vec!["quick", "capture", "note", "fast"],
        BuiltInFeature::NotesCommand(NotesCommandType::QuickCapture),
        "⚡",
    ));

    // =========================================================================
    // AI Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-open-ai",
        "Open AI Chat",
        "Open the AI Chat window",
        vec!["open", "ai", "chat", "assistant", "window"],
        BuiltInFeature::AiCommand(AiCommandType::OpenAi),
        "🤖",
    ));

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
        "Capture the full screen and send it to AI Chat",
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
        "Capture the focused window and send it to AI Chat",
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

    // =========================================================================
    // Script Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-script",
        "New Script (Template)",
        "Create a new Script Kit script from a guided starter template",
        vec![
            "new",
            "script",
            "create",
            "template",
            "starter",
            "boilerplate",
            "scaffold",
            "code",
        ],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewScript),
        "➕",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-extension",
        "New Scriptlet Bundle",
        "Create a new extension bundle with YAML frontmatter and scriptlet examples",
        vec![
            "new",
            "scriptlet",
            "bundle",
            "extension",
            "frontmatter",
            "yaml",
            "snippet",
            "create",
        ],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewExtension),
        "✨",
    ));

    // =========================================================================
    // Permission Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-check-permissions",
        "Check Permissions",
        "Run a check for all required macOS permissions",
        vec!["check", "permissions", "accessibility", "privacy"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::CheckPermissions),
        "✅",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-request-accessibility",
        "Request Accessibility Permission",
        "Request accessibility permission for Script Kit in System Settings",
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
        "Clear all items from Suggested / Recently Used",
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
}

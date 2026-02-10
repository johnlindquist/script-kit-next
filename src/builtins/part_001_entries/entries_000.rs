{
    if config.clipboard_history {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-clipboard-history",
            "Clipboard History",
            "Open clipboard history to view, search, and reuse copied items",
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
            "Open window switcher to focus, tile, and manage open windows",
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
        "Open AI Chat with Claude, GPT, and other configured assistants",
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

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-favorites",
        "Favorites",
        "Open your starred scripts and shortcuts",
        vec![
            "favorites",
            "favorite",
            "starred",
            "star",
            "pinned",
            "saved",
        ],
        BuiltInFeature::Favorites,
        "⭐",
    ));
    debug!("Added Favorites built-in entry");

    // Notes is always available
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-notes",
        "Notes",
        "Open quick notes and a scratchpad editor",
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

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-emoji-picker",
        "Emoji Picker",
        "Pick an emoji from the built-in list and copy it to the clipboard",
        vec![
            "emoji",
            "picker",
            "symbols",
            "unicode",
            "copy",
            "clipboard",
        ],
        BuiltInFeature::EmojiPicker,
        "😀",
    ));
    debug!("Added Emoji Picker built-in entry");

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quicklinks",
        "Quicklinks",
        "Manage quick links and open URLs with optional {query} expansion",
        vec![
            "quicklinks",
            "quicklink",
            "link",
            "url",
            "bookmark",
            "open",
            "search",
        ],
        BuiltInFeature::Quicklinks,
        "🔗",
    ));
    debug!("Added Quicklinks built-in entry");

    // Design Gallery is only available in debug builds (developer tool)
    #[cfg(debug_assertions)]
    {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-design-gallery",
            "Design Gallery",
            "Open the design gallery to browse separator styles and icon variations",
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
            "Open the confirmation dialog test tool (dev only)",
            vec!["test", "confirmation", "dev", "debug"],
            BuiltInFeature::SystemAction(SystemActionType::TestConfirmation),
            "🧪",
        ));
        debug!("Added Test Confirmation built-in entry");
    }

    // =========================================================================
}

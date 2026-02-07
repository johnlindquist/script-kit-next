#[test]
fn test_get_grouped_results_selection_priority_with_frecency() {
    // This test verifies the SELECTION behavior, not just grouping.
    //
    // Bug: When user opens Script Kit, the FIRST SELECTABLE item should be
    // the most recently used item (from SUGGESTED), not the first item in MAIN.
    //
    // The grouped list structure determines what gets selected initially.
    // With frecency, the first Item (not SectionHeader) should be the
    // frecency script, which means selected_index=0 should point to
    // the frecency script when we skip headers.

    let scripts = wrap_scripts(vec![
        Script {
            name: "alpha-script".to_string(),
            path: PathBuf::from("/alpha-script.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "zebra-script".to_string(),
            path: PathBuf::from("/zebra-script.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins = create_test_builtins(); // Clipboard History, App Launcher
    let apps: Vec<AppInfo> = vec![];

    let mut frecency_store = FrecencyStore::new();
    frecency_store.record_use("/zebra-script.ts"); // Give frecency to zebra

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Find the first Item (not SectionHeader) - this is what gets selected
    let first_selectable_idx = grouped
        .iter()
        .find_map(|item| {
            if let GroupedListItem::Item(idx) = item {
                Some(*idx)
            } else {
                None
            }
        })
        .expect("Should have at least one selectable item");

    let first_result = &results[first_selectable_idx];

    // The first selectable item MUST be the frecency script
    // NOT Clipboard History (which would be first alphabetically in MAIN)
    assert_eq!(
        first_result.name(),
        "zebra-script",
        "First selectable item should be the frecency script 'zebra-script', got '{}'. \
             This bug causes Clipboard History to appear first regardless of user's frecency.",
        first_result.name()
    );

    // Verify the structure explicitly
    // grouped[0] = SectionHeader("SUGGESTED")
    // grouped[1] = Item(zebra-script) <- THIS should be first selection
    // grouped[2] = SectionHeader("SCRIPTS") or next type-based section
    // grouped[3+] = Other items sorted alphabetically within their sections

    let grouped_names: Vec<String> = grouped
        .iter()
        .map(|item| match item {
            GroupedListItem::SectionHeader(s, _) => {
                let name = s.split(" · ").next().unwrap_or(s);
                format!("[{}]", name)
            }
            GroupedListItem::Item(idx) => results[*idx].name().to_string(),
        })
        .collect();

    // First 3 items should be: SUGGESTED header, frecency item, MAIN header (kit-based section)
    // Scripts without kit_name default to "main" kit
    assert_eq!(
        &grouped_names[..3],
        &["[SUGGESTED]", "zebra-script", "[MAIN]"],
        "First 3 items should be: SUGGESTED header, frecency item, MAIN header. Got: {:?}",
        grouped_names
    );
}

#[test]
fn test_get_grouped_results_no_frecency_items_in_type_sections() {
    // This test verifies the type-based sectioning behavior.
    //
    // When there's NO frecency data, items are grouped into type-based sections:
    // - SCRIPTS: user scripts
    // - COMMANDS: builtins
    //
    // Items are sorted alphabetically within each section.
    // Sections appear in order: SCRIPTS, SCRIPTLETS, COMMANDS, APPS

    let scripts = wrap_scripts(vec![
        Script {
            name: "alpha-script".to_string(),
            path: PathBuf::from("/alpha-script.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "zebra-script".to_string(),
            path: PathBuf::from("/zebra-script.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins = create_test_builtins(); // Clipboard History, App Launcher
    let apps: Vec<AppInfo> = vec![];

    // No frecency - fresh start
    let frecency_store = FrecencyStore::new();

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // With no frecency and empty frecency store, items matching DEFAULT_SUGGESTED_ITEMS
    // appear in SUGGESTED section, remaining items go to their type-based sections.
    let grouped_names: Vec<String> = grouped
        .iter()
        .map(|item| match item {
            GroupedListItem::SectionHeader(s, _) => {
                let name = s.split(" · ").next().unwrap_or(s);
                format!("[{}]", name)
            }
            GroupedListItem::Item(idx) => results[*idx].name().to_string(),
        })
        .collect();

    // With default suggestions enabled and empty frecency, "Clipboard History" (which is in
    // DEFAULT_SUGGESTED_ITEMS) appears in SUGGESTED. App Launcher is not in defaults, stays in COMMANDS.
    // Expected structure:
    // [SUGGESTED], Clipboard History (matches DEFAULT_SUGGESTED_ITEMS)
    // [MAIN], alpha-script, zebra-script
    // [COMMANDS], App Launcher
    assert_eq!(
        grouped_names[0], "[SUGGESTED]",
        "First item should be SUGGESTED header when frecency is empty but defaults match. Got: {:?}",
        grouped_names
    );

    assert_eq!(
        grouped_names,
        vec![
            "[SUGGESTED]",
            "Clipboard History",
            "[MAIN]",
            "alpha-script",
            "zebra-script",
            "[COMMANDS]",
            "App Launcher",
        ],
        "Items matching DEFAULT_SUGGESTED_ITEMS should be in SUGGESTED, rest in type-based sections. Got: {:?}",
        grouped_names
    );
}

#[test]
fn test_get_grouped_results_default_suggestions_for_new_users() {
    // When frecency store is empty (new user), items matching DEFAULT_SUGGESTED_ITEMS
    // should appear in the SUGGESTED section to help users discover features.
    use crate::builtins::{BuiltInFeature, BuiltInGroup};

    // Create builtins that match some of the DEFAULT_SUGGESTED_ITEMS
    let builtins = vec![
        BuiltInEntry {
            id: "builtin-ai-chat".to_string(),
            name: "AI Chat".to_string(),
            description: "Chat with AI assistants".to_string(),
            keywords: vec!["ai".to_string(), "chat".to_string()],
            feature: BuiltInFeature::AiChat,
            icon: Some("🤖".to_string()),
            group: BuiltInGroup::Core,
        },
        BuiltInEntry {
            id: "builtin-notes".to_string(),
            name: "Notes".to_string(),
            description: "Quick notes".to_string(),
            keywords: vec!["notes".to_string()],
            feature: BuiltInFeature::Notes,
            icon: Some("📝".to_string()),
            group: BuiltInGroup::Core,
        },
        BuiltInEntry {
            id: "builtin-clipboard-history".to_string(),
            name: "Clipboard History".to_string(),
            description: "View clipboard history".to_string(),
            keywords: vec!["clipboard".to_string()],
            feature: BuiltInFeature::ClipboardHistory,
            icon: Some("📋".to_string()),
            group: BuiltInGroup::Core,
        },
        BuiltInEntry {
            id: "builtin-other".to_string(),
            name: "Some Other Command".to_string(),
            description: "Not in defaults".to_string(),
            keywords: vec!["other".to_string()],
            feature: BuiltInFeature::AppLauncher,
            icon: Some("🔧".to_string()),
            group: BuiltInGroup::Core,
        },
    ];

    let scripts: Vec<Arc<Script>> = vec![];
    let scriptlets: Vec<Arc<Scriptlet>> = vec![];
    let apps: Vec<AppInfo> = vec![];
    let frecency_store = FrecencyStore::new(); // Empty frecency

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    let grouped_names: Vec<String> = grouped
        .iter()
        .map(|item| match item {
            GroupedListItem::SectionHeader(s, _) => {
                let name = s.split(" · ").next().unwrap_or(s);
                format!("[{}]", name)
            }
            GroupedListItem::Item(idx) => results[*idx].name().to_string(),
        })
        .collect();

    // Default suggestions should appear in order from DEFAULT_SUGGESTED_ITEMS
    // AI Chat, Notes, Clipboard History are in defaults (in that order)
    // Some Other Command is NOT in defaults, goes to COMMANDS
    assert_eq!(
        grouped_names,
        vec![
            "[SUGGESTED]",
            "AI Chat",
            "Notes",
            "Clipboard History",
            "[COMMANDS]",
            "Some Other Command",
        ],
        "Default suggested items should appear in SUGGESTED for new users, others in COMMANDS. Got: {:?}",
        grouped_names
    );
}

#[test]
fn test_get_grouped_results_no_default_suggestions_when_frecency_exists() {
    // When frecency store has data, don't use default suggestions
    use crate::builtins::{BuiltInFeature, BuiltInGroup};

    let builtins = vec![
        BuiltInEntry {
            id: "builtin-ai-chat".to_string(),
            name: "AI Chat".to_string(),
            description: "Chat with AI assistants".to_string(),
            keywords: vec!["ai".to_string()],
            feature: BuiltInFeature::AiChat,
            icon: Some("🤖".to_string()),
            group: BuiltInGroup::Core,
        },
        BuiltInEntry {
            id: "builtin-notes".to_string(),
            name: "Notes".to_string(),
            description: "Quick notes".to_string(),
            keywords: vec!["notes".to_string()],
            feature: BuiltInFeature::Notes,
            icon: Some("📝".to_string()),
            group: BuiltInGroup::Core,
        },
    ];

    let scripts: Vec<Arc<Script>> = vec![];
    let scriptlets: Vec<Arc<Scriptlet>> = vec![];
    let apps: Vec<AppInfo> = vec![];

    // Create frecency store with some data (user is not new)
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!(
        "frecency_test_defaults_{}.json",
        uuid::Uuid::new_v4()
    ));
    let mut frecency_store = FrecencyStore::with_path(temp_path.clone());
    frecency_store.record_use("builtin:Notes"); // Record usage for Notes

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    let grouped_names: Vec<String> = grouped
        .iter()
        .map(|item| match item {
            GroupedListItem::SectionHeader(s, _) => {
                let name = s.split(" · ").next().unwrap_or(s);
                format!("[{}]", name)
            }
            GroupedListItem::Item(idx) => results[*idx].name().to_string(),
        })
        .collect();

    // With frecency data, SUGGESTED shows frecency-based items (Notes has usage)
    // AI Chat has no usage, goes to COMMANDS
    assert_eq!(
        grouped_names,
        vec!["[SUGGESTED]", "Notes", "[COMMANDS]", "AI Chat",],
        "With frecency data, should use frecency-based suggestions, not defaults. Got: {:?}",
        grouped_names
    );

    // Cleanup temp file
    let _ = std::fs::remove_file(temp_path);
}

#[test]
fn test_get_grouped_results_empty_inputs() {
    let scripts: Vec<Arc<Script>> = wrap_scripts(vec![]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<AppInfo> = vec![];
    let frecency_store = FrecencyStore::new();

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Both should be empty when no inputs
    assert!(results.is_empty());
    assert!(grouped.is_empty());
}

#[test]
fn test_get_grouped_results_items_reference_correct_indices() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "first".to_string(),
            path: PathBuf::from("/first.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "second".to_string(),
            path: PathBuf::from("/second.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<AppInfo> = vec![];
    let frecency_store = FrecencyStore::new();

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // All Item indices should be valid indices into results
    for item in &grouped {
        if let GroupedListItem::Item(idx) = item {
            assert!(
                *idx < results.len(),
                "Index {} out of bounds for results len {}",
                idx,
                results.len()
            );
        }
    }
}


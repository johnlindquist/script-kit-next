//! Script Kit environment setup and initialization.
//!
//! Ensures ~/.scriptkit exists with required directories and starter files.
//! The path can be overridden via the SK_PATH environment variable.
//! Idempotent: user-owned files are never overwritten; app-owned files may be refreshed.

// --- merged from part_000.rs ---
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument, warn};
/// Embedded config template (included at compile time)
const EMBEDDED_CONFIG_TEMPLATE: &str = include_str!("../../kit-init/config-template.ts");
/// Embedded SDK content (included at compile time)
const EMBEDDED_SDK: &str = include_str!("../../scripts/kit-sdk.ts");
/// Optional theme example (included at compile time)
const EMBEDDED_THEME_EXAMPLE: &str = include_str!("../../kit-init/theme.example.json");
/// Embedded package.json template for user's kit directory
/// The "type": "module" enables top-level await in all .ts scripts
const EMBEDDED_PACKAGE_JSON: &str = r#"{
  "name": "@scriptkit/kit",
  "type": "module",
  "private": true,
  "scripts": {
    "typecheck": "tsc --noEmit"
  }
}
"#;
/// Embedded GUIDE.md comprehensive user guide
const EMBEDDED_GUIDE_MD: &str = include_str!("../../kit-init/GUIDE.md");
/// Embedded CleanShot X extension (built-in extension that ships with the app)
const EMBEDDED_CLEANSHOT_EXTENSION: &str =
    include_str!("../../kit-init/extensions/cleanshot/main.md");
/// Embedded CleanShot X shared actions (built-in actions for all cleanshot scriptlets)
const EMBEDDED_CLEANSHOT_ACTIONS: &str =
    include_str!("../../kit-init/extensions/cleanshot/main.actions.md");
/// Embedded 1Password extension (built-in extension that ships with the app)
const EMBEDDED_1PASSWORD_EXTENSION: &str =
    include_str!("../../kit-init/extensions/1password/main.md");
/// Embedded Quick Links extension (built-in extension that ships with the app)
const EMBEDDED_QUICKLINKS_EXTENSION: &str =
    include_str!("../../kit-init/extensions/quicklinks/main.md");
/// Embedded Quick Links shared actions (built-in actions for all quicklinks scriptlets)
const EMBEDDED_QUICKLINKS_ACTIONS: &str =
    include_str!("../../kit-init/extensions/quicklinks/main.actions.md");
/// Embedded Window Management extension (built-in extension that ships with the app)
const EMBEDDED_WINDOW_MANAGEMENT_EXTENSION: &str =
    include_str!("../../kit-init/extensions/window-management/main.md");
/// Embedded AI Text Tools extension (built-in extension that ships with the app)
const EMBEDDED_AI_TEXT_TOOLS_EXTENSION: &str =
    include_str!("../../kit-init/extensions/ai-text-tools/main.md");
/// Embedded Examples extension - main scriptlet examples (built-in extension that ships with the app)
const EMBEDDED_EXAMPLES_MAIN: &str = include_str!("../../kit-init/extensions/examples/main.md");
/// Embedded Examples extension - advanced scriptlet examples (built-in extension that ships with the app)
const EMBEDDED_EXAMPLES_ADVANCED: &str =
    include_str!("../../kit-init/extensions/examples/advanced.md");
/// Embedded Examples extension - howto guide (built-in extension that ships with the app)
const EMBEDDED_EXAMPLES_HOWTO: &str = include_str!("../../kit-init/extensions/examples/howto.md");
/// Root-level CLAUDE.md for the ~/.scriptkit workspace (the harness cwd)
const EMBEDDED_ROOT_CLAUDE_MD: &str = include_str!("../../kit-init/ROOT_CLAUDE.md");
/// Root-level AGENTS.md SDK reference for the ~/.scriptkit workspace
const EMBEDDED_ROOT_AGENTS_MD: &str = include_str!("../../kit-init/ROOT_AGENTS.md");
/// Skills README
const EMBEDDED_SKILLS_README: &str = include_str!("../../kit-init/skills/README.md");
/// Skill: script authoring
const EMBEDDED_SKILL_SCRIPT_AUTHORING: &str =
    include_str!("../../kit-init/skills/script-authoring/SKILL.md");
/// Skill: scriptlets
const EMBEDDED_SKILL_SCRIPTLETS: &str = include_str!("../../kit-init/skills/scriptlets/SKILL.md");
/// Skill: config & theming
const EMBEDDED_SKILL_CONFIG: &str = include_str!("../../kit-init/skills/config/SKILL.md");
/// Skill: troubleshooting
const EMBEDDED_SKILL_TROUBLESHOOTING: &str =
    include_str!("../../kit-init/skills/troubleshooting/SKILL.md");
/// Example script: hello-world
const EMBEDDED_EXAMPLE_HELLO_WORLD: &str =
    include_str!("../../kit-init/examples/scripts/hello-world.ts");
/// Example script: choose-from-list
const EMBEDDED_EXAMPLE_CHOOSE_FROM_LIST: &str =
    include_str!("../../kit-init/examples/scripts/choose-from-list.ts");
/// Example script: clipboard-transform
const EMBEDDED_EXAMPLE_CLIPBOARD_TRANSFORM: &str =
    include_str!("../../kit-init/examples/scripts/clipboard-transform.ts");
/// Example script: path-picker
const EMBEDDED_EXAMPLE_PATH_PICKER: &str =
    include_str!("../../kit-init/examples/scripts/path-picker.ts");
/// Examples README
const EMBEDDED_EXAMPLES_README: &str = include_str!("../../kit-init/examples/README.md");
/// Skill: agent authoring
const EMBEDDED_SKILL_AGENTS: &str = include_str!("../../kit-init/skills/agents/SKILL.md");
/// Example agent: review-pr
const EMBEDDED_EXAMPLE_AGENT_REVIEW_PR: &str =
    include_str!("../../kit-init/examples/agents/review-pr.claude.md");
/// Example agent: plan-feature
const EMBEDDED_EXAMPLE_AGENT_PLAN_FEATURE: &str =
    include_str!("../../kit-init/examples/agents/plan-feature.i.gemini.md");
// --- merged from part_001.rs ---
// (Old kit-level agents doc constant removed — canonical docs live at root level)
// --- merged from part_002.rs ---
// (Old kit-level claude doc constant removed — canonical docs live at root level)
/// Environment variable to override the default ~/.scriptkit path
pub const SK_PATH_ENV: &str = "SK_PATH";
/// Result of setup process
#[derive(Debug)]
pub struct SetupResult {
    /// Whether ~/.scriptkit didn't exist before this run
    pub is_fresh_install: bool,
    /// Path to ~/.scriptkit (or SK_PATH override, or fallback if home dir couldn't be resolved)
    pub kit_path: PathBuf,
    /// Whether bun looks discoverable on this machine
    pub bun_available: bool,
    /// Any warnings encountered during setup
    pub warnings: Vec<String>,
}
/// Get the kit path, respecting SK_PATH environment variable
///
/// Priority:
/// 1. SK_PATH environment variable (if set)
/// 2. ~/.scriptkit (default)
/// 3. Temp directory fallback (if home dir unavailable)
pub fn get_kit_path() -> PathBuf {
    // Check for SK_PATH override first
    if let Ok(sk_path) = std::env::var(SK_PATH_ENV) {
        if let Ok(expanded) = shellexpand::full(&sk_path) {
            return PathBuf::from(expanded.as_ref());
        }
        return PathBuf::from(shellexpand::tilde(&sk_path).as_ref());
    }

    // Default: ~/.scriptkit
    match dirs::home_dir() {
        Some(home) => home.join(".scriptkit"),
        None => std::env::temp_dir().join("script-kit"),
    }
}
/// Migrate from legacy ~/.kenv to new ~/.scriptkit structure
///
/// This function handles one-time migration from the old directory structure:
/// - Moves ~/.kenv contents to ~/.scriptkit
/// - Moves ~/.kenv/scripts to ~/.scriptkit/kit/main/scripts
/// - Moves ~/.kenv/scriptlets to ~/.scriptkit/kit/main/extensions
/// - Creates a symlink ~/.kenv -> ~/.scriptkit for backwards compatibility
///
/// Returns true if migration was performed, false if not needed
#[instrument(level = "info", name = "migrate_from_kenv")]
pub fn migrate_from_kenv() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    let old_kenv = home.join(".kenv");
    let new_scriptkit = home.join(".scriptkit");

    // Only migrate if old path exists and new path doesn't
    if !old_kenv.exists() || new_scriptkit.exists() {
        return false;
    }

    info!(
        old_path = %old_kenv.display(),
        new_path = %new_scriptkit.display(),
        "Migrating from ~/.kenv to ~/.scriptkit"
    );

    // Create the new structure (under kit/ subdirectory)
    let main_scripts = new_scriptkit.join("kit").join("main").join("scripts");
    let main_extensions = new_scriptkit.join("kit").join("main").join("extensions");

    if let Err(e) = fs::create_dir_all(&main_scripts) {
        warn!(error = %e, "Failed to create main/scripts directory");
        return false;
    }

    if let Err(e) = fs::create_dir_all(&main_extensions) {
        warn!(error = %e, "Failed to create main/extensions directory");
        return false;
    }

    // Move scripts from ~/.kenv/scripts to ~/.scriptkit/kit/main/scripts
    let old_scripts = old_kenv.join("scripts");
    if old_scripts.exists() && old_scripts.is_dir() {
        if let Ok(entries) = fs::read_dir(&old_scripts) {
            for entry in entries.flatten() {
                let old_path = entry.path();
                let file_name = old_path.file_name().unwrap_or_default();
                let new_path = main_scripts.join(file_name);

                if let Err(e) = fs::rename(&old_path, &new_path) {
                    warn!(
                        error = %e,
                        old = %old_path.display(),
                        new = %new_path.display(),
                        "Failed to move script"
                    );
                }
            }
        }
    }

    // Move scriptlets from ~/.kenv/scriptlets to ~/.scriptkit/kit/main/extensions
    let old_scriptlets = old_kenv.join("scriptlets");
    if old_scriptlets.exists() && old_scriptlets.is_dir() {
        if let Ok(entries) = fs::read_dir(&old_scriptlets) {
            for entry in entries.flatten() {
                let old_path = entry.path();
                let file_name = old_path.file_name().unwrap_or_default();
                let new_path = main_extensions.join(file_name);

                if let Err(e) = fs::rename(&old_path, &new_path) {
                    warn!(
                        error = %e,
                        old = %old_path.display(),
                        new = %new_path.display(),
                        "Failed to move scriptlet"
                    );
                }
            }
        }
    }

    // Move config files to new root
    let config_files = ["config.ts", "theme.json", "tsconfig.json", ".gitignore"];
    for file in config_files {
        let old_path = old_kenv.join(file);
        let new_path = new_scriptkit.join(file);
        if old_path.exists() && !new_path.exists() {
            if let Err(e) = fs::rename(&old_path, &new_path) {
                warn!(error = %e, file = file, "Failed to move config file");
            }
        }
    }

    // Move data directories to new root
    let data_dirs = ["logs", "cache", "db", "sdk"];
    for dir in data_dirs {
        let old_path = old_kenv.join(dir);
        let new_path = new_scriptkit.join(dir);
        if old_path.exists() && old_path.is_dir() && !new_path.exists() {
            if let Err(e) = fs::rename(&old_path, &new_path) {
                warn!(error = %e, dir = dir, "Failed to move data directory");
            }
        }
    }

    // Move data files to new root
    let data_files = [
        "frecency.json",
        "store.json",
        "server.json",
        "agent-token",
        "notes.db",
        "ai-chats.db",
        "clipboard-history.db",
    ];
    for file in data_files {
        let old_path = old_kenv.join(file);
        let new_path = new_scriptkit.join(file);
        if old_path.exists() && !new_path.exists() {
            if let Err(e) = fs::rename(&old_path, &new_path) {
                warn!(error = %e, file = file, "Failed to move data file");
            }
        }
    }

    // Remove the old ~/.kenv directory (should be mostly empty now)
    if let Err(e) = fs::remove_dir_all(&old_kenv) {
        warn!(error = %e, "Failed to remove old ~/.kenv directory, may have remaining files");
    }

    // Create symlink for backwards compatibility (Unix only)
    #[cfg(unix)]
    {
        if let Err(e) = std::os::unix::fs::symlink(&new_scriptkit, &old_kenv) {
            warn!(error = %e, "Failed to create ~/.kenv symlink for backwards compatibility");
        } else {
            info!("Created ~/.kenv -> ~/.scriptkit symlink for backwards compatibility");
        }
    }

    info!("Migration from ~/.kenv to ~/.scriptkit complete");
    true
}
// --- merged from part_003.rs ---
/// Ensure the ~/.scriptkit environment is properly set up.
///
/// This function is idempotent - it will create missing directories and files
/// without overwriting existing user configurations.
///
/// # Directory Structure Created
/// ```text
/// ~/.scriptkit/                  # Root (can be overridden via SK_PATH)
/// ├── kit/                       # All kits container (for easy version control)
/// │   ├── main/                  # Default user kit
/// │   │   ├── scripts/           # User scripts (.ts, .js files)
/// │   │   ├── extensions/         # Markdown extension files
/// │   │   └── agents/             # AI agent definitions (.md)
/// │   └── custom-kit/            # Additional custom kits
/// │       ├── scripts/
/// │       ├── extensions/
/// │       └── agents/
/// │   ├── package.json           # Node.js module config (type: module for top-level await)
/// │   └── tsconfig.json          # TypeScript path mappings
/// │   ├── config.ts              # User configuration (created from template if missing)
/// │   ├── theme.json             # Theme configuration (created from example if missing)
/// │   ├── AGENTS.md              # Redirect stub → ../AGENTS.md
/// │   └── CLAUDE.md              # Redirect stub → ../CLAUDE.md
/// ├── sdk/                       # Runtime SDK (kit-sdk.ts)
/// ├── db/                        # Databases
/// ├── logs/                      # Application logs
/// ├── cache/
/// │   └── app-icons/             # Cached application icons
/// ├── GUIDE.md                   # User guide
/// └── .gitignore                 # Ignore transient files
/// ```
///
/// # Environment Variables
/// - `SK_PATH`: Override the default ~/.scriptkit path
///
/// # Returns
/// `SetupResult` with information about the setup process.
#[instrument(level = "info", name = "ensure_kit_setup")]
pub fn ensure_kit_setup() -> SetupResult {
    let mut warnings = Vec::new();

    let kit_dir = get_kit_path();

    // Check if this is a fresh install before we create anything
    let is_fresh_install = !kit_dir.exists();

    // Log if using SK_PATH override
    if std::env::var(SK_PATH_ENV).is_ok() {
        info!(
            kit_path = %kit_dir.display(),
            "Using SK_PATH override"
        );
    }

    // Ensure root kit directory exists first
    if let Err(e) = fs::create_dir_all(&kit_dir) {
        warnings.push(format!(
            "Failed to create kit root {}: {}",
            kit_dir.display(),
            e
        ));
        // If we can't create the root, there's not much else we can safely do.
        return SetupResult {
            is_fresh_install,
            kit_path: kit_dir,
            bun_available: false,
            warnings,
        };
    }

    // Required directory structure
    // Note: kit/main/scripts and kit/main/extensions are the default user workspace
    // All kits live under kit/ for easier version control
    let required_dirs = [
        kit_dir.join("kit").join("main").join("scripts"),
        kit_dir.join("kit").join("main").join("extensions"),
        kit_dir.join("kit").join("main").join("agents"),
        // Built-in CleanShot extension kit
        kit_dir.join("kit").join("cleanshot").join("extensions"),
        // Built-in 1Password extension kit
        kit_dir.join("kit").join("1password").join("extensions"),
        // Built-in Quick Links extension kit
        kit_dir.join("kit").join("quicklinks").join("extensions"),
        // Built-in Window Management extension kit
        kit_dir
            .join("kit")
            .join("window-management")
            .join("extensions"),
        // Built-in AI Text Tools extension kit
        kit_dir.join("kit").join("ai-text-tools").join("extensions"),
        // Built-in Examples extension kit (scriptlet pattern reference)
        kit_dir.join("kit").join("examples").join("extensions"),
        // Root-level agent workspace directories
        kit_dir.join("skills").join("script-authoring"),
        kit_dir.join("skills").join("scriptlets"),
        kit_dir.join("skills").join("config"),
        kit_dir.join("skills").join("troubleshooting"),
        kit_dir.join("skills").join("agents"),
        kit_dir.join("examples").join("scripts"),
        kit_dir.join("examples").join("extensions"),
        kit_dir.join("examples").join("agents"),
        kit_dir.join("docs"),
        // Root-level harness temp workspace used by kit://sdk-reference
        kit_dir.join("tmp").join("test-scripts"),
        kit_dir.join("tmp").join("test-scriptlets"),
        kit_dir.join("sdk"),
        kit_dir.join("db"),
        kit_dir.join("logs"),
        kit_dir.join("cache").join("app-icons"),
    ];

    for dir in required_dirs {
        ensure_dir(&dir, &mut warnings);
    }

    // App-managed: SDK (refresh if changed)
    let sdk_path = kit_dir.join("sdk").join("kit-sdk.ts");
    write_string_if_changed(&sdk_path, EMBEDDED_SDK, &mut warnings, "sdk/kit-sdk.ts");

    // App-managed: Built-in CleanShot X extension (refresh if changed)
    // This extension ships with the app and provides screenshot/recording commands
    let cleanshot_path = kit_dir
        .join("kit")
        .join("cleanshot")
        .join("extensions")
        .join("main.md");
    write_string_if_changed(
        &cleanshot_path,
        EMBEDDED_CLEANSHOT_EXTENSION,
        &mut warnings,
        "kit/cleanshot/extensions/main.md",
    );

    // App-managed: Built-in CleanShot X shared actions (refresh if changed)
    // These actions are automatically available for all CleanShot scriptlets
    let cleanshot_actions_path = kit_dir
        .join("kit")
        .join("cleanshot")
        .join("extensions")
        .join("main.actions.md");
    write_string_if_changed(
        &cleanshot_actions_path,
        EMBEDDED_CLEANSHOT_ACTIONS,
        &mut warnings,
        "kit/cleanshot/extensions/main.actions.md",
    );

    // App-managed: Built-in 1Password extension (refresh if changed)
    // This extension ships with the app and provides password manager CLI commands
    let onepassword_path = kit_dir
        .join("kit")
        .join("1password")
        .join("extensions")
        .join("main.md");
    write_string_if_changed(
        &onepassword_path,
        EMBEDDED_1PASSWORD_EXTENSION,
        &mut warnings,
        "kit/1password/extensions/main.md",
    );

    // App-managed: Built-in Quick Links extension (refresh if changed)
    // This extension ships with the app and provides quick access to common websites
    let quicklinks_path = kit_dir
        .join("kit")
        .join("quicklinks")
        .join("extensions")
        .join("main.md");
    write_string_if_changed(
        &quicklinks_path,
        EMBEDDED_QUICKLINKS_EXTENSION,
        &mut warnings,
        "kit/quicklinks/extensions/main.md",
    );

    // App-managed: Built-in Quick Links shared actions (refresh if changed)
    // These actions are automatically available for all Quick Links scriptlets
    let quicklinks_actions_path = kit_dir
        .join("kit")
        .join("quicklinks")
        .join("extensions")
        .join("main.actions.md");
    write_string_if_changed(
        &quicklinks_actions_path,
        EMBEDDED_QUICKLINKS_ACTIONS,
        &mut warnings,
        "kit/quicklinks/extensions/main.actions.md",
    );

    // App-managed: Built-in Window Management extension (refresh if changed)
    // This extension ships with the app and provides window tiling and positioning
    let window_management_path = kit_dir
        .join("kit")
        .join("window-management")
        .join("extensions")
        .join("main.md");
    write_string_if_changed(
        &window_management_path,
        EMBEDDED_WINDOW_MANAGEMENT_EXTENSION,
        &mut warnings,
        "kit/window-management/extensions/main.md",
    );

    // App-managed: Built-in AI Text Tools extension (refresh if changed)
    // This extension ships with the app and provides AI-powered text transformations
    let ai_text_tools_path = kit_dir
        .join("kit")
        .join("ai-text-tools")
        .join("extensions")
        .join("main.md");
    write_string_if_changed(
        &ai_text_tools_path,
        EMBEDDED_AI_TEXT_TOOLS_EXTENSION,
        &mut warnings,
        "kit/ai-text-tools/extensions/main.md",
    );

    // App-managed: Built-in Examples extension (refresh if changed)
    // This extension ships with the app and provides complete scriptlet pattern reference
    let examples_dir = kit_dir.join("kit").join("examples").join("extensions");
    write_string_if_changed(
        &examples_dir.join("main.md"),
        EMBEDDED_EXAMPLES_MAIN,
        &mut warnings,
        "kit/examples/extensions/main.md",
    );
    write_string_if_changed(
        &examples_dir.join("advanced.md"),
        EMBEDDED_EXAMPLES_ADVANCED,
        &mut warnings,
        "kit/examples/extensions/advanced.md",
    );
    write_string_if_changed(
        &examples_dir.join("howto.md"),
        EMBEDDED_EXAMPLES_HOWTO,
        &mut warnings,
        "kit/examples/extensions/howto.md",
    );

    // User-owned: config.ts (only create if missing)
    // Located in kit/ directory so it can be version controlled with user scripts
    let config_path = kit_dir.join("kit").join("config.ts");
    write_string_if_missing(
        &config_path,
        EMBEDDED_CONFIG_TEMPLATE,
        &mut warnings,
        "kit/config.ts",
    );

    // User-owned (optional): theme.json (only create if missing)
    // Located in kit/ directory so it can be version controlled with user scripts
    let theme_path = kit_dir.join("kit").join("theme.json");
    write_string_if_missing(
        &theme_path,
        EMBEDDED_THEME_EXAMPLE,
        &mut warnings,
        "kit/theme.json",
    );

    // App-managed: tsconfig.json path mappings in kit/ directory (merge-safe)
    // Located at ~/.scriptkit/kit/tsconfig.json to be alongside user scripts
    ensure_tsconfig_paths(&kit_dir.join("kit").join("tsconfig.json"), &mut warnings);

    // App-managed: package.json for top-level await support in kit/ directory
    // The "type": "module" allows scripts in kit/main/scripts/*.ts to use top-level await
    let package_json_path = kit_dir.join("kit").join("package.json");
    write_string_if_missing(
        &package_json_path,
        EMBEDDED_PACKAGE_JSON,
        &mut warnings,
        "kit/package.json",
    );

    // User-owned: GUIDE.md (only create if missing)
    // Comprehensive user guide for learning Script Kit
    let guide_md_path = kit_dir.join("GUIDE.md");
    write_string_if_missing(&guide_md_path, EMBEDDED_GUIDE_MD, &mut warnings, "GUIDE.md");

    // Root-level CLAUDE.md — the canonical agent instructions file.
    // Lives at ~/.scriptkit/CLAUDE.md so harnesses that cwd into ~/.scriptkit find it.
    let root_claude_md_path = kit_dir.join("CLAUDE.md");
    write_string_if_changed(
        &root_claude_md_path,
        EMBEDDED_ROOT_CLAUDE_MD,
        &mut warnings,
        "CLAUDE.md",
    );

    // Root-level AGENTS.md — SDK reference for all agents.
    let root_agents_md_path = kit_dir.join("AGENTS.md");
    write_string_if_changed(
        &root_agents_md_path,
        EMBEDDED_ROOT_AGENTS_MD,
        &mut warnings,
        "AGENTS.md",
    );

    // Redirect stubs for kit/CLAUDE.md and kit/AGENTS.md so agents that
    // happen to look there are pointed to the root-level canonical files.
    let kit_claude_redirect = "# See ../CLAUDE.md\n\n\
        This file has moved to the workspace root for better agent discoverability.\n\
        Read `~/.scriptkit/CLAUDE.md` instead.\n";
    write_string_if_changed(
        &kit_dir.join("kit").join("CLAUDE.md"),
        kit_claude_redirect,
        &mut warnings,
        "kit/CLAUDE.md redirect",
    );
    let kit_agents_redirect = "# See ../AGENTS.md\n\n\
        This file has moved to the workspace root for better agent discoverability.\n\
        Read `~/.scriptkit/AGENTS.md` instead.\n";
    write_string_if_changed(
        &kit_dir.join("kit").join("AGENTS.md"),
        kit_agents_redirect,
        &mut warnings,
        "kit/AGENTS.md redirect",
    );

    // App-managed: Skills library (refresh if changed)
    write_string_if_changed(
        &kit_dir.join("skills").join("README.md"),
        EMBEDDED_SKILLS_README,
        &mut warnings,
        "skills/README.md",
    );
    write_string_if_changed(
        &kit_dir
            .join("skills")
            .join("script-authoring")
            .join("SKILL.md"),
        EMBEDDED_SKILL_SCRIPT_AUTHORING,
        &mut warnings,
        "skills/script-authoring/SKILL.md",
    );
    write_string_if_changed(
        &kit_dir.join("skills").join("scriptlets").join("SKILL.md"),
        EMBEDDED_SKILL_SCRIPTLETS,
        &mut warnings,
        "skills/scriptlets/SKILL.md",
    );
    write_string_if_changed(
        &kit_dir.join("skills").join("config").join("SKILL.md"),
        EMBEDDED_SKILL_CONFIG,
        &mut warnings,
        "skills/config/SKILL.md",
    );
    write_string_if_changed(
        &kit_dir
            .join("skills")
            .join("troubleshooting")
            .join("SKILL.md"),
        EMBEDDED_SKILL_TROUBLESHOOTING,
        &mut warnings,
        "skills/troubleshooting/SKILL.md",
    );
    write_string_if_changed(
        &kit_dir.join("skills").join("agents").join("SKILL.md"),
        EMBEDDED_SKILL_AGENTS,
        &mut warnings,
        "skills/agents/SKILL.md",
    );

    // App-managed: Example scripts (refresh if changed)
    write_string_if_changed(
        &kit_dir.join("examples").join("README.md"),
        EMBEDDED_EXAMPLES_README,
        &mut warnings,
        "examples/README.md",
    );
    write_string_if_changed(
        &kit_dir
            .join("examples")
            .join("scripts")
            .join("hello-world.ts"),
        EMBEDDED_EXAMPLE_HELLO_WORLD,
        &mut warnings,
        "examples/scripts/hello-world.ts",
    );
    write_string_if_changed(
        &kit_dir
            .join("examples")
            .join("scripts")
            .join("choose-from-list.ts"),
        EMBEDDED_EXAMPLE_CHOOSE_FROM_LIST,
        &mut warnings,
        "examples/scripts/choose-from-list.ts",
    );
    write_string_if_changed(
        &kit_dir
            .join("examples")
            .join("scripts")
            .join("clipboard-transform.ts"),
        EMBEDDED_EXAMPLE_CLIPBOARD_TRANSFORM,
        &mut warnings,
        "examples/scripts/clipboard-transform.ts",
    );
    write_string_if_changed(
        &kit_dir
            .join("examples")
            .join("scripts")
            .join("path-picker.ts"),
        EMBEDDED_EXAMPLE_PATH_PICKER,
        &mut warnings,
        "examples/scripts/path-picker.ts",
    );

    // App-managed: Example extension references (refresh if changed)
    // These are copies of the built-in extension bundles for agent discoverability
    write_string_if_changed(
        &kit_dir.join("examples").join("extensions").join("main.md"),
        EMBEDDED_EXAMPLES_MAIN,
        &mut warnings,
        "examples/extensions/main.md",
    );
    write_string_if_changed(
        &kit_dir
            .join("examples")
            .join("extensions")
            .join("advanced.md"),
        EMBEDDED_EXAMPLES_ADVANCED,
        &mut warnings,
        "examples/extensions/advanced.md",
    );
    write_string_if_changed(
        &kit_dir.join("examples").join("extensions").join("howto.md"),
        EMBEDDED_EXAMPLES_HOWTO,
        &mut warnings,
        "examples/extensions/howto.md",
    );

    // App-managed: Example agents (refresh if changed)
    write_string_if_changed(
        &kit_dir
            .join("examples")
            .join("agents")
            .join("review-pr.claude.md"),
        EMBEDDED_EXAMPLE_AGENT_REVIEW_PR,
        &mut warnings,
        "examples/agents/review-pr.claude.md",
    );
    write_string_if_changed(
        &kit_dir
            .join("examples")
            .join("agents")
            .join("plan-feature.i.gemini.md"),
        EMBEDDED_EXAMPLE_AGENT_PLAN_FEATURE,
        &mut warnings,
        "examples/agents/plan-feature.i.gemini.md",
    );

    // App-managed: .gitignore (refresh if changed)
    let gitignore_path = kit_dir.join(".gitignore");
    let gitignore_content = r#"# Script Kit managed .gitignore
# This file is regenerated on app start - edit with caution

# =============================================================================
# Node.js / Bun dependencies
# =============================================================================
# Root node_modules (for package.json at ~/.scriptkit/kit/)
node_modules/

# Kit-specific node_modules (e.g., main/node_modules, examples/node_modules)
*/node_modules/

# Package manager files
package-lock.json
yarn.lock
pnpm-lock.yaml
bun.lockb
.pnpm-store/

# =============================================================================
# Databases
# =============================================================================
# SQLite databases
*.db
*.db-journal
*.db-shm
*.db-wal

# Specific databases (redundant with *.db but explicit for clarity)
db/
clipboard-history.db
notes.db
ai-chats.db

# =============================================================================
# Runtime & Cache
# =============================================================================
# SDK is managed by the app, always regenerated
sdk/

# Application logs
logs/

# Cache files (app icons, etc.)
cache/

# Frecency tracking (regenerated from usage)
frecency.json

# Server state
server.json

# Authentication tokens
agent-token

# =============================================================================
# Build & Tooling
# =============================================================================
# TypeScript build output
*.tsbuildinfo
dist/
build/
.turbo/

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# macOS
.DS_Store
._*

# =============================================================================
# Secrets & Environment
# =============================================================================
.env
.env.local
.env.*.local
*.pem
*.key

# =============================================================================
# Temporary files
# =============================================================================
*.tmp
*.temp
*.log
tmp/
temp/
"#;
    write_string_if_changed(
        &gitignore_path,
        gitignore_content,
        &mut warnings,
        ".gitignore",
    );

    // Dependency check: bun (no process spawn; just path checks)
    let bun_available = bun_is_discoverable();
    if !bun_available {
        warnings.push(
            "bun not found (PATH/common install locations). Config/scripts may not run until bun is installed.".to_string(),
        );
    }

    // Optional "getting started" content only on truly fresh installs
    if is_fresh_install {
        create_sample_files(&kit_dir, &mut warnings);
    }

    info!(
        kit_path = %kit_dir.display(),
        is_fresh_install,
        bun_available,
        warning_count = warnings.len(),
        "Kit setup complete"
    );

    SetupResult {
        is_fresh_install,
        kit_path: kit_dir,
        bun_available,
        warnings,
    }
}
fn ensure_dir(path: &Path, warnings: &mut Vec<String>) {
    if path.exists() {
        return;
    }
    if let Err(e) = fs::create_dir_all(path) {
        warnings.push(format!(
            "Failed to create directory {}: {}",
            path.display(),
            e
        ));
    } else {
        debug!(path = %path.display(), "Created directory");
    }
}
fn write_string_if_missing(path: &Path, contents: &str, warnings: &mut Vec<String>, label: &str) {
    if path.exists() {
        return;
    }
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            warnings.push(format!(
                "Failed to create parent dir for {} ({}): {}",
                label,
                parent.display(),
                e
            ));
            return;
        }
    }
    if let Err(e) = fs::write(path, contents) {
        warnings.push(format!(
            "Failed to write {} ({}): {}",
            label,
            path.display(),
            e
        ));
    } else {
        info!(path = %path.display(), "Created {}", label);
    }
}
// --- merged from part_004.rs ---
/// Write string to path if content changed, using atomic rename for safety
///
/// This function uses an atomic write pattern to prevent race conditions and
/// partial writes:
/// 1. Write to a temporary file in the same directory
/// 2. Atomically rename temp file to target path
///
/// The rename is atomic on most filesystems, so readers will either see the
/// old content or the new content, never a partial write.
fn write_string_if_changed(path: &Path, contents: &str, warnings: &mut Vec<String>, label: &str) {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == contents {
            return;
        }
    }

    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            warnings.push(format!(
                "Failed to create parent dir for {} ({}): {}",
                label,
                parent.display(),
                e
            ));
            return;
        }
    }

    // Atomic write: write to temp file then rename
    // This prevents readers from seeing partial writes during concurrent access
    let temp_path = path.with_extension("tmp");

    if let Err(e) = fs::write(&temp_path, contents) {
        warnings.push(format!(
            "Failed to write temp file for {} ({}): {}",
            label,
            temp_path.display(),
            e
        ));
        return;
    }

    // Atomic rename - this is atomic on most filesystems
    if let Err(e) = fs::rename(&temp_path, path) {
        warnings.push(format!(
            "Failed to rename {} to {}: {}",
            temp_path.display(),
            path.display(),
            e
        ));
        // Clean up temp file on failure
        let _ = fs::remove_file(&temp_path);
    } else {
        debug!(path = %path.display(), "Updated {}", label);
    }
}
/// Ensure tsconfig.json has proper TypeScript/Bun settings (merge-safe)
/// The tsconfig lives at ~/.scriptkit/kit/tsconfig.json, SDK at ~/.scriptkit/sdk/
///
/// Sets essential options while preserving user customizations:
/// - target: ESNext (for top-level await and modern features)
/// - module: ESNext (ES modules)
/// - moduleResolution: Bundler (optimal for Bun)
/// - paths: @scriptkit/sdk mapping
/// - noEmit: true (Bun runs .ts directly)
/// - skipLibCheck: true (faster)
/// - esModuleInterop: true (CommonJS compat)
fn ensure_tsconfig_paths(tsconfig_path: &Path, warnings: &mut Vec<String>) {
    use serde_json::{json, Value};

    // Path is relative from kit/ to sdk/: ../sdk/kit-sdk.ts
    let expected_sdk_path = json!(["../sdk/kit-sdk.ts"]);

    let mut config: Value = if tsconfig_path.exists() {
        match fs::read_to_string(tsconfig_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| json!({})),
            Err(_) => json!({}),
        }
    } else {
        json!({})
    };

    // Ensure compilerOptions exists
    if config.get("compilerOptions").is_none() {
        config["compilerOptions"] = json!({});
    }

    let Some(compiler_options) = config["compilerOptions"].as_object_mut() else {
        warnings.push("tsconfig.json compilerOptions is not an object".to_string());
        return;
    };
    let mut changed = false;

    // Essential settings for Bun/TypeScript scripts (set if missing)
    let defaults = [
        ("target", json!("ESNext")),
        ("module", json!("ESNext")),
        ("moduleResolution", json!("Bundler")),
        ("noEmit", json!(true)),
        ("skipLibCheck", json!(true)),
        ("esModuleInterop", json!(true)),
        ("allowImportingTsExtensions", json!(true)),
        ("verbatimModuleSyntax", json!(true)),
    ];

    for (key, value) in defaults {
        if !compiler_options.contains_key(key) {
            compiler_options.insert(key.to_string(), value);
            changed = true;
        }
    }

    // Ensure paths exists
    if !compiler_options.contains_key("paths") {
        compiler_options.insert("paths".to_string(), json!({}));
        changed = true;
    }

    // Always ensure @scriptkit/sdk path is correct
    let Some(paths) = compiler_options
        .get_mut("paths")
        .and_then(|v| v.as_object_mut())
    else {
        warnings.push("tsconfig.json paths is not an object".to_string());
        return;
    };
    if paths.get("@scriptkit/sdk") != Some(&expected_sdk_path) {
        paths.insert("@scriptkit/sdk".to_string(), expected_sdk_path);
        changed = true;
    }

    if !changed {
        return;
    }

    match serde_json::to_string_pretty(&config) {
        Ok(json_str) => {
            if let Err(e) = fs::write(tsconfig_path, json_str) {
                warnings.push(format!(
                    "Failed to write tsconfig.json ({}): {}",
                    tsconfig_path.display(),
                    e
                ));
                warn!(error = %e, "Failed to write tsconfig.json");
            } else {
                info!("Updated tsconfig.json with TypeScript/Bun settings");
            }
        }
        Err(e) => {
            warnings.push(format!("Failed to serialize tsconfig.json: {}", e));
            warn!(error = %e, "Failed to serialize tsconfig.json");
        }
    }
}
/// Fast check: looks for bun in common locations and PATH without spawning a process.
fn bun_is_discoverable() -> bool {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Common install locations
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".bun").join("bin").join(bun_exe_name()));
    }
    candidates.push(PathBuf::from("/opt/homebrew/bin").join(bun_exe_name()));
    candidates.push(PathBuf::from("/usr/local/bin").join(bun_exe_name()));
    candidates.push(PathBuf::from("/usr/bin").join(bun_exe_name()));

    // PATH scan
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            candidates.push(dir.join(bun_exe_name()));
        }
    }

    candidates.into_iter().any(|p| p.exists())
}
fn bun_exe_name() -> &'static str {
    #[cfg(windows)]
    {
        "bun.exe"
    }
    #[cfg(not(windows))]
    {
        "bun"
    }
}
// --- merged from part_005.rs ---
fn create_sample_files(kit_dir: &Path, warnings: &mut Vec<String>) {
    // Create sample files in the main kit (under kit/ subdirectory)
    let main_scripts_dir = kit_dir.join("kit").join("main").join("scripts");
    let main_extensions_dir = kit_dir.join("kit").join("main").join("extensions");
    let main_agents_dir = kit_dir.join("kit").join("main").join("agents");

    // Create hello-world.ts script
    let hello_script_path = main_scripts_dir.join("hello-world.ts");
    if !hello_script_path.exists() {
        let hello_script = r#"/*
# Hello World

A simple greeting script demonstrating Script Kit basics.

## Features shown:
- `arg()` - Prompt for user input with choices
- `div()` - Display HTML content with Tailwind CSS
- `md()` - Render markdown to HTML
*/

export const metadata = {
  name: "Hello World",
  description: "A simple greeting script",
  // shortcut: "cmd shift h",  // Uncomment to add a global hotkey
};

// Prompt the user to select or type their name
const name = await arg("What's your name?", [
  "World",
  "Script Kit",
  "Friend",
]);

// Display a greeting using HTML with Tailwind CSS classes
await div(`
  <div class="flex flex-col items-center justify-center h-full p-8">
    <h1 class="text-4xl font-bold text-yellow-400 mb-4">
      Hello, ${name}! 👋
    </h1>
    <p class="text-gray-400 text-lg">
      Welcome to Script Kit
    </p>
    <div class="mt-6 text-sm text-gray-500">
      Press <kbd class="px-2 py-1 bg-gray-700 rounded">Escape</kbd> to close
    </div>
  </div>
`);
"#;
        if let Err(e) = fs::write(&hello_script_path, hello_script) {
            warnings.push(format!(
                "Failed to create sample script {}: {}",
                hello_script_path.display(),
                e
            ));
        } else {
            info!(path = %hello_script_path.display(), "Created sample script");
        }
    }

    // Create hello-world.md extension
    let hello_extension_path = main_extensions_dir.join("hello-world.md");
    if !hello_extension_path.exists() {
        let hello_extension = r#"# Hello World Extensions

Quick shell commands you can run from Script Kit.
Each code block is a separate scriptlet that appears in the menu.

---

## Say Hello
<!-- 
name: Say Hello
description: Display a greeting notification
shortcut: ctrl h
-->

```bash
echo "Hello from Script Kit! 🎉"
```

---

## Current Date
<!-- 
name: Current Date
description: Copy today's date to clipboard
shortcut: ctrl d
-->

```bash
date +"%Y-%m-%d" | pbcopy
echo "Date copied: $(date +"%Y-%m-%d")"
```

---

## Open Downloads
<!-- 
name: Open Downloads
description: Open the Downloads folder in Finder
-->

```bash
open ~/Downloads
```

---

## Quick Note
<!-- 
name: Quick Note
description: Append a timestamped note to notes.txt
-->

```bash
echo "[$(date +"%Y-%m-%d %H:%M")] $1" >> ~/notes.txt
echo "Note saved!"
```

---

## System Info
<!-- 
name: System Info
description: Show basic system information
-->

```bash
echo "User: $(whoami)"
echo "Host: $(hostname)"
echo "OS: $(sw_vers -productName) $(sw_vers -productVersion)"
echo "Shell: $SHELL"
```
"#;
        if let Err(e) = fs::write(&hello_extension_path, hello_extension) {
            warnings.push(format!(
                "Failed to create sample extension {}: {}",
                hello_extension_path.display(),
                e
            ));
        } else {
            info!(path = %hello_extension_path.display(), "Created sample extension");
        }
    }

    // Create hello-world.claude.md agent
    let hello_agent_path = main_agents_dir.join("hello-world.claude.md");
    if !hello_agent_path.exists() {
        let hello_agent = r#"---
_sk_name: Hello World Assistant
_sk_description: A friendly assistant that helps with simple tasks
_sk_interactive: true
---

You are a friendly, helpful assistant. Keep responses concise and practical.

When the user asks for help:
1. Understand their request clearly
2. Provide a direct, actionable answer
3. Offer to help with follow-up questions

Be conversational but efficient. Focus on solving the user's immediate needs.
"#;
        if let Err(e) = fs::write(&hello_agent_path, hello_agent) {
            warnings.push(format!(
                "Failed to create sample agent {}: {}",
                hello_agent_path.display(),
                e
            ));
        } else {
            info!(path = %hello_agent_path.display(), "Created sample agent");
        }
    }

    // Create README.md at kit root
    let readme_path = kit_dir.join("README.md");
    if !readme_path.exists() {
        let readme = r##"# Script Kit

Welcome to Script Kit! This directory contains your scripts, configuration, and data.

## Directory Structure

```
~/.scriptkit/
├── kit/                    # All kits (version control friendly)
│   ├── main/               # Your default kit
│   │   ├── scripts/        # TypeScript/JavaScript scripts (.ts, .js)
│   │   ├── extensions/     # Markdown extension files (.md)
│   │   └── agents/         # AI agent definitions (.md)
│   ├── package.json        # Node.js module config (enables top-level await)
│   └── tsconfig.json       # TypeScript path mappings
├── sdk/                    # Runtime SDK (managed by app)
├── db/                     # Databases (clipboard history, etc.)
├── logs/                   # Application logs
├── cache/                  # Cached data (app icons, etc.)
├── config.ts               # Your configuration
├── theme.json              # Theme customization
└── README.md               # This file
```

## File Watching

Script Kit watches these files and reloads automatically:

| File/Directory | What happens on change |
|----------------|------------------------|
| `config.ts` | Reloads configuration (hotkeys, settings) |
| `theme.json` | Applies new theme colors immediately |
| `main/scripts/*.ts` | Updates script list and metadata |
| `main/extensions/*.md` | Updates extension list |

## Scripts

Scripts are TypeScript files in `main/scripts/`. They have full access to the Script Kit SDK.

### Example Script

```typescript
// main/scripts/my-script.ts

export const metadata = {
  name: "My Script",
  description: "Does something useful",
  shortcut: "cmd shift m",  // Optional global hotkey
};

// Prompt for input
const choice = await arg("Pick an option", ["Option 1", "Option 2"]);

// Show result
await div(`<div class="p-4">You chose: ${choice}</div>`);
```

### Script Metadata

Use the `metadata` export for type-safe configuration:

```typescript
export const metadata = {
  name: "Script Name",           // Display name in menu
  description: "What it does",   // Shown below the name
  shortcut: "cmd shift x",       // Global hotkey (optional)
  alias: "sn",                   // Quick search alias (optional)
};
```

## Scriptlets

Scriptlets are Markdown files containing quick shell commands. Each code block becomes a menu item.

### Example Scriptlet

```markdown
# My Scriptlets

## Open Project
<!-- shortcut: cmd shift p -->

\`\`\`bash
cd ~/projects/myapp && code .
\`\`\`

## Git Status
<!-- name: Check Git Status -->

\`\`\`bash
git status
\`\`\`
```

### Scriptlet Metadata

Add HTML comments before code blocks:

```markdown
<!-- 
name: Display Name
description: What this does
shortcut: cmd shift x
-->
```

## Configuration (config.ts)

Your `config.ts` controls Script Kit behavior:

```typescript
export default {
  // Global hotkey to open Script Kit
  hotkey: {
    key: "Semicolon",
    modifiers: ["meta"],  // cmd+;
  },
  
  // UI Settings
  editorFontSize: 16,
  terminalFontSize: 14,
  
  // Built-in features
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
  },
} satisfies Config;
```

## Theme (theme.json)

Customize colors in `theme.json`:

```json
{
  "colors": {
    "background": {
      "main": "#1E1E1E"
    },
    "text": {
      "primary": "#FFFFFF",
      "secondary": "#CCCCCC"
    },
    "accent": {
      "selected": "#FBBF24"
    }
  }
}
```

Colors can be specified as:
- Hex strings: `"#FBBF24"` or `"FBBF24"`
- RGB: `"rgb(251, 191, 36)"`
- RGBA: `"rgba(251, 191, 36, 1.0)"`

## Environment Variable

Set `SK_PATH` to use a different directory:

```bash
export SK_PATH=~/my-scripts
```

## Quick Tips

1. **Create a new script**: Add a `.ts` file to `main/scripts/`
2. **Add a hotkey**: Set `shortcut` in the metadata
3. **Test changes**: Scripts reload automatically on save
4. **View logs**: Check `logs/script-kit-gpui.jsonl` for debugging
5. **Complete guide**: See `GUIDE.md` for comprehensive tutorials and documentation

## Links

- Documentation: https://scriptkit.com/docs
- GitHub: https://github.com/johnlindquist/kit

---

Happy scripting! 🚀
"##;
        if let Err(e) = fs::write(&readme_path, readme) {
            warnings.push(format!(
                "Failed to create README {}: {}",
                readme_path.display(),
                e
            ));
        } else {
            info!(path = %readme_path.display(), "Created README.md");
        }
    }
}
// --- merged from part_006.rs ---
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Run a test body while holding the shared SK_PATH lock.
    /// Automatically sets SK_PATH to `kit_root` and removes it on exit.
    fn with_sk_path<F: FnOnce(&std::path::Path)>(f: F) {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().to_path_buf();
        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        f(&kit_root);

        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    /// Test that kit directory structure uses kit/ subdirectory
    /// Expected structure: ~/.scriptkit/kit/main/scripts, ~/.scriptkit/kit/main/extensions
    #[test]
    fn test_kit_directory_uses_kit_subdirectory() {
        with_sk_path(|kit_root| {
            let result = ensure_kit_setup();

            let kit_main_scripts = kit_root.join("kit").join("main").join("scripts");
            let kit_main_extensions = kit_root.join("kit").join("main").join("extensions");

            assert!(
                kit_main_scripts.exists(),
                "Expected kit/main/scripts to exist at {:?}",
                kit_main_scripts
            );
            assert!(
                kit_main_extensions.exists(),
                "Expected kit/main/extensions to exist at {:?}",
                kit_main_extensions
            );

            let old_main_scripts = kit_root.join("main").join("scripts");
            assert!(
                !old_main_scripts.exists(),
                "Old structure main/scripts should NOT exist at {:?}",
                old_main_scripts
            );

            assert!(!result.warnings.iter().any(|w| w.contains("Failed")));
        });
    }

    /// Test that sample files are created in kit/main/scripts
    #[test]
    fn test_sample_files_in_kit_subdirectory() {
        with_sk_path(|kit_root| {
            let result = ensure_kit_setup();

            if result.is_fresh_install {
                let hello_script = kit_root
                    .join("kit")
                    .join("main")
                    .join("scripts")
                    .join("hello-world.ts");
                assert!(
                    hello_script.exists(),
                    "Expected hello-world.ts at {:?}",
                    hello_script
                );
            }
        });
    }

    #[test]
    fn test_bun_is_discoverable() {
        // This test just verifies the function doesn't panic
        let _ = bun_is_discoverable();
    }

    #[test]
    fn test_bun_exe_name() {
        let name = bun_exe_name();
        #[cfg(windows)]
        assert_eq!(name, "bun.exe");
        #[cfg(not(windows))]
        assert_eq!(name, "bun");
    }

    #[test]
    fn test_get_kit_path_default() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var(SK_PATH_ENV);
        let path = get_kit_path();
        assert!(path.to_string_lossy().contains(".scriptkit"));
        drop(lock);
    }

    #[test]
    fn test_get_kit_path_with_override() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var(SK_PATH_ENV, "/custom/path");
        let path = get_kit_path();
        assert_eq!(path, PathBuf::from("/custom/path"));
        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    #[test]
    fn test_get_kit_path_with_tilde() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var(SK_PATH_ENV, "~/.config/kit");
        let path = get_kit_path();
        assert!(!path.to_string_lossy().contains("~"));
        assert!(path.to_string_lossy().contains(".config/kit"));
        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    #[test]
    fn test_get_kit_path_with_env_var_expansion() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let env_var = "SCRIPT_KIT_TEST_SK_PATH_ROOT";
        std::env::set_var(env_var, "/tmp/script-kit-env-root");
        std::env::set_var(SK_PATH_ENV, format!("${env_var}/kit"));

        let path = get_kit_path();
        assert_eq!(path, PathBuf::from("/tmp/script-kit-env-root/kit"));

        std::env::remove_var(SK_PATH_ENV);
        std::env::remove_var(env_var);
        drop(lock);
    }

    /// Comprehensive setup verification test
    /// Verifies the complete directory structure matches documentation:
    /// ```
    /// ~/.scriptkit/
    /// ├── kit/
    /// │   ├── main/
    /// │   │   ├── scripts/
    /// │   │   ├── extensions/
    /// │   │   └── agents/
    /// │   ├── config.ts
    /// │   ├── theme.json
    /// │   ├── package.json
    /// │   ├── tsconfig.json
    /// │   ├── AGENTS.md
    /// │   └── CLAUDE.md
    /// ├── sdk/
    /// │   └── kit-sdk.ts
    /// ├── db/
    /// ├── logs/
    /// ├── cache/
    /// └── GUIDE.md
    /// ```
    #[test]
    fn test_complete_setup_structure() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = TempDir::new().unwrap();
        // Use a subdirectory that definitely doesn't exist for fresh install detection
        let kit_root = temp_dir.path().join("scriptkit-test");

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

        let result = ensure_kit_setup();
        // Don't assert is_fresh_install - just verify the structure is correct
        assert!(
            result.warnings.is_empty() || !result.warnings.iter().any(|w| w.contains("Failed"))
        );

        // Verify kit/ subdirectory structure
        let kit_dir = kit_root.join("kit");
        assert!(kit_dir.exists(), "kit/ directory should exist");

        // Verify main kit directories
        let main_dir = kit_dir.join("main");
        assert!(
            main_dir.join("scripts").exists(),
            "kit/main/scripts/ should exist"
        );
        assert!(
            main_dir.join("extensions").exists(),
            "kit/main/extensions/ should exist"
        );
        assert!(
            main_dir.join("agents").exists(),
            "kit/main/agents/ should exist"
        );

        // Verify user config files in kit/
        assert!(
            kit_dir.join("config.ts").exists(),
            "kit/config.ts should exist"
        );
        assert!(
            kit_dir.join("theme.json").exists(),
            "kit/theme.json should exist"
        );
        assert!(
            kit_dir.join("package.json").exists(),
            "kit/package.json should exist"
        );
        assert!(
            kit_dir.join("tsconfig.json").exists(),
            "kit/tsconfig.json should exist"
        );
        assert!(
            kit_dir.join("AGENTS.md").exists(),
            "kit/AGENTS.md should exist"
        );
        assert!(
            kit_dir.join("CLAUDE.md").exists(),
            "kit/CLAUDE.md should exist"
        );

        // Verify SDK directory
        assert!(
            kit_root.join("sdk").join("kit-sdk.ts").exists(),
            "sdk/kit-sdk.ts should exist"
        );

        // Verify other directories
        assert!(kit_root.join("db").exists(), "db/ directory should exist");
        assert!(
            kit_root.join("logs").exists(),
            "logs/ directory should exist"
        );
        assert!(
            kit_root.join("cache").exists(),
            "cache/ directory should exist"
        );

        // Verify GUIDE.md at root
        assert!(
            kit_root.join("GUIDE.md").exists(),
            "GUIDE.md should exist at root"
        );

        // Verify sample script on fresh install
        let hello_script = main_dir.join("scripts").join("hello-world.ts");
        assert!(
            hello_script.exists(),
            "hello-world.ts sample script should exist"
        );

        // Verify config.ts content
        let config_content = fs::read_to_string(kit_dir.join("config.ts")).unwrap();
        assert!(
            config_content.contains("@scriptkit/sdk"),
            "config.ts should import @scriptkit/sdk"
        );
        assert!(
            config_content.contains("hotkey"),
            "config.ts should have hotkey config"
        );

        // Verify package.json has correct name and type
        let package_content = fs::read_to_string(kit_dir.join("package.json")).unwrap();
        assert!(
            package_content.contains("@scriptkit/kit"),
            "package.json should have @scriptkit/kit name"
        );
        assert!(
            package_content.contains("\"type\": \"module\""),
            "package.json should enable ESM"
        );

        // kit/AGENTS.md and kit/CLAUDE.md are now redirect stubs
        let kit_agents_content = fs::read_to_string(kit_dir.join("AGENTS.md")).unwrap();
        assert!(
            kit_agents_content.contains("../AGENTS.md"),
            "kit/AGENTS.md should redirect to root"
        );
        let kit_claude_content = fs::read_to_string(kit_dir.join("CLAUDE.md")).unwrap();
        assert!(
            kit_claude_content.contains("../CLAUDE.md"),
            "kit/CLAUDE.md should redirect to root"
        );

        // Root-level AGENTS.md has the actual SDK reference
        let agents_content = fs::read_to_string(kit_root.join("AGENTS.md")).unwrap();
        assert!(
            agents_content.contains("Script Kit"),
            "Root AGENTS.md should mention Script Kit"
        );
        assert!(
            agents_content.contains("~/.scriptkit/kit/config.ts"),
            "Root AGENTS.md should have correct config path"
        );

        // Root-level CLAUDE.md has the actual agent instructions
        let claude_content = fs::read_to_string(kit_root.join("CLAUDE.md")).unwrap();
        assert!(
            claude_content.contains("Script Kit"),
            "Root CLAUDE.md should mention Script Kit"
        );
        assert!(
            claude_content.contains("@scriptkit/sdk"),
            "Root CLAUDE.md should reference the SDK"
        );

        // Verify CleanShot X built-in extension
        let cleanshot_dir = kit_dir.join("cleanshot").join("extensions");
        assert!(
            cleanshot_dir.exists(),
            "kit/cleanshot/extensions/ should exist"
        );
        let cleanshot_extension = cleanshot_dir.join("main.md");
        assert!(
            cleanshot_extension.exists(),
            "kit/cleanshot/extensions/main.md should exist"
        );
        let cleanshot_content = fs::read_to_string(&cleanshot_extension).unwrap();
        assert!(
            cleanshot_content.contains("CleanShot X"),
            "CleanShot extension should have CleanShot X title"
        );
        assert!(
            cleanshot_content.contains("cleanshot://capture-area"),
            "CleanShot extension should have Capture Area command"
        );
        assert!(
            cleanshot_content.contains("cleanshot://record-screen"),
            "CleanShot extension should have Record Screen command"
        );

        // Verify 1Password built-in extension
        let onepassword_dir = kit_dir.join("1password").join("extensions");
        assert!(
            onepassword_dir.exists(),
            "kit/1password/extensions/ should exist"
        );
        let onepassword_extension = onepassword_dir.join("main.md");
        assert!(
            onepassword_extension.exists(),
            "kit/1password/extensions/main.md should exist"
        );
        let onepassword_content = fs::read_to_string(&onepassword_extension).unwrap();
        assert!(
            onepassword_content.contains("1Password"),
            "1Password extension should have 1Password title"
        );
        assert!(
            onepassword_content.contains("op item list"),
            "1Password extension should have item list command"
        );
        assert!(
            onepassword_content.contains("op whoami"),
            "1Password extension should have whoami command"
        );

        // Verify Quick Links built-in extension
        let quicklinks_dir = kit_dir.join("quicklinks").join("extensions");
        assert!(
            quicklinks_dir.exists(),
            "kit/quicklinks/extensions/ should exist"
        );
        let quicklinks_extension = quicklinks_dir.join("main.md");
        assert!(
            quicklinks_extension.exists(),
            "kit/quicklinks/extensions/main.md should exist"
        );
        let quicklinks_content = fs::read_to_string(&quicklinks_extension).unwrap();
        assert!(
            quicklinks_content.contains("Quick Links"),
            "Quick Links extension should have Quick Links title"
        );
        assert!(
            quicklinks_content.contains("https://github.com"),
            "Quick Links extension should have GitHub link"
        );
        assert!(
            quicklinks_content.contains("https://www.google.com"),
            "Quick Links extension should have Google link"
        );

        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }

    /// Test that paths in AGENTS.md match actual setup paths
    #[test]
    fn test_agents_md_paths_match_setup() {
        let lock = crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = TempDir::new().unwrap();
        let kit_root = temp_dir.path().to_path_buf();

        std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());
        let _ = ensure_kit_setup();

        // Root-level AGENTS.md is the canonical location now
        let agents_content = fs::read_to_string(kit_root.join("AGENTS.md")).unwrap();

        // Verify documented paths actually exist
        let documented_paths = [
            ("kit/main/scripts", "~/.scriptkit/kit/main/scripts/"),
            ("kit/main/extensions", "~/.scriptkit/kit/main/extensions/"),
            ("kit/config.ts", "~/.scriptkit/kit/config.ts"),
            ("kit/theme.json", "~/.scriptkit/kit/theme.json"),
            ("sdk/kit-sdk.ts", "~/.scriptkit/sdk/"),
        ];

        for (relative_path, doc_path) in documented_paths {
            assert!(
                agents_content.contains(doc_path),
                "AGENTS.md should document path: {}",
                doc_path
            );

            let actual_path = kit_root.join(relative_path);
            // For directories, check they exist; for files, check the parent exists
            if relative_path.contains('.') {
                assert!(
                    actual_path.exists(),
                    "Documented path {} should exist as file: {:?}",
                    doc_path,
                    actual_path
                );
            } else {
                assert!(
                    actual_path.exists(),
                    "Documented path {} should exist as directory: {:?}",
                    doc_path,
                    actual_path
                );
            }
        }

        std::env::remove_var(SK_PATH_ENV);
        drop(lock);
    }
}

#[cfg(test)]
mod tab_ai_agent_doc_contract_tests {
    const ROOT_CLAUDE: &str = include_str!("../../kit-init/ROOT_CLAUDE.md");
    const ROOT_AGENTS: &str = include_str!("../../kit-init/ROOT_AGENTS.md");
    const REPO_CLAUDE: &str = include_str!("../../CLAUDE.md");
    const REPO_AGENTS: &str = include_str!("../../AGENTS.md");
    const AI_MOD_SOURCE: &str = include_str!("../ai/mod.rs");

    fn assert_tab_ai_doc_contract(source: &str, label: &str) {
        for needle in [
            "Quick Terminal with Flat Context Injection",
            "AppView::QuickTerminalView",
            "TermPrompt",
            "TabAiHarnessSubmissionMode",
            "PasteOnly",
            "Submit",
            "~/.scriptkit/harness.json",
            "CaptureContextOptions::tab_ai_submit()",
            "Cmd+W",
            "Escape",
        ] {
            assert!(source.contains(needle), "{label} must contain `{needle}`");
        }

        assert!(
            source.contains("PTY") || source.contains("pty"),
            "{label} must describe the landed PTY-backed path"
        );

        assert!(
            !source.contains("instant terminal session"),
            "{label} must not describe Claude Code as an instant terminal session"
        );

        if source.contains("TabAiChat") || source.contains("open_tab_ai_full_view_chat") {
            assert!(
                source.contains("Legacy compatibility only"),
                "{label} may mention legacy Tab AI symbols only behind an explicit legacy notice"
            );
        }
    }

    fn assert_tab_ai_schema_detail_contract(source: &str, label: &str) {
        for needle in [
            "`TabAiExecutionRecord` (v2)",
            "Compatibility-only types still present",
            "PanelOnlyElements",
            "CollectorFallback",
            "NoSemanticElements",
            "MissingFocusTarget",
            "InputNotExtractable",
            "InputNotApplicable",
        ] {
            assert!(source.contains(needle), "{label} must contain `{needle}`");
        }

        for stale in [
            "`TabAiExecutionRecord` (v1)",
            "record + status + output + duration",
            "Persisted memory: intent, script, target bundle_id, outcome",
            "PanelOnlyWarning",
            "MissingInput",
        ] {
            assert!(
                !source.contains(stale),
                "{label} contains stale Tab AI schema detail: {stale}"
            );
        }
    }

    #[test]
    fn root_claude_doc_matches_landed_tab_ai_contract() {
        assert_tab_ai_doc_contract(ROOT_CLAUDE, "kit-init/ROOT_CLAUDE.md");
    }

    #[test]
    fn root_agents_doc_matches_landed_tab_ai_contract() {
        assert_tab_ai_doc_contract(ROOT_AGENTS, "kit-init/ROOT_AGENTS.md");
    }

    #[test]
    fn repo_claude_doc_matches_landed_tab_ai_contract() {
        assert_tab_ai_doc_contract(REPO_CLAUDE, "CLAUDE.md");
    }

    #[test]
    fn repo_agents_doc_matches_landed_tab_ai_contract() {
        assert_tab_ai_doc_contract(REPO_AGENTS, "AGENTS.md");
    }

    #[test]
    fn repo_claude_schema_detail_matches_current_tab_ai_types() {
        assert_tab_ai_schema_detail_contract(REPO_CLAUDE, "CLAUDE.md");
    }

    #[test]
    fn repo_agents_schema_detail_matches_current_tab_ai_types() {
        assert_tab_ai_schema_detail_contract(REPO_AGENTS, "AGENTS.md");
    }

    #[test]
    fn ai_mod_docs_reflect_tab_ai_harness_primary_path() {
        for needle in [
            "//! AI surfaces and shared contracts.",
            "//! - Submission shape: flat text-native",
            "//! - Surface: `AppView::QuickTerminalView` rendered by `TermPrompt`",
            "//! - Legacy chat/window code still exists, but it is not the default Tab AI surface.",
        ] {
            assert!(
                AI_MOD_SOURCE.contains(needle),
                "src/ai/mod.rs docs must contain `{needle}`"
            );
        }

        assert!(
            !AI_MOD_SOURCE.contains("//! AI Chat Module"),
            "src/ai/mod.rs must not describe the module as chat-only anymore"
        );
    }
}

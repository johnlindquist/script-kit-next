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
/// │   ├── AGENTS.md              # AI agent guide (SDK documentation)
/// │   └── CLAUDE.md              # Claude-specific instructions
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

    // User guide: AGENTS.md for AI agents writing scripts (in kit/ directory)
    let agents_md_path = kit_dir.join("kit").join("AGENTS.md");
    write_string_if_missing(
        &agents_md_path,
        EMBEDDED_AGENTS_MD,
        &mut warnings,
        "kit/AGENTS.md",
    );

    // Claude-specific instructions (in kit/ directory)
    let claude_md_path = kit_dir.join("kit").join("CLAUDE.md");
    write_string_if_missing(
        &claude_md_path,
        EMBEDDED_CLAUDE_MD,
        &mut warnings,
        "kit/CLAUDE.md",
    );

    // User-owned: GUIDE.md (only create if missing)
    // Comprehensive user guide for learning Script Kit
    let guide_md_path = kit_dir.join("GUIDE.md");
    write_string_if_missing(&guide_md_path, EMBEDDED_GUIDE_MD, &mut warnings, "GUIDE.md");

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

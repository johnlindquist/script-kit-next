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
/// Embedded user-facing command line shim
const EMBEDDED_SCRIPTKIT_CLI: &str = include_str!("../../scripts/mcp-cli.ts");
/// Optional theme example (included at compile time)
const EMBEDDED_THEME_EXAMPLE: &str = include_str!("../../kit-init/theme.example.json");
pub(crate) const EMBEDDED_TIPS: &str = include_str!("../../kit-init/tips.json");
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
    include_str!("../../kit-init/scriptlets/cleanshot/main.md");
/// Embedded CleanShot X shared actions (built-in actions for all cleanshot scriptlets)
const EMBEDDED_CLEANSHOT_ACTIONS: &str =
    include_str!("../../kit-init/scriptlets/cleanshot/main.actions.md");
/// Embedded 1Password extension (built-in extension that ships with the app)
const EMBEDDED_1PASSWORD_EXTENSION: &str =
    include_str!("../../kit-init/scriptlets/1password/main.md");
/// Embedded Quick Links extension (built-in extension that ships with the app)
const EMBEDDED_QUICKLINKS_EXTENSION: &str =
    include_str!("../../kit-init/scriptlets/quicklinks/main.md");
/// Embedded Quick Links shared actions (built-in actions for all quicklinks scriptlets)
const EMBEDDED_QUICKLINKS_ACTIONS: &str =
    include_str!("../../kit-init/scriptlets/quicklinks/main.actions.md");
/// Embedded Window Management extension (built-in extension that ships with the app)
const EMBEDDED_WINDOW_MANAGEMENT_EXTENSION: &str =
    include_str!("../../kit-init/scriptlets/window-management/main.md");
/// Embedded AI Text Tools extension (built-in extension that ships with the app)
const EMBEDDED_AI_TEXT_TOOLS_EXTENSION: &str =
    include_str!("../../kit-init/scriptlets/ai-text-tools/main.md");
/// Root-level CLAUDE.md for the ~/.scriptkit workspace (the harness cwd)
const EMBEDDED_ROOT_CLAUDE_MD: &str = include_str!("../../kit-init/ROOT_CLAUDE.md");
/// Root-level AGENTS.md SDK reference for the ~/.scriptkit workspace
const EMBEDDED_ROOT_AGENTS_MD: &str = include_str!("../../kit-init/ROOT_AGENTS.md");
/// Skills README
const EMBEDDED_SKILLS_README: &str = include_str!("../../kit-init/skills/README.md");
/// Skill: new script
const EMBEDDED_SKILL_NEW_SCRIPT: &str = include_str!("../../kit-init/skills/new-script/SKILL.md");
/// Skill: new scriptlet
const EMBEDDED_SKILL_NEW_SCRIPTLET: &str =
    include_str!("../../kit-init/skills/new-scriptlet/SKILL.md");
/// Skill: config updates
const EMBEDDED_SKILL_UPDATE_CONFIG: &str =
    include_str!("../../kit-init/skills/update-config/SKILL.md");
/// Skill: external MCP server configuration
const EMBEDDED_SKILL_CONFIGURE_MCP: &str =
    include_str!("../../kit-init/skills/configure-mcp/SKILL.md");
/// Skill: troubleshooting
const EMBEDDED_SKILL_TROUBLESHOOTING: &str =
    include_str!("../../kit-init/skills/troubleshoot/SKILL.md");
/// Example script: Todo app
const EMBEDDED_EXAMPLE_TODO_APP: &str = include_str!("../../kit-init/examples/scripts/todo-app.ts");
/// Canonical menu syntax handler: todo inbox
const EMBEDDED_CANONICAL_CAPTURE_TODO_INBOX: &str =
    include_str!("../../scripts/examples/menu-syntax/capture-todo-inbox.ts");
/// Canonical menu syntax handler: calendar event
const EMBEDDED_CANONICAL_CREATE_CALENDAR_EVENT: &str =
    include_str!("../../scripts/examples/menu-syntax/create-calendar-event.ts");
/// Canonical menu syntax handler: macOS Calendar event
const EMBEDDED_CANONICAL_CREATE_MAC_CALENDAR_EVENT: &str =
    include_str!("../../scripts/examples/menu-syntax/create-mac-calendar-event.ts");
/// Canonical menu syntax handler: Google Calendar event
const EMBEDDED_CANONICAL_ADD_GOOGLE_CALENDAR_EVENT: &str =
    include_str!("../../scripts/examples/menu-syntax/add-google-calendar-event.ts");
/// Canonical menu syntax handler: reminder
const EMBEDDED_CANONICAL_CREATE_REMINDER: &str =
    include_str!("../../scripts/examples/menu-syntax/create-reminder.ts");
/// Canonical menu syntax handler: snooze task
const EMBEDDED_CANONICAL_SNOOZE_TASK: &str =
    include_str!("../../scripts/examples/menu-syntax/snooze-task.ts");
/// Canonical menu syntax handler: defer task
const EMBEDDED_CANONICAL_DEFER_TASK: &str =
    include_str!("../../scripts/examples/menu-syntax/defer-task.ts");
/// Canonical menu syntax handler: daily note
const EMBEDDED_CANONICAL_APPEND_DAILY_NOTE: &str =
    include_str!("../../scripts/examples/menu-syntax/append-daily-note.ts");
/// Canonical menu syntax handler: social draft
const EMBEDDED_CANONICAL_DRAFT_SOCIAL_POST: &str =
    include_str!("../../scripts/examples/menu-syntax/draft-social-post.ts");
/// Canonical menu syntax handler: tagged link
const EMBEDDED_CANONICAL_SAVE_TAGGED_LINK: &str =
    include_str!("../../scripts/examples/menu-syntax/save-tagged-link.ts");
/// Examples README
const EMBEDDED_EXAMPLES_README: &str = include_str!("../../kit-init/examples/README.md");
/// Examples START_HERE launchpad
const EMBEDDED_EXAMPLES_START_HERE: &str = include_str!("../../kit-init/examples/START_HERE.md");
/// Skill: notes — working with the Notes window and automation targets
const EMBEDDED_SKILL_MANAGE_NOTES: &str =
    include_str!("../../kit-init/skills/manage-notes/SKILL.md");
/// Skill: new agent (compatibility — skills are now the preferred reusable AI unit)
const EMBEDDED_SKILL_NEW_AGENT: &str = include_str!("../../kit-init/skills/new-agent/SKILL.md");
/// Skill: Agent Chat — programmatic chat flows, typed context parts, streaming, and lifecycle
const EMBEDDED_SKILL_START_CHAT: &str = include_str!("../../kit-init/skills/start-chat/SKILL.md");
/// Skill: custom actions — Actions Menu commands in scripts and companion .actions.md files
const EMBEDDED_SKILL_ADD_ACTIONS: &str = include_str!("../../kit-init/skills/add-actions/SKILL.md");
/// Default Agent Chat agent catalog (seeded on first run — provider/catalog selection, not plugin skills)
const EMBEDDED_AGENT_CHAT_AGENTS_JSON: &str = r#"{
  "schemaVersion": 1,
  "agents": [
    {
      "id": "opencode",
      "displayName": "OpenCode",
      "command": "opencode",
      "args": ["agent_chat"],
      "env": {},
      "models": [],
      "install": {
        "command": "npm",
        "args": ["install", "-g", "opencode-ai"]
      }
    },
    {
      "id": "codex-agent_chat",
      "displayName": "Codex",
      "command": "codex-agent_chat",
      "args": [],
      "env": {},
      "models": [],
      "auth": {
        "summary": "Authenticate with ChatGPT, CODEX_API_KEY, or OPENAI_API_KEY."
      }
    }
  ]
}"#;
// --- merged from part_001.rs ---
// (Old kit-level agents doc constant removed — canonical docs live at root level)
// --- merged from part_002.rs ---
// (Old kit-level claude doc constant removed — canonical docs live at root level)
/// Environment variable to override the default ~/.scriptkit path
pub const SK_PATH_ENV: &str = "SK_PATH";
/// Result of setup process
#[derive(Debug)]
pub struct SetupResult {
    /// Whether this run created a new, non-migrated workspace root.
    pub is_fresh_install: bool,
    /// Path to ~/.scriptkit (or SK_PATH override, or fallback if home dir couldn't be resolved)
    pub kit_path: PathBuf,
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

/// Resolve Bun for tools without executing config.ts or consulting a login shell.
pub fn resolve_bun() -> Option<PathBuf> {
    if matches!(
        std::env::var(SK_PATH_ENV),
        Err(std::env::VarError::NotUnicode(_))
    ) {
        return None;
    }
    crate::config::passive_bun()
}

mod private_ignores;

/// Container for all Script Kit plugins under the active workspace.
pub fn plugins_path() -> PathBuf {
    get_kit_path().join("plugins")
}

/// User-owned TypeScript configuration file under the active workspace.
pub fn config_ts_path() -> PathBuf {
    get_kit_path().join("config.ts")
}

const SETTINGS_MODULES: &[(&str, &str)] = &[
    ("config-cli.ts", include_str!("../../scripts/config-cli.ts")),
    (
        "config-document.ts",
        include_str!("../../scripts/config-document.ts"),
    ),
    (
        "config-schema.ts",
        include_str!("../../scripts/config-schema.ts"),
    ),
    (
        "kit-sdk-config.ts",
        include_str!("../../scripts/kit-sdk-config.ts"),
    ),
];

/// Run the installed, version-matched settings helper. Never discover code
/// via the launching application's CWD (including its Bun project files).
/// In particular, a read-only MCP call must not load config.ts to find Bun.
pub fn settings_cli_command(configured_bun: Option<&str>) -> Result<std::process::Command, String> {
    let cli = std::env::var_os("SCRIPT_KIT_CONFIG_CLI_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| get_kit_path().join("sdk/config-cli.ts"));
    if !cli.is_absolute() || !cli.is_file() {
        return Err(format!(
            "Missing or non-absolute settings helper: {}",
            cli.display()
        ));
    }
    let sdk = cli.parent().ok_or("Invalid settings helper path")?;
    // An interrupted upgrade (or a stale development override) is not an
    // invitation to execute a different version's policy or schema.
    for &(name, source) in SETTINGS_MODULES {
        if name == "kit-sdk-config.ts" {
            continue;
        } // not loaded by the CLI
        let file = if name == "config-cli.ts" {
            cli.clone()
        } else {
            sdk.join(name)
        };
        let matches = fs::symlink_metadata(&file)
            .is_ok_and(|m| m.is_file() && m.len() == source.len() as u64)
            && fs::read(&file).is_ok_and(|bytes| bytes == source.as_bytes());
        if !matches {
            return Err(format!(
                "Unverified settings helper {name}; rerun workspace setup"
            ));
        }
    }
    let config = std::env::var_os("SCRIPT_KIT_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(config_ts_path);
    if !config.is_absolute() {
        return Err("Settings require an absolute config path".into());
    }
    let bun = match configured_bun {
        Some(pin) => crate::config::resolve_bun(Some(pin)).map_err(|error| error.to_string())?,
        None => resolve_bun().ok_or("Bun is unavailable")?,
    };
    let mut command = std::process::Command::new(bun);
    command.env_clear(); // no NODE_OPTIONS, loader hooks, BUN_OPTIONS, DYLD_* or PATH
    for key in ["HOME", "SCRIPT_KIT_NONINTERACTIVE", "CI"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .current_dir(sdk)
        .env("SCRIPT_KIT_CONFIG_PATH", config)
        .arg(if cfg!(windows) {
            "--config=NUL"
        } else {
            "--config=/dev/null"
        })
        .args(["--no-install", "--no-env-file"])
        .arg(cli);
    Ok(command)
}

/// User-owned theme override file under the active workspace.
pub fn theme_json_path() -> PathBuf {
    get_kit_path().join("theme.json")
}

/// User-authored theme presets directory under the active workspace.
pub fn themes_dir() -> PathBuf {
    get_kit_path().join("themes")
}
/// Migrate from legacy ~/.kenv to new ~/.scriptkit structure
///
/// This function handles one-time migration from the old directory structure:
/// - Requires a missing default workspace and an entirely recognized legacy tree
/// - Keeps imported logs in ~/.scriptkit/logs/legacy-v1, separate from live logs
/// - Moves ~/.kenv/scripts to ~/.scriptkit/plugins/main/scripts
/// - Moves ~/.kenv/scriptlets to ~/.scriptkit/plugins/main/scriptlets
/// - Creates a symlink ~/.kenv -> ~/.scriptkit for backwards compatibility
///
/// Returns true if migration was performed, false if not needed
#[instrument(level = "info", name = "migrate_from_kenv")]
pub fn migrate_from_kenv() -> bool {
    migration_home().is_some_and(|home| migrate_from_kenv_in(&home, &get_kit_path()))
}

mod legacy_move;

fn migration_home() -> Option<PathBuf> {
    if crate::runtime_policy::is_owned_evaluation()
        || matches!(std::env::var(SK_PATH_ENV), Err(std::env::VarError::NotUnicode(_)))
    {
        None
    } else {
        dirs::home_dir()
    }
}

fn migrate_from_kenv_in(home: &Path, kit_path: &Path) -> bool {
    let mut warnings = Vec::new();
    let result = import_legacy(home, kit_path, &mut warnings);
    for message in warnings {
        warn!("{message}");
    }
    match result {
        Ok(root) => root.is_some(),
        Err(error) => {
            warn!(%error, "Legacy import stopped");
            false
        }
    }
}

// No plan leaves the legacy kit where it is. Return the actual destination
// handle on success. An interrupted move stops this launch before defaults or services.
const IMPORT_MARKER: &str = ".scriptkit-v1-import-in-progress";

fn check_import_idle(home: &Path) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    match fs::symlink_metadata(home.join(IMPORT_MARKER)) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
        Ok(_) => Err(Error::new(ErrorKind::WouldBlock, "A legacy import is active or was interrupted. Check ~/.kenv and ~/.scriptkit before starting Kit again.")),
    }
}

fn import_legacy(home: &Path, kit_path: &Path, warnings: &mut Vec<String>) -> std::io::Result<Option<legacy_move::Directory>> {
    use std::ffi::OsString;
    use std::io::{self, ErrorKind};
    let old_kenv = home.join(".kenv");
    if !home.is_absolute() || kit_path != home.join(".scriptkit") { return Ok(None); }
    check_import_idle(home)?;
    if !matches!(fs::symlink_metadata(kit_path), Err(e) if e.kind() == ErrorKind::NotFound)
        || !fs::symlink_metadata(&old_kenv).is_ok_and(|m| m.is_dir())
    {
        return Ok(None);
    }
    struct Entry { name: OsString, original: fs::Metadata, plugin: bool, archive: bool }
    let legacy = legacy_move::Directory::open(&old_kenv)?;
    let plan = (|| -> io::Result<Vec<Entry>> {
        let mut moves = Vec::new();
        for entry in fs::read_dir(&old_kenv)? {
            let entry = entry?;
            let name = entry.file_name();
            let original = fs::symlink_metadata(entry.path())?;
            let (plugin, archive) = match name.to_str() {
                Some("scripts" | "scriptlets") if original.is_dir() => (true, false),
                // The current session truncates logs/latest-session.jsonl. Never
                // make an imported log the destination of a new logging session.
                Some("logs") if original.is_dir() => (false, true),
                Some("cache" | "db" | "sdk") if original.is_dir() => (false, false),
                Some("config.ts" | "theme.json" | "tsconfig.json" | "package.json"
                    | "settings.json" | ".gitignore" | "frecency.json" | "store.json"
                    | "server.json" | "agent-token" | "notes.db" | "ai-chats.db"
                    | "clipboard-history.db") if original.is_file() => (false, false),
                _ => return Err(io::Error::other("unrecognized or indirect legacy entry")),
            };
            moves.push(Entry { name, original, plugin, archive });
        }
        for (i, entry) in walkdir::WalkDir::new(&old_kenv).follow_links(false).into_iter().enumerate() {
            let entry = entry?;
            let kind = entry.file_type();
            if i >= 16_384 || (!kind.is_file() && !kind.is_dir()) {
                return Err(io::Error::other("large or indirect legacy tree"));
            }
            #[cfg(unix)]
            if kind.is_file() {
                use std::os::unix::fs::MetadataExt;
                if fs::symlink_metadata(entry.path())?.nlink() != 1 {
                    return Err(io::Error::other("linked legacy file"));
                }
            }
        }
        legacy.check()?;
        Ok(moves)
    })();
    let Ok(plan) = plan else {
        warnings.push("Legacy workspace left intact; import its scripts manually".into());
        return Ok(None);
    };
    if plan.is_empty() { return Ok(None); }
    if !cfg!(any(target_os = "macos", target_os = "linux", windows)) {
        warnings.push("Automatic legacy import is unavailable; import scripts manually".into());
        return Ok(None);
    }
    // Publish the hold BEFORE the destination exists. A second default launch
    // must not see a half-imported kit as an ordinary upgrade and seed defaults.
    // Do not remove this on Drop: a crash or transfer error needs human recovery.
    let marker_path = home.join(IMPORT_MARKER);
    create_kit_dir(&marker_path).map_err(|e| if e.kind() == ErrorKind::AlreadyExists {
        io::Error::new(ErrorKind::WouldBlock, "Another legacy import is active or needs recovery")
    } else { e })?;
    let marker = legacy_move::Directory::open(&marker_path)?;
    let release_marker = || { marker.check()?; fs::remove_dir(&marker_path) };
    // No legacy entry moved if another launch or editor won the destination.
    if let Err(e) = create_kit_dir(kit_path) {
        release_marker()?;
        return if e.kind() == ErrorKind::AlreadyExists { Ok(None) } else { Err(e) };
    }
    let kit = legacy_move::Directory::open(kit_path)?;
    let transfer = (|| -> io::Result<()> {
        let plugins_path = kit_path.join("plugins");
        create_kit_dir(&plugins_path)?;
        let plugins = legacy_move::Directory::open(&plugins_path)?;
        let main_path = plugins_path.join("main");
        create_kit_dir(&main_path)?;
        let main = legacy_move::Directory::open(&main_path)?;
        for entry in plan {
            marker.check()?;
            kit.check()?;
            plugins.check()?;
            if entry.archive {
                // Keep archived diagnostics inside the normal log exclusion.
                let log_path = kit_path.join("logs");
                create_kit_dir(&log_path)?;
                let logs = legacy_move::Directory::open(&log_path)?;
                // Even a v1 kit with a custom root .gitignore should not offer
                // the archive's private contents to an ordinary git add.
                let ignore = logs.path().join(".gitignore");
                use std::io::Write as _;
                fs::File::create_new(ignore)?.write_all(b"*\n")?;
                legacy.move_entry(&entry.name, &entry.original, &logs, "legacy-v1".as_ref())?;
            } else {
                legacy.move_entry(&entry.name, &entry.original, if entry.plugin { &main } else { &kit }, &entry.name)?;
            }
        }
        kit.check()?;
        plugins.check()?;
        main.check()?;
        legacy.check()?;
        // Never recursively remove a legacy root. A newly arrived entry stops here.
        fs::remove_dir(&old_kenv)
    })();
    transfer.map_err(|e| io::Error::new(e.kind(), format!("Legacy import stopped. Inspect both ~/.kenv and ~/.scriptkit before another launch: {e}")))?;
    #[cfg(unix)]
    if let Err(e) = std::os::unix::fs::symlink(kit_path, &old_kenv) {
        warnings.push(format!("Imported the legacy files but could not create the ~/.kenv shortcut: {e}"));
    }
    kit.check()?;
    release_marker()?;
    Ok(Some(kit))
}

fn create_kit_dir(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

// Only the launch that actually creates the leaf gets to seed first-run files.
fn claim_kit_root(path: &Path) -> std::io::Result<bool> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    match create_kit_dir(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => Ok(false),
        Err(e) => Err(e),
    }
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
/// │   │   ├── scriptlets/         # Markdown scriptlet bundles
/// │   │   └── agents/             # AI agent definitions (.md)
/// │   └── custom-kit/            # Additional custom kits
/// │       ├── scripts/
/// │       ├── scriptlets/
/// │       └── agents/
/// │   ├── package.json           # Node.js module config (type: module for top-level await)
/// │   └── tsconfig.json          # TypeScript path mappings
/// │   ├── config.ts              # User configuration (created from template if missing)
/// │   ├── theme.json             # Theme configuration (created from example if missing)
/// │   ├── AGENTS.md              # Redirect stub → ../AGENTS.md
/// │   └── CLAUDE.md              # Redirect stub → ../CLAUDE.md
/// ├── sdk/                       # Runtime SDK (kit-sdk.ts)
/// ├── bin/                       # App-managed command shims
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
    let kit_dir = get_kit_path();
    setup_workspace(&kit_dir, migration_home().as_deref(), false, || {}).unwrap_or_else(|error| SetupResult {
        is_fresh_install: false,
        kit_path: kit_dir,
        warnings: vec![format!("Could not prepare the workspace: {error}")],
    })
}

/// Normal app launch. The logger and PID registry remain per-home, not in an
/// arbitrary SK_PATH. No callback or service touches the default root before
/// migration and fresh-root selection have settled.
pub fn prepare_workspace_for_launch(start_diagnostics: impl FnOnce()) -> std::io::Result<SetupResult> {
    setup_before_diagnostics(&get_kit_path(), migration_home().as_deref(), start_diagnostics)
}

fn setup_before_diagnostics(kit_dir: &Path, legacy_home: Option<&Path>, start_diagnostics: impl FnOnce()) -> std::io::Result<SetupResult> {
    setup_workspace(kit_dir, legacy_home, true, start_diagnostics)
}

fn setup_workspace(kit_dir: &Path, home: Option<&Path>, allow_import: bool, start_diagnostics: impl FnOnce()) -> std::io::Result<SetupResult> {
    let default_home = home.filter(|h| h.is_absolute() && kit_dir == h.join(".scriptkit"));
    let mut warnings = Vec::new();
    if let Some(home) = default_home { check_import_idle(home)?; }
    let imported = if let Some(home) = default_home.filter(|_| allow_import) { import_legacy(home, kit_dir, &mut warnings)? } else { None };
    let migrated = imported.is_some();
    let (fresh, claimed) = if let Some(root) = imported {
        (false, Some(root))
    } else {
        let fresh = claim_kit_root(kit_dir)?;
        (fresh, fresh.then(|| legacy_move::Directory::open(kit_dir)).transpose()?)
    };
    // Pair the preflight with a check AFTER observing or claiming the leaf. A
    // concurrent importer can have created both its hold and its root in between.
    if let Some(home) = default_home { check_import_idle(home)?; }
    start_diagnostics();
    if let Some(claimed) = claimed { claimed.check()?; }
    if migrated { info!("Migrated from ~/.kenv to ~/.scriptkit"); }
    Ok(install_workspace_files(kit_dir.to_owned(), fresh, warnings))
}

fn install_workspace_files(kit_dir: PathBuf, is_fresh_install: bool, mut warnings: Vec<String>) -> SetupResult {
    if std::env::var(SK_PATH_ENV).is_ok() {
        info!(kit_path = %kit_dir.display(), "Using SK_PATH override");
    }

    // Seed the user-themes directory so save/load operations don't race the
    // first-use creation on the hot path.
    if let Err(e) = crate::theme::user_themes::ensure_user_themes_dir() {
        warnings.push(format!("Failed to create user themes dir: {e}"));
    }

    // Plugin roots: each plugin gets {scripts,scriptlets,agents,skills} + plugin.json
    ensure_plugin_root(
        &kit_dir,
        "main",
        "Main",
        "Default personal plugin",
        &mut warnings,
    );
    ensure_plugin_root(
        &kit_dir,
        "cleanshot",
        "CleanShot X",
        "Built-in screenshot commands",
        &mut warnings,
    );
    ensure_plugin_root(
        &kit_dir,
        "1password",
        "1Password",
        "Built-in password-manager commands",
        &mut warnings,
    );
    ensure_plugin_root(
        &kit_dir,
        "quicklinks",
        "Quick Links",
        "Built-in quick link commands",
        &mut warnings,
    );
    ensure_plugin_root(
        &kit_dir,
        "window-management",
        "Window Management",
        "Built-in window commands",
        &mut warnings,
    );
    ensure_plugin_root(
        &kit_dir,
        "ai-text-tools",
        "AI Text Tools",
        "Built-in text tools",
        &mut warnings,
    );
    ensure_plugin_root(
        &kit_dir,
        "examples",
        "Examples",
        "Built-in examples",
        &mut warnings,
    );
    ensure_plugin_root(
        &kit_dir,
        "scriptkit",
        "Script Kit",
        "Built-in Script Kit skills and references",
        &mut warnings,
    );

    // Migrate legacy plugin-scoped extensions/ directories to scriptlets/.
    migrate_plugin_extensions_to_scriptlets(&kit_dir, &mut warnings);

    // Non-plugin infrastructure directories
    let required_dirs = [
        // Context bundle directory for Tab AI argv-based launch
        kit_dir.join("context"),
        // Root-level harness temp workspace used by kit://sdk-reference
        kit_dir.join("tmp").join("test-scripts"),
        kit_dir.join("tmp").join("test-scriptlets"),
        kit_dir.join("sdk"),
        kit_dir.join("bin"),
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

    // Embedded in the executable, so first run and upgrades do not require
    // a checkout, node_modules, npm installation, or a writable .app bundle.
    for &(name, source) in SETTINGS_MODULES {
        write_string_if_changed(&kit_dir.join("sdk").join(name), source, &mut warnings, name);
    }

    // App-managed: user-facing `scriptkit` command shim (refresh if changed)
    let agent_docs = kit_dir.join("share/external-agent");
    ensure_dir(&agent_docs, &mut warnings);
    write_string_if_changed(
        &agent_docs.join("manual.md"),
        include_str!("../../kit-init/agent-manual.md"),
        &mut warnings,
        "share/external-agent/manual.md",
    );
    write_string_if_changed(
        &agent_docs.join("SKILL.md"),
        include_str!("../../skills/script-kit/SKILL.md"),
        &mut warnings,
        "share/external-agent/SKILL.md",
    );
    write_string_if_changed(
        &kit_dir.join("bin/agent-bridge-cli.ts"),
        include_str!("../../scripts/agent-bridge-cli.ts"),
        &mut warnings,
        "bin/agent-bridge-cli.ts",
    );
    write_string_if_changed(
        &agent_docs.join("prepare-handout.ts"),
        include_str!("../../kit-init/examples/scripts/prepare-handout.ts"),
        &mut warnings,
        "share/external-agent/prepare-handout.ts",
    );
    let scriptkit_cli_path = kit_dir.join("bin").join("scriptkit");
    write_executable_string_if_changed(
        &scriptkit_cli_path,
        EMBEDDED_SCRIPTKIT_CLI,
        &mut warnings,
        "bin/scriptkit",
    );

    let plugins_dir = kit_dir.join("plugins");

    // App-managed: Built-in CleanShot X extension (refresh if changed)
    // This extension ships with the app and provides screenshot/recording commands
    let cleanshot_path = kit_dir
        .join("plugins")
        .join("cleanshot")
        .join("scriptlets")
        .join("main.md");
    write_string_if_changed(
        &cleanshot_path,
        EMBEDDED_CLEANSHOT_EXTENSION,
        &mut warnings,
        "plugins/cleanshot/scriptlets/main.md",
    );

    // App-managed: Built-in CleanShot X shared actions (refresh if changed)
    // These actions are automatically available for all CleanShot scriptlets
    let cleanshot_actions_path = kit_dir
        .join("plugins")
        .join("cleanshot")
        .join("scriptlets")
        .join("main.actions.md");
    write_string_if_changed(
        &cleanshot_actions_path,
        EMBEDDED_CLEANSHOT_ACTIONS,
        &mut warnings,
        "plugins/cleanshot/scriptlets/main.actions.md",
    );

    // App-managed: Built-in 1Password extension (refresh if changed)
    // This extension ships with the app and provides password manager CLI commands
    let onepassword_path = kit_dir
        .join("plugins")
        .join("1password")
        .join("scriptlets")
        .join("main.md");
    write_string_if_changed(
        &onepassword_path,
        EMBEDDED_1PASSWORD_EXTENSION,
        &mut warnings,
        "plugins/1password/scriptlets/main.md",
    );

    // App-managed: Built-in Quick Links extension (refresh if changed)
    // This extension ships with the app and provides quick access to common websites
    let quicklinks_path = kit_dir
        .join("plugins")
        .join("quicklinks")
        .join("scriptlets")
        .join("main.md");
    write_string_if_changed(
        &quicklinks_path,
        EMBEDDED_QUICKLINKS_EXTENSION,
        &mut warnings,
        "plugins/quicklinks/scriptlets/main.md",
    );

    // App-managed: Built-in Quick Links shared actions (refresh if changed)
    // These actions are automatically available for all Quick Links scriptlets
    let quicklinks_actions_path = kit_dir
        .join("plugins")
        .join("quicklinks")
        .join("scriptlets")
        .join("main.actions.md");
    write_string_if_changed(
        &quicklinks_actions_path,
        EMBEDDED_QUICKLINKS_ACTIONS,
        &mut warnings,
        "plugins/quicklinks/scriptlets/main.actions.md",
    );

    // App-managed: Built-in Window Management extension (refresh if changed)
    // This extension ships with the app and provides window tiling and positioning
    let window_management_path = kit_dir
        .join("plugins")
        .join("window-management")
        .join("scriptlets")
        .join("main.md");
    write_string_if_changed(
        &window_management_path,
        EMBEDDED_WINDOW_MANAGEMENT_EXTENSION,
        &mut warnings,
        "plugins/window-management/scriptlets/main.md",
    );

    // App-managed: Built-in AI Text Tools extension (refresh if changed)
    // This extension ships with the app and provides AI-powered text transformations
    let ai_text_tools_path = kit_dir
        .join("plugins")
        .join("ai-text-tools")
        .join("scriptlets")
        .join("main.md");
    write_string_if_changed(
        &ai_text_tools_path,
        EMBEDDED_AI_TEXT_TOOLS_EXTENSION,
        &mut warnings,
        "plugins/ai-text-tools/scriptlets/main.md",
    );

    // User-owned: config.ts (only create if missing)
    let config_path = kit_dir.join("config.ts");
    write_string_if_missing(
        &config_path,
        EMBEDDED_CONFIG_TEMPLATE,
        &mut warnings,
        "config.ts",
    );

    // User-owned (optional): theme.json (only create if missing)
    let theme_path = kit_dir.join("theme.json");
    write_string_if_missing(
        &theme_path,
        EMBEDDED_THEME_EXAMPLE,
        &mut warnings,
        "theme.json",
    );

    // App-managed: the Help catalog. Its keys and places follow the app, so a copy written
    // by an older version would describe keys that do something else now.
    write_string_if_changed(
        &kit_dir.join("tips.json"),
        EMBEDDED_TIPS,
        &mut warnings,
        "tips.json",
    );

    // App-managed: tsconfig.json path mappings in the workspace root (merge-safe)
    ensure_tsconfig_paths(&kit_dir.join("tsconfig.json"), &mut warnings);

    // App-managed: package.json for top-level await support in plugin scripts
    let package_json_path = kit_dir.join("package.json");
    write_string_if_missing(
        &package_json_path,
        EMBEDDED_PACKAGE_JSON,
        &mut warnings,
        "package.json",
    );

    // User-owned: GUIDE.md (only create if missing)
    // Comprehensive user guide for learning Script Kit
    let guide_md_path = kit_dir.join("GUIDE.md");
    write_string_if_missing(&guide_md_path, EMBEDDED_GUIDE_MD, &mut warnings, "GUIDE.md");

    // User-owned: Agent Chat agent catalog (only create if missing)
    // Users add/edit Agent Chat agent entries here for multi-agent support
    let agent_chat_agents_path = kit_dir.join("agent_chat").join("agents.json");
    write_string_if_missing(
        &agent_chat_agents_path,
        EMBEDDED_AGENT_CHAT_AGENTS_JSON,
        &mut warnings,
        "agent_chat/agents.json",
    );

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

    // App-managed: Skills library — seeded into the Script Kit plugin (refresh if changed)
    let scriptkit_skills = plugins_dir.join("scriptkit").join("skills");
    write_string_if_changed(
        &scriptkit_skills.join("README.md"),
        EMBEDDED_SKILLS_README,
        &mut warnings,
        "plugins/scriptkit/skills/README.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("new-script").join("SKILL.md"),
        EMBEDDED_SKILL_NEW_SCRIPT,
        &mut warnings,
        "plugins/scriptkit/skills/new-script/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("new-scriptlet").join("SKILL.md"),
        EMBEDDED_SKILL_NEW_SCRIPTLET,
        &mut warnings,
        "plugins/scriptkit/skills/new-scriptlet/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("update-config").join("SKILL.md"),
        EMBEDDED_SKILL_UPDATE_CONFIG,
        &mut warnings,
        "plugins/scriptkit/skills/update-config/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("configure-mcp").join("SKILL.md"),
        EMBEDDED_SKILL_CONFIGURE_MCP,
        &mut warnings,
        "plugins/scriptkit/skills/configure-mcp/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("troubleshoot").join("SKILL.md"),
        EMBEDDED_SKILL_TROUBLESHOOTING,
        &mut warnings,
        "plugins/scriptkit/skills/troubleshoot/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("manage-notes").join("SKILL.md"),
        EMBEDDED_SKILL_MANAGE_NOTES,
        &mut warnings,
        "plugins/scriptkit/skills/manage-notes/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("new-agent").join("SKILL.md"),
        EMBEDDED_SKILL_NEW_AGENT,
        &mut warnings,
        "plugins/scriptkit/skills/new-agent/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("start-chat").join("SKILL.md"),
        EMBEDDED_SKILL_START_CHAT,
        &mut warnings,
        "plugins/scriptkit/skills/start-chat/SKILL.md",
    );
    write_string_if_changed(
        &scriptkit_skills.join("add-actions").join("SKILL.md"),
        EMBEDDED_SKILL_ADD_ACTIONS,
        &mut warnings,
        "plugins/scriptkit/skills/add-actions/SKILL.md",
    );

    // App-managed: Examples plugin. Keep this intentionally small: one real
    // Todo app example plus a short README/launchpad.
    let examples_plugin = plugins_dir.join("examples");
    prune_managed_examples_plugin(&examples_plugin, &mut warnings);
    write_string_if_changed(
        &examples_plugin.join("README.md"),
        EMBEDDED_EXAMPLES_README,
        &mut warnings,
        "plugins/examples/README.md",
    );
    write_string_if_changed(
        &examples_plugin.join("START_HERE.md"),
        EMBEDDED_EXAMPLES_START_HERE,
        &mut warnings,
        "plugins/examples/START_HERE.md",
    );
    write_string_if_changed(
        &examples_plugin.join("scripts").join("todo-app.ts"),
        EMBEDDED_EXAMPLE_TODO_APP,
        &mut warnings,
        "plugins/examples/scripts/todo-app.ts",
    );

    // Seed .gitignore once; local additions may protect personal files.
    let gitignore_path = kit_dir.join(".gitignore");
    let gitignore_content = r#"# Script Kit managed .gitignore
# Personal additions are preserved when Kit starts.

# =============================================================================
# Node.js / Bun dependencies
# =============================================================================
# Root node_modules (for package.json at ~/.scriptkit/plugins/)
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

# Command shims are managed by the app, always regenerated
bin/

# Application logs
logs/

# Cache files (app icons, etc.)
cache/

# Frecency tracking (regenerated from usage)
frecency.json

# Server state
server.json

# Authentication tokens and encrypted stores
agent-token
secrets.age
.secrets-*
agent_chat/auth/
*.p12
*.pfx

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
    // This is a user-owned source of exclusions once created. The backup implementation
    // independently allowlists personal files; it does not rely on mutable ignore rules.
    if let Err(error) = private_ignores::seed(&gitignore_path, gitignore_content) {
        warnings.push(format!(
            "Could not protect private files in .gitignore: {error}"
        ));
    }

    // Optional "getting started" content only on truly fresh installs
    if is_fresh_install {
        create_sample_files(&kit_dir, &mut warnings);
    }

    info!(
        kit_path = %kit_dir.display(),
        is_fresh_install,
        warning_count = warnings.len(),
        "Kit setup complete"
    );

    SetupResult {
        is_fresh_install,
        kit_path: kit_dir,
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

/// Migrate legacy plugin-scoped `extensions/` directories to `scriptlets/`.
fn migrate_plugin_extensions_to_scriptlets(kit_dir: &Path, warnings: &mut Vec<String>) {
    let plugins_root = kit_dir.join("plugins");
    let Ok(entries) = fs::read_dir(&plugins_root) else {
        return;
    };

    for entry in entries.flatten() {
        let plugin_root = entry.path();
        if !plugin_root.is_dir() {
            continue;
        }

        let legacy_dir = plugin_root.join("extensions");
        if !legacy_dir.exists() || !legacy_dir.is_dir() {
            continue;
        }

        let scriptlets_dir = plugin_root.join("scriptlets");
        if !scriptlets_dir.exists() {
            match fs::rename(&legacy_dir, &scriptlets_dir) {
                Ok(()) => {
                    info!(
                        plugin_root = %plugin_root.display(),
                        "Migrated legacy extensions directory to scriptlets"
                    );
                    continue;
                }
                Err(error) => {
                    warnings.push(format!(
                        "Failed to rename legacy extensions directory {} to {}: {}",
                        legacy_dir.display(),
                        scriptlets_dir.display(),
                        error
                    ));
                    continue;
                }
            }
        }

        let Ok(legacy_entries) = fs::read_dir(&legacy_dir) else {
            warnings.push(format!(
                "Failed to read legacy extensions directory {} during migration",
                legacy_dir.display()
            ));
            continue;
        };

        for legacy_entry in legacy_entries.flatten() {
            let legacy_path = legacy_entry.path();
            let Some(file_name) = legacy_path.file_name() else {
                continue;
            };
            let target_path = scriptlets_dir.join(file_name);

            if target_path.exists() {
                warnings.push(format!(
                    "Skipped legacy scriptlet {} because {} already exists",
                    legacy_path.display(),
                    target_path.display()
                ));
                continue;
            }

            if let Err(error) = fs::rename(&legacy_path, &target_path) {
                warnings.push(format!(
                    "Failed to migrate legacy scriptlet {} to {}: {}",
                    legacy_path.display(),
                    target_path.display(),
                    error
                ));
            }
        }

        if let Err(error) = fs::remove_dir(&legacy_dir) {
            debug!(
                error = %error,
                path = %legacy_dir.display(),
                "Legacy extensions directory not removed after migration"
            );
        }
    }
}
/// Remove only files this app previously managed in the examples plugin.
///
/// The examples plugin may contain user experiments, so do not delete whole
/// artifact directories just because the managed starter pack changed.
fn prune_managed_examples_plugin(root: &Path, warnings: &mut Vec<String>) {
    const STALE_MANAGED_EXAMPLE_FILES: &[&str] = &[
        "agents/plan-feature.i.agy.md",
        "agents/review-pr.claude.md",
        "scripts/choose-from-list.ts",
        "scripts/clipboard-transform.ts",
        "scripts/generic-oauth-device-flow.ts",
        "scripts/github-device-login.ts",
        "scripts/google-calendar-device-login.ts",
        "scripts/hello-world.ts",
        "scripts/lib/oauth-device-flow.ts",
        "scripts/microsoft-graph-device-login.ts",
        "scripts/path-picker.ts",
        "scripts/power-syntax-capture-expense-ledger.ts",
        "scripts/power-syntax-capture-github-local.ts",
        "scripts/power-syntax-capture-snippet-library.ts",
        "scripts/power-syntax-command-env-dump.ts",
        "scripts/power-syntax-duplicate-command.ts",
        "scripts/power-syntax-payload-lab.ts",
        "scripts/power-syntax-refine-fixture.ts",
        "scripts/todoist-demo.ts",
        "scriptlets/agent_chat-chat/main.md",
        "scriptlets/advanced.md",
        "scriptlets/custom-actions/main.actions.md",
        "scriptlets/custom-actions/main.md",
        "scriptlets/howto.md",
        "scriptlets/main.md",
        "scriptlets/notes/main.md",
        "scriptlets/power-syntax.md",
        "scriptlets/starter.md",
        "skills/explain-code/SKILL.md",
        "skills/plan-feature/SKILL.md",
        "skills/review-pr/SKILL.md",
    ];

    for rel in STALE_MANAGED_EXAMPLE_FILES {
        let path = root.join(rel);
        if !path.is_file() {
            continue;
        }
        if let Err(error) = fs::remove_file(&path) {
            warnings.push(format!(
                "Failed to prune stale managed example {}: {}",
                path.display(),
                error
            ));
        }
    }

    for rel in [
        "scripts/lib",
        "scriptlets/agent_chat-chat",
        "scriptlets/custom-actions",
        "scriptlets/notes",
        "skills/explain-code",
        "skills/plan-feature",
        "skills/review-pr",
        "agents",
        "scriptlets",
        "skills",
    ] {
        let path = root.join(rel);
        if path.is_dir() {
            let _ = fs::remove_dir(&path);
        }
    }

    ensure_dir(&root.join("scripts"), warnings);
}

/// Ensure a plugin root exists with its standard subdirectories and a `plugin.json` manifest.
fn ensure_plugin_root(
    kit_dir: &Path,
    plugin_id: &str,
    title: &str,
    description: &str,
    warnings: &mut Vec<String>,
) {
    let root = kit_dir.join("plugins").join(plugin_id);
    ensure_dir(&root.join("scripts"), warnings);
    ensure_dir(&root.join("scriptlets"), warnings);
    ensure_dir(&root.join("agents"), warnings);
    ensure_dir(&root.join("skills"), warnings);
    ensure_dir(&root.join("profiles"), warnings);

    let manifest = format!(
        "{{\n  \"id\": \"{plugin_id}\",\n  \"title\": \"{title}\",\n  \"description\": \"{description}\"\n}}"
    );
    write_string_if_missing(
        &root.join("plugin.json"),
        &manifest,
        warnings,
        &format!("plugins/{plugin_id}/plugin.json"),
    );

    info!(plugin_id = %plugin_id, "plugin_root_ensured");
}

fn write_string_if_missing(path: &Path, contents: &str, warnings: &mut Vec<String>, label: &str) {
    if fs::symlink_metadata(path).is_ok() {
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
    // A first run can race another launch or a settings edit. Publish the
    // complete template only if the user file *still* does not exist.
    use std::io::Write;
    let publish = || -> std::io::Result<()> {
        let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))?;
        temp.write_all(contents.as_bytes())?;
        temp.persist_noclobber(path).map_err(|error| error.error)?;
        Ok(())
    };
    match publish() {
        Ok(()) => info!(path = %path.display(), "Created {}", label),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => warnings.push(format!(
            "Failed to write {} ({}): {}",
            label,
            path.display(),
            e
        )),
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
    write_setup_asset(path, contents, warnings, label, false);
}

fn write_executable_string_if_changed(
    path: &Path,
    contents: &str,
    warnings: &mut Vec<String>,
    label: &str,
) {
    write_setup_asset(path, contents, warnings, label, true);
}

fn write_setup_asset(
    path: &Path,
    contents: &str,
    warnings: &mut Vec<String>,
    label: &str,
    executable: bool,
) {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(not(unix))]
    let _ = executable;

    if fs::symlink_metadata(path).is_ok_and(|metadata| {
        #[cfg(unix)]
        if executable && metadata.permissions().mode() & 0o755 != 0o755 {
            return false;
        }
        metadata.is_file() && metadata.len() == contents.len() as u64
    }) && fs::read(path).is_ok_and(|existing| existing == contents.as_bytes())
    {
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

    // Atomic write: write to temp file then rename
    // This prevents readers from seeing partial writes during concurrent access
    use std::io::Write;
    let mut temp = match tempfile::NamedTempFile::new_in(path.parent().unwrap_or(Path::new("."))) {
        Ok(temp) => temp,
        Err(e) => {
            warnings.push(format!(
                "Failed to create private temp file for {label}: {e}"
            ));
            return;
        }
    };
    if let Err(e) = temp.write_all(contents.as_bytes()) {
        warnings.push(format!(
            "Failed to write temp file for {} ({}): {}",
            label,
            temp.path().display(),
            e
        ));
        return;
    }

    // Set the mode on the file we wrote. A failed rename can leave a private
    // directory or link at the destination; a successful rename can be followed
    // by another writer. Neither gives us permission to chmod the name there.
    #[cfg(unix)]
    if executable {
        if let Err(error) = temp
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o755))
        {
            warnings.push(format!(
                "Failed to make the new {label} executable: {error}"
            ));
            return;
        }
    }

    if let Err(e) = temp.persist(path) {
        warnings.push(format!(
            "Failed to publish {} ({}): {}",
            label,
            path.display(),
            e
        ));
        // NamedTempFile removes the unpublished file on drop.
    } else {
        debug!(path = %path.display(), "Updated {}", label);
    }
}

/// Ensure tsconfig.json has proper TypeScript/Bun settings (merge-safe)
/// The tsconfig lives at ~/.scriptkit/plugins/tsconfig.json, SDK at ~/.scriptkit/sdk/
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

    // Path is relative from the workspace root to sdk/.
    let expected_sdk_path = json!(["./sdk/kit-sdk.ts"]);

    let mut config: Value = if tsconfig_path.exists() {
        match fs::read_to_string(tsconfig_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    let err_msg = format!(
                        "tsconfig.json exists at {} but is not valid JSON (parsing failed: {}). Preserving original file to prevent data loss.",
                        tsconfig_path.display(),
                        e
                    );
                    warnings.push(err_msg.clone());
                    warn!("{}", err_msg);
                    return;
                }
            },
            Err(e) => {
                let err_msg = format!(
                    "Failed to read existing tsconfig.json at {}: {}. Preserving original file.",
                    tsconfig_path.display(),
                    e
                );
                warnings.push(err_msg.clone());
                warn!("{}", err_msg);
                return;
            }
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
// --- merged from part_005.rs ---
fn create_sample_files(kit_dir: &Path, warnings: &mut Vec<String>) {
    // Create sample files in the main plugin.
    let main_scripts_dir = kit_dir.join("plugins").join("main").join("scripts");
    let main_scriptlets_dir = kit_dir.join("plugins").join("main").join("scriptlets");
    let main_agents_dir = kit_dir.join("plugins").join("main").join("agents");

    let canonical_menu_syntax_scripts = [
        (
            "capture-todo-inbox.ts",
            EMBEDDED_CANONICAL_CAPTURE_TODO_INBOX,
        ),
        (
            "create-calendar-event.ts",
            EMBEDDED_CANONICAL_CREATE_CALENDAR_EVENT,
        ),
        (
            "create-mac-calendar-event.ts",
            EMBEDDED_CANONICAL_CREATE_MAC_CALENDAR_EVENT,
        ),
        (
            "add-google-calendar-event.ts",
            EMBEDDED_CANONICAL_ADD_GOOGLE_CALENDAR_EVENT,
        ),
        ("create-reminder.ts", EMBEDDED_CANONICAL_CREATE_REMINDER),
        ("snooze-task.ts", EMBEDDED_CANONICAL_SNOOZE_TASK),
        ("defer-task.ts", EMBEDDED_CANONICAL_DEFER_TASK),
        ("append-daily-note.ts", EMBEDDED_CANONICAL_APPEND_DAILY_NOTE),
        ("draft-social-post.ts", EMBEDDED_CANONICAL_DRAFT_SOCIAL_POST),
        ("save-tagged-link.ts", EMBEDDED_CANONICAL_SAVE_TAGGED_LINK),
    ];
    for (filename, contents) in canonical_menu_syntax_scripts {
        write_string_if_missing(
            &main_scripts_dir.join(filename),
            contents,
            warnings,
            &format!("plugins/main/scripts/{filename}"),
        );
    }

    // Create hello-world.ts script
    let hello_script_path = main_scripts_dir.join("hello-world.ts");
    if !hello_script_path.exists() {
        let hello_script = r#"/*
# Hello World

A simple greeting script demonstrating Script Kit basics.

## Features shown:
- `arg()` - Prompt for user input with choices
- `div()` - Display HTML content with Tailwind CSS
*/

export const metadata = {
  name: "Hello World",
  description: "A simple greeting script",
  // shortcut: "cmd shift h",  // Uncomment to add a global hotkey
  sdkCapabilities: ["arg", "div"],
  executionTopology: "typescript-script",
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
        write_string_if_missing(&hello_script_path, hello_script, warnings, "sample script");
    }

    // Create hello-world.md scriptlet bundle
    let hello_scriptlet_path = main_scriptlets_dir.join("hello-world.md");
    if !hello_scriptlet_path.exists() {
        let hello_scriptlet = r#"# Hello World Scriptlets

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
        write_string_if_missing(&hello_scriptlet_path, hello_scriptlet, warnings, "sample scriptlet bundle");
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
        write_string_if_missing(&hello_agent_path, hello_agent, warnings, "sample agent");
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
│   │   ├── scriptlets/     # Markdown scriptlet bundles (.md)
│   │   └── agents/         # AI agent definitions (.md)
│   ├── package.json        # Node.js module config (enables top-level await)
│   └── tsconfig.json       # TypeScript path mappings
├── sdk/                    # Runtime SDK (managed by app)
├── db/                     # Databases (clipboard history, etc.)
├── logs/                   # Application logs
├── cache/                  # Cached data (app icons, etc.)
└── README.md               # This file
```

## File Watching

Script Kit watches these files and reloads automatically:

| File/Directory | What happens on change |
|----------------|------------------------|
| `config.ts` | Reloads configuration (hotkeys, settings) |
| `theme.json` | Applies new theme colors immediately |
| `plugins/main/scripts/*.ts` | Updates script list and metadata |
| `plugins/main/scriptlets/*.md` | Updates scriptlet list |

## Scripts

Scripts are TypeScript files in `plugins/main/scripts/`. They have full access to the Script Kit SDK.

### Example Script

```typescript
// plugins/main/scripts/my-script.ts

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

1. **Create a new script**: Add a `.ts` file to `plugins/main/scripts/`
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
        write_string_if_missing(&readme_path, readme, warnings, "README.md");
    }
}
// --- merged from part_006.rs ---
#[cfg(test)]
include!("mod_tests.rs");

#[cfg(test)]
mod window_management_extension_tests {
    /// Every shipped window command parses, and each `tileWindow` position it
    /// uses is one the app understands.
    #[test]
    fn every_window_command_parses_and_uses_a_known_position() {
        let scriptlets = crate::scriptlets::parse_markdown_as_scriptlets(
            super::EMBEDDED_WINDOW_MANAGEMENT_EXTENSION,
            None,
        );
        assert!(scriptlets.len() >= 48, "{}", scriptlets.len());
        for name in [
            "Tile {{app}} Left Fourth",
            "Make {{app}} Larger",
            "Restore Window",
        ] {
            assert!(scriptlets.iter().any(|s| s.name == name), "{name}");
        }
        let mut positions = 0;
        for scriptlet in &scriptlets {
            for part in scriptlet
                .scriptlet_content
                .split("tileWindow(win.windowId, '")
                .skip(1)
            {
                let position = part.split('\'').next().unwrap_or_default();
                let parsed: Result<crate::protocol::TilePosition, _> =
                    serde_json::from_value(serde_json::json!(position));
                assert!(
                    parsed.is_ok(),
                    "{} uses unknown position {position}",
                    scriptlet.name
                );
                positions += 1;
            }
        }
        assert!(positions >= 40, "{positions}");
    }
}

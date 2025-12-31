//! Kenv environment setup and initialization.
//!
//! Ensures ~/.kenv exists with required directories and starter files.
//! Idempotent: user-owned files are never overwritten; app-owned files may be refreshed.

use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, info, instrument, warn};

/// Embedded config template (included at compile time)
const EMBEDDED_CONFIG_TEMPLATE: &str = include_str!("../scripts/config-template.ts");

/// Embedded SDK content (included at compile time)
const EMBEDDED_SDK: &str = include_str!("../scripts/kit-sdk.ts");

/// Optional theme example (included at compile time)
const EMBEDDED_THEME_EXAMPLE: &str = include_str!("../theme.example.json");

/// Result of setup process
#[derive(Debug)]
pub struct SetupResult {
    /// Whether ~/.kenv didn't exist before this run
    pub is_fresh_install: bool,
    /// Path to ~/.kenv (or fallback if home dir couldn't be resolved)
    pub kenv_path: PathBuf,
    /// Whether bun looks discoverable on this machine
    pub bun_available: bool,
    /// Any warnings encountered during setup
    pub warnings: Vec<String>,
}

/// Ensure the ~/.kenv environment is properly set up.
///
/// This function is idempotent - it will create missing directories and files
/// without overwriting existing user configurations.
///
/// # Directory Structure Created
/// ```text
/// ~/.kenv/
/// ├── scripts/           # User scripts (.ts, .js files)
/// ├── scriptlets/        # Markdown scriptlet files
/// ├── sdk/               # Runtime SDK (kit-sdk.ts)
/// ├── logs/              # Application logs
/// ├── cache/
/// │   └── app-icons/     # Cached application icons
/// ├── config.ts          # User configuration (created from template if missing)
/// ├── theme.json         # Theme configuration (created from example if missing)
/// ├── tsconfig.json      # TypeScript path mappings
/// └── .gitignore         # Ignore transient files
/// ```
///
/// # Returns
/// `SetupResult` with information about the setup process.
#[instrument(level = "info", name = "ensure_kenv_setup")]
pub fn ensure_kenv_setup() -> SetupResult {
    let mut warnings = Vec::new();

    let kenv_dir = match dirs::home_dir() {
        Some(home) => home.join(".kenv"),
        None => {
            let fallback = std::env::temp_dir().join("script-kit-kenv");
            warnings
                .push("Could not determine home directory; using temp dir fallback".to_string());
            fallback
        }
    };

    let is_fresh_install = !kenv_dir.exists();

    // Ensure ~/.kenv exists first
    if let Err(e) = fs::create_dir_all(&kenv_dir) {
        warnings.push(format!(
            "Failed to create kenv root {}: {}",
            kenv_dir.display(),
            e
        ));
        // If we can't create the root, there's not much else we can safely do.
        return SetupResult {
            is_fresh_install,
            kenv_path: kenv_dir,
            bun_available: false,
            warnings,
        };
    }

    // Required directory structure
    let required_dirs = [
        kenv_dir.join("scripts"),
        kenv_dir.join("scriptlets"),
        kenv_dir.join("sdk"),
        kenv_dir.join("logs"),
        kenv_dir.join("cache").join("app-icons"),
    ];

    for dir in required_dirs {
        ensure_dir(&dir, &mut warnings);
    }

    // App-managed: SDK (refresh if changed)
    let sdk_path = kenv_dir.join("sdk").join("kit-sdk.ts");
    write_string_if_changed(&sdk_path, EMBEDDED_SDK, &mut warnings, "sdk/kit-sdk.ts");

    // User-owned: config.ts (only create if missing)
    let config_path = kenv_dir.join("config.ts");
    write_string_if_missing(
        &config_path,
        EMBEDDED_CONFIG_TEMPLATE,
        &mut warnings,
        "config.ts",
    );

    // User-owned (optional): theme.json (only create if missing)
    let theme_path = kenv_dir.join("theme.json");
    write_string_if_missing(
        &theme_path,
        EMBEDDED_THEME_EXAMPLE,
        &mut warnings,
        "theme.json",
    );

    // App-managed: tsconfig.json path mappings (merge-safe)
    ensure_tsconfig_paths(&kenv_dir.join("tsconfig.json"), &mut warnings);

    // App-managed: .gitignore (refresh if changed)
    let gitignore_path = kenv_dir.join(".gitignore");
    let gitignore_content = r#"# Script Kit managed files (may be regenerated on start)
sdk/
logs/
cache/
clipboard-history.db
frecency.json
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
        create_sample_files(&kenv_dir, &mut warnings);
    }

    info!(
        kenv_path = %kenv_dir.display(),
        is_fresh_install,
        bun_available,
        warning_count = warnings.len(),
        "Kenv setup complete"
    );

    SetupResult {
        is_fresh_install,
        kenv_path: kenv_dir,
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

    if let Err(e) = fs::write(path, contents) {
        warnings.push(format!(
            "Failed to write {} ({}): {}",
            label,
            path.display(),
            e
        ));
    } else {
        debug!(path = %path.display(), "Updated {}", label);
    }
}

/// Ensure tsconfig.json has the @johnlindquist/kit path mapping (merge-safe)
fn ensure_tsconfig_paths(tsconfig_path: &Path, warnings: &mut Vec<String>) {
    use serde_json::{json, Value};

    let kit_path = json!(["./sdk/kit-sdk.ts"]);

    let mut config: Value = if tsconfig_path.exists() {
        match fs::read_to_string(tsconfig_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| json!({})),
            Err(_) => json!({}),
        }
    } else {
        json!({})
    };

    if config.get("compilerOptions").is_none() {
        config["compilerOptions"] = json!({});
    }
    if config["compilerOptions"].get("paths").is_none() {
        config["compilerOptions"]["paths"] = json!({});
    }

    let current_kit_path = config["compilerOptions"]["paths"].get("@johnlindquist/kit");
    if current_kit_path == Some(&kit_path) {
        return;
    }

    config["compilerOptions"]["paths"]["@johnlindquist/kit"] = kit_path;

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
                info!("Updated tsconfig.json with @johnlindquist/kit path mapping");
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

fn create_sample_files(kenv_dir: &Path, warnings: &mut Vec<String>) {
    let scripts_dir = kenv_dir.join("scripts");
    let scriptlets_dir = kenv_dir.join("scriptlets");

    let hello_script_path = scripts_dir.join("hello-world.ts");
    if !hello_script_path.exists() {
        let hello_script = r#"// Name: Hello World
// Description: A simple greeting script

const name = await arg("What's your name?");
await div(`<h1 class="text-2xl p-4">Hello, ${name}! Welcome to Script Kit.</h1>`);
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

    let getting_started_path = scriptlets_dir.join("getting-started.md");
    if !getting_started_path.exists() {
        let sample_scriptlet = r#"# Getting Started

## Current Date
<!-- shortcut: cmd d -->

```bash
date +"%Y-%m-%d"
```

## Open Downloads
<!-- shortcut: cmd shift d -->

```bash
open ~/Downloads
```
"#;
        if let Err(e) = fs::write(&getting_started_path, sample_scriptlet) {
            warnings.push(format!(
                "Failed to create sample scriptlet {}: {}",
                getting_started_path.display(),
                e
            ));
        } else {
            info!(path = %getting_started_path.display(), "Created sample scriptlet");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

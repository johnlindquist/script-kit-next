# Script Kit GPUI - Expert Review Request

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**Architecture:**
- **GPUI** for UI rendering (custom immediate-mode reactive UI framework from Zed)
- **Bun** as the TypeScript runtime for user scripts
- **Stdin/stdout JSON protocol** for bidirectional script ↔ app communication
- **SQLite** for persistence (clipboard history, notes, chat)
- **macOS-first** with floating panel window behavior

**Key Constraints:**
- Must maintain backwards compatibility with existing Script Kit scripts
- Performance-critical: launcher must appear instantly, list scrolling at 60fps
- Multi-window: main launcher + Notes window + AI chat window (all independent)
- Theme hot-reload across all windows

---

## Bundle: Environment Setup (src/setup.rs)

This bundle documents the `~/.scriptkit` environment initialization system.

---

## Directory Structure Created

```
~/.scriptkit/                      # Root (can be overridden via SK_PATH)
├── kit/                           # All kits container (version control friendly)
│   ├── main/                      # Default user kit
│   │   ├── scripts/               # User scripts (.ts, .js files)
│   │   ├── extensions/            # Markdown extension files
│   │   └── agents/                # AI agent definitions (.md)
│   ├── package.json               # Node.js module config (type: module)
│   ├── tsconfig.json              # TypeScript path mappings
│   ├── config.ts                  # User configuration
│   ├── theme.json                 # Theme configuration
│   ├── AGENTS.md                  # AI agent guide
│   └── CLAUDE.md                  # Claude-specific instructions
├── sdk/                           # Runtime SDK
│   └── kit-sdk.ts                 # Embedded SDK (managed by app)
├── db/                            # Databases
├── logs/                          # Application logs
├── cache/
│   └── app-icons/                 # Cached application icons
├── GUIDE.md                       # User guide
└── .gitignore                     # Ignore transient files
```

---

## File Categories

### User-Owned Files (never overwritten)

| File | Purpose |
|------|---------|
| `kit/config.ts` | User configuration |
| `kit/theme.json` | Theme customization |

### App-Managed Files (refreshed if changed)

| File | Purpose |
|------|---------|
| `sdk/kit-sdk.ts` | SDK runtime |
| `kit/tsconfig.json` | TypeScript path mappings |
| `.gitignore` | Ignore patterns |

### Fresh-Install Only

| File | Purpose |
|------|---------|
| `kit/main/scripts/hello-world.ts` | Sample script |
| `kit/main/extensions/hello-world.md` | Sample extension |
| `kit/main/agents/hello-world.claude.md` | Sample agent |
| `README.md` | Documentation |
| `GUIDE.md` | User guide |

---

## Setup Process

```rust
pub fn ensure_kit_setup() -> SetupResult {
    let kit_dir = get_kit_path();  // ~/.scriptkit or $SK_PATH
    
    // Check if fresh install
    let is_fresh_install = !kit_dir.exists();
    
    // Create directory structure
    let required_dirs = [
        kit_dir.join("kit").join("main").join("scripts"),
        kit_dir.join("kit").join("main").join("extensions"),
        kit_dir.join("kit").join("main").join("agents"),
        kit_dir.join("sdk"),
        kit_dir.join("db"),
        kit_dir.join("logs"),
        kit_dir.join("cache").join("app-icons"),
    ];
    for dir in required_dirs {
        ensure_dir(&dir, &mut warnings);
    }
    
    // SDK (app-managed: refresh if changed)
    write_string_if_changed(
        &kit_dir.join("sdk").join("kit-sdk.ts"),
        EMBEDDED_SDK,  // include_str! at compile time
        ...
    );
    
    // Config (user-owned: only create if missing)
    write_string_if_missing(
        &kit_dir.join("kit").join("config.ts"),
        EMBEDDED_CONFIG_TEMPLATE,
        ...
    );
    
    // Theme (user-owned: only create if missing)
    write_string_if_missing(
        &kit_dir.join("kit").join("theme.json"),
        EMBEDDED_THEME_EXAMPLE,
        ...
    );
    
    // tsconfig.json (merge-safe)
    ensure_tsconfig_paths(&kit_dir.join("kit").join("tsconfig.json"), ...);
    
    // package.json (user-owned: only create if missing)
    write_string_if_missing(
        &kit_dir.join("kit").join("package.json"),
        EMBEDDED_PACKAGE_JSON,  // { "type": "module" }
        ...
    );
    
    // Check bun availability
    let bun_available = bun_is_discoverable();
    
    // Sample files on fresh install
    if is_fresh_install {
        create_sample_files(&kit_dir, &mut warnings);
    }
    
    SetupResult { is_fresh_install, kit_path: kit_dir, bun_available, warnings }
}
```

---

## Environment Variable Override

```rust
pub const SK_PATH_ENV: &str = "SK_PATH";

pub fn get_kit_path() -> PathBuf {
    // Check for SK_PATH override first
    if let Ok(sk_path) = std::env::var(SK_PATH_ENV) {
        return PathBuf::from(shellexpand::tilde(&sk_path).as_ref());
    }
    
    // Default: ~/.scriptkit
    match dirs::home_dir() {
        Some(home) => home.join(".scriptkit"),
        None => std::env::temp_dir().join("script-kit"),
    }
}
```

**Usage:**
```bash
export SK_PATH=~/my-scripts
./script-kit-gpui
```

---

## Legacy Migration

Migrates from old `~/.kenv` to new `~/.scriptkit`:

```rust
pub fn migrate_from_kenv() -> bool {
    let old_kenv = home.join(".kenv");
    let new_scriptkit = home.join(".scriptkit");
    
    // Only migrate if old exists and new doesn't
    if !old_kenv.exists() || new_scriptkit.exists() {
        return false;
    }
    
    // Move scripts
    move_contents(&old_kenv.join("scripts"), &new_scriptkit.join("kit/main/scripts"));
    
    // Move scriptlets → extensions
    move_contents(&old_kenv.join("scriptlets"), &new_scriptkit.join("kit/main/extensions"));
    
    // Move config files
    for file in ["config.ts", "theme.json", "tsconfig.json"] {
        move_file(&old_kenv.join(file), &new_scriptkit.join(file));
    }
    
    // Create symlink for backwards compatibility
    #[cfg(unix)]
    std::os::unix::fs::symlink(&new_scriptkit, &old_kenv)?;
    
    true
}
```

---

## Atomic File Writes

App-managed files use atomic writes to prevent corruption:

```rust
fn write_string_if_changed(path: &Path, contents: &str, ...) {
    // Check if content changed
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == contents {
            return;  // No change needed
        }
    }
    
    // Atomic write: temp file + rename
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, contents)?;
    fs::rename(&temp_path, path)?;  // Atomic on most filesystems
}
```

---

## TypeScript Configuration

The tsconfig.json is merge-safe - existing user settings are preserved:

```rust
fn ensure_tsconfig_paths(tsconfig_path: &Path, warnings: &mut Vec<String>) {
    let mut config: Value = if tsconfig_path.exists() {
        serde_json::from_str(&fs::read_to_string(tsconfig_path)?)?
    } else {
        json!({})
    };
    
    // Ensure compilerOptions exists
    if config.get("compilerOptions").is_none() {
        config["compilerOptions"] = json!({});
    }
    
    // Set defaults if missing (doesn't overwrite user values)
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
            compiler_options.insert(key, value);
        }
    }
    
    // Always ensure @scriptkit/sdk path is correct
    paths.insert("@scriptkit/sdk", json!(["../sdk/kit-sdk.ts"]));
}
```

---

## Bun Discovery

Checks for bun without spawning a process:

```rust
fn bun_is_discoverable() -> bool {
    let mut candidates = Vec::new();
    
    // Common install locations
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".bun/bin/bun"));
    }
    candidates.push(PathBuf::from("/opt/homebrew/bin/bun"));
    candidates.push(PathBuf::from("/usr/local/bin/bun"));
    candidates.push(PathBuf::from("/usr/bin/bun"));
    
    // PATH scan
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            candidates.push(dir.join("bun"));
        }
    }
    
    candidates.into_iter().any(|p| p.exists())
}
```

---

## Sample Files

Created only on fresh install:

### hello-world.ts

```typescript
export const metadata = {
  name: "Hello World",
  description: "A simple greeting script",
};

const name = await arg("What's your name?", [
  "World",
  "Script Kit",
  "Friend",
]);

await div(`
  <div class="flex flex-col items-center justify-center h-full p-8">
    <h1 class="text-4xl font-bold text-yellow-400 mb-4">
      Hello, ${name}! 👋
    </h1>
  </div>
`);
```

### hello-world.md (extension)

```markdown
# Hello World Extensions

## Say Hello
<!-- 
name: Say Hello
description: Display a greeting notification
shortcut: ctrl h
-->

\`\`\`bash
echo "Hello from Script Kit! 🎉"
\`\`\`
```

### hello-world.claude.md (agent)

```markdown
---
_sk_name: Hello World Assistant
_sk_description: A friendly assistant
_sk_interactive: true
---

You are a friendly, helpful assistant.
```

---

## SetupResult

```rust
pub struct SetupResult {
    /// Whether ~/.scriptkit didn't exist before this run
    pub is_fresh_install: bool,
    
    /// Path to ~/.scriptkit
    pub kit_path: PathBuf,
    
    /// Whether bun is discoverable
    pub bun_available: bool,
    
    /// Any warnings encountered
    pub warnings: Vec<String>,
}
```

---

## Error Handling

Setup collects warnings instead of failing:

```rust
fn ensure_dir(path: &Path, warnings: &mut Vec<String>) {
    if path.exists() { return; }
    
    if let Err(e) = fs::create_dir_all(path) {
        warnings.push(format!("Failed to create {}: {}", path.display(), e));
    }
}
```

This allows the app to continue with degraded functionality.

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.

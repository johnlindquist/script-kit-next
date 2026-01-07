# Configuration System - Expert Bundle

## Overview

Script Kit uses a TypeScript configuration file (`~/.scriptkit/config.ts`) with type-safe defaults and hot-reload support.

## Config File Location

```
~/.scriptkit/config.ts
```

## Config Structure (src/config/types.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Main launcher hotkey
    pub hotkey: HotkeyConfig,
    
    /// UI padding
    #[serde(default)]
    pub padding: PaddingConfig,
    
    /// Editor font size (default: 14)
    #[serde(default = "default_font_size")]
    pub editor_font_size: u32,
    
    /// Terminal font size (default: 14)
    #[serde(default = "default_font_size")]
    pub terminal_font_size: u32,
    
    /// UI scale factor (default: 1.0)
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    
    /// Built-in features
    #[serde(default)]
    pub built_ins: BuiltInsConfig,
    
    /// Path to bun executable
    #[serde(default)]
    pub bun_path: Option<String>,
    
    /// Default editor command
    #[serde(default = "default_editor")]
    pub editor: String,
    
    /// Notes window hotkey
    #[serde(default)]
    pub notes_hotkey: Option<HotkeyConfig>,
    
    /// AI window hotkey
    #[serde(default)]
    pub ai_hotkey: Option<HotkeyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaddingConfig {
    #[serde(default = "default_padding_top")]
    pub top: u32,
    #[serde(default = "default_padding_left")]
    pub left: u32,
    #[serde(default = "default_padding_right")]
    pub right: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltInsConfig {
    #[serde(default = "default_true")]
    pub clipboard_history: bool,
    #[serde(default = "default_true")]
    pub app_launcher: bool,
    #[serde(default = "default_true")]
    pub file_search: bool,
}
```

## Default Values (src/config/defaults.rs)

```rust
fn default_font_size() -> u32 { 14 }
fn default_ui_scale() -> f32 { 1.0 }
fn default_editor() -> String { "code".to_string() }
fn default_padding_top() -> u32 { 8 }
fn default_padding_left() -> u32 { 12 }
fn default_padding_right() -> u32 { 12 }
fn default_true() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            },
            padding: PaddingConfig::default(),
            editor_font_size: default_font_size(),
            terminal_font_size: default_font_size(),
            ui_scale: default_ui_scale(),
            built_ins: BuiltInsConfig::default(),
            bun_path: None,
            editor: default_editor(),
            notes_hotkey: None,
            ai_hotkey: None,
        }
    }
}

impl Default for BuiltInsConfig {
    fn default() -> Self {
        Self {
            clipboard_history: true,
            app_launcher: true,
            file_search: true,
        }
    }
}
```

## Config Helpers

```rust
impl Config {
    pub fn get_editor_font_size(&self) -> u32 {
        self.editor_font_size.max(8).min(72)
    }
    
    pub fn get_terminal_font_size(&self) -> u32 {
        self.terminal_font_size.max(8).min(72)
    }
    
    pub fn get_padding(&self) -> (u32, u32, u32) {
        (self.padding.top, self.padding.left, self.padding.right)
    }
    
    pub fn get_ui_scale(&self) -> f32 {
        self.ui_scale.max(0.5).min(3.0)
    }
    
    pub fn get_builtins(&self) -> &BuiltInsConfig {
        &self.built_ins
    }
    
    pub fn get_editor(&self) -> &str {
        &self.editor
    }
    
    pub fn get_notes_hotkey(&self) -> HotkeyConfig {
        self.notes_hotkey.clone().unwrap_or_else(|| HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyN".to_string(),
        })
    }
    
    pub fn get_ai_hotkey(&self) -> HotkeyConfig {
        self.ai_hotkey.clone().unwrap_or_else(|| HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "Space".to_string(),
        })
    }
}
```

## Config Loading (src/config/loader.rs)

```rust
use std::process::Command;

pub fn load_config() -> Config {
    let config_path = dirs::home_dir()
        .unwrap()
        .join(".scriptkit/config.ts");
    
    if !config_path.exists() {
        logging::log("CONFIG", "No config file, using defaults");
        return Config::default();
    }
    
    // Execute config.ts with bun to get JSON
    match execute_config(&config_path) {
        Ok(json) => {
            match serde_json::from_str(&json) {
                Ok(config) => {
                    logging::log("CONFIG", "Config loaded successfully");
                    config
                }
                Err(e) => {
                    logging::log_warn("CONFIG", &format!("Parse error: {}", e));
                    Config::default()
                }
            }
        }
        Err(e) => {
            logging::log_warn("CONFIG", &format!("Execution error: {}", e));
            Config::default()
        }
    }
}

fn execute_config(path: &Path) -> Result<String> {
    // Create a wrapper script that exports the config as JSON
    let wrapper = format!(r#"
        import config from "{}";
        console.log(JSON.stringify(config));
    "#, path.display());
    
    let output = Command::new("bun")
        .arg("--eval")
        .arg(&wrapper)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Config execution failed: {}", stderr));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

## TypeScript Config Example

```typescript
// ~/.scriptkit/config.ts
import type { Config } from "@scriptkit/sdk";

export default {
    // Main launcher hotkey
    hotkey: {
        modifiers: ["meta"],
        key: "Semicolon"
    },
    
    // UI customization
    padding: {
        top: 8,
        left: 12,
        right: 12
    },
    
    // Font sizes
    editorFontSize: 16,
    terminalFontSize: 14,
    
    // Scale factor for HiDPI
    uiScale: 1.0,
    
    // Built-in features
    builtIns: {
        clipboardHistory: true,
        appLauncher: true,
        fileSearch: true
    },
    
    // Custom bun path (optional)
    bun_path: "/opt/homebrew/bin/bun",
    
    // Default editor
    editor: "code",
    
    // Window hotkeys
    notesHotkey: {
        modifiers: ["meta", "shift"],
        key: "KeyN"
    },
    aiHotkey: {
        modifiers: ["meta", "shift"],
        key: "Space"
    }
} satisfies Config;
```

## Hot-Reload Support

### File Watcher Integration

```rust
pub fn setup_config_watcher(app: Weak<App>, cx: &mut Context<App>) {
    let config_path = dirs::home_dir()
        .unwrap()
        .join(".scriptkit/config.ts");
    
    let (tx, rx) = async_channel::bounded(1);
    
    // Watch for changes
    std::thread::spawn(move || {
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    let _ = tx.try_send(());
                }
            }
        }).unwrap();
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive).unwrap();
        
        // Keep watcher alive
        loop {
            std::thread::sleep(Duration::from_secs(60));
        }
    });
    
    // Handle reload
    cx.spawn(|this, mut cx| async move {
        while rx.recv().await.is_ok() {
            // Debounce
            Timer::after(Duration::from_millis(100)).await;
            
            let _ = this.update(&mut cx, |app, cx| {
                app.reload_config(cx);
            });
        }
    }).detach();
}

impl App {
    fn reload_config(&mut self, cx: &mut Context<Self>) {
        let new_config = load_config();
        
        // Update hotkeys
        crate::hotkeys::update_hotkeys(&new_config);
        
        // Update UI settings
        self.config = Arc::new(new_config);
        
        logging::log("CONFIG", "Config hot-reloaded");
        cx.notify();
    }
}
```

## Config Validation

```rust
impl Config {
    pub fn validate(&self) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        
        // Check font sizes
        if self.editor_font_size < 8 || self.editor_font_size > 72 {
            warnings.push(ValidationWarning {
                field: "editorFontSize".to_string(),
                message: "Font size should be between 8 and 72".to_string(),
            });
        }
        
        // Check UI scale
        if self.ui_scale < 0.5 || self.ui_scale > 3.0 {
            warnings.push(ValidationWarning {
                field: "uiScale".to_string(),
                message: "UI scale should be between 0.5 and 3.0".to_string(),
            });
        }
        
        // Check hotkey format
        if !is_valid_key(&self.hotkey.key) {
            warnings.push(ValidationWarning {
                field: "hotkey.key".to_string(),
                message: format!("Unknown key code: {}", self.hotkey.key),
            });
        }
        
        warnings
    }
}

fn is_valid_key(key: &str) -> bool {
    matches!(key, 
        "Semicolon" | "Space" | "Enter" |
        "KeyA" | "KeyB" | "KeyC" | "KeyD" | "KeyE" | "KeyF" |
        "KeyG" | "KeyH" | "KeyI" | "KeyJ" | "KeyK" | "KeyL" |
        "KeyM" | "KeyN" | "KeyO" | "KeyP" | "KeyQ" | "KeyR" |
        "KeyS" | "KeyT" | "KeyU" | "KeyV" | "KeyW" | "KeyX" |
        "KeyY" | "KeyZ" |
        "Digit0" | "Digit1" | "Digit2" | "Digit3" | "Digit4" |
        "Digit5" | "Digit6" | "Digit7" | "Digit8" | "Digit9" |
        "F1" | "F2" | "F3" | "F4" | "F5" | "F6" |
        "F7" | "F8" | "F9" | "F10" | "F11" | "F12"
    )
}
```

## Using Config in Components

### Font Sizing

```rust
impl EditorPrompt {
    fn render(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let font_size = self.config.get_editor_font_size() as f32;
        let line_height = font_size * 1.43;
        
        div()
            .text_size(px(font_size))
            .line_height(px(line_height))
            // ...
    }
}
```

### Padding

```rust
impl App {
    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (top, left, right) = self.config.get_padding();
        
        div()
            .pt(px(top as f32))
            .pl(px(left as f32))
            .pr(px(right as f32))
            // ...
    }
}
```

### Built-in Feature Flags

```rust
impl App {
    fn get_available_scripts(&self) -> Vec<ScriptItem> {
        let mut items = Vec::new();
        
        // User scripts
        items.extend(self.user_scripts.iter().cloned());
        
        // Built-in features (if enabled)
        let builtins = self.config.get_builtins();
        
        if builtins.clipboard_history {
            items.push(ScriptItem::builtin("Clipboard History", "clipboard"));
        }
        
        if builtins.app_launcher {
            items.push(ScriptItem::builtin("App Launcher", "apps"));
        }
        
        if builtins.file_search {
            items.push(ScriptItem::builtin("File Search", "files"));
        }
        
        items
    }
}
```

## Best Practices

1. **Always use helpers** - `get_editor_font_size()` not `config.editor_font_size`
2. **Provide defaults** - Never panic on missing config
3. **Validate on load** - Warn about invalid values
4. **Hot-reload safely** - Update hotkeys transactionally
5. **Use Arc<Config>** - Share across views cheaply
6. **Debounce reloads** - Avoid rapid reloads during editing

## Key Code Reference

| Key Name | Code |
|----------|------|
| Semicolon | `Semicolon` |
| Space | `Space` |
| A-Z | `KeyA` - `KeyZ` |
| 0-9 | `Digit0` - `Digit9` |
| F1-F12 | `F1` - `F12` |

| Modifier | Name |
|----------|------|
| Cmd/Win | `meta` |
| Ctrl | `ctrl` |
| Alt/Option | `alt` |
| Shift | `shift` |

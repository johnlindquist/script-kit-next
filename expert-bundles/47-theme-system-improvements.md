# Script Kit GPUI - Expert Bundle 47: Theme System Improvements

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner.

**Theme Requirements:**
- Vibrancy/transparency support on macOS
- Hot-reload when theme.json changes
- Sync across multiple windows (main, Notes, AI)
- Support for light/dark mode
- User customization via theme.json

---

## Goal

Improve the **theme system** to:
1. Provide automatic light/dark mode switching
2. Support theme presets (beyond just colors)
3. Centralize vibrancy and opacity handling
4. Enable per-component theming
5. Add theme preview and editing tools

---

## Current State

### Theme Structure

```rust
// src/theme.rs
pub struct Theme {
    pub colors: ColorScheme,
    pub focus_aware: Option<FocusAwareColors>,
    pub opacity: Option<ThemeOpacity>,
    pub drop_shadow: Option<bool>,
    pub vibrancy: Option<bool>,
}

pub struct ColorScheme {
    pub background: BackgroundColors,
    pub text: TextColors,
    pub accent: AccentColors,
    pub ui: UiColors,
}

// 50+ color fields across nested structs
```

### Problems

1. **No Light Mode** - Only dark theme supported
2. **No Presets** - Can't switch between themes easily
3. **Vibrancy Scattered** - Each window handles opacity differently
4. **No Semantic Tokens** - Colors not named by purpose
5. **Hot Reload Partial** - Some colors cached, don't update
6. **No Validation** - Invalid theme.json silently fails

---

## Proposed Architecture

### 1. Semantic Color Tokens

```rust
/// Semantic color tokens that map to specific UI purposes
/// These are the "public API" for theming - components use these
pub struct SemanticColors {
    // Backgrounds
    pub bg_app: Hsla,           // Main app background
    pub bg_surface: Hsla,       // Cards, panels
    pub bg_elevated: Hsla,      // Floating panels, dialogs
    pub bg_input: Hsla,         // Input fields
    pub bg_selected: Hsla,      // Selected list items
    pub bg_hover: Hsla,         // Hovered items
    
    // Text
    pub text_primary: Hsla,     // Main content text
    pub text_secondary: Hsla,   // Descriptions, labels
    pub text_muted: Hsla,       // Placeholders, hints
    pub text_inverse: Hsla,     // Text on accent bg
    
    // Borders
    pub border_default: Hsla,   // Standard borders
    pub border_strong: Hsla,    // Emphasized borders
    pub border_focus: Hsla,     // Focused element rings
    
    // Accents
    pub accent_primary: Hsla,   // Primary brand color
    pub accent_secondary: Hsla, // Secondary actions
    pub accent_success: Hsla,
    pub accent_warning: Hsla,
    pub accent_error: Hsla,
    
    // Interactive
    pub interactive_hover: Hsla,
    pub interactive_active: Hsla,
    pub interactive_disabled: Hsla,
}

impl SemanticColors {
    /// Get colors adjusted for focus state
    pub fn for_focus_state(&self, is_focused: bool) -> &Self {
        // Return dimmed variant when unfocused
        if is_focused { self } else { &self.unfocused }
    }
}
```

### 2. Theme Presets

```rust
/// Pre-built theme configurations
pub enum ThemePreset {
    /// Default Script Kit dark theme
    ScriptKitDark,
    /// Light mode variant
    ScriptKitLight,
    /// High contrast for accessibility
    HighContrast,
    /// Nord-inspired palette
    Nord,
    /// Dracula-inspired palette
    Dracula,
    /// Custom user theme
    Custom(PathBuf),
}

impl ThemePreset {
    pub fn load(&self) -> Theme {
        match self {
            Self::ScriptKitDark => Theme::script_kit_dark(),
            Self::ScriptKitLight => Theme::script_kit_light(),
            Self::HighContrast => Theme::high_contrast(),
            Self::Nord => Theme::nord(),
            Self::Dracula => Theme::dracula(),
            Self::Custom(path) => Theme::load_from_file(path).unwrap_or_default(),
        }
    }
    
    /// Get preset for current system appearance
    pub fn for_system_appearance() -> Self {
        if platform::is_dark_mode() {
            Self::ScriptKitDark
        } else {
            Self::ScriptKitLight
        }
    }
}
```

### 3. Light/Dark Mode Support

```rust
/// Theme configuration with light/dark variants
pub struct AdaptiveTheme {
    /// Theme for light system appearance
    light: Theme,
    /// Theme for dark system appearance
    dark: Theme,
    /// Override system preference
    preference: ColorSchemePreference,
}

pub enum ColorSchemePreference {
    /// Follow system setting
    System,
    /// Always light
    Light,
    /// Always dark
    Dark,
}

impl AdaptiveTheme {
    /// Get current effective theme
    pub fn current(&self) -> &Theme {
        match self.preference {
            ColorSchemePreference::System => {
                if platform::is_dark_mode() { &self.dark } else { &self.light }
            }
            ColorSchemePreference::Light => &self.light,
            ColorSchemePreference::Dark => &self.dark,
        }
    }
    
    /// Called when system appearance changes
    pub fn on_system_appearance_change(&self, cx: &mut App) {
        if self.preference == ColorSchemePreference::System {
            // Notify all windows to re-render
            windows::notify_all_windows(cx);
        }
    }
}
```

### 4. Centralized Vibrancy

```rust
/// Vibrancy configuration for macOS
pub struct VibrancyConfig {
    /// Whether vibrancy is enabled
    pub enabled: bool,
    /// Material type (affects blur amount and tinting)
    pub material: VibrancyMaterial,
    /// Opacity overrides for different regions
    pub opacity: VibrancyOpacity,
}

#[derive(Clone, Copy)]
pub enum VibrancyMaterial {
    /// Sidebar material (more opaque)
    Sidebar,
    /// Content background (standard)
    ContentBackground,
    /// Under window (most transparent)
    UnderWindow,
    /// Menu material
    Menu,
    /// Popover material
    Popover,
}

pub struct VibrancyOpacity {
    pub main_bg: f32,      // 0.0-1.0
    pub sidebar: f32,
    pub panel: f32,
    pub input: f32,
}

impl Theme {
    /// Get computed background color with vibrancy
    pub fn bg_with_vibrancy(&self, base: Hsla, region: VibrancyRegion) -> Hsla {
        if !self.vibrancy_enabled() {
            return base;
        }
        
        let opacity = match region {
            VibrancyRegion::Main => self.vibrancy.opacity.main_bg,
            VibrancyRegion::Sidebar => self.vibrancy.opacity.sidebar,
            VibrancyRegion::Panel => self.vibrancy.opacity.panel,
            VibrancyRegion::Input => self.vibrancy.opacity.input,
        };
        
        base.with_alpha(opacity)
    }
}
```

### 5. Theme Service

```rust
/// Centralized theme management service
pub struct ThemeService {
    /// Current adaptive theme
    theme: AdaptiveTheme,
    /// Theme file watcher
    watcher: Option<ThemeWatcher>,
    /// Registered windows to notify
    windows: Vec<WindowHandle<Root>>,
}

impl ThemeService {
    /// Get global theme service
    pub fn global() -> &'static Mutex<ThemeService> {
        static SERVICE: OnceLock<Mutex<ThemeService>> = OnceLock::new();
        SERVICE.get_or_init(|| Mutex::new(ThemeService::new()))
    }
    
    /// Get current semantic colors
    pub fn colors(&self) -> &SemanticColors {
        &self.theme.current().semantic
    }
    
    /// Get current theme for rendering
    pub fn current_theme(&self) -> &Theme {
        self.theme.current()
    }
    
    /// Set color scheme preference
    pub fn set_preference(&mut self, pref: ColorSchemePreference, cx: &mut App) {
        self.theme.preference = pref;
        self.notify_all(cx);
    }
    
    /// Load theme from preset
    pub fn load_preset(&mut self, preset: ThemePreset, cx: &mut App) {
        self.theme = AdaptiveTheme {
            light: preset.load(),
            dark: preset.load(),
            preference: self.theme.preference,
        };
        self.notify_all(cx);
    }
    
    /// Reload theme from file
    pub fn reload(&mut self, cx: &mut App) -> Result<()> {
        self.theme = AdaptiveTheme::load_from_config()?;
        self.notify_all(cx);
        Ok(())
    }
    
    fn notify_all(&self, cx: &mut App) {
        for handle in &self.windows {
            let _ = handle.update(cx, |_, _, cx| cx.notify());
        }
    }
}
```

### 6. Theme Configuration File

```json
// ~/.scriptkit/theme.json
{
  "version": 2,
  "colorScheme": "system",  // "system" | "light" | "dark"
  "preset": "script-kit",   // Preset name or null for custom
  
  "vibrancy": {
    "enabled": true,
    "material": "sidebar",
    "opacity": {
      "main": 0.85,
      "sidebar": 0.90,
      "panel": 0.80,
      "input": 0.95
    }
  },
  
  "dark": {
    "background": {
      "app": "#1e1e1e",
      "surface": "#252526",
      "elevated": "#2d2d30"
    },
    "text": {
      "primary": "#ffffff",
      "secondary": "#cccccc",
      "muted": "#808080"
    },
    "accent": {
      "primary": "#fbbf24",
      "success": "#4ade80",
      "warning": "#fb923c",
      "error": "#f87171"
    }
  },
  
  "light": {
    "background": {
      "app": "#ffffff",
      "surface": "#f5f5f5",
      "elevated": "#ffffff"
    },
    "text": {
      "primary": "#1a1a1a",
      "secondary": "#4a4a4a",
      "muted": "#808080"
    }
  }
}
```

---

## Theme Editor Built-in

```typescript
// Script Kit theme editor
const preview = async (colors: ColorPalette) => {
  await div(`
    <div class="p-4 space-y-4" style="background: ${colors.bg.app}">
      <div class="p-3 rounded" style="background: ${colors.bg.surface}">
        <h2 style="color: ${colors.text.primary}">Primary Text</h2>
        <p style="color: ${colors.text.secondary}">Secondary text</p>
      </div>
      <button style="background: ${colors.accent.primary}; color: ${colors.text.inverse}">
        Button
      </button>
    </div>
  `);
};
```

---

## Implementation Checklist

### Phase 1: Semantic Colors
- [ ] Define `SemanticColors` struct
- [ ] Create mapping from current ColorScheme
- [ ] Update components to use semantic tokens
- [ ] Add focus-aware variants

### Phase 2: Light Mode
- [ ] Create light theme preset
- [ ] Implement `AdaptiveTheme`
- [ ] Add system appearance detection
- [ ] Handle appearance change events

### Phase 3: Theme Service
- [ ] Create `ThemeService` singleton
- [ ] Implement window notification
- [ ] Add hot-reload support
- [ ] Integrate with config watcher

### Phase 4: Vibrancy Centralization
- [ ] Create `VibrancyConfig` struct
- [ ] Move opacity helpers to theme
- [ ] Update all windows to use theme vibrancy
- [ ] Test on different macOS versions

### Phase 5: Theme Tools
- [ ] Add theme preset selector built-in
- [ ] Create theme preview component
- [ ] Add theme editor script template
- [ ] Document theme.json format

---

## Key Questions

1. Should light/dark themes be separate files or combined?
2. How to handle third-party theme imports (VS Code themes)?
3. Should vibrancy be per-window configurable?
4. How to validate theme.json without crashing?
5. Should there be a "theme gallery" for discovery?

---

## Related Bundles

- Bundle 22: Vibrancy Bundle - current vibrancy implementation
- Bundle 39: Window Management - theme syncing across windows
- Bundle 42: App Shell - uses theme for chrome styling

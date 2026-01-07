# Script Kit GPUI - Expert Bundle 45: Icon Library Integration

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**Current Icon Usage:**
- Logo SVG embedded in binary
- Lucide icons via gpui-component
- Custom SVG icons in `designs/icon_variations.rs`
- App icons loaded from bundle paths

---

## Goal

Create a **comprehensive icon system** that:
1. Provides consistent icons across all UI surfaces
2. Supports multiple icon libraries (Lucide, SF Symbols, custom)
3. Enables icon theming (color, size, weight)
4. Allows scripts to specify custom icons
5. Optimizes icon loading and caching

---

## Current State

### Icon Sources

| Source | Location | Count | Usage |
|--------|----------|-------|-------|
| Logo | `assets/logo.svg` | 1 | Main menu header |
| Lucide | gpui-component | ~100 | Buttons, dialogs |
| Custom | `designs/icon_variations.rs` | ~50 | Design gallery |
| App Icons | `/Applications/*.app` | N/A | App launcher |

### Current Implementation

```rust
// gpui-component provides IconName enum
use gpui_component::{Icon, IconName};

// Usage in buttons
Button::new("Delete")
    .icon(IconName::Trash)
    .on_click(...)

// Custom SVG icons need external_path
use crate::designs::icon_variations::IconName as LocalIconName;

svg()
    .external_path(LocalIconName::Settings.external_path())
    .size(px(16.))
    .text_color(rgb(0xffffff))
```

### Problems

1. **Two IconName enums** - gpui-component's and our custom one conflict
2. **No unified API** - Different patterns for different icon types
3. **No dynamic icons** - Scripts can't specify custom icons
4. **No caching** - App icons loaded repeatedly
5. **Limited SF Symbols** - Can't use macOS native symbols
6. **No icon search** - Can't browse available icons

---

## Proposed Architecture

### 1. Unified Icon System

```rust
/// Unified icon reference that can resolve to any icon source
#[derive(Clone, Debug)]
pub enum IconRef {
    /// Lucide icon from gpui-component
    Lucide(LucideIcon),
    /// SF Symbol (macOS native)
    SFSymbol(String),
    /// Custom SVG embedded in binary
    Embedded(EmbeddedIcon),
    /// External SVG file path
    File(PathBuf),
    /// App bundle icon
    App { bundle_id: String },
    /// Base64-encoded SVG
    Inline(String),
    /// URL to fetch icon from
    Url(String),
}

#[derive(Clone, Copy, Debug)]
pub enum LucideIcon {
    Search,
    Settings,
    Terminal,
    File,
    Folder,
    // ... all Lucide icons
}

#[derive(Clone, Copy, Debug)]
pub enum EmbeddedIcon {
    Logo,
    Clipboard,
    AppLauncher,
    WindowSwitcher,
    Notes,
    AI,
    // ... our custom icons
}

impl IconRef {
    /// Resolve to renderable element
    pub fn render(&self, config: IconConfig) -> impl IntoElement {
        match self {
            Self::Lucide(icon) => render_lucide(*icon, config),
            Self::SFSymbol(name) => render_sf_symbol(name, config),
            Self::Embedded(icon) => render_embedded(*icon, config),
            Self::File(path) => render_file_icon(path, config),
            Self::App { bundle_id } => render_app_icon(bundle_id, config),
            Self::Inline(svg) => render_inline_svg(svg, config),
            Self::Url(url) => render_url_icon(url, config),
        }
    }
    
    /// Parse from string (for scripts)
    pub fn parse(s: &str) -> Self {
        if s.starts_with("lucide:") {
            Self::Lucide(LucideIcon::from_str(&s[7..]).unwrap_or(LucideIcon::File))
        } else if s.starts_with("sf:") {
            Self::SFSymbol(s[3..].to_string())
        } else if s.starts_with("app:") {
            Self::App { bundle_id: s[4..].to_string() }
        } else if s.starts_with("data:") || s.starts_with("<svg") {
            Self::Inline(s.to_string())
        } else if s.starts_with("http://") || s.starts_with("https://") {
            Self::Url(s.to_string())
        } else if s.ends_with(".svg") {
            Self::File(PathBuf::from(s))
        } else {
            // Default to Lucide icon lookup
            Self::Lucide(LucideIcon::from_str(s).unwrap_or(LucideIcon::File))
        }
    }
}
```

### 2. Icon Configuration

```rust
#[derive(Clone, Copy, Debug)]
pub struct IconConfig {
    /// Size in pixels
    pub size: f32,
    /// Color (inherits from text_color if None)
    pub color: Option<u32>,
    /// Weight/stroke width for line icons
    pub weight: IconWeight,
    /// Filled vs outlined variant
    pub variant: IconVariant,
    /// Opacity (0.0 - 1.0)
    pub opacity: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum IconWeight {
    Light,
    #[default]
    Regular,
    Medium,
    Bold,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum IconVariant {
    #[default]
    Outline,
    Filled,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: None,
            weight: IconWeight::Regular,
            variant: IconVariant::Outline,
            opacity: 1.0,
        }
    }
}

impl IconConfig {
    pub fn new() -> Self { Self::default() }
    pub fn size(mut self, s: f32) -> Self { self.size = s; self }
    pub fn color(mut self, c: u32) -> Self { self.color = Some(c); self }
    pub fn weight(mut self, w: IconWeight) -> Self { self.weight = w; self }
    pub fn filled(mut self) -> Self { self.variant = IconVariant::Filled; self }
}
```

### 3. Icon Component

```rust
/// Unified icon component
#[derive(IntoElement)]
pub struct IconView {
    icon: IconRef,
    config: IconConfig,
}

impl IconView {
    pub fn new(icon: impl Into<IconRef>) -> Self {
        Self {
            icon: icon.into(),
            config: IconConfig::default(),
        }
    }
    
    pub fn config(mut self, c: IconConfig) -> Self {
        self.config = c;
        self
    }
    
    // Convenience methods
    pub fn size(mut self, s: f32) -> Self { self.config.size = s; self }
    pub fn color(mut self, c: u32) -> Self { self.config.color = Some(c); self }
}

impl RenderOnce for IconView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.icon.render(self.config)
    }
}
```

### 4. SF Symbols Integration

```rust
#[cfg(target_os = "macos")]
mod sf_symbols {
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl};
    
    /// Render SF Symbol as NSImage, convert to GPUI texture
    pub fn render_sf_symbol(name: &str, config: IconConfig) -> impl IntoElement {
        // 1. Create NSImage from SF Symbol name
        let ns_name = NSString::alloc(nil).init_str(name);
        let image: id = msg_send![class!(NSImage), imageWithSystemSymbolName:ns_name accessibilityDescription:nil];
        
        // 2. Apply weight configuration
        let weight = match config.weight {
            IconWeight::Light => 1,  // NSFontWeightLight
            IconWeight::Regular => 5, // NSFontWeightRegular
            IconWeight::Medium => 6,  // NSFontWeightMedium  
            IconWeight::Bold => 8,    // NSFontWeightBold
        };
        
        // 3. Convert to CGImage
        // 4. Create GPUI texture
        // 5. Return svg() or img() element
    }
    
    /// List all available SF Symbols (for icon picker)
    pub fn list_sf_symbols() -> Vec<String> {
        // Parse SF Symbols catalog
    }
}
```

### 5. Icon Cache

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global icon cache for expensive-to-load icons
pub struct IconCache {
    /// App icon textures (bundle_id -> texture)
    app_icons: RwLock<HashMap<String, Arc<gpui::RenderImage>>>,
    /// URL icon textures (url -> texture)
    url_icons: RwLock<HashMap<String, Arc<gpui::RenderImage>>>,
    /// File icon textures (path -> texture)
    file_icons: RwLock<HashMap<PathBuf, Arc<gpui::RenderImage>>>,
}

impl IconCache {
    pub fn global() -> &'static Self {
        static CACHE: OnceLock<IconCache> = OnceLock::new();
        CACHE.get_or_init(IconCache::new)
    }
    
    pub fn get_app_icon(&self, bundle_id: &str) -> Option<Arc<gpui::RenderImage>> {
        self.app_icons.read().ok()?.get(bundle_id).cloned()
    }
    
    pub fn cache_app_icon(&self, bundle_id: &str, image: Arc<gpui::RenderImage>) {
        if let Ok(mut cache) = self.app_icons.write() {
            cache.insert(bundle_id.to_string(), image);
        }
    }
    
    /// Clear cache (e.g., on memory pressure)
    pub fn clear(&self) {
        if let Ok(mut c) = self.app_icons.write() { c.clear(); }
        if let Ok(mut c) = self.url_icons.write() { c.clear(); }
        if let Ok(mut c) = self.file_icons.write() { c.clear(); }
    }
}
```

### 6. SDK Integration

```typescript
// TypeScript SDK for scripts
interface Choice {
  name: string;
  value: any;
  description?: string;
  // Icon specification
  icon?: string | IconSpec;
}

interface IconSpec {
  // Type of icon
  type: "lucide" | "sf" | "app" | "url" | "svg";
  // Icon identifier (name, bundle_id, url, or SVG string)
  value: string;
  // Optional customization
  color?: string;  // Hex color
  size?: number;   // Pixels
}

// Examples:
const choices: Choice[] = [
  { 
    name: "Terminal", 
    icon: "terminal"  // Lucide icon
  },
  { 
    name: "Finder", 
    icon: "app:com.apple.finder"  // App icon
  },
  { 
    name: "Settings", 
    icon: "sf:gear"  // SF Symbol
  },
  { 
    name: "Custom",
    icon: {
      type: "url",
      value: "https://example.com/icon.svg",
      color: "#ff0000"
    }
  },
];
```

---

## Icon Library Catalog

### Proposed Embedded Icons

```rust
pub enum EmbeddedIcon {
    // Branding
    Logo,
    LogoMono,
    
    // Features
    Clipboard,
    ClipboardSearch,
    AppLauncher,
    WindowSwitcher,
    Notes,
    AI,
    Terminal,
    Editor,
    
    // Actions
    Run,
    Edit,
    Delete,
    Duplicate,
    Share,
    
    // Navigation
    Back,
    Forward,
    Up,
    Down,
    
    // Status
    Success,
    Warning,
    Error,
    Info,
    Loading,
    
    // File types
    Script,
    Scriptlet,
    Markdown,
    JSON,
    TypeScript,
    
    // Misc
    Gear,
    Keyboard,
    Magic,
}
```

---

## Implementation Checklist

### Phase 1: Core Icon System
- [ ] Create `src/icons/mod.rs` module
- [ ] Implement `IconRef` enum with all variants
- [ ] Implement `IconConfig` with builder pattern
- [ ] Create `IconView` component
- [ ] Migrate existing icon usage

### Phase 2: Icon Sources
- [ ] Embed custom SVG icons in binary
- [ ] Implement SF Symbols renderer (macOS)
- [ ] Implement app icon loader with cache
- [ ] Implement URL icon fetcher with cache

### Phase 3: SDK Integration
- [ ] Add icon field to Choice protocol
- [ ] Parse icon specifications in protocol handler
- [ ] Document icon format in SDK docs

### Phase 4: Icon Tools
- [ ] Create icon browser built-in command
- [ ] Add icon search to design gallery
- [ ] Support icon preview in list items

### Phase 5: Performance
- [ ] Implement icon cache with LRU eviction
- [ ] Add lazy loading for URL icons
- [ ] Profile icon rendering performance

---

## Key Questions

1. Should we bundle SF Symbols font, or require macOS 11+?
2. How large should the icon cache be (memory vs. performance)?
3. Should we support animated icons (Lottie)?
4. How to handle icon fallbacks when primary source fails?
5. Should scripts be able to define custom icon sets?

---

## Related Bundles

- Bundle 43: Shared UI Components - uses icons in list items
- Bundle 34: Main Menu Patterns - displays script icons
- Bundle 51: Actions/Context Menu - uses icons in menus

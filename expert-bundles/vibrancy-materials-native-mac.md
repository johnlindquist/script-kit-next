# Vibrancy & Materials - Native macOS Appearance Bundle

## Goal
Make the app look like a native macOS app (Raycast/Spotlight). Something is missing.

## Current Implementation

### Window Background (GPUI)
```rust
// main.rs, notes/window.rs, actions/window.rs
let window_background = if theme.is_vibrancy_enabled() {
    gpui::WindowBackgroundAppearance::Blurred  // Enables NSVisualEffectView
} else {
    gpui::WindowBackgroundAppearance::Opaque
};
```

### NSVisualEffectView Config (platform.rs)
```rust
pub fn configure_window_vibrancy_material() {
    // 1. Set appearance to DarkAqua
    let dark_appearance: id = msg_send![
        class!(NSAppearance), appearanceNamed: NSAppearanceNameDarkAqua
    ];
    let _: () = msg_send![window, setAppearance: dark_appearance];

    // 2. Find NSVisualEffectView and configure
    let _: () = msg_send![subview, setMaterial: SIDEBAR]; // 7
    let _: () = msg_send![subview, setState: 1isize];     // Active
    let _: () = msg_send![subview, setBlendingMode: 0isize]; // BehindWindow
    let _: () = msg_send![subview, setEmphasized: true];
}
```

### Available Materials
```rust
pub mod ns_visual_effect_material {
    pub const SELECTION: isize = 4;    // GPUI default
    pub const MENU: isize = 5;
    pub const POPOVER: isize = 6;
    pub const SIDEBAR: isize = 7;      // Currently used
    pub const HUD_WINDOW: isize = 13;  // Dark, high contrast
    pub const DARK: isize = 2;         // Deprecated
    pub const MEDIUM_DARK: isize = 8;  // Undocumented
    pub const ULTRA_DARK: isize = 9;   // Undocumented
}
```

### Available Appearances
```rust
extern "C" {
    static NSAppearanceNameDarkAqua: id;      // Currently used
    static NSAppearanceNameAqua: id;
    static NSAppearanceNameVibrantDark: id;   // Raycast likely uses this
    static NSAppearanceNameVibrantLight: id;
}
```

### Background Opacity (theme/types.rs)
```rust
BackgroundOpacity {
    main: 0.30,        // Root background
    title_bar: 0.30,
    search_box: 0.40,
    selected: 0.15,
    hover: 0.08,
    preview: 0.0,      // Fully transparent
    dialog: 0.35,
}
```

### GPUI Theme Integration (gpui_integration.rs)
```rust
// Main background is FULLY TRANSPARENT when vibrancy enabled
let main_bg = if vibrancy_enabled {
    hsla(0.0, 0.0, 0.0, 0.0) // Fully transparent
} else {
    hex_to_hsla(colors.background.main)
};
```

## Debug Tool
Press **Cmd+Shift+M** to cycle through all material/appearance combinations.

## What Raycast Does Differently

1. **Appearance**: Uses `VibrantDark` (not `DarkAqua`)
2. **Material**: Likely `HUD_WINDOW` (13) or private material
3. **Background**: Near-transparent (5-15%)
4. **Window backgroundColor**: Probably `[NSColor clearColor]`

## Key Questions

1. **Material**: Should we use `HUD_WINDOW` instead of `SIDEBAR`?
2. **Appearance**: `VibrantDark` vs `DarkAqua` - what's the difference?
3. **Window backgroundColor**: Should we set `[NSColor clearColor]`?
4. **Blur intensity**: Any way to control it?

## Files

| File | Purpose |
|------|---------|
| `src/platform.rs:420-720` | NSVisualEffectView config, material cycling |
| `src/theme/types.rs:17-200` | VibrancySettings, BackgroundOpacity |
| `src/theme/gpui_integration.rs` | GPUI theme color mapping |
| `src/main.rs:1445-1460` | Window creation |
| `src/actions/window.rs:84-170` | Popup window vibrancy |
| `src/notes/window.rs:1900-1920` | Notes window vibrancy |

## Recommended Experiments

Try these combinations with Cmd+Shift+M:
1. `HUD_WINDOW` + `VibrantDark` + `BehindWindow` + `Emphasized`
2. `ULTRA_DARK` + `VibrantDark` + `BehindWindow` + `Emphasized`
3. `POPOVER` + `VibrantDark` + `BehindWindow` + `Emphasized`

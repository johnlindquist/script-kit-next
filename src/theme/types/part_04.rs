/// Convert a HexColor to RGBA components (0.0-1.0 range)
fn hex_to_rgba_components(hex: HexColor, alpha: f32) -> (f32, f32, f32, f32) {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    (r, g, b, alpha.clamp(0.0, 1.0))
}

#[allow(dead_code)]
impl Theme {
    /// Get the appropriate color scheme based on window focus state
    ///
    /// If focus-aware colors are configured:
    /// - Returns focused colors when focused=true
    /// - Returns unfocused colors when focused=false
    ///
    /// If focus-aware colors are not configured:
    /// - Always returns the standard colors (automatic dimmed version for unfocused)
    pub fn get_colors(&self, is_focused: bool) -> ColorScheme {
        if let Some(ref focus_aware) = self.focus_aware {
            if is_focused {
                if let Some(ref focused) = focus_aware.focused {
                    return focused.to_color_scheme();
                }
            } else if let Some(ref unfocused) = focus_aware.unfocused {
                return unfocused.to_color_scheme();
            }
        }

        // Fallback: use standard colors, with automatic dimming for unfocused
        if is_focused {
            self.colors.clone()
        } else {
            self.colors.to_unfocused()
        }
    }

    /// Get cursor style if window is focused
    pub fn get_cursor_style(&self, is_focused: bool) -> Option<CursorStyle> {
        if !is_focused {
            return None;
        }

        if let Some(ref focus_aware) = self.focus_aware {
            if let Some(ref focused) = focus_aware.focused {
                return focused.cursor.clone();
            }
        }

        // Return default blinking cursor if focused
        Some(CursorStyle::default_focused())
    }

    /// Get background opacity settings
    /// Returns the configured opacity or sensible defaults
    pub fn get_opacity(&self) -> BackgroundOpacity {
        self.opacity.clone().unwrap_or_default()
    }

    /// Create a new theme with opacity adjusted by an offset
    ///
    /// Use Cmd+Shift+[ to decrease and Cmd+Shift+] to increase opacity.
    /// The offset is added to all opacity values (clamped to 0.0-1.0).
    ///
    /// # Arguments
    /// * `offset` - The amount to add to opacity values (can be negative)
    ///
    /// # Returns
    /// A new Theme with adjusted opacity values
    pub fn with_opacity_offset(&self, offset: f32) -> Theme {
        let mut theme = self.clone();
        let base = theme.get_opacity();
        theme.opacity = Some(BackgroundOpacity {
            main: (base.main + offset).clamp(0.0, 1.0),
            title_bar: (base.title_bar + offset).clamp(0.0, 1.0),
            search_box: (base.search_box + offset).clamp(0.0, 1.0),
            log_panel: (base.log_panel + offset).clamp(0.0, 1.0),
            selected: base.selected, // Keep selection/hover unchanged
            hover: base.hover,
            preview: base.preview,
            dialog: (base.dialog + offset).clamp(0.0, 1.0),
            input: (base.input + offset).clamp(0.0, 1.0),
            panel: (base.panel + offset).clamp(0.0, 1.0),
            input_inactive: (base.input_inactive + offset).clamp(0.0, 1.0),
            input_active: (base.input_active + offset).clamp(0.0, 1.0),
            border_inactive: base.border_inactive,
            border_active: base.border_active,
            vibrancy_background: base.vibrancy_background,
        });
        theme
    }

    /// Get opacity adjusted for focus state
    /// Unfocused windows are slightly more transparent
    pub fn get_opacity_for_focus(&self, is_focused: bool) -> BackgroundOpacity {
        let base = self.get_opacity();
        if is_focused {
            base
        } else {
            // Reduce opacity by 10% when unfocused
            BackgroundOpacity {
                main: (base.main * 0.9).clamp(0.0, 1.0),
                title_bar: (base.title_bar * 0.9).clamp(0.0, 1.0),
                search_box: (base.search_box * 0.9).clamp(0.0, 1.0),
                log_panel: (base.log_panel * 0.9).clamp(0.0, 1.0),
                selected: base.selected,
                hover: base.hover,
                preview: (base.preview * 0.9).clamp(0.0, 1.0),
                dialog: (base.dialog * 0.9).clamp(0.0, 1.0),
                input: (base.input * 0.9).clamp(0.0, 1.0),
                panel: (base.panel * 0.9).clamp(0.0, 1.0),
                input_inactive: (base.input_inactive * 0.9).clamp(0.0, 1.0),
                input_active: (base.input_active * 0.9).clamp(0.0, 1.0),
                border_inactive: (base.border_inactive * 0.9).clamp(0.0, 1.0),
                border_active: (base.border_active * 0.9).clamp(0.0, 1.0),
                vibrancy_background: base.vibrancy_background.map(|v| (v * 0.9).clamp(0.0, 1.0)),
            }
        }
    }

    /// Get drop shadow configuration
    /// Returns the configured shadow or sensible defaults
    pub fn get_drop_shadow(&self) -> DropShadow {
        self.drop_shadow.clone().unwrap_or_default()
    }

    /// Get vibrancy/blur effect settings
    /// Returns the configured vibrancy or sensible defaults
    pub fn get_vibrancy(&self) -> VibrancySettings {
        self.vibrancy.clone().unwrap_or_default()
    }

    /// Check if vibrancy effect should be enabled
    pub fn is_vibrancy_enabled(&self) -> bool {
        self.get_vibrancy().enabled
    }

    /// Get font configuration
    /// Returns the configured fonts or sensible defaults
    pub fn get_fonts(&self) -> FontConfig {
        self.fonts.clone().unwrap_or_default()
    }

    /// Get background RGBA color for a specific role
    ///
    /// This is the single correct way to get background colors with opacity applied.
    /// It combines the color from the color scheme with the appropriate opacity value,
    /// ensuring consistent vibrancy support across the UI.
    ///
    /// # Arguments
    /// * `role` - The background role (Main, TitleBar, SearchBox, LogPanel)
    /// * `is_focused` - Whether the window is currently focused
    ///
    /// # Returns
    /// A tuple of (r, g, b, a) with values in the 0.0-1.0 range
    ///
    /// # Example
    /// ```ignore
    /// let (r, g, b, a) = theme.background_rgba(BackgroundRole::Main, true);
    /// div().bg(rgba(r, g, b, a))
    /// ```
    pub fn background_rgba(&self, role: BackgroundRole, is_focused: bool) -> (f32, f32, f32, f32) {
        let colors = self.get_colors(is_focused);
        let opacity = self.get_opacity_for_focus(is_focused).clamped();

        match role {
            BackgroundRole::Main => hex_to_rgba_components(colors.background.main, opacity.main),
            BackgroundRole::TitleBar => {
                hex_to_rgba_components(colors.background.title_bar, opacity.title_bar)
            }
            BackgroundRole::SearchBox => {
                hex_to_rgba_components(colors.background.search_box, opacity.search_box)
            }
            BackgroundRole::LogPanel => {
                hex_to_rgba_components(colors.background.log_panel, opacity.log_panel)
            }
        }
    }
}

/// Detect system appearance preference on macOS (cached)
///
/// Returns true if dark mode is enabled, false if light mode is enabled.
/// On non-macOS systems or if detection fails, defaults to true (dark mode).
///
/// This function caches the result for 5 seconds to avoid spawning subprocesses
/// on every render call. The system appearance doesn't change frequently, so
/// a small TTL is acceptable.
///
/// Uses the `defaults read -g AppleInterfaceStyle` command to detect the system appearance.
/// Note: On macOS in light mode, the command exits with non-zero status because the
/// AppleInterfaceStyle key doesn't exist, so we check exit status explicitly.
pub fn detect_system_appearance() -> bool {
    let cache = APPEARANCE_CACHE.get_or_init(|| Mutex::new(AppearanceCache::default()));

    let mut cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            // Mutex poisoned, return default
            return true;
        }
    };

    // Check if cache is still valid
    if cache_guard.last_check.elapsed() < APPEARANCE_CACHE_TTL {
        return cache_guard.is_dark;
    }

    // Cache expired, re-detect
    let is_dark = detect_system_appearance_uncached();
    cache_guard.is_dark = is_dark;
    cache_guard.last_check = Instant::now();
    is_dark
}

/// Invalidate the appearance cache
///
/// Call this when the system appearance changes (e.g., from observe_window_appearance)
/// to force immediate re-detection on the next call to `detect_system_appearance()`.
pub fn invalidate_appearance_cache() {
    if let Some(cache) = APPEARANCE_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            // Set last_check to past the TTL so next call will re-detect
            guard.last_check = Instant::now() - APPEARANCE_CACHE_TTL - Duration::from_secs(1);
            debug!("Appearance cache invalidated");
        }
    }
}

/// Uncached system appearance detection (internal use)
fn detect_system_appearance_uncached() -> bool {
    // Default to dark mode if detection fails or we're not on macOS
    const DEFAULT_DARK: bool = true;

    // Try to detect macOS dark mode using system defaults
    match Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        Ok(output) => {
            // In light mode, the AppleInterfaceStyle key typically doesn't exist,
            // causing the command to exit with non-zero status
            if !output.status.success() {
                debug!(
                    appearance = "light",
                    "System appearance detected (key not present)"
                );
                return false; // light mode
            }

            // If the command succeeds and returns "Dark", we're in dark mode
            let stdout = String::from_utf8_lossy(&output.stdout);
            let is_dark = stdout.to_lowercase().contains("dark");
            debug!(
                appearance = if is_dark { "dark" } else { "light" },
                "System appearance detected"
            );
            is_dark
        }
        Err(e) => {
            // Command failed to execute (e.g., not on macOS, or `defaults` not found)
            debug!(
                error = %e,
                default = DEFAULT_DARK,
                "System appearance detection failed, using default"
            );
            DEFAULT_DARK
        }
    }
}


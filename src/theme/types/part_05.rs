fn default_theme_from_system_appearance() -> Theme {
    if detect_system_appearance() {
        Theme::dark_default()
    } else {
        Theme::light_default()
    }
}

fn theme_from_user_preferences(
    preferences: &crate::config::ScriptKitUserPreferences,
    correlation_id: &str,
) -> Option<Theme> {
    let preset_id = preferences.theme.preset_id.as_ref()?.trim();
    if preset_id.is_empty() {
        warn!(
            correlation_id = %correlation_id,
            "Theme preset id in settings is empty; ignoring"
        );
        return None;
    }

    let preset = super::presets::all_presets()
        .into_iter()
        .find(|candidate| candidate.id == preset_id);

    match preset {
        Some(selected) => {
            debug!(
                correlation_id = %correlation_id,
                preset_id = selected.id,
                preset_name = selected.name,
                "Using theme preset from user preferences"
            );
            Some(selected.create_theme())
        }
        None => {
            warn!(
                correlation_id = %correlation_id,
                preset_id,
                "Unknown theme preset id in settings; falling back to theme file/default"
            );
            None
        }
    }
}

fn load_theme_from_user_preferences(correlation_id: &str) -> Option<Theme> {
    let preferences = crate::config::load_user_preferences();
    theme_from_user_preferences(&preferences, correlation_id)
}

/// Load theme from `<SK_PATH>/kit/theme.json` (or `~/.scriptkit/kit/theme.json`)
///
/// Colors should be specified as decimal integers in the JSON file.
/// For example, 0x1e1e1e (hex) = 1980410 (decimal).
///
/// Example theme.json structure:
/// ```json
/// {
///   "colors": {
///     "background": {
///       "main": 1980410,
///       "title_bar": 2961712,
///       "search_box": 3947580,
///       "log_panel": 851213
///     },
///     "text": {
///       "primary": 16777215,
///       "secondary": 14737920,
///       "tertiary": 10066329,
///       "muted": 8421504,
///       "dimmed": 6710886
///     },
///     "accent": {
///       "selected": 31948
///     },
///     "ui": {
///       "border": 4609607,
///       "success": 65280
///     }
///   }
/// }
/// ```
///
/// If the file doesn't exist or fails to parse, returns a theme based on system appearance detection.
/// If system appearance detection is not available, defaults to dark mode.
/// Logs errors to stderr but doesn't fail the application.
pub fn load_theme() -> Theme {
    let correlation_id = format!("theme_load:{}", uuid::Uuid::new_v4());

    if let Some(theme) = load_theme_from_user_preferences(&correlation_id) {
        log_theme_config(&theme);
        return theme;
    }

    let theme_path = crate::setup::get_kit_path().join("kit").join("theme.json");

    // Check if theme file exists
    if !theme_path.exists() {
        warn!(
            correlation_id = %correlation_id,
            path = %theme_path.display(),
            "Theme file not found, using defaults based on system appearance"
        );
        let theme = default_theme_from_system_appearance();
        log_theme_config(&theme);
        return theme;
    }

    // Read and parse the JSON file
    match std::fs::read_to_string(&theme_path) {
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                path = %theme_path.display(),
                error = %e,
                "Failed to read theme file, using defaults"
            );
            let theme = default_theme_from_system_appearance();
            log_theme_config(&theme);
            theme
        }
        Ok(contents) => match serde_json::from_str::<Theme>(&contents) {
            Ok(mut theme) => {
                debug!(
                    correlation_id = %correlation_id,
                    path = %theme_path.display(),
                    "Successfully loaded theme"
                );

                // Key behavior: When appearance is Auto, use system appearance to
                // determine which color scheme to use (light or dark).
                // This allows the app to follow macOS light/dark mode automatically.
                let is_system_dark = detect_system_appearance();
                let should_use_light = match theme.appearance {
                    AppearanceMode::Light => true,
                    AppearanceMode::Dark => false,
                    AppearanceMode::Auto => !is_system_dark, // Follow system
                };

                if should_use_light {
                    // System is in light mode (or explicitly set to light)
                    // Use light color scheme, but preserve any non-color settings from theme.json
                    let light_colors = ColorScheme::light_default();
                    theme.colors = light_colors;
                    theme.appearance = AppearanceMode::Light; // Mark as light for consistency

                    // Use light opacity defaults
                    if theme.opacity.is_none() {
                        theme.opacity = Some(BackgroundOpacity::light_default());
                    }

                    debug!(
                        correlation_id = %correlation_id,
                        system_appearance = if is_system_dark { "dark" } else { "light" },
                        "Using light theme colors (system is in light mode)"
                    );
                } else {
                    // System is in dark mode (or explicitly set to dark)
                    // Use the colors from theme.json (which are dark)
                    if theme.opacity.is_none() {
                        theme.opacity = Some(BackgroundOpacity::dark_default());
                    }
                }

                log_theme_config(&theme);
                theme
            }
            Err(e) => {
                error!(
                    correlation_id = %correlation_id,
                    path = %theme_path.display(),
                    error = %e,
                    "Failed to parse theme JSON, using defaults"
                );
                debug!(correlation_id = %correlation_id, content = %contents, "Malformed theme file content");
                let theme = default_theme_from_system_appearance();
                log_theme_config(&theme);
                theme
            }
        },
    }
}

/// Get a cached version of the theme for use in render functions
///
/// This avoids file I/O on every render call by caching the loaded theme.
/// The cache is automatically invalidated when `invalidate_theme_cache()` is called
/// (typically by the theme file watcher).
///
/// # Performance
///
/// Use this function instead of `load_theme()` in render paths:
/// - Render methods
/// - Background color calculations
/// - Any code that runs frequently
///
/// Use `load_theme()` for:
/// - Initial setup
/// - When you need guaranteed fresh theme data
/// - After explicitly invalidating the cache
pub fn get_cached_theme() -> Theme {
    let cache = THEME_CACHE.get_or_init(|| Mutex::new(ThemeCache::default()));

    let cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            // Mutex poisoned, return default
            return Theme::dark_default();
        }
    };

    cache_guard.theme.clone()
}

/// Reload and cache the theme from disk
///
/// Call this when you need to refresh the cached theme (e.g., from the theme watcher).
/// This function loads the theme from disk and updates the cache.
pub fn reload_theme_cache() -> Theme {
    let theme = load_theme();

    let cache = THEME_CACHE.get_or_init(|| Mutex::new(ThemeCache::default()));
    if let Ok(mut guard) = cache.lock() {
        guard.theme = theme.clone();
        guard.loaded_at = Instant::now();
        debug!("Theme cache reloaded");
    }

    theme
}

/// Initialize the theme cache on startup
///
/// Call this during app initialization to ensure the theme is loaded
/// before any render calls. This ensures `get_cached_theme()` returns
/// the correct theme from the first render.
pub fn init_theme_cache() {
    reload_theme_cache();
    debug!("Theme cache initialized");
}

/// Invalidate the theme cache, forcing a reload on next access
///
/// Call this when the theme file changes to ensure the next call to
/// `get_cached_theme()` or `reload_theme_cache()` loads fresh data.
#[allow(dead_code)]
pub fn invalidate_theme_cache() {
    if let Some(cache) = THEME_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            // Force reload on next access by setting old timestamp
            guard.loaded_at = Instant::now() - Duration::from_secs(3600);
            debug!("Theme cache invalidated");
        }
    }
}

// ============================================================================
// End Lightweight Theme Extraction Helpers
// ============================================================================

/// Log theme configuration for debugging
fn log_theme_config(theme: &Theme) {
    let opacity = theme.get_opacity();
    let shadow = theme.get_drop_shadow();
    let vibrancy = theme.get_vibrancy();
    debug!(
        opacity_main = opacity.main,
        opacity_title_bar = opacity.title_bar,
        opacity_search_box = opacity.search_box,
        opacity_log_panel = opacity.log_panel,
        "Theme opacity configured"
    );
    debug!(
        shadow_enabled = shadow.enabled,
        blur_radius = shadow.blur_radius,
        spread_radius = shadow.spread_radius,
        offset_x = shadow.offset_x,
        offset_y = shadow.offset_y,
        shadow_opacity = shadow.opacity,
        "Theme shadow configured"
    );
    debug!(
        vibrancy_enabled = vibrancy.enabled,
        material = %vibrancy.material,
        "Theme vibrancy configured"
    );
    debug!(
        selected = format!("#{:06x}", theme.colors.accent.selected),
        selected_subtle = format!("#{:06x}", theme.colors.accent.selected_subtle),
        "Theme accent colors"
    );
    debug!(
        error = format!("#{:06x}", theme.colors.ui.error),
        warning = format!("#{:06x}", theme.colors.ui.warning),
        info = format!("#{:06x}", theme.colors.ui.info),
        "Theme status colors"
    );
}


#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SystemAppearanceAccessibility {
    pub reduce_transparency: bool,
    pub increase_contrast: bool,
    pub reduce_motion: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeMaterialPolicy {
    GlassEligible,
    LegacyVibrancy,
    OpaqueFallback,
}

#[derive(Debug)]
struct AccessibilityCache {
    settings: SystemAppearanceAccessibility,
    last_check: std::time::Instant,
}

impl Default for AccessibilityCache {
    fn default() -> Self {
        Self {
            settings: SystemAppearanceAccessibility::default(),
            last_check: std::time::Instant::now() - ACCESSIBILITY_CACHE_TTL,
        }
    }
}

static ACCESSIBILITY_CACHE: std::sync::LazyLock<std::sync::Mutex<AccessibilityCache>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(AccessibilityCache::default()));

const ACCESSIBILITY_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) fn system_appearance_accessibility() -> SystemAppearanceAccessibility {
    let mut cache = ACCESSIBILITY_CACHE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if cache.last_check.elapsed() < ACCESSIBILITY_CACHE_TTL {
        return cache.settings;
    }

    cache.settings = read_system_appearance_accessibility();
    cache.last_check = std::time::Instant::now();
    cache.settings
}

pub(crate) fn native_material_policy(
    vibrancy_enabled: bool,
    accessibility: SystemAppearanceAccessibility,
) -> NativeMaterialPolicy {
    if accessibility.reduce_transparency {
        NativeMaterialPolicy::OpaqueFallback
    } else if vibrancy_enabled {
        NativeMaterialPolicy::GlassEligible
    } else {
        NativeMaterialPolicy::LegacyVibrancy
    }
}

#[cfg(target_os = "macos")]
fn read_system_appearance_accessibility() -> SystemAppearanceAccessibility {
    SystemAppearanceAccessibility {
        reduce_transparency: read_defaults_bool("com.apple.universalaccess", "reduceTransparency"),
        increase_contrast: read_defaults_bool("com.apple.universalaccess", "increaseContrast"),
        reduce_motion: read_defaults_bool("com.apple.universalaccess", "reduceMotion"),
    }
}

#[cfg(not(target_os = "macos"))]
fn read_system_appearance_accessibility() -> SystemAppearanceAccessibility {
    SystemAppearanceAccessibility::default()
}

#[cfg(target_os = "macos")]
fn read_defaults_bool(domain: &str, key: &str) -> bool {
    let Ok(output) = std::process::Command::new("defaults")
        .args(["read", domain, key])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let value = String::from_utf8_lossy(&output.stdout);
    matches!(value.trim(), "1" | "true" | "TRUE" | "YES" | "yes")
}

#[cfg(test)]
mod accessibility_material_policy_tests {
    use super::{
        native_material_policy, NativeMaterialPolicy, SystemAppearanceAccessibility,
    };

    #[test]
    fn reduced_transparency_forces_opaque_material_fallback() {
        let policy = native_material_policy(
            true,
            SystemAppearanceAccessibility {
                reduce_transparency: true,
                increase_contrast: false,
                reduce_motion: false,
            },
        );
        assert_eq!(policy, NativeMaterialPolicy::OpaqueFallback);
    }

    #[test]
    fn vibrancy_enabled_is_glass_eligible_when_accessibility_allows_it() {
        let policy = native_material_policy(true, SystemAppearanceAccessibility::default());
        assert_eq!(policy, NativeMaterialPolicy::GlassEligible);
    }
}

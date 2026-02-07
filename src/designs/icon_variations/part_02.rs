/// Configuration for rendering an icon
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IconConfig {
    pub size: f32,
    pub color: u32,
    pub opacity: f32,
    pub background: Option<IconBackground>,
}

/// Background style for icons
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum IconBackground {
    Circle { color: u32, radius: f32 },
    RoundedSquare { color: u32, radius: f32 },
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: 0xcccccc,
            opacity: 1.0,
            background: None,
        }
    }
}

impl IconConfig {
    /// Create config from style
    #[allow(dead_code)]
    pub fn from_style(style: IconStyle, base_color: u32, accent_color: u32) -> Self {
        let size = style.size();
        let opacity = style.opacity();

        let (color, background) = match style {
            IconStyle::Accent => (accent_color, None),
            IconStyle::CircleBackground => (
                base_color,
                Some(IconBackground::Circle {
                    color: 0x333333,
                    radius: size + 8.0,
                }),
            ),
            IconStyle::SquareBackground => (
                base_color,
                Some(IconBackground::RoundedSquare {
                    color: 0x333333,
                    radius: 4.0,
                }),
            ),
            _ => (base_color, None),
        };

        Self {
            size,
            color,
            opacity,
            background,
        }
    }
}

// Legacy compatibility - keep old types for existing code
pub use IconName as ScriptIcon;
pub use IconName as ScriptletIcon;
pub use IconName as BuiltInIcon;
pub use IconName as AppIcon;
pub use IconName as WindowIcon;
pub use IconName as FolderIcon;

/// Legacy function for compatibility
pub fn total_icon_count() -> usize {
    IconName::count()
}

/// Convert a string icon name to IconName enum
///
/// Supports various formats:
/// - Exact match: "File", "FileCode", "Terminal"
/// - Lowercase: "file", "terminal", "code"  
/// - With spaces: "file code", "folder open"
/// - Kebab case: "file-code", "folder-open"
/// - Snake case: "file_code", "folder_open"
///
/// Returns None if the name doesn't match any known icon.
pub fn icon_name_from_str(name: &str) -> Option<IconName> {
    // Normalize: lowercase, replace separators with nothing
    let normalized = name.to_lowercase().replace(['-', '_', ' '], "");

    match normalized.as_str() {
        // Files
        "file" => Some(IconName::File),
        "filecode" => Some(IconName::FileCode),
        "folder" => Some(IconName::Folder),
        "folderopen" => Some(IconName::FolderOpen),

        // Actions
        "plus" | "add" => Some(IconName::Plus),
        "trash" | "delete" | "remove" => Some(IconName::Trash),
        "copy" | "clipboard" => Some(IconName::Copy),
        "settings" | "gear" | "cog" | "config" => Some(IconName::Settings),
        "magnifyingglass" | "search" | "find" => Some(IconName::MagnifyingGlass),
        "terminal" | "console" | "shell" | "cli" => Some(IconName::Terminal),
        "code" | "script" | "dev" => Some(IconName::Code),
        "pencil" | "edit" | "rename" => Some(IconName::Pencil),

        // Status
        "check" | "checkmark" | "done" | "complete" => Some(IconName::Check),
        "star" | "favorite" => Some(IconName::Star),
        "starfilled" => Some(IconName::StarFilled),
        "boltfilled" | "bolt" | "lightning" | "flash" => Some(IconName::BoltFilled),
        "boltoutlined" => Some(IconName::BoltOutlined),
        "warning" | "alert" | "caution" => Some(IconName::Warning),

        // Arrows
        "arrowright" | "right" => Some(IconName::ArrowRight),
        "arrowdown" | "down" => Some(IconName::ArrowDown),
        "arrowup" | "up" => Some(IconName::ArrowUp),
        "chevronright" => Some(IconName::ChevronRight),
        "chevrondown" => Some(IconName::ChevronDown),

        // UI
        "close" | "x" | "dismiss" => Some(IconName::Close),

        // Media
        "playfilled" | "play" | "run" | "execute" => Some(IconName::PlayFilled),
        "playoutlined" => Some(IconName::PlayOutlined),

        // UI/Layout
        "sidebar" | "panel" | "layout" => Some(IconName::Sidebar),

        // Communication
        "messagecircle" | "message" | "chat" | "conversation" => Some(IconName::MessageCircle),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_count() {
        assert_eq!(IconName::count(), 30); // 29 + MessageCircle
    }

    #[test]
    fn test_style_count() {
        assert_eq!(IconStyle::count(), 7);
    }

    #[test]
    fn test_all_icons_have_paths() {
        for icon in IconName::all() {
            let path = icon.path();
            assert!(
                path.ends_with(".svg"),
                "Icon {:?} path doesn't end with .svg",
                icon
            );
            assert!(
                path.starts_with("icons/"),
                "Icon {:?} path doesn't start with icons/",
                icon
            );
        }
    }

    #[test]
    fn test_category_coverage() {
        let mut covered = 0;
        for cat in IconCategory::all() {
            covered += cat.icons().len();
        }
        assert_eq!(
            covered,
            IconName::count(),
            "Categories don't cover all icons"
        );
    }

    #[test]
    fn test_icon_name_from_str() {
        // Exact match
        assert_eq!(icon_name_from_str("File"), Some(IconName::File));
        assert_eq!(icon_name_from_str("Terminal"), Some(IconName::Terminal));

        // Lowercase
        assert_eq!(icon_name_from_str("file"), Some(IconName::File));
        assert_eq!(icon_name_from_str("code"), Some(IconName::Code));

        // With spaces
        assert_eq!(icon_name_from_str("file code"), Some(IconName::FileCode));
        assert_eq!(
            icon_name_from_str("folder open"),
            Some(IconName::FolderOpen)
        );

        // Kebab case
        assert_eq!(icon_name_from_str("file-code"), Some(IconName::FileCode));
        assert_eq!(
            icon_name_from_str("bolt-filled"),
            Some(IconName::BoltFilled)
        );

        // Snake case
        assert_eq!(icon_name_from_str("file_code"), Some(IconName::FileCode));
        assert_eq!(
            icon_name_from_str("magnifying_glass"),
            Some(IconName::MagnifyingGlass)
        );

        // Aliases
        assert_eq!(
            icon_name_from_str("search"),
            Some(IconName::MagnifyingGlass)
        );
        assert_eq!(icon_name_from_str("add"), Some(IconName::Plus));
        assert_eq!(icon_name_from_str("delete"), Some(IconName::Trash));
        assert_eq!(icon_name_from_str("gear"), Some(IconName::Settings));
        assert_eq!(icon_name_from_str("lightning"), Some(IconName::BoltFilled));
        assert_eq!(icon_name_from_str("run"), Some(IconName::PlayFilled));

        // Unknown
        assert_eq!(icon_name_from_str("unknown"), None);
        assert_eq!(icon_name_from_str(""), None);
    }

    #[test]
    fn test_style_sizes() {
        assert_eq!(IconStyle::Small.size(), 12.0);
        assert_eq!(IconStyle::Default.size(), 16.0);
        assert_eq!(IconStyle::Large.size(), 24.0);
    }
}

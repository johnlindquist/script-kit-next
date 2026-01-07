//! Script Kit's embedded SVG icons

/// Script Kit's embedded SVG icons
///
/// These are icons bundled with Script Kit that aren't in the Lucide set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedIcon {
    // Files
    File,
    FileCode,
    Folder,
    FolderOpen,
    // Actions
    Plus,
    Trash,
    Copy,
    Settings,
    MagnifyingGlass,
    Terminal,
    Code,
    // Status
    Check,
    Star,
    StarFilled,
    BoltFilled,
    BoltOutlined,
    // Arrows
    ArrowRight,
    ArrowDown,
    ArrowUp,
    ChevronRight,
    ChevronDown,
    // UI
    Close,
    Sidebar,
    // Media
    PlayFilled,
    PlayOutlined,
}

impl EmbeddedIcon {
    /// Get all embedded icons
    pub fn all() -> &'static [EmbeddedIcon] {
        &[
            Self::File,
            Self::FileCode,
            Self::Folder,
            Self::FolderOpen,
            Self::Plus,
            Self::Trash,
            Self::Copy,
            Self::Settings,
            Self::MagnifyingGlass,
            Self::Terminal,
            Self::Code,
            Self::Check,
            Self::Star,
            Self::StarFilled,
            Self::BoltFilled,
            Self::BoltOutlined,
            Self::ArrowRight,
            Self::ArrowDown,
            Self::ArrowUp,
            Self::ChevronRight,
            Self::ChevronDown,
            Self::Close,
            Self::Sidebar,
            Self::PlayFilled,
            Self::PlayOutlined,
        ]
    }

    /// Parse from a string (case-insensitive, supports kebab-case)
    pub fn parse(s: &str) -> Option<Self> {
        let normalized = s.to_lowercase().replace(['-', '_'], "");
        match normalized.as_str() {
            "file" => Some(Self::File),
            "filecode" => Some(Self::FileCode),
            "folder" => Some(Self::Folder),
            "folderopen" => Some(Self::FolderOpen),
            "plus" | "add" => Some(Self::Plus),
            "trash" | "delete" => Some(Self::Trash),
            "copy" => Some(Self::Copy),
            "settings" | "gear" | "cog" => Some(Self::Settings),
            "magnifyingglass" | "search" => Some(Self::MagnifyingGlass),
            "terminal" | "console" => Some(Self::Terminal),
            "code" => Some(Self::Code),
            "check" | "checkmark" => Some(Self::Check),
            "star" => Some(Self::Star),
            "starfilled" => Some(Self::StarFilled),
            "boltfilled" | "bolt" | "lightning" => Some(Self::BoltFilled),
            "boltoutlined" => Some(Self::BoltOutlined),
            "arrowright" => Some(Self::ArrowRight),
            "arrowdown" => Some(Self::ArrowDown),
            "arrowup" => Some(Self::ArrowUp),
            "chevronright" => Some(Self::ChevronRight),
            "chevrondown" => Some(Self::ChevronDown),
            "close" | "x" => Some(Self::Close),
            "sidebar" | "panel" => Some(Self::Sidebar),
            "playfilled" | "play" => Some(Self::PlayFilled),
            "playoutlined" => Some(Self::PlayOutlined),
            _ => None,
        }
    }

    /// Get the asset path for this icon
    pub fn asset_path(&self) -> &'static str {
        match self {
            Self::File => "icons/file.svg",
            Self::FileCode => "icons/file_code.svg",
            Self::Folder => "icons/folder.svg",
            Self::FolderOpen => "icons/folder_open.svg",
            Self::Plus => "icons/plus.svg",
            Self::Trash => "icons/trash.svg",
            Self::Copy => "icons/copy.svg",
            Self::Settings => "icons/settings.svg",
            Self::MagnifyingGlass => "icons/magnifying_glass.svg",
            Self::Terminal => "icons/terminal.svg",
            Self::Code => "icons/code.svg",
            Self::Check => "icons/check.svg",
            Self::Star => "icons/star.svg",
            Self::StarFilled => "icons/star_filled.svg",
            Self::BoltFilled => "icons/bolt_filled.svg",
            Self::BoltOutlined => "icons/bolt_outlined.svg",
            Self::ArrowRight => "icons/arrow_right.svg",
            Self::ArrowDown => "icons/arrow_down.svg",
            Self::ArrowUp => "icons/arrow_up.svg",
            Self::ChevronRight => "icons/chevron_right.svg",
            Self::ChevronDown => "icons/chevron_down.svg",
            Self::Close => "icons/close.svg",
            Self::Sidebar => "icons/sidebar.svg",
            Self::PlayFilled => "icons/play_filled.svg",
            Self::PlayOutlined => "icons/play_outlined.svg",
        }
    }
}

use std::sync::Arc;

/// Categories of icons based on their use case
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconCategory {
    /// File type indicators (file, folder, code)
    Files,
    /// Navigation and UI actions (plus, trash, copy)
    Actions,
    /// Indicators and status (check, star, bolt)
    Status,
    /// Directional arrows and chevrons
    Arrows,
    /// Media controls (play, stop)
    Media,
}

impl IconCategory {
    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Actions => "Actions",
            Self::Status => "Status",
            Self::Arrows => "Arrows",
            Self::Media => "Media",
        }
    }

    /// Get all categories
    pub fn all() -> &'static [IconCategory] {
        &[
            Self::Files,
            Self::Actions,
            Self::Status,
            Self::Arrows,
            Self::Media,
        ]
    }

    /// Get icons belonging to this category
    pub fn icons(&self) -> Vec<IconName> {
        IconName::all()
            .iter()
            .filter(|icon| icon.category() == *self)
            .copied()
            .collect()
    }
}

/// Available SVG icons from assets/icons/
///
/// These map to actual .svg files that can be rendered with GPUI's svg() element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconName {
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
    Pencil,

    // Status
    Check,
    Star,
    StarFilled,
    BoltFilled,
    BoltOutlined,
    EyeOff,
    Warning,

    // Arrows
    ArrowRight,
    ArrowDown,
    ArrowUp,
    ChevronRight,
    ChevronDown,

    // UI
    Close,

    // Media
    PlayFilled,
    PlayOutlined,

    // UI/Layout
    Sidebar,
    Refresh,

    // Communication
    MessageCircle,
}

impl IconName {
    /// Get all available icons
    pub fn all() -> &'static [IconName] {
        &[
            // Files
            Self::File,
            Self::FileCode,
            Self::Folder,
            Self::FolderOpen,
            // Actions
            Self::Plus,
            Self::Trash,
            Self::Copy,
            Self::Settings,
            Self::MagnifyingGlass,
            Self::Terminal,
            Self::Code,
            Self::Pencil,
            // Status
            Self::Check,
            Self::Star,
            Self::StarFilled,
            Self::BoltFilled,
            Self::BoltOutlined,
            Self::EyeOff,
            Self::Warning,
            // Arrows
            Self::ArrowRight,
            Self::ArrowDown,
            Self::ArrowUp,
            Self::ChevronRight,
            Self::ChevronDown,
            // UI
            Self::Close,
            // Media
            Self::PlayFilled,
            Self::PlayOutlined,
            // UI/Layout
            Self::Sidebar,
            Self::Refresh,
            // Communication
            Self::MessageCircle,
        ]
    }

    /// Get total count of icons
    pub fn count() -> usize {
        Self::all().len()
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::FileCode => "File Code",
            Self::Folder => "Folder",
            Self::FolderOpen => "Folder Open",
            Self::Plus => "Plus",
            Self::Trash => "Trash",
            Self::Copy => "Copy",
            Self::Settings => "Settings",
            Self::MagnifyingGlass => "Search",
            Self::Terminal => "Terminal",
            Self::Code => "Code",
            Self::Pencil => "Pencil",
            Self::Check => "Check",
            Self::Star => "Star",
            Self::StarFilled => "Star Filled",
            Self::BoltFilled => "Bolt Filled",
            Self::BoltOutlined => "Bolt Outlined",
            Self::EyeOff => "Eye Off",
            Self::Warning => "Warning",
            Self::ArrowRight => "Arrow Right",
            Self::ArrowDown => "Arrow Down",
            Self::ArrowUp => "Arrow Up",
            Self::ChevronRight => "Chevron Right",
            Self::ChevronDown => "Chevron Down",
            Self::Close => "Close",
            Self::PlayFilled => "Play Filled",
            Self::PlayOutlined => "Play Outlined",
            Self::Sidebar => "Sidebar",
            Self::Refresh => "Refresh",
            Self::MessageCircle => "Message Circle",
        }
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        match self {
            Self::File => "Generic file indicator",
            Self::FileCode => "Source code file",
            Self::Folder => "Closed directory",
            Self::FolderOpen => "Open/expanded directory",
            Self::Plus => "Add/create action",
            Self::Trash => "Delete action",
            Self::Copy => "Copy to clipboard",
            Self::Settings => "Configuration/preferences",
            Self::MagnifyingGlass => "Search/find",
            Self::Terminal => "Terminal/command line",
            Self::Code => "Code/development",
            Self::Pencil => "Edit/rename action",
            Self::Check => "Complete/success",
            Self::Star => "Favorite (outline)",
            Self::StarFilled => "Favorite (filled)",
            Self::BoltFilled => "Quick action (filled)",
            Self::BoltOutlined => "Quick action (outline)",
            Self::EyeOff => "Hidden/secret content",
            Self::Warning => "Warning/alert indicator",
            Self::ArrowRight => "Navigate forward",
            Self::ArrowDown => "Navigate down/expand",
            Self::ArrowUp => "Navigate up/collapse",
            Self::ChevronRight => "Expand right",
            Self::ChevronDown => "Expand down",
            Self::Close => "Close/dismiss",
            Self::PlayFilled => "Run/execute (filled)",
            Self::PlayOutlined => "Run/execute (outline)",
            Self::Sidebar => "Toggle sidebar panel",
            Self::Refresh => "Refresh/regenerate action",
            Self::MessageCircle => "Chat/conversation message",
        }
    }

    /// Get the SVG file path (relative to assets/)
    #[allow(dead_code)]
    pub fn path(&self) -> Arc<str> {
        let file_name = match self {
            Self::File => "file",
            Self::FileCode => "file_code",
            Self::Folder => "folder",
            Self::FolderOpen => "folder_open",
            Self::Plus => "plus",
            Self::Trash => "trash",
            Self::Copy => "copy",
            Self::Settings => "settings",
            Self::MagnifyingGlass => "magnifying_glass",
            Self::Terminal => "terminal",
            Self::Code => "code",
            Self::Pencil => "edit_3",
            Self::Check => "check",
            Self::Star => "star",
            Self::StarFilled => "star_filled",
            Self::BoltFilled => "bolt_filled",
            Self::BoltOutlined => "bolt_outlined",
            Self::EyeOff => "eye_off",
            Self::Warning => "warning",
            Self::ArrowRight => "arrow_right",
            Self::ArrowDown => "arrow_down",
            Self::ArrowUp => "arrow_up",
            Self::ChevronRight => "chevron_right",
            Self::ChevronDown => "chevron_down",
            Self::Close => "close",
            Self::PlayFilled => "play_filled",
            Self::PlayOutlined => "play_outlined",
            Self::Sidebar => "sidebar",
            Self::Refresh => "refresh",
            Self::MessageCircle => "message_circle",
        };
        format!("icons/{}.svg", file_name).into()
    }

    /// Get the full external path for GPUI svg().external_path()
    /// Returns a &'static str for GPUI compatibility
    pub fn external_path(&self) -> &'static str {
        match self {
            Self::File => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/file.svg"),
            Self::FileCode => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/file_code.svg"),
            Self::Folder => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/folder.svg"),
            Self::FolderOpen => {
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/folder_open.svg")
            }
            Self::Plus => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/plus.svg"),
            Self::Trash => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/trash.svg"),
            Self::Copy => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/copy.svg"),
            Self::Settings => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/settings.svg"),
            Self::MagnifyingGlass => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/icons/magnifying_glass.svg"
            ),
            Self::Terminal => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/terminal.svg"),
            Self::Code => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/code.svg"),
            Self::Pencil => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/edit_3.svg"),
            Self::Check => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/check.svg"),
            Self::Star => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/star.svg"),
            Self::StarFilled => {
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/star_filled.svg")
            }
            Self::BoltFilled => {
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/bolt_filled.svg")
            }
            Self::BoltOutlined => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/icons/bolt_outlined.svg"
            ),
            Self::EyeOff => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/eye_off.svg"),
            Self::Warning => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/warning.svg"),
            Self::ArrowRight => {
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/arrow_right.svg")
            }
            Self::ArrowDown => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/arrow_down.svg"),
            Self::ArrowUp => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/arrow_up.svg"),
            Self::ChevronRight => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/icons/chevron_right.svg"
            ),
            Self::ChevronDown => {
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/chevron_down.svg")
            }
            Self::Close => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/close.svg"),
            Self::PlayFilled => {
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/play_filled.svg")
            }
            Self::PlayOutlined => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/icons/play_outlined.svg"
            ),
            Self::Sidebar => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/sidebar.svg"),
            Self::Refresh => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/refresh.svg"),
            Self::MessageCircle => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/icons/message_circle.svg"
            ),
        }
    }

    /// Get the category this icon belongs to
    pub fn category(&self) -> IconCategory {
        match self {
            Self::File | Self::FileCode | Self::Folder | Self::FolderOpen => IconCategory::Files,
            Self::Plus
            | Self::Trash
            | Self::Copy
            | Self::Settings
            | Self::MagnifyingGlass
            | Self::Terminal
            | Self::Code
            | Self::Pencil => IconCategory::Actions,
            Self::Check
            | Self::Star
            | Self::StarFilled
            | Self::BoltFilled
            | Self::BoltOutlined
            | Self::EyeOff
            | Self::Warning => IconCategory::Status,
            Self::ArrowRight
            | Self::ArrowDown
            | Self::ArrowUp
            | Self::ChevronRight
            | Self::ChevronDown => IconCategory::Arrows,
            Self::Close => IconCategory::Actions,
            Self::PlayFilled | Self::PlayOutlined => IconCategory::Media,
            Self::Sidebar | Self::Refresh => IconCategory::Actions,
            Self::MessageCircle => IconCategory::Status,
        }
    }
}

/// Visual rendering styles for icons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum IconStyle {
    /// Standard 16px size, default color
    Default,
    /// Compact 12px size
    Small,
    /// Large 24px size
    Large,
    /// Reduced opacity (muted)
    Muted,
    /// Accent/highlight color
    Accent,
    /// With circular background
    CircleBackground,
    /// With rounded square background
    SquareBackground,
}

#[allow(dead_code)]
impl IconStyle {
    /// Get all styles
    pub fn all() -> &'static [IconStyle] {
        &[
            Self::Default,
            Self::Small,
            Self::Large,
            Self::Muted,
            Self::Accent,
            Self::CircleBackground,
            Self::SquareBackground,
        ]
    }

    /// Get style count
    pub fn count() -> usize {
        Self::all().len()
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Small => "Small",
            Self::Large => "Large",
            Self::Muted => "Muted",
            Self::Accent => "Accent",
            Self::CircleBackground => "Circle BG",
            Self::SquareBackground => "Square BG",
        }
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Default => "Standard 16px icon",
            Self::Small => "Compact 12px icon",
            Self::Large => "Prominent 24px icon",
            Self::Muted => "Reduced opacity",
            Self::Accent => "Highlighted color",
            Self::CircleBackground => "Circular background",
            Self::SquareBackground => "Rounded square background",
        }
    }

    /// Get the icon size in pixels
    pub fn size(&self) -> f32 {
        match self {
            Self::Small => 12.0,
            Self::Large => 24.0,
            _ => 16.0,
        }
    }

    /// Get the opacity (0.0-1.0)
    pub fn opacity(&self) -> f32 {
        match self {
            Self::Muted => 0.5,
            _ => 1.0,
        }
    }
}

/// Get total count of icon gallery items
/// (categories + icons * styles)
#[allow(dead_code)]
pub fn total_gallery_items() -> usize {
    IconCategory::all().len() + (IconName::count() * IconStyle::count())
}


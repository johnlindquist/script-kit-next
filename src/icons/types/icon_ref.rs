//! IconRef - Unified icon reference type

use gpui::SharedString;
use std::fmt;
use std::path::PathBuf;

use super::{lucide_from_str, EmbeddedIcon};

/// Unified icon reference that can represent any icon source
#[derive(Clone)]
pub enum IconRef {
    /// Lucide icon from gpui_component
    Lucide(crate::icons::LucideIcon),
    /// SF Symbol name (macOS 11+ only)
    SFSymbol(SharedString),
    /// Script Kit's embedded icons
    Embedded(EmbeddedIcon),
    /// SVG path relative to assets folder
    AssetSvg(SharedString),
    /// File path (relative to script directory)
    File(PathBuf),
    /// Remote URL (opt-in, gated by settings)
    Url(SharedString),
    /// macOS app bundle icon by bundle ID
    AppBundle(SharedString),
}

impl IconRef {
    /// Parse an icon string in the format "scheme:value"
    pub fn parse(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }

        let (scheme, value) = s.split_once(':')?;
        if value.is_empty() {
            return None;
        }

        match scheme {
            "lucide" => lucide_from_str(value).map(IconRef::Lucide),
            "sf" => Some(IconRef::SFSymbol(SharedString::from(value.to_string()))),
            "embedded" => EmbeddedIcon::parse(value).map(IconRef::Embedded),
            "asset" => Some(IconRef::AssetSvg(SharedString::from(value.to_string()))),
            "file" => Some(IconRef::File(PathBuf::from(value))),
            "url" => Some(IconRef::Url(SharedString::from(value.to_string()))),
            "app" => Some(IconRef::AppBundle(SharedString::from(value.to_string()))),
            _ => None,
        }
    }

    /// Get the fallback icon if this one fails to load
    pub fn fallback(&self) -> Option<IconRef> {
        match self {
            IconRef::SFSymbol(name) => {
                let lucide = sf_symbol_to_lucide(name.as_ref());
                lucide.map(IconRef::Lucide)
            }
            IconRef::AppBundle(_) => Some(IconRef::Lucide(crate::icons::LucideIcon::Frame)),
            IconRef::Url(_) => Some(IconRef::Lucide(crate::icons::LucideIcon::ExternalLink)),
            IconRef::File(_) => Some(IconRef::Lucide(crate::icons::LucideIcon::File)),
            IconRef::AssetSvg(_) => Some(IconRef::Lucide(crate::icons::LucideIcon::File)),
            IconRef::Lucide(_) | IconRef::Embedded(_) => None,
        }
    }

    /// Whether this icon type supports tinting
    pub fn is_tintable(&self) -> bool {
        match self {
            IconRef::Lucide(_) | IconRef::Embedded(_) | IconRef::SFSymbol(_) => true,
            IconRef::AssetSvg(_) | IconRef::File(_) => true,
            IconRef::AppBundle(_) | IconRef::Url(_) => false,
        }
    }
}

/// Map SF Symbol names to Lucide equivalents
fn sf_symbol_to_lucide(name: &str) -> Option<crate::icons::LucideIcon> {
    match name {
        "gear" | "gearshape" | "gearshape.fill" => Some(crate::icons::LucideIcon::Settings),
        "star" | "star.fill" => Some(crate::icons::LucideIcon::Star),
        "trash" | "trash.fill" => Some(crate::icons::LucideIcon::Delete),
        "doc" | "doc.fill" => Some(crate::icons::LucideIcon::File),
        "folder" | "folder.fill" => Some(crate::icons::LucideIcon::Folder),
        "magnifyingglass" => Some(crate::icons::LucideIcon::Search),
        "plus" => Some(crate::icons::LucideIcon::Plus),
        "minus" => Some(crate::icons::LucideIcon::Minus),
        "xmark" => Some(crate::icons::LucideIcon::Close),
        "checkmark" => Some(crate::icons::LucideIcon::Check),
        "arrow.up" => Some(crate::icons::LucideIcon::ArrowUp),
        "arrow.down" => Some(crate::icons::LucideIcon::ArrowDown),
        "arrow.left" => Some(crate::icons::LucideIcon::ArrowLeft),
        "arrow.right" => Some(crate::icons::LucideIcon::ArrowRight),
        "chevron.up" => Some(crate::icons::LucideIcon::ChevronUp),
        "chevron.down" => Some(crate::icons::LucideIcon::ChevronDown),
        "chevron.left" => Some(crate::icons::LucideIcon::ChevronLeft),
        "chevron.right" => Some(crate::icons::LucideIcon::ChevronRight),
        "bell" | "bell.fill" => Some(crate::icons::LucideIcon::Bell),
        "person" | "person.fill" => Some(crate::icons::LucideIcon::User),
        "globe" => Some(crate::icons::LucideIcon::Globe),
        "calendar" => Some(crate::icons::LucideIcon::Calendar),
        "eye" | "eye.fill" => Some(crate::icons::LucideIcon::Eye),
        "eye.slash" | "eye.slash.fill" => Some(crate::icons::LucideIcon::EyeOff),
        "info.circle" | "info.circle.fill" => Some(crate::icons::LucideIcon::Info),
        "line.3.horizontal" => Some(crate::icons::LucideIcon::Menu),
        _ => Some(crate::icons::LucideIcon::File), // Generic fallback
    }
}

impl From<crate::icons::LucideIcon> for IconRef {
    fn from(icon: crate::icons::LucideIcon) -> Self {
        IconRef::Lucide(icon)
    }
}

impl From<EmbeddedIcon> for IconRef {
    fn from(icon: EmbeddedIcon) -> Self {
        IconRef::Embedded(icon)
    }
}

// Manual Debug impl since crate::icons::LucideIcon doesn't derive Debug
impl fmt::Debug for IconRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IconRef::Lucide(_) => write!(f, "IconRef::Lucide(...)"),
            IconRef::SFSymbol(name) => write!(f, "IconRef::SFSymbol({:?})", name),
            IconRef::Embedded(icon) => write!(f, "IconRef::Embedded({:?})", icon),
            IconRef::AssetSvg(path) => write!(f, "IconRef::AssetSvg({:?})", path),
            IconRef::File(path) => write!(f, "IconRef::File({:?})", path),
            IconRef::Url(url) => write!(f, "IconRef::Url({:?})", url),
            IconRef::AppBundle(id) => write!(f, "IconRef::AppBundle({:?})", id),
        }
    }
}

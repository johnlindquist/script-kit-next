//! IconRef - Unified icon reference type

use gpui::SharedString;
use std::fmt;
use std::path::PathBuf;

use super::{lucide_from_str, EmbeddedIcon};

/// Unified icon reference that can represent any icon source
#[derive(Clone)]
pub enum IconRef {
    /// Lucide icon from gpui_component
    Lucide(gpui_component::IconName),
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
            IconRef::AppBundle(_) => Some(IconRef::Lucide(gpui_component::IconName::Frame)),
            IconRef::Url(_) => Some(IconRef::Lucide(gpui_component::IconName::ExternalLink)),
            IconRef::File(_) => Some(IconRef::Lucide(gpui_component::IconName::File)),
            IconRef::AssetSvg(_) => Some(IconRef::Lucide(gpui_component::IconName::File)),
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
fn sf_symbol_to_lucide(name: &str) -> Option<gpui_component::IconName> {
    match name {
        "gear" | "gearshape" | "gearshape.fill" => Some(gpui_component::IconName::Settings),
        "star" | "star.fill" => Some(gpui_component::IconName::Star),
        "trash" | "trash.fill" => Some(gpui_component::IconName::Delete),
        "doc" | "doc.fill" => Some(gpui_component::IconName::File),
        "folder" | "folder.fill" => Some(gpui_component::IconName::Folder),
        "magnifyingglass" => Some(gpui_component::IconName::Search),
        "plus" => Some(gpui_component::IconName::Plus),
        "minus" => Some(gpui_component::IconName::Minus),
        "xmark" => Some(gpui_component::IconName::Close),
        "checkmark" => Some(gpui_component::IconName::Check),
        "arrow.up" => Some(gpui_component::IconName::ArrowUp),
        "arrow.down" => Some(gpui_component::IconName::ArrowDown),
        "arrow.left" => Some(gpui_component::IconName::ArrowLeft),
        "arrow.right" => Some(gpui_component::IconName::ArrowRight),
        "chevron.up" => Some(gpui_component::IconName::ChevronUp),
        "chevron.down" => Some(gpui_component::IconName::ChevronDown),
        "chevron.left" => Some(gpui_component::IconName::ChevronLeft),
        "chevron.right" => Some(gpui_component::IconName::ChevronRight),
        "bell" | "bell.fill" => Some(gpui_component::IconName::Bell),
        "person" | "person.fill" => Some(gpui_component::IconName::User),
        "globe" => Some(gpui_component::IconName::Globe),
        "calendar" => Some(gpui_component::IconName::Calendar),
        "eye" | "eye.fill" => Some(gpui_component::IconName::Eye),
        "eye.slash" | "eye.slash.fill" => Some(gpui_component::IconName::EyeOff),
        "info.circle" | "info.circle.fill" => Some(gpui_component::IconName::Info),
        "line.3.horizontal" => Some(gpui_component::IconName::Menu),
        _ => Some(gpui_component::IconName::File), // Generic fallback
    }
}

impl From<gpui_component::IconName> for IconRef {
    fn from(icon: gpui_component::IconName) -> Self {
        IconRef::Lucide(icon)
    }
}

impl From<EmbeddedIcon> for IconRef {
    fn from(icon: EmbeddedIcon) -> Self {
        IconRef::Embedded(icon)
    }
}

// Manual Debug impl since gpui_component::IconName doesn't derive Debug
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

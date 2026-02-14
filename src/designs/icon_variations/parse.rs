use super::IconName;

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

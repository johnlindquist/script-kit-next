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
    for category in IconCategory::all() {
        covered += category.icons().len();
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

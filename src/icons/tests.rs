//! Tests for the unified icon system
//!
//! Following TDD: these tests define the expected behavior before implementation.

use super::*;

mod icon_ref_parsing {
    use super::*;

    #[test]
    fn parse_lucide_prefixed() {
        let icon = IconRef::parse("lucide:trash");
        assert!(matches!(icon, Some(IconRef::Lucide(_))));
    }

    #[test]
    fn parse_lucide_kebab_case() {
        let icon = IconRef::parse("lucide:arrow-right");
        assert!(matches!(icon, Some(IconRef::Lucide(_))));
    }

    #[test]
    fn parse_sf_symbol() {
        let icon = IconRef::parse("sf:gear");
        assert!(matches!(icon, Some(IconRef::SFSymbol(_))));
    }

    #[test]
    fn parse_sf_symbol_with_dots() {
        let icon = IconRef::parse("sf:gear.badge.checkmark");
        assert!(matches!(icon, Some(IconRef::SFSymbol(_))));
        if let Some(IconRef::SFSymbol(name)) = icon {
            assert_eq!(name.as_ref(), "gear.badge.checkmark");
        }
    }

    #[test]
    fn parse_app_bundle() {
        let icon = IconRef::parse("app:com.apple.finder");
        assert!(matches!(icon, Some(IconRef::AppBundle(_))));
        if let Some(IconRef::AppBundle(bundle_id)) = icon {
            assert_eq!(bundle_id.as_ref(), "com.apple.finder");
        }
    }

    #[test]
    fn parse_embedded() {
        let icon = IconRef::parse("embedded:terminal");
        assert!(matches!(icon, Some(IconRef::Embedded(_))));
    }

    #[test]
    fn parse_file_relative() {
        let icon = IconRef::parse("file:icons/custom.svg");
        assert!(matches!(icon, Some(IconRef::File(_))));
    }

    #[test]
    fn parse_asset_svg() {
        let icon = IconRef::parse("asset:icons/my-icon.svg");
        assert!(matches!(icon, Some(IconRef::AssetSvg(_))));
    }

    #[test]
    fn parse_url_https() {
        let icon = IconRef::parse("url:https://example.com/icon.png");
        assert!(matches!(icon, Some(IconRef::Url(_))));
    }

    #[test]
    fn parse_invalid_scheme() {
        let icon = IconRef::parse("invalid:something");
        assert!(icon.is_none());
    }

    #[test]
    fn parse_no_colon_returns_none() {
        let icon = IconRef::parse("trash");
        // Without a scheme, should try to match as a known alias or return None
        // We might want to support bare names as Lucide aliases later
        assert!(icon.is_none() || matches!(icon, Some(IconRef::Lucide(_))));
    }

    #[test]
    fn parse_empty_returns_none() {
        assert!(IconRef::parse("").is_none());
    }

    #[test]
    fn parse_scheme_only_returns_none() {
        assert!(IconRef::parse("lucide:").is_none());
    }
}

mod icon_color {
    use super::*;

    #[test]
    fn inherit_is_default() {
        let color = IconColor::default();
        assert!(matches!(color, IconColor::Inherit));
    }

    #[test]
    fn token_variants_exist() {
        let _primary = IconColor::Token(ColorToken::Primary);
        let _muted = IconColor::Token(ColorToken::Muted);
        let _accent = IconColor::Token(ColorToken::Accent);
        let _danger = IconColor::Token(ColorToken::Danger);
        let _success = IconColor::Token(ColorToken::Success);
    }

    #[test]
    fn fixed_from_hex() {
        let color = IconColor::from_hex(0xFF0000);
        assert!(matches!(color, IconColor::Fixed(_)));
    }

    #[test]
    fn none_disables_tinting() {
        let color = IconColor::None;
        assert!(matches!(color, IconColor::None));
    }
}

mod icon_size {
    use super::*;

    #[test]
    fn size_to_pixels() {
        assert_eq!(IconSize::XSmall.to_px(), 12.0);
        assert_eq!(IconSize::Small.to_px(), 14.0);
        assert_eq!(IconSize::Medium.to_px(), 16.0);
        assert_eq!(IconSize::Large.to_px(), 20.0);
        assert_eq!(IconSize::XLarge.to_px(), 24.0);
    }

    #[test]
    fn custom_size() {
        let size = IconSize::Custom(32.0);
        assert_eq!(size.to_px(), 32.0);
    }
}

mod icon_style {
    use super::*;

    #[test]
    fn default_style() {
        let style = IconStyle::default();
        assert_eq!(style.size, IconSize::Medium);
        assert!(matches!(style.color, IconColor::Inherit));
        assert!((style.opacity - 1.0).abs() < f32::EPSILON);
        assert!(style.rotation.is_none());
    }

    #[test]
    fn builder_pattern() {
        let style = IconStyle::default()
            .with_size(IconSize::Large)
            .with_color(IconColor::Token(ColorToken::Accent))
            .with_opacity(0.5);

        assert_eq!(style.size, IconSize::Large);
        assert!(matches!(style.color, IconColor::Token(ColorToken::Accent)));
        assert!((style.opacity - 0.5).abs() < f32::EPSILON);
    }
}

mod icon_ref_conversions {
    use super::*;

    #[test]
    fn from_gpui_component_icon_name() {
        let icon: IconRef = gpui_component::IconName::Check.into();
        assert!(matches!(icon, IconRef::Lucide(_)));
    }

    #[test]
    fn from_embedded_icon() {
        let icon: IconRef = EmbeddedIcon::Terminal.into();
        assert!(matches!(icon, IconRef::Embedded(_)));
    }
}

mod embedded_icon {
    use super::*;

    #[test]
    fn embedded_icons_have_paths() {
        for icon in EmbeddedIcon::all() {
            let path = icon.asset_path();
            assert!(
                path.ends_with(".svg"),
                "{:?} path doesn't end with .svg",
                icon
            );
        }
    }

    #[test]
    fn from_string_exact() {
        assert!(matches!(
            EmbeddedIcon::parse("terminal"),
            Some(EmbeddedIcon::Terminal)
        ));
        assert!(matches!(
            EmbeddedIcon::parse("code"),
            Some(EmbeddedIcon::Code)
        ));
    }

    #[test]
    fn from_string_case_insensitive() {
        assert!(matches!(
            EmbeddedIcon::parse("Terminal"),
            Some(EmbeddedIcon::Terminal)
        ));
        assert!(matches!(
            EmbeddedIcon::parse("TERMINAL"),
            Some(EmbeddedIcon::Terminal)
        ));
    }

    #[test]
    fn from_string_kebab_case() {
        assert!(matches!(
            EmbeddedIcon::parse("file-code"),
            Some(EmbeddedIcon::FileCode)
        ));
    }

    #[test]
    fn from_string_unknown() {
        assert!(EmbeddedIcon::parse("unknown-icon").is_none());
    }
}

mod fallback_policy {
    use super::*;

    #[test]
    fn sf_symbol_has_lucide_fallback() {
        let icon = IconRef::SFSymbol("gear".into());
        let fallback = icon.fallback();
        // SF Symbol "gear" should fall back to Lucide Settings
        assert!(matches!(fallback, Some(IconRef::Lucide(_))));
    }

    #[test]
    fn app_bundle_has_fallback() {
        let icon = IconRef::AppBundle("com.nonexistent.app".into());
        let fallback = icon.fallback();
        // Missing app should have a fallback icon
        assert!(fallback.is_some());
    }

    #[test]
    fn lucide_has_no_fallback() {
        let icon = IconRef::Lucide(gpui_component::IconName::Check);
        assert!(icon.fallback().is_none());
    }
}

mod icon_category {
    use super::*;

    #[test]
    fn is_tintable() {
        // Vector icons should be tintable
        assert!(IconRef::Lucide(gpui_component::IconName::Check).is_tintable());
        assert!(IconRef::Embedded(EmbeddedIcon::Terminal).is_tintable());
        assert!(IconRef::SFSymbol("gear".into()).is_tintable());

        // Raster icons should not be tintable by default
        assert!(!IconRef::AppBundle("com.apple.finder".into()).is_tintable());
        assert!(!IconRef::Url("https://example.com/logo.png".into()).is_tintable());
    }
}

mod lucide_name_mapping {
    use super::*;

    #[test]
    fn parse_common_lucide_names() {
        // Test that we can parse common Lucide icon names
        let mappings = [
            ("check", gpui_component::IconName::Check),
            ("arrow-down", gpui_component::IconName::ArrowDown),
            ("arrow-up", gpui_component::IconName::ArrowUp),
            ("arrow-left", gpui_component::IconName::ArrowLeft),
            ("arrow-right", gpui_component::IconName::ArrowRight),
            ("chevron-down", gpui_component::IconName::ChevronDown),
            ("chevron-up", gpui_component::IconName::ChevronUp),
            ("chevron-left", gpui_component::IconName::ChevronLeft),
            ("chevron-right", gpui_component::IconName::ChevronRight),
            ("plus", gpui_component::IconName::Plus),
            ("minus", gpui_component::IconName::Minus),
            ("close", gpui_component::IconName::Close),
            ("copy", gpui_component::IconName::Copy),
            ("delete", gpui_component::IconName::Delete),
            ("search", gpui_component::IconName::Search),
            ("settings", gpui_component::IconName::Settings),
            ("star", gpui_component::IconName::Star),
            ("file", gpui_component::IconName::File),
            ("folder", gpui_component::IconName::Folder),
            ("folder-open", gpui_component::IconName::FolderOpen),
            ("eye", gpui_component::IconName::Eye),
            ("eye-off", gpui_component::IconName::EyeOff),
            ("info", gpui_component::IconName::Info),
            ("menu", gpui_component::IconName::Menu),
            ("user", gpui_component::IconName::User),
            ("globe", gpui_component::IconName::Globe),
            ("calendar", gpui_component::IconName::Calendar),
            ("bell", gpui_component::IconName::Bell),
        ];

        for (name, expected) in mappings {
            let parsed = lucide_from_str(name);
            assert!(parsed.is_some(), "Failed to parse Lucide icon: {}", name);
            // We compare paths since IconName doesn't derive PartialEq
            if let Some(parsed_icon) = parsed {
                use gpui_component::IconNamed;
                assert_eq!(
                    parsed_icon.path(),
                    expected.path(),
                    "Mismatch for Lucide icon: {}",
                    name
                );
            }
        }
    }

    #[test]
    fn parse_lucide_aliases() {
        // Common aliases
        assert!(lucide_from_str("x").is_some()); // alias for close
        assert!(lucide_from_str("trash").is_some()); // alias for delete
    }

    #[test]
    fn parse_unknown_lucide_returns_none() {
        assert!(lucide_from_str("definitely-not-an-icon").is_none());
    }
}

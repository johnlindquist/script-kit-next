// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_action_id_roundtrip() {
        // Every action should roundtrip through id() and from_id()
        for action in TrayMenuAction::all() {
            let id = action.id();
            let recovered = TrayMenuAction::from_id(id);
            assert_eq!(
                recovered,
                Some(*action),
                "Action {:?} with id '{}' should roundtrip",
                action,
                id
            );
        }
    }

    #[test]
    fn test_tray_menu_action_ids_are_unique() {
        let all = TrayMenuAction::all();
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        a.id(),
                        b.id(),
                        "Actions {:?} and {:?} have duplicate IDs",
                        a,
                        b
                    );
                }
            }
        }
    }

    #[test]
    fn test_tray_menu_action_ids_are_prefixed() {
        // All IDs should start with "tray." for namespacing
        for action in TrayMenuAction::all() {
            assert!(
                action.id().starts_with("tray."),
                "Action {:?} ID '{}' should start with 'tray.'",
                action,
                action.id()
            );
        }
    }

    #[test]
    fn test_tray_menu_action_from_id_unknown() {
        assert_eq!(TrayMenuAction::from_id("unknown"), None);
        assert_eq!(TrayMenuAction::from_id(""), None);
        assert_eq!(TrayMenuAction::from_id("tray.nonexistent"), None);
    }

    #[test]
    fn test_tray_menu_action_all_count() {
        // Verify all() returns all variants
        assert_eq!(TrayMenuAction::all().len(), 10);
    }

    // ========================================================================
    // SVG rendering tests
    // ========================================================================

    #[test]
    fn test_render_svg_to_rgba_valid_svg() {
        // A simple valid SVG with visible content
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">
            <rect x="0" y="0" width="16" height="16" fill="white"/>
        </svg>"#;

        let result = render_svg_to_rgba(svg, 16, 16);
        assert!(result.is_ok(), "Valid SVG should render: {:?}", result);

        let rgba = result.unwrap();
        assert_eq!(
            rgba.len(),
            16 * 16 * 4,
            "RGBA data should be width*height*4 bytes"
        );
    }

    #[test]
    fn test_render_svg_to_rgba_invalid_svg() {
        let svg = "not valid svg at all";

        let result = render_svg_to_rgba(svg, 16, 16);
        assert!(result.is_err(), "Invalid SVG should fail");
        assert!(
            result.unwrap_err().to_string().contains("parse"),
            "Error should mention parsing"
        );
    }

    #[test]
    fn test_render_svg_to_rgba_empty_svg() {
        // An SVG with no visible content (all transparent)
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"></svg>"#;

        let result = render_svg_to_rgba(svg, 16, 16);
        assert!(result.is_err(), "Empty SVG should fail validation");
        assert!(
            result.unwrap_err().to_string().contains("transparent"),
            "Error should mention transparency"
        );
    }

    #[test]
    fn test_render_svg_to_rgba_logo_renders() {
        // Test that our actual logo SVG renders successfully
        let result = render_svg_to_rgba(LOGO_SVG, 32, 32);
        assert!(result.is_ok(), "Logo SVG should render: {:?}", result);
    }

    #[test]
    fn test_render_svg_to_rgba_menu_icons_render() {
        // Test all menu icon SVGs render successfully
        let icons = [
            ("ICON_HOME", ICON_HOME),
            ("ICON_EDIT", ICON_EDIT),
            ("ICON_MESSAGE", ICON_MESSAGE),
            ("ICON_GITHUB", ICON_GITHUB),
            ("ICON_BOOK", ICON_BOOK),
            ("ICON_DISCORD", ICON_DISCORD),
            ("ICON_AT_SIGN", ICON_AT_SIGN),
            ("ICON_SETTINGS", ICON_SETTINGS),
            ("ICON_LOG_OUT", ICON_LOG_OUT),
        ];

        for (name, svg) in icons {
            let result = render_svg_to_rgba(svg, MENU_ICON_SIZE, MENU_ICON_SIZE);
            assert!(result.is_ok(), "{} should render: {:?}", name, result);
        }
    }
}

//! System tray icon management for Script Kit
//!
//! Provides a TrayManager that creates a macOS menu bar icon with a context menu.
//! The icon uses the Script Kit logo rendered as a template image for proper
//! light/dark mode adaptation.

// --- merged from part_000.rs ---
use crate::login_item;
use anyhow::{bail, Context, Result};
use tray_icon::{
    menu::{
        CheckMenuItem, ContextMenu, IconMenuItem, MenuEvent, MenuEventReceiver, MenuItem,
        NativeIcon, PredefinedMenuItem, Submenu,
    },
    Icon, TrayIcon, TrayIconBuilder,
};
/// Renders an SVG string to RGBA pixel data with validation.
///
/// # Arguments
/// * `svg` - The SVG string to render
/// * `width` - Target width in pixels
/// * `height` - Target height in pixels
///
/// # Errors
/// Returns an error if:
/// - SVG parsing fails
/// - Pixmap creation fails
/// - The rendered output is completely transparent (likely a rendering failure)
///
/// # Returns
/// RGBA pixel data as a `Vec<u8>` (length = width * height * 4)
fn render_svg_to_rgba(svg: &str, width: u32, height: u32) -> Result<Vec<u8>> {
    // Parse SVG
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opts).context("Failed to parse SVG")?;

    // Create pixmap for rendering
    let mut pixmap = tiny_skia::Pixmap::new(width, height).context("Failed to create pixmap")?;

    // Calculate scale to fit SVG into target dimensions
    let size = tree.size();
    let scale_x = width as f32 / size.width();
    let scale_y = height as f32 / size.height();
    let scale = scale_x.min(scale_y);

    let transform = tiny_skia::Transform::from_scale(scale, scale);

    // Render SVG to pixmap
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Take ownership of pixel data
    let rgba = pixmap.take();

    // Validate: check that at least some pixels have non-zero alpha
    // This catches "failed silently" scenarios where nothing was rendered
    let has_visible_content = rgba.chunks_exact(4).any(|px| px[3] != 0);
    if !has_visible_content {
        bail!(
            "SVG rendered to fully transparent image ({}x{}) - likely a rendering failure",
            width,
            height
        );
    }

    Ok(rgba)
}
/// SVG logo for Script Kit (32x32, monochrome)
/// This will be rendered as a template image on macOS for light/dark mode adaptation
const LOGO_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" viewBox="0 0 32 32">
  <path fill="currentColor" d="M14 25a2 2 0 0 1 2-2h14a2 2 0 1 1 0 4H16a2 2 0 0 1-2-2ZM0 7.381c0-1.796 1.983-2.884 3.498-1.92l13.728 8.736c1.406.895 1.406 2.946 0 3.84L3.498 26.775C1.983 27.738 0 26.649 0 24.854V7.38Z"/>
</svg>"#;
/// Menu item identifiers for matching events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrayMenuAction {
    OpenScriptKit,
    OpenNotes,
    OpenAiChat,
    OpenOnGitHub,
    OpenManual,
    JoinCommunity,
    FollowUs,
    Settings,
    LaunchAtLogin,
    Quit,
}
impl TrayMenuAction {
    /// Returns a stable string ID for this action.
    /// Used with `with_id()` when creating menu items.
    pub const fn id(self) -> &'static str {
        match self {
            Self::OpenScriptKit => "tray.open_script_kit",
            Self::OpenNotes => "tray.open_notes",
            Self::OpenAiChat => "tray.open_ai_chat",
            Self::OpenOnGitHub => "tray.open_github",
            Self::OpenManual => "tray.open_manual",
            Self::JoinCommunity => "tray.join_community",
            Self::FollowUs => "tray.follow_us",
            Self::Settings => "tray.settings",
            Self::LaunchAtLogin => "tray.launch_at_login",
            Self::Quit => "tray.quit",
        }
    }

    /// Looks up a TrayMenuAction from its string ID.
    /// Returns None if the ID is not recognized.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "tray.open_script_kit" => Some(Self::OpenScriptKit),
            "tray.open_notes" => Some(Self::OpenNotes),
            "tray.open_ai_chat" => Some(Self::OpenAiChat),
            "tray.open_github" => Some(Self::OpenOnGitHub),
            "tray.open_manual" => Some(Self::OpenManual),
            "tray.join_community" => Some(Self::JoinCommunity),
            "tray.follow_us" => Some(Self::FollowUs),
            "tray.settings" => Some(Self::Settings),
            "tray.launch_at_login" => Some(Self::LaunchAtLogin),
            "tray.quit" => Some(Self::Quit),
            _ => None,
        }
    }

    /// Returns all TrayMenuAction variants for iteration.
    #[cfg(test)]
    pub const fn all() -> &'static [Self] {
        &[
            Self::OpenScriptKit,
            Self::OpenNotes,
            Self::OpenAiChat,
            Self::OpenOnGitHub,
            Self::OpenManual,
            Self::JoinCommunity,
            Self::FollowUs,
            Self::Settings,
            Self::LaunchAtLogin,
            Self::Quit,
        ]
    }
}
/// Manages the system tray icon and menu
pub struct TrayManager {
    #[allow(dead_code)]
    tray_icon: TrayIcon,
    /// The "Launch at Login" checkbox, stored for updating its checked state
    launch_at_login_item: CheckMenuItem,
}
impl TrayManager {
    /// Creates a new TrayManager with the Script Kit logo and menu.
    ///
    /// # Errors
    /// Returns an error if:
    /// - SVG parsing fails
    /// - PNG rendering fails
    /// - Tray icon creation fails
    pub fn new() -> Result<Self> {
        let icon = Self::create_icon_from_svg()?;
        let (menu, launch_at_login_item) = Self::create_menu()?;

        let mut builder = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip("Script Kit")
            .with_menu(menu);

        // Template mode is macOS-only; adapts icon to light/dark menu bar
        #[cfg(target_os = "macos")]
        {
            builder = builder.with_icon_as_template(true);
        }

        let tray_icon = builder.build().context("Failed to create tray icon")?;

        Ok(Self {
            tray_icon,
            launch_at_login_item,
        })
    }

    /// Converts the embedded SVG logo to a tray icon.
    ///
    /// Uses `render_svg_to_rgba` for validated rendering.
    fn create_icon_from_svg() -> Result<Icon> {
        // Get dimensions from SVG (logo is 32x32)
        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_str(LOGO_SVG, &opts).context("Failed to parse logo SVG")?;
        let size = tree.size();
        let width = size.width() as u32;
        let height = size.height() as u32;

        // Render with validation
        let rgba = render_svg_to_rgba(LOGO_SVG, width, height)
            .context("Failed to render tray logo SVG")?;

        // Create tray icon from RGBA data
        Icon::from_rgba(rgba, width, height).context("Failed to create tray icon from RGBA data")
    }

    /// Creates the tray menu with standard items.
    ///
    /// Uses `Submenu` as the root context menu for cross-platform compatibility.
    /// On macOS, `Menu::append` only allows `Submenu`, but `Submenu::append`
    /// allows any menu item type.
    ///
    /// Menu structure (Raycast-style):
    /// 1. Open Script Kit
    /// 2. ---
    /// 3. Open Notes
    /// 4. Open ACP Chat
    /// 5. ---
    /// 6. Open on GitHub
    /// 7. Manual
    /// 8. Join Community
    /// 9. Follow Us
    /// 10. ---
    /// 11. Settings
    /// 12. ---
    /// 13. Launch at Login (checkmark)
    /// 14. Version X.Y.Z (disabled)
    /// 15. ---
    /// 16. Quit Script Kit
    fn create_menu() -> Result<(Box<dyn ContextMenu>, CheckMenuItem)> {
        // Use Submenu as context menu root - works cross-platform
        // (Menu::append only allows Submenu on macOS, but Submenu::append allows any item)
        let menu = Submenu::with_id("tray.root", "Script Kit", true);

        // Use native macOS template icons so AppKit can tint them correctly
        // for normal, highlighted, and selected menu states.
        let open_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::OpenScriptKit.id(),
            "Open Script Kit",
            true,
            Some(NativeIcon::Home),
            None,
        );
        let open_notes_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::OpenNotes.id(),
            "Open Notes",
            true,
            Some(NativeIcon::FontPanel),
            None,
        );
        let open_ai_chat_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::OpenAiChat.id(),
            "Open ACP Chat",
            true,
            Some(NativeIcon::IChatTheater),
            None,
        );

        // External links
        let open_on_github_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::OpenOnGitHub.id(),
            "Open on GitHub",
            true,
            Some(NativeIcon::FollowLinkFreestanding),
            None,
        );
        let open_manual_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::OpenManual.id(),
            "Manual",
            true,
            Some(NativeIcon::Bookmarks),
            None,
        );
        let join_community_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::JoinCommunity.id(),
            "Join Community",
            true,
            Some(NativeIcon::UserGroup),
            None,
        );
        let follow_us_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::FollowUs.id(),
            "Follow Us",
            true,
            Some(NativeIcon::User),
            None,
        );

        // Settings
        let settings_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::Settings.id(),
            "Settings",
            true,
            Some(NativeIcon::PreferencesGeneral),
            None,
        );

        // Create check menu item for Launch at Login with current state
        let launch_at_login_item = CheckMenuItem::with_id(
            TrayMenuAction::LaunchAtLogin.id(),
            "Launch at Login",
            true, // enabled
            login_item::is_login_item_enabled(),
            None, // no accelerator
        );

        // Version display (disabled, informational only)
        let version_item = MenuItem::new(
            format!("Version {}", env!("CARGO_PKG_VERSION")),
            false,
            None,
        );

        let quit_item = IconMenuItem::with_id_and_native_icon(
            TrayMenuAction::Quit.id(),
            "Quit Script Kit",
            true,
            Some(NativeIcon::StopProgress),
            None,
        );

        // Add items to menu in Raycast-style order
        // Section 1: Main action
        menu.append(&open_item).context("Failed to add Open item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 2: App features
        menu.append(&open_notes_item)
            .context("Failed to add Open Notes item")?;
        menu.append(&open_ai_chat_item)
            .context("Failed to add Open ACP Chat item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 3: External links
        menu.append(&open_on_github_item)
            .context("Failed to add Open on GitHub item")?;
        menu.append(&open_manual_item)
            .context("Failed to add Manual item")?;
        menu.append(&join_community_item)
            .context("Failed to add Join Community item")?;
        menu.append(&follow_us_item)
            .context("Failed to add Follow Us item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 4: Settings
        menu.append(&settings_item)
            .context("Failed to add Settings item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 5: App state
        menu.append(&launch_at_login_item)
            .context("Failed to add Launch at Login item")?;
        menu.append(&version_item)
            .context("Failed to add Version item")?;
        menu.append(&PredefinedMenuItem::separator())
            .context("Failed to add separator")?;

        // Section 6: Quit
        menu.append(&quit_item).context("Failed to add Quit item")?;

        Ok((Box::new(menu), launch_at_login_item))
    }

    /// Returns the menu event receiver for handling menu clicks.
    pub fn menu_event_receiver(&self) -> &MenuEventReceiver {
        MenuEvent::receiver()
    }

    /// Converts a menu event to a `TrayMenuAction` (pure function).
    ///
    /// Returns `Some(action)` if the event matches a known menu item,
    /// or `None` if the event is from an unknown source.
    ///
    /// This is a pure function with no side effects - use `handle_action()`
    /// separately to perform the associated action.
    pub fn action_from_event(event: &MenuEvent) -> Option<TrayMenuAction> {
        TrayMenuAction::from_id(&event.id.0)
    }

    /// Handles any side effects for a menu action.
    ///
    /// Currently only `LaunchAtLogin` has side effects (toggling the OS setting
    /// and updating the checkbox).
    ///
    /// # Errors
    /// Returns an error if the action's side effect fails (e.g., login item toggle).
    pub fn handle_action(&self, action: TrayMenuAction) -> Result<()> {
        if action == TrayMenuAction::LaunchAtLogin {
            // Toggle login item then re-read state from OS (never trust "intended" state)
            login_item::toggle_login_item().context("Failed to toggle login item")?;
            self.refresh_launch_at_login_checkmark();
        }
        // Other actions have no side effects in TrayManager
        Ok(())
    }

    /// Refreshes the "Launch at Login" checkbox to match OS state.
    ///
    /// Call this:
    /// - After toggling the login item
    /// - When the tray menu is about to be shown
    /// - On app startup
    pub fn refresh_launch_at_login_checkmark(&self) {
        let enabled = login_item::is_login_item_enabled();
        self.launch_at_login_item.set_checked(enabled);
    }
}

// --- merged from part_001.rs ---
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
    fn test_create_menu_uses_native_menu_icons() {
        const TRAY_SOURCE: &str = include_str!("mod.rs");
        for native_icon in [
            "NativeIcon::Home",
            "NativeIcon::FontPanel",
            "NativeIcon::IChatTheater",
            "NativeIcon::FollowLinkFreestanding",
            "NativeIcon::Bookmarks",
            "NativeIcon::UserGroup",
            "NativeIcon::User",
            "NativeIcon::PreferencesGeneral",
            "NativeIcon::StopProgress",
        ] {
            assert!(
                TRAY_SOURCE.contains(native_icon),
                "Tray menu should use {native_icon}"
            );
        }
    }
}

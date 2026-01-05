//! System tray icon management for Script Kit
//!
//! Provides a TrayManager that creates a macOS menu bar icon with a context menu.
//! The icon uses the Script Kit logo rendered as a template image for proper
//! light/dark mode adaptation.

use anyhow::{bail, Context, Result};
use tracing::warn;
use tray_icon::{
    menu::{
        CheckMenuItem, ContextMenu, Icon as MenuIcon, IconMenuItem, MenuEvent, MenuEventReceiver,
        MenuItem, PredefinedMenuItem, Submenu,
    },
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::login_item;

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

/// Menu icon size (32x32 for Retina display quality)
const MENU_ICON_SIZE: u32 = 32;

/// SVG logo for Script Kit (32x32, monochrome)
/// This will be rendered as a template image on macOS for light/dark mode adaptation
const LOGO_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" viewBox="0 0 32 32">
  <path fill="currentColor" d="M14 25a2 2 0 0 1 2-2h14a2 2 0 1 1 0 4H16a2 2 0 0 1-2-2ZM0 7.381c0-1.796 1.983-2.884 3.498-1.92l13.728 8.736c1.406.895 1.406 2.946 0 3.84L3.498 26.775C1.983 27.738 0 26.649 0 24.854V7.38Z"/>
</svg>"#;

// Menu item SVG icons (16x16, white outline style for dark menus)
// These are rendered as white icons for macOS dark mode menu bar

const ICON_HOME: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M2.5 6.5L8 2L13.5 6.5V13C13.5 13.2761 13.2761 13.5 13 13.5H10V10C10 8.89543 9.10457 8 8 8C6.89543 8 6 8.89543 6 10V13.5H3C2.72386 13.5 2.5 13.2761 2.5 13V6.5Z" stroke="white" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

const ICON_EDIT: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M11 2.5L13.5 5L6 12.5H3.5V10L11 2.5Z" stroke="white" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

const ICON_MESSAGE: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M14.5 7.5C14.5 10.8137 11.5899 13.5 8 13.5C7.10444 13.5 6.25147 13.3347 5.47266 13.0352L2 14L3.12132 11.0607C2.11929 10.0587 1.5 8.84315 1.5 7.5C1.5 4.18629 4.41015 1.5 8 1.5C11.5899 1.5 14.5 4.18629 14.5 7.5Z" stroke="white" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

const ICON_GITHUB: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path fill-rule="evenodd" clip-rule="evenodd" d="M8 1C4.13 1 1 4.13 1 8C1 11.1 3.01 13.7 5.77 14.67C6.11 14.73 6.24 14.52 6.24 14.34C6.24 14.18 6.23 13.68 6.23 13.15C4.5 13.51 4.08 12.83 3.96 12.47C3.89 12.29 3.57 11.73 3.29 11.57C3.06 11.44 2.72 11.08 3.28 11.07C3.81 11.06 4.19 11.57 4.32 11.78C4.92 12.81 5.87 12.53 6.26 12.35C6.32 11.94 6.49 11.66 6.68 11.5C5.17 11.33 3.59 10.78 3.59 8.15C3.59 7.38 3.89 6.74 4.34 6.25C4.27 6.08 4.03 5.35 4.41 4.38C4.41 4.38 4.97 4.2 6.24 5.1C6.78 4.95 7.35 4.88 7.92 4.87C8.49 4.88 9.06 4.95 9.6 5.1C10.87 4.19 11.43 4.38 11.43 4.38C11.81 5.35 11.57 6.08 11.5 6.25C11.95 6.74 12.25 7.38 12.25 8.15C12.25 10.79 10.66 11.33 9.15 11.5C9.39 11.7 9.6 12.09 9.6 12.69C9.6 13.54 9.59 14.22 9.59 14.35C9.59 14.53 9.72 14.74 10.06 14.68C12.99 13.7 15 11.09 15 8C15 4.13 11.87 1 8 1Z" fill="white"/>
</svg>"#;

const ICON_BOOK: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M2.5 3C2.5 2.44772 2.94772 2 3.5 2H5.5C6.69347 2 7.67833 2.83755 7.93586 3.95336C7.97856 4.13921 8.02144 4.13921 8.06414 3.95336C8.32167 2.83755 9.30653 2 10.5 2H12.5C13.0523 2 13.5 2.44772 13.5 3V11C13.5 11.5523 13.0523 12 12.5 12H10C8.89543 12 8 12.8954 8 14C8 12.8954 7.10457 12 6 12H3.5C2.94772 12 2.5 11.5523 2.5 11V3Z" stroke="white" stroke-width="1.2"/>
<path d="M8 4V14" stroke="white" stroke-width="1.2"/>
</svg>"#;

const ICON_DISCORD: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M13.07 3.26C12.14 2.83 11.14 2.51 10.1 2.33C9.97 2.56 9.82 2.86 9.72 3.1C8.61 2.94 7.51 2.94 6.42 3.1C6.32 2.86 6.17 2.56 6.03 2.33C4.99 2.51 3.99 2.83 3.06 3.27C1.21 6.07 0.72 8.8 0.96 11.49C2.21 12.42 3.42 12.97 4.61 13.33C4.9 12.93 5.16 12.51 5.38 12.06C4.95 11.9 4.54 11.7 4.16 11.46C4.26 11.39 4.36 11.31 4.45 11.23C6.74 12.31 9.29 12.31 11.55 11.23C11.65 11.31 11.75 11.39 11.84 11.46C11.46 11.7 11.05 11.9 10.62 12.06C10.84 12.51 11.1 12.94 11.39 13.33C12.58 12.97 13.79 12.42 15.04 11.49C15.33 8.36 14.48 5.66 13.07 3.26ZM5.52 9.8C4.82 9.8 4.24 9.14 4.24 8.34C4.24 7.54 4.8 6.88 5.52 6.88C6.23 6.88 6.82 7.54 6.8 8.34C6.8 9.14 6.23 9.8 5.52 9.8ZM10.48 9.8C9.78 9.8 9.2 9.14 9.2 8.34C9.2 7.54 9.76 6.88 10.48 6.88C11.19 6.88 11.78 7.54 11.76 8.34C11.76 9.14 11.19 9.8 10.48 9.8Z" fill="white"/>
</svg>"#;

const ICON_AT_SIGN: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<circle cx="8" cy="8" r="2.5" stroke="white" stroke-width="1.2"/>
<path d="M14.5 8C14.5 11.5899 11.5899 14.5 8 14.5C4.41015 14.5 1.5 11.5899 1.5 8C1.5 4.41015 4.41015 1.5 8 1.5C11.5899 1.5 14.5 4.41015 14.5 8ZM14.5 8V9.5C14.5 10.6046 13.6046 11.5 12.5 11.5C11.3954 11.5 10.5 10.6046 10.5 9.5V5.5" stroke="white" stroke-width="1.2" stroke-linecap="round"/>
</svg>"#;

const ICON_SETTINGS: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M6.5 2.5H9.5L10 4L11.5 5L13.5 4.5L15 7L13.5 8.5V9.5L15 11L13.5 13.5L11.5 13L10 14L9.5 15.5H6.5L6 14L4.5 13L2.5 13.5L1 11L2.5 9.5V8.5L1 7L2.5 4.5L4.5 5L6 4L6.5 2.5Z" stroke="white" stroke-width="1.2" stroke-linejoin="round"/>
<circle cx="8" cy="9" r="2" stroke="white" stroke-width="1.2"/>
</svg>"#;

const ICON_LOG_OUT: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M6 14H3C2.44772 14 2 13.5523 2 13V3C2 2.44772 2.44772 2 3 2H6" stroke="white" stroke-width="1.2" stroke-linecap="round"/>
<path d="M11 11L14 8L11 5" stroke="white" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M14 8H6" stroke="white" stroke-width="1.2" stroke-linecap="round"/>
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

    /// Creates a menu icon from an SVG string.
    ///
    /// Returns `None` if rendering fails (logs a warning).
    /// This allows the menu to still function even if an icon fails to render.
    fn create_menu_icon_from_svg(svg: &str) -> Option<MenuIcon> {
        match render_svg_to_rgba(svg, MENU_ICON_SIZE, MENU_ICON_SIZE) {
            Ok(rgba) => MenuIcon::from_rgba(rgba, MENU_ICON_SIZE, MENU_ICON_SIZE).ok(),
            Err(e) => {
                warn!("Failed to render menu icon SVG: {}", e);
                None
            }
        }
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
    /// 4. Open AI Chat
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

        // Create menu icons from embedded SVGs
        let icon_home = Self::create_menu_icon_from_svg(ICON_HOME);
        let icon_edit = Self::create_menu_icon_from_svg(ICON_EDIT);
        let icon_message = Self::create_menu_icon_from_svg(ICON_MESSAGE);
        let icon_github = Self::create_menu_icon_from_svg(ICON_GITHUB);
        let icon_book = Self::create_menu_icon_from_svg(ICON_BOOK);
        let icon_discord = Self::create_menu_icon_from_svg(ICON_DISCORD);
        let icon_at_sign = Self::create_menu_icon_from_svg(ICON_AT_SIGN);
        let icon_settings = Self::create_menu_icon_from_svg(ICON_SETTINGS);
        let icon_log_out = Self::create_menu_icon_from_svg(ICON_LOG_OUT);

        // Create menu items with stable IDs from TrayMenuAction
        let open_item = IconMenuItem::with_id(
            TrayMenuAction::OpenScriptKit.id(),
            "Open Script Kit",
            true,
            icon_home,
            None,
        );
        let open_notes_item = IconMenuItem::with_id(
            TrayMenuAction::OpenNotes.id(),
            "Open Notes",
            true,
            icon_edit,
            None,
        );
        let open_ai_chat_item = IconMenuItem::with_id(
            TrayMenuAction::OpenAiChat.id(),
            "Open AI Chat",
            true,
            icon_message,
            None,
        );

        // External links
        let open_on_github_item = IconMenuItem::with_id(
            TrayMenuAction::OpenOnGitHub.id(),
            "Open on GitHub",
            true,
            icon_github,
            None,
        );
        let open_manual_item = IconMenuItem::with_id(
            TrayMenuAction::OpenManual.id(),
            "Manual",
            true,
            icon_book,
            None,
        );
        let join_community_item = IconMenuItem::with_id(
            TrayMenuAction::JoinCommunity.id(),
            "Join Community",
            true,
            icon_discord,
            None,
        );
        let follow_us_item = IconMenuItem::with_id(
            TrayMenuAction::FollowUs.id(),
            "Follow Us",
            true,
            icon_at_sign,
            None,
        );

        // Settings
        let settings_item = IconMenuItem::with_id(
            TrayMenuAction::Settings.id(),
            "Settings",
            true,
            icon_settings,
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

        let quit_item = IconMenuItem::with_id(
            TrayMenuAction::Quit.id(),
            "Quit Script Kit",
            true,
            icon_log_out,
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
            .context("Failed to add Open AI Chat item")?;
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
        assert_eq!(rgba.len(), 16 * 16 * 4, "RGBA data should be width*height*4 bytes");
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

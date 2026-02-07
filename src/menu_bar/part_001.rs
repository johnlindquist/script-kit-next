/// Parse children of a menu/submenu
fn parse_submenu_children(
    element: AXUIElementRef,
    parent_path: &[usize],
    depth: usize,
) -> Vec<MenuBarItem> {
    let mut children = Vec::new();

    // First, try to get the AXMenu child (the actual menu container)
    if let Ok((menu_children, menu_count)) = get_ax_children(element) {
        for i in 0..menu_count {
            let child = unsafe { CFArrayGetValueAtIndex(menu_children, i) };
            if child.is_null() {
                continue;
            }

            let child_role = get_ax_string_attribute(child as AXUIElementRef, AX_ROLE);

            // If this is an AXMenu, descend into it
            if let Some(ref role) = child_role {
                if role == AX_ROLE_MENU {
                    // Parse the menu's children
                    if let Ok((items, count)) = get_ax_children(child as AXUIElementRef) {
                        for j in 0..count {
                            let item = unsafe { CFArrayGetValueAtIndex(items, j) };
                            if item.is_null() {
                                continue;
                            }

                            let mut item_path = parent_path.to_vec();
                            item_path.push(j as usize);

                            if let Some(menu_item) =
                                parse_menu_item(item as AXUIElementRef, item_path, depth + 1)
                            {
                                children.push(menu_item);
                            }
                        }
                        cf_release(items as CFTypeRef);
                    }
                    break; // Found the menu, no need to continue
                }
            }
        }
        cf_release(menu_children as CFTypeRef);
    }

    children
}
/// Get the menu bar owning application's PID
///
/// Since Script Kit is an accessory app (LSUIElement), it doesn't take menu bar
/// ownership when activated. This function returns the PID of the application
/// that currently owns the system menu bar, which is typically the app that was
/// active before Script Kit was shown.
fn get_menu_bar_owner_pid() -> Result<i32> {
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let menu_owner: *mut Object = msg_send![workspace, menuBarOwningApplication];

        if menu_owner.is_null() {
            bail!("No menu bar owning application found");
        }

        let pid: i32 = msg_send![menu_owner, processIdentifier];

        if pid <= 0 {
            bail!("Invalid process identifier for menu bar owner");
        }

        // Log the menu bar owner for debugging
        let bundle_id: *mut Object = msg_send![menu_owner, bundleIdentifier];
        let bundle_str = if !bundle_id.is_null() {
            let utf8: *const i8 = msg_send![bundle_id, UTF8String];
            if !utf8.is_null() {
                std::ffi::CStr::from_ptr(utf8).to_str().unwrap_or("unknown")
            } else {
                "unknown"
            }
        } else {
            "unknown"
        };
        crate::logging::log(
            "APP",
            &format!("Menu bar owner PID {} = {}", pid, bundle_str),
        );

        Ok(pid)
    }
}
// ============================================================================
// Public API
// ============================================================================

/// Get the menu bar of the frontmost application.
///
/// Returns a vector of `MenuBarItem` representing the top-level menu bar items
/// (e.g., Apple, File, Edit, View, etc.) with their children populated.
///
/// # Returns
/// A vector of menu bar items with hierarchy up to 3 levels deep.
///
/// # Errors
/// Returns error if:
/// - Accessibility permission is not granted
/// - No frontmost application
/// - Failed to read menu bar
///
/// # Example
/// ```ignore
/// let items = get_frontmost_menu_bar()?;
/// for item in items {
///     println!("{}", item.title);
///     for child in &item.children {
///         println!("  - {}", child.title);
///     }
/// }
/// ```
/// Get the menu bar of the current menu bar owning application.
///
/// This queries `menuBarOwningApplication` at call time. For better control,
/// use `get_menu_bar_for_pid()` with a pre-captured PID.
#[instrument]
pub fn get_frontmost_menu_bar() -> Result<Vec<MenuBarItem>> {
    if !has_accessibility_permission() {
        bail!("Accessibility permission required for menu bar access");
    }

    let pid = get_menu_bar_owner_pid()?;
    get_menu_bar_for_pid(pid)
}
/// Get the menu bar for a specific application by PID.
///
/// Use this when you've pre-captured the target PID before any window activation
/// that might change which app owns the menu bar.
#[instrument]
pub fn get_menu_bar_for_pid(pid: i32) -> Result<Vec<MenuBarItem>> {
    if !has_accessibility_permission() {
        bail!("Accessibility permission required for menu bar access");
    }

    debug!(pid, "Getting menu bar for app");

    let ax_app = unsafe { AXUIElementCreateApplication(pid) };
    if ax_app.is_null() {
        bail!("Failed to create AXUIElement for app (pid: {})", pid);
    }

    // Get the menu bar
    let menu_bar =
        get_ax_attribute(ax_app, AX_MENU_BAR).context("Failed to get menu bar from application")?;

    if menu_bar.is_null() {
        cf_release(ax_app);
        bail!("Application has no menu bar");
    }

    // Get menu bar children (top-level menu items like File, Edit, etc.)
    let (children, count) =
        get_ax_children(menu_bar as AXUIElementRef).context("Failed to get menu bar children")?;

    let mut items = Vec::with_capacity(count as usize);

    for i in 0..count {
        let child = unsafe { CFArrayGetValueAtIndex(children, i) };
        if child.is_null() {
            continue;
        }

        let path = vec![i as usize];
        if let Some(item) = parse_menu_item(child as AXUIElementRef, path, 0) {
            items.push(item);
        }
    }

    cf_release(children as CFTypeRef);
    cf_release(menu_bar);
    cf_release(ax_app);

    debug!(item_count = items.len(), "Parsed menu bar items");
    Ok(items)
}
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "../menu_bar_tests.rs"]
mod tests;

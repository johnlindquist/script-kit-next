/// Internal function to navigate the menu path and execute the action
fn navigate_and_execute_menu_path(menu_bar: AXUIElementRef, menu_path: &[String]) -> Result<()> {
    let mut current_menu_container = menu_bar;
    let mut path_so_far: Vec<String> = Vec::new();
    let mut retained_submenus: Vec<OwnedAxElement> = Vec::new();

    for (i, menu_title) in menu_path.iter().enumerate() {
        let is_last = i == menu_path.len() - 1;
        path_so_far.push(menu_title.clone());

        // Get children of current container
        let (children, count) = get_ax_children(current_menu_container).map_err(|e| {
            MenuExecutorError::MenuStructureChanged {
                expected_path: path_so_far.clone(),
                reason: format!("Failed to get children: {}", e),
            }
        })?;

        // Find the menu item by title
        let Some(menu_item) = find_menu_item_by_title(children, count, menu_title) else {
            cf_release(children as CFTypeRef);
            return Err(MenuExecutorError::MenuItemNotFound {
                path: path_so_far,
                searched_in: format!("menu level {}", i),
            }
            .into());
        };

        // Check if enabled (only matters for the final item)
        if is_last {
            let enabled = get_ax_bool_attribute(menu_item.as_ptr(), AX_ENABLED).unwrap_or(true);
            if !enabled {
                cf_release(children as CFTypeRef);
                return Err(MenuExecutorError::MenuItemDisabled { path: path_so_far }.into());
            }

            // Execute the action
            debug!(menu_title, "Pressing final menu item");
            perform_ax_action(menu_item.as_ptr(), AX_PRESS).map_err(|e| {
                MenuExecutorError::ActionFailed(format!(
                    "Failed to press menu item '{}': {}",
                    menu_title, e
                ))
            })?;

            cf_release(children as CFTypeRef);
            return Ok(());
        }

        // Not the last item - need to open the submenu
        debug!(menu_title, "Opening intermediate menu");

        // We need to release children before opening menu (menu opening may change hierarchy)
        cf_release(children as CFTypeRef);

        // Open the menu to get to its children
        let submenu = open_menu_at_element(menu_item.as_ptr()).map_err(|e| {
            MenuExecutorError::MenuStructureChanged {
                expected_path: path_so_far.clone(),
                reason: format!("Failed to open submenu at '{}': {}", menu_title, e),
            }
        })?;

        current_menu_container = submenu.as_ptr();
        retained_submenus.push(submenu);
    }

    Ok(())
}
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "../menu_executor_tests.rs"]
mod tests;

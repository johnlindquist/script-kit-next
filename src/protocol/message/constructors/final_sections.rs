use super::*;

impl Message {
    // ============================================================
    // Constructor methods for test infrastructure
    // ============================================================

    /// Create a simulate click request
    ///
    /// Coordinates are relative to the window's content area.
    pub fn simulate_click(request_id: String, x: f64, y: f64) -> Self {
        Message::SimulateClick {
            request_id,
            x,
            y,
            button: None,
        }
    }

    /// Create a simulate click request with a specific button
    ///
    /// Coordinates are relative to the window's content area.
    /// Button can be "left", "right", or "middle".
    pub fn simulate_click_with_button(request_id: String, x: f64, y: f64, button: String) -> Self {
        Message::SimulateClick {
            request_id,
            x,
            y,
            button: Some(button),
        }
    }

    /// Create a successful simulate click result
    pub fn simulate_click_success(request_id: String) -> Self {
        Message::SimulateClickResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a failed simulate click result
    pub fn simulate_click_error(request_id: String, error: String) -> Self {
        Message::SimulateClickResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }

    // ============================================================
    // Constructor methods for Actions API
    // ============================================================

    /// Create an ActionTriggered message to send to SDK
    ///
    /// This is sent when an action with `has_action=true` is triggered.
    pub fn action_triggered(action: String, value: Option<String>, input: String) -> Self {
        Message::ActionTriggered {
            action,
            value,
            input,
        }
    }

    /// Create a SetActions message
    pub fn set_actions(actions: Vec<ProtocolAction>) -> Self {
        Message::SetActions { actions }
    }

    /// Create a SetInput message
    pub fn set_input(text: String) -> Self {
        Message::SetInput { text }
    }

    // ============================================================
    // Constructor methods for debug grid
    // ============================================================

    /// Create a ShowGrid message with default options
    pub fn show_grid() -> Self {
        Message::ShowGrid {
            options: GridOptions::default(),
        }
    }

    /// Create a ShowGrid message with custom options
    pub fn show_grid_with_options(options: GridOptions) -> Self {
        Message::ShowGrid { options }
    }

    /// Create a HideGrid message
    pub fn hide_grid() -> Self {
        Message::HideGrid
    }

    // ============================================================
    // Constructor methods for menu bar integration
    // ============================================================

    /// Create a GetMenuBar request message
    pub fn get_menu_bar(request_id: String, bundle_id: Option<String>) -> Self {
        Message::GetMenuBar {
            request_id,
            bundle_id,
        }
    }

    /// Create a MenuBarResult response message
    pub fn menu_bar_result(request_id: String, items: Vec<MenuBarItemData>) -> Self {
        Message::MenuBarResult { request_id, items }
    }

    /// Create an ExecuteMenuAction request message
    pub fn execute_menu_action(request_id: String, bundle_id: String, path: Vec<String>) -> Self {
        Message::ExecuteMenuAction {
            request_id,
            bundle_id,
            path,
        }
    }

    /// Create a successful MenuActionResult response message
    pub fn menu_action_success(request_id: String) -> Self {
        Message::MenuActionResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a failed MenuActionResult response message
    pub fn menu_action_error(request_id: String, error: String) -> Self {
        Message::MenuActionResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }
}

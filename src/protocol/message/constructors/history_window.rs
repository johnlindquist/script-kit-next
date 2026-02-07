use super::*;

impl Message {
    // ============================================================
    // Constructor methods for window information
    // ============================================================

    /// Create a get window bounds request
    pub fn get_window_bounds(request_id: String) -> Self {
        Message::GetWindowBounds { request_id }
    }

    /// Create a window bounds response
    pub fn window_bounds(x: f64, y: f64, width: f64, height: f64, request_id: String) -> Self {
        Message::WindowBounds {
            x,
            y,
            width,
            height,
            request_id,
        }
    }

    // ============================================================
    // Constructor methods for clipboard history
    // ============================================================

    /// Create a clipboard history list request
    pub fn clipboard_history_list(request_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::List,
            entry_id: None,
        }
    }

    /// Create a clipboard history pin request
    pub fn clipboard_history_pin(request_id: String, entry_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Pin,
            entry_id: Some(entry_id),
        }
    }

    /// Create a clipboard history unpin request
    pub fn clipboard_history_unpin(request_id: String, entry_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Unpin,
            entry_id: Some(entry_id),
        }
    }

    /// Create a clipboard history remove request
    pub fn clipboard_history_remove(request_id: String, entry_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Remove,
            entry_id: Some(entry_id),
        }
    }

    /// Create a clipboard history clear request
    pub fn clipboard_history_clear(request_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::Clear,
            entry_id: None,
        }
    }

    /// Create a clipboard history trim oversize request
    pub fn clipboard_history_trim_oversize(request_id: String) -> Self {
        Message::ClipboardHistory {
            request_id,
            action: ClipboardHistoryAction::TrimOversize,
            entry_id: None,
        }
    }

    /// Create a clipboard history entry response
    pub fn clipboard_history_entry(
        request_id: String,
        entry_id: String,
        content: String,
        content_type: ClipboardEntryType,
        timestamp: String,
        pinned: bool,
    ) -> Self {
        Message::ClipboardHistoryEntry {
            request_id,
            entry_id,
            content,
            content_type,
            timestamp,
            pinned,
        }
    }

    /// Create a clipboard history list response
    pub fn clipboard_history_list_response(
        request_id: String,
        entries: Vec<ClipboardHistoryEntryData>,
    ) -> Self {
        Message::ClipboardHistoryList {
            request_id,
            entries,
        }
    }

    /// Create a clipboard history result (success)
    pub fn clipboard_history_success(request_id: String) -> Self {
        Message::ClipboardHistoryResult {
            request_id,
            success: true,
            error: None,
        }
    }

    /// Create a clipboard history result (error)
    pub fn clipboard_history_error(request_id: String, error: String) -> Self {
        Message::ClipboardHistoryResult {
            request_id,
            success: false,
            error: Some(error),
        }
    }

    // ============================================================
    // Constructor methods for window management
    // ============================================================

    /// Create a window list request
    pub fn window_list(request_id: String) -> Self {
        Message::WindowList { request_id }
    }

    /// Create a window action request
    pub fn window_action(
        request_id: String,
        action: WindowActionType,
        window_id: Option<u32>,
        bounds: Option<TargetWindowBounds>,
    ) -> Self {
        Message::WindowAction {
            request_id,
            action,
            window_id,
            bounds,
            tile_position: None,
        }
    }

    /// Create a window tile action
    pub fn window_tile_action(
        request_id: String,
        window_id: Option<u32>,
        tile_position: TilePosition,
    ) -> Self {
        Message::WindowAction {
            request_id,
            action: WindowActionType::Tile,
            window_id,
            bounds: None,
            tile_position: Some(tile_position),
        }
    }

    /// Create a window list response
    pub fn window_list_result(request_id: String, windows: Vec<SystemWindowInfo>) -> Self {
        Message::WindowListResult {
            request_id,
            windows,
        }
    }

    /// Create a window action result (success)
    pub fn window_action_success(request_id: String) -> Self {
        Message::WindowActionResult {
            request_id,
            success: true,
            error: None,
            window: None,
        }
    }

    /// Create a window action result (error)
    pub fn window_action_error(request_id: String, error: String) -> Self {
        Message::WindowActionResult {
            request_id,
            success: false,
            error: Some(error),
            window: None,
        }
    }

    /// Create a display list request
    pub fn display_list(request_id: String) -> Self {
        Message::DisplayList { request_id }
    }

    /// Create a display list response
    pub fn display_list_result(request_id: String, displays: Vec<DisplayInfo>) -> Self {
        Message::DisplayListResult {
            request_id,
            displays,
        }
    }

    /// Create a frontmost window request
    pub fn frontmost_window(request_id: String) -> Self {
        Message::FrontmostWindow { request_id }
    }

    /// Create a frontmost window response
    pub fn frontmost_window_result(
        request_id: String,
        window: Option<SystemWindowInfo>,
        error: Option<String>,
    ) -> Self {
        Message::FrontmostWindowResult {
            request_id,
            window,
            error,
        }
    }
}

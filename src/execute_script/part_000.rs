fn protocol_tile_to_window_control(pos: &protocol::TilePosition) -> window_control::TilePosition {
    use protocol::TilePosition as P;
    use window_control::TilePosition as WC;
    match pos {
        P::Left => WC::LeftHalf,
        P::Right => WC::RightHalf,
        P::Top => WC::TopHalf,
        P::Bottom => WC::BottomHalf,
        P::TopLeft => WC::TopLeft,
        P::TopRight => WC::TopRight,
        P::BottomLeft => WC::BottomLeft,
        P::BottomRight => WC::BottomRight,
        P::LeftThird => WC::LeftThird,
        P::CenterThird => WC::CenterThird,
        P::RightThird => WC::RightThird,
        P::TopThird => WC::TopThird,
        P::MiddleThird => WC::MiddleThird,
        P::BottomThird => WC::BottomThird,
        P::FirstTwoThirds => WC::FirstTwoThirds,
        P::LastTwoThirds => WC::LastTwoThirds,
        P::TopTwoThirds => WC::TopTwoThirds,
        P::BottomTwoThirds => WC::BottomTwoThirds,
        P::Center => WC::Center,
        P::AlmostMaximize => WC::AlmostMaximize,
        // Maximize fills the entire visible screen area (fullscreen)
        P::Maximize => WC::Fullscreen,
    }
}
/// Standard macOS menu bar height in points (consistent since macOS 10.0)
/// Note: This is an approximation - the actual height can vary with accessibility settings
const MACOS_MENU_BAR_HEIGHT: i32 = 24;
const CLIPBOARD_HISTORY_PREVIEW_CHAR_LIMIT: usize = 1000;
/// Get information about all displays/monitors
fn get_displays() -> anyhow::Result<Vec<protocol::DisplayInfo>> {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;

        let display_ids = CGDisplay::active_displays()
            .map_err(|_| anyhow::anyhow!("Failed to get active displays"))?;

        let mut displays = Vec::new();
        let main_display_id = CGDisplay::main().id;

        for (index, &display_id) in display_ids.iter().enumerate() {
            let display = CGDisplay::new(display_id);
            let bounds = display.bounds();
            let is_primary = display_id == main_display_id;

            // Estimate visible bounds by subtracting menu bar height from the top
            // This is an approximation - dock and menu bar sizes can vary with settings
            let visible_y = bounds.origin.y as i32 + MACOS_MENU_BAR_HEIGHT;
            let visible_height =
                (bounds.size.height as u32).saturating_sub(MACOS_MENU_BAR_HEIGHT as u32);

            displays.push(protocol::DisplayInfo {
                display_id,
                name: format!("Display {}", index + 1),
                is_primary,
                bounds: protocol::TargetWindowBounds {
                    x: bounds.origin.x as i32,
                    y: bounds.origin.y as i32,
                    width: bounds.size.width as u32,
                    height: bounds.size.height as u32,
                },
                visible_bounds: protocol::TargetWindowBounds {
                    x: bounds.origin.x as i32,
                    y: visible_y,
                    width: bounds.size.width as u32,
                    height: visible_height,
                },
                // Scale factor is typically 2.0 for Retina displays, 1.0 otherwise
                // We can't easily detect this from CGDisplay, so we'll leave it as None
                scale_factor: None,
            });
        }

        Ok(displays)
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Fallback for non-macOS platforms
        Ok(vec![protocol::DisplayInfo {
            display_id: 0,
            name: "Primary Display".to_string(),
            is_primary: true,
            bounds: protocol::TargetWindowBounds {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            visible_bounds: protocol::TargetWindowBounds {
                x: 0,
                y: 24,
                width: 1920,
                height: 1056,
            },
            scale_factor: Some(1.0),
        }])
    }
}
fn format_missing_interactive_session_error(script_name: &str, script_path: &std::path::Path) -> String {
    format!(
        "interactive_session_missing: script='{}' path='{}' state=script_session:none operation=split_interactive_session",
        script_name,
        script_path.display()
    )
}

fn truncate_clipboard_history_preview(content: &str) -> String {
    let Some((cutoff_idx, _)) = content
        .char_indices()
        .nth(CLIPBOARD_HISTORY_PREVIEW_CHAR_LIMIT)
    else {
        return content.to_string();
    };

    format!("{}...", &content[..cutoff_idx])
}

fn take_active_script_session(
    script_session: &SharedSession,
    script_name: &str,
    script_path: &std::path::Path,
) -> Result<executor::ScriptSession, String> {
    script_session
        .lock()
        .take()
        .ok_or_else(|| format_missing_interactive_session_error(script_name, script_path))
}

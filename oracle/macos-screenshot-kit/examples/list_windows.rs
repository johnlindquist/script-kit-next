use macos_screenshot_kit::{ScreenshotClient, WindowListOptions};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    for window in client.windows(WindowListOptions::default())? {
        println!(
            "#{:<10} pid={:<7} layer={:<3} {:>5.0}x{:<5.0} at {:>5.0},{:<5.0} {}",
            window.id,
            window.owner_pid,
            window.layer,
            window.bounds.width,
            window.bounds.height,
            window.bounds.x,
            window.bounds.y,
            window.display_title(),
        );
    }
    Ok(())
}

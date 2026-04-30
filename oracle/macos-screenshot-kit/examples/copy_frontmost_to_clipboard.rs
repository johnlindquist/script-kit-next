use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    client.capture_to_clipboard(CaptureTarget::FrontmostWindow, CaptureOptions::png())?;
    println!("frontmost window copied to clipboard");
    Ok(())
}

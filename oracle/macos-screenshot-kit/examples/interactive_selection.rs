use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(CaptureTarget::InteractiveSelection, CaptureOptions::png())?;
    image.save("interactive-selection.png")?;
    println!("saved interactive-selection.png");
    Ok(())
}

use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(CaptureTarget::InteractiveToolbar, CaptureOptions::png())?;
    image.save("interactive-toolbar.png")?;
    println!("saved interactive-toolbar.png");
    Ok(())
}

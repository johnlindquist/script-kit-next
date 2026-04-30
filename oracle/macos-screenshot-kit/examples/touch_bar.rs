use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(CaptureTarget::TouchBar, CaptureOptions::png())?;
    image.save("touch-bar.png")?;
    println!("saved touch-bar.png");
    Ok(())
}

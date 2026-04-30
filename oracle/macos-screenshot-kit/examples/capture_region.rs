use macos_screenshot_kit::{CaptureOptions, CaptureTarget, Rect, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(
        CaptureTarget::Region(Rect::new(100.0, 100.0, 800.0, 500.0)),
        CaptureOptions::png(),
    )?;
    image.save("region.png")?;
    println!("saved region.png ({}x{})", image.width(), image.height());
    Ok(())
}

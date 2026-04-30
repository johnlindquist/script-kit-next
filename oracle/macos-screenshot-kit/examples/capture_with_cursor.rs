use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};
use std::time::Duration;

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(
        CaptureTarget::AllDisplays,
        CaptureOptions::png()
            .with_cursor(true)
            .with_delay(Duration::from_secs(2)),
    )?;
    image.save("with-cursor.png")?;
    println!("saved with-cursor.png");
    Ok(())
}

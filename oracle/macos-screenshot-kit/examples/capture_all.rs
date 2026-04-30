use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    if !client.permission_status().is_granted() {
        eprintln!("Screen Recording permission is not granted. Opening settings...");
        let _ = client.open_screen_recording_settings();
        let _ = client.request_permission();
        return Ok(());
    }

    let image = client.capture(CaptureTarget::AllDisplays, CaptureOptions::png())?;
    image.save("all-displays.png")?;
    println!("saved all-displays.png ({}x{})", image.width(), image.height());
    Ok(())
}

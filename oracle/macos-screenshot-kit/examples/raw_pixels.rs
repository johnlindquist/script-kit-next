use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(CaptureTarget::MainDisplay, CaptureOptions::png())?;
    let rgba = image.to_rgba8()?;
    println!(
        "{}x{} pixels, {} bytes per row, {} bytes total",
        rgba.width,
        rgba.height,
        rgba.bytes_per_row,
        rgba.data.len(),
    );
    Ok(())
}

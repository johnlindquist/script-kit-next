use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let image = client.capture(
        CaptureTarget::VisibleWindows {
            exclude_window_ids: vec![],
            exclude_pids: vec![std::process::id() as i32],
            include_all_layers: true,
        },
        CaptureOptions::png(),
    )?;
    image.save("without-this-process.png")?;
    println!("saved without-this-process.png");
    Ok(())
}

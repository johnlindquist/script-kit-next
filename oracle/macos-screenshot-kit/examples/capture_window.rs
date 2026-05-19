use macos_screenshot_kit::{CaptureOptions, CaptureTarget, ScreenshotClient};

fn main() -> macos_screenshot_kit::Result<()> {
    let client = ScreenshotClient::new();
    let window = client.frontmost_window()?;
    println!("capturing {}", window.display_title());

    let image = client.capture(
        CaptureTarget::Window(window.id),
        CaptureOptions::png().without_window_shadow(),
    )?;
    image.save("frontmost-window.png")?;
    println!(
        "saved frontmost-window.png ({}x{})",
        image.width(),
        image.height()
    );
    Ok(())
}

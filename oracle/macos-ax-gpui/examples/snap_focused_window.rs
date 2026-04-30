use macos_ax_gpui::{AxClient, AxClientOptions, Rect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let window = client.focused_window()?;

    // Example monitor/work-area rectangle. In a real app, use your display layout.
    let work_area = Rect::new(80.0, 80.0, 1280.0, 800.0);
    let left_half = Rect::new(
        work_area.x(),
        work_area.y(),
        work_area.width() / 2.0,
        work_area.height(),
    );

    window.set_frame(left_half)?;
    window.bring_to_front()?;

    if let Some(info) = client.window_info_for_element(&window)? {
        println!("CGWindowID for screenshot/capture APIs: {}", info.id);
    }

    Ok(())
}

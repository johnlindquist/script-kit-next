use macos_ax_gpui::{AxClient, AxClientOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let app = client.focused_application()?;

    if app.press_menu_item_by_path(&["Window", "Minimize"])? {
        println!("Pressed Window → Minimize in the focused app");
    } else {
        println!("Menu item was not found");
    }

    Ok(())
}

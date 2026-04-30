use macos_ax_gpui::{AxClient, AxClientOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let focused = client.focused_element()?;

    println!("selected text: {:?}", focused.selected_text()?);
    if let Some(range) = focused.selected_text_range()? {
        println!("selected range: {:?}", range);
        println!("selected bounds: {:?}", focused.bounds_for_range(range)?);
    }

    Ok(())
}

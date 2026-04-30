use macos_ax_gpui::{AxClient, AxClientOptions, WindowQuery};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let windows = client.window_list(WindowQuery::default().regular_windows())?;

    for window in windows {
        println!(
            "#{:<8} pid={:<7} layer={:<3} {:<24} {:?} {:?}",
            window.id,
            window.owner_pid,
            window.layer,
            window.owner_name.as_deref().unwrap_or(""),
            window.title,
            window.bounds,
        );
    }

    Ok(())
}

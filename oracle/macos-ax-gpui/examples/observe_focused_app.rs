use macos_ax_gpui::{notification, AxClient, AxClientOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let app = client.focused_application()?;
    let pid = app.pid()?;

    println!("Observing focused app pid {pid}. Press Ctrl-C to stop.");
    let (_observer, rx) = client.observe_application(
        pid,
        [
            notification::FOCUSED_UI_ELEMENT_CHANGED,
            notification::FOCUSED_WINDOW_CHANGED,
            notification::WINDOW_CREATED,
            notification::WINDOW_MOVED,
            notification::WINDOW_RESIZED,
            notification::TITLE_CHANGED,
            notification::VALUE_CHANGED,
        ],
    )?;

    while let Ok(event) = rx.recv() {
        let label = event
            .element
            .as_ref()
            .map(|element| element.label())
            .unwrap_or_default();
        println!("{} pid={} {label:?}", event.notification, event.pid);
    }

    Ok(())
}

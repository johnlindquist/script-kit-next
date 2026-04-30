use macos_ax_gpui::{AxClient, AxClientOptions, ElementSnapshot, TreeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !AxClient::trusted(true)? {
        eprintln!("Accessibility permission is not granted yet. Re-run after approving the prompt.");
        return Ok(());
    }

    let client = AxClient::new(AxClientOptions::default())?;
    let focused = client.focused_element()?;
    let snapshot = focused.snapshot(TreeOptions {
        max_depth: 3,
        max_children_per_node: 32,
        include_all_children: false,
    })?;

    print_snapshot(&snapshot, 0);
    Ok(())
}

fn print_snapshot(snapshot: &ElementSnapshot, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!(
        "{prefix}{} {:?} {:?} {:?}",
        snapshot.role.as_deref().unwrap_or("AXUnknown"),
        snapshot.label(),
        snapshot.frame,
        snapshot.pid
    );

    for child in &snapshot.children {
        print_snapshot(child, indent + 1);
    }
}

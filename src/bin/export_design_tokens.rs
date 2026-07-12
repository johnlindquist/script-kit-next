//! Regenerates design/mockups/generated/tokens.{json,css} from the resolved
//! Rust design contract. Run through the repo cargo wrapper:
//!
//! ```bash
//! ./scripts/agentic/agent-cargo.sh run --bin export_design_tokens -- design/mockups/generated
//! ```

use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("design/mockups/generated"));

    let bundle = script_kit_gpui::design_contract::checked_in_design_bundle()
        .map_err(std::io::Error::other)?;
    let json = serde_json::to_string_pretty(&bundle)? + "\n";
    let css = script_kit_gpui::design_contract::render_css(&bundle);

    fs::create_dir_all(&out)?;
    write_if_changed(out.join("tokens.json"), json.as_bytes())?;
    write_if_changed(out.join("tokens.css"), css.as_bytes())?;
    println!("wrote {} ({})", out.display(), bundle.bundle_hash);
    Ok(())
}

fn write_if_changed(path: PathBuf, bytes: &[u8]) -> std::io::Result<()> {
    if fs::read(&path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    fs::write(path, bytes)
}

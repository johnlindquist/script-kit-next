// Build script for script-kit-gpui
// This ensures the binary is rebuilt when the SDK changes and
// copies the SDK to ~/.kenv/sdk/ for immediate use during development

use std::fs;
use std::path::PathBuf;

fn main() {
    // Ensure rebuild when SDK changes
    println!("cargo:rerun-if-changed=scripts/kit-sdk.ts");

    let sdk_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/kit-sdk.ts");

    if let Some(home) = dirs::home_dir() {
        // Copy SDK to ~/.kenv/sdk/ (the only location now)
        let dest_dir = home.join(".kenv/sdk");
        let sdk_dest = dest_dir.join("kit-sdk.ts");

        // Create directory if needed
        if !dest_dir.exists() {
            if let Err(e) = fs::create_dir_all(&dest_dir) {
                println!(
                    "cargo:warning=Failed to create {}: {}",
                    dest_dir.display(),
                    e
                );
                return;
            }
        }

        // Copy SDK
        match fs::copy(&sdk_src, &sdk_dest) {
            Ok(bytes) => {
                println!(
                    "cargo:warning=Copied SDK to {} ({} bytes)",
                    sdk_dest.display(),
                    bytes
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to copy SDK to {}: {}",
                    sdk_dest.display(),
                    e
                );
            }
        }
    }
}

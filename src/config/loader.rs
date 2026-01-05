//! Configuration loading from file system
//!
//! Handles loading and parsing the config.ts file using bun.

use std::path::PathBuf;
use std::process::Command;
use tracing::{info, instrument, warn};

use super::types::Config;

/// Load configuration from ~/.scriptkit/kit/config.ts
///
/// This function:
/// 1. Checks if the config file exists
/// 2. Transpiles TypeScript to JavaScript using bun build
/// 3. Executes the JS to extract the default export as JSON
/// 4. Parses the JSON into a Config struct
///
/// Returns Config::default() if any step fails.
#[instrument(name = "load_config")]
pub fn load_config() -> Config {
    let config_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());

    // Check if config file exists
    if !config_path.exists() {
        info!(path = %config_path.display(), "Config file not found, using defaults");
        return Config::default();
    }

    // Step 1: Transpile TypeScript to JavaScript using bun build
    let tmp_js_path = "/tmp/kit-config.js";
    let build_output = Command::new("bun")
        .arg("build")
        .arg("--target=bun")
        .arg(config_path.to_string_lossy().to_string())
        .arg(format!("--outfile={}", tmp_js_path))
        .output();

    match build_output {
        Err(e) => {
            warn!(error = %e, "Failed to transpile config with bun, using defaults");
            return Config::default();
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun build failed, using defaults"
                );
                return Config::default();
            }
        }
    }

    // Step 2: Execute the transpiled JS and extract the default export as JSON
    let json_output = Command::new("bun")
        .arg("-e")
        .arg(format!(
            "console.log(JSON.stringify(require('{}').default))",
            tmp_js_path
        ))
        .output();

    match json_output {
        Err(e) => {
            warn!(error = %e, "Failed to execute bun to extract JSON, using defaults");
            Config::default()
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun execution failed, using defaults"
                );
                Config::default()
            } else {
                // Step 3: Parse the JSON output into Config struct
                let json_str = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<Config>(json_str.trim()) {
                    Ok(config) => {
                        info!(path = %config_path.display(), "Successfully loaded config");
                        config
                    }
                    Err(e) => {
                        // Provide helpful error message for common config mistakes
                        let error_hint = if e.to_string().contains("missing field `hotkey`") {
                            "\n\nHint: Your config.ts must include a 'hotkey' field. Example:\n\
                            import type { Config } from \"@scriptkit/sdk\";\n\n\
                            export default {\n\
                              hotkey: {\n\
                                modifiers: [\"meta\"],\n\
                                key: \"Semicolon\"\n\
                              }\n\
                            } satisfies Config;"
                        } else if e.to_string().contains("missing field `modifiers`")
                            || e.to_string().contains("missing field `key`")
                        {
                            "\n\nHint: The 'hotkey' field requires 'modifiers' (array) and 'key' (string). Example:\n\
                            hotkey: {\n\
                              modifiers: [\"meta\"],  // \"meta\", \"ctrl\", \"alt\", \"shift\"\n\
                              key: \"Digit0\"         // e.g., \"Semicolon\", \"KeyK\", \"Digit0\"\n\
                            }"
                        } else {
                            ""
                        };

                        warn!(
                            error = %e,
                            json_output = %json_str,
                            hint = %error_hint,
                            "Failed to parse config JSON, using defaults"
                        );
                        Config::default()
                    }
                }
            }
        }
    }
}

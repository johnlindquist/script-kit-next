use std::path::Path;
use std::process::Command;

/// Execute a script and return its output.
///
/// Attempts to run the script using available CLI tools in this order:
/// 1. `kit run <script_path>` - Script Kit CLI
/// 2. `bun run <script_path>` - For TypeScript files (.ts)
/// 3. `node <script_path>` - For JavaScript files (.js)
///
/// # Arguments
/// * `path` - The path to the script file to execute
///
/// # Returns
/// * `Ok(String)` - The combined stdout and stderr output from the script
/// * `Err(String)` - An error message if all execution methods fail
pub fn execute_script(path: &Path) -> Result<String, String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid path encoding".to_string())?;

    // Try kit CLI first (preferred for script-kit)
    if let Ok(output) = run_command("kit", &["run", path_str]) {
        return Ok(output);
    }

    // Try bun for TypeScript files
    if is_typescript(path) {
        if let Ok(output) = run_command("bun", &["run", path_str]) {
            return Ok(output);
        }
    }

    // Try node for JavaScript files
    if is_javascript(path) {
        if let Ok(output) = run_command("node", &[path_str]) {
            return Ok(output);
        }
    }

    // If we get here, no execution method worked
    Err(format!(
        "Failed to execute script '{}'. Make sure kit, bun, or node is installed.",
        path.display()
    ))
}

/// Run a command and capture its output.
///
/// # Arguments
/// * `cmd` - The command to execute
/// * `args` - Arguments to pass to the command
///
/// # Returns
/// * `Ok(String)` - The combined stdout and stderr
/// * `Err(String)` - An error message if the command fails
fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run '{}': {}", cmd, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Return stdout, or stderr if stdout is empty
        if stdout.is_empty() {
            Ok(stderr.into_owned())
        } else {
            Ok(stdout.into_owned())
        }
    } else {
        // Return error with stderr
        if stderr.is_empty() {
            Err(format!(
                "Command '{}' failed with status: {}",
                cmd, output.status
            ))
        } else {
            Err(stderr.into_owned())
        }
    }
}

/// Check if the path points to a TypeScript file.
fn is_typescript(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "ts")
        .unwrap_or(false)
}

/// Check if the path points to a JavaScript file.
fn is_javascript(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "js")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_typescript() {
        assert!(is_typescript(&PathBuf::from("script.ts")));
        assert!(!is_typescript(&PathBuf::from("script.js")));
        assert!(!is_typescript(&PathBuf::from("script.txt")));
    }

    #[test]
    fn test_is_javascript() {
        assert!(is_javascript(&PathBuf::from("script.js")));
        assert!(!is_javascript(&PathBuf::from("script.ts")));
        assert!(!is_javascript(&PathBuf::from("script.txt")));
    }
}

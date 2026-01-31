//! Scriptlet Execution
//!
//! This module handles execution of scriptlets (small scripts embedded in markdown)
//! with support for various tool types (shell, scripting languages, TypeScript, etc.)

use crate::logging;
use crate::scriptlets::{format_scriptlet, process_conditionals, Scriptlet, SHELL_TOOLS};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};

use super::runner::find_executable;
use super::runner::find_sdk_path;

// Conditionally import selected_text for macOS only
#[cfg(target_os = "macos")]
use crate::selected_text;

/// Options for scriptlet execution
#[derive(Debug, Clone, Default)]
pub struct ScriptletExecOptions {
    /// Current working directory for script execution
    pub cwd: Option<PathBuf>,
    /// Commands to prepend before the main script content
    pub prepend: Option<String>,
    /// Commands to append after the main script content
    pub append: Option<String>,
    /// Named inputs for variable substitution
    pub inputs: HashMap<String, String>,
    /// Positional arguments for variable substitution
    pub positional_args: Vec<String>,
    /// Flags for conditional processing
    pub flags: HashMap<String, bool>,
}

/// Result of a scriptlet execution
#[derive(Debug)]
pub struct ScriptletResult {
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Whether execution was successful
    pub success: bool,
}

/// Get the file extension for a given tool type
#[cfg(test)]
pub fn tool_extension(tool: &str) -> &'static str {
    match tool {
        "ruby" => "rb",
        "python" => "py",
        "perl" => "pl",
        "php" => "php",
        "bash" | "sh" => "sh",
        "zsh" => "zsh",
        "fish" => "fish",
        "node" | "js" => "js",
        "ts" | "kit" | "bun" | "deno" => "ts",
        "applescript" => "applescript",
        "powershell" | "pwsh" => "ps1",
        "cmd" => "bat",
        _ => "sh", // Default to shell script
    }
}

/// Execute a scriptlet based on its tool type
///
/// # Arguments
/// * `scriptlet` - The scriptlet to execute
/// * `options` - Execution options (cwd, prepend, append, inputs, etc.)
///
/// # Returns
/// A `ScriptletResult` with exit code, stdout, stderr, and success flag
///
/// # Tool Types Supported
/// - Shell (bash, zsh, sh, fish): Write temp file, execute via shell
/// - Scripting (python, ruby, perl, php, node): Write temp file with extension, execute
/// - TypeScript (kit, ts, bun, deno): Write temp .ts file, run via bun
/// - transform: Wrap with getSelectedText/setSelectedText (macOS only)
/// - template: Returns content for template prompt invocation
/// - open: Use `open` command (macOS) or `xdg-open` (Linux)
/// - edit: Open in editor
/// - paste: Set selected text via clipboard
/// - type: Simulate keyboard typing
/// - submit: Paste + enter
#[instrument(skip_all, fields(tool = %scriptlet.tool, name = %scriptlet.name))]
pub fn run_scriptlet(
    scriptlet: &Scriptlet,
    options: ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    let start = Instant::now();
    debug!(tool = %scriptlet.tool, name = %scriptlet.name, "Running scriptlet");
    logging::log(
        "EXEC",
        &format!(
            "run_scriptlet: {} (tool: {})",
            scriptlet.name, scriptlet.tool
        ),
    );

    // Process conditionals and variable substitution
    let content = process_conditionals(&scriptlet.scriptlet_content, &options.flags);
    let is_windows = cfg!(target_os = "windows");
    let content = format_scriptlet(
        &content,
        &options.inputs,
        &options.positional_args,
        is_windows,
    );

    // Apply prepend/append
    let content = build_final_content(&content, &options.prepend, &options.append);

    let tool = scriptlet.tool.to_lowercase();

    let result = match tool.as_str() {
        // Shell tools
        t if SHELL_TOOLS.contains(&t) => execute_shell_scriptlet(&tool, &content, &options),

        // Scripting languages
        "python" => execute_with_interpreter("python3", &content, "py", &options),
        "ruby" => execute_with_interpreter("ruby", &content, "rb", &options),
        "perl" => execute_with_interpreter("perl", &content, "pl", &options),
        "php" => execute_with_interpreter("php", &content, "php", &options),
        "node" | "js" => execute_with_interpreter("node", &content, "js", &options),
        "applescript" => execute_applescript(&content, &options),

        // TypeScript tools (run via bun)
        "kit" | "ts" | "bun" | "deno" => execute_typescript(&content, &options),

        // Transform (get selected text, process, set selected text)
        "transform" => execute_transform(&content, &options),

        // Template (return content for prompt invocation)
        "template" => {
            // Template just returns the processed content - the caller handles prompt invocation
            Ok(ScriptletResult {
                exit_code: 0,
                stdout: content,
                stderr: String::new(),
                success: true,
            })
        }

        // Open URL/file
        "open" => execute_open(&content, &options),

        // Edit file in editor
        "edit" => execute_edit(&content, &options),

        // Paste text (set selected text)
        "paste" => execute_paste(&content),

        // Type text via keyboard simulation
        "type" => execute_type(&content),

        // Submit (paste + enter)
        "submit" => execute_submit(&content),

        // Unknown tool - try as shell
        _ => {
            warn!(tool = %tool, "Unknown tool type, falling back to shell");
            execute_shell_scriptlet("sh", &content, &options)
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    match &result {
        Ok(r) => {
            info!(
                duration_ms = duration_ms,
                exit_code = r.exit_code,
                tool = %tool,
                "Scriptlet execution complete"
            );
            logging::log(
                "EXEC",
                &format!(
                    "Scriptlet '{}' completed: exit={}, duration={}ms",
                    scriptlet.name, r.exit_code, duration_ms
                ),
            );
        }
        Err(e) => {
            error!(duration_ms = duration_ms, error = %e, tool = %tool, "Scriptlet execution failed");
            logging::log(
                "EXEC",
                &format!("Scriptlet '{}' failed: {}", scriptlet.name, e),
            );
        }
    }

    result
}

/// Build final content with prepend/append
pub fn build_final_content(
    content: &str,
    prepend: &Option<String>,
    append: &Option<String>,
) -> String {
    let mut result = String::new();

    if let Some(pre) = prepend {
        result.push_str(pre);
        if !pre.ends_with('\n') {
            result.push('\n');
        }
    }

    result.push_str(content);

    if let Some(app) = append {
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(app);
    }

    result
}

/// Execute a shell scriptlet (bash, zsh, sh, fish, etc.)
#[tracing::instrument(skip(content, options), fields(shell = %shell, content_len = content.len()))]
pub fn execute_shell_scriptlet(
    shell: &str,
    content: &str,
    options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    logging::log("EXEC", &format!("Executing shell scriptlet with {}", shell));

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("scriptlet-{}.sh", std::process::id()));

    std::fs::write(&temp_file, content)
        .map_err(|e| format!("Failed to write temp script: {}", e))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_file)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_file, perms)
            .map_err(|e| format!("Failed to set executable permission: {}", e))?;
    }

    // Find the shell executable
    let shell_path = find_executable(shell)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| shell.to_string());

    let mut cmd = Command::new(&shell_path);
    let temp_file_str = temp_file
        .to_str()
        .ok_or_else(|| "Temporary file path contains invalid UTF-8".to_string())?;
    cmd.arg(temp_file_str);

    if let Some(ref cwd) = options.cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd.output().map_err(|e| {
        // Clean up temp file before returning error
        let _ = std::fs::remove_file(&temp_file);

        // Provide helpful error message with installation suggestions
        let suggestions = shell_not_found_suggestions(shell);
        format!(
            "Failed to execute shell script with '{}': {}\n\n{}",
            shell, e, suggestions
        )
    })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Get installation suggestions for a missing shell
pub fn shell_not_found_suggestions(shell: &str) -> String {
    let install_hint = match shell {
        "bash" => {
            if cfg!(target_os = "macos") {
                "bash is usually pre-installed on macOS. Try: brew install bash"
            } else if cfg!(target_os = "linux") {
                "Install with: apt install bash (Debian/Ubuntu) or yum install bash (RHEL/CentOS)"
            } else {
                "bash is typically available through Git for Windows or WSL"
            }
        }
        "zsh" => {
            if cfg!(target_os = "macos") {
                "zsh is the default shell on macOS. If missing, try: brew install zsh"
            } else if cfg!(target_os = "linux") {
                "Install with: apt install zsh (Debian/Ubuntu) or yum install zsh (RHEL/CentOS)"
            } else {
                "zsh can be installed through WSL or Git Bash on Windows"
            }
        }
        "sh" => {
            "sh (POSIX shell) should be available on all Unix systems. Check your PATH."
        }
        "fish" => {
            if cfg!(target_os = "macos") {
                "Install with: brew install fish"
            } else if cfg!(target_os = "linux") {
                "Install with: apt install fish (Debian/Ubuntu) or check https://fishshell.com"
            } else {
                "fish can be installed through WSL on Windows. See https://fishshell.com"
            }
        }
        "cmd" => {
            if cfg!(target_os = "windows") {
                "cmd.exe should be available at C:\\Windows\\System32\\cmd.exe"
            } else {
                "cmd is a Windows-only shell. On Unix, use bash, zsh, or sh instead."
            }
        }
        "powershell" => {
            if cfg!(target_os = "windows") {
                "PowerShell should be pre-installed on Windows. Check System32\\WindowsPowerShell"
            } else {
                "For cross-platform PowerShell, install pwsh: https://aka.ms/install-powershell"
            }
        }
        "pwsh" => {
            "Install PowerShell Core from: https://aka.ms/install-powershell\n\
             macOS: brew install powershell\n\
             Linux: See https://docs.microsoft.com/powershell/scripting/install/installing-powershell-on-linux"
        }
        _ => {
            "Shell not recognized. Make sure it is installed and in your PATH."
        }
    };

    format!(
        "Suggestions:\n\
         - Make sure '{}' is installed and accessible in your PATH\n\
         - {}\n\
         - Alternative shells in SHELL_TOOLS: bash, zsh, sh, fish, cmd, powershell, pwsh",
        shell, install_hint
    )
}

/// Execute a script with a specific interpreter
#[tracing::instrument(skip(content, options), fields(interpreter = %interpreter, extension = %extension, content_len = content.len()))]
pub fn execute_with_interpreter(
    interpreter: &str,
    content: &str,
    extension: &str,
    options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    logging::log(
        "EXEC",
        &format!("Executing with interpreter: {}", interpreter),
    );

    // Create temp file with appropriate extension
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("scriptlet-{}.{}", std::process::id(), extension));

    std::fs::write(&temp_file, content)
        .map_err(|e| format!("Failed to write temp script: {}", e))?;

    // Find the interpreter
    let interp_path = find_executable(interpreter)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| interpreter.to_string());

    let mut cmd = Command::new(&interp_path);
    let temp_file_str = temp_file
        .to_str()
        .ok_or_else(|| "Temporary file path contains invalid UTF-8".to_string())?;
    cmd.arg(temp_file_str);

    if let Some(ref cwd) = options.cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute {} script: {}", interpreter, e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Execute AppleScript
pub fn execute_applescript(
    content: &str,
    options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    logging::log("EXEC", "Executing AppleScript");

    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(content);

    if let Some(ref cwd) = options.cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Execute TypeScript via bun
#[tracing::instrument(skip(content, options), fields(content_len = content.len()))]
pub fn execute_typescript(
    content: &str,
    options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    logging::log("EXEC", "Executing TypeScript via bun");

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("scriptlet-{}.ts", std::process::id()));

    std::fs::write(&temp_file, content)
        .map_err(|e| format!("Failed to write temp script: {}", e))?;

    // Find bun
    let bun_path = find_executable("bun")
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "bun".to_string());

    // Check if we should use SDK preload
    let sdk_path = find_sdk_path();

    let mut cmd = Command::new(&bun_path);
    cmd.arg("run");

    // Add preload if SDK exists
    if let Some(ref sdk) = sdk_path {
        if let Some(sdk_str) = sdk.to_str() {
            cmd.arg("--preload").arg(sdk_str);
        }
    }

    let temp_file_str = temp_file
        .to_str()
        .ok_or_else(|| "Temporary file path contains invalid UTF-8".to_string())?;
    cmd.arg(temp_file_str);

    if let Some(ref cwd) = options.cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute TypeScript: {}", e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Execute transform scriptlet (get selected text, process, set selected text)
#[cfg(target_os = "macos")]
pub fn execute_transform(
    content: &str,
    options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    logging::log("EXEC", "Executing transform scriptlet");

    // Get selected text
    let selected = selected_text::get_selected_text()
        .map_err(|e| format!("Failed to get selected text: {}", e))?;

    // Create script that processes the input
    // Wrap content in a function that receives selectedText and returns transformed text
    let wrapper_script = format!(
        r#"
const selectedText = {};
const transform = (text: string): string => {{
{}
}};
const result = transform(selectedText);
console.log(result);
"#,
        serde_json::to_string(&selected).unwrap_or_else(|_| "\"\"".to_string()),
        content
    );

    // Execute the transform script
    let result = execute_typescript(&wrapper_script, options)?;

    if result.success {
        // Set the transformed text back
        let transformed = result.stdout.trim();
        selected_text::set_selected_text(transformed)
            .map_err(|e| format!("Failed to set selected text: {}", e))?;
    }

    Ok(result)
}

#[cfg(not(target_os = "macos"))]
pub fn execute_transform(
    _content: &str,
    _options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    Err("Transform scriptlets are only supported on macOS".to_string())
}

/// Execute open command (open URL or file)
pub fn execute_open(
    content: &str,
    _options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    logging::log("EXEC", &format!("Opening: {}", content.trim()));

    let target = content.trim();

    #[cfg(target_os = "macos")]
    let cmd_name = "open";
    #[cfg(target_os = "linux")]
    let cmd_name = "xdg-open";
    #[cfg(target_os = "windows")]
    let cmd_name = "start";

    let output = Command::new(cmd_name)
        .arg(target)
        .output()
        .map_err(|e| format!("Failed to open '{}': {}", target, e))?;

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Execute edit command (open file in editor)
pub fn execute_edit(
    content: &str,
    _options: &ScriptletExecOptions,
) -> Result<ScriptletResult, String> {
    logging::log("EXEC", &format!("Editing: {}", content.trim()));

    let file_path = content.trim();

    // Get editor from environment or default
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "code".to_string());

    // Find the editor executable
    let editor_path = find_executable(&editor)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(editor);

    let output = Command::new(&editor_path)
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor_path, e))?;

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Execute paste command (set selected text via clipboard)
#[cfg(target_os = "macos")]
pub fn execute_paste(content: &str) -> Result<ScriptletResult, String> {
    logging::log("EXEC", "Executing paste scriptlet");

    let text = content.trim();

    selected_text::set_selected_text(text).map_err(|e| format!("Failed to paste text: {}", e))?;

    Ok(ScriptletResult {
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
        success: true,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn execute_paste(_content: &str) -> Result<ScriptletResult, String> {
    Err("Paste scriptlets are only supported on macOS".to_string())
}

/// Execute type command (simulate keyboard typing)
#[cfg(target_os = "macos")]
pub fn execute_type(content: &str) -> Result<ScriptletResult, String> {
    logging::log("EXEC", "Executing type scriptlet");

    let text = content.trim();

    // Use AppleScript to simulate typing
    let script = format!(
        r#"tell application "System Events" to keystroke "{}""#,
        text.replace('\\', "\\\\").replace('"', "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to type text: {}", e))?;

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

#[cfg(not(target_os = "macos"))]
pub fn execute_type(_content: &str) -> Result<ScriptletResult, String> {
    Err("Type scriptlets are only supported on macOS".to_string())
}

/// Execute submit command (paste + enter)
#[cfg(target_os = "macos")]
pub fn execute_submit(content: &str) -> Result<ScriptletResult, String> {
    logging::log("EXEC", "Executing submit scriptlet");

    // First paste the text
    let paste_result = execute_paste(content)?;
    if !paste_result.success {
        return Ok(paste_result);
    }

    // Small delay to let paste complete
    std::thread::sleep(Duration::from_millis(50));

    // Then press Enter using AppleScript
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to key code 36"#) // 36 is Return key
        .output()
        .map_err(|e| format!("Failed to press Enter: {}", e))?;

    Ok(ScriptletResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

#[cfg(not(target_os = "macos"))]
pub fn execute_submit(_content: &str) -> Result<ScriptletResult, String> {
    Err("Submit scriptlets are only supported on macOS".to_string())
}

/// Process a single if block, returning (result, bytes_consumed)
fn process_if_block(
    content: &str,
    flag_name: &str,
    flags: &HashMap<String, bool>,
) -> (String, usize) {
    let flag_value = flags.get(flag_name).copied().unwrap_or(false);

    let mut depth = 1;
    let mut if_content = String::new();
    let mut else_content = String::new();
    let mut else_if_chains: Vec<(String, String)> = Vec::new(); // (flag, content)
    let mut in_else = false;
    let mut current_else_if_flag: Option<String> = None;
    let mut consumed = 0;

    let mut chars = content.chars().peekable();
    let mut pos = 0;

    while let Some(c) = chars.next() {
        pos += c.len_utf8();

        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            pos += 1;

            // Read what's inside
            let mut inner = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    break;
                }
                inner.push(ch);
                chars.next();
                pos += ch.len_utf8();
            }

            // Skip closing }}
            if chars.peek() == Some(&'}') {
                chars.next();
                pos += 1;
                if chars.peek() == Some(&'}') {
                    chars.next();
                    pos += 1;
                }
            }

            let inner_trimmed = inner.trim();

            if inner_trimmed.starts_with("#if ") {
                depth += 1;
                // Add to current content - inner already contains the #
                let tag = format!("{{{{{}}}}}", inner_trimmed);
                if in_else {
                    if current_else_if_flag.is_some() {
                        if let Some((_, chain_content)) = else_if_chains.last_mut() {
                            chain_content.push_str(&tag);
                        } else {
                            else_content.push_str(&tag);
                        }
                    } else {
                        else_content.push_str(&tag);
                    }
                } else {
                    if_content.push_str(&tag);
                }
            } else if inner_trimmed == "/if" {
                depth -= 1;
                if depth == 0 {
                    consumed = pos;
                    break;
                } else {
                    let tag = "{{/if}}";
                    if in_else {
                        if current_else_if_flag.is_some() {
                            if let Some((_, chain_content)) = else_if_chains.last_mut() {
                                chain_content.push_str(tag);
                            } else {
                                else_content.push_str(tag);
                            }
                        } else {
                            else_content.push_str(tag);
                        }
                    } else {
                        if_content.push_str(tag);
                    }
                }
            } else if inner_trimmed == "else" && depth == 1 {
                in_else = true;
                current_else_if_flag = None;
            } else if inner_trimmed.starts_with("else if ") && depth == 1 {
                let Some(else_if_flag) = inner_trimmed
                    .strip_prefix("else if ")
                    .map(str::trim)
                    .map(str::to_string)
                else {
                    continue;
                };
                in_else = true;
                current_else_if_flag = Some(else_if_flag.clone());
                else_if_chains.push((else_if_flag, String::new()));
            } else {
                // Some other tag, add to current content
                let tag = format!("{{{{{}}}}}", inner);
                if in_else {
                    if current_else_if_flag.is_some() {
                        if let Some((_, chain_content)) = else_if_chains.last_mut() {
                            chain_content.push_str(&tag);
                        } else {
                            else_content.push_str(&tag);
                        }
                    } else {
                        else_content.push_str(&tag);
                    }
                } else {
                    if_content.push_str(&tag);
                }
            }
        } else if in_else {
            if current_else_if_flag.is_some() {
                if let Some((_, chain_content)) = else_if_chains.last_mut() {
                    chain_content.push(c);
                } else {
                    else_content.push(c);
                }
            } else {
                else_content.push(c);
            }
        } else {
            if_content.push(c);
        }
    }

    // Determine which content to use
    let result = if flag_value {
        // Process nested conditionals in if_content
        process_conditionals(&if_content, flags)
    } else {
        // Check else-if chains
        let mut found = false;
        let mut selected_content = String::new();

        for (chain_flag, chain_content) in &else_if_chains {
            if flags.get(chain_flag).copied().unwrap_or(false) {
                selected_content = process_conditionals(chain_content, flags);
                found = true;
                break;
            }
        }

        if !found {
            // Use else content
            process_conditionals(&else_content, flags)
        } else {
            selected_content
        }
    };

    (result, consumed)
}
// ============================================================================
// Interpreter Tool Constants and Error Helpers
// ============================================================================

/// Interpreter tools that require an external interpreter to execute
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub const INTERPRETER_TOOLS: &[&str] = &["python", "ruby", "perl", "php", "node"];
/// Get the interpreter command for a given tool
///
/// # Arguments
/// * `tool` - The tool name (e.g., "python", "ruby")
///
/// # Returns
/// The interpreter command to use (e.g., "python3" for "python")
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn get_interpreter_command(tool: &str) -> String {
    match tool {
        "python" => "python3".to_string(),
        "ruby" => "ruby".to_string(),
        "perl" => "perl".to_string(),
        "php" => "php".to_string(),
        "node" => "node".to_string(),
        _ => tool.to_string(),
    }
}
/// Get platform-specific installation instructions for an interpreter
///
/// # Arguments
/// * `interpreter` - The interpreter name (e.g., "python3", "ruby")
///
/// # Returns
/// A user-friendly error message with installation instructions
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn interpreter_not_found_message(interpreter: &str) -> String {
    let tool_name = match interpreter {
        "python3" | "python" => "Python",
        "ruby" => "Ruby",
        "perl" => "Perl",
        "php" => "PHP",
        "node" | "nodejs" => "Node.js",
        _ => interpreter,
    };

    let install_instructions = get_platform_install_instructions(interpreter);

    format!(
        "{} interpreter not found.\n\n{}\n\nAfter installation, restart Script Kit.",
        tool_name, install_instructions
    )
}
/// Get platform-specific installation instructions
///
/// # Arguments
/// * `interpreter` - The interpreter name
///
/// # Returns
/// Platform-specific installation command suggestions
#[allow(dead_code)] // Used by interpreter_not_found_message
fn get_platform_install_instructions(interpreter: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        get_macos_install_instructions(interpreter)
    }
    #[cfg(target_os = "linux")]
    {
        get_linux_install_instructions(interpreter)
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_install_instructions(interpreter)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        format!(
            "Please install {} using your system's package manager.",
            interpreter
        )
    }
}
/// Get macOS installation instructions (Homebrew)
#[cfg(target_os = "macos")]
#[allow(dead_code)] // Used by get_platform_install_instructions
fn get_macos_install_instructions(interpreter: &str) -> String {
    let brew_package = match interpreter {
        "python3" | "python" => "python",
        "ruby" => "ruby",
        "perl" => "perl",
        "php" => "php",
        "node" | "nodejs" => "node",
        _ => interpreter,
    };

    format!(
        "Install using Homebrew:\n  brew install {}\n\nOr download from the official website.",
        brew_package
    )
}
/// Get Linux installation instructions (apt/dnf)
#[cfg(target_os = "linux")]
fn get_linux_install_instructions(interpreter: &str) -> String {
    let (apt_package, dnf_package) = match interpreter {
        "python3" | "python" => ("python3", "python3"),
        "ruby" => ("ruby", "ruby"),
        "perl" => ("perl", "perl"),
        "php" => ("php", "php-cli"),
        "node" | "nodejs" => ("nodejs", "nodejs"),
        _ => (interpreter, interpreter),
    };

    format!(
        "Install using your package manager:\n\n  Debian/Ubuntu:\n    sudo apt install {}\n\n  Fedora/RHEL:\n    sudo dnf install {}",
        apt_package, dnf_package
    )
}
/// Get Windows installation instructions
#[cfg(target_os = "windows")]
fn get_windows_install_instructions(interpreter: &str) -> String {
    let (choco_package, download_url) = match interpreter {
        "python3" | "python" => ("python", "https://www.python.org/downloads/"),
        "ruby" => ("ruby", "https://rubyinstaller.org/"),
        "perl" => ("strawberryperl", "https://strawberryperl.com/"),
        "php" => ("php", "https://windows.php.net/download/"),
        "node" | "nodejs" => ("nodejs", "https://nodejs.org/"),
        _ => (interpreter, ""),
    };

    if download_url.is_empty() {
        format!(
            "Install using Chocolatey:\n  choco install {}",
            choco_package
        )
    } else {
        format!(
            "Install using Chocolatey:\n  choco install {}\n\nOr download from:\n  {}",
            choco_package, download_url
        )
    }
}
/// Check if a tool is an interpreter tool
///
/// # Arguments
/// * `tool` - The tool name to check
///
/// # Returns
/// `true` if the tool requires an external interpreter
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn is_interpreter_tool(tool: &str) -> bool {
    INTERPRETER_TOOLS.contains(&tool)
}
/// Get the file extension for a given interpreter tool
///
/// # Arguments
/// * `tool` - The tool name
///
/// # Returns
/// The appropriate file extension for scripts of that type
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn get_interpreter_extension(tool: &str) -> &'static str {
    match tool {
        "python" => "py",
        "ruby" => "rb",
        "perl" => "pl",
        "php" => "php",
        "node" => "js",
        _ => "txt",
    }
}
/// Validate that a tool name is a known interpreter
///
/// # Arguments
/// * `tool` - The tool name to validate
///
/// # Returns
/// `Ok(())` if valid, `Err` with descriptive message if not
#[allow(dead_code)] // Infrastructure ready for use in executor.rs
pub fn validate_interpreter_tool(tool: &str) -> Result<(), String> {
    if is_interpreter_tool(tool) {
        Ok(())
    } else if VALID_TOOLS.contains(&tool) {
        Err(format!(
            "'{}' is a valid tool but not an interpreter tool",
            tool
        ))
    } else {
        Err(format!("'{}' is not a recognized tool type", tool))
    }
}
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "../scriptlet_tests.rs"]
mod tests;

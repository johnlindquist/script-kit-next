/// Execute a fallback action based on the fallback ID and input text.
///
/// This handles the various fallback action types:
/// - run-in-terminal: Open terminal with command
/// - add-to-notes: Open Notes window with quick capture
/// - copy-to-clipboard: Copy text to clipboard
/// - search-google/search-duckduckgo: Open browser with search URL
/// - open-url: Open the input as a URL
/// - calculate: Evaluate math expression (basic)
/// - open-file: Open file/folder with default app
fn execute_fallback_action(
    app: &mut ScriptListApp,
    fallback_id: &str,
    input: &str,
    _window: &mut Window,
    cx: &mut Context<ScriptListApp>,
) {
    use fallbacks::builtins::{get_builtin_fallbacks, FallbackResult};

    logging::log(
        "FALLBACK",
        &format!("Executing fallback '{}' with input: {}", fallback_id, input),
    );

    // Find the fallback by ID
    let fallbacks = get_builtin_fallbacks();
    let fallback = fallbacks.iter().find(|f| f.id == fallback_id);

    let Some(fallback) = fallback else {
        logging::log("FALLBACK", &format!("Unknown fallback ID: {}", fallback_id));
        return;
    };

    // Execute the fallback and get the result
    match fallback.execute(input) {
        Ok(result) => {
            match result {
                FallbackResult::RunTerminal { command } => {
                    logging::log("FALLBACK", &format!("RunTerminal: {}", command));
                    // Open Terminal.app with the command
                    #[cfg(target_os = "macos")]
                    {
                        // Use AppleScript to open Terminal and run the command
                        let script = format!(
                            r#"tell application "Terminal"
                                activate
                                do script "{}"
                            end tell"#,
                            command.replace("\"", "\\\"").replace("\\", "\\\\")
                        );
                        match std::process::Command::new("osascript")
                            .arg("-e")
                            .arg(&script)
                            .spawn()
                        {
                            Ok(_) => logging::log("FALLBACK", "Opened Terminal with command"),
                            Err(e) => {
                                logging::log("FALLBACK", &format!("Failed to open Terminal: {}", e))
                            }
                        }
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        logging::log("FALLBACK", "RunTerminal not implemented for this platform");
                    }
                }

                FallbackResult::AddNote { content } => {
                    logging::log("FALLBACK", &format!("AddNote: {}", content));
                    // First copy content to clipboard so user can paste it
                    let item = gpui::ClipboardItem::new_string(content.clone());
                    cx.write_to_clipboard(item);
                    // Then open Notes window - user can paste with Cmd+V
                    if let Err(e) = notes::open_notes_window(cx) {
                        logging::log("FALLBACK", &format!("Failed to open Notes: {}", e));
                    } else {
                        hud_manager::show_hud(
                            "Text copied - paste into Notes".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                }

                FallbackResult::Copy { text } => {
                    logging::log("FALLBACK", &format!("Copy: {} chars", text.len()));
                    // Copy to clipboard using GPUI
                    let item = gpui::ClipboardItem::new_string(text);
                    cx.write_to_clipboard(item);
                    logging::log("FALLBACK", "Text copied to clipboard");
                }

                FallbackResult::OpenUrl { url } => {
                    logging::log("FALLBACK", &format!("OpenUrl: {}", url));
                    // Open URL in default browser
                    if let Err(e) = open::that(&url) {
                        logging::log("FALLBACK", &format!("Failed to open URL: {}", e));
                    } else {
                        logging::log("FALLBACK", "URL opened in browser");
                    }
                }

                FallbackResult::Calculate { expression } => {
                    logging::log("FALLBACK", &format!("Calculate: {}", expression));
                    // Basic math evaluation using meval crate
                    match meval::eval_str(&expression) {
                        Ok(result) => {
                            let result_str = result.to_string();
                            logging::log("FALLBACK", &format!("Result: {}", result_str));
                            // Copy result to clipboard
                            let item = gpui::ClipboardItem::new_string(result_str.clone());
                            cx.write_to_clipboard(item);
                            // Show HUD with result
                            hud_manager::show_hud(format!("= {}", result_str), Some(2000), cx);
                        }
                        Err(e) => {
                            logging::log("FALLBACK", &format!("Calculation error: {}", e));
                            hud_manager::show_hud(format!("Error: {}", e), Some(3000), cx);
                        }
                    }
                }

                FallbackResult::OpenFile { path } => {
                    logging::log("FALLBACK", &format!("OpenFile: {}", path));
                    // Expand ~ to home directory
                    let expanded = shellexpand::tilde(&path).to_string();
                    // Open with default application
                    if let Err(e) = open::that(&expanded) {
                        logging::log("FALLBACK", &format!("Failed to open file: {}", e));
                    } else {
                        logging::log("FALLBACK", "File opened with default application");
                    }
                }

                FallbackResult::SearchFiles { query } => {
                    logging::log("FALLBACK", &format!("SearchFiles: {}", query));
                    app.open_file_search(query, cx);
                }
            }
        }
        Err(e) => {
            logging::log("FALLBACK", &format!("Fallback execution failed: {}", e));
        }
    }
}

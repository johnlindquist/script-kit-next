#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptErrorAgentChatContextBundle {
    script_snapshot_path: String,
    script_snapshot_label: String,
    error_report_path: String,
    error_report_label: String,
}

struct ScriptErrorAgentChatContext<'a> {
    script_path: &'a str,
    error_message: &'a str,
    stderr_output: Option<&'a str>,
    exit_code: Option<i32>,
    stack_trace: Option<&'a str>,
    suggestions: &'a [String],
}

fn sanitize_script_error_context_name(value: &str, fallback: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}

/// The question that waits in the input: one line, since the report goes with it.
fn build_script_error_agent_chat_prompt(script_path: &str) -> String {
    let script_name = std::path::Path::new(script_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("the script");
    format!("Why did {script_name} fail? Fix it and check that it runs")
}

fn build_script_error_report_markdown(
    script_path: &str,
    error_message: &str,
    stderr_output: Option<&str>,
    exit_code: Option<i32>,
    stack_trace: Option<&str>,
    suggestions: &[String],
) -> String {
    let mut report = format!(
        "# Script Failure Report\n\n## Script Path\n`{script_path}`\n\n## Error Summary\n{error_message}\n"
    );

    if let Some(code) = exit_code {
        report.push_str(&format!("\n## Exit Code\n`{code}`\n"));
    }

    if !suggestions.is_empty() {
        report.push_str("\n## Suggestions\n");
        for suggestion in suggestions {
            report.push_str(&format!("- {suggestion}\n"));
        }
    }

    if let Some(stderr) = stderr_output {
        report.push_str("\n## Stderr\n```text\n");
        report.push_str(stderr);
        if !stderr.ends_with('\n') {
            report.push('\n');
        }
        report.push_str("```\n");
    }

    if let Some(trace) = stack_trace {
        report.push_str("\n## Stack Trace\n```text\n");
        report.push_str(trace);
        if !trace.ends_with('\n') {
            report.push('\n');
        }
        report.push_str("```\n");
    }

    report
}

fn persist_script_error_agent_chat_context_bundle_in_dir(
    root_dir: &std::path::Path,
    script_path: &str,
    error_message: &str,
    stderr_output: Option<&str>,
    exit_code: Option<i32>,
    stack_trace: Option<&str>,
    suggestions: &[String],
) -> Result<ScriptErrorAgentChatContextBundle, String> {
    let bundle_dir = root_dir.join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&bundle_dir).map_err(|error| {
        format!(
            "failed to create script-error Agent Chat context directory '{}': {error}",
            bundle_dir.display()
        )
    })?;

    let source_path = std::path::Path::new(script_path);
    let script_snapshot_label = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| sanitize_script_error_context_name(value, "script.ts"))
        .unwrap_or_else(|| "script.ts".to_string());
    let script_snapshot_path = bundle_dir.join(&script_snapshot_label);

    let script_snapshot_contents = match std::fs::read_to_string(source_path) {
        Ok(contents) => contents,
        Err(error) => format!(
            "// Script snapshot unavailable\n// Original path: {script_path}\n// Read error: {error}\n"
        ),
    };

    std::fs::write(&script_snapshot_path, script_snapshot_contents).map_err(|error| {
        format!(
            "failed to write script snapshot '{}': {error}",
            script_snapshot_path.display()
        )
    })?;

    let script_stem = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| sanitize_script_error_context_name(value, "script"))
        .unwrap_or_else(|| "script".to_string());
    let error_report_label = format!("{script_stem}-error-report.md");
    let error_report_path = bundle_dir.join(&error_report_label);
    let error_report = build_script_error_report_markdown(
        script_path,
        error_message,
        stderr_output,
        exit_code,
        stack_trace,
        suggestions,
    );

    std::fs::write(&error_report_path, error_report).map_err(|error| {
        format!(
            "failed to write script error report '{}': {error}",
            error_report_path.display()
        )
    })?;

    Ok(ScriptErrorAgentChatContextBundle {
        script_snapshot_path: script_snapshot_path.to_string_lossy().into_owned(),
        script_snapshot_label,
        error_report_path: error_report_path.to_string_lossy().into_owned(),
        error_report_label,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptErrorToastOpenMode {
    Default,
    TextEditor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptErrorToastActionTarget {
    id: &'static str,
    label: &'static str,
    path: std::path::PathBuf,
    open_mode: ScriptErrorToastOpenMode,
}

fn script_error_toast_action_targets(
    log_path: &std::path::Path,
    script_path: &str,
) -> Vec<ScriptErrorToastActionTarget> {
    let mut targets = Vec::new();

    if log_path.is_file() {
        targets.push(ScriptErrorToastActionTarget {
            id: "open-logs",
            label: "Open Logs",
            path: log_path.to_path_buf(),
            open_mode: ScriptErrorToastOpenMode::Default,
        });
    }

    let script_path = std::path::PathBuf::from(script_path);
    if script_path.is_file() {
        targets.push(ScriptErrorToastActionTarget {
            id: "edit-script",
            label: "Edit Script",
            path: script_path,
            open_mode: ScriptErrorToastOpenMode::TextEditor,
        });
    }

    targets
}

fn open_script_error_toast_target(target: &ScriptErrorToastActionTarget) -> std::io::Result<()> {
    let edit_as_text = target.open_mode == ScriptErrorToastOpenMode::TextEditor;

    #[cfg(target_os = "macos")]
    {
        let mut command = std::process::Command::new("open");
        if edit_as_text {
            command.arg("-t");
        }
        command.arg(&target.path).spawn().map(|_| ())
    }

    #[cfg(target_os = "linux")]
    {
        let _ = edit_as_text;
        std::process::Command::new("xdg-open")
            .arg(&target.path)
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "windows")]
    {
        let _ = edit_as_text;
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&target.path)
            .spawn()
            .map(|_| ())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = (target, edit_as_text);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "opening script error recovery targets is unsupported on this platform",
        ))
    }
}

fn build_script_error_toast(
    error_message: String,
    details_text: Option<String>,
    script_path: &str,
    log_path: &std::path::Path,
    theme: &crate::theme::Theme,
) -> Toast {
    let mut toast = Toast::error(error_message, theme)
        .details_opt(details_text.clone())
        .duration_ms(Some(TOAST_CRITICAL_MS));

    if let Some(trace) = details_text {
        toast = toast.action(ToastAction::new(
            "copy-error",
            "Copy Error",
            Box::new(move |_, _, _| {
                use arboard::Clipboard;

                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(trace.clone());
                    tracing::info!(category = "UI", "Error copied to clipboard");
                }
            }),
        ));
    }

    for target in script_error_toast_action_targets(log_path, script_path) {
        let action_id = target.id;
        let action_label = target.label;
        let target_for_callback = target.clone();
        toast = toast.action(ToastAction::new(
            action_id,
            action_label,
            Box::new(move |_, _, _| {
                if let Err(error) = open_script_error_toast_target(&target_for_callback) {
                    tracing::warn!(
                        category = "UI",
                        action_id = target_for_callback.id,
                        path = %target_for_callback.path.display(),
                        error = %error,
                        "Failed to open script error recovery target"
                    );
                } else {
                    tracing::info!(
                        category = "UI",
                        action_id = target_for_callback.id,
                        path = %target_for_callback.path.display(),
                        "Opened script error recovery target"
                    );
                }
            }),
        ));
    }

    toast
}

impl ScriptListApp {
    /// A failed script opens the chat with its snapshot and error report waiting in the line
    /// above the input and a question about them in the input; Enter asks it.
    fn route_script_error_to_agent_chat(
        &mut self,
        context: ScriptErrorAgentChatContext<'_>,
        cx: &mut Context<Self>,
    ) {
        let ScriptErrorAgentChatContext {
            script_path,
            error_message,
            stderr_output,
            exit_code,
            stack_trace,
            suggestions,
        } = context;
        let context_root = crate::setup::get_kit_path()
            .join("agent_chat")
            .join("script-error-context");
        let bundle = match persist_script_error_agent_chat_context_bundle_in_dir(
            &context_root,
            script_path,
            error_message,
            stderr_output,
            exit_code,
            stack_trace,
            suggestions,
        ) {
            Ok(bundle) => bundle,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "script_error_agent_chat_context_bundle_failed",
                    script_path = %script_path,
                    error = %error,
                );
                return;
            }
        };
        let prompt = build_script_error_agent_chat_prompt(script_path);
        self.one_window_open_chat(
            crate::one_window::chat_entry::ChatEntry::seeded(prompt)
                .with(crate::ai::AiContextPart::FilePath {
                    path: bundle.error_report_path,
                    label: bundle.error_report_label,
                })
                .with(crate::ai::AiContextPart::FilePath {
                    path: bundle.script_snapshot_path,
                    label: bundle.script_snapshot_label,
                }),
            cx,
        );
    }
}

#[cfg(test)]
mod script_error_toast_action_tests {
    use super::*;

    fn unique_test_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "script-kit-script-error-toast-{label}-{}",
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn existing_targets_append_copy_open_logs_and_edit_script_actions() {
        let root = unique_test_dir("existing");
        std::fs::create_dir_all(&root).expect("create script-error toast test directory");
        let log_path = root.join("script-kit-gpui.jsonl");
        let script_path = root.join("failed-script.ts");
        std::fs::write(&log_path, "{}\n").expect("write test log");
        std::fs::write(&script_path, "throw new Error('boom')\n").expect("write test script");
        let script_path_string = script_path.to_string_lossy().into_owned();

        let targets = script_error_toast_action_targets(&log_path, &script_path_string);
        assert_eq!(
            targets,
            vec![
                ScriptErrorToastActionTarget {
                    id: "open-logs",
                    label: "Open Logs",
                    path: log_path.clone(),
                    open_mode: ScriptErrorToastOpenMode::Default,
                },
                ScriptErrorToastActionTarget {
                    id: "edit-script",
                    label: "Edit Script",
                    path: script_path,
                    open_mode: ScriptErrorToastOpenMode::TextEditor,
                },
            ]
        );

        let toast = build_script_error_toast(
            "boom".to_string(),
            Some("trace".to_string()),
            &script_path_string,
            &log_path,
            &crate::theme::Theme::default(),
        );
        let actions = toast
            .get_actions()
            .iter()
            .map(|action| (action.id.as_str(), action.label.as_ref()))
            .collect::<Vec<_>>();
        assert_eq!(
            actions,
            vec![
                ("copy-error", "Copy Error"),
                ("open-logs", "Open Logs"),
                ("edit-script", "Edit Script"),
            ]
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn missing_file_targets_leave_copy_error_as_the_only_action() {
        let root = unique_test_dir("missing");
        let log_path = root.join("missing-log.jsonl");
        let script_path = root.join("missing-script.ts");
        let script_path_string = script_path.to_string_lossy().into_owned();

        let toast = build_script_error_toast(
            "boom".to_string(),
            Some("trace".to_string()),
            &script_path_string,
            &log_path,
            &crate::theme::Theme::default(),
        );
        let actions = toast
            .get_actions()
            .iter()
            .map(|action| (action.id.as_str(), action.label.as_ref()))
            .collect::<Vec<_>>();
        assert_eq!(actions, vec![("copy-error", "Copy Error")]);
    }
}

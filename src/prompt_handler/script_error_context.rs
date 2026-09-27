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

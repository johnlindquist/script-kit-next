#[cfg(target_os = "macos")]
pub fn run_osascript(script: &str, context: &str) -> anyhow::Result<String> {
    use anyhow::Context as _;

    tracing::debug!(
        context = context,
        script_len = script.len(),
        "running osascript"
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .with_context(|| {
            format!(
                "platform_run_osascript_failed: stage=spawn context={} attempt=osascript -e",
                context
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        tracing::error!(
            context = context,
            status = %output.status,
            stderr = %stderr,
            "osascript exited with non-zero status"
        );
        anyhow::bail!(
            "platform_run_osascript_failed: stage=exit_status context={} status={} stderr={}",
            context,
            output.status,
            if stderr.is_empty() {
                "<empty>"
            } else {
                stderr.as_str()
            }
        );
    }

    tracing::debug!(
        context = context,
        status = %output.status,
        stdout_len = stdout.len(),
        "osascript completed"
    );

    Ok(stdout)
}

#[cfg(not(target_os = "macos"))]
pub fn run_osascript(_script: &str, context: &str) -> anyhow::Result<String> {
    anyhow::bail!(
        "platform_run_osascript_failed: stage=unsupported_platform context={} platform={}",
        context,
        std::env::consts::OS
    );
}

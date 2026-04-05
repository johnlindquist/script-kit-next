use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("script_kit::audit=info")
            }),
        )
        .try_init();

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let outputs =
        script_kit_gpui::storybook::audit_report::write_standard_audit_reports(&repo_root, &repo_root)?;

    for output in outputs {
        println!("{}", output.display());
    }

    Ok(())
}

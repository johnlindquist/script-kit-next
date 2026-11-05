use std::io::{self, Seek, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// Bun's no-bundle build still executes macros. Its build CLI's macro switch
/// applies to the input, unlike a separately constructed Bun.Transpiler.
pub(super) fn check(bun: &Path, source: &str) -> Result<(), String> {
    // A pipe can yield only its first buffer to Bun's extensionless build
    // input. Use a seekable descriptor, and never reopen settings by name.
    let mut input =
        tempfile::tempfile().map_err(|e| format!("Could not create syntax input: {e}"))?;
    let meta = input
        .metadata()
        .map_err(|e| format!("Could not inspect syntax input: {e}"))?;
    if !meta.is_file() || meta.nlink() != 0 {
        return Err("Settings syntax input is not anonymous".into());
    }
    input
        .write_all(source.as_bytes())
        .and_then(|_| input.rewind())
        .map_err(|e| format!("Could not prepare syntax input: {e}"))?;
    let mut child = Command::new(bun)
        .env_clear()
        .env("HOME", "/dev/null")
        .env("BUN_RUNTIME_TRANSPILER_CACHE_PATH", "0")
        .current_dir("/")
        .args([
            "--config=/dev/null",
            "--no-env-file",
            "--no-install",
            "--no-macros",
            "--log-level=error",
            "build",
            "--target=bun",
            "--no-bundle",
            // '-' is a filename; without the TS loader an extensionless file is an asset.
            "--loader",
            ":ts",
            "/dev/fd/0",
            "--outfile=/dev/null",
        ])
        .stdin(Stdio::from(input))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not start the settings syntax checker: {e}"))?;
    // Missing inputs can emit errors with status zero. Count, but never retain
    // or publish, diagnostics: Bun quotes private source in its error messages.
    let diagnostics = io::copy(
        &mut child.stderr.take().expect("piped checker diagnostics"),
        &mut io::sink(),
    );
    let status = child
        .wait()
        .map_err(|e| format!("Could not wait for the settings syntax checker: {e}"))?;
    let diagnostics =
        diagnostics.map_err(|e| format!("Could not verify the settings syntax check: {e}"))?;
    if !status.success() || diagnostics != 0 {
        return Err(
            "Settings must be valid TypeScript without calling a compile-time macro".into(),
        );
    }
    Ok(())
}

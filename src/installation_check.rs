//! The installed app can qualify its own first-run resources without launching the desktop.
//! This entry point runs before hooks, services, hotkeys, credentials or GUI initialization.
mod settings;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub(crate) fn run_if_requested() -> bool {
    let mut args = std::env::args_os().skip(1);
    let Some(first) = args.next() else {
        return false;
    };
    if first != "--check-installation" && first != "--version" {
        return false;
    }
    if args.next().is_some() {
        eprintln!("Unexpected installation-check arguments");
        std::process::exit(64);
    }
    if first == "--version" {
        println!("Script Kit {}", env!("CARGO_PKG_VERSION"));
        return true;
    }
    match check() {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("Installation check failed: {e}");
            std::process::exit(1);
        }
    }
    true
}

fn check() -> Result<Value> {
    let private = tempfile::Builder::new()
        .prefix("sk-install-check-")
        .tempdir()?;
    let home = private.path().join("home");
    fs::create_dir(&home)?;
    let root = home.join("workspace");
    // A standalone process, before starting any threads. Do not inspect or migrate an operator home.
    for k in [
        "SCRIPT_KIT_CONFIG_PATH",
        "SCRIPT_KIT_CONFIG_CLI_PATH",
        "SCRIPT_KIT_PATH",
        "BUN_INSTALL",
        "MISE_DATA_DIR",
        "ASDF_DATA_DIR",
    ] {
        std::env::remove_var(k);
    }
    std::env::set_var("HOME", &home);
    std::env::set_var("SK_PATH", &root);
    std::env::set_var("SCRIPT_KIT_NONINTERACTIVE", "1");
    let first = crate::setup::ensure_kit_setup();
    ensure!(
        first.is_fresh_install && first.kit_path == root && first.warnings.is_empty(),
        "Fresh workspace could not be initialized"
    );
    let sdk = fs::read(root.join("sdk/kit-sdk.ts"))?;
    ensure!(
        sdk == include_bytes!("../scripts/kit-sdk.ts").as_slice(),
        "Installed SDK has the wrong bytes"
    );
    let scripts = crate::scripts::read_scripts()?;
    let starter = root.join("plugins/main/scripts/hello-world.ts");
    ensure!(
        scripts.iter().any(|s| s.path == starter),
        "Starter scripts missing from the actual script catalog"
    );
    let exe = std::env::current_exe()?;
    let dir = exe.parent().context("Missing executable directory")?;
    let packaged = dir.file_name().is_some_and(|s| s == "MacOS")
        && dir
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|s| s == "Contents");
    if packaged {
        ensure!(
            fs::read(dir.join("../Resources/scripts/kit-sdk.ts"))? == sdk
                && fs::metadata(dir.join("goose"))?.is_file(),
            "Missing or mismatched bundled resources"
        );
    }
    let config = root.join("config.ts");
    let mut original = fs::read(&config)?;
    original.extend_from_slice(settings::PERSONAL_COMMENT.as_bytes());
    fs::write(&config, &original)?;
    let personal = b"// Name: Personal check\nexport {};\n";
    fs::write(&starter, personal)?;
    let again = crate::setup::ensure_kit_setup();
    ensure!(
        !again.is_fresh_install
            && again.warnings.is_empty()
            && fs::read(&starter)? == personal
            && fs::read(&config)? == original,
        "A second launch overwrote personal files"
    );
    let settings_proof = if let Some(bun) = crate::setup::resolve_bun() {
        let pin = bun.to_str().context("Settings runtime path is not UTF-8")?;
        let mut run = |args: &[&str]| -> Result<(Option<i32>, Value)> {
            let out = crate::setup::settings_cli_command(Some(pin))
                .map_err(anyhow::Error::msg)?
                .args(args)
                .output()?;
            Ok((
                out.status.code(),
                serde_json::from_slice(&out.stdout).context("Invalid settings helper response")?,
            ))
        };
        let proof = settings::verify(&config, &original, &mut run)?;
        let after = fs::read(&config)?;
        let previous_font = f32::from(proof.value);
        // The personal release separately exercises its real launcher reading and Undo.
        let choice = sk_one_window::font_preference::read("Use 21px text in the editor")
            .context("Missing local text-size reading")?;
        let undo = crate::mcp_config_tools::font::apply(choice, previous_font)?
            .context("Text-size edit did not produce an undo")?;
        let (exit, changed) = run(&["get", "editorFontSize"])?;
        ensure!(
            exit == Some(0) && changed["success"] == true && changed["data"]["value"] == 21,
            "Text-size readback failed"
        );
        ensure!(
            undo.restore()? == Some(previous_font) && fs::read(&config)? == after,
            "Text-size undo failed"
        );
        Some(proof)
    } else {
        None
    };
    let sdk_hash = format!("{:x}", Sha256::digest(&sdk));
    private.close()?;
    Ok(
        json!({"schemaVersion":1,"version":env!("CARGO_PKG_VERSION"),"packaged":packaged,"freshWorkspace":true,"sdkSha256":sdk_hash,"starterScriptsIndexed":scripts.len(),"userEditsPreserved":true,"settingsTransaction":if settings_proof.is_some(){"verified"}else{"bun_unavailable"},"launcherTextPreferenceTransaction":if settings_proof.is_some(){"verified"}else{"bun_unavailable"},"settingsProof":settings_proof,"scratchRemoved":true}),
    )
}

//! Exercise the production launcher transaction against the distributed settings helper. The
//! private workspace has no app, clipboard, providers, inherited project config or GPUI linkage.
use serde_json::Value;
use sk_one_window::font_preference::{change, read};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Workspace(PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_text_size_request_changes_only_the_named_setting_and_undo_refuses_conflicts() {
    let temp = std::env::temp_dir().join(format!(
        "kit-font-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut dir = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        dir.mode(0o700);
    }
    dir.create(&temp).unwrap();
    let root = Workspace(temp);
    let sdk = root.0.join("sdk");
    fs::create_dir(&sdk).unwrap();
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts");
    for name in ["config-cli.ts", "config-schema.ts", "config-document.ts"] {
        fs::copy(shipped.join(name), sdk.join(name)).unwrap();
    }
    let bun = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .filter(|p| p.is_absolute())
        .map(|p| p.join(if cfg!(windows) { "bun.exe" } else { "bun" }))
        .find(|p| p.is_file())
        .expect("Bun required for the shipped settings helper");
    let config = root.0.join("config.ts");
    let original = "export default { /* keep */ terminalFontSize: 15 };\n";
    fs::write(&config, original).unwrap();
    let cli = |args: &[&str]| -> Result<Value, String> {
        let p = Command::new(&bun)
            .args([
                if cfg!(windows) {
                    "--config=NUL"
                } else {
                    "--config=/dev/null"
                },
                "--no-env-file",
                "--no-install",
            ])
            .arg(sdk.join("config-cli.ts"))
            .args(args)
            .current_dir(&sdk)
            .env_clear()
            .env("HOME", &root.0)
            .env("SCRIPT_KIT_CONFIG_PATH", &config)
            .env("SCRIPT_KIT_NONINTERACTIVE", "1")
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&p.stdout).unwrap();
        if p.status.success() && v["success"] == true {
            Ok(v["data"].clone())
        } else {
            Err(v["error"].as_str().unwrap_or("failed").to_owned())
        }
    };
    let choice = read("Use 23px type in my editor").unwrap();
    let undo = change::apply(choice, 16., cli).unwrap().unwrap();
    assert_eq!(cli(&["get", "editorFontSize"]).unwrap()["value"], 23);
    assert_eq!(cli(&["get", "terminalFontSize"]).unwrap()["value"], 15);
    assert!(change::apply(choice, 23., cli).unwrap().is_none());
    assert_eq!(undo.restore(cli).unwrap(), None);
    assert_eq!(cli(&["get", "editorFontSize"]).unwrap()["isDefault"], true);
    let term = read("Set my terminal font size to 19").unwrap();
    let undo = change::apply(term, 15., cli).unwrap().unwrap();
    assert_eq!(undo.restore(cli).unwrap(), Some(15.));
    let undo = change::apply(choice, 16., cli).unwrap().unwrap();
    // Another writer saves after the user's request. Neither an old Undo nor a stale row wins.
    let human = b"export default { editorFontSize: 20, terminalFontSize: 18 }; // hand edit\n";
    fs::write(&config, human).unwrap();
    assert!(undo.restore(cli).is_err());
    assert!(change::apply(choice, 16., cli).is_err());
    assert_eq!(fs::read(&config).unwrap(), human);
    // Also change the file after the initial read but before publication: the real CLI's CAS must refuse.
    let raced = change::apply(choice, 20., |args| {
        if args[0] == "set-json" {
            cli(&["set-json", "terminalFontSize", "17"])?;
        }
        cli(args)
    });
    assert!(raced.is_err());
    assert_eq!(cli(&["get", "editorFontSize"]).unwrap()["value"], 20);
    assert_eq!(cli(&["get", "terminalFontSize"]).unwrap()["value"], 17);
}

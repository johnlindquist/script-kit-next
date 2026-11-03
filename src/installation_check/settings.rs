//! The settings CLI does not grade its own write. Read the disposable config as data,
//! bind both successful replies to its bytes and require the actual stale-revision refusal.
use anyhow::{ensure, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs::OpenOptions, io::Read, path::Path};

const KEY: &str = "editorFontSize";
pub(super) const PERSONAL_COMMENT: &str = "\n// Personal comment to preserve on upgrade.\n";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Proof {
    key: &'static str,
    before_value: f64,
    pub(super) value: u16,
    before_sha256: String,
    written_sha256: String,
    after_rejection_sha256: String,
    rejection: &'static str,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// Read only the known fixture path, not a path named by the child. No permission repair.
fn bytes(path: &Path) -> Result<Vec<u8>> {
    const MAX: u64 = 1_048_576;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    let meta = file.metadata()?;
    ensure!(
        meta.is_file() && meta.len() <= MAX,
        "Not a bounded regular settings file"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(meta.nlink() == 1, "Unexpected linked settings file");
    }
    let mut result = Vec::new();
    file.take(MAX + 1).read_to_end(&mut result)?;
    ensure!(
        result.len() as u64 <= MAX,
        "Settings file grew during the check"
    );
    Ok(result)
}

fn literal(source: &[u8]) -> Result<Value> {
    crate::config::editor::parse_static_config(std::str::from_utf8(source)?)
        .map_err(anyhow::Error::msg)
}

fn confirm_read(
    (exit, reply): (Option<i32>, Value),
    path: &Path,
    sha: &str,
    value: f64,
    default: bool,
) -> Result<()> {
    let data = &reply["data"];
    ensure!(
        exit == Some(0)
            && reply["success"] == true
            && data["key"] == KEY
            && data["exists"] == true
            && data["path"].as_str().map(Path::new) == Some(path)
            && data["revision"].as_str() == Some(sha)
            && data["value"].as_f64() == Some(value)
            && data["isDefault"] == default,
        "Settings read disagrees with the saved file"
    );
    Ok(())
}

pub(super) fn verify(
    path: &Path,
    original: &[u8],
    mut run: impl FnMut(&[&str]) -> Result<(Option<i32>, Value)>,
) -> Result<Proof> {
    let mut expected = literal(original)?;
    let declared = expected.get(KEY);
    let before_value = match declared {
        Some(v) => v
            .as_f64()
            .context("Fixture editor font size is not numeric")?,
        None => f64::from(crate::config::defaults::DEFAULT_EDITOR_FONT_SIZE),
    };
    let before_sha256 = digest(original);
    confirm_read(
        run(&["get", KEY])?,
        path,
        &before_sha256,
        before_value,
        declared.is_none(),
    )?;
    ensure!(
        bytes(path)? == original,
        "Reading a setting changed the file"
    );

    let value: u16 = if before_value == 27.0 { 28 } else { 27 };
    expected
        .as_object_mut()
        .context("Fixture config is not an object")?
        .insert(KEY.into(), json!(value));
    let (exit, set) = run(&[
        "set-json",
        KEY,
        &value.to_string(),
        "--if-revision",
        &before_sha256,
    ])?;
    let written = bytes(path)?;
    ensure!(
        exit == Some(0) && set["success"] == true,
        "Installed settings write failed"
    );
    ensure!(
        written != original
            && written.ends_with(PERSONAL_COMMENT.as_bytes())
            && literal(&written)? == expected,
        "Settings write did not persist the requested change and preserve neighboring settings"
    );
    let written_sha256 = digest(&written);
    ensure!(
        set["data"]["key"] == KEY
            && set["data"]["value"].as_u64() == Some(u64::from(value))
            && set["data"]["revision"].as_str() == Some(written_sha256.as_str()),
        "Settings write receipt does not match the file"
    );
    confirm_read(
        run(&["get", KEY])?,
        path,
        &written_sha256,
        f64::from(value),
        false,
    )?;
    ensure!(
        bytes(path)? == written,
        "Reading back a setting changed the file"
    );
    let (exit, refused) = run(&["set-json", KEY, "29", "--if-revision", &before_sha256])?;
    ensure!(
        exit == Some(1) && refused["success"] == false && refused["errorCode"] == "stale_revision",
        "Stale settings write did not return the revision refusal"
    );
    let after = bytes(path)?;
    ensure!(
        after == written,
        "Stale settings update changed the saved file"
    );
    Ok(Proof {
        key: KEY,
        before_value,
        value,
        before_sha256,
        written_sha256,
        after_rejection_sha256: digest(&after),
        rejection: "stale_revision",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[derive(Clone, Copy, PartialEq)]
    enum Fault {
        None,
        NoWrite,
        WrongNeighbor,
        WrongRefusal,
        StaleWrite,
    }

    fn transaction(fault: Fault) -> Result<Proof> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("config.ts");
        let original = format!(
            "export default {{ editorFontSize: 16, terminalFontSize: 14 }};{PERSONAL_COMMENT}"
        );
        let updated = original.replace("editorFontSize: 16", "editorFontSize: 27");
        fs::write(&path, &original)?;
        let old = digest(original.as_bytes());
        let new = digest(updated.as_bytes());
        let mut calls = 0;
        verify(&path, original.as_bytes(), |args| {
            calls += 1;
            match calls {
                1 | 3 => {
                    assert_eq!(args, ["get", KEY]);
                    Ok((
                        Some(0),
                        json!({"success":true,"data":{"key":KEY,"path":path,"exists":true,"isDefault":false,"value":if calls==1{16}else{27},"revision":if calls==1{&old}else{&new}}}),
                    ))
                }
                2 => {
                    assert_eq!(args, ["set-json", KEY, "27", "--if-revision", old.as_str()]);
                    match fault {
                        Fault::NoWrite => (),
                        Fault::WrongNeighbor => {
                            fs::write(&path, updated.replace("Size: 14", "Size: 18"))?
                        }
                        _ => fs::write(&path, &updated)?,
                    }
                    Ok((
                        Some(0),
                        json!({"success":true,"data":{"key":KEY,"value":27,"revision":new}}),
                    ))
                }
                4 => {
                    assert_eq!(args, ["set-json", KEY, "29", "--if-revision", old.as_str()]);
                    if fault == Fault::StaleWrite {
                        fs::write(&path, &original)?;
                    }
                    Ok((
                        Some(1),
                        json!({"success":false,"errorCode":if fault==Fault::WrongRefusal{"unavailable"}else{"stale_revision"}}),
                    ))
                }
                _ => unreachable!(),
            }
        })
    }

    #[test]
    fn settings_proof_requires_the_persisted_edit_and_the_actual_revision_refusal() {
        let proof = transaction(Fault::None).unwrap();
        assert_ne!(proof.before_sha256, proof.written_sha256);
        assert_eq!(proof.written_sha256, proof.after_rejection_sha256);
        for fault in [
            Fault::NoWrite,
            Fault::WrongNeighbor,
            Fault::WrongRefusal,
            Fault::StaleWrite,
        ] {
            assert!(transaction(fault).is_err());
        }
    }
}

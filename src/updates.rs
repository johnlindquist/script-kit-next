//! GitHub-releases-backed update checker for Script Kit.
//!
//! Polls `https://api.github.com/repos/johnlindquist/script-kit-next/releases/latest` off the
//! GPUI main thread and exposes the result as an `Arc<RwLock<UpdateState>>` the
//! tray menu can render. No Sparkle, no signed appcast — minimal first pass.

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

const RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/johnlindquist/script-kit-next/releases/latest";
const USER_AGENT: &str = "script-kit-gpui";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    Idle,
    Checking,
    UpToDate,
    Available { version: String, url: String },
    Error(String),
}

impl UpdateState {
    pub fn release_url(&self) -> Option<&str> {
        if let UpdateState::Available { url, .. } = self {
            Some(url.as_str())
        } else {
            None
        }
    }
}

/// Spawn a background HTTP fetch and update `state` when the response arrives.
/// Calls `on_complete` (on the worker thread) once `state` has been written so
/// the caller can refresh UI (e.g. tray menu) without polling.
pub fn check_now<F>(state: Arc<RwLock<UpdateState>>, on_complete: F)
where
    F: FnOnce() + Send + 'static,
{
    {
        let Ok(mut guard) = state.write() else {
            on_complete();
            return;
        };
        if matches!(*guard, UpdateState::Checking) {
            return;
        }
        *guard = UpdateState::Checking;
    }

    thread::spawn(move || {
        let next = fetch_latest();
        if let Ok(mut guard) = state.write() {
            *guard = next;
        }
        on_complete();
    });
}

fn fetch_latest() -> UpdateState {
    let agent = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(Duration::from_secs(10)))
        .timeout_connect(Some(Duration::from_secs(5)))
        .build()
        .new_agent();

    let response = agent
        .get(RELEASES_LATEST_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call();

    let mut response = match response {
        Ok(r) => r,
        Err(e) => return UpdateState::Error(format!("request failed: {e}")),
    };

    let json: serde_json::Value = match response.body_mut().read_json() {
        Ok(v) => v,
        Err(e) => return UpdateState::Error(format!("parse failed: {e}")),
    };

    pick_release(&json, env!("CARGO_PKG_VERSION"))
}

fn pick_release(json: &serde_json::Value, current: &str) -> UpdateState {
    let tag = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();
    let url = json
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if tag.is_empty() {
        return UpdateState::Error("missing tag_name".into());
    }

    if version_gt(&tag, current) {
        let asset_url = downloadable_asset_url(json);
        if asset_url.is_none() {
            tracing::debug!("update.no_assets_yet tag={tag}");
            return UpdateState::UpToDate;
        }

        let repo_path = RELEASES_LATEST_URL
            .trim_start_matches("https://api.github.com/repos/")
            .trim_end_matches("/releases/latest");
        tracing::debug!(
            "update.manifest_expected url=https://github.com/{repo_path}/releases/download/v{tag}/release-manifest.json"
        );

        UpdateState::Available {
            version: tag,
            url: asset_url.filter(|url| !url.is_empty()).unwrap_or(url),
        }
    } else {
        UpdateState::UpToDate
    }
}

/// Pull the manifest entry matching `asset_name` from the parsed manifest JSON.
/// Returns the expected SHA256 hex digest if present.
#[allow(dead_code)]
fn manifest_sha256_for<'a>(manifest: &'a serde_json::Value, asset_name: &str) -> Option<&'a str> {
    manifest
        .get("artifacts")?
        .as_array()?
        .iter()
        .find(|a| a.get("name").and_then(|v| v.as_str()) == Some(asset_name))?
        .get("sha256")?
        .as_str()
}

fn downloadable_asset_url(json: &serde_json::Value) -> Option<String> {
    json.get("assets")
        .and_then(|v| v.as_array())
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name").and_then(|v| v.as_str())?;
                let lowercase_name = name.to_ascii_lowercase();
                if !(lowercase_name.ends_with(".zip")
                    || lowercase_name.ends_with(".dmg")
                    || lowercase_name.ends_with(".tar.gz"))
                {
                    return None;
                }

                Some(
                    asset
                        .get("browser_download_url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                )
            })
        })
}

/// Lightweight `a > b` comparator for dotted numeric versions.
/// Non-numeric segments compare as zero. Pre-release suffixes are ignored
/// (`1.2.3-rc1` → `1.2.3`). Adequate for "is GitHub tag newer than CARGO_PKG_VERSION?".
pub fn version_gt(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('-')
            .next()
            .unwrap_or(s)
            .split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let av = parse(a);
    let bv = parse(b);
    let len = av.len().max(bv.len());
    for i in 0..len {
        let ai = av.get(i).copied().unwrap_or(0);
        let bi = bv.get(i).copied().unwrap_or(0);
        if ai != bi {
            return ai > bi;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn version_gt_basic() {
        assert!(version_gt("1.0.1", "1.0.0"));
        assert!(version_gt("1.1.0", "1.0.99"));
        assert!(version_gt("2.0.0", "1.99.99"));
        assert!(!version_gt("1.0.0", "1.0.0"));
        assert!(!version_gt("1.0.0", "1.0.1"));
    }

    #[test]
    fn version_gt_uneven_lengths() {
        assert!(version_gt("1.2", "1.1.9"));
        assert!(!version_gt("1.0", "1.0.0"));
        assert!(version_gt("1.0.1", "1.0"));
    }

    #[test]
    fn version_gt_strips_prerelease() {
        // 1.2.3-rc1 is treated as 1.2.3
        assert!(!version_gt("1.2.3-rc1", "1.2.3"));
        assert!(version_gt("1.2.3-rc1", "1.2.2"));
    }

    #[test]
    fn release_url_returns_some_only_for_available() {
        assert_eq!(UpdateState::Idle.release_url(), None);
        assert_eq!(UpdateState::Checking.release_url(), None);
        assert_eq!(UpdateState::UpToDate.release_url(), None);
        assert_eq!(UpdateState::Error("x".into()).release_url(), None);
        let s = UpdateState::Available {
            version: "1.0.0".into(),
            url: "https://example.com".into(),
        };
        assert_eq!(s.release_url(), Some("https://example.com"));
    }

    #[test]
    fn manifest_sha256_pick_by_name() {
        let m = serde_json::json!({
            "version": "1.0.0",
            "artifacts": [
                { "name": "Script-Kit-macos.zip", "sha256": "deadbeef", "size_bytes": 100 }
            ]
        });
        assert_eq!(
            manifest_sha256_for(&m, "Script-Kit-macos.zip"),
            Some("deadbeef")
        );
        assert_eq!(manifest_sha256_for(&m, "missing.zip"), None);
    }

    #[test]
    fn manifest_sha256_handles_empty_artifacts() {
        let m = serde_json::json!({ "version": "1.0.0", "artifacts": [] });
        assert_eq!(manifest_sha256_for(&m, "anything.zip"), None);
    }

    #[test]
    fn manifest_sha256_handles_malformed() {
        let m = serde_json::json!({ "version": "1.0.0" });
        assert_eq!(manifest_sha256_for(&m, "anything.zip"), None);
    }

    #[test]
    fn picks_zip_asset_url() {
        let release = json!({
            "tag_name": "v9.9.9",
            "html_url": "https://example.com/releases/v9.9.9",
            "assets": [
                {
                    "name": "checksums.txt",
                    "browser_download_url": "https://example.com/downloads/checksums.txt"
                },
                {
                    "name": "ScriptKit.zip",
                    "browser_download_url": "https://example.com/downloads/ScriptKit.zip"
                }
            ]
        });

        assert_eq!(
            pick_release(&release, "1.0.0"),
            UpdateState::Available {
                version: "9.9.9".into(),
                url: "https://example.com/downloads/ScriptKit.zip".into(),
            }
        );
    }

    #[test]
    fn falls_back_to_uptodate_when_no_assets() {
        let release = json!({
            "tag_name": "v9.9.9",
            "html_url": "https://example.com/releases/v9.9.9",
            "assets": []
        });

        assert_eq!(pick_release(&release, "1.0.0"), UpdateState::UpToDate);
    }

    #[test]
    fn case_insensitive_extension() {
        let uppercase = json!({
            "tag_name": "v9.9.9",
            "html_url": "https://example.com/releases/v9.9.9",
            "assets": [
                {
                    "name": "ScriptKit.ZIP",
                    "browser_download_url": "https://example.com/downloads/ScriptKit.ZIP"
                }
            ]
        });
        let mixed_case = json!({
            "tag_name": "v9.9.9",
            "html_url": "https://example.com/releases/v9.9.9",
            "assets": [
                {
                    "name": "ScriptKit.Zip",
                    "browser_download_url": "https://example.com/downloads/ScriptKit.Zip"
                }
            ]
        });

        assert_eq!(
            pick_release(&uppercase, "1.0.0"),
            UpdateState::Available {
                version: "9.9.9".into(),
                url: "https://example.com/downloads/ScriptKit.ZIP".into(),
            }
        );
        assert_eq!(
            pick_release(&mixed_case, "1.0.0"),
            UpdateState::Available {
                version: "9.9.9".into(),
                url: "https://example.com/downloads/ScriptKit.Zip".into(),
            }
        );
    }
}

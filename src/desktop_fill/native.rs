use super::{Backend, Failure, Window};
use serde_json::Value;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

pub struct Native {
    session: String,
}
impl Default for Native {
    fn default() -> Self {
        Self {
            session: format!("kit-fill-{}", Uuid::new_v4()),
        }
    }
}
impl Native {
    fn path() -> Result<PathBuf, Failure> {
        let path = std::env::var_os("SCRIPT_KIT_AGENT_DESKTOP_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                crate::setup::get_kit_path().join("tools/agent-desktop/1.0.2/agent-desktop")
            });
        if !path.is_absolute() || !path.is_file() {
            return Err("provider_missing");
        };
        Ok(path)
    }
    // Only called off the GPUI / MCP event loop. No shell, inherited flags,
    // trace, --headed, clipboard, click, or generic command transport.
    fn cmd(&self, cmd: &str, args: &[&str]) -> Result<Value, Failure> {
        use crate::runtime_policy::{check, ExternalEffect};
        if !cfg!(target_os = "macos")
            || std::env::var_os("CI").is_some_and(|s| s != "false" && s != "0")
            || std::env::var_os("SCRIPT_KIT_NONINTERACTIVE").is_some_and(|s| s != "0")
            || check(ExternalEffect::Process).is_err()
            || check(if cmd == "set-value" {
                ExternalEffect::NativeInput
            } else {
                ExternalEffect::SystemDiscovery
            })
            .is_err()
        {
            return Err("native_effects_forbidden");
        }
        let path = Self::path()?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| "provider_unavailable")?;
        runtime.block_on(async {
            let mut child = tokio::process::Command::new(path);
            child.env_clear();
            for k in ["HOME", "TMPDIR", "LANG"] {
                if let Some(v) = std::env::var_os(k) {
                    child.env(k, v);
                }
            }
            child.env(
                "AGENT_DESKTOP_HOME",
                crate::setup::get_kit_path().join("db/agent-desktop"),
            );
            let mut p = child
                .arg("--session")
                .arg(&self.session)
                .arg(cmd)
                .args(args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()
                .map_err(|_| "provider_unavailable")?;
            tokio::time::timeout(Duration::from_secs(7), async {
                let mut buf = Vec::new();
                let cap = 2 * 1024 * 1024;
                p.stdout
                    .take()
                    .ok_or("provider_unavailable")?
                    .take(cap + 1)
                    .read_to_end(&mut buf)
                    .await
                    .map_err(|_| "provider_io")?;
                if buf.len() > cap as usize {
                    return Err("provider_output_limit");
                }
                let status = p.wait().await.map_err(|_| "provider_io")?;
                let mut result: Value =
                    serde_json::from_slice(&buf).map_err(|_| "invalid_provider_response")?;
                if result["version"] != "2.5" {
                    return Err("provider_version");
                }
                if !status.success() || result["ok"] != true {
                    return Err(match result["error"]["code"].as_str() {
                        Some("STALE_REF" | "WINDOW_NOT_FOUND" | "APP_NOT_FOUND") => {
                            "window_changed"
                        }
                        Some("PERM_DENIED" | "PERMISSION_DENIED" | "ACCESSIBILITY_DENIED") => {
                            "permission_denied"
                        }
                        Some("SURFACE_NOT_FOUND" | "ELEMENT_NOT_FOUND") => "unsafe_surface",
                        _ => "provider_refused",
                    });
                }
                if result["command"] != cmd {
                    return Err("invalid_provider_response");
                };
                Ok(result["data"].take())
            })
            .await
            .map_err(|_| "provider_timeout")?
        })
    }
}
impl Backend for Native {
    fn windows(&self, app: &str) -> Result<Vec<Window>, Failure> {
        if self.cmd("version", &[])?["version"] != "1.0.2" {
            return Err("provider_version");
        };
        let data = self.cmd("list-windows", &["--app", app])?;
        Ok(data
            .as_array()
            .ok_or("invalid_provider_response")?
            .iter()
            .filter(|w| w["visible"] == true)
            .filter_map(|w| {
                Some(Window {
                    app: w["app_name"].as_str()?.into(),
                    pid: w["pid"].as_u64()?.try_into().ok()?,
                    window_id: w["id"].as_str()?.into(),
                    title: w["title"].as_str()?.into(),
                    process_instance: w["process_instance"]
                        .as_str()
                        .filter(|s| !s.is_empty())?
                        .into(),
                })
            })
            .collect())
    }
    fn sheet(&self, w: &Window) -> Result<Value, Failure> {
        self.cmd(
            "snapshot",
            &[
                "--app",
                &w.app,
                "--window-id",
                &w.window_id,
                "--surface",
                "sheet",
                "--compact",
                "--timeout-ms",
                "3000",
            ],
        )
    }
    fn value(&self, r: &str) -> Result<String, Failure> {
        let d = self.cmd("get", &[r, "--property", "value"])?;
        if d["ref"] != r || d["property"] != "value" {
            return Err("invalid_provider_response");
        };
        d["value"]
            .as_str()
            .map(String::from)
            .ok_or("unreadable_field")
    }
    fn set_value(&self, r: &str, value: &str) -> Result<(), Failure> {
        let d = self.cmd("set-value", &["--timeout-ms", "1500", "--", r, value])?;
        if d["disposition"]["delivery"] != "delivered_verified" || d["post_state"]["value"] != value
        {
            return Err("delivery_unverified");
        };
        Ok(())
    }
}

//! One bounded child, no shell. Discard stderr (may contain private data). The process group
//! belongs to this invocation; never signal a borrowed server, terminal, or an agent's process.
use std::{path::Path, process::Stdio, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub(crate) async fn output(
    program: &Path,
    args: &[&str],
    cwd: &Path,
    env: &[(&str, std::ffi::OsString)],
    input: Vec<u8>,
    seconds: u64,
    limit: usize,
) -> Result<Vec<u8>, &'static str> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)
        .map_err(|_| "process_forbidden")?;
    let noninteractive = std::env::var_os("SCRIPT_KIT_NONINTERACTIVE").is_some_and(|s| s != "0")
        || std::env::var_os("CI").is_some_and(|s| s != "false" && s != "0");
    if noninteractive && !isolated_script_fixture() {
        return Err("process_forbidden");
    }
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args).current_dir(cwd).env_clear();
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.as_std_mut().process_group(0);
    }
    let mut child = cmd.spawn().map_err(|_| "could_not_start")?;
    let pid = child.id().ok_or("could_not_start")? as i32;
    let mut stdin = child.stdin.take().ok_or("process_io")?;
    let stdout = child.stdout.take().ok_or("process_io")?;
    let io = smol::future::try_zip(
        async move {
            if !input.is_empty() {
                stdin.write_all(&input).await.map_err(|_| "input_unknown")?;
            }
            drop(stdin);
            Ok(())
        },
        async move {
            let mut bytes = Vec::new();
            stdout
                .take(limit as u64 + 1)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| "output_unknown")?;
            if bytes.len() > limit {
                return Err("output_limit");
            }
            Ok(bytes)
        },
    );
    let result = tokio::time::timeout(Duration::from_secs(seconds), async {
        let (_, bytes) = io.await?;
        #[cfg(unix)]
        while !exited_unreaped(pid)? {
            tokio::time::sleep(Duration::from_millis(8)).await;
        }
        #[cfg(not(unix))]
        if !child.wait().await.map_err(|_| "exit_unknown")?.success() {
            return Err("nonzero_exit");
        }
        Ok(bytes)
    })
    .await
    .unwrap_or(Err("timeout_delivery_unknown"));
    // On failure the command may already have produced effects. Do not retry it.
    #[cfg(unix)]
    if pid > 1 && exited_unreaped(pid).is_ok() {
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
    }
    if result.is_err() {
        let _ = child.start_kill();
    }
    let status = child.wait().await.map_err(|_| "exit_unknown")?;
    let bytes = result?;
    if !status.success() {
        return Err("nonzero_exit");
    }
    Ok(bytes)
}
// Keep the leader's PID reserved until group cleanup. Never signal a recycled PGID.
#[cfg(unix)]
fn exited_unreaped(pid: i32) -> Result<bool, &'static str> {
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    if unsafe {
        libc::waitid(
            libc::P_PID,
            pid as _,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    } != 0
    {
        return Err("child_identity_lost");
    }
    Ok(info.si_signo != 0)
}
fn isolated_script_fixture() -> bool {
    #[cfg(test)]
    {
        return super::herdr::TEST_SOCKET.get().is_some()
            && std::env::var_os("SK_BRIDGE_FIXTURE").is_some_and(|root| {
                crate::setup::get_kit_path()
                    == std::path::PathBuf::from(root).join("home/.scriptkit")
            });
    }
    #[cfg(not(test))]
    {
        false
    }
}
pub(crate) fn environment(keys: &[&'static str]) -> Vec<(&'static str, std::ffi::OsString)> {
    keys.iter()
        .filter_map(|k| std::env::var_os(k).map(|v| (*k, v)))
        .collect()
}

//! Export user-authored Kit files to a dedicated branch of a verified private repository.
//! The workspace's index, refs and history are never part of the upload.

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[path = "snapshot.rs"]
mod snapshot;

pub const SCRIPT_KIT_SYNC_DRY_RUN_ENV: &str = "SCRIPT_KIT_SYNC_DRY_RUN";
const BRANCH: &str = "script-kit-personal";
const OWNER_MARKER: &str = ".script-kit-snapshot-v1";
const MARK: &str = "Script Kit personal workspace snapshot v1\n";

pub struct SyncToGithubReport {
    pub workspace: PathBuf,
    pub dry_run: bool,
    pub files: usize,
}
impl SyncToGithubReport {
    pub fn summary_message(&self) -> String {
        if self.dry_run {
            format!("{} personal files ready; nothing uploaded", self.files)
        } else {
            format!(
                "{} personal files synced to private GitHub backup",
                self.files
            )
        }
    }
}

pub fn sync_to_github_workspace() -> Result<SyncToGithubReport> {
    sync_to_github_at_path(crate::setup::get_kit_path())
}

pub fn sync_to_github_at_path(workspace: PathBuf) -> Result<SyncToGithubReport> {
    ensure!(
        std::env::var("SCRIPT_KIT_NONINTERACTIVE").as_deref() != Ok("1"),
        "Personal GitHub backup is disabled in noninteractive mode"
    );
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)?;
    let dry_run = std::env::var(SCRIPT_KIT_SYNC_DRY_RUN_ENV).as_deref() == Ok("1");
    ensure!(
        workspace.symlink_metadata()?.is_dir(),
        "Invalid workspace root"
    );
    let workspace = workspace.canonicalize()?;
    let scratch = tempfile::tempdir()?;
    let files = snapshot::collect(&workspace, scratch.path())?;
    ensure!(!files.is_empty(), "No personal files to sync");
    if !dry_run {
        let repo = configured_destination(&workspace)?;
        publish(&files, repo.as_deref(), |method, path, body| {
            request(scratch.path(), method, path, body)
        })?;
    }
    Ok(SyncToGithubReport {
        workspace,
        dry_run,
        files: files.len(),
    })
}

fn valid_component(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s != "."
        && s != ".."
        && !s.starts_with('-')
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}
fn github_name(url: &str) -> Result<String> {
    let name = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("git@github.com:"))
        .or_else(|| url.strip_prefix("ssh://git@github.com/"))
        .context("Only a github.com origin can be synced")?;
    let name = name.strip_suffix(".git").unwrap_or(name);
    let (owner, repo) = name.split_once('/').context("Invalid GitHub origin")?;
    ensure!(
        valid_component(owner) && valid_component(repo),
        "Invalid GitHub origin"
    );
    Ok(name.to_owned())
}
fn configured_destination(workspace: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(workspace.join(".git")) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Ok(m) => ensure!(
            m.is_dir(),
            "Automatic sync requires an ordinary workspace repository"
        ),
        Err(e) => return Err(e.into()),
    }
    let (ok, raw) = git(workspace, &["remote"])?;
    ensure!(ok, "Couldn't read workspace remotes");
    if !std::str::from_utf8(&raw)?.lines().any(|l| l == "origin") {
        return Ok(None);
    }
    let one = |args: &[&str]| -> Result<String> {
        let (ok, raw) = git(workspace, args)?;
        let urls = std::str::from_utf8(&raw)?.lines().collect::<Vec<_>>();
        ensure!(
            ok && urls.len() == 1,
            "Sync requires one unambiguous GitHub origin"
        );
        github_name(urls[0])
    };
    let name = one(&["remote", "get-url", "--all", "origin"])?;
    ensure!(
        one(&["remote", "get-url", "--push", "--all", "origin"])?.eq_ignore_ascii_case(&name),
        "Fetch and push destinations differ"
    );
    Ok(Some(name))
}

type Reply = (u16, Value);
fn accepted(r: Reply) -> Result<Value> {
    ensure!(
        (200..300).contains(&r.0),
        "GitHub refused the backup operation (HTTP {})",
        r.0
    );
    Ok(r.1)
}
fn sha(v: &Value) -> Result<&str> {
    v.as_str()
        .filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .context("GitHub returned an invalid object identity")
}
fn verify_private(repo: &Value, name: &str, id: Option<u64>) -> Result<u64> {
    ensure!(
        repo["private"] == true
            && repo["visibility"] == "private"
            && repo["full_name"]
                .as_str()
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
            && repo["permissions"]["push"] == true,
        "Sync requires a verified private GitHub repository you can write"
    );
    repo["id"]
        .as_u64()
        .filter(|v| *v > 0 && id.is_none_or(|old| old == *v))
        .context("GitHub repository identity changed")
}
/// Upload a new root tree, not a user's local git ancestry. Verify privacy before any
/// content leaves the machine; update only our branch, by non-forced fast-forward.
fn publish(
    files: &[(String, String)],
    origin: Option<&str>,
    mut api: impl FnMut(&str, &str, Option<Value>) -> Result<Reply>,
) -> Result<()> {
    let name = if let Some(o) = origin {
        o.to_owned()
    } else {
        let user = accepted(api("GET", "user", None)?)?;
        let login = user["login"]
            .as_str()
            .filter(|s| valid_component(s))
            .context("No GitHub user")?;
        format!("{login}/scriptkit-sync")
    };
    let base = format!("repos/{name}");
    let mut repo = api("GET", &base, None)?;
    if origin.is_none() && repo.0 == 404 {
        accepted(api(
            "POST",
            "user/repos",
            Some(json!({"name":"scriptkit-sync","private":true,"auto_init":true})),
        )?)?;
        repo = api("GET", &base, None)?;
    }
    let id = verify_private(&accepted(repo)?, &name, None)?;
    let git_base = format!("repositories/{id}/git");
    let head = format!("{git_base}/ref/heads/{BRANCH}");
    let prior = match api("GET", &head, None)? {
        (404, _) => None,
        r => Some(sha(&accepted(r)?["object"]["sha"])?.to_owned()),
    };
    if let Some(prior) = &prior {
        let marker = accepted(api(
            "GET",
            &format!("{base}/contents/{OWNER_MARKER}?ref={prior}"),
            None,
        )?)?;
        use base64::Engine;
        ensure!(
            marker["type"] == "file"
                && marker["encoding"] == "base64"
                && marker["content"]
                    .as_str()
                    .and_then(|s| base64::engine::general_purpose::STANDARD
                        .decode(s.replace('\n', ""))
                        .ok())
                    .as_deref()
                    == Some(MARK.as_bytes()),
            "Refusing to replace a branch not owned by Script Kit backup"
        );
    }
    let mut tree: Vec<_> = files
        .iter()
        .map(|(p, s)| json!({"path":p,"mode":"100644","type":"blob","content":s}))
        .collect();
    tree.push(json!({"path":OWNER_MARKER,"mode":"100644","type":"blob","content":MARK}));
    verify_private(&accepted(api("GET", &base, None)?)?, &name, Some(id))?;
    let native_tree = accepted(api(
        "POST",
        &format!("{git_base}/trees"),
        Some(json!({"tree":tree})),
    )?)?;
    let tree_id = sha(&native_tree["sha"])?;
    if let Some(p) = &prior {
        let current = accepted(api("GET", &format!("{git_base}/commits/{p}"), None)?)?;
        if sha(&current["tree"]["sha"])? == tree_id {
            ensure!(
                accepted(api("GET", &head, None)?)?["object"]["sha"] == p.as_str(),
                "Backup branch changed; retry"
            );
            return Ok(());
        }
    }
    let commit = accepted(api(
        "POST",
        &format!("{git_base}/commits"),
        Some(
            json!({"message":"Back up personal Script Kit files","tree":tree_id,"parents":prior.iter().collect::<Vec<_>>()}),
        ),
    )?)?;
    let new = sha(&commit["sha"])?;
    verify_private(&accepted(api("GET", &base, None)?)?, &name, Some(id))?;
    if prior.is_some() {
        accepted(api(
            "PATCH",
            &format!("{git_base}/refs/heads/{BRANCH}"),
            Some(json!({"sha":new,"force":false})),
        )?)?;
    } else {
        accepted(api(
            "POST",
            &format!("{git_base}/refs"),
            Some(json!({"ref":format!("refs/heads/{BRANCH}"),"sha":new})),
        )?)?;
    }
    ensure!(
        accepted(api("GET", &head, None)?)?["object"]["sha"] == new,
        "Backup tip could not be verified"
    );
    Ok(())
}

/// Only resolve tools from absolute program directories. An empty/relative PATH component
/// must not turn a file in the workspace or backup staging directory into the GitHub client.
fn binary(name: &str, cwd: &Path) -> Result<Command> {
    let active = crate::setup::get_kit_path();
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).chain([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
    ]) {
        if !dir.is_absolute() {
            continue;
        }
        let Ok(tool) = dir.join(name).canonicalize() else {
            continue;
        };
        if tool.starts_with(cwd) || tool.starts_with(&active) || !tool.is_file() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if tool.metadata()?.permissions().mode() & 0o111 == 0 {
                continue;
            }
        }
        return Ok(Command::new(tool));
    }
    bail!("Missing trusted {name} executable")
}
fn git(dir: &Path, args: &[&str]) -> Result<(bool, Vec<u8>)> {
    let mut c = binary("git", dir)?;
    c.args([
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "core.fsmonitor=false",
    ])
    .args(args);
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("GIT_") {
            c.env_remove(name);
        }
    }
    c.env("GIT_TERMINAL_PROMPT", "0");
    command(c, dir, None)
}
fn request(dir: &Path, method: &str, path: &str, body: Option<Value>) -> Result<Reply> {
    let mut c = binary("gh", dir)?;
    c.args([
        "api",
        "--hostname",
        "github.com",
        "--include",
        "--method",
        method,
        path,
    ])
    .env("GH_PROMPT_DISABLED", "1")
    .env_remove("GH_REPO")
    .env_remove("GH_HOST");
    if body.is_some() {
        c.args(["--input", "-"]);
    }
    let b = body.map(|v| serde_json::to_vec(&v)).transpose()?;
    let (ok, out) = command(c, dir, b.as_deref())?;
    let raw = std::str::from_utf8(&out)?.replace("\r\n", "\n");
    let (headers, body) = raw
        .split_once("\n\n")
        .context("GitHub CLI returned no HTTP response")?;
    let code = headers
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .context("Invalid GitHub response")?;
    ensure!(ok || !(200..300).contains(&code), "GitHub CLI failed");
    Ok((code, serde_json::from_str(body)?))
}
fn command(mut c: Command, dir: &Path, body: Option<&[u8]>) -> Result<(bool, Vec<u8>)> {
    let mut input = tempfile::tempfile()?;
    if let Some(b) = body {
        input.write_all(b)?;
        input.rewind()?;
    }
    let mut output = tempfile::tempfile()?;
    let mut child = c
        .current_dir(dir)
        .stdin(input)
        .stdout(Stdio::from(output.try_clone()?))
        .stderr(Stdio::null())
        .spawn()
        .context("Couldn't start git or GitHub CLI")?;
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if start.elapsed() > Duration::from_secs(60) || output.metadata()?.len() > 8 * 1024 * 1024 {
            let _ = child.kill();
            let _ = child.wait();
            bail!("Backup subprocess exceeded its limit");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    ensure!(
        output.metadata()?.len() <= 8 * 1024 * 1024,
        "Backup response too large"
    );
    output.seek(SeekFrom::Start(0))?;
    let mut data = Vec::new();
    output
        .read_to_end(&mut data)
        .map_err(|_| anyhow!("Couldn't read backup result"))?;
    Ok((status.success(), data))
}

#[cfg(test)]
mod tests;

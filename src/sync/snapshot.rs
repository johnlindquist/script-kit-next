//! Build an export from the workspace files, not from its git index or commit history.
use super::git;
use anyhow::{bail, ensure, Context, Result};
use std::{
    fs::File,
    io::Read,
    path::{Component, Path},
};

fn allowed(path: &Path) -> bool {
    let Some(parts) = path
        .components()
        .map(|p| match p {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    if parts.iter().any(|p| {
        let name = p.to_ascii_lowercase();
        p.starts_with('.')
            || p.contains(['\\', '\n', '\r', '\t'])
            || matches!(
                name.as_str(),
                "node_modules"
                    | "db"
                    | "logs"
                    | "cache"
                    | "auth"
                    | "tmp"
                    | "sessions"
                    | "credentials"
                    | "secrets"
                    | "secrets.age"
                    | "agent-token"
            )
            || name.starts_with("secrets.")
            || name.starts_with("credentials.")
    }) {
        return false;
    }
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    match parts.as_slice() {
        ["theme.json"] | ["themes", _] => ext == "json",
        ["plugins", id, "plugin.json"] => super::valid_component(id),
        ["plugins", id, kind, rest @ ..] if super::valid_component(id) && !rest.is_empty() => {
            match *kind {
                "scripts" => matches!(
                    ext,
                    "ts" | "js" | "mjs" | "tsx" | "jsx" | "sh" | "py" | "rb"
                ),
                "skills" | "scriptlets" | "agents" => ext == "md",
                "customizations" => ext == "json",
                _ => false,
            }
        }
        _ => false,
    }
}

// Open every component relative to the last opened directory, disallowing links. An
// attacker racing a parent rename cannot redirect the descriptor outside the workspace.
#[cfg(unix)]
fn read_regular(root: &File, path: &Path) -> Result<String> {
    use std::ffi::CString;
    use std::os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    };
    let mut fd = root.try_clone()?;
    let mut parts = path.components().peekable();
    while let Some(Component::Normal(s)) = parts.next() {
        let name = CString::new(s.as_bytes())?;
        let dir = if parts.peek().is_some() {
            libc::O_DIRECTORY
        } else {
            0
        };
        // SAFETY: name is one validated, NUL-terminated normal component; fd stays live.
        let n = unsafe {
            libc::openat(
                fd.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK | dir,
            )
        };
        if n < 0 {
            bail!("A personal file changed or is an unsafe link; nothing was uploaded");
        }
        // SAFETY: openat returned a new owned descriptor.
        fd = unsafe { File::from_raw_fd(n) };
    }
    let meta = fd.metadata()?;
    ensure!(
        meta.is_file() && meta.nlink() == 1 && meta.len() <= 2_000_000,
        "Personal backup requires ordinary text files under 2 MB"
    );
    let mut text = String::new();
    fd.take(2_000_001)
        .read_to_string(&mut text)
        .context("Non-text personal file in backup")?;
    ensure!(
        text.len() <= 2_000_000 && !text.contains('\0'),
        "Invalid personal file"
    );
    Ok(text)
}
#[cfg(not(unix))]
fn read_regular(_root: &File, _path: &Path) -> Result<String> {
    bail!("Secure personal backup requires Unix")
}

pub(super) fn collect(workspace: &Path, scratch: &Path) -> Result<Vec<(String, String)>> {
    // An empty private index: even a previously force-added vault or old workspace commit
    // can never become part of this backup. Honor the user's ordinary ignore files too.
    let empty = scratch.join("empty.git");
    let dir = empty.to_str().context("Invalid scratch path")?;
    ensure!(
        git(scratch, &["init", "--bare", "--template=", "--quiet", dir])?.0,
        "Couldn't initialize isolated backup index"
    );
    let args = [
        "--git-dir",
        dir,
        "--work-tree",
        workspace.to_str().context("Invalid workspace path")?,
        "ls-files",
        "--others",
        "--exclude-standard",
        "-z",
        "--",
        "plugins",
        "themes",
        "theme.json",
    ];
    let (ok, list) = git(workspace, &args)?;
    ensure!(ok, "Couldn't enumerate personal files");
    let root = File::open(workspace)?;
    let mut files = Vec::new();
    let mut size: usize = 0;
    for item in list.split(|c| *c == 0).filter(|s| !s.is_empty()) {
        let name = std::str::from_utf8(item).context("Non-UTF-8 workspace filename")?;
        if !allowed(Path::new(name)) {
            continue;
        }
        let body = read_regular(&root, Path::new(name))?;
        ensure!(
            !crate::clipboard_history::matches_secret_content(&body)
                && !body.contains("PRIVATE KEY-----"),
            "A personal file resembles a credential; nothing was uploaded"
        );
        size = size.saturating_add(body.len());
        ensure!(
            size <= 16_000_000 && files.len() < 4096,
            "Personal backup exceeds the size limit"
        );
        files.push((name.to_owned(), body));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

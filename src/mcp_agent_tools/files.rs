//! Filesystem tools for agent sessions, confined to the session's path policy.

use std::io::Read as _;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{arg_str, AgentToolPolicy};

const MAX_READ_BYTES: u64 = 2 * 1024 * 1024;
const DEFAULT_READ_LINES: usize = 400;
const MAX_LIST_ENTRIES: usize = 500;
const MAX_FIND_RESULTS: usize = 200;
const MAX_GREP_MATCHES: usize = 200;
const MAX_GREP_FILE_BYTES: u64 = 1024 * 1024;
const MAX_DESKTOP_RESULTS: usize = 30;
const SKIPPED_DIRS: [&str; 5] = [".git", "node_modules", "target", ".build", "dist"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Access {
    Read,
    Write,
}

/// Resolve a tool path against the session cwd and check it against the
/// policy roots. Paths are canonicalized so `..` and symlinks cannot escape.
fn resolve(policy: &AgentToolPolicy, raw: Option<&str>, access: Access) -> Result<PathBuf, String> {
    let raw = raw.unwrap_or(".");
    let expanded = crate::ai::agent_chat::goose::binary::expand_tilde_path(raw);
    let joined = if expanded.is_absolute() {
        expanded
    } else {
        policy.cwd.join(expanded)
    };
    let canonical = match access {
        Access::Read => joined
            .canonicalize()
            .map_err(|error| format!("{}: {error}", joined.display()))?,
        // A file being created does not exist yet: canonicalize its deepest
        // existing ancestor and re-append the rest. Nothing is created until
        // the policy check below passes.
        Access::Write => {
            let mut existing = joined.as_path();
            let mut missing = Vec::new();
            while !existing.exists() {
                let name = existing
                    .file_name()
                    .ok_or_else(|| format!("{} is not a file path", joined.display()))?;
                missing.push(name.to_os_string());
                existing = existing
                    .parent()
                    .ok_or_else(|| format!("{} has no parent directory", joined.display()))?;
            }
            let mut canonical = existing
                .canonicalize()
                .map_err(|error| format!("{}: {error}", existing.display()))?;
            canonical.extend(missing.iter().rev());
            canonical
        }
    };
    let roots = match access {
        Access::Read => policy.read_roots.as_ref(),
        Access::Write => policy.write_roots.as_ref(),
    };
    if let Some(roots) = roots {
        let allowed = roots.iter().any(|root| {
            root.canonicalize()
                .map(|root| canonical.starts_with(root))
                .unwrap_or(false)
        });
        if !allowed {
            let verb = if access == Access::Read {
                "read"
            } else {
                "write"
            };
            return Err(format!(
                "This session is not allowed to {verb} {}.",
                canonical.display()
            ));
        }
    }
    Ok(canonical)
}

fn read_text(path: &Path, limit: u64) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.contains(&0) {
        return Err(format!("{} is a binary file", path.display()));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub(super) fn read(policy: &AgentToolPolicy, args: &Value) -> Result<String, String> {
    let path = resolve(policy, arg_str(args, "path"), Access::Read)?;
    if path.is_dir() {
        return Err(format!("{} is a directory; use ls", path.display()));
    }
    let text = read_text(&path, MAX_READ_BYTES)?;
    let offset = args
        .get("offset")
        .and_then(Value::as_u64)
        .map(|offset| offset.max(1) as usize)
        .unwrap_or(1);
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .map(|limit| limit.clamp(1, 2000) as usize)
        .unwrap_or(DEFAULT_READ_LINES);
    let total = text.lines().count();
    let body: Vec<String> = text
        .lines()
        .enumerate()
        .skip(offset - 1)
        .take(limit)
        .map(|(index, line)| format!("{:>5}\t{line}", index + 1))
        .collect();
    let shown_end = (offset - 1 + body.len()).min(total);
    let mut out = body.join("\n");
    if shown_end < total {
        out.push_str(&format!(
            "\n[lines {offset}-{shown_end} of {total}; read with offset {} for more]",
            shown_end + 1
        ));
    }
    Ok(out)
}

pub(super) fn ls(policy: &AgentToolPolicy, args: &Value) -> Result<String, String> {
    let path = resolve(policy, arg_str(args, "path"), Access::Read)?;
    let mut entries: Vec<String> = std::fs::read_dir(&path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .filter_map(Result::ok)
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect();
    entries.sort();
    let total = entries.len();
    entries.truncate(MAX_LIST_ENTRIES);
    let mut out = format!("{}:\n{}", path.display(), entries.join("\n"));
    if total > MAX_LIST_ENTRIES {
        out.push_str(&format!("\n[{} more entries]", total - MAX_LIST_ENTRIES));
    }
    Ok(out)
}

fn walk(root: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !(entry.file_type().is_dir()
                    && entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| SKIPPED_DIRS.contains(&name)))
        })
        .filter_map(Result::ok)
}

pub(super) fn find(policy: &AgentToolPolicy, args: &Value) -> Result<String, String> {
    let pattern = arg_str(args, "pattern").ok_or("find needs a pattern")?;
    let root = resolve(policy, arg_str(args, "path"), Access::Read)?;
    let glob = globset::GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .map_err(|error| format!("Invalid glob: {error}"))?
        .compile_matcher();
    let matches: Vec<String> = walk(&root)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let relative = entry.path().strip_prefix(&root).ok()?.to_path_buf();
            (glob.is_match(&relative) || glob.is_match(entry.file_name()))
                .then(|| relative.to_string_lossy().into_owned())
        })
        .take(MAX_FIND_RESULTS + 1)
        .collect();
    if matches.is_empty() {
        return Ok(format!(
            "No files under {} match {pattern}.",
            root.display()
        ));
    }
    let truncated = matches.len() > MAX_FIND_RESULTS;
    let mut out = format!(
        "{}:\n{}",
        root.display(),
        matches[..matches.len().min(MAX_FIND_RESULTS)].join("\n")
    );
    if truncated {
        out.push_str("\n[more matches omitted; narrow the pattern]");
    }
    Ok(out)
}

pub(super) fn grep(policy: &AgentToolPolicy, args: &Value) -> Result<String, String> {
    let pattern = arg_str(args, "pattern").ok_or("grep needs a pattern")?;
    let regex = regex::Regex::new(pattern).map_err(|error| format!("Invalid regex: {error}"))?;
    let root = resolve(policy, arg_str(args, "path"), Access::Read)?;
    let glob = arg_str(args, "glob")
        .map(|glob| {
            globset::Glob::new(glob)
                .map(|glob| glob.compile_matcher())
                .map_err(|error| format!("Invalid glob: {error}"))
        })
        .transpose()?;
    let files: Vec<PathBuf> = if root.is_file() {
        vec![root.clone()]
    } else {
        walk(&root)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .metadata()
                    .is_ok_and(|meta| meta.len() <= MAX_GREP_FILE_BYTES)
            })
            .filter(|entry| {
                glob.as_ref()
                    .is_none_or(|glob| glob.is_match(entry.file_name()))
            })
            .map(walkdir::DirEntry::into_path)
            .collect()
    };
    let base = if root.is_file() {
        root.parent().unwrap_or(&root).to_path_buf()
    } else {
        root.clone()
    };
    let mut matches = Vec::new();
    'files: for file in files {
        let Ok(text) = read_text(&file, MAX_GREP_FILE_BYTES) else {
            continue;
        };
        let display = file
            .strip_prefix(&base)
            .unwrap_or(&file)
            .display()
            .to_string();
        for (index, line) in text.lines().enumerate() {
            if regex.is_match(line) {
                let line: String = line.chars().take(300).collect();
                matches.push(format!("{display}:{}: {line}", index + 1));
                if matches.len() > MAX_GREP_MATCHES {
                    break 'files;
                }
            }
        }
    }
    if matches.is_empty() {
        return Ok(format!(
            "No matches for /{pattern}/ under {}.",
            root.display()
        ));
    }
    let truncated = matches.len() > MAX_GREP_MATCHES;
    matches.truncate(MAX_GREP_MATCHES);
    let mut out = matches.join("\n");
    if truncated {
        out.push_str("\n[more matches omitted; narrow the pattern or path]");
    }
    Ok(out)
}

pub(super) fn create_file(policy: &AgentToolPolicy, args: &Value) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(Value::as_str)
        .ok_or("create_file needs content")?;
    let path = resolve(policy, arg_str(args, "path"), Access::Write)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::AlreadyExists => {
                format!(
                    "{} already exists; create_file never overwrites",
                    path.display()
                )
            }
            _ => format!("{}: {error}", path.display()),
        })?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(format!(
        "Created {} ({} bytes).",
        path.display(),
        content.len()
    ))
}

pub(super) fn desktop_search(args: &Value) -> Result<String, String> {
    let query = arg_str(args, "query").ok_or("desktop_search needs a query")?;
    let home = dirs::home_dir().ok_or("No home directory")?;
    let output = std::process::Command::new("/usr/bin/mdfind")
        .arg("-onlyin")
        .arg(&home)
        .arg(query)
        .output()
        .map_err(|error| format!("Spotlight search failed: {error}"))?;
    let results: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(MAX_DESKTOP_RESULTS)
        .map(str::to_string)
        .collect();
    if results.is_empty() {
        return Ok(format!("Spotlight found nothing for \"{query}\"."));
    }
    Ok(results.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy(root: &Path, write: bool) -> AgentToolPolicy {
        AgentToolPolicy {
            allowed_tools: Vec::new(),
            cwd: root.to_path_buf(),
            read_roots: Some(vec![root.to_path_buf()]),
            write_roots: Some(if write {
                vec![root.to_path_buf()]
            } else {
                Vec::new()
            }),
            web_search_budget: None,
        }
    }

    #[test]
    fn reads_are_confined_to_the_policy_roots() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("note.txt"), "one\ntwo\n").unwrap();
        let policy = policy(temp.path(), false);
        let text = read(&policy, &json!({"path": "note.txt"})).unwrap();
        assert!(text.contains("1\tone") && text.contains("2\ttwo"), "{text}");
        let escaped = read(&policy, &json!({"path": "../"}));
        assert!(escaped.is_err());
        assert!(read(&policy, &json!({"path": "/etc/hosts"})).is_err());
    }

    #[test]
    fn create_file_needs_a_write_root_and_never_overwrites() {
        let temp = tempfile::tempdir().unwrap();
        let denied = create_file(
            &policy(temp.path(), false),
            &json!({"path": "a.txt", "content": "x"}),
        );
        assert!(denied.is_err());
        let allowed = policy(temp.path(), true);
        create_file(&allowed, &json!({"path": "sub/a.txt", "content": "x"})).unwrap();
        assert_eq!(
            std::fs::read_to_string(temp.path().join("sub/a.txt")).unwrap(),
            "x"
        );
        assert!(create_file(&allowed, &json!({"path": "sub/a.txt", "content": "y"})).is_err());
    }

    #[test]
    fn find_and_grep_search_below_the_root() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/lib.rs"),
            "fn main() {}\n// TODO: ship\n",
        )
        .unwrap();
        std::fs::create_dir_all(temp.path().join("node_modules")).unwrap();
        std::fs::write(temp.path().join("node_modules/skip.rs"), "TODO").unwrap();
        let policy = policy(temp.path(), false);
        let found = find(&policy, &json!({"pattern": "**/*.rs"})).unwrap();
        assert!(
            found.contains("src/lib.rs") && !found.contains("skip.rs"),
            "{found}"
        );
        let grepped = grep(&policy, &json!({"pattern": "TODO"})).unwrap();
        assert_eq!(
            grepped.lines().collect::<Vec<_>>(),
            vec!["src/lib.rs:2: // TODO: ship"]
        );
    }
}

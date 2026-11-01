//! Migrate the default exclusions without replacing a person's ignore file or following links.
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    path::Path,
};
const BLOCK: &str = "\n# Script Kit private workspace data\n/secrets.age\n/.secrets-*\n/agent-token\n/agent_chat/auth/\n";

pub(super) fn seed(path: &Path, default: &str) -> io::Result<()> {
    let parent = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
    if !path.try_exists()? {
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        temp.write_all(default.as_bytes())?;
        temp.as_file().sync_all()?;
        match temp.persist_noclobber(path) {
            Ok(_) => return Ok(()),
            Err(e) if e.error.kind() == io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.error),
        }
    }
    let mut opts = OpenOptions::new();
    opts.read(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = opts.open(path)?;
    file.try_lock().map_err(io::Error::other)?;
    let m = file.metadata()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.nlink() != 1 {
            return Err(io::ErrorKind::InvalidInput.into());
        }
    }
    if !m.is_file() || m.len() > 1024 * 1024 {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let mut bytes = vec![];
    Read::by_ref(&mut file)
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 1024 * 1024 {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let text = String::from_utf8_lossy(&bytes);
    if ![
        "secrets.age",
        ".secrets-*",
        "agent-token",
        "agent_chat/auth/",
    ]
    .iter()
    .all(|rule| text.lines().any(|l| l.trim_start_matches('/') == *rule))
    {
        file.write_all(BLOCK.as_bytes())?;
        file.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_exclusions_upgrade_once_without_losing_user_rules() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join(".gitignore");
        let user = b"# keep\nmy-personal-notes/";
        std::fs::write(&p, user).unwrap();
        seed(&p, "new default").unwrap();
        let got = std::fs::read(&p).unwrap();
        assert!(got.starts_with(user));
        assert!(got.ends_with(BLOCK.as_bytes()));
        seed(&p, "different default").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), got);
    }
    #[test]
    #[cfg(unix)]
    fn private_exclusions_refuse_indirect_user_file() {
        let d = tempfile::tempdir().unwrap();
        let target = d.path().join("elsewhere");
        std::fs::write(&target, b"mine").unwrap();
        let p = d.path().join(".gitignore");
        std::os::unix::fs::symlink(&target, &p).unwrap();
        assert!(seed(&p, "new").is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"mine");
    }
}

use super::{config_snapshot, confirm_snapshot, ConfigSnapshot, ConfigWriteError, SettingsLock};
use std::io::{self, Write};
use std::path::Path;
use tempfile::NamedTempFile;

#[cfg(unix)]
use std::os::unix::fs::{FileExt, MetadataExt};

fn changed() -> io::Error {
    io::Error::other("Settings draft changed; inspect the settings lock before another save")
}

// Keep the descriptor open until the rename or checked cleanup. A TempPath by itself
// can rename or unlink whatever now occupies its name, even with the original file open.
// These are immediate checks under the settings lock, not an OS compare-and-swap against
// a noncooperating process changing the directory during the final rename or unlink.
pub(super) struct ConfigDraft<'a> {
    temp: Option<NamedTempFile>,
    lock: &'a SettingsLock,
    #[cfg(unix)]
    bytes: &'a [u8],
    #[cfg(unix)]
    owner: (u64, u64, u32, u32, u32),
}

#[cfg(unix)]
fn identity(m: &std::fs::Metadata) -> (u64, u64, u32, u32, u32) {
    (m.dev(), m.ino(), m.mode(), m.uid(), m.gid())
}

impl<'a> ConfigDraft<'a> {
    pub(super) fn prepare(lock: &'a SettingsLock, bytes: &'a [u8]) -> io::Result<Self> {
        lock.confirm()?;
        // Stage inside the private lock on Unix, including macOS. Other targets retain
        // tempfile's previous sibling-file publication and cleanup behavior.
        #[cfg(unix)]
        let dir = lock.path.as_path();
        #[cfg(not(unix))]
        let dir = lock
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let temp = tempfile::Builder::new()
            .prefix(".config-write.")
            .disable_cleanup(cfg!(unix))
            .tempfile_in(dir)?;
        #[cfg(unix)]
        let owner = {
            let m = temp.as_file().metadata()?;
            if !m.is_file() || m.nlink() != 1 || (m.mode() & 0o7777) & !0o600 != 0 {
                return Err(changed());
            }
            identity(&m)
        };
        let mut draft = Self {
            lock,
            temp: Some(temp),
            #[cfg(unix)]
            bytes,
            #[cfg(unix)]
            owner,
        };
        #[cfg(unix)]
        draft.confirm_bytes(&[])?; // before writing any private settings
        let temp = draft.temp.as_mut().ok_or_else(changed)?;
        temp.write_all(bytes)?;
        temp.as_file().sync_all()?;
        Ok(draft)
    }

    #[cfg(unix)]
    fn confirm_bytes(&self, expected: &[u8]) -> io::Result<()> {
        let temp = self.temp.as_ref().ok_or_else(changed)?;
        let check_name = || -> io::Result<(i64, i64)> {
            self.lock.confirm()?;
            let held = temp.as_file().metadata()?;
            let name = std::fs::symlink_metadata(temp.path())?;
            for m in [&held, &name] {
                if identity(m) != self.owner || m.nlink() != 1 || m.len() != expected.len() as u64 {
                    return Err(changed());
                }
            }
            let stamp = (held.ctime(), held.ctime_nsec());
            if (name.ctime(), name.ctime_nsec()) != stamp {
                return Err(changed());
            }
            Ok(stamp)
        };
        let before = check_name()?;
        let mut buf = [0; 8192];
        let mut offset = 0;
        for chunk in expected.chunks(buf.len()) {
            temp.as_file()
                .read_exact_at(&mut buf[..chunk.len()], offset)?;
            if &buf[..chunk.len()] != chunk {
                return Err(changed());
            }
            offset += chunk.len() as u64;
        }
        if check_name()? != before {
            return Err(changed());
        }
        Ok(())
    }

    fn confirm(&self) -> io::Result<()> {
        #[cfg(unix)]
        {
            self.confirm_bytes(self.bytes)
        }
        #[cfg(not(unix))]
        {
            self.lock.confirm()
        }
    }

    pub(super) fn publish_edit(
        self,
        path: &Path,
        original: Option<&ConfigSnapshot>,
    ) -> Result<(), ConfigWriteError> {
        if let Some(old) = original.filter(|s| !s.bytes.is_empty()) {
            let backup = path.with_extension("ts.bak");
            let prior = config_snapshot(&backup)?;
            let backup_draft = ConfigDraft::prepare(self.lock, &old.bytes)?;
            // Detect an already-disturbed candidate before changing either destination.
            // A later failure after the backup rename leaves that valid original backup alone.
            confirm_snapshot(path, Some(old))?;
            self.confirm()?;
            backup_draft.publish(&backup, prior.as_ref())?;
        }
        self.publish(path, original)
    }

    pub(super) fn publish(
        mut self,
        path: &Path,
        expected: Option<&ConfigSnapshot>,
    ) -> Result<(), ConfigWriteError> {
        self.lock.confirm()?;
        confirm_snapshot(path, expected)?;
        self.confirm()?;
        let temp = self.temp.take().ok_or_else(changed)?;
        let result = if expected.is_some() {
            temp.persist(path)
        } else {
            temp.persist_noclobber(path)
        };
        if let Err(failed) = result {
            self.temp = Some(failed.file); // keep the descriptor through cleanup of an ordinary failed rename
            return Err(failed.error.into());
        }
        Ok(())
    }
}

#[cfg(unix)]
impl Drop for ConfigDraft<'_> {
    fn drop(&mut self) {
        use std::io::Seek;
        // A failed write can leave a prefix. Only this descriptor advanced its write offset;
        // positional verification above never moves it. Never unlink a replacement or edited draft.
        let Some(temp) = self.temp.as_mut() else {
            return;
        };
        let Ok(pos) = temp.as_file_mut().stream_position() else {
            return;
        };
        let Ok(n) = usize::try_from(pos) else { return };
        let Some(prefix) = self.bytes.get(..n) else {
            return;
        };
        if self.confirm_bytes(prefix).is_err() {
            return;
        }
        let Some(temp) = self.temp.take() else { return };
        let (_held, name) = temp.into_parts();
        if let Err(e) = name.close() {
            tracing::warn!(kind = ?e.kind(), "Could not remove verified settings draft");
        }
    }
}

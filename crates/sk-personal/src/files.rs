//! All descendant names are constants or generated UUIDs, resolved relative to held directories.
//! A swapped or planted symlink cannot redirect a read, lock or atomic publication.
use super::{Error, Result};
use rustix::fs::{self as at, AtFlags, Mode, OFlags};
use rustix::io::Errno;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub(crate) struct Dir(File);
const OPEN_DIR: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);
fn err(e: Errno) -> Error {
    if e == Errno::LOOP || e == Errno::NOTDIR {
        Error::UnsafePath
    } else {
        Error::StorageUnavailable
    }
}
impl Dir {
    pub fn root(path: &Path) -> Result<Self> {
        if !path.is_absolute() {
            return Err(Error::WrongWorkspace);
        }
        Ok(Self(File::from(
            at::open(path, OPEN_DIR, Mode::empty()).map_err(err)?,
        )))
    }
    pub fn identity(&self) -> Result<(u64, u64)> {
        let m = self.0.metadata().map_err(|_| Error::StorageUnavailable)?;
        Ok((m.dev() as u64, m.ino()))
    }
    pub fn path(&self, parts: &[&str], make: bool) -> Result<Option<Self>> {
        let mut dir = self.0.try_clone().map_err(|_| Error::StorageUnavailable)?;
        for part in parts {
            if make {
                match at::mkdirat(&dir, *part, Mode::RWXU) {
                    Ok(()) => dir.sync_all().map_err(|_| Error::StorageUnavailable)?,
                    Err(Errno::EXIST) => {}
                    Err(e) => return Err(err(e)),
                }
            }
            match at::openat(&dir, *part, OPEN_DIR, Mode::empty()) {
                Ok(next) => dir = File::from(next),
                Err(Errno::NOENT) if !make => return Ok(None),
                Err(e) => return Err(err(e)),
            }
        }
        Ok(Some(Self(dir)))
    }
    pub fn lock(&self) -> Result<()> {
        self.0.try_lock().map_err(|_| Error::WriterBusy)
    }
    pub fn read(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let file = match at::openat(
            &self.0,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Err(Errno::NOENT) => return Ok(None),
            Ok(f) => File::from(f),
            Err(e) => return Err(err(e)),
        };
        let m = file.metadata().map_err(|_| Error::StorageUnavailable)?;
        if !m.is_file() || m.nlink() != 1 || m.len() > 65536 {
            return Err(Error::UnsafePath);
        }
        let mut bytes = Vec::new();
        (&file)
            .take(65537)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::StorageUnavailable)?;
        if bytes.len() > 65536 {
            return Err(Error::UnsafePath);
        }
        Ok(Some(bytes))
    }
    pub fn put(&self, name: &str, bytes: &[u8], exclusive: bool) -> Result<()> {
        if bytes.len() > 65536 {
            return Err(Error::InvalidDocument);
        }
        let tmp = format!(".kit-{}.tmp", uuid::Uuid::new_v4());
        let mut f = File::from(
            at::openat(
                &self.0,
                tmp.as_str(),
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::RUSR | Mode::WUSR,
            )
            .map_err(err)?,
        );
        let result = (|| {
            f.write_all(bytes)
                .and_then(|_| f.sync_all())
                .map_err(|_| Error::StorageUnavailable)?;
            drop(f);
            if exclusive {
                at::linkat(&self.0, tmp.as_str(), &self.0, name, AtFlags::empty()).map_err(
                    |e| {
                        if e == Errno::EXIST {
                            Error::RequestConflict
                        } else {
                            err(e)
                        }
                    },
                )?;
                at::unlinkat(&self.0, tmp.as_str(), AtFlags::empty()).map_err(err)?;
            } else {
                at::renameat(&self.0, tmp.as_str(), &self.0, name).map_err(err)?;
            }
            self.0.sync_all().map_err(|_| Error::StorageUnavailable)
        })();
        if result.is_err() {
            let _ = at::unlinkat(&self.0, tmp.as_str(), AtFlags::empty());
        }
        result
    }
}

//! A v1 import moves user files. Never turn an occupied destination or an
//! unsupported exclusive rename into an ordinary overwriting rename.
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(super) struct Directory {
    path: PathBuf,
    #[cfg(unix)]
    dir: fs::File,
}

impl Directory {
    pub(super) fn open(path: &Path) -> io::Result<Self> {
        if !fs::symlink_metadata(path)?.is_dir() {
            return Err(io::Error::other("Indirect legacy migration directory"));
        }
        Ok(Self {
            path: path.to_owned(),
            #[cfg(unix)]
            dir: {
                use std::os::unix::fs::OpenOptionsExt;
                fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY)
                    .open(path)?
            },
        })
    }

    pub(super) fn path(&self) -> &Path { &self.path }

    pub(super) fn check(&self) -> io::Result<()> {
        let named = fs::symlink_metadata(&self.path)?;
        if !named.is_dir() {
            return Err(io::Error::other("Legacy migration directory changed"));
        }
        #[cfg(unix)]
        if !same_entry(&self.dir.metadata()?, &named) {
            return Err(io::Error::other("Legacy migration directory changed"));
        }
        Ok(())
    }

    pub(super) fn move_entry(
        &self,
        source: &OsStr,
        expected: &fs::Metadata,
        to: &Self,
        dest: &OsStr,
    ) -> io::Result<()> {
        self.check()?;
        to.check()?;
        if !same_entry(expected, &fs::symlink_metadata(self.path.join(source))?) {
            return Err(io::Error::other("Legacy entry changed"));
        }
        self.rename_exclusive(source, to, dest)?;
        self.check()?;
        to.check()?;
        if !same_entry(expected, &fs::symlink_metadata(to.path.join(dest))?) {
            return Err(io::Error::other("Could not confirm the imported entry"));
        }
        Ok(())
    }

    fn rename_exclusive(&self, source: &OsStr, to: &Self, dest: &OsStr) -> io::Result<()> {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            use std::ffi::CString;
            use std::os::fd::AsRawFd;
            use std::os::unix::ffi::OsStrExt;
            // Both names are single components from read_dir or the fixed archive name.
            let source = CString::new(source.as_bytes())?;
            let dest = CString::new(dest.as_bytes())?;
            // SAFETY: the directories stay open and the two C strings outlive the syscall.
            #[cfg(target_os = "macos")]
            let rc = unsafe {
                libc::renameatx_np(self.dir.as_raw_fd(), source.as_ptr(), to.dir.as_raw_fd(), dest.as_ptr(), libc::RENAME_EXCL)
            };
            #[cfg(target_os = "linux")]
            let rc = unsafe {
                libc::syscall(libc::SYS_renameat2, self.dir.as_raw_fd(), source.as_ptr(), to.dir.as_raw_fd(), dest.as_ptr(), libc::RENAME_NOREPLACE)
            };
            if rc != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn MoveFileW(from: *const u16, to: *const u16) -> i32;
            }
            let from: Vec<_> = self.path.join(source).as_os_str().encode_wide().chain([0]).collect();
            let to: Vec<_> = to.path.join(dest).as_os_str().encode_wide().chain([0]).collect();
            // SAFETY: these names came from existing filesystem entries, and MoveFileW
            // takes two terminated strings and refuses any existing destination.
            if unsafe { MoveFileW(from.as_ptr(), to.as_ptr()) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
        {
            let _ = dest;
            Err(io::ErrorKind::Unsupported.into())
        }
    }
}

fn same_entry(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        a.dev() == b.dev() && a.ino() == b.ino() && a.file_type() == b.file_type()
    }
    #[cfg(not(unix))]
    {
        a.file_type() == b.file_type() && a.len() == b.len() && a.modified().ok() == b.modified().ok()
    }
}

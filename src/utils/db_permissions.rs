//! Shared helper for restricting SQLite files (and their WAL/SHM sidecars) to
//! owner-only permissions.
//!
//! Several stores hold sensitive user content — clipboard history, notes, the
//! brain index, AI chat history. SQLite opens files at the process umask
//! (typically `0644`, group/world-readable in `$HOME`), so without this any
//! local process running as the user could read the raw content. Call this
//! right after opening/initializing a DB (WAL mode on) so the sidecars exist.

use std::path::Path;

/// Restrict a SQLite database file and its `-wal`/`-shm` sidecars to owner
/// read/write (`0600`). Best-effort: a looser-permission DB is not worth
/// failing to open the store over, so errors are logged at debug and ignored.
pub(crate) fn harden_sqlite_permissions(db_path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for suffix in ["", "-wal", "-shm"] {
            let path = if suffix.is_empty() {
                db_path.to_path_buf()
            } else {
                let mut os = db_path.as_os_str().to_owned();
                os.push(suffix);
                std::path::PathBuf::from(os)
            };
            if path.exists() {
                if let Err(err) =
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                {
                    tracing::debug!("could not restrict DB permissions on {path:?}: {err}");
                }
            }
        }
    }
    #[cfg(not(unix))]
    let _ = db_path;
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn hardens_db_and_sidecars_to_0600() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().join("secret.sqlite");
        std::fs::write(&db, b"data").expect("write db");
        std::fs::write(dir.path().join("secret.sqlite-wal"), b"wal").expect("write wal");
        // -shm intentionally absent: the helper must tolerate a missing sidecar.

        // Loosen first so we can prove the helper tightens it.
        std::fs::set_permissions(&db, std::fs::Permissions::from_mode(0o644)).unwrap();

        harden_sqlite_permissions(&db);

        let mode = std::fs::metadata(&db).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "db file should be owner-only");
        let wal_mode = std::fs::metadata(dir.path().join("secret.sqlite-wal"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(wal_mode, 0o600, "wal sidecar should be owner-only");
    }
}

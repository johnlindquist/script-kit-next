/// Get the apps database path (~/.scriptkit/db/apps.sqlite)
fn get_apps_db_path() -> PathBuf {
    let kit = PathBuf::from(shellexpand::tilde("~/.scriptkit").as_ref());
    kit.join("db").join("apps.sqlite")
}

/// Initialize the apps database schema
fn init_apps_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS apps (
            bundle_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            icon_blob BLOB,
            mtime INTEGER NOT NULL,
            last_seen INTEGER NOT NULL
        )",
        [],
    )
    .context("Failed to create apps table")?;

    // Index for path lookups (used during directory scan)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_apps_path ON apps(path)", [])
        .context("Failed to create path index")?;

    Ok(())
}

/// Get or initialize the apps database connection
fn get_apps_db() -> Result<Arc<Mutex<Connection>>> {
    if let Some(db) = APPS_DB.get() {
        return Ok(Arc::clone(db));
    }

    let db_path = get_apps_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create db directory")?;
    }

    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open apps database: {}", db_path.display()))?;

    init_apps_db(&conn)?;

    let db = Arc::new(Mutex::new(conn));

    // Try to store it, but another thread might beat us
    match APPS_DB.set(Arc::clone(&db)) {
        Ok(()) => Ok(db),
        Err(_) => {
            // Another thread initialized it first, use theirs
            Ok(Arc::clone(APPS_DB.get().unwrap()))
        }
    }
}

/// Set the current loading state
fn set_loading_state(state: AppLoadingState) {
    let mutex = APP_LOADING_STATE.get_or_init(|| Mutex::new(AppLoadingState::LoadingFromCache));
    if let Ok(mut guard) = mutex.lock() {
        *guard = state;
    }
}

/// Get the current loading state
#[allow(dead_code)]
pub fn get_app_loading_state() -> AppLoadingState {
    APP_LOADING_STATE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|g| *g)
        .unwrap_or(AppLoadingState::Ready)
}

/// Get a human-readable message for the current loading state
#[allow(dead_code)]
pub fn get_app_loading_message() -> &'static str {
    get_app_loading_state().message()
}

/// Check if apps are still loading
#[allow(dead_code)]
pub fn is_apps_loading() -> bool {
    get_app_loading_state() != AppLoadingState::Ready
}

/// Get the in-memory app cache (may be empty if not yet loaded)
#[allow(dead_code)]
pub fn get_cached_apps() -> Vec<AppInfo> {
    APP_CACHE
        .get()
        .and_then(|arc| arc.lock().ok())
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// Get modification time for a path as Unix timestamp
fn get_mtime(path: &Path) -> Option<i64> {
    path.metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

// ============================================================================
// SQLite Cache Operations
// ============================================================================

fn with_apps_db<T>(default: T, f: impl FnOnce(&Connection) -> T) -> T {
    let db = match get_apps_db() {
        Ok(db) => db,
        Err(e) => {
            warn!(error = %e, "Failed to get apps database");
            return default;
        }
    };

    let conn = match db.lock() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to lock apps database");
            return default;
        }
    };

    f(&conn)
}

/// Load all apps from the SQLite cache with icons decoded synchronously.
///
/// Returns apps with their icons already decoded as RenderImages.
/// This is the fast path for startup - no filesystem scanning needed.
fn load_apps_from_db() -> Vec<AppInfo> {
    let _span = info_span!("load_apps_from_db").entered();
    let start = Instant::now();

    with_apps_db(Vec::new(), |conn| {
        let mut stmt = match conn.prepare(
            "SELECT bundle_id, name, path, icon_blob FROM apps ORDER BY name COLLATE NOCASE",
        ) {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "Failed to prepare apps query");
                return Vec::new();
            }
        };

        let apps_iter = stmt.query_map([], |row| {
            let bundle_id: Option<String> = row.get(0)?;
            let name: String = row.get(1)?;
            let path_str: String = row.get(2)?;
            let icon_blob: Option<Vec<u8>> = row.get(3)?;

            Ok((bundle_id, name, path_str, icon_blob))
        });

        let mut apps = Vec::new();
        let mut icons_decoded = 0;

        if let Ok(iter) = apps_iter {
            for (bundle_id, name, path_str, icon_blob) in iter.flatten() {
                let path = PathBuf::from(&path_str);

                // Skip apps that no longer exist
                if !path.exists() {
                    continue;
                }

                // Decode icon synchronously if present
                let icon = icon_blob.and_then(|bytes| {
                    crate::list_item::decode_png_to_render_image_with_bgra_conversion(&bytes).ok()
                });

                if icon.is_some() {
                    icons_decoded += 1;
                }

                apps.push(AppInfo {
                    name,
                    path,
                    bundle_id,
                    icon,
                });
            }
        }

        info!(
            app_count = apps.len(),
            icons_decoded,
            duration_ms = start.elapsed().as_millis(),
            "Loaded apps from DB with icons"
        );

        apps
    })
}

/// Save or update an app in the SQLite cache
fn save_app_to_db(app: &AppInfo, icon_bytes: Option<&[u8]>, mtime: i64) {
    with_apps_db((), |conn| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let path_str = app.path.to_string_lossy().to_string();
        let bundle_id = app.bundle_id.as_deref().unwrap_or(&path_str);

        let result = conn.execute(
            "INSERT INTO apps (bundle_id, name, path, icon_blob, mtime, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(bundle_id) DO UPDATE SET
                 name = excluded.name,
                 path = excluded.path,
                 icon_blob = COALESCE(excluded.icon_blob, apps.icon_blob),
                 mtime = excluded.mtime,
                 last_seen = excluded.last_seen",
            params![bundle_id, app.name, path_str, icon_bytes, mtime, now],
        );

        if let Err(e) = result {
            warn!(error = %e, app = %app.name, "Failed to save app to database");
        }
    });
}

/// Get database statistics for logging
pub fn get_apps_db_stats() -> (usize, u64) {
    with_apps_db((0, 0), |conn| {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM apps", [], |row| row.get(0))
            .unwrap_or(0);

        let total_icon_size: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(LENGTH(icon_blob)), 0) FROM apps WHERE icon_blob IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        (count as usize, total_icon_size as u64)
    })
}

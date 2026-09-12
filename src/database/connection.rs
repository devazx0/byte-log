use crate::error::{PcStatsError, Result};
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct DbConnection;

impl DbConnection {
    /// Opens a SQLite connection to the specified database path with mandatory WAL pragmas.
    /// Recovers automatically if database corruption is detected.
    pub fn open(db_path: &Path) -> Result<Connection> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        match Self::try_open_and_configure(db_path) {
            Ok(conn) => Ok(conn),
            Err(e) => {
                warn!(
                    "Database error during open ({}). Checking for corruption...",
                    e
                );
                if Self::is_corrupt(&e) || !Self::quick_check(db_path) {
                    Self::recover_corrupt_db(db_path)?;
                    Self::try_open_and_configure(db_path)
                } else {
                    Err(e)
                }
            }
        }
    }

    fn try_open_and_configure(db_path: &Path) -> Result<Connection> {
        let conn = Connection::open(db_path).map_err(PcStatsError::Database)?;

        // Apply mandatory PRAGMAs per PRD Sec 27
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(PcStatsError::Database)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(PcStatsError::Database)?;
        conn.pragma_update(None, "busy_timeout", 5000)
            .map_err(PcStatsError::Database)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(PcStatsError::Database)?;
        conn.pragma_update(None, "cache_size", -2000)
            .map_err(PcStatsError::Database)?;
        conn.pragma_update(None, "auto_vacuum", "INCREMENTAL")
            .map_err(PcStatsError::Database)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(db_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = fs::set_permissions(db_path, perms);
            }
        }

        Ok(conn)
    }

    fn is_corrupt(err: &PcStatsError) -> bool {
        match err {
            PcStatsError::Database(rusqlite::Error::SqliteFailure(err, _)) => {
                err.code == rusqlite::ffi::ErrorCode::DatabaseCorrupt
                    || err.extended_code == 11 // SQLITE_CORRUPT
                    || err.extended_code == 26 // SQLITE_NOTADB
            }
            _ => false,
        }
    }

    fn quick_check(db_path: &Path) -> bool {
        if !db_path.exists() {
            return true;
        }
        if let Ok(conn) = Connection::open(db_path) {
            let res: std::result::Result<String, _> =
                conn.query_row("PRAGMA quick_check(1);", [], |row| row.get(0));
            match res {
                Ok(status) => status == "ok",
                Err(_) => false,
            }
        } else {
            false
        }
    }

    fn recover_corrupt_db(db_path: &Path) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let corrupt_path = db_path.with_extension(format!("corrupt.{}", timestamp));
        error!(
            "SQLite database is corrupt! Moving {} to {} and initializing fresh database.",
            db_path.display(),
            corrupt_path.display()
        );

        if db_path.exists() {
            fs::rename(db_path, &corrupt_path)?;
        }
        let wal_path = db_path.with_extension("db-wal");
        if wal_path.exists() {
            let _ = fs::remove_file(wal_path);
        }
        let shm_path = db_path.with_extension("db-shm");
        if shm_path.exists() {
            let _ = fs::remove_file(shm_path);
        }

        info!("Corrupt database quarantined. Fresh database will be created.");
        Ok(())
    }

    /// Executes an operation with 3-stage exponential backoff (25ms, 75ms, 200ms) on SQLITE_BUSY.
    pub fn retry_on_busy<F, T>(mut op: F) -> Result<T>
    where
        F: FnMut() -> std::result::Result<T, rusqlite::Error>,
    {
        let backoffs = [
            Duration::from_millis(25),
            Duration::from_millis(75),
            Duration::from_millis(200),
        ];
        let mut attempt = 0;

        loop {
            match op() {
                Ok(val) => return Ok(val),
                Err(rusqlite::Error::SqliteFailure(err, _))
                    if err.code == rusqlite::ffi::ErrorCode::DatabaseBusy =>
                {
                    if attempt < backoffs.len() {
                        thread::sleep(backoffs[attempt]);
                        attempt += 1;
                    } else {
                        return Err(PcStatsError::Database(rusqlite::Error::SqliteFailure(
                            err, None,
                        )));
                    }
                }
                Err(e) => return Err(PcStatsError::Database(e)),
            }
        }
    }

    pub fn incremental_vacuum(conn: &Connection) -> Result<()> {
        conn.execute("PRAGMA incremental_vacuum(1000);", [])
            .map_err(PcStatsError::Database)?;
        Ok(())
    }
}

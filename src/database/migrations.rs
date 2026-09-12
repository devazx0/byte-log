use crate::error::{PcStatsError, Result};
use rusqlite::Connection;
use tracing::info;

const V01_SQL: &str = include_str!("../../migrations/V01__init.sql");
const V02_SQL: &str = include_str!("../../migrations/V02__features.sql");

pub struct Migrator;

impl Migrator {
    pub fn apply_all(conn: &mut Connection) -> Result<()> {
        let tx = conn.transaction().map_err(PcStatsError::Database)?;

        // Ensure schema_migrations table exists
        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                description TEXT NOT NULL
            );",
            [],
        )
        .map_err(PcStatsError::Database)?;

        let current_version: Option<i64> = tx
            .query_row("SELECT MAX(version) FROM schema_migrations;", [], |row| {
                row.get(0)
            })
            .unwrap_or(None);

        let latest = current_version.unwrap_or(0);

        if latest < 1 {
            info!("Applying database migration V01__init.sql...");
            tx.execute_batch(V01_SQL).map_err(PcStatsError::Database)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, description) VALUES (1, 'V01__init.sql');",
                [],
            )
            .map_err(PcStatsError::Database)?;
            info!("Migration V01__init.sql applied successfully.");
        }

        if latest < 2 {
            info!("Applying database migration V02__features.sql...");
            tx.execute_batch(V02_SQL).map_err(PcStatsError::Database)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, description) VALUES (2, 'V02__features.sql');",
                [],
            )
            .map_err(PcStatsError::Database)?;
            info!("Migration V02__features.sql applied successfully.");
        }

        tx.commit().map_err(PcStatsError::Database)?;
        Ok(())
    }
}

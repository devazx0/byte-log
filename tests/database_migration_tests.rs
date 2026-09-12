use bytelog::database::{DbConnection, Migrator, Repository};
use std::fs;

#[test]
fn test_database_migrations_and_pragmas() {
    let temp_dir = std::env::temp_dir().join("bytelog_test_db");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test_migration.db");
    let _ = fs::remove_file(&db_path);

    // 1. Open and apply migrations
    let mut conn = DbConnection::open(&db_path).expect("Failed to open connection");
    Migrator::apply_all(&mut conn).expect("Failed to apply migrations");

    // 2. Verify PRAGMAs
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
        .expect("Query journal_mode");
    assert_eq!(journal_mode.to_lowercase(), "wal");

    let foreign_keys: i64 = conn
        .query_row("PRAGMA foreign_keys;", [], |row| row.get(0))
        .expect("Query foreign_keys");
    assert_eq!(foreign_keys, 1);

    // 3. Verify schema migrations table
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations;", [], |row| {
            row.get(0)
        })
        .expect("Query version");
    assert_eq!(version, 2);

    // Verify projects table exists
    let projects_table_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='projects';",
            [],
            |row| row.get(0),
        )
        .expect("Query projects table");
    assert_eq!(projects_table_count, 1);

    // 4. Test Repository instantiation
    let repo = Repository::open(&db_path).expect("Open repository");
    let records = repo.get_personal_records().expect("Query records");
    assert!(records.is_empty());

    let _ = fs::remove_file(&db_path);
}

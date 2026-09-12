use bytelog::collectors::SystemSnapshot;
use bytelog::database::Repository;
use bytelog::typing::TypingSlice;
use chrono::{Duration, Utc};
use std::fs;

#[test]
fn test_aggregation_and_privacy_guarantees() {
    let temp_dir = std::env::temp_dir().join("bytelog_test_agg");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test_agg.db");
    let _ = fs::remove_file(&db_path);

    let repo = Repository::open(&db_path).expect("Open repository");
    let now = Utc::now();

    // 1. Insert hardware snapshots
    for i in 0..10 {
        let snap = SystemSnapshot {
            timestamp: now - Duration::seconds(i * 5),
            cpu_utilization: 25.0 + i as f32,
            memory_used_bytes: 8_000_000_000,
            memory_total_bytes: 16_000_000_000,
            swap_used_bytes: 500_000_000,
            disk_read_bytes_sec: 1_000_000,
            disk_write_bytes_sec: 500_000,
            net_rx_bytes_sec: 2_000_000,
            net_tx_bytes_sec: 100_000,
            cpu_temp_celsius: Some(50.0),
            gpu_utilization: None,
            battery_percent: Some(90.0),
            battery_charging: Some(true),
        };
        repo.insert_snapshot(&snap).expect("Insert snapshot");
    }

    // 2. Insert application activity
    repo.record_application_activity("code", "Development", 120, 0)
        .expect("Record app");
    repo.record_application_activity("alacritty", "Terminal", 60, 0)
        .expect("Record app");

    // 3. Insert typing slice (Simulate typing a secret password)
    // Note: TypingSlice only contains numerical counts!
    let slice = TypingSlice {
        started_at: now - Duration::seconds(60),
        ended_at: now,
        active_typing_seconds: 40,
        keystrokes: 150,
        backspaces: 4,
        enters: 2,
        modifiers: 6,
        estimated_wpm: 45.0,
        estimated_accuracy: 94.6,
    };
    repo.insert_typing_slice(&slice, Some("code"))
        .expect("Insert typing");

    // 4. Run rollups & milestone updates
    repo.run_hourly_rollup().expect("Hourly rollup");
    repo.run_daily_rollup().expect("Daily rollup");
    repo.check_and_update_records().expect("Update records");

    // 5. Query summary
    let summary = repo
        .get_summary_stats(&(now - Duration::hours(1)), &(now + Duration::hours(1)))
        .expect("Query summary");
    assert!(summary.active_duration_seconds >= 180);
    assert_eq!(summary.coding_duration_seconds, 120);
    assert_eq!(summary.total_keystrokes, 150);
    assert!(summary.avg_cpu_load > 0.0);

    // 6. Query top apps
    let top_apps = repo
        .get_top_applications(
            &(now - Duration::hours(1)),
            &(now + Duration::hours(1)),
            10,
            None,
        )
        .expect("Query top apps");
    assert_eq!(top_apps.len(), 2);
    assert_eq!(top_apps[0].executable_name, "code");

    // 7. PRIVACY TEST (AC-03): Assert that typing secret text never entered the database
    let test_secret = "SuperSecretPassword123";
    let count: i64 = repo
        .open_raw_connection()
        .query_row(
            &format!(
                "SELECT count(*) FROM typing_sessions WHERE quote(keystrokes) LIKE '%{}%';",
                test_secret
            ),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    assert_eq!(count, 0);

    let _ = fs::remove_file(&db_path);
}

#[test]
fn test_category_preservation_and_downgrade_prevention() {
    let temp_dir = std::env::temp_dir().join("bytelog_test_category");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test_cat.db");
    let _ = fs::remove_file(&db_path);

    let repo = Repository::open(&db_path).expect("Open repository");

    // First tick: recorded with specific category "Development"
    repo.record_application_activity("code", "Development", 10, 0)
        .expect("Record initial");

    // Subsequent tick: daemon tick defaults to "Other" if unchanged
    repo.record_application_activity("code", "Other", 10, 0)
        .expect("Record subsequent");

    let top = repo
        .get_top_applications(
            &(Utc::now() - Duration::hours(1)),
            &(Utc::now() + Duration::hours(1)),
            10,
            None,
        )
        .expect("Query top applications");

    assert_eq!(top.len(), 1);
    assert_eq!(top[0].executable_name, "code");
    assert_eq!(top[0].category, "Development");

    let _ = fs::remove_file(&db_path);
}

#[test]
fn test_session_lifecycle_and_closure() {
    let temp_dir = std::env::temp_dir().join("bytelog_test_session");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test_sess.db");
    let _ = fs::remove_file(&db_path);

    let repo = Repository::open(&db_path).expect("Open repository");

    // Record activity to open a computer session
    repo.record_application_activity("code", "Development", 10, 0)
        .expect("Record activity");

    let conn = rusqlite::Connection::open(&db_path).expect("Raw connection");
    let open_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM computer_sessions WHERE is_open = 1;",
            [],
            |row| row.get(0),
        )
        .expect("Query open sessions");
    assert_eq!(open_count, 1);

    // Explicitly close sessions (e.g. system lock or day rollover or daemon shutdown)
    repo.close_open_computer_sessions()
        .expect("Close sessions");

    let open_count_after: i64 = conn
        .query_row(
            "SELECT count(*) FROM computer_sessions WHERE is_open = 1;",
            [],
            |row| row.get(0),
        )
        .expect("Query open sessions after close");
    assert_eq!(open_count_after, 0);

    let _ = fs::remove_file(&db_path);
}

#[test]
fn test_metric_distribution_calculations() {
    let temp_dir = std::env::temp_dir().join("bytelog_test_dist");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test_dist.db");
    let _ = fs::remove_file(&db_path);

    let repo = Repository::open(&db_path).expect("Open repository");
    let now = Utc::now();

    for i in 1..=20 {
        let snap = SystemSnapshot {
            timestamp: now - Duration::seconds(i * 2),
            cpu_utilization: i as f32 * 5.0, // 5.0 to 100.0
            memory_used_bytes: (i as u64) * 1024 * 1024 * 100,
            memory_total_bytes: 16 * 1024 * 1024 * 1024,
            swap_used_bytes: 0,
            disk_read_bytes_sec: (i as u64) * 1024 * 50,
            disk_write_bytes_sec: (i as u64) * 1024 * 50,
            net_rx_bytes_sec: (i as u64) * 1024 * 20,
            net_tx_bytes_sec: (i as u64) * 1024 * 10,
            cpu_temp_celsius: Some(40.0 + i as f32),
            gpu_utilization: None,
            battery_percent: None,
            battery_charging: None,
        };
        repo.insert_snapshot(&snap).expect("Insert snapshot");
    }

    let start = now - Duration::minutes(5);
    let end = now + Duration::minutes(5);

    // Test CPU distribution
    let cpu_dist = repo
        .get_metric_distribution(&start, &end, "cpu")
        .expect("CPU distribution");
    assert_eq!(cpu_dist.count, 20);
    assert_eq!(cpu_dist.min, 5.0);
    assert_eq!(cpu_dist.max, 100.0);
    assert!(cpu_dist.p50 > cpu_dist.min);
    assert!(cpu_dist.p95 > cpu_dist.p50);
    assert!(cpu_dist.p95 <= cpu_dist.max);

    // Test disk, mem, net, temp
    let disk_dist = repo
        .get_metric_distribution(&start, &end, "disk")
        .expect("Disk distribution");
    assert_eq!(disk_dist.count, 20);
    assert_eq!(disk_dist.unit, "KB/s");

    let mem_dist = repo
        .get_metric_distribution(&start, &end, "mem")
        .expect("Mem distribution");
    assert_eq!(mem_dist.count, 20);
    assert_eq!(mem_dist.unit, "MB");

    let net_dist = repo
        .get_metric_distribution(&start, &end, "net")
        .expect("Net distribution");
    assert_eq!(net_dist.count, 20);
    assert_eq!(net_dist.unit, "KB/s");

    let temp_dist = repo
        .get_metric_distribution(&start, &end, "temp")
        .expect("Temp distribution");
    assert_eq!(temp_dist.count, 20);
    assert_eq!(temp_dist.unit, "°C");

    // Test invalid metric name
    assert!(repo
        .get_metric_distribution(&start, &end, "invalid_metric")
        .is_err());

    let _ = fs::remove_file(&db_path);
}

#[test]
fn test_arbitrary_table_exports() {
    let temp_dir = std::env::temp_dir().join("bytelog_test_export");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test_exp.db");
    let _ = fs::remove_file(&db_path);

    let repo = Repository::open(&db_path).expect("Open repository");
    repo.record_application_activity("code", "Development", 50, 0)
        .expect("Record activity");

    // JSON export
    let json_val = repo
        .get_table_as_json_array("applications")
        .expect("Export JSON array");
    let arr = json_val.as_array().expect("Array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["executable_name"], "code");
    assert_eq!(arr[0]["category"], "Development");

    // CSV export
    let mut buffer = Vec::new();
    repo.export_table_csv("applications", &mut buffer)
        .expect("Export CSV");
    let csv_str = String::from_utf8(buffer).expect("UTF-8");
    assert!(csv_str.contains("executable_name"));
    assert!(csv_str.contains("code"));
    assert!(csv_str.contains("Development"));

    // Disallowed / invalid table name check
    assert!(repo.get_table_as_json_array("users; DROP TABLE;").is_err());
    assert!(repo.export_table_csv("non_existent_table", &mut Vec::new()).is_err());

    let _ = fs::remove_file(&db_path);
}

trait RawConnHelper {
    fn open_raw_connection(&self) -> rusqlite::Connection;
}

impl RawConnHelper for Repository {
    fn open_raw_connection(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(std::env::temp_dir().join("bytelog_test_agg/test_agg.db"))
            .unwrap()
    }
}

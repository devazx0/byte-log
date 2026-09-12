use bytelog::database::repository::HeatmapMetric;
use bytelog::database::Repository;
use chrono::{Duration, Utc};
use std::fs;

#[test]
fn test_heatmap_data_aggregation() {
    let temp_dir = std::env::temp_dir().join("bytelog_test_heatmap");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test_heatmap.db");
    let _ = fs::remove_file(&db_path);

    let repo = Repository::open(&db_path).expect("Open repository");
    let now = Utc::now();
    let year = now.format("%Y").to_string().parse::<i32>().unwrap();

    // 1. Record application activity across multiple days
    repo.record_application_activity("code", "Development", 7200, 0)
        .expect("Record dev app");
    repo.record_application_activity("firefox", "Browser", 3600, 0)
        .expect("Record browser app");

    // 2. Record project activity
    repo.record_project_activity("byte-log", "code", 7200)
        .expect("Record project");

    // 3. Run rollups
    repo.run_daily_rollup().expect("Daily rollup");

    // 4. Query heatmap data for active time
    let active_days = repo
        .get_heatmap_data(year, HeatmapMetric::Active)
        .expect("Get active heatmap");
    assert!(!active_days.is_empty());
    assert!(active_days[0].value > 0.0);

    // 5. Query heatmap data for coding time
    let coding_days = repo
        .get_heatmap_data(year, HeatmapMetric::Coding)
        .expect("Get coding heatmap");
    assert!(!coding_days.is_empty());
    assert_eq!(coding_days[0].value, 7200.0);

    // 6. Query top projects
    let projects = repo
        .get_top_projects(&(now - Duration::days(1)), &(Utc::now() + Duration::seconds(60)), 10)
        .expect("Get top projects");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].project_name, "byte-log");
    assert_eq!(projects[0].active_seconds, 7200);
}

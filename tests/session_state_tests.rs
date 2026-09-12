use bytelog::activity::{Categorizer, SessionManager, SessionState};
use bytelog::platform::traits::ActiveWindowInfo;
use chrono::{Duration as ChronoDuration, Utc};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_idle_transition_at_threshold() {
    let mut rules = HashMap::new();
    rules.insert("code".to_string(), "Development".to_string());
    let categorizer = Categorizer::new(rules);

    // Set 300s idle threshold
    let mut manager = SessionManager::new(categorizer, 300);

    let win = Some(ActiveWindowInfo {
        process_id: 1234,
        executable_name: "code".to_string(),
        window_title: Some("main.rs".to_string()),
    });

    let now = Utc::now();

    // 1. User active (< 300s idle)
    let tick1 = manager.handle_tick(win.clone(), Duration::from_secs(5), false, now);
    assert_eq!(tick1.state, SessionState::Active);
    assert_eq!(tick1.app_name.as_deref(), Some("code"));
    assert_eq!(tick1.category, "Development");
    assert_eq!(tick1.idle_seconds, 0);

    // 2. User goes idle (>= 300s idle)
    let tick2 = manager.handle_tick(
        win,
        Duration::from_secs(305),
        false,
        now + ChronoDuration::seconds(5),
    );
    assert_eq!(tick2.state, SessionState::Idle);
    assert_eq!(tick2.active_seconds, 0);

    // 3. User becomes active again
    let tick3 = manager.handle_tick(
        None,
        Duration::from_secs(0),
        false,
        now + ChronoDuration::seconds(10),
    );
    assert_eq!(tick3.state, SessionState::Active);
}

#[test]
fn test_midnight_rollover() {
    let categorizer = Categorizer::new(HashMap::new());
    let mut manager = SessionManager::new(categorizer, 300);

    let t1 = Utc::now();
    let t2 = t1 + ChronoDuration::days(1); // Next day

    let tick1 = manager.handle_tick(None, Duration::from_secs(0), false, t1);
    assert!(!tick1.rollover);

    let tick2 = manager.handle_tick(None, Duration::from_secs(0), false, t2);
    assert!(tick2.rollover); // Rollover triggered on date change
}

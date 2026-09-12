use crate::activity::categorizer::Categorizer;
use crate::platform::traits::ActiveWindowInfo;
use chrono::{DateTime, NaiveDate, Utc};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Idle,
    Locked,
}

#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub computer_session_id: Option<i64>,
    pub app_session_id: Option<i64>,
    pub app_name: String,
    pub category: String,
    pub active_duration_seconds: u64,
    pub idle_duration_seconds: u64,
    pub is_rollover: bool,
}

pub struct SessionManager {
    categorizer: Categorizer,
    idle_threshold: Duration,
    state: SessionState,
    current_app: Option<String>,
    current_date: NaiveDate,
    last_tick: Instant,
    active_elapsed_secs: u64,
    idle_elapsed_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickResult {
    pub state: SessionState,
    pub app_name: Option<String>,
    pub category: String,
    pub active_seconds: u64,
    pub idle_seconds: u64,
    pub rollover: bool,
    pub state_changed: bool,
}

impl SessionManager {
    pub fn new(categorizer: Categorizer, idle_threshold_seconds: u64) -> Self {
        let now_utc = Utc::now();
        Self {
            categorizer,
            idle_threshold: Duration::from_secs(idle_threshold_seconds),
            state: SessionState::Active,
            current_app: None,
            current_date: now_utc.date_naive(),
            last_tick: Instant::now(),
            active_elapsed_secs: 0,
            idle_elapsed_secs: 0,
        }
    }

    pub fn handle_tick(
        &mut self,
        active_window: Option<ActiveWindowInfo>,
        idle_duration: Duration,
        is_locked: bool,
        current_timestamp: DateTime<Utc>,
    ) -> TickResult {
        let now_instant = Instant::now();
        let elapsed = now_instant.duration_since(self.last_tick).as_secs();
        self.last_tick = now_instant;

        let today = current_timestamp.date_naive();
        let mut rollover_occurred = false;

        if today != self.current_date {
            self.current_date = today;
            rollover_occurred = true;
        }

        let previous_state = self.state;

        if is_locked {
            self.state = SessionState::Locked;
            self.current_app = None;
            return TickResult {
                state: SessionState::Locked,
                app_name: None,
                category: "System".to_string(),
                active_seconds: 0,
                idle_seconds: 0,
                rollover: rollover_occurred,
                state_changed: previous_state != SessionState::Locked,
            };
        }

        if idle_duration >= self.idle_threshold {
            self.state = SessionState::Idle;
            self.idle_elapsed_secs += elapsed;
            return TickResult {
                state: SessionState::Idle,
                app_name: Some("idle".to_string()),
                category: "System".to_string(),
                active_seconds: 0,
                idle_seconds: elapsed,
                rollover: rollover_occurred,
                state_changed: previous_state != SessionState::Idle,
            };
        }

        // Active state
        self.state = SessionState::Active;
        self.active_elapsed_secs += elapsed;

        let app_name = active_window
            .map(|w| w.executable_name)
            .unwrap_or_else(|| "unknown".to_string());

        let category = self.categorizer.categorize(&app_name);
        self.current_app = Some(app_name.clone());

        TickResult {
            state: SessionState::Active,
            app_name: Some(app_name),
            category,
            active_seconds: elapsed,
            idle_seconds: 0,
            rollover: rollover_occurred,
            state_changed: previous_state != SessionState::Active,
        }
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn active_elapsed_secs(&self) -> u64 {
        self.active_elapsed_secs
    }

    pub fn idle_elapsed_secs(&self) -> u64 {
        self.idle_elapsed_secs
    }

    pub fn current_app(&self) -> Option<&str> {
        self.current_app.as_deref()
    }
}

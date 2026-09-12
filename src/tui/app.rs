use crate::collectors::{SystemSnapshot, SystemTelemetryCollector};
use crate::database::repository::{AppUsageStat, SummaryStats};
use crate::database::Repository;
use crate::platform::{create_platform, Platform};
use chrono::{Duration, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeHorizon {
    Today,
    Week,
    Month,
}

pub const CATEGORIES: &[&str] = &[
    "Development",
    "Terminal",
    "Browser",
    "Communication",
    "Productivity",
    "Media",
    "Gaming",
    "System",
    "Other",
];

#[derive(Debug, Clone)]
pub struct CategoryModalState {
    pub app_name: String,
    pub current_category: String,
    pub selected_index: usize,
}

pub struct App {
    pub db_path: PathBuf,
    pub horizon: TimeHorizon,
    pub live_snapshot: Option<SystemSnapshot>,
    pub summary_stats: Option<SummaryStats>,
    pub top_apps: Vec<AppUsageStat>,
    pub cpu_history: Vec<u64>,
    pub app_scroll_index: usize,
    pub active_pane: usize, // 0 = Left, 1 = Right, 2 = Bottom
    pub category_modal: Option<CategoryModalState>,
    pub running: bool,
    collector: SystemTelemetryCollector,
    platform: Platform,
}

impl App {
    pub fn new(db_path: PathBuf) -> Self {
        let platform = create_platform();
        let collector = SystemTelemetryCollector::new();

        let mut app = Self {
            db_path,
            horizon: TimeHorizon::Today,
            live_snapshot: None,
            summary_stats: None,
            top_apps: Vec::new(),
            cpu_history: Vec::with_capacity(60),
            app_scroll_index: 0,
            active_pane: 0,
            category_modal: None,
            running: true,
            collector,
            platform,
        };

        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        let snap = self.collector.collect_snapshot(&self.platform);
        let cpu_val = snap.cpu_utilization.round() as u64;

        if self.cpu_history.len() >= 60 {
            self.cpu_history.remove(0);
        }
        self.cpu_history.push(cpu_val);
        self.live_snapshot = Some(snap);

        // Refresh database queries
        if let Ok(repo) = Repository::open(&self.db_path) {
            let now = Utc::now();
            let (start, end) = match self.horizon {
                TimeHorizon::Today => {
                    let s = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                    (s, now)
                }
                TimeHorizon::Week => (now - Duration::days(7), now),
                TimeHorizon::Month => (now - Duration::days(30), now),
            };

            self.summary_stats = repo.get_summary_stats(&start, &end).ok();
            self.top_apps = repo
                .get_top_applications(&start, &end, 15, None)
                .unwrap_or_default();
        }
    }

    pub fn next_horizon(&mut self) {
        self.horizon = match self.horizon {
            TimeHorizon::Today => TimeHorizon::Week,
            TimeHorizon::Week => TimeHorizon::Month,
            TimeHorizon::Month => TimeHorizon::Today,
        };
        self.refresh();
    }

    pub fn set_horizon(&mut self, horizon: TimeHorizon) {
        self.horizon = horizon;
        self.refresh();
    }

    pub fn scroll_apps_down(&mut self) {
        if !self.top_apps.is_empty() && self.app_scroll_index + 1 < self.top_apps.len() {
            self.app_scroll_index += 1;
        }
    }

    pub fn scroll_apps_up(&mut self) {
        if self.app_scroll_index > 0 {
            self.app_scroll_index -= 1;
        }
    }

    pub fn cycle_focus(&mut self) {
        self.active_pane = (self.active_pane + 1) % 3;
    }

    pub fn open_category_modal(&mut self) {
        if !self.top_apps.is_empty() && self.app_scroll_index < self.top_apps.len() {
            let app_stat = &self.top_apps[self.app_scroll_index];
            let current = app_stat.category.clone();
            let selected_index = CATEGORIES
                .iter()
                .position(|&c| c.eq_ignore_ascii_case(&current))
                .unwrap_or(0);
            self.category_modal = Some(CategoryModalState {
                app_name: app_stat.executable_name.clone(),
                current_category: current,
                selected_index,
            });
        }
    }

    pub fn close_category_modal(&mut self) {
        self.category_modal = None;
    }

    pub fn apply_category(&mut self, new_category: &str) {
        if let Some(modal) = self.category_modal.take() {
            // 1. Update SQLite
            if let Ok(repo) = Repository::open(&self.db_path) {
                let _ = repo.update_application_category(&modal.app_name, new_category);
            }
            // 2. Update config.toml
            if let Ok(mut config) = crate::config::load_config(None) {
                config
                    .categories
                    .insert(modal.app_name.clone(), new_category.to_string());
                let _ = crate::config::save_config(&config, None);
            }
            // 3. Refresh views immediately
            self.refresh();
        }
    }
}

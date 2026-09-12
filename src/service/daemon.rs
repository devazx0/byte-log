use crate::activity::{Categorizer, SessionManager};
use crate::collectors::{SystemSnapshot, SystemTelemetryCollector};
use crate::config::Config;
use crate::database::Repository;
use crate::error::Result;
use crate::platform::create_platform;
use crate::platform::traits::*;
use crate::typing::{TypingSlice, TypingState};
use chrono::Utc;
use crossbeam_channel::{bounded, Receiver, Sender};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

pub enum DaemonMessage {
    Telemetry(SystemSnapshot),
    Activity {
        app_name: String,
        category: String,
        active_seconds: u64,
        idle_seconds: u64,
    },
    ProjectActivity {
        project_name: String,
        app_name: String,
        active_seconds: u64,
    },
    Typing(TypingSlice, Option<String>),
    SessionClose,
    Shutdown,
}

pub struct Daemon {
    config: Config,
    db_path: PathBuf,
}

impl Daemon {
    pub fn new(config: Config, db_path: PathBuf) -> Self {
        Self { config, db_path }
    }

    pub fn run(&self) -> Result<()> {
        info!(
            "Starting ByteLog daemon with database: {}",
            self.db_path.display()
        );

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            info!("Received shutdown signal. Stopping ByteLog daemon...");
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        let (tx, rx): (Sender<DaemonMessage>, Receiver<DaemonMessage>) = bounded(1000);

        // Run initial startup rollup and retention cleanup
        {
            if let Ok(repo) = Repository::open(&self.db_path) {
                let _ = repo.run_hourly_rollup();
                let _ = repo.run_daily_rollup();
                let _ = repo.prune_retention(
                    self.config.retention.retention_raw_days,
                    self.config.retention.retention_hourly_days,
                );
                let _ = repo.check_and_update_records();
            }
        }

        // Writer Thread
        let writer_db_path = self.db_path.clone();
        let writer_running = running.clone();
        let writer_handle = thread::spawn(move || {
            let repo = match Repository::open(&writer_db_path) {
                Ok(r) => r,
                Err(e) => {
                    error!("Writer thread failed to open database: {}", e);
                    return;
                }
            };

            let mut last_hourly_rollup = Instant::now();
            let mut last_daily_rollup = Instant::now();

            while writer_running.load(Ordering::SeqCst) || !rx.is_empty() {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(DaemonMessage::Telemetry(snapshot)) => {
                        let _ = repo.insert_snapshot(&snapshot);
                    }
                    Ok(DaemonMessage::Activity {
                        app_name,
                        category,
                        active_seconds,
                        idle_seconds,
                    }) => {
                        let _ = repo.record_application_activity(
                            &app_name,
                            &category,
                            active_seconds,
                            idle_seconds,
                        );
                    }
                    Ok(DaemonMessage::ProjectActivity {
                        project_name,
                        app_name,
                        active_seconds,
                    }) => {
                        let _ = repo.record_project_activity(
                            &project_name,
                            &app_name,
                            active_seconds,
                        );
                    }
                    Ok(DaemonMessage::Typing(slice, app)) => {
                        let _ = repo.insert_typing_slice(&slice, app.as_deref());
                    }
                    Ok(DaemonMessage::SessionClose) => {
                        let _ = repo.close_open_computer_sessions();
                    }
                    Ok(DaemonMessage::Shutdown) => {
                        let _ = repo.close_open_computer_sessions();
                        break;
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }

                // Check 60m hourly rollup
                if last_hourly_rollup.elapsed() >= Duration::from_secs(3600) {
                    let _ = repo.run_hourly_rollup();
                    last_hourly_rollup = Instant::now();
                }

                // Check 24h daily rollup
                if last_daily_rollup.elapsed() >= Duration::from_secs(86400) {
                    let _ = repo.run_daily_rollup();
                    let _ = repo.check_and_update_records();
                    last_daily_rollup = Instant::now();
                }
            }

            info!("Writer thread completed flush.");
        });

        // Hardware Collector Thread
        let collector_tx = tx.clone();
        let collector_running = running.clone();
        let interval_secs = self.config.general.collection_interval_seconds.max(1);
        let collector_handle = thread::spawn(move || {
            let platform = create_platform();
            let mut collector = SystemTelemetryCollector::new();

            while collector_running.load(Ordering::SeqCst) {
                let snapshot = collector.collect_snapshot(&platform);
                if collector_tx
                    .send(DaemonMessage::Telemetry(snapshot))
                    .is_err()
                {
                    break;
                }
                thread::sleep(Duration::from_secs(interval_secs));
            }
        });

        // Activity & Window Polling Loop
        let platform = create_platform();
        let categorizer = Categorizer::new(self.config.categories.clone());
        let mut session_manager =
            SessionManager::new(categorizer, self.config.activity.idle_threshold_seconds);
        let mut typing_state = TypingState::new();

        // Typing Hook
        let (typing_event_tx, typing_event_rx) = bounded::<TypingInputEvent>(500);
        if self.config.typing.enabled {
            match platform.start_hook(typing_event_tx) {
                Ok(()) => info!("Keyboard ergonomics hook activated."),
                Err(e) => warn!(
                    "Failed to start keyboard hook ({}). Typing analytics disabled.",
                    e
                ),
            }
        }

        let poll_interval = Duration::from_secs(
            self.config
                .general
                .active_window_poll_interval_seconds
                .max(1),
        );
        let mut last_typing_flush = Instant::now();
        let flush_interval = Duration::from_secs(self.config.typing.flush_interval_seconds.max(10));

        while running.load(Ordering::SeqCst) {
            // Drain keyboard events
            while let Ok(evt) = typing_event_rx.try_recv() {
                typing_state.record_event(evt);
            }

            // Flush typing slices every 60s
            if last_typing_flush.elapsed() >= flush_interval {
                if let Some(slice) = typing_state.flush_slice(Utc::now()) {
                    let current_app = session_manager.current_app().map(|s| s.to_string());
                    let _ = tx.send(DaemonMessage::Typing(slice, current_app));
                }
                last_typing_flush = Instant::now();
            }

            // Query active window and idle
            let active_win = platform.get_active_window();
            let idle_duration = platform.get_idle_time().unwrap_or(Duration::from_secs(0));
            let is_locked = platform.is_session_locked();

            let tick = session_manager.handle_tick(active_win.clone(), idle_duration, is_locked, Utc::now());

            if tick.rollover || (tick.state_changed && tick.state == crate::activity::SessionState::Locked) {
                let _ = tx.send(DaemonMessage::SessionClose);
            }

            match tick.state {
                crate::activity::SessionState::Active => {
                    if let Some(app) = tick.app_name {
                        let active_sec = if tick.active_seconds > 0 {
                            tick.active_seconds
                        } else {
                            poll_interval.as_secs()
                        };
                        let _ = tx.send(DaemonMessage::Activity {
                            app_name: app.clone(),
                            category: tick.category,
                            active_seconds: active_sec,
                            idle_seconds: 0,
                        });

                        if let Some(win) = active_win {
                            if let Some(project) = crate::activity::ProjectExtractor::extract(
                                &win.executable_name,
                                win.window_title.as_deref(),
                            ) {
                                let _ = tx.send(DaemonMessage::ProjectActivity {
                                    project_name: project,
                                    app_name: app,
                                    active_seconds: active_sec,
                                });
                            }
                        }
                    }
                }
                crate::activity::SessionState::Idle => {
                    let idle_sec = if tick.idle_seconds > 0 {
                        tick.idle_seconds
                    } else {
                        poll_interval.as_secs()
                    };
                    let _ = tx.send(DaemonMessage::Activity {
                        app_name: "idle".to_string(),
                        category: "System".to_string(),
                        active_seconds: 0,
                        idle_seconds: idle_sec,
                    });
                }
                crate::activity::SessionState::Locked => {
                    // Session closed; no activity or idle time attributed to foreground windows
                }
            }

            thread::sleep(poll_interval);
        }

        // Cleanup
        platform.stop_hook();
        let _ = tx.send(DaemonMessage::Shutdown);
        let _ = collector_handle.join();
        let _ = writer_handle.join();

        info!("ByteLog daemon shutdown complete.");
        Ok(())
    }
}

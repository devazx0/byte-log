-- PCStats SQLite Database Schema: V01__init.sql
-- Conforms strictly to PRD.md Section 28

-- Schema Migrations Tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    description TEXT NOT NULL
);

-- Applications Master Table
CREATE TABLE IF NOT EXISTS applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    executable_name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'Other',
    is_ignored BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_apps_exec ON applications(executable_name);
CREATE INDEX IF NOT EXISTS idx_apps_cat ON applications(category);

-- Computer Usage Sessions (System Active/Idle cycles)
CREATE TABLE IF NOT EXISTS computer_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    active_duration_seconds INTEGER NOT NULL DEFAULT 0,
    idle_duration_seconds INTEGER NOT NULL DEFAULT 0,
    is_open BOOLEAN NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_comp_sessions_started ON computer_sessions(started_at);

-- Application Usage Sessions (Foreground Active Windows)
CREATE TABLE IF NOT EXISTS application_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    computer_session_id INTEGER REFERENCES computer_sessions(id) ON DELETE CASCADE,
    application_id INTEGER NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    duration_seconds INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_app_sessions_started ON application_sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_app_sessions_app ON application_sessions(application_id);

-- Ephemeral Typing Slices
CREATE TABLE IF NOT EXISTS typing_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    application_id INTEGER REFERENCES applications(id) ON DELETE SET NULL,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP NOT NULL,
    active_typing_seconds INTEGER NOT NULL,
    keystrokes INTEGER NOT NULL,
    backspaces INTEGER NOT NULL,
    enters INTEGER NOT NULL,
    modifiers INTEGER NOT NULL,
    estimated_wpm REAL NOT NULL,
    estimated_accuracy REAL NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_typing_started ON typing_sessions(started_at);

-- Raw System Telemetry Snapshots (Tier 1: 5s interval, 7-day retention)
CREATE TABLE IF NOT EXISTS system_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cpu_utilization REAL NOT NULL,
    memory_used_bytes INTEGER NOT NULL,
    memory_total_bytes INTEGER NOT NULL,
    swap_used_bytes INTEGER NOT NULL,
    disk_read_bytes_sec INTEGER NOT NULL,
    disk_write_bytes_sec INTEGER NOT NULL,
    net_rx_bytes_sec INTEGER NOT NULL,
    net_tx_bytes_sec INTEGER NOT NULL,
    cpu_temp_celsius REAL,
    gpu_utilization REAL,
    battery_percent REAL,
    battery_charging BOOLEAN
);
CREATE INDEX IF NOT EXISTS idx_snapshots_time ON system_snapshots(timestamp);

-- Hourly Aggregated Telemetry (Tier 2: 90-day retention)
CREATE TABLE IF NOT EXISTS hourly_aggregates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hour_timestamp TIMESTAMP NOT NULL UNIQUE,
    avg_cpu_utilization REAL NOT NULL,
    max_cpu_utilization REAL NOT NULL,
    avg_memory_used_bytes INTEGER NOT NULL,
    max_memory_used_bytes INTEGER NOT NULL,
    total_net_rx_bytes INTEGER NOT NULL,
    total_net_tx_bytes INTEGER NOT NULL,
    total_disk_read_bytes INTEGER NOT NULL,
    total_disk_write_bytes INTEGER NOT NULL,
    sample_count INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_hourly_time ON hourly_aggregates(hour_timestamp);

-- Daily Summary Aggregates (Tier 3: Permanent)
CREATE TABLE IF NOT EXISTS daily_aggregates (
    date TEXT PRIMARY KEY, -- 'YYYY-MM-DD'
    total_active_seconds INTEGER NOT NULL DEFAULT 0,
    total_idle_seconds INTEGER NOT NULL DEFAULT 0,
    total_keystrokes INTEGER NOT NULL DEFAULT 0,
    total_backspaces INTEGER NOT NULL DEFAULT 0,
    average_wpm REAL NOT NULL DEFAULT 0.0,
    top_application_id INTEGER REFERENCES applications(id),
    avg_cpu_utilization REAL NOT NULL DEFAULT 0.0,
    peak_cpu_utilization REAL NOT NULL DEFAULT 0.0,
    avg_memory_used_bytes INTEGER NOT NULL DEFAULT 0,
    total_net_bytes INTEGER NOT NULL DEFAULT 0
);

-- Personal Records High Watermarks
CREATE TABLE IF NOT EXISTS personal_records (
    record_key TEXT PRIMARY KEY,
    record_value REAL NOT NULL,
    formatted_value TEXT NOT NULL,
    achieved_at TIMESTAMP NOT NULL,
    notes TEXT
);

-- Unlocked Achievements
CREATE TABLE IF NOT EXISTS achievements (
    badge_key TEXT PRIMARY KEY,
    unlocked_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    progress REAL NOT NULL DEFAULT 1.0
);

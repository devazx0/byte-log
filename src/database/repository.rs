use crate::collectors::SystemSnapshot;
use crate::error::{PcStatsError, Result};
use crate::typing::TypingSlice;
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use tracing::info;

pub struct Repository {
    conn: Connection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SummaryStats {
    pub active_duration_seconds: u64,
    pub idle_duration_seconds: u64,
    pub coding_duration_seconds: u64,
    pub total_keystrokes: u64,
    pub total_backspaces: u64,
    pub average_wpm: f32,
    pub peak_wpm: f32,
    pub estimated_accuracy: f32,
    pub avg_cpu_load: f32,
    pub max_cpu_load: f32,
    pub avg_memory_bytes: u64,
    pub total_memory_bytes: u64,
    pub total_net_rx_bytes: u64,
    pub total_net_tx_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppUsageStat {
    pub executable_name: String,
    pub display_name: String,
    pub category: String,
    pub active_seconds: u64,
    pub percentage: f32,
    pub session_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonalRecord {
    pub key: String,
    pub value: f64,
    pub formatted: String,
    pub achieved_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AchievementBadge {
    pub key: String,
    pub title: String,
    pub description: String,
    pub unlocked_at: Option<String>,
    pub progress: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricDistribution {
    pub metric: String,
    pub unit: String,
    pub min: f32,
    pub mean: f32,
    pub p50: f32,
    pub p95: f32,
    pub max: f32,
    pub count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectUsageStat {
    pub project_name: String,
    pub active_seconds: u64,
    pub percentage: f32,
    pub top_app: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HeatmapMetric {
    Active,
    Coding,
    Typing,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeatmapDay {
    pub date: NaiveDate,
    pub value: f64,
}

impl Repository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn open(db_path: &Path) -> Result<Self> {
        let mut conn = crate::database::DbConnection::open(db_path)?;
        crate::database::Migrator::apply_all(&mut conn)?;
        Ok(Self::new(conn))
    }

    pub fn insert_snapshot(&self, snapshot: &SystemSnapshot) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO system_snapshots (
                timestamp, cpu_utilization, memory_used_bytes, memory_total_bytes, swap_used_bytes,
                disk_read_bytes_sec, disk_write_bytes_sec, net_rx_bytes_sec, net_tx_bytes_sec,
                cpu_temp_celsius, gpu_utilization, battery_percent, battery_charging
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13);",
                params![
                    snapshot.timestamp.to_rfc3339(),
                    snapshot.cpu_utilization,
                    snapshot.memory_used_bytes as i64,
                    snapshot.memory_total_bytes as i64,
                    snapshot.swap_used_bytes as i64,
                    snapshot.disk_read_bytes_sec as i64,
                    snapshot.disk_write_bytes_sec as i64,
                    snapshot.net_rx_bytes_sec as i64,
                    snapshot.net_tx_bytes_sec as i64,
                    snapshot.cpu_temp_celsius,
                    snapshot.gpu_utilization,
                    snapshot.battery_percent,
                    snapshot.battery_charging,
                ],
            )
            .map_err(PcStatsError::Database)?;
        Ok(())
    }

    pub fn close_open_computer_sessions(&self) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE computer_sessions SET is_open = 0, ended_at = ?1 WHERE is_open = 1;",
                params![now],
            )
            .map_err(PcStatsError::Database)?;
        Ok(())
    }

    pub fn record_application_activity(
        &self,
        app_name: &str,
        category: &str,
        active_seconds: u64,
        idle_seconds: u64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        if active_seconds == 0 && idle_seconds == 0 {
            return Ok(());
        }

        // Ensure application exists without downgrading specific category to 'Other'
        self.conn
            .execute(
                "INSERT INTO applications (executable_name, display_name, category)
             VALUES (?1, ?1, ?2)
             ON CONFLICT(executable_name) DO UPDATE SET 
                category = CASE WHEN ?2 != 'Other' THEN ?2 ELSE category END;",
                params![app_name, category],
            )
            .map_err(PcStatsError::Database)?;

        let app_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM applications WHERE executable_name = ?1;",
                params![app_name],
                |row| row.get(0),
            )
            .map_err(PcStatsError::Database)?;

        // Update or insert into open computer session
        let computer_session_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM computer_sessions WHERE is_open = 1 ORDER BY id DESC LIMIT 1;",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| {
                let _ = self.conn.execute(
                    "INSERT INTO computer_sessions (started_at, is_open) VALUES (?1, 1);",
                    params![now],
                );
                self.conn.last_insert_rowid()
            });

        self.conn
            .execute(
                "UPDATE computer_sessions SET 
                active_duration_seconds = active_duration_seconds + ?1,
                idle_duration_seconds = idle_duration_seconds + ?2,
                ended_at = ?3
             WHERE id = ?4;",
                params![
                    active_seconds as i64,
                    idle_seconds as i64,
                    now,
                    computer_session_id
                ],
            )
            .map_err(PcStatsError::Database)?;

        // Record application session slice only when there is active user focus
        if active_seconds > 0 && app_name != "idle" {
            self.conn
                .execute(
                    "INSERT INTO application_sessions (
                    computer_session_id, application_id, started_at, ended_at, duration_seconds
                ) VALUES (?1, ?2, ?3, ?3, ?4);",
                    params![computer_session_id, app_id, now, active_seconds as i64],
                )
                .map_err(PcStatsError::Database)?;
        }

        Ok(())
    }

    pub fn insert_typing_slice(&self, slice: &TypingSlice, app_name: Option<&str>) -> Result<()> {
        let app_id: Option<i64> = if let Some(name) = app_name {
            self.conn
                .query_row(
                    "SELECT id FROM applications WHERE executable_name = ?1;",
                    params![name],
                    |row| row.get(0),
                )
                .ok()
        } else {
            None
        };

        self.conn
            .execute(
                "INSERT INTO typing_sessions (
                application_id, started_at, ended_at, active_typing_seconds,
                keystrokes, backspaces, enters, modifiers, estimated_wpm, estimated_accuracy
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
                params![
                    app_id,
                    slice.started_at.to_rfc3339(),
                    slice.ended_at.to_rfc3339(),
                    slice.active_typing_seconds as i64,
                    slice.keystrokes as i64,
                    slice.backspaces as i64,
                    slice.enters as i64,
                    slice.modifiers as i64,
                    slice.estimated_wpm,
                    slice.estimated_accuracy,
                ],
            )
            .map_err(PcStatsError::Database)?;

        Ok(())
    }

    pub fn run_hourly_rollup(&self) -> Result<()> {
        // Roll up snapshots from previous hours into hourly_aggregates
        self.conn
            .execute(
                "INSERT OR IGNORE INTO hourly_aggregates (
                hour_timestamp, avg_cpu_utilization, max_cpu_utilization,
                avg_memory_used_bytes, max_memory_used_bytes,
                total_net_rx_bytes, total_net_tx_bytes,
                total_disk_read_bytes, total_disk_write_bytes, sample_count
            )
            SELECT 
                strftime('%Y-%m-%dT%H:00:00Z', timestamp) as hr,
                AVG(cpu_utilization), MAX(cpu_utilization),
                CAST(AVG(memory_used_bytes) AS INTEGER), MAX(memory_used_bytes),
                SUM(net_rx_bytes_sec * 5), SUM(net_tx_bytes_sec * 5),
                SUM(disk_read_bytes_sec * 5), SUM(disk_write_bytes_sec * 5),
                COUNT(*)
            FROM system_snapshots
            WHERE timestamp < strftime('%Y-%m-%dT%H:00:00Z', 'now')
            GROUP BY hr;",
                [],
            )
            .map_err(PcStatsError::Database)?;
        Ok(())
    }

    pub fn run_daily_rollup(&self) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO daily_aggregates (
                date, total_active_seconds, total_idle_seconds, total_keystrokes,
                total_backspaces, average_wpm, top_application_id,
                avg_cpu_utilization, peak_cpu_utilization, avg_memory_used_bytes, total_net_bytes,
                total_coding_seconds
            )
            SELECT 
                d.dt,
                COALESCE(cs.act_sec, 0),
                COALESCE(cs.idl_sec, 0),
                COALESCE(ts.strokes, 0),
                COALESCE(ts.backs, 0),
                COALESCE(ts.wpm, 0.0),
                top_app.id,
                COALESCE(ss.avg_cpu, 0.0),
                COALESCE(ss.max_cpu, 0.0),
                COALESCE(ss.avg_mem, 0),
                COALESCE(ss.total_net, 0),
                COALESCE(dev.code_sec, 0)
            FROM (
                SELECT DISTINCT date(started_at) as dt FROM computer_sessions
                UNION SELECT DISTINCT date(timestamp) as dt FROM system_snapshots
                UNION SELECT DISTINCT date(started_at) as dt FROM application_sessions
            ) d
            LEFT JOIN (
                SELECT date(started_at) as dt, SUM(active_duration_seconds) as act_sec, SUM(idle_duration_seconds) as idl_sec
                FROM computer_sessions GROUP BY dt
            ) cs ON d.dt = cs.dt
            LEFT JOIN (
                SELECT date(started_at) as dt, SUM(keystrokes) as strokes, SUM(backspaces) as backs, AVG(estimated_wpm) as wpm
                FROM typing_sessions GROUP BY dt
            ) ts ON d.dt = ts.dt
            LEFT JOIN (
                SELECT date(timestamp) as dt, AVG(cpu_utilization) as avg_cpu, MAX(cpu_utilization) as max_cpu,
                       CAST(AVG(memory_used_bytes) AS INTEGER) as avg_mem,
                       SUM(net_rx_bytes_sec * 5 + net_tx_bytes_sec * 5) as total_net
                FROM system_snapshots GROUP BY dt
            ) ss ON d.dt = ss.dt
            LEFT JOIN (
                SELECT date(s.started_at) as dt, SUM(s.duration_seconds) as code_sec
                FROM application_sessions s
                JOIN applications a ON s.application_id = a.id
                WHERE a.category = 'Development'
                GROUP BY dt
            ) dev ON d.dt = dev.dt
            LEFT JOIN (
                SELECT dt, id FROM (
                    SELECT date(started_at) as dt, application_id as id, SUM(duration_seconds) as dur,
                           ROW_NUMBER() OVER (PARTITION BY date(started_at) ORDER BY SUM(duration_seconds) DESC) as rn
                    FROM application_sessions
                    GROUP BY date(started_at), application_id
                ) WHERE rn = 1
            ) top_app ON d.dt = top_app.dt;",
            [],
        ).map_err(PcStatsError::Database)?;
        Ok(())
    }

    pub fn prune_retention(&self, raw_days: u32, hourly_days: u32) -> Result<()> {
        let raw_limit = format!("-{} days", raw_days);
        let hourly_limit = format!("-{} days", hourly_days);

        info!(
            "Pruning raw snapshots older than {} and hourly aggregates older than {}...",
            raw_limit, hourly_limit
        );

        self.conn
            .execute(
                "DELETE FROM system_snapshots WHERE timestamp < datetime('now', ?1);",
                params![raw_limit],
            )
            .map_err(PcStatsError::Database)?;

        self.conn
            .execute(
                "DELETE FROM hourly_aggregates WHERE hour_timestamp < datetime('now', ?1);",
                params![hourly_limit],
            )
            .map_err(PcStatsError::Database)?;

        let _ = crate::database::DbConnection::incremental_vacuum(&self.conn);
        Ok(())
    }

    pub fn check_and_update_records(&self) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        // 1. Peak Keystrokes in a single day
        let max_daily_strokes: Option<(i64, String)> = self.conn.query_row(
            "SELECT total_keystrokes, date FROM daily_aggregates ORDER BY total_keystrokes DESC LIMIT 1;",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).ok();

        if let Some((strokes, dt)) = max_daily_strokes {
            if strokes > 0 {
                self.conn.execute(
                    "INSERT INTO personal_records (record_key, record_value, formatted_value, achieved_at, notes)
                     VALUES ('peak_keystrokes_single_day', ?1, ?2, ?3, 'Highest daily keystrokes')
                     ON CONFLICT(record_key) DO UPDATE SET
                        record_value = MAX(record_value, ?1),
                        formatted_value = CASE WHEN ?1 > record_value THEN ?2 ELSE formatted_value END,
                        achieved_at = CASE WHEN ?1 > record_value THEN ?3 ELSE achieved_at END;",
                    params![strokes as f64, format!("{} keystrokes", strokes), dt],
                ).map_err(PcStatsError::Database)?;
            }
        }

        // 2. Peak WPM in a typing slice
        let peak_wpm: Option<(f64, String)> = self.conn.query_row(
            "SELECT estimated_wpm, started_at FROM typing_sessions ORDER BY estimated_wpm DESC LIMIT 1;",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).ok();

        if let Some((wpm, dt)) = peak_wpm {
            if wpm > 0.0 {
                self.conn.execute(
                    "INSERT INTO personal_records (record_key, record_value, formatted_value, achieved_at, notes)
                     VALUES ('fastest_sustained_wpm', ?1, ?2, ?3, 'Peak typing velocity')
                     ON CONFLICT(record_key) DO UPDATE SET
                        record_value = MAX(record_value, ?1),
                        formatted_value = CASE WHEN ?1 > record_value THEN ?2 ELSE formatted_value END,
                        achieved_at = CASE WHEN ?1 > record_value THEN ?3 ELSE achieved_at END;",
                    params![wpm, format!("{:.1} WPM", wpm), dt],
                ).map_err(PcStatsError::Database)?;

                if wpm >= 80.0 {
                    let _ = self.unlock_achievement("speed_80wpm", &now);
                }
                if wpm >= 100.0 {
                    let _ = self.unlock_achievement("speed_100wpm", &now);
                }
            }
        }

        // 3. Longest active session
        let longest_session: Option<(i64, String)> = self.conn.query_row(
            "SELECT active_duration_seconds, started_at FROM computer_sessions ORDER BY active_duration_seconds DESC LIMIT 1;",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).ok();

        if let Some((secs, dt)) = longest_session {
            if secs > 0 {
                let formatted = format!("{}h {}m", secs / 3600, (secs % 3600) / 60);
                self.conn.execute(
                    "INSERT INTO personal_records (record_key, record_value, formatted_value, achieved_at, notes)
                     VALUES ('longest_computer_session_seconds', ?1, ?2, ?3, 'Longest continuous session')
                     ON CONFLICT(record_key) DO UPDATE SET
                        record_value = MAX(record_value, ?1),
                        formatted_value = CASE WHEN ?1 > record_value THEN ?2 ELSE formatted_value END,
                        achieved_at = CASE WHEN ?1 > record_value THEN ?3 ELSE achieved_at END;",
                    params![secs as f64, formatted, dt],
                ).map_err(PcStatsError::Database)?;
            }
        }

        // 4. Longest system uptime
        let sys_uptime = sysinfo::System::uptime();
        if sys_uptime > 0 {
            let formatted = format!(
                "{}d {}h {}m",
                sys_uptime / 86400,
                (sys_uptime % 86400) / 3600,
                (sys_uptime % 3600) / 60
            );
            let _ = self.conn.execute(
                "INSERT INTO personal_records (record_key, record_value, formatted_value, achieved_at, notes)
                 VALUES ('longest_system_uptime_seconds', ?1, ?2, ?3, 'Peak system uptime')
                 ON CONFLICT(record_key) DO UPDATE SET
                    record_value = MAX(record_value, ?1),
                    formatted_value = CASE WHEN ?1 > record_value THEN ?2 ELSE formatted_value END,
                    achieved_at = CASE WHEN ?1 > record_value THEN ?3 ELSE achieved_at END;",
                params![sys_uptime as f64, formatted, now],
            );

            if sys_uptime >= 7 * 86400 {
                let _ = self.unlock_achievement("uptime_7d", &now);
            }
            if sys_uptime >= 30 * 86400 {
                let _ = self.unlock_achievement("uptime_30d", &now);
            }
        }

        // 5. Total Development hours
        let coding_sec: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(s.duration_seconds), 0)
                 FROM application_sessions s
                 JOIN applications a ON s.application_id = a.id
                 WHERE a.category = 'Development';",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if coding_sec >= 360_000 {
            let _ = self.unlock_achievement("coding_100h", &now);
        }

        // 6. Night owl hours (active between 00:00 and 05:00)
        let night_sec: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(duration_seconds), 0)
                 FROM application_sessions
                 WHERE CAST(strftime('%H', started_at) AS INTEGER) >= 0
                   AND CAST(strftime('%H', started_at) AS INTEGER) < 5;",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if night_sec >= 180_000 {
            let _ = self.unlock_achievement("night_owl_50h", &now);
        }

        // 7. Check Keystrokes milestones
        let total_strokes: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(keystrokes), 0) FROM typing_sessions;",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if total_strokes >= 10_000 {
            let _ = self.unlock_achievement("keystrokes_10k", &now);
        }
        if total_strokes >= 100_000 {
            let _ = self.unlock_achievement("keystrokes_100k", &now);
        }
        if total_strokes >= 1_000_000 {
            let _ = self.unlock_achievement("keystrokes_1m", &now);
        }

        Ok(())
    }

    fn unlock_achievement(&self, key: &str, timestamp: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO achievements (badge_key, unlocked_at, progress) VALUES (?1, ?2, 1.0);",
            params![key, timestamp],
        ).map_err(PcStatsError::Database)?;
        Ok(())
    }

    pub fn get_summary_stats(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<SummaryStats> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let (active_sec, idle_sec): (i64, i64) = self.conn.query_row(
            "SELECT COALESCE(SUM(active_duration_seconds), 0), COALESCE(SUM(idle_duration_seconds), 0)
             FROM computer_sessions WHERE started_at >= ?1 AND started_at <= ?2;",
            params![start_str, end_str],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0, 0));

        let coding_sec: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(s.duration_seconds), 0)
             FROM application_sessions s
             JOIN applications a ON s.application_id = a.id
             WHERE s.started_at >= ?1 AND s.started_at <= ?2 AND a.category = 'Development';",
                params![start_str, end_str],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let (strokes, backs, avg_wpm, peak_wpm): (i64, i64, f64, f64) = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(keystrokes), 0), COALESCE(SUM(backspaces), 0),
                    COALESCE(AVG(estimated_wpm), 0.0), COALESCE(MAX(estimated_wpm), 0.0)
             FROM typing_sessions WHERE started_at >= ?1 AND started_at <= ?2;",
                params![start_str, end_str],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap_or((0, 0, 0.0, 0.0));

        let accuracy = crate::typing::TypingAnalytics::calculate_estimated_accuracy(
            strokes as u64,
            backs as u64,
        );

        let (avg_cpu, max_cpu, avg_mem, tot_mem, rx, tx): (f64, f64, i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(cpu_utilization), 0.0), COALESCE(MAX(cpu_utilization), 0.0),
                    COALESCE(CAST(AVG(memory_used_bytes) AS INTEGER), 0),
                    COALESCE(MAX(memory_total_bytes), 0),
                    COALESCE(SUM(net_rx_bytes_sec * 5), 0),
                    COALESCE(SUM(net_tx_bytes_sec * 5), 0)
             FROM system_snapshots WHERE timestamp >= ?1 AND timestamp <= ?2;",
                params![start_str, end_str],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .unwrap_or((0.0, 0.0, 0, 0, 0, 0));

        Ok(SummaryStats {
            active_duration_seconds: active_sec as u64,
            idle_duration_seconds: idle_sec as u64,
            coding_duration_seconds: coding_sec as u64,
            total_keystrokes: strokes as u64,
            total_backspaces: backs as u64,
            average_wpm: avg_wpm as f32,
            peak_wpm: peak_wpm as f32,
            estimated_accuracy: accuracy,
            avg_cpu_load: avg_cpu as f32,
            max_cpu_load: max_cpu as f32,
            avg_memory_bytes: avg_mem as u64,
            total_memory_bytes: tot_mem as u64,
            total_net_rx_bytes: rx as u64,
            total_net_tx_bytes: tx as u64,
        })
    }

    pub fn get_top_applications(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        limit: u32,
        category_filter: Option<&str>,
    ) -> Result<Vec<AppUsageStat>> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let total_active_seconds: f64 = self.conn.query_row(
            "SELECT COALESCE(SUM(duration_seconds), 1) FROM application_sessions WHERE started_at >= ?1 AND started_at <= ?2;",
            params![start_str, end_str],
            |row| row.get(0),
        ).unwrap_or(1.0);

        let query = if let Some(cat) = category_filter {
            format!(
                "SELECT a.executable_name, a.display_name, a.category,
                        SUM(s.duration_seconds) as total_sec,
                        COUNT(s.id) as sessions
                 FROM application_sessions s
                 JOIN applications a ON s.application_id = a.id
                 WHERE s.started_at >= ?1 AND s.started_at <= ?2 AND a.category = '{}'
                 GROUP BY a.id
                 ORDER BY total_sec DESC
                 LIMIT ?3;",
                cat
            )
        } else {
            "SELECT a.executable_name, a.display_name, a.category,
                    SUM(s.duration_seconds) as total_sec,
                    COUNT(s.id) as sessions
             FROM application_sessions s
             JOIN applications a ON s.application_id = a.id
             WHERE s.started_at >= ?1 AND s.started_at <= ?2
             GROUP BY a.id
             ORDER BY total_sec DESC
             LIMIT ?3;"
                .to_string()
        };

        let mut stmt = self.conn.prepare(&query).map_err(PcStatsError::Database)?;
        let rows = stmt
            .query_map(params![start_str, end_str, limit], |row| {
                let exec: String = row.get(0)?;
                let disp: String = row.get(1)?;
                let cat: String = row.get(2)?;
                let secs: i64 = row.get(3)?;
                let sess: i64 = row.get(4)?;
                let pct = if total_active_seconds > 0.0 {
                    ((secs as f64 / total_active_seconds) * 100.0) as f32
                } else {
                    0.0
                };

                Ok(AppUsageStat {
                    executable_name: exec,
                    display_name: disp,
                    category: cat,
                    active_seconds: secs as u64,
                    percentage: pct,
                    session_count: sess as u32,
                })
            })
            .map_err(PcStatsError::Database)?;

        let mut stats = Vec::new();
        for r in rows {
            stats.push(r.map_err(PcStatsError::Database)?);
        }

        Ok(stats)
    }

    pub fn get_personal_records(&self) -> Result<Vec<PersonalRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT record_key, record_value, formatted_value, achieved_at, notes FROM personal_records ORDER BY achieved_at DESC;"
        ).map_err(PcStatsError::Database)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(PersonalRecord {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    formatted: row.get(2)?,
                    achieved_at: row.get(3)?,
                    notes: row.get(4)?,
                })
            })
            .map_err(PcStatsError::Database)?;

        let mut records = Vec::new();
        for r in rows {
            records.push(r.map_err(PcStatsError::Database)?);
        }
        Ok(records)
    }

    pub fn get_achievements(&self) -> Result<Vec<AchievementBadge>> {
        let catalog = [
            (
                "keystrokes_10k",
                "Century Scribe",
                "10,000 keystrokes recorded",
            ),
            (
                "keystrokes_100k",
                "Keyboard Warrior",
                "100,000 keystrokes recorded",
            ),
            (
                "keystrokes_1m",
                "Million Keystroke Club",
                "1,000,000 keystrokes recorded",
            ),
            (
                "speed_80wpm",
                "Touch Typist",
                "Sustained 80 WPM over 5 minutes",
            ),
            (
                "speed_100wpm",
                "Century Speed",
                "Sustained 100 WPM over 5 minutes",
            ),
            (
                "coding_100h",
                "Code Machine",
                "100 hours in Development applications",
            ),
            (
                "uptime_7d",
                "Uptime Disciple",
                "7 consecutive days of system uptime",
            ),
            (
                "uptime_30d",
                "Uptime Demon",
                "30 consecutive days of system uptime",
            ),
            (
                "night_owl_50h",
                "Night Owl",
                "50 hours active between 00:00 and 05:00",
            ),
        ];

        let mut badges = Vec::new();
        for (key, title, desc) in catalog {
            let unlocked_at: Option<String> = self
                .conn
                .query_row(
                    "SELECT unlocked_at FROM achievements WHERE badge_key = ?1;",
                    params![key],
                    |row| row.get(0),
                )
                .ok();

            let progress = if unlocked_at.is_some() { 1.0 } else { 0.0 };

            badges.push(AchievementBadge {
                key: key.to_string(),
                title: title.to_string(),
                description: desc.to_string(),
                unlocked_at,
                progress,
            });
        }

        Ok(badges)
    }

    pub fn reset_database(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "DELETE FROM system_snapshots;
             DELETE FROM hourly_aggregates;
             DELETE FROM daily_aggregates;
             DELETE FROM application_sessions;
             DELETE FROM computer_sessions;
             DELETE FROM typing_sessions;
             DELETE FROM personal_records;
             DELETE FROM achievements;
             VACUUM;",
            )
            .map_err(PcStatsError::Database)?;
        Ok(())
    }

    pub fn get_metric_distribution(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        metric_name: &str,
    ) -> Result<MetricDistribution> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let (column, unit) = match metric_name.to_lowercase().as_str() {
            "cpu" => ("cpu_utilization", "%"),
            "mem" | "memory" => ("(memory_used_bytes * 1.0 / (1024 * 1024))", "MB"),
            "disk" => ("((disk_read_bytes_sec + disk_write_bytes_sec) * 1.0 / 1024)", "KB/s"),
            "net" | "network" => ("((net_rx_bytes_sec + net_tx_bytes_sec) * 1.0 / 1024)", "KB/s"),
            "temp" | "temperature" => ("COALESCE(cpu_temp_celsius, 0.0)", "°C"),
            other => {
                return Err(PcStatsError::Validation(format!(
                    "Unknown metric '{}'. Supported metrics: cpu, mem, disk, net, temp",
                    other
                )));
            }
        };

        let query = format!(
            "SELECT {} FROM system_snapshots WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY 1 ASC;",
            column
        );
        let mut stmt = self.conn.prepare(&query).map_err(PcStatsError::Database)?;
        let rows = stmt
            .query_map(params![start_str, end_str], |row| {
                let val: f64 = row.get(0)?;
                Ok(val as f32)
            })
            .map_err(PcStatsError::Database)?;

        let mut values = Vec::new();
        for r in rows {
            values.push(r.map_err(PcStatsError::Database)?);
        }

        if values.is_empty() {
            return Ok(MetricDistribution {
                metric: metric_name.to_string(),
                unit: unit.to_string(),
                min: 0.0,
                mean: 0.0,
                p50: 0.0,
                p95: 0.0,
                max: 0.0,
                count: 0,
            });
        }

        let count = values.len();
        let min = values[0];
        let max = values[count - 1];
        let sum: f32 = values.iter().sum();
        let mean = sum / count as f32;

        let p50_idx = ((count as f32 * 0.50).round() as usize).min(count - 1);
        let p95_idx = ((count as f32 * 0.95).round() as usize).min(count - 1);
        let p50 = values[p50_idx];
        let p95 = values[p95_idx];

        Ok(MetricDistribution {
            metric: metric_name.to_string(),
            unit: unit.to_string(),
            min,
            mean,
            p50,
            p95,
            max,
            count,
        })
    }

    pub fn get_table_as_json_array(&self, table_name: &str) -> Result<serde_json::Value> {
        let valid_tables = [
            "system_snapshots",
            "applications",
            "computer_sessions",
            "application_sessions",
            "typing_sessions",
            "hourly_aggregates",
            "daily_aggregates",
            "personal_records",
            "achievements",
            "projects",
            "project_sessions",
        ];

        if !valid_tables.contains(&table_name) {
            return Err(PcStatsError::Validation(format!(
                "Invalid table '{}'. Valid tables: {}",
                table_name,
                valid_tables.join(", ")
            )));
        }

        let query = format!("SELECT * FROM {};", table_name);
        let mut stmt = self.conn.prepare(&query).map_err(PcStatsError::Database)?;
        let col_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();

        let mut rows_json = Vec::new();
        let mut rows = stmt.query([]).map_err(PcStatsError::Database)?;

        while let Some(row) = rows.next().map_err(PcStatsError::Database)? {
            let mut map = serde_json::Map::new();
            for (i, name) in col_names.iter().enumerate() {
                let val_ref = row.get_ref(i).map_err(PcStatsError::Database)?;
                let json_val = match val_ref {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Real(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Text(v) => {
                        serde_json::json!(String::from_utf8_lossy(v).to_string())
                    }
                    rusqlite::types::ValueRef::Blob(v) => serde_json::json!(v),
                };
                map.insert(name.clone(), json_val);
            }
            rows_json.push(serde_json::Value::Object(map));
        }

        Ok(serde_json::Value::Array(rows_json))
    }

    pub fn export_table_csv<W: std::io::Write>(
        &self,
        table_name: &str,
        writer: &mut W,
    ) -> Result<()> {
        let valid_tables = [
            "system_snapshots",
            "applications",
            "computer_sessions",
            "application_sessions",
            "typing_sessions",
            "hourly_aggregates",
            "daily_aggregates",
            "personal_records",
            "achievements",
            "projects",
            "project_sessions",
        ];

        if !valid_tables.contains(&table_name) {
            return Err(PcStatsError::Validation(format!(
                "Invalid table '{}'. Valid tables: {}",
                table_name,
                valid_tables.join(", ")
            )));
        }

        let query = format!("SELECT * FROM {};", table_name);
        let mut stmt = self.conn.prepare(&query).map_err(PcStatsError::Database)?;
        let col_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();

        writeln!(writer, "{}", col_names.join(",")).map_err(PcStatsError::Io)?;

        let mut rows = stmt.query([]).map_err(PcStatsError::Database)?;
        while let Some(row) = rows.next().map_err(PcStatsError::Database)? {
            let mut line = Vec::new();
            for i in 0..col_names.len() {
                let val_ref = row.get_ref(i).map_err(PcStatsError::Database)?;
                let s = match val_ref {
                    rusqlite::types::ValueRef::Null => "".to_string(),
                    rusqlite::types::ValueRef::Integer(v) => v.to_string(),
                    rusqlite::types::ValueRef::Real(v) => v.to_string(),
                    rusqlite::types::ValueRef::Text(v) => {
                        let text = String::from_utf8_lossy(v);
                        if text.contains(',') || text.contains('"') {
                            format!("\"{}\"", text.replace('"', "\"\""))
                        } else {
                            text.to_string()
                        }
                    }
                    rusqlite::types::ValueRef::Blob(_) => "<blob>".to_string(),
                };
                line.push(s);
            }
            writeln!(writer, "{}", line.join(",")).map_err(PcStatsError::Io)?;
        }

        Ok(())
    }

    pub fn record_project_activity(
        &self,
        project_name: &str,
        app_name: &str,
        active_seconds: u64,
    ) -> Result<()> {
        if active_seconds == 0 || project_name.trim().is_empty() {
            return Ok(());
        }

        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO projects (name) VALUES (?1)
                 ON CONFLICT(name) DO NOTHING;",
                params![project_name],
            )
            .map_err(PcStatsError::Database)?;

        let project_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM projects WHERE name = ?1;",
                params![project_name],
                |row| row.get(0),
            )
            .map_err(PcStatsError::Database)?;

        let app_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM applications WHERE executable_name = ?1;",
                params![app_name],
                |row| row.get(0),
            )
            .ok();

        self.conn
            .execute(
                "INSERT INTO project_sessions (
                    project_id, application_id, started_at, ended_at, duration_seconds
                ) VALUES (?1, ?2, ?3, ?3, ?4);",
                params![project_id, app_id, now, active_seconds as i64],
            )
            .map_err(PcStatsError::Database)?;

        Ok(())
    }

    pub fn get_top_projects(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<ProjectUsageStat>> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let total_project_seconds: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(duration_seconds), 1) FROM project_sessions
                 WHERE started_at >= ?1 AND started_at <= ?2;",
                params![start_str, end_str],
                |row| row.get(0),
            )
            .unwrap_or(1.0);

        let mut stmt = self
            .conn
            .prepare(
                "SELECT 
                    p.name,
                    SUM(ps.duration_seconds) as total_dur,
                    COALESCE(
                        (SELECT a.display_name FROM project_sessions ps2
                         JOIN applications a ON ps2.application_id = a.id
                         WHERE ps2.project_id = p.id AND ps2.started_at >= ?1 AND ps2.started_at <= ?2
                         GROUP BY ps2.application_id ORDER BY SUM(ps2.duration_seconds) DESC LIMIT 1),
                        'unknown'
                    ) as top_tool
                 FROM project_sessions ps
                 JOIN projects p ON ps.project_id = p.id
                 WHERE ps.started_at >= ?1 AND ps.started_at <= ?2
                 GROUP BY p.id
                 ORDER BY total_dur DESC
                 LIMIT ?3;",
            )
            .map_err(PcStatsError::Database)?;

        let rows = stmt
            .query_map(params![start_str, end_str, limit], |row| {
                let dur: i64 = row.get(1)?;
                let active_sec = dur.max(0) as u64;
                let pct = ((active_sec as f64 / total_project_seconds) * 100.0) as f32;
                Ok(ProjectUsageStat {
                    project_name: row.get(0)?,
                    active_seconds: active_sec,
                    percentage: pct,
                    top_app: row.get(2)?,
                })
            })
            .map_err(PcStatsError::Database)?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(PcStatsError::Database)?);
        }
        Ok(results)
    }

    pub fn get_heatmap_data(
        &self,
        year: i32,
        metric: HeatmapMetric,
    ) -> Result<Vec<HeatmapDay>> {
        let year_str = format!("{:04}", year);
        let mut results = Vec::new();

        match metric {
            HeatmapMetric::Active => {
                let mut stmt = self
                    .conn
                    .prepare(
                        "SELECT date, total_active_seconds FROM daily_aggregates
                         WHERE strftime('%Y', date) = ?1
                         ORDER BY date ASC;",
                    )
                    .map_err(PcStatsError::Database)?;

                let rows = stmt
                    .query_map(params![year_str], |row| {
                        let date_str: String = row.get(0)?;
                        let val: i64 = row.get(1)?;
                        Ok((date_str, val as f64))
                    })
                    .map_err(PcStatsError::Database)?;

                for r in rows {
                    let (dt_str, val) = r.map_err(PcStatsError::Database)?;
                    if let Ok(naive) = NaiveDate::parse_from_str(&dt_str, "%Y-%m-%d") {
                        results.push(HeatmapDay { date: naive, value: val });
                    }
                }
            }
            HeatmapMetric::Typing => {
                let mut stmt = self
                    .conn
                    .prepare(
                        "SELECT date, total_keystrokes FROM daily_aggregates
                         WHERE strftime('%Y', date) = ?1
                         ORDER BY date ASC;",
                    )
                    .map_err(PcStatsError::Database)?;

                let rows = stmt
                    .query_map(params![year_str], |row| {
                        let date_str: String = row.get(0)?;
                        let val: i64 = row.get(1)?;
                        Ok((date_str, val as f64))
                    })
                    .map_err(PcStatsError::Database)?;

                for r in rows {
                    let (dt_str, val) = r.map_err(PcStatsError::Database)?;
                    if let Ok(naive) = NaiveDate::parse_from_str(&dt_str, "%Y-%m-%d") {
                        results.push(HeatmapDay { date: naive, value: val });
                    }
                }
            }
            HeatmapMetric::Coding => {
                let mut stmt = self
                    .conn
                    .prepare(
                        "SELECT date(s.started_at) as dt, SUM(s.duration_seconds)
                         FROM application_sessions s
                         JOIN applications a ON s.application_id = a.id
                         WHERE a.category = 'Development' AND strftime('%Y', s.started_at) = ?1
                         GROUP BY dt
                         ORDER BY dt ASC;",
                    )
                    .map_err(PcStatsError::Database)?;

                let rows = stmt
                    .query_map(params![year_str], |row| {
                        let date_str: String = row.get(0)?;
                        let val: i64 = row.get(1)?;
                        Ok((date_str, val as f64))
                    })
                    .map_err(PcStatsError::Database)?;

                for r in rows {
                    let (dt_str, val) = r.map_err(PcStatsError::Database)?;
                    if let Ok(naive) = NaiveDate::parse_from_str(&dt_str, "%Y-%m-%d") {
                        results.push(HeatmapDay { date: naive, value: val });
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn update_application_category(&self, app_name: &str, new_category: &str) -> Result<()> {
        let app_name_clean = app_name.trim();
        self.conn
            .execute(
                "INSERT INTO applications (executable_name, display_name, category)
                 VALUES (?1, ?1, ?2)
                 ON CONFLICT(executable_name) DO UPDATE SET category = ?2;",
                params![app_name_clean, new_category],
            )
            .map_err(PcStatsError::Database)?;

        Ok(())
    }
}

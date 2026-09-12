use crate::cli::args::ExportArgs;
use crate::database::Repository;
use crate::error::{PcStatsError, Result};
use chrono::{Duration, Utc};
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize)]
struct FullExportJson {
    export_version: String,
    generated_at: String,
    summary_today: crate::database::repository::SummaryStats,
    top_applications: Vec<crate::database::repository::AppUsageStat>,
    personal_records: Vec<crate::database::repository::PersonalRecord>,
    achievements: Vec<crate::database::repository::AchievementBadge>,
}

pub fn handle_export(args: ExportArgs, db_path: &Path) -> Result<()> {
    let repo = Repository::open(db_path)?;
    let now = Utc::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

    let summary = repo.get_summary_stats(&start, &now)?;
    let apps = repo.get_top_applications(&(now - Duration::days(30)), &now, 100, None)?;
    let records = repo.get_personal_records()?;
    let badges = repo.get_achievements()?;

    let mut writer: Box<dyn Write> = if let Some(ref path) = args.output {
        Box::new(File::create(path).map_err(PcStatsError::Io)?)
    } else {
        Box::new(io::stdout())
    };

    if let Some(ref table) = args.table {
        match args.format.to_lowercase().as_str() {
            "json" => {
                let json_data = repo.get_table_as_json_array(table)?;
                let json_str = serde_json::to_string_pretty(&json_data)
                    .map_err(|e| PcStatsError::Serialization(e.to_string()))?;
                writeln!(writer, "{}", json_str).map_err(PcStatsError::Io)?;
                return Ok(());
            }
            "csv" => {
                repo.export_table_csv(table, &mut writer)?;
                return Ok(());
            }
            other => {
                return Err(PcStatsError::Validation(format!(
                    "Unsupported export format: {}",
                    other
                )));
            }
        }
    }

    match args.format.to_lowercase().as_str() {
        "json" => {
            let export = FullExportJson {
                export_version: "1.0".to_string(),
                generated_at: now.to_rfc3339(),
                summary_today: summary,
                top_applications: apps,
                personal_records: records,
                achievements: badges,
            };
            let json_str = serde_json::to_string_pretty(&export)
                .map_err(|e| PcStatsError::Serialization(e.to_string()))?;
            writeln!(writer, "{}", json_str).map_err(PcStatsError::Io)?;
        }
        "csv" => {
            writeln!(
                writer,
                "# ByteLog CSV Export Generated At {}",
                now.to_rfc3339()
            )
            .map_err(PcStatsError::Io)?;
            writeln!(writer, "# Table: top_applications").map_err(PcStatsError::Io)?;
            writeln!(
                writer,
                "rank,executable,category,active_seconds,percentage,sessions"
            )
            .map_err(PcStatsError::Io)?;
            for (i, a) in apps.iter().enumerate() {
                writeln!(
                    writer,
                    "{},{},{},{},{:.1},{}",
                    i + 1,
                    a.executable_name,
                    a.category,
                    a.active_seconds,
                    a.percentage,
                    a.session_count
                )
                .map_err(PcStatsError::Io)?;
            }
        }
        other => {
            return Err(PcStatsError::Validation(format!(
                "Unsupported export format: {}",
                other
            )))
        }
    }

    Ok(())
}

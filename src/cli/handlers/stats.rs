use crate::cli::args::StatsArgs;
use crate::cli::formatters::Formatter;
use crate::database::Repository;
use crate::error::{PcStatsError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct StatsJsonOutput {
    period: String,
    active_screen_time_seconds: u64,
    idle_time_seconds: u64,
    coding_time_seconds: u64,
    total_keystrokes: u64,
    average_wpm: f32,
    peak_wpm: f32,
    estimated_accuracy: f32,
    avg_cpu_load: f32,
    max_cpu_load: f32,
    avg_memory_bytes: u64,
    total_net_rx_bytes: u64,
    total_net_tx_bytes: u64,
}

pub fn handle_stats(args: StatsArgs, json: bool, csv: bool, db_path: &Path) -> Result<()> {
    let repo = Repository::open(db_path)?;
    let now = Utc::now();

    let (start, end, period_label) = if args.yesterday {
        let yesterday = now - Duration::days(1);
        let start = yesterday
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let end = yesterday
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();
        (start, end, "Yesterday")
    } else if args.week {
        (now - Duration::days(7), now, "Past 7 Days")
    } else if args.month {
        (now - Duration::days(30), now, "Past 30 Days")
    } else if let Some(ref range) = args.range {
        let parts: Vec<&str> = range.split("..").collect();
        if parts.len() != 2 {
            return Err(PcStatsError::Validation(
                "Range must be START..END in ISO format".to_string(),
            ));
        }
        let s = DateTime::parse_from_rfc3339(parts[0])
            .map_err(|e| PcStatsError::Validation(format!("Invalid start date: {}", e)))?
            .with_timezone(&Utc);
        let e = DateTime::parse_from_rfc3339(parts[1])
            .map_err(|e| PcStatsError::Validation(format!("Invalid end date: {}", e)))?
            .with_timezone(&Utc);
        (s, e, "Custom Range")
    } else {
        // Today
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        (start, now, "Today")
    };

    let summary = repo.get_summary_stats(&start, &end)?;

    if json {
        let json_out = StatsJsonOutput {
            period: period_label.to_string(),
            active_screen_time_seconds: summary.active_duration_seconds,
            idle_time_seconds: summary.idle_duration_seconds,
            coding_time_seconds: summary.coding_duration_seconds,
            total_keystrokes: summary.total_keystrokes,
            average_wpm: summary.average_wpm,
            peak_wpm: summary.peak_wpm,
            estimated_accuracy: summary.estimated_accuracy,
            avg_cpu_load: summary.avg_cpu_load,
            max_cpu_load: summary.max_cpu_load,
            avg_memory_bytes: summary.avg_memory_bytes,
            total_net_rx_bytes: summary.total_net_rx_bytes,
            total_net_tx_bytes: summary.total_net_tx_bytes,
        };
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());
        return Ok(());
    }

    if csv {
        println!("period,active_seconds,idle_seconds,coding_seconds,keystrokes,avg_wpm,accuracy,avg_cpu_percent,rx_bytes,tx_bytes");
        println!(
            "{},{},{},{},{},{:.1},{:.1},{:.1},{},{}",
            period_label,
            summary.active_duration_seconds,
            summary.idle_duration_seconds,
            summary.coding_duration_seconds,
            summary.total_keystrokes,
            summary.average_wpm,
            summary.estimated_accuracy,
            summary.avg_cpu_load,
            summary.total_net_rx_bytes,
            summary.total_net_tx_bytes,
        );
        return Ok(());
    }

    let coding_pct = if summary.active_duration_seconds > 0 {
        ((summary.coding_duration_seconds as f32 / summary.active_duration_seconds as f32) * 100.0)
            .clamp(0.0, 100.0)
    } else {
        0.0
    };

    println!(
        "ByteLog Summary: {} ({})",
        period_label,
        start.format("%Y-%m-%d")
    );
    println!("-----------------------------------------------------------------");
    println!(
        "Active Screen Time:   {} (Idle: {})",
        Formatter::format_duration(summary.active_duration_seconds),
        Formatter::format_duration(summary.idle_duration_seconds)
    );
    println!(
        "Development Time:     {} {} {:.1}%",
        Formatter::format_duration(summary.coding_duration_seconds),
        Formatter::render_progress_bar(coding_pct, 16),
        coding_pct
    );
    println!("Total Keystrokes:     {} strokes", summary.total_keystrokes);
    println!(
        "Gross Typing Speed:   {:.0} WPM (Peak: {:.0} WPM)",
        summary.average_wpm, summary.peak_wpm
    );
    println!(
        "Correction Ratio:     {:.1}% (Estimated Accuracy: {:.1}%)",
        100.0 - summary.estimated_accuracy,
        summary.estimated_accuracy
    );
    println!();
    println!(
        "Average CPU Load:     {:.1}% (Max: {:.1}%)",
        summary.avg_cpu_load, summary.max_cpu_load
    );
    println!(
        "Average Memory:       {} / {}",
        Formatter::format_bytes(summary.avg_memory_bytes),
        Formatter::format_bytes(summary.total_memory_bytes)
    );
    println!(
        "Network Consumption:  ↓ {}  ↑ {}",
        Formatter::format_bytes(summary.total_net_rx_bytes),
        Formatter::format_bytes(summary.total_net_tx_bytes)
    );

    Ok(())
}

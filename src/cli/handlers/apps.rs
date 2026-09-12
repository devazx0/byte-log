use crate::cli::args::AppsArgs;
use crate::cli::formatters::Formatter;
use crate::database::Repository;
use crate::error::Result;
use chrono::{Duration, Utc};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct AppJsonItem {
    rank: usize,
    executable_name: String,
    display_name: String,
    category: String,
    active_seconds: u64,
    percentage: f32,
    sessions: u32,
}

pub fn handle_apps(args: AppsArgs, json: bool, csv: bool, db_path: &Path) -> Result<()> {
    let repo = Repository::open(db_path)?;
    let now = Utc::now();

    let (start, end, period_label) = if args.week {
        (now - Duration::days(7), now, "Past 7 Days")
    } else if args.month {
        (now - Duration::days(30), now, "Past 30 Days")
    } else {
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        (start, now, "Today")
    };

    let apps = repo.get_top_applications(&start, &end, args.limit, args.category.as_deref())?;

    if json {
        let json_items: Vec<AppJsonItem> = apps
            .into_iter()
            .enumerate()
            .map(|(i, a)| AppJsonItem {
                rank: i + 1,
                executable_name: a.executable_name,
                display_name: a.display_name,
                category: a.category,
                active_seconds: a.active_seconds,
                percentage: a.percentage,
                sessions: a.session_count,
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_items).unwrap());
        return Ok(());
    }

    if csv {
        println!("rank,executable,category,active_seconds,percentage,sessions");
        for (i, a) in apps.iter().enumerate() {
            println!(
                "{},{},{},{},{:.1},{}",
                i + 1,
                a.executable_name,
                a.category,
                a.active_seconds,
                a.percentage,
                a.session_count
            );
        }
        return Ok(());
    }

    println!("Top Applications ({})", period_label);
    println!("-----------------------------------------------------------------------------");
    println!(
        "{:<4} {:<20} {:<15} {:<12} {:<8} {:<8}",
        "RANK", "APPLICATION", "CATEGORY", "ACTIVE TIME", "PERCENT", "SESSIONS"
    );

    if apps.is_empty() {
        println!("No application usage recorded for this timeframe.");
        return Ok(());
    }

    for (i, a) in apps.iter().enumerate() {
        println!(
            "{:<4} {:<20} {:<15} {:<12} {:>6.1}%  {:>6}",
            i + 1,
            a.executable_name,
            a.category,
            Formatter::format_duration(a.active_seconds),
            a.percentage,
            a.session_count
        );
    }

    Ok(())
}

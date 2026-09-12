use crate::cli::args::ProjectsArgs;
use crate::cli::formatters::Formatter;
use crate::database::Repository;
use crate::error::Result;
use chrono::{Duration, Utc};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct ProjectJsonItem {
    rank: usize,
    project_name: String,
    active_seconds: u64,
    percentage: f32,
    top_tool: String,
}

pub fn handle_projects(
    args: ProjectsArgs,
    json: bool,
    csv: bool,
    db_path: &Path,
) -> Result<()> {
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

    let projects = repo.get_top_projects(&start, &end, args.limit)?;

    if json {
        let json_items: Vec<ProjectJsonItem> = projects
            .into_iter()
            .enumerate()
            .map(|(i, p)| ProjectJsonItem {
                rank: i + 1,
                project_name: p.project_name,
                active_seconds: p.active_seconds,
                percentage: p.percentage,
                top_tool: p.top_app,
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_items).unwrap());
        return Ok(());
    }

    if csv {
        println!("rank,project_name,active_seconds,percentage,top_tool");
        for (i, p) in projects.iter().enumerate() {
            println!(
                "{},{},{},{:.1},{}",
                i + 1,
                p.project_name,
                p.active_seconds,
                p.percentage,
                p.top_app
            );
        }
        return Ok(());
    }

    println!("Active Projects & Workspaces ({})", period_label);
    println!("-----------------------------------------------------------------------------");
    println!(
        "{:<4} {:<24} {:<14} {:<14} {:<16}",
        "RANK", "PROJECT", "TIME SPENT", "% OF DEV TIME", "PRIMARY TOOL"
    );

    if projects.is_empty() {
        println!("No project or workspace activity recorded for this timeframe.");
        return Ok(());
    }

    for (i, p) in projects.iter().enumerate() {
        println!(
            "{:<4} {:<24} {:<14} {:>10.1}%   {:<16}",
            i + 1,
            p.project_name,
            Formatter::format_duration(p.active_seconds),
            p.percentage,
            p.top_app
        );
    }

    Ok(())
}

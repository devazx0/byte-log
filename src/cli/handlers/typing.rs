use crate::cli::args::TypingArgs;
use crate::database::Repository;
use crate::error::Result;
use chrono::{Duration, Utc};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TypingJsonOutput {
    period: String,
    total_keystrokes: u64,
    total_backspaces: u64,
    average_wpm: f32,
    peak_wpm: f32,
    estimated_accuracy: f32,
}

pub fn handle_typing(args: TypingArgs, json: bool, csv: bool, db_path: &Path) -> Result<()> {
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

    let summary = repo.get_summary_stats(&start, &end)?;

    if json {
        let json_out = TypingJsonOutput {
            period: period_label.to_string(),
            total_keystrokes: summary.total_keystrokes,
            total_backspaces: summary.total_backspaces,
            average_wpm: summary.average_wpm,
            peak_wpm: summary.peak_wpm,
            estimated_accuracy: summary.estimated_accuracy,
        };
        println!("{}", serde_json::to_string_pretty(&json_out).unwrap());
        return Ok(());
    }

    if csv {
        println!(
            "period,total_keystrokes,total_backspaces,average_wpm,peak_wpm,estimated_accuracy"
        );
        println!(
            "{},{},{},{:.1},{:.1},{:.1}",
            period_label,
            summary.total_keystrokes,
            summary.total_backspaces,
            summary.average_wpm,
            summary.peak_wpm,
            summary.estimated_accuracy
        );
        return Ok(());
    }

    println!(
        "ByteLog Typing Ergonomics: {} ({})",
        period_label,
        start.format("%Y-%m-%d")
    );
    println!("-----------------------------------------------------------------");
    println!(
        "Total Keystrokes:        {} strokes",
        summary.total_keystrokes
    );
    println!(
        "Total Backspaces:        {} corrections",
        summary.total_backspaces
    );
    println!("Average Typing Speed:    {:.0} WPM", summary.average_wpm);
    println!("Peak Typing Speed:       {:.0} WPM", summary.peak_wpm);
    println!(
        "Behavioral Accuracy:     {:.1}%",
        summary.estimated_accuracy
    );
    println!("\nPrivacy Guarantee: No raw text, words, or key sequences are ever stored.");

    Ok(())
}

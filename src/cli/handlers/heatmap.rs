use crate::cli::args::HeatmapArgs;
use crate::cli::formatters::Formatter;
use crate::database::repository::HeatmapMetric;
use crate::database::Repository;
use crate::error::{PcStatsError, Result};
use chrono::{Datelike, NaiveDate, Utc};
use std::collections::HashMap;
use std::path::Path;

pub fn handle_heatmap(
    args: HeatmapArgs,
    json: bool,
    csv: bool,
    db_path: &Path,
) -> Result<()> {
    let repo = Repository::open(db_path)?;
    let now = Utc::now();
    let year = args.year.unwrap_or_else(|| now.year());

    let metric = if args.coding {
        HeatmapMetric::Coding
    } else if args.typing {
        HeatmapMetric::Typing
    } else {
        HeatmapMetric::Active
    };

    let metric_name = match metric {
        HeatmapMetric::Active => "Active Computer Time",
        HeatmapMetric::Coding => "Development / Coding Time",
        HeatmapMetric::Typing => "Typing Volume (Keystrokes)",
    };

    let data = repo.get_heatmap_data(year, metric)?;
    let mut day_map: HashMap<NaiveDate, f64> = HashMap::new();
    for d in data {
        day_map.insert(d.date, d.value);
    }

    // Determine year boundary
    let start_date = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| PcStatsError::Validation(format!("Invalid year {}", year)))?;
    let end_date = NaiveDate::from_ymd_opt(year, 12, 31)
        .ok_or_else(|| PcStatsError::Validation(format!("Invalid year {}", year)))?;

    // Collect values to calculate quartiles
    let mut non_zero_values: Vec<f64> = day_map
        .values()
        .copied()
        .filter(|&v| v > 0.0)
        .collect();
    non_zero_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let q1 = quantile(&non_zero_values, 0.25);
    let q2 = quantile(&non_zero_values, 0.50);
    let q3 = quantile(&non_zero_values, 0.75);

    let get_level = |val: f64| -> u8 {
        if val <= 0.0 {
            0
        } else if val <= q1 {
            1
        } else if val <= q2 {
            2
        } else if val <= q3 {
            3
        } else {
            4
        }
    };

    // Calculate streaks
    let mut current_streak = 0u32;
    let mut max_streak = 0u32;
    let mut active_days = 0u32;
    let mut total_val = 0.0f64;

    let mut temp_streak = 0u32;
    let mut curr = start_date;
    let today_date = now.date_naive();

    while curr <= end_date {
        let val = day_map.get(&curr).copied().unwrap_or(0.0);
        if val > 0.0 {
            active_days += 1;
            total_val += val;
            temp_streak += 1;
            if temp_streak > max_streak {
                max_streak = temp_streak;
            }
        } else {
            temp_streak = 0;
        }

        if curr <= today_date {
            if val > 0.0 {
                current_streak += 1;
            } else {
                current_streak = 0;
            }
        }

        curr = curr.succ_opt().unwrap_or(curr);
        if curr == end_date {
            let last_val = day_map.get(&curr).copied().unwrap_or(0.0);
            if last_val > 0.0 {
                active_days += 1;
                total_val += last_val;
            }
            break;
        }
    }

    if json {
        let mut list = Vec::new();
        let mut c = start_date;
        while c <= end_date {
            let val = day_map.get(&c).copied().unwrap_or(0.0);
            list.push(serde_json::json!({
                "date": c.to_string(),
                "value": val,
                "level": get_level(val),
            }));
            if c == end_date {
                break;
            }
            c = c.succ_opt().unwrap_or(c);
        }

        let output = serde_json::json!({
            "year": year,
            "metric": metric_name,
            "total_value": total_val,
            "active_days": active_days,
            "current_streak": current_streak,
            "longest_streak": max_streak,
            "days": list
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    if csv {
        println!("date,value,level");
        let mut c = start_date;
        while c <= end_date {
            let val = day_map.get(&c).copied().unwrap_or(0.0);
            println!("{},{},{}", c, val, get_level(val));
            if c == end_date {
                break;
            }
            c = c.succ_opt().unwrap_or(c);
        }
        return Ok(());
    }

    // ANSI Terminal Presentation
    println!();
    println!(
        "\x1b[1;38;2;180;190;254m--- {} Activity Heatmap: {} ---\x1b[0m",
        year, metric_name
    );
    println!();

    // Map year into 53 calendar weeks x 7 days (0 = Sun, 1 = Mon, ..., 6 = Sat)
    // Grid: grid[day_of_week][week_idx]
    let mut grid = vec![vec![' '; 53]; 7];
    let mut level_grid = vec![vec![0u8; 53]; 7];
    let mut month_positions: Vec<(usize, &str)> = Vec::new();
    let mut last_month = 0;

    let mut cur = start_date;
    let jan1_weekday = start_date.weekday().num_days_from_sunday() as usize;

    while cur <= end_date {
        let day_of_year = cur.ordinal0() as usize;
        let week_idx = (day_of_year + jan1_weekday) / 7;
        let day_idx = cur.weekday().num_days_from_sunday() as usize;

        if week_idx < 53 {
            let val = day_map.get(&cur).copied().unwrap_or(0.0);
            let lvl = get_level(val);
            level_grid[day_idx][week_idx] = lvl;
            grid[day_idx][week_idx] = match lvl {
                0 => '·',
                1 => '░',
                2 => '▒',
                3 => '▓',
                _ => '█',
            };

            let m = cur.month();
            if m != last_month {
                let month_name = match m {
                    1 => "Jan",
                    2 => "Feb",
                    3 => "Mar",
                    4 => "Apr",
                    5 => "May",
                    6 => "Jun",
                    7 => "Jul",
                    8 => "Aug",
                    9 => "Sep",
                    10 => "Oct",
                    11 => "Nov",
                    12 => "Dec",
                    _ => "",
                };
                month_positions.push((week_idx, month_name));
                last_month = m;
            }
        }

        if cur == end_date {
            break;
        }
        cur = cur.succ_opt().unwrap_or(cur);
    }

    // Render Month Header Line
    let mut header_chars = vec![' '; 53 * 2 + 6];
    for (w_idx, m_name) in month_positions {
        let char_pos = 6 + w_idx * 2;
        for (i, ch) in m_name.chars().enumerate() {
            if char_pos + i < header_chars.len() {
                header_chars[char_pos + i] = ch;
            }
        }
    }
    let header_str: String = header_chars.into_iter().collect();
    println!("\x1b[38;2;137;220;235m{}\x1b[0m", header_str);

    // Day Labels
    let day_labels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    for d in 0..7 {
        let label = if d == 1 || d == 3 || d == 5 {
            day_labels[d]
        } else {
            "   "
        };
        print!("\x1b[38;2;147;153;178m{} \x1b[0m", label);

        for w in 0..53 {
            let ch = grid[d][w];
            let lvl = level_grid[d][w];
            let colored = match lvl {
                0 => format!("\x1b[38;2;69;71;90m{} \x1b[0m", ch),
                1 => format!("\x1b[38;2;74;133;92m{} \x1b[0m", ch),
                2 => format!("\x1b[38;2;114;180;125m{} \x1b[0m", ch),
                3 => format!("\x1b[38;2;166;227;161m{} \x1b[0m", ch),
                _ => format!("\x1b[38;2;57;211;83m{} \x1b[0m", ch),
            };
            print!("{}", colored);
        }
        println!();
    }

    println!();
    // Summary line & Legend
    let formatted_total = match metric {
        HeatmapMetric::Active | HeatmapMetric::Coding => {
            Formatter::format_duration(total_val as u64)
        }
        HeatmapMetric::Typing => Formatter::format_number(total_val as u64),
    };

    println!(
        "\x1b[1mTotal:\x1b[0m {}  |  \x1b[1mActive Days:\x1b[0m {}/365  |  \x1b[1mCurrent Streak:\x1b[0m {} days  |  \x1b[1mLongest Streak:\x1b[0m {} days",
        formatted_total, active_days, current_streak, max_streak
    );
    println!(
        "Legend: Less \x1b[38;2;69;71;90m·\x1b[0m \x1b[38;2;74;133;92m░\x1b[0m \x1b[38;2;114;180;125m▒\x1b[0m \x1b[38;2;166;227;161m▓\x1b[0m \x1b[38;2;57;211;83m█\x1b[0m More"
    );
    println!();

    Ok(())
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[idx]
}

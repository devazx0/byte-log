use crate::cli::args::PerformanceArgs;
use crate::cli::formatters::Formatter;
use crate::database::{MetricDistribution, Repository};
use crate::error::Result;
use chrono::{Duration, Utc};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct PerfSummaryJson {
    period: String,
    distributions: Vec<MetricDistribution>,
}

pub fn handle_performance(
    args: PerformanceArgs,
    json: bool,
    csv: bool,
    db_path: &Path,
) -> Result<()> {
    let repo = Repository::open(db_path)?;
    let now = Utc::now();

    let (start, end, period_label) = if args.week {
        (now - Duration::days(7), now, "Past 7 Days")
    } else {
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        (start, now, "Today")
    };

    let metrics_to_query: Vec<&str> = if let Some(ref m) = args.metric {
        vec![m.as_str()]
    } else {
        vec!["cpu", "mem", "disk", "net", "temp"]
    };

    let mut dists = Vec::new();
    for m in metrics_to_query {
        if let Ok(d) = repo.get_metric_distribution(&start, &end, m) {
            dists.push(d);
        }
    }

    if json {
        let out = PerfSummaryJson {
            period: period_label.to_string(),
            distributions: dists,
        };
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return Ok(());
    }

    if csv {
        println!("period,metric,unit,min,mean,p50,p95,max,samples");
        for d in &dists {
            println!(
                "{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{}",
                period_label, d.metric, d.unit, d.min, d.mean, d.p50, d.p95, d.max, d.count
            );
        }
        return Ok(());
    }

    let summary = repo.get_summary_stats(&start, &end)?;

    println!(
        "ByteLog Hardware Performance: {} ({})",
        period_label,
        start.format("%Y-%m-%d")
    );
    println!("-----------------------------------------------------------------------------");
    println!("CPU Average Load:        {:.1}%", summary.avg_cpu_load);
    println!("CPU Peak Load:           {:.1}%", summary.max_cpu_load);
    println!(
        "Memory Usage:            {} / {}",
        Formatter::format_bytes(summary.avg_memory_bytes),
        Formatter::format_bytes(summary.total_memory_bytes)
    );
    println!(
        "Network Download:        ↓ {}",
        Formatter::format_bytes(summary.total_net_rx_bytes)
    );
    println!(
        "Network Upload:          ↑ {}",
        Formatter::format_bytes(summary.total_net_tx_bytes)
    );
    println!();
    println!("Statistical Distribution (Min, Mean, P50, P95, Max):");
    println!("-----------------------------------------------------------------------------");
    println!(
        "{:<12} {:<6} {:>10} {:>10} {:>10} {:>10} {:>10} {:>8}",
        "METRIC", "UNIT", "MIN", "MEAN", "P50", "P95", "MAX", "SAMPLES"
    );

    for d in &dists {
        println!(
            "{:<12} {:<6} {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>8}",
            d.metric.to_uppercase(),
            d.unit,
            d.min,
            d.mean,
            d.p50,
            d.p95,
            d.max,
            d.count
        );
    }

    Ok(())
}

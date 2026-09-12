use bytelog::database::Repository;
use bytelog::typing::TypingAnalytics;
use chrono::Utc;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;

fn bench_typing_math(c: &mut Criterion) {
    c.bench_function("wpm_calculation", |b| {
        b.iter(|| TypingAnalytics::calculate_gross_wpm(1250, 300))
    });

    c.bench_function("accuracy_calculation", |b| {
        b.iter(|| TypingAnalytics::calculate_estimated_accuracy(1250, 42))
    });
}

fn bench_db_queries(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir().join("bytelog_bench");
    let _ = fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("bench.db");
    let _ = fs::remove_file(&db_path);

    let repo = Repository::open(&db_path).unwrap();
    let now = Utc::now();

    c.bench_function("summary_query", |b| {
        b.iter(|| {
            let _ = repo.get_summary_stats(&(now - chrono::Duration::days(7)), &now);
        })
    });

    let _ = fs::remove_file(&db_path);
}

criterion_group!(benches, bench_typing_math, bench_db_queries);
criterion_main!(benches);

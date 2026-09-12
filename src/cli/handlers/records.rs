use crate::database::Repository;
use crate::error::Result;
use std::path::Path;

pub fn handle_records(json: bool, csv: bool, db_path: &Path) -> Result<()> {
    let repo = Repository::open(db_path)?;
    let records = repo.get_personal_records()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&records).unwrap());
        return Ok(());
    }

    if csv {
        println!("key,formatted_value,achieved_at,notes");
        for r in &records {
            println!(
                "{},{},{},{}",
                r.key,
                r.formatted,
                r.achieved_at,
                r.notes.as_deref().unwrap_or("")
            );
        }
        return Ok(());
    }

    println!("PERSONAL HIGH WATERMARKS & RECORDS");
    println!("-----------------------------------------------------------------");
    if records.is_empty() {
        println!("No personal records established yet. Keep using your computer!");
        return Ok(());
    }

    for r in &records {
        println!(
            "{:<32} {:<16} (Achieved: {})",
            r.key, r.formatted, r.achieved_at
        );
    }

    Ok(())
}

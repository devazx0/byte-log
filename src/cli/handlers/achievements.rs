use crate::database::Repository;
use crate::error::Result;
use std::path::Path;

pub fn handle_achievements(json: bool, csv: bool, db_path: &Path) -> Result<()> {
    let repo = Repository::open(db_path)?;
    let badges = repo.get_achievements()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&badges).unwrap());
        return Ok(());
    }

    if csv {
        println!("badge_key,title,description,unlocked_at");
        for b in &badges {
            println!(
                "{},{},{},{}",
                b.key,
                b.title,
                b.description,
                b.unlocked_at.as_deref().unwrap_or("LOCKED")
            );
        }
        return Ok(());
    }

    println!("COMPUTING MILESTONE ACHIEVEMENTS");
    println!("-----------------------------------------------------------------");
    for b in &badges {
        let status_tag = if let Some(ref date) = b.unlocked_at {
            format!("[UNLOCKED: {}]", date)
        } else {
            "[LOCKED]".to_string()
        };

        println!("{:<24} {:<10} - {}", b.title, status_tag, b.description);
    }

    Ok(())
}

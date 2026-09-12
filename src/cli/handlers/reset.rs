use crate::database::Repository;
use crate::error::{PcStatsError, Result};
use std::io::{self, Write};
use std::path::Path;

pub fn handle_reset(force: bool, db_path: &Path) -> Result<()> {
    if !force {
        println!("WARNING: This will permanently and irreversibly delete all collected ByteLog telemetry.");
        println!("Configuration in config.toml will be preserved.");
        print!("To confirm data deletion, type 'DELETE': ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(PcStatsError::Io)?;

        if input.trim() != "DELETE" {
            println!("Reset cancelled. Input did not match 'DELETE'.");
            return Ok(());
        }
    }

    println!("Purging ByteLog database at {}...", db_path.display());
    let repo = Repository::open(db_path)?;
    repo.reset_database()?;
    println!("Database successfully reset to pristine state.");

    Ok(())
}

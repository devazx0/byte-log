use crate::config::{default_config_path, default_database_path, save_config, Config};
use crate::database::Repository;
use crate::error::{PcStatsError, Result};
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub fn handle_init(
    force: bool,
    config_override: Option<&Path>,
    db_override: Option<&Path>,
) -> Result<()> {
    let cfg_path = match config_override {
        Some(p) => p.to_path_buf(),
        None => default_config_path()?,
    };

    let db_path = match db_override {
        Some(p) => p.to_path_buf(),
        None => default_database_path()?,
    };

    if cfg_path.exists() && !force {
        return Err(PcStatsError::Config(format!(
            "Configuration already exists at {}. Use --force to overwrite.",
            cfg_path.display()
        )));
    }

    println!("Initializing ByteLog...");

    let mut cfg = Config::default();

    // In interactive mode, prompt for typing tracking opt-in (default: false)
    if std::io::stdin().is_terminal() {
        print!("Enable keyboard typing ergonomics analytics? (Requires input hook permissions) [y/N]: ");
        let _ = io::stdout().flush();
        let mut response = String::new();
        if io::stdin().read_line(&mut response).is_ok() {
            let trimmed = response.trim().to_lowercase();
            if trimmed == "y" || trimmed == "yes" {
                cfg.typing.enabled = true;
                println!("Typing analytics: Enabled");
            } else {
                cfg.typing.enabled = false;
                println!("Typing analytics: Disabled (Privacy Default)");
            }
        }
    } else {
        cfg.typing.enabled = false;
    }

    save_config(&cfg, Some(&cfg_path))?;
    println!("Configuration created at: {}", cfg_path.display());

    // Initialize database
    let _ = Repository::open(&db_path)?;
    println!("Database initialized at: {}", db_path.display());
    println!(
        "\nByteLog initialization complete! Run 'bytelog doctor' to verify system permissions."
    );

    Ok(())
}

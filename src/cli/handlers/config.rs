use crate::cli::args::ConfigSubcommands;
use crate::config::{default_config_path, load_config, save_config};
use crate::error::{PcStatsError, Result};
use std::path::Path;

pub fn handle_config(sub: ConfigSubcommands, config_override: Option<&Path>) -> Result<()> {
    let cfg_path = match config_override {
        Some(p) => p.to_path_buf(),
        None => default_config_path()?,
    };

    match sub {
        ConfigSubcommands::Path => {
            println!("{}", cfg_path.display());
            Ok(())
        }
        ConfigSubcommands::Get { key } => {
            let cfg = load_config(Some(&cfg_path))?;
            match key.as_str() {
                "general.collection_interval_seconds" => {
                    println!("{}", cfg.general.collection_interval_seconds)
                }
                "general.active_window_poll_interval_seconds" => {
                    println!("{}", cfg.general.active_window_poll_interval_seconds)
                }
                "activity.idle_threshold_seconds" => {
                    println!("{}", cfg.activity.idle_threshold_seconds)
                }
                "activity.anonymize_window_titles" => {
                    println!("{}", cfg.activity.anonymize_window_titles)
                }
                "typing.enabled" => println!("{}", cfg.typing.enabled),
                "typing.flush_interval_seconds" => {
                    println!("{}", cfg.typing.flush_interval_seconds)
                }
                "retention.retention_raw_days" => println!("{}", cfg.retention.retention_raw_days),
                "retention.retention_hourly_days" => {
                    println!("{}", cfg.retention.retention_hourly_days)
                }
                "tui.theme" => println!("{}", cfg.tui.theme),
                _ => {
                    return Err(PcStatsError::Config(format!(
                        "Unknown configuration key: {}",
                        key
                    )))
                }
            }
            Ok(())
        }
        ConfigSubcommands::Set { key, value } => {
            let mut cfg = load_config(Some(&cfg_path))?;
            match key.as_str() {
                "general.collection_interval_seconds" => {
                    cfg.general.collection_interval_seconds = value
                        .parse()
                        .map_err(|_| PcStatsError::Validation("Expected integer".to_string()))?;
                }
                "activity.idle_threshold_seconds" => {
                    cfg.activity.idle_threshold_seconds = value
                        .parse()
                        .map_err(|_| PcStatsError::Validation("Expected integer".to_string()))?;
                }
                "activity.anonymize_window_titles" => {
                    cfg.activity.anonymize_window_titles = value.parse().map_err(|_| {
                        PcStatsError::Validation("Expected boolean (true/false)".to_string())
                    })?;
                }
                "typing.enabled" => {
                    cfg.typing.enabled = value.parse().map_err(|_| {
                        PcStatsError::Validation("Expected boolean (true/false)".to_string())
                    })?;
                }
                "tui.theme" => {
                    cfg.tui.theme = value.clone();
                }
                _ => {
                    return Err(PcStatsError::Config(format!(
                        "Setting key '{}' is not supported via CLI",
                        key
                    )))
                }
            }
            save_config(&cfg, Some(&cfg_path))?;
            println!("Configuration updated: {} = {}", key, value);
            Ok(())
        }
        ConfigSubcommands::Edit => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
                if cfg!(windows) {
                    "notepad".to_string()
                } else {
                    "nano".to_string()
                }
            });
            let _ = std::process::Command::new(editor).arg(&cfg_path).status();
            Ok(())
        }
    }
}

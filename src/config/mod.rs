pub mod model;

pub use model::Config;

use crate::error::{PcStatsError, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

const QUALIFIER: &str = "";
const ORGANIZATION: &str = "ByteLog";
const APPLICATION: &str = "bytelog";

pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or_else(|| {
        PcStatsError::Config("Failed to resolve standard OS project directories".to_string())
    })
}

pub fn default_config_path() -> Result<PathBuf> {
    let dirs = get_project_dirs()?;
    Ok(dirs.config_dir().join("config.toml"))
}

pub fn default_data_dir() -> Result<PathBuf> {
    let dirs = get_project_dirs()?;
    Ok(dirs.data_dir().to_path_buf())
}

pub fn default_database_path() -> Result<PathBuf> {
    let data_dir = default_data_dir()?;
    Ok(data_dir.join("bytelog.db"))
}

pub fn default_log_dir() -> Result<PathBuf> {
    let dirs = get_project_dirs()?;
    Ok(dirs.data_dir().join("logs"))
}

pub fn load_config(override_path: Option<&Path>) -> Result<Config> {
    let path = match override_path {
        Some(p) => p.to_path_buf(),
        None => default_config_path()?,
    };

    if !path.exists() {
        let default_cfg = Config::default();
        save_config(&default_cfg, Some(&path))?;
        return Ok(default_cfg);
    }

    let content = fs::read_to_string(&path).map_err(|e| {
        PcStatsError::Config(format!(
            "Failed to read config at {}: {}",
            path.display(),
            e
        ))
    })?;

    toml::from_str(&content).map_err(|e| {
        PcStatsError::Config(format!(
            "Invalid TOML in config at {}: {}",
            path.display(),
            e
        ))
    })
}

pub fn save_config(config: &Config, override_path: Option<&Path>) -> Result<PathBuf> {
    let path = match override_path {
        Some(p) => p.to_path_buf(),
        None => default_config_path()?,
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(PcStatsError::Io)?;
    }

    let toml_str =
        toml::to_string_pretty(config).map_err(|e| PcStatsError::Serialization(e.to_string()))?;

    fs::write(&path, toml_str).map_err(PcStatsError::Io)?;

    Ok(path)
}

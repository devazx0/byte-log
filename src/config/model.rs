use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub activity: ActivityConfig,
    #[serde(default)]
    pub typing: TypingConfig,
    #[serde(default)]
    pub retention: RetentionConfig,
    #[serde(default)]
    pub tui: TuiConfig,
    #[serde(default = "default_categories")]
    pub categories: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_collection_interval")]
    pub collection_interval_seconds: u64,
    #[serde(default = "default_window_poll_interval")]
    pub active_window_poll_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityConfig {
    #[serde(default = "default_idle_threshold")]
    pub idle_threshold_seconds: u64,
    #[serde(default = "default_true")]
    pub anonymize_window_titles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_flush_interval")]
    pub flush_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    #[serde(default = "default_retention_raw_days")]
    pub retention_raw_days: u32,
    #[serde(default = "default_retention_hourly_days")]
    pub retention_hourly_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_tui_refresh_ms")]
    pub refresh_rate_ms: u64,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_collection_interval() -> u64 {
    5
}
fn default_window_poll_interval() -> u64 {
    2
}
fn default_idle_threshold() -> u64 {
    300
}
fn default_flush_interval() -> u64 {
    60
}
fn default_retention_raw_days() -> u32 {
    7
}
fn default_retention_hourly_days() -> u32 {
    90
}
fn default_tui_refresh_ms() -> u64 {
    1000
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_theme() -> String {
    "catppuccin".to_string()
}

fn default_categories() -> HashMap<String, String> {
    let mut map = HashMap::new();
    // Development
    map.insert("code".to_string(), "Development".to_string());
    map.insert("devenv".to_string(), "Development".to_string());
    map.insert("idea64".to_string(), "Development".to_string());
    map.insert("pycharm".to_string(), "Development".to_string());
    map.insert("nvim".to_string(), "Development".to_string());
    map.insert("vim".to_string(), "Development".to_string());
    map.insert("emacs".to_string(), "Development".to_string());
    map.insert("cargo".to_string(), "Development".to_string());
    map.insert("rustc".to_string(), "Development".to_string());
    map.insert("git".to_string(), "Development".to_string());

    // Terminal
    map.insert("alacritty".to_string(), "Terminal".to_string());
    map.insert("wezterm".to_string(), "Terminal".to_string());
    map.insert("kitty".to_string(), "Terminal".to_string());
    map.insert("foot".to_string(), "Terminal".to_string());
    map.insert("gnome-terminal".to_string(), "Terminal".to_string());
    map.insert("cmd.exe".to_string(), "Terminal".to_string());
    map.insert("powershell.exe".to_string(), "Terminal".to_string());
    map.insert("ghostty".to_string(), "Terminal".to_string());
    map.insert("windowsterminal.exe".to_string(), "Terminal".to_string());

    // Browser
    map.insert("firefox".to_string(), "Browser".to_string());
    map.insert("chrome".to_string(), "Browser".to_string());
    map.insert("msedge".to_string(), "Browser".to_string());
    map.insert("brave".to_string(), "Browser".to_string());
    map.insert("zen".to_string(), "Browser".to_string());
    map.insert("safari".to_string(), "Browser".to_string());
    map.insert("arc".to_string(), "Browser".to_string());

    // Communication
    map.insert("slack".to_string(), "Communication".to_string());
    map.insert("discord".to_string(), "Communication".to_string());
    map.insert("teams".to_string(), "Communication".to_string());
    map.insert("telegram".to_string(), "Communication".to_string());
    map.insert("signal".to_string(), "Communication".to_string());
    map.insert("element".to_string(), "Communication".to_string());

    // Productivity
    map.insert("obsidian".to_string(), "Productivity".to_string());
    map.insert("notion".to_string(), "Productivity".to_string());
    map.insert("acrobat".to_string(), "Productivity".to_string());
    map.insert("excel".to_string(), "Productivity".to_string());
    map.insert("winword".to_string(), "Productivity".to_string());

    // Media
    map.insert("spotify".to_string(), "Media".to_string());
    map.insert("vlc".to_string(), "Media".to_string());
    map.insert("mpv".to_string(), "Media".to_string());

    // Gaming
    map.insert("steam".to_string(), "Gaming".to_string());
    map.insert("steamwebhelper".to_string(), "Gaming".to_string());
    map.insert("lutris".to_string(), "Gaming".to_string());
    map.insert("heroic".to_string(), "Gaming".to_string());

    // System
    map.insert("explorer.exe".to_string(), "System".to_string());
    map.insert("finder".to_string(), "System".to_string());
    map.insert("kwin".to_string(), "System".to_string());
    map.insert("gnome-shell".to_string(), "System".to_string());

    map
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: default_collection_interval(),
            active_window_poll_interval_seconds: default_window_poll_interval(),
        }
    }
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            idle_threshold_seconds: default_idle_threshold(),
            anonymize_window_titles: default_true(),
        }
    }
}

impl Default for TypingConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            flush_interval_seconds: default_flush_interval(),
        }
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            retention_raw_days: default_retention_raw_days(),
            retention_hourly_days: default_retention_hourly_days(),
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: default_tui_refresh_ms(),
            theme: default_theme(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            activity: ActivityConfig::default(),
            typing: TypingConfig::default(),
            retention: RetentionConfig::default(),
            tui: TuiConfig::default(),
            categories: default_categories(),
        }
    }
}

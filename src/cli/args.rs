use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "bytelog", author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long, global = true, help = "Override default configuration file path")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Override default SQLite database path")]
    pub db: Option<PathBuf>,

    #[arg(long, global = true, help = "Format output as JSON")]
    pub json: bool,

    #[arg(long, global = true, help = "Format output as CSV")]
    pub csv: bool,

    #[arg(
        short = 'v',
        long,
        global = true,
        help = "Enable verbose debug logging output to stderr"
    )]
    pub verbose: bool,

    #[arg(
        short = 'q',
        long,
        global = true,
        help = "Suppress non-essential informational messages"
    )]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Initialize configuration and database migrations")]
    Init(InitArgs),

    #[command(about = "Manage background service lifecycle")]
    Service(ServiceArgs),

    #[command(about = "Internal command to run background collector")]
    Daemon(DaemonArgs),

    #[command(about = "Display aggregated usage and system summary statistics")]
    Stats(StatsArgs),

    #[command(about = "Display active application usage breakdown")]
    Apps(AppsArgs),

    #[command(about = "Display typing volume, WPM, and ergonomics statistics")]
    Typing(TypingArgs),

    #[command(about = "Display system performance and hardware telemetry distribution")]
    Performance(PerformanceArgs),

    #[command(about = "Display all-time personal high watermarks and records")]
    Records,

    #[command(about = "Display computing milestone achievement badges")]
    Achievements,

    #[command(about = "Export collected metrics to JSON or CSV")]
    Export(ExportArgs),

    #[command(about = "Query or update configuration settings")]
    Config(ConfigArgs),

    #[command(about = "Run diagnostic probes and permissions verification")]
    Doctor,

    #[command(about = "Permanently delete collected telemetry data")]
    Reset(ResetArgs),

    #[command(about = "Print version and build environment metadata")]
    Version,

    #[command(about = "Launch the interactive terminal dashboard")]
    Tui,

    #[command(about = "Display 52-week GitHub-style activity contribution heatmap")]
    Heatmap(HeatmapArgs),

    #[command(about = "Display active project and workspace tracking breakdown")]
    Projects(ProjectsArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long, help = "Overwrite existing configuration if already initialized")]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct ServiceArgs {
    #[arg(help = "Service action: install, start, stop, restart, status, uninstall")]
    pub action: ServiceAction,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ServiceAction {
    Install,
    Start,
    Stop,
    Restart,
    Status,
    Uninstall,
}

#[derive(Debug, Args)]
pub struct DaemonArgs {
    #[arg(help = "Daemon action (run)")]
    pub action: String,
}

#[derive(Debug, Args)]
pub struct StatsArgs {
    #[arg(long, help = "Show statistics for today (default)")]
    pub today: bool,

    #[arg(long, help = "Show statistics for yesterday")]
    pub yesterday: bool,

    #[arg(long, help = "Show statistics for the past 7 days")]
    pub week: bool,

    #[arg(long, help = "Show statistics for the past 30 days")]
    pub month: bool,

    #[arg(long, help = "Custom ISO 8601 date range in START..END format")]
    pub range: Option<String>,
}

#[derive(Debug, Args)]
pub struct AppsArgs {
    #[arg(long, help = "Filter for today (default)")]
    pub today: bool,

    #[arg(long, help = "Filter for the past 7 days")]
    pub week: bool,

    #[arg(long, help = "Filter for the past 30 days")]
    pub month: bool,

    #[arg(
        long,
        default_value_t = 10,
        help = "Maximum number of applications to list"
    )]
    pub limit: u32,

    #[arg(long, help = "Filter applications by category")]
    pub category: Option<String>,
}

#[derive(Debug, Args)]
pub struct TypingArgs {
    #[arg(long, help = "Show typing metrics for today (default)")]
    pub today: bool,

    #[arg(long, help = "Show typing metrics for the past 7 days")]
    pub week: bool,

    #[arg(long, help = "Show typing metrics for the past 30 days")]
    pub month: bool,

    #[arg(long, help = "Show detailed historical typing slices")]
    pub history: bool,
}

#[derive(Debug, Args)]
pub struct PerformanceArgs {
    #[arg(long, help = "Show metrics for today (default)")]
    pub today: bool,

    #[arg(long, help = "Show metrics for the past 7 days")]
    pub week: bool,

    #[arg(
        long,
        help = "Filter by specific hardware metric: cpu, mem, disk, net, temp"
    )]
    pub metric: Option<String>,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(long, default_value = "json", help = "Export format: json or csv")]
    pub format: String,

    #[arg(long, help = "Output file path (defaults to stdout)")]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        help = "Specific table to export (defaults to all summary tables)"
    )]
    pub table: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommands {
    #[command(about = "Get a configuration value")]
    Get { key: String },
    #[command(about = "Set a configuration value")]
    Set { key: String, value: String },
    #[command(about = "Print configuration file path")]
    Path,
    #[command(about = "Open configuration file in system editor")]
    Edit,
}

#[derive(Debug, Args)]
pub struct ResetArgs {
    #[arg(long, help = "Bypass interactive confirmation prompt")]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct HeatmapArgs {
    #[arg(long, help = "Show active computer time heatmap (default)")]
    pub active: bool,

    #[arg(long, help = "Show development/coding time heatmap")]
    pub coding: bool,

    #[arg(long, help = "Show typing volume/keystrokes heatmap")]
    pub typing: bool,

    #[arg(long, help = "Specific calendar year to render (defaults to current year)")]
    pub year: Option<i32>,
}

#[derive(Debug, Args)]
pub struct ProjectsArgs {
    #[arg(long, help = "Filter for today (default)")]
    pub today: bool,

    #[arg(long, help = "Filter for the past 7 days")]
    pub week: bool,

    #[arg(long, help = "Filter for the past 30 days")]
    pub month: bool,

    #[arg(
        long,
        default_value_t = 10,
        help = "Maximum number of projects to list"
    )]
    pub limit: u32,
}


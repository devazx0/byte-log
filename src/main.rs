use bytelog::cli::args::{Cli, Commands, StatsArgs};
use bytelog::cli::handlers::{
    achievements::handle_achievements, apps::handle_apps, config::handle_config,
    doctor::handle_doctor, heatmap::handle_heatmap, init::handle_init, performance::handle_performance,
    projects::handle_projects, records::handle_records, reset::handle_reset, service::handle_service,
    stats::handle_stats, typing::handle_typing, version::handle_version,
};
use bytelog::config::{default_database_path, load_config};
use bytelog::error::ExitCode;
use bytelog::export::handle_export;
use bytelog::logging::init_logging;
use bytelog::service::Daemon;
use bytelog::tui::run_tui;
use clap::Parser;
use std::io::{self, IsTerminal};
use std::process::ExitCode as StdExitCode;

fn main() -> StdExitCode {
    let cli = Cli::parse();

    let is_daemon = matches!(cli.command, Some(Commands::Daemon(_)));
    let _log_guard = match init_logging(cli.verbose, is_daemon) {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Failed to initialize logging: {}", e);
            None
        }
    };

    let config = match load_config(cli.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            return StdExitCode::from(ExitCode::ConfigurationError as u8);
        }
    };

    let db_path = match cli.db.clone() {
        Some(p) => p,
        None => match default_database_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to resolve database path: {}", e);
                return StdExitCode::from(ExitCode::DatabaseError as u8);
            }
        },
    };

    let result = match cli.command {
        Some(Commands::Init(args)) => {
            handle_init(args.force, cli.config.as_deref(), cli.db.as_deref())
        }
        Some(Commands::Service(args)) => handle_service(args.action),
        Some(Commands::Daemon(_)) => {
            let daemon = Daemon::new(config, db_path);
            daemon.run()
        }
        Some(Commands::Stats(args)) => handle_stats(args, cli.json, cli.csv, &db_path),
        Some(Commands::Apps(args)) => handle_apps(args, cli.json, cli.csv, &db_path),
        Some(Commands::Typing(args)) => handle_typing(args, cli.json, cli.csv, &db_path),
        Some(Commands::Performance(args)) => handle_performance(args, cli.json, cli.csv, &db_path),
        Some(Commands::Records) => handle_records(cli.json, cli.csv, &db_path),
        Some(Commands::Achievements) => handle_achievements(cli.json, cli.csv, &db_path),
        Some(Commands::Export(args)) => handle_export(args, &db_path),
        Some(Commands::Config(args)) => handle_config(args.action, cli.config.as_deref()),
        Some(Commands::Doctor) => handle_doctor(cli.config.as_deref(), cli.db.as_deref()),
        Some(Commands::Reset(args)) => handle_reset(args.force, &db_path),
        Some(Commands::Version) => {
            handle_version();
            Ok(())
        }
        Some(Commands::Heatmap(args)) => handle_heatmap(args, cli.json, cli.csv, &db_path),
        Some(Commands::Projects(args)) => handle_projects(args, cli.json, cli.csv, &db_path),
        Some(Commands::Tui) => run_tui(db_path, config.tui.refresh_rate_ms),
        None => {
            // Bare `bytelog` command
            if io::stdout().is_terminal() {
                run_tui(db_path, config.tui.refresh_rate_ms)
            } else {
                handle_stats(
                    StatsArgs {
                        today: true,
                        yesterday: false,
                        week: false,
                        month: false,
                        range: None,
                    },
                    cli.json,
                    cli.csv,
                    &db_path,
                )
            }
        }
    };

    match result {
        Ok(()) => StdExitCode::SUCCESS,
        Err(e) => {
            if !cli.quiet {
                eprintln!("Error: {}", e);
            }
            StdExitCode::from(e.exit_code() as u8)
        }
    }
}

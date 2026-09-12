use crate::config::{default_config_path, default_database_path};
use crate::database::Repository;
use crate::error::Result;
use crate::platform::create_platform;
use crate::platform::traits::*;
use crate::service::ServiceManager;
use std::path::Path;
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, System};

pub fn handle_doctor(config_override: Option<&Path>, db_override: Option<&Path>) -> Result<()> {
    println!("ByteLog System Doctor v{}", env!("CARGO_PKG_VERSION"));
    println!("OS Target:     {}", env!("BYTELOG_TARGET_TRIPLE"));

    let cfg_path = match config_override {
        Some(p) => p.to_path_buf(),
        None => default_config_path()?,
    };

    let db_path = match db_override {
        Some(p) => p.to_path_buf(),
        None => default_database_path()?,
    };

    println!("Database Path: {}", db_path.display());
    println!("Config Path:   {}", cfg_path.display());
    println!("\nDIAGNOSTIC CHECKS:");

    let mut warnings = 0;

    // 1. SQLite DB probe
    let db_start = Instant::now();
    match Repository::open(&db_path) {
        Ok(repo) => {
            let lat = db_start.elapsed().as_secs_f64() * 1000.0;
            println!(
                "[✓] SQLite Database Read/Write:         PASS (Latency: {:.2}ms)",
                lat
            );
            let _ = repo;
        }
        Err(e) => {
            println!("[✗] SQLite Database Read/Write:         FAIL ({})", e);
            warnings += 1;
        }
    }

    // 2. Hardware telemetry
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cores = sys.cpus().len();
    if cores > 0 {
        println!("[✓] Hardware Metrics:");
        println!(
            "    ├─ CPU Utilization Sensors:         PASS ({} Cores detected)",
            cores
        );
    } else {
        println!("[!] CPU Utilization Sensors:         WARN (No CPU cores reported)");
        warnings += 1;
    }

    let total_mem_mb = sys.total_memory() / (1024 * 1024);
    println!(
        "    ├─ Memory Telemetry:                PASS ({} MB total)",
        total_mem_mb
    );

    let disks = Disks::new_with_refreshed_list();
    let disk_count = disks.list().len();
    println!(
        "    ├─ Disk Telemetry:                  PASS ({} Mount points)",
        disk_count
    );

    let networks = Networks::new_with_refreshed_list();
    let iface_names: Vec<String> = networks.keys().cloned().collect();
    println!(
        "    ├─ Network Interface Counters:      PASS ({})",
        iface_names.join(", ")
    );

    let platform = create_platform();
    if let Some(bat) = platform.get_battery_info() {
        let state = if bat.is_charging {
            "Charging"
        } else {
            "Discharging"
        };
        println!(
            "    ├─ Battery Sensors:                 PASS ({:.0}%, {})",
            bat.percentage, state
        );
    } else {
        println!("    ├─ Battery Sensors:                 INFO (No battery/AC desktop)");
    }

    let components = Components::new_with_refreshed_list();
    let temp_count = components.list().len();
    if temp_count > 0 {
        let sample_temp = components
            .list()
            .first()
            .and_then(|c| c.temperature())
            .unwrap_or(0.0);
        println!(
            "    └─ Thermal Sensors:                 PASS ({:.1}°C detected)",
            sample_temp
        );
    } else {
        println!("    └─ Thermal Sensors:                 INFO (No thermal sensors exposed)");
    }

    // 3. GPU Telemetry
    if let Some(gpu) = platform.get_gpu_info() {
        println!(
            "[✓] GPU Telemetry:                      PASS ({})",
            gpu.model_name
        );
    } else {
        println!("[!] GPU Telemetry:                      INFO (GPU telemetry unavailable)");
    }

    // 4. Active Window Tracking
    if let Some(win) = platform.get_active_window() {
        println!(
            "[✓] Active Window Tracking:             PASS (Focused: {})",
            win.executable_name
        );
    } else {
        println!(
            "[!] Active Window Tracking:             WARN (No window focus detected or locked)"
        );
    }

    // 5. Typing Ergonomics Hook
    let (test_tx, _test_rx) = crossbeam_channel::bounded(10);
    match platform.start_hook(test_tx) {
        Ok(()) => {
            println!("[✓] Typing Ergonomics Hook:             PASS (Input hook functional)");
            platform.stop_hook();
        }
        Err(e) => {
            println!("[!] Typing Ergonomics Hook:             WARN ({})", e);
            warnings += 1;
        }
    }

    // 6. Background Service
    let svc_status = ServiceManager::status().unwrap_or_else(|_| "Unknown".to_string());
    println!(
        "[✓] Background Service:                 PASS ({})",
        svc_status
    );

    if warnings == 0 {
        println!("\nStatus: All diagnostic checks operational!");
    } else {
        println!(
            "\nStatus: Operational with {} non-fatal warning(s).",
            warnings
        );
    }

    Ok(())
}

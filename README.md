# ByteLog

**ByteLog** is an open-source, privacy-first, local-first computer usage analytics and performance monitoring tool written in native Rust.

Designed for software engineers, power users, and data-driven professionals, ByteLog bridges the gap between ephemeral low-level system monitors (like `htop` and `btop`) and invasive cloud-based trackers (like RescueTime). All metrics reside exclusively on your machine in an embedded SQLite database.

---

## Table of Contents

- [Key Highlights](#key-highlights)
- [How It Works](#how-it-works)
  - [Concurrency & Threading Pipeline](#concurrency--threading-pipeline)
  - [Privacy-Preserving Typing Analytics](#privacy-preserving-typing-analytics)
  - [Tiered Storage & Rollup Pipeline](#tiered-storage--rollup-pipeline)
- [Installation & Quickstart](#installation--quickstart)
  - [1. Global Installation (Recommended)](#1-global-installation-recommended)
  - [2. Initialize Configuration](#2-initialize-configuration)
  - [3. Verify Sensors & Permissions](#3-verify-sensors--permissions)
  - [4. Start Background Data Collection](#4-start-background-data-collection)
  - [5. Launch the Dashboard](#5-launch-the-dashboard)
- [Interactive TUI Dashboard](#interactive-tui-dashboard)
- [CLI Command Reference](#cli-command-reference)
- [Configuration & File Locations](#configuration--file-locations)
- [Troubleshooting & FAQ](#troubleshooting--faq)
- [License](#license)

---

## Key Highlights

- **Privacy Invariant:** Zero text logging, zero keystroke buffering, zero telemetry, and zero network calls. Typing analytics rely solely on volatile in-memory atomic counters.
- **Ultra-Low Footprint:** Written in native Rust without heavy async runtimes. Uses $\le 0.5\%$ single-core CPU and $\le 20\text{ MB}$ RSS RAM.
- **Local Embedded SQLite (WAL Mode):** Queries from the CLI or TUI never block or compete with the background daemon writer.
- **Dual Interfaces:** Fast, scriptable CLI with JSON/CSV export support + rich interactive Terminal User Interface (TUI) powered by `ratatui`.
- **Cross-Platform:** Native support for Windows 10/11, macOS 12+, and Linux (X11 & Wayland).

---

## How It Works

### Concurrency & Threading Pipeline

ByteLog avoids runtime executor overhead by using native OS threads (`std::thread`) and bounded communication channels (`crossbeam-channel`):

```text
┌────────────────────────────────────────────────────────────────────────┐
│                          BACKGROUND DAEMON                             │
│                                                                        │
│   ┌─────────────────────┐   ┌───────────────────┐   ┌────────────────┐ │
│   │ Telemetry Collector │   │  Activity Poller  │   │  Typing Hook   │ │
│   │  (CPU/RAM/Disk/Net) │   │ (Foreground Win)  │   │ (Key Counters) │ │
│   └──────────┬──────────┘   └─────────┬─────────┘   └────────┬───────┘ │
│              │                        │                      │         │
│              └────────────────┐       │       ┌──────────────┘         │
│                               ▼       ▼       ▼                        │
│                   ┌──────────────────────────────────────┐             │
│                   │ Bounded Channel (crossbeam-channel)  │             │
│                   └───────────────────┬──────────────────┘             │
│                                       ▼                                │
│                   ┌──────────────────────────────────────┐             │
│                   │      Dedicated DB Writer Thread      │             │
│                   └───────────────────┬──────────────────┘             │
│                                       ▼ WAL Writes                     │
│                   ┌──────────────────────────────────────┐             │
│                   │      Local SQLite (bytelog.db)       │             │
│                   └───────────────────▲──────────────────┘             │
└───────────────────────────────────────┼────────────────────────────────┘
                                        │ Non-blocking Read-Only Queries
                            ┌───────────┴───────────┐
                            │   CLI / TUI Process   │
                            │ (bytelog stats / tui) │
                            └───────────────────────┘
```

1. **Hardware Collector Thread (5s tick):** Samples CPU load, RAM, swap, disk I/O, network throughput, and battery status.
2. **Active Window Poller (2s tick):** Detects the focused foreground executable, evaluates user idle duration, and tracks workstation lock/sleep states.
3. **Dedicated SQLite Writer Thread:** All collectors push messages to a single worker thread that performs all database writes, completely eliminating database lock contention (`SQLITE_BUSY`).

---

### Privacy-Preserving Typing Analytics

Unlike conventional keyloggers, ByteLog **cannot reconstruct what you type**:

* **Atomic Counters Only:** The low-level OS hook classifies key events exclusively into four atomic counters in RAM: `StandardKey`, `Backspace`, `Enter`, and `Modifier`.
* **60-Second Slices:** Key events are held in volatile RAM and reduced every 60 seconds into gross WPM and behavioral accuracy before the raw counters are reset to zero.
* **Active Typing Filtering:** If key events occur within 3 seconds of each other, that duration is counted as active typing. Pauses to think or read are excluded from the WPM denominator to prevent artificial speed penalties.
* **Privacy Default:** Typing analytics is **disabled by default** on initial install until explicitly opted in.

$$\text{Gross WPM} = \frac{\text{Total Keystrokes} / 5}{\text{Active Typing Minutes}}$$

$$\text{Behavioral Accuracy} = \left(1 - \frac{\text{Backspaces}}{\text{Keystrokes}}\right) \times 100\%$$

---

### Tiered Storage & Rollup Pipeline

To prevent database bloat while preserving long-term insights, ByteLog compresses metrics automatically:

| Tier | Resolution | Retention | Data Stored |
| :--- | :--- | :--- | :--- |
| **Tier 1 (Raw)** | 5-second snapshots | **7 Days** | Full CPU/RAM vitals, per-window session ticks |
| **Tier 2 (Hourly)** | 1-hour rollups | **90 Days** | Hourly average & peak CPU, memory, network & disk totals |
| **Tier 3 (Daily)** | 1-day summaries | **Permanent** | Active time, top apps, keystroke totals, personal records (~150 KB/year) |

---

## Installation & Quickstart

### 1. Global Installation (Recommended)

From the project root directory, install ByteLog globally into your Cargo bin path:

```bash
cargo install --path .
```

*This places `bytelog.exe` (or `bytelog`) into your user `PATH`, allowing you to run `bytelog` from **any terminal or directory** on your computer.*

---

### 2. Initialize Configuration

Run the initialization command to create configuration files and run initial database migrations:

```bash
bytelog init
```

*During setup, you will be prompted whether to enable keyboard typing analytics (or keep it disabled for maximum privacy).*

---

### 3. Verify Sensors & Permissions

Run diagnostics to verify that operating system sensors and permissions are operational:

```bash
bytelog doctor
```

---

### 4. Start Background Data Collection

To log metrics continuously in the background, register ByteLog as an automated OS service:

#### Windows (Task Scheduler)
Open **PowerShell as Administrator** and run:
```powershell
bytelog service install
bytelog service start
bytelog service status
```

*(Alternative without Administrator rights):*
```powershell
# Start daemon silently in background right now:
Start-Process bytelog -ArgumentList "daemon run" -WindowStyle Hidden

# Add to user startup registry:
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "ByteLog" -Value "`"$env:USERPROFILE\.cargo\bin\bytelog.exe`" daemon run"
```

#### macOS (`launchd`)
```bash
bytelog service install
bytelog service start
```

#### Linux (`systemd` User Service)
```bash
bytelog service install
bytelog service start
```

#### Foreground / Testing
You can also run the daemon directly in a terminal tab:
```bash
bytelog daemon run
```

---

### 5. Launch the Dashboard

Once the daemon is active, simply open any terminal and type:

```bash
bytelog
```
*(Or `bytelog tui`). It immediately launches the interactive real-time dashboard.*

---

## Interactive TUI Dashboard

```text
┌ LIVE HARDWARE TELEMETRY ───────────────┐┌ APPLICATION USAGE (TODAY) ──────────────────────┐
│CPU Package: 18.9%                      ││APP                     TIME           %         │
│████████          19%                   ││code.exe                3h 12m         62.4%     │
│RAM: 14.79 GB / 15.82 GB (93%)          ││chrome.exe              1h 45m         28.1%     │
│Network I/O:   ↓ 2.5 KB/s   ↑ 1.2 KB/s  ││windowsterminal.exe     34m            9.5%      │
│Battery:       83% (Charging)           │└─────────────────────────────────────────────────┘
└────────────────────────────────────────┘┌ TYPING ERGONOMICS ──────────────────────────────┐
┌ HISTORICAL CPU (PAST 60 SECONDS) ──────┐│Total Keystrokes:   14,280 strokes               │
│█ ▄ ▅ █ ▃ ▂ ▅ ▆ █ ▅ ▄ ▃                 ││Average Speed:      68 WPM                       │
└────────────────────────────────────────┘│Behavioral Accuracy:96.4%                        │
                                          └─────────────────────────────────────────────────┘
[1] Today  [2] Week  [3] Month  [r] Refresh  [j/k] Scroll Apps  [Tab] Focus  [q] Quit
```

### Dashboard Keybindings

| Key | Action |
| :--- | :--- |
| `1` | Show metrics for **Today** |
| `2` | Show metrics for **This Week** |
| `3` | Show metrics for **This Month** |
| `j` / `Down` | Scroll application usage list down |
| `k` / `Up` | Scroll application usage list up |
| `Tab` | Cycle focused widget |
| `r` | Manually refresh view |
| `q` / `Esc` | Exit dashboard |

---

## CLI Command Reference

All commands support `--json` and `--csv` for scripting and data pipelines.

```bash
# General summary of today's screen time, coding hours, and system load
bytelog stats --today
bytelog stats --week
bytelog stats --month
bytelog stats --range 2026-09-01..2026-09-10

# Foreground applications breakdown
bytelog apps --limit 10
bytelog apps --category Development

# Typing ergonomics (keystrokes, gross WPM, accuracy)
bytelog typing --today
bytelog typing --history

# Hardware telemetry distributions
bytelog performance
bytelog performance --metric cpu

# Personal high watermarks (peak WPM, longest active session)
bytelog records

# Milestone badges and achievements
bytelog achievements

# Export database tables to JSON or CSV
bytelog export --format json --output usage_backup.json
bytelog export --format csv --table system_snapshots

# Read or modify configuration settings
bytelog config get typing.enabled
bytelog config set typing.enabled true
bytelog config path
bytelog config edit

# Health check probe
bytelog doctor

# Service control
bytelog service <install|start|stop|status|uninstall>

# Purge local database tables (requires typing 'DELETE')
bytelog reset
```

---

## Configuration & File Locations

Configuration is stored in a clean TOML file (`config.toml`), and metrics are stored in `bytelog.db`:

| Platform | Configuration Path | SQLite Database Path | Log Directory |
| :--- | :--- | :--- | :--- |
| **Windows** | `%APPDATA%\ByteLog\bytelog\config\config.toml` | `%APPDATA%\ByteLog\bytelog\data\bytelog.db` | `...\data\logs\daemon.log` |
| **macOS** | `~/Library/Application Support/ByteLog/config.toml` | `~/Library/Application Support/ByteLog/bytelog.db` | `.../logs/daemon.log` |
| **Linux** | `~/.config/bytelog/config.toml` | `~/.local/share/bytelog/bytelog.db` | `.../logs/daemon.log` |

To quickly edit your configuration in your default text editor:
```bash
bytelog config edit
```

---

## Troubleshooting & FAQ

### 1. `bytelog service install` gives `Access is denied` (Windows)
Windows Scheduled Tasks (`schtasks`) require administrative privileges.
* **Fix:** Right-click PowerShell or Windows Terminal and select **"Run as Administrator"**, then re-run `bytelog service install`.
* *Alternatively, use the user-level startup shortcut detailed in the Quickstart section.*

### 2. Typing stats are stuck at 0
1. Check if typing analytics is enabled:
   ```bash
   bytelog config get typing.enabled
   ```
2. If `false`, enable it:
   ```bash
   bytelog config set typing.enabled true
   ```
3. Restart the daemon (`bytelog service stop && bytelog service start` or restart `bytelog daemon run`).
4. **Remember the 60-second window:** Keystrokes are counted in memory and committed to the database once every 60 seconds.

### 3. Release build takes a long time linking
ByteLog enables full whole-program Link-Time Optimization (`lto = "fat"`) and `codegen-units = 1` in release mode for maximum runtime speed and minimal binary size. Linking all 190+ dependencies can take 1–3 minutes on the final step.
* For fast development builds, simply run `cargo build` (debug mode).
* To speed up release builds, set `lto = "thin"` in `Cargo.toml`.

---

## License

Dual-licensed under either of:
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))

at your option.

# Product Requirements Document (PRD)
# PCStats: Privacy-First Local Computer Usage & Performance Analytics

**Document Version:** 1.0.0-PROD-SPEC  
**Status:** Approved for Implementation  
**Target Release:** MVP v0.1.0  
**Target Rust Toolchain:** Rust 1.82+ (Stable Edition 2021)  
**Primary Architect:** Principal Systems Architect & Engineering Lead  
**Last Updated:** 2026-09-12  

---

## 1. Executive Summary

**PCStats** is an open-source, privacy-first, local-first computer usage analytics and performance monitoring tool built in native Rust. Designed for software engineers, power users, terminal enthusiasts, and data-driven professionals, PCStats bridges the divide between low-level system diagnostic monitors (such as `htop`, `btop`, and Activity Monitor) and high-level productivity time-trackers (such as RescueTime and ActivityWatch).

Operating exclusively as an unprivileged, lightweight local background daemon alongside a companion CLI and Terminal User Interface (TUI), PCStats continuously captures hardware health telemetry (CPU, RAM, disk, network, battery, thermals, and GPU where supported) correlated with active foreground application usage, computer session active/idle states, and behavioral typing ergonomics (gross keystrokes, backspace correction ratios, and statistical WPM).

PCStats strictly adheres to three fundamental engineering guarantees:
1. **Zero Outbound Telemetry & Local Storage:** 100% of telemetry is stored locally in an embedded SQLite database using Write-Ahead Logging (WAL). No cloud backend, user accounts, external databases, or network transmissions exist within the codebase.
2. **Privacy by Construction:** The application physically cannot log typed characters, passwords, message contents, browser URLs, clipboard text, or screen pixels. Keystroke analytics are reduced in volatile memory to atomic counters before raw OS event structures are discarded.
3. **Ultra-Low Resource Footprint:** The daemon operates within a strict budget of $\le 0.5\%$ single-core CPU utilization and $\le 20\text{ MB}$ Resident Set Size (RSS) memory, ensuring the observer does not degrade the workstation it monitors.

---

## 2. Product Vision

To give technical users total, uncompromised sovereignty over their digital activity and hardware vitals through an analytical lens that feels like:
> **"GitHub contribution graphs + system health telemetry + typing mechanics + Spotify Wrapped for your personal computer."**

PCStats empowers individuals to quantify their productivity patterns, evaluate hardware stress points, measure typing ergonomics, and celebrate personal computing milestones across days, months, and years—with absolute data isolation and zero surveillance risk.

---

## 3. Problem Statement

Modern computing environments lack unified, privacy-respecting self-analytics:
1. **Cloud Surveillance & Data Leaks:** Mainstream commercial productivity trackers upload window titles, application logs, and behavioral heartbeats to third-party cloud infrastructure, presenting unacceptable privacy risks for proprietary code, security credentials, and personal communications.
2. **Ephemerality of System Monitors:** Tools like `htop`, `btop`, and Windows Task Manager provide high-fidelity real-time vitals but discard data immediately upon closing, preventing long-term trend analysis, baseline profiling, and correlation with application usage.
3. **Bloated Resource Consumption:** Cross-platform monitoring utilities written in Electron, Python, or Node.js frequently consume 150 MB to 500 MB of RAM and 2% to 5% of CPU, ironically contributing to the performance degradation they are measuring.
4. **Keylogger Paranoia vs. Ergonomic Insight:** Users interested in typing velocity and typing fatigue have had to choose between synthetic web typing tests (which do not reflect actual day-to-day coding) and intrusive keyloggers that capture sensitive passwords and raw text.
5. **Platform Silos:** Existing tools are heavily fragmented across macOS-only menu bar apps, Linux-only bash scripts, and Windows-specific background utilities.

---

## 4. Goals

- **G-1 (Strict Local-First):** Store all metrics in a local, single-file SQLite database with zero external server dependencies.
- **G-2 (Privacy Invariant):** Ensure raw keystroke characters, clipboard data, window contents, and private document titles cannot be captured or stored.
- **G-3 (Ultra-Low Overhead):** Cap background daemon utilization at $\le 0.5\%$ average single-core CPU and $\le 20\text{ MB}$ RSS RAM.
- **G-4 (Cross-Platform Support):** Support Windows 10/11, macOS 12+, and Linux (X11 and Wayland) via a unified platform abstraction layer with capability-based degradation.
- **G-5 (Dual User Interface):** Provide a scriptable, sub-50ms CLI for headless queries alongside an interactive, information-dense TUI dashboard built with `ratatui`.
- **G-6 (Implementation Readiness):** Specify every database schema, trait, CLI argument, error code, and algorithm with sufficient rigor that an autonomous coding agent can implement the MVP without ambiguity.

---

## 5. Non-Goals

The following features are explicitly out of scope for the MVP:
- **NG-1 (Surveillance / Monitoring):** No stealth execution, no hidden process disguises, no employee surveillance features, and no remote administrative monitoring.
- **NG-2 (Deep Packet Inspection):** No network packet payload interception, DNS request logging, or URL sniffing.
- **NG-3 (Keylogging):** No recording of typed characters, words, sentences, or key sequence buffers.
- **NG-4 (Cloud Synchronization):** No remote backups, hosted API sync, or centralized dashboards.
- **NG-5 (Antivirus / Security EDR):** No malware scanning, signature matching, or executable blocking.
- **NG-6 (AST / Source Code Snooping):** No automatic scanning or parsing of git repository source files or project trees.

---

## 6. Target Users

1. **Software Engineers & Developers:** Desiring objective metrics on time spent in IDEs, compilers, local test suites, and terminal environments.
2. **Terminal Power Users & Linux Enthusiasts:** Users operating within tmux, neovim, and tiling window managers who demand native CLI/TUI tools without graphical bloat.
3. **Mechanical Keyboard & Ergonomics Enthusiasts:** Users tuning custom keymaps and hardware who want real-world typing metrics (WPM, backspace penalty) across daily workflows.
4. **Hardware Tuners & Homelab Operators:** Users tracking thermal throttling, memory trends, battery degradation, and network bandwidth over extended uptime cycles.

---

## 7. User Personas

### Persona A: Alex (Senior Systems Engineer)
- **Environment:** Arch Linux (Wayland / Sway) & macOS M3 Max.
- **Pain Point:** Wants to know how much time is spent in Neovim vs. Slack without running an Electron-based time tracker that leaks company proprietary window titles.
- **Workflow:** Runs `pcstats` as a systemd user unit / launchd agent; pipes JSON output into custom scripts; views terminal graphs at end of week.

### Persona B: Beatrice (Full-Stack Dev & Keyboard Hobbyist)
- **Environment:** Windows 11 workstation with custom split mechanical keyboard.
- **Pain Point:** Wants to measure whether switching to an ergonomic keyboard layout reduces backspace error corrections and increases sustained typing speed in VS Code.
- **Workflow:** Checks `pcstats typing --today` and `pcstats apps` daily to evaluate typing ergonomics without worrying about password leakage.

### Persona C: Charlie (Student & Digital Minimalist)
- **Environment:** ThinkPad T480 running Fedora Linux.
- **Pain Point:** Experiences battery drain from poorly optimized background applications; wants an honest log of daily computer uptime and active work vs. idle media consumption.
- **Workflow:** Inspects `pcstats performance` and `pcstats stats --week` to manage study habits and battery health.

---

## 8. User Stories

| ID | As a... | I want to... | So that I can... | Priority |
|---|---|---|---|---|
| **US-01** | User | Run `pcstats stats --today` | Instantly view my active screen time, coding hours, and network usage today. | P0 (MVP) |
| **US-02** | User | Launch `pcstats tui` | Monitor live hardware vitals and daily usage distributions in an interactive dashboard. | P0 (MVP) |
| **US-03** | User | Track active foreground applications | Understand my distribution of time across tools (IDE, browser, terminal). | P0 (MVP) |
| **US-04** | User | Track typing volume and estimated speed | Measure my typing productivity without compromising passwords or text. | P0 (MVP) |
| **US-05** | User | Install PCStats as a background service | Keep metrics running seamlessly across reboots without manual terminal launches. | P0 (MVP) |
| **US-06** | User | Run `pcstats doctor` | Verify whether OS permissions (e.g., input monitoring, accessibility) are active. | P0 (MVP) |
| **US-07** | User | Export my database to JSON or CSV | Analyze my personal metrics in external data tools (Python, SQLite Studio). | P0 (MVP) |
| **US-08** | User | Run `pcstats reset` | Completely and irreversibly purge collected local data on demand. | P0 (MVP) |
| **US-09** | User | Inspect hardware health (battery, thermals) | Monitor laptop battery cycle health and CPU thermal throttling trends. | P1 (MVP) |
| **US-10** | User | Review personal records (PRs) | View all-time high watermarks for longest session, peak WPM, and max uptime. | P1 (MVP) |
| **US-11** | User | View unlocked achievements | Celebrate local computing milestones without gamified notifications. | P1 (MVP) |
| **US-12** | User | Configure custom app categories | Tailor application groupings to my personal development stack. | P1 (MVP) |

---

## 9. Core Features

1. **System Health Engine:** Low-overhead periodic sampling of CPU (per-core and package), Memory (Physical & Swap), Storage Volumes, Network Interfaces, Battery, and Thermals.
2. **Activity & Session Tracker:** Heuristic-driven detection of computer session start, active user time, idle timeouts, sleep/lock state transitions, and day-boundary rollovers.
3. **Application Usage Profiler:** Foreground window tracking mapped to sanitized executable names, categorizing time into Development, Browser, Terminal, Communication, etc.
4. **Privacy-Preserving Typing Ergonomics:** Ring-buffer event tracking calculating gross keystroke frequency, backspace correction rates, and statistical WPM without character capture.
5. **Time-Series Aggregation & Compaction:** Tiered rollups (5-second raw samples $\to$ 1-hour intervals $\to$ 1-day rollups) preserving sub-millisecond query latency over multi-year datasets.
6. **Dual User Interface:**
   - Scriptable CLI with human-readable and JSON/CSV outputs.
   - Rich terminal dashboard built on `ratatui` with sparklines, gauges, and tables.
7. **Service & Diagnostic Subsystem:** Single-command service management (`install`, `start`, `stop`, `status`) and self-diagnosing `doctor` command.

---

## 10. MVP Scope

The MVP encompasses the minimal set of features required to provide an autonomous, reliable, and privacy-preserving computer telemetry experience:
- Single Rust native binary (`pcstats`).
- Embedded SQLite with WAL mode, foreign keys, and tiered retention.
- CPU, Memory, Disk, Network, Battery, and Temperature collectors via `sysinfo` and platform hooks.
- GPU detection (where native sysfs or standard APIs exist; graceful degradation to unavailable).
- Active foreground application tracking on Windows, macOS, and Linux (X11 + supported Wayland compositors).
- Inactivity / Idle detection (configurable, default: 300s).
- Ephemeral typing event analytics (gross strokes, backspace count, estimated WPM, behavioral accuracy).
- Application categorization (built-in rule table + user TOML config).
- Personal records tracking (longest uptime, peak WPM, max keystrokes/day).
- Standard CLI commands: `init`, `service`, `daemon`, `stats`, `apps`, `typing`, `performance`, `records`, `achievements`, `export`, `config`, `doctor`, `reset`, `version`.
- Full-screen interactive TUI (`pcstats tui` or bare `pcstats` when attached to TTY).
- Background service management:
  - Windows: Task Scheduler logon task (Interactive user session).
  - macOS: `launchd` LaunchAgent (`~/Library/LaunchAgents`).
  - Linux: `systemd --user` unit (`~/.config/systemd/user`).

---

## 11. Post-MVP Scope

Features reserved for subsequent minor releases (v0.2.0+):
- **Local Git Repository Analytics:** Opt-in scanning of configured repository paths for commit frequency, branch changes, and language distributions (without code inspection).
- **Audio Stream Inactivity Suppression:** Detecting active CoreAudio / WASAPI / PipeWire output to prevent false-positive idle states during media playback or calls.
- **"Computer Wrapped" Annual Review:** Generating a comprehensive yearly Markdown and SVG infographic summary of computing metrics.
- **Database Backup & Restore Commands:** Dedicated `pcstats backup` and `pcstats restore` with checksum verification.
- **Lightweight Local HTTP API:** Opt-in local-only loopback daemon endpoint (`127.0.0.1`) for Prometheus scraping or custom desktop widgets.

---

## 12. Functional Requirements

- **FR-01 (Continuous Sampling):** The background collector must sample system metrics at a default 5-second interval (configurable between 1s and 60s).
- **FR-02 (Foreground Detection):** The application monitor must poll the active window every 2 seconds and accumulate active duration against the associated application ID.
- **FR-03 (Idle State Management):** If no user input occurs for `idle_threshold_seconds` (default: 300s), session tracking must transition to `IDLE` and stop crediting active application time.
- **FR-04 (Sleep & Lock Detection):** System sleep, hibernate, or screen lock events must immediately close active computer sessions and application sessions.
- **FR-05 (Typing Metric Pipeline):** The typing monitor must ingest raw OS key-down events, extract only event category (`StandardKey`, `Backspace`, `Enter`, `Modifier`), increment atomic counters, and drop raw key codes immediately.
- **FR-06 (Data Aggregation):** The database engine must execute a scheduled rollup job every 60 minutes, converting raw 5-second samples into hourly averages/peaks and pruning expired raw records.
- **FR-07 (CLI Query Responsiveness):** Historical CLI queries (e.g., `pcstats stats --week`) must execute against pre-computed aggregates in $< 50\text{ ms}$.
- **FR-08 (Graceful Hardware Degradation):** Missing hardware sensors (e.g., no battery on desktop, no GPU sensors) must result in clean `null` storage and UI indicators without logging errors or crashing.
- **FR-09 (Doctor Verification):** `pcstats doctor` must perform live probes of SQLite access, OS permissions, sensor availability, and service registration, returning human-actionable remediation instructions.
- **FR-10 (Application Sanitization):** Window titles captured from OS window managers must be sanitized by default; only the process executable name is stored unless explicitly enabled by user configuration.
- **FR-11 (Data Deletion):** `pcstats reset` must permanently delete all metric records upon explicit confirmation by typing `DELETE`.
- **FR-12 (Data Export):** `pcstats export` must stream all historical records to stdout or a file in valid JSON or RFC 4180 CSV format without memory exhaustion.

---

## 13. Non-Functional Requirements

- **NFR-01 (Resource Bounds - CPU):** Daemon idle background CPU usage must not exceed 0.5% of a single CPU core under normal workloads.
- **NFR-02 (Resource Bounds - RAM):** Daemon Resident Set Size (RSS) must remain $\le 20\text{ MB}$ during steady-state background collection.
- **NFR-03 (Disk Write Efficiency):** Disk writes must be batched using SQLite WAL mode; `fsync` operations must not occur more frequently than once every 30 seconds during normal tracking.
- **NFR-04 (Binary Footprint):** Release binary size must not exceed 15 MB (stripped).
- **NFR-05 (Startup Latency):** CLI query commands must complete cold execution in $< 80\text{ ms}$.
- **NFR-06 (Fault Tolerance):** Hard power loss or kill signals (`SIGKILL`) must never corrupt historical data. The SQLite WAL file must automatically recover on subsequent process start.
- **NFR-07 (Portability):** The application must be compilable across `x86_64` and `aarch64` targets for Windows, macOS, and Linux with zero dynamic library dependencies outside of standard OS system libraries.
- **NFR-08 (Concurrency Safety):** Thread communication must use bounded channels to prevent unbounded memory growth under high system load.
- **NFR-09 (Maintainability):** Codebase must maintain zero compiler warnings on stable Rust and pass `cargo clippy -- -D warnings`.

---

## 14. Privacy Requirements

### Mandatory Data Invariants
1. **Zero Text Capture:** No string typed by the user, whether letters, numbers, symbols, whitespace, or clipboard text, shall ever be read into a persistent buffer or written to storage.
2. **Zero URL / Document Introspection:** Window titles captured from OS window managers must be sanitized. By default, only process executable names (e.g., `code.exe`, `firefox`, `slack`) are stored. Private document names (e.g., `bank_statement.pdf - Adobe Acrobat`) must have their title strings stripped.
3. **Local Enclave:** The application binary must contain zero networking libraries with outbound transmission logic. No telemetry SDKs (e.g., Sentry, Segment, PostHog) are permitted in dependencies.
4. **Granular Opt-In:** Typing monitoring must default to `false` in configuration and require explicit user activation during `pcstats init` or via `pcstats config set typing.enabled true`.
5. **Auditable Storage:** All metrics reside in an unencrypted standard SQLite database file inspectable by the user using the standard `sqlite3` CLI.

---

## 15. Security Requirements

- **SR-01 (Least Privilege):** The daemon must run strictly within the unprivileged user's GUI session. It must never require `root` or `Administrator` privileges for standard execution.
- **SR-02 (File Permissions):** Database files and configuration files must be created with restrictive POSIX permissions (`0600` for files, `0700` for directories) on macOS and Linux, and restricted ACLs inheriting only the current user SID on Windows.
- **SR-03 (IPC Security):** CLI-to-daemon communication must rely on local OS process table inspection and SQLite WAL concurrency, eliminating network listening sockets.
- **SR-04 (Input Hook Safety):** Low-level keyboard hooks must be purely passive (listen-only) and never modify, suppress, or inject synthetic input events (`CallNextHookEx` on Windows, `kCGEventTapOptionListenOnly` on macOS).
- **SR-05 (Command Injection Prevention):** Application tracking and diagnostic probes must never pass unescaped process or environment strings to OS shell interpreters. All OS API interactions must use direct system calls or typed Rust bindings.

---

## 16. Cross-Platform Requirements

| Capability | Windows (10/11 x64/ARM64) | macOS (Intel & Apple Silicon) | Linux X11 | Linux Wayland |
|---|---|---|---|---|
| **CPU/RAM/Disk/Net** | Fully supported (`sysinfo`) | Fully supported (`sysinfo`) | Fully supported (`sysinfo`) | Fully supported (`sysinfo`) |
| **Active Window** | `GetForegroundWindow` + `QueryFullProcessImageNameW` | `NSWorkspace.sharedWorkspace.frontmostApplication` | `_NET_ACTIVE_WINDOW` via `x11rb` | Compositor IPC (Hyprland, Sway, GNOME ext); Degrades gracefully |
| **Idle Detection** | `GetLastInputInfo` | `CGEventSourceSecondsSinceLastEventType` | `XScreenSaverQueryInfo` | `org.freedesktop.ScreenSaver` / `login1` D-Bus |
| **Typing Volume** | `SetWindowsHookExW(WH_KEYBOARD_LL)` | `CGEventTapCreate` (Input Monitoring) | `XRecord` or `/dev/input` | `/dev/input` (requires `input` group); otherwise disabled |
| **Battery Metrics** | `GetSystemPowerStatus` | `IOPSGetPowerSourceDescription` | `/sys/class/power_supply` | `/sys/class/power_supply` |
| **GPU Metrics** | DXGI / D3D12 / NVML fallback | Metal Device query / IOKit (VRAM) | `amdgpu`/`i915` sysfs or NVML | `amdgpu`/`i915` sysfs or NVML |
| **Service Mgr** | Task Scheduler (Logon Task) | `launchd` LaunchAgent | `systemd --user` | `systemd --user` |

---

## 17. CLI Specification

The CLI is structured via `clap` using the `derive` pattern. The binary is named `pcstats`.

### Global Flags
- `--config <PATH>`: Override default configuration path.
- `--db <PATH>`: Override default SQLite database path.
- `--json`: Output raw data formatted as JSON.
- `--csv`: Output tabular data formatted as CSV.
- `-v, --verbose`: Enable debug logging output.
- `-q, --quiet`: Suppress non-essential output.

### Subcommands Specification

#### 1. `pcstats init`
- **Syntax:** `pcstats init [--force]`
- **Description:** Initializes user configuration directory, writes default `config.toml`, applies embedded database migrations, and interactively prompts for optional typing monitoring.
- **Exit Codes:** `0` (Success), `2` (Already initialized without `--force`), `4` (Filesystem error).

#### 2. `pcstats service <ACTION>`
- **Syntax:** `pcstats service <install|start|stop|restart|status|uninstall>`
- **Description:** Manages native OS background daemon registration without manual script wrangling.
- **Exit Codes:** `0` (Success), `5` (Service manager error), `6` (Service not installed).

#### 3. `pcstats daemon run`
- **Syntax:** `pcstats daemon run`
- **Description:** Invoked by OS service managers to run the telemetry loop in the foreground. Blocks until a termination signal is received.

#### 4. `pcstats stats`
- **Syntax:** `pcstats stats [--today | --yesterday | --week | --month | --range <START>..<END>]`
- **Default:** `--today`.
- **Output (Plaintext):**
```text
PCStats Summary: Today (2026-09-12)
Active Screen Time:   6h 42m 10s (Idle: 1h 15m)
Development Time:     4h 18m 00s [████████████░░░░░░░░] 64.1%
Total Keystrokes:     24,180 strokes
Gross Typing Speed:   76 WPM (Peak: 112 WPM)
Correction Ratio:     4.2% (Estimated Accuracy: 95.8%)

Average CPU Load:     18.4% (Max: 89.2%)
Average Memory:       11.4 GB / 32.0 GB (35.6%)
Network Consumption:  ↓ 3.82 GB  ↑ 412 MB
```

#### 5. `pcstats apps`
- **Syntax:** `pcstats apps [--today | --week | --month] [--limit <N>] [--category <CAT>]`
- **Default:** `--today --limit 10`.
- **Output:**
```text
Top Applications (Today)
RANK  APPLICATION       CATEGORY       ACTIVE TIME    PERCENT   SESSIONS
1     Visual Studio Code Development    4h 18m 22s     64.1%     14
2     Zen Browser       Browser        1h 22m 10s     20.4%     28
3     Alacritty         Terminal       0h 48m 15s     12.0%     42
4     Slack             Communication  0h 13m 23s      3.5%      9
```

#### 6. `pcstats typing`
- **Syntax:** `pcstats typing [--today | --week | --month | --history]`
- **Output:** Displays keystroke volume, backspace counts, estimated accuracy, and WPM distribution.

#### 7. `pcstats performance`
- **Syntax:** `pcstats performance [--today | --week] [--metric <cpu|mem|disk|net|temp>]`
- **Output:** Displays statistical distribution (Min, Mean, P50, P95, Max) of hardware performance metrics.

#### 8. `pcstats records`
- **Syntax:** `pcstats records`
- **Output:** Displays all-time personal high watermarks with timestamps.

#### 9. `pcstats achievements`
- **Syntax:** `pcstats achievements`
- **Output:** Displays unlocked and in-progress local milestone badges.

#### 10. `pcstats doctor`
- **Syntax:** `pcstats doctor`
- **Description:** Runs comprehensive verification of database, permissions, and sensors.

#### 11. `pcstats export`
- **Syntax:** `pcstats export --format <json|csv> [--output <PATH>] [--table <NAME>]`
- **Description:** Streams historical data in specified format.

#### 12. `pcstats config`
- **Syntax:** `pcstats config <get <KEY> | set <KEY> <VALUE> | path | edit>`
- **Description:** Reads or updates configuration keys in `config.toml`.

#### 13. `pcstats reset`
- **Syntax:** `pcstats reset [--force]`
- **Description:** Destructive database purge requiring explicit `DELETE` confirmation.

#### 14. `pcstats version`
- **Syntax:** `pcstats version`
- **Description:** Prints version, target triple, commit hash, and compiler metadata.

---

## 18. TUI Specification

The TUI provides a high-density, real-time interactive terminal dashboard built using `ratatui` and `crossterm`. It is launched via `pcstats tui` or automatically when running `pcstats` with no arguments in an interactive terminal.

### Layout Geometry
```text
┌─────────────────────────────────────────────────────────────────────────────┐
│ PCSTATS v0.1.0  ● DAEMON ACTIVE (PID 14202)         2026-09-12 19:30:00 UTC │
├─────────────────────────────────────┬───────────────────────────────────────┤
│ LIVE HARDWARE TELEMETRY             │ APPLICATION USAGE (TODAY)             │
│ CPU Package [||||||||||||░░░]  42%  │ VS Code        5h 12m [████████░] 68% │
│  ├─ Core 0: 48%   Core 1: 36%       │ Zen Browser    1h 45m [██░░░░░░░] 22% │
│  └─ Core 2: 41%   Core 3: 43%       │ Ghostty        0h 38m [█░░░░░░░░]  8% │
│ RAM Usage   12.4 / 32.0 GB     38%  │ Slack          0h 14m [░░░░░░░░░]  2% │
│ Swap Usage   0.1 /  8.0 GB      1%  ├───────────────────────────────────────┤
│ Network I/O ↓ 4.2 MB/s   ↑ 310 KB/s │ TYPING ERGONOMICS (TODAY)             │
│ Disk I/O    R: 1.2 MB/s  W: 480 KB/s│ Strokes: 28,412   Estimated WPM: 82   │
│ Battery     88% (Discharging, 4h10m)│ Backspaces: 1,102 Accuracy: 96.1%     │
│ Temp        CPU: 54°C    GPU: 48°C  │ Peak Speed: 114 WPM                   │
├─────────────────────────────────────┴───────────────────────────────────────┤
│ HISTORICAL CPU UTILIZATION (PAST 60 MINUTES)                                │
│ 100% |                                                                      │
│  50% |       ╭──╮      ╭───────╮                                            │
│   0% |_______│  │______│       ╰─────────────────────────────────────────── │
├─────────────────────────────────────────────────────────────────────────────┤
│ [1] Today  [2] Week  [3] Month  [d] Doctor  [p] Pause  [c] Config  [q] Quit │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Visual Standards & UX Behavior
- **Theme Palette:** High-contrast neutral base (dark gray `#1e1e2e`, foreground `#cdd6f4`) with semantic accent colors:
  - System Normal / Green: `#a6e3a1`
  - System Warning / Yellow: `#f9e2af`
  - System Critical (>85%) / Red: `#f38ba8`
  - Primary Visual Accents: Cyan `#89dceb`, Lavender `#b4befe`
- **Fallback Mode:** Detects terminal capabilities via `crossterm`. If truecolor is unavailable, falls back to 16 ANSI colors with ASCII-only symbols (`#`, `=`, `-`, `*`).
- **Interaction Model:**
  - `q` / `Esc`: Exit TUI.
  - `Tab` / `Shift+Tab`: Cycle active focus pane.
  - `1`, `2`, `3`: Switch time horizon between Today, Last 7 Days, Last 30 Days.
  - `r`: Force instant query refresh.
  - `Up` / `Down` or `j` / `k`: Scroll application list.
  - Auto-refresh tick rate: Default 1.0 second.

---

## 19. System Metrics

### Metrics Definition and Sampling Specifications

| Metric Category | Target Data Points | Units | Sampling Freq | Aggregation Type | Source Provider |
|---|---|---|---|---|---|
| **CPU** | Total utilization, Per-core utilization, Frequency | %, MHz | 5s | Avg, Min, Max, P95 | `sysinfo::System::cpus` |
| **CPU Load** | Load average 1m, 5m, 15m | Float | 5s | Instantaneous | `sysinfo::System::load_average` |
| **Memory** | Total, Used, Available, Free | Bytes | 5s | Avg, Max | `sysinfo::System::total_memory`, `used_memory` |
| **Swap** | Total swap, Used swap | Bytes | 5s | Avg, Max | `sysinfo::System::total_swap`, `used_swap` |
| **Storage** | Total disk space, Used space, Available space | Bytes | 60s | Min Available | `sysinfo::Disks::new_with_refreshed_list` |
| **Disk I/O** | Read bytes/sec, Write bytes/sec | Bytes/s | 5s | Sum, Avg Rate | OS Counters / `/proc/diskstats` |
| **Network** | Total RX bytes, Total TX bytes, RX rate, TX rate | Bytes, Bytes/s | 5s | Sum, Peak Rate | `sysinfo::Networks::new_with_refreshed_list` |
| **Battery** | Percentage, Charging state, Cycle count, Health % | %, Enum, Count | 30s | Latest | OS Power API (`GetSystemPowerStatus`, `IOPS`, `sysfs`) |
| **Thermal** | Package CPU temp, GPU temp | °Celsius | 10s | Max, Avg | `sysinfo::Components` / Hardware sensors |
| **GPU** | Core utilization, VRAM used, VRAM total | %, Bytes | 10s | Avg, Max | Native Platform Fallback (NVML/DXGI/sysfs) |

---

## 20. Application Tracking

### Process Attribution & Foreground Polling
Active application time is credited strictly to the process owning the current OS foreground window while the user is non-idle:
- Foreground window is polled every 2 seconds.
- Process ID (PID) is resolved to the full executable path.
- Path is normalized to the executable file name (lowercased, e.g., `code`, `alacritty`, `firefox`).
- Window titles are discarded by default to prevent leaking document names, URLs, or repository titles.

### Application Sanitization & Categorization
Applications are categorized using a deterministic rules engine:
```text
Default Categorization Rules:
- Development:   code, devenv, idea64, pycharm, nvim, vim, emacs, cargo, rustc, git
- Terminal:      alacritty, wezterm, kitty, foot, gnome-terminal, cmd.exe, powershell.exe, ghostty
- Browser:       firefox, chrome, msedge, brave, zen, safari, arc
- Communication: slack, discord, teams, telegram, signal, element
- Productivity:  obsidian, notion, acrobat, excel, winword
- Media:         spotify, vlc, mpv
- Gaming:        steam, steamwebhelper, lutris, heroic
- System:        explorer.exe, finder, kwin, gnome-shell
```
Users can configure custom overrides in `config.toml` using exact process names or glob patterns.

---

## 21. Activity Tracking

### Inactivity State Machine
```text
User Input Event (Key or Mouse)
  │
  ▼
Reset Monotonic Idle Clock (idle_time = 0s)
  │
  ├─ If idle_time < idle_threshold (default 300s) ──> State = ACTIVE
  │     └─ Accumulate active_duration on current computer_session & application_session
  │
  └─ If idle_time >= idle_threshold ───────────────> State = IDLE
        └─ Freeze active_duration at (Now - idle_threshold)
        └─ Accumulate idle_duration on current computer_session
```

### Sleep, Lock, and Rollover Handlers
- **Lock / Sleep Event:** Immediately closes current open sessions with `is_open = 0`.
- **Wake / Unlock Event:** Opens a new `computer_session` with `started_at = CurrentTimestamp`.
- **Midnight Rollover:** A session crossing 00:00:00 is closed at 23:59:59 and a new session opened at 00:00:00 attributed to the new calendar day.

---

## 22. Typing Analytics

### Privacy Invariant Architecture
To guarantee absolute privacy while producing meaningful ergonomic data:
1. **No Raw Text Buffer:** The application allocates zero memory buffers for storing sequences of keys.
2. **Immediate Event Reduction:** Key input events are reduced at the OS hook callback into one of four atomic integer operations:
   - `KEY_COUNT.fetch_add(1, Ordering::Relaxed)`
   - `BACKSPACE_COUNT.fetch_add(1, Ordering::Relaxed)`
   - `ENTER_COUNT.fetch_add(1, Ordering::Relaxed)`
   - `MODIFIER_COUNT.fetch_add(1, Ordering::Relaxed)`
3. **Event Flushing:** Every 60 seconds, if `KEY_COUNT > 0`, the background worker flushes a typing slice to the active typing session record and resets memory accumulators to zero.

### Mathematical Definitions: WPM & Accuracy
- **Standard Word Normalization:** $1 \text{ Word} \equiv 5 \text{ Keystrokes}$.
- **Gross Typing Speed (Gross WPM):**
  $$\text{Gross WPM} = \frac{\text{Total Keystrokes} / 5}{\text{Active Typing Duration in Minutes}}$$
  *(Where Active Typing Duration is the sum of 5-second sampling buckets in which at least 1 keystroke occurred).*
- **Behavioral Correction Ratio & Estimated Accuracy:**
  $$\text{Correction Ratio} = \frac{\text{Backspace Count}}{\text{Total Keystrokes}}$$
  $$\text{Estimated Accuracy (\%)} = \max\left(0.0, \, \left(1.0 - \frac{\text{Backspace Count} \times 2}{\text{Total Keystrokes}}\right) \times 100\right)$$
  *(Labeled explicitly as "Behavioral Accuracy (Estimated)" in all views).*

---

## 23. Historical Analytics

PCStats supports multi-horizon historical queries:
- **Intervals:** Today, Yesterday, Last 7 Days, Last 30 Days, Last 90 Days, Year-to-Date, All Time.
- **Aggregations:** Computes active time, coding proportion, total keystrokes, average WPM, average CPU load, peak RAM, and total network consumption.
- **Period-over-Period Deltas:** Calculates percentage and absolute deltas (e.g., "This week vs Last week: Active screen time +14%, Coding time +2.5h, Avg CPU -4%").

---

## 24. Achievements

PCStats includes deterministic, local milestone badges evaluated during daily rollups:
| Badge Identifier | Title | Requirement | Category |
|---|---|---|---|
| `keystrokes_10k` | **Century Scribe** | 10,000 keystrokes recorded | Typing |
| `keystrokes_100k` | **Keyboard Warrior** | 100,000 keystrokes recorded | Typing |
| `keystrokes_1m` | **Million Keystroke Club** | 1,000,000 keystrokes recorded | Typing |
| `speed_80wpm` | **Touch Typist** | Sustained 80 WPM over 5 minutes | Typing |
| `speed_100wpm` | **Century Speed** | Sustained 100 WPM over 5 minutes | Typing |
| `coding_100h` | **Code Machine** | 100 hours in `Development` applications | Productivity |
| `uptime_7d` | **Uptime Disciple** | 7 consecutive days of system uptime | Hardware |
| `uptime_30d` | **Uptime Demon** | 30 consecutive days of system uptime | Hardware |
| `night_owl_50h` | **Night Owl** | 50 hours of active usage between 00:00 and 05:00 | Ergonomics |

---

## 25. Personal Records

High-watermark metrics stored in `personal_records` with timestamp of attainment:
- `longest_computer_session_seconds`: Longest continuous active session without exceeding idle limits.
- `peak_keystrokes_single_day`: Highest total keystrokes in a single calendar day.
- `fastest_sustained_wpm_5m`: Highest gross WPM sustained over a continuous 5-minute active window.
- `longest_system_uptime_seconds`: Highest uptime observed before a reboot.
- `longest_streak_days`: Consecutive calendar days with $\ge 2$ hours of active computer usage.

---

## 26. Data Retention

Hierarchical tiered rollup pipeline:
```text
Tier 1: Raw Telemetry (5s snapshots)
  │  Retention: 7 days (Configurable: 1–30 days)
  │  Storage Size: ~30 MB / week
  ▼
Tier 2: Hourly Rollups (1-hour averages, sums, and maxes)
  │  Retention: 90 days (Configurable: 30–365 days)
  │  Storage Size: ~5 MB / quarter
  ▼
Tier 3: Daily Summary Records (1-day aggregates)
     Retention: Permanent (Indefinite)
     Storage Size: ~150 KB / year
```
- Daily automated compaction runs at daemon startup and every 24 hours.
- Pruning query: `DELETE FROM system_snapshots WHERE timestamp < datetime('now', '-7 days')`.
- Reclaims unused space via `PRAGMA incremental_vacuum(1000)`.

---

## 27. Database Architecture

- **Engine:** SQLite 3 via `rusqlite` with `bundled` feature.
- **Connection Pragmas:**
  ```sql
  PRAGMA journal_mode = WAL;
  PRAGMA synchronous = NORMAL;
  PRAGMA busy_timeout = 5000;
  PRAGMA foreign_keys = ON;
  PRAGMA cache_size = -2000;
  PRAGMA auto_vacuum = INCREMENTAL;
  ```
- **Thread Topology:** Single writer thread receiving batched inserts via bounded channel; multiple reader connections for CLI/TUI queries.

---

## 28. Database Schema

```sql
-- Schema Migrations Tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    description TEXT NOT NULL
);

-- Applications Master Table
CREATE TABLE IF NOT EXISTS applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    executable_name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'Other',
    is_ignored BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_apps_exec ON applications(executable_name);
CREATE INDEX IF NOT EXISTS idx_apps_cat ON applications(category);

-- Computer Usage Sessions (System Active/Idle cycles)
CREATE TABLE IF NOT EXISTS computer_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    active_duration_seconds INTEGER NOT NULL DEFAULT 0,
    idle_duration_seconds INTEGER NOT NULL DEFAULT 0,
    is_open BOOLEAN NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_comp_sessions_started ON computer_sessions(started_at);

-- Application Usage Sessions (Foreground Active Windows)
CREATE TABLE IF NOT EXISTS application_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    computer_session_id INTEGER REFERENCES computer_sessions(id) ON DELETE CASCADE,
    application_id INTEGER NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    duration_seconds INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_app_sessions_started ON application_sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_app_sessions_app ON application_sessions(application_id);

-- Ephemeral Typing Slices
CREATE TABLE IF NOT EXISTS typing_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    application_id INTEGER REFERENCES applications(id) ON DELETE SET NULL,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP NOT NULL,
    active_typing_seconds INTEGER NOT NULL,
    keystrokes INTEGER NOT NULL,
    backspaces INTEGER NOT NULL,
    enters INTEGER NOT NULL,
    modifiers INTEGER NOT NULL,
    estimated_wpm REAL NOT NULL,
    estimated_accuracy REAL NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_typing_started ON typing_sessions(started_at);

-- Raw System Telemetry Snapshots (Tier 1: 5s interval, 7-day retention)
CREATE TABLE IF NOT EXISTS system_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cpu_utilization REAL NOT NULL,
    memory_used_bytes INTEGER NOT NULL,
    memory_total_bytes INTEGER NOT NULL,
    swap_used_bytes INTEGER NOT NULL,
    disk_read_bytes_sec INTEGER NOT NULL,
    disk_write_bytes_sec INTEGER NOT NULL,
    net_rx_bytes_sec INTEGER NOT NULL,
    net_tx_bytes_sec INTEGER NOT NULL,
    cpu_temp_celsius REAL,
    gpu_utilization REAL,
    battery_percent REAL,
    battery_charging BOOLEAN
);
CREATE INDEX IF NOT EXISTS idx_snapshots_time ON system_snapshots(timestamp);

-- Hourly Aggregated Telemetry (Tier 2: 90-day retention)
CREATE TABLE IF NOT EXISTS hourly_aggregates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hour_timestamp TIMESTAMP NOT NULL UNIQUE,
    avg_cpu_utilization REAL NOT NULL,
    max_cpu_utilization REAL NOT NULL,
    avg_memory_used_bytes INTEGER NOT NULL,
    max_memory_used_bytes INTEGER NOT NULL,
    total_net_rx_bytes INTEGER NOT NULL,
    total_net_tx_bytes INTEGER NOT NULL,
    total_disk_read_bytes INTEGER NOT NULL,
    total_disk_write_bytes INTEGER NOT NULL,
    sample_count INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_hourly_time ON hourly_aggregates(hour_timestamp);

-- Daily Summary Aggregates (Tier 3: Permanent)
CREATE TABLE IF NOT EXISTS daily_aggregates (
    date TEXT PRIMARY KEY, -- 'YYYY-MM-DD'
    total_active_seconds INTEGER NOT NULL DEFAULT 0,
    total_idle_seconds INTEGER NOT NULL DEFAULT 0,
    total_keystrokes INTEGER NOT NULL DEFAULT 0,
    total_backspaces INTEGER NOT NULL DEFAULT 0,
    average_wpm REAL NOT NULL DEFAULT 0.0,
    top_application_id INTEGER REFERENCES applications(id),
    avg_cpu_utilization REAL NOT NULL DEFAULT 0.0,
    peak_cpu_utilization REAL NOT NULL DEFAULT 0.0,
    avg_memory_used_bytes INTEGER NOT NULL DEFAULT 0,
    total_net_bytes INTEGER NOT NULL DEFAULT 0
);

-- Personal Records High Watermarks
CREATE TABLE IF NOT EXISTS personal_records (
    record_key TEXT PRIMARY KEY,
    record_value REAL NOT NULL,
    formatted_value TEXT NOT NULL,
    achieved_at TIMESTAMP NOT NULL,
    notes TEXT
);

-- Unlocked Achievements
CREATE TABLE IF NOT EXISTS achievements (
    badge_key TEXT PRIMARY KEY,
    unlocked_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    progress REAL NOT NULL DEFAULT 1.0
);
```

---

## 29. Rust Architecture

### Concurrency Model: Synchronous Threaded Pipeline
Rather than adding an asynchronous runtime (Tokio) which adds binary bloat and complicates low-level OS hook integration, PCStats employs native OS threads (`std::thread`) and bounded crossbeam channels (`crossbeam-channel`).

- **Hardware Collector Thread:** Awakens every 5 seconds, queries `sysinfo`, reads platform sensors, and dispatches a `TelemetrySnapshot` message.
- **Window Focus Poll Thread:** Awakens every 2 seconds, resolves foreground application and idle state, and dispatches an `ActivityUpdate` message.
- **Input Hook Thread:** Runs synchronous native OS message loop (`GetMessageW`, `CFRunLoopRun`, or `x11rb` event pump) to receive raw OS events, increments atomic counters, and dispatches non-blocking summaries.
- **Database Writer Thread:** The sole thread holding write access to the SQLite connection, receiving messages from a bounded channel (`capacity = 1000`) and executing batched writes inside transactions.

---

## 30. Module Structure

```text
pcstats/
├── Cargo.toml
├── build.rs
├── README.md
├── migrations/
│   └── V01__init.sql
├── src/
│   ├── main.rs                  # CLI entry, dispatch, and signal handling
│   ├── lib.rs                   # Core domain re-exports
│   ├── error.rs                 # Domain error definitions via thiserror
│   ├── cli/                     # CLI argument parser & subcommands
│   │   ├── mod.rs
│   │   ├── args.rs              # Clap struct definitions
│   │   ├── formatters.rs        # Table, JSON, CSV output helpers
│   │   └── handlers/            # Handlers for stats, apps, typing, doctor, etc.
│   ├── config/                  # Configuration TOML management
│   │   ├── mod.rs
│   │   └── model.rs
│   ├── database/                # SQLite connection pool & repository
│   │   ├── mod.rs
│   │   ├── connection.rs        # Connection management & WAL pragmas
│   │   ├── migrations.rs        # Embedded SQL migration runner
│   │   └── repository.rs        # CRUD & aggregation queries
│   ├── collectors/              # Periodic hardware collectors
│   │   ├── mod.rs
│   │   ├── cpu.rs
│   │   ├── memory.rs
│   │   ├── disk.rs
│   │   ├── network.rs
│   │   ├── battery.rs
│   │   ├── thermal.rs
│   │   └── gpu.rs
│   ├── activity/                # Window tracking & session state machine
│   │   ├── mod.rs
│   │   ├── session_manager.rs
│   │   └── categorizer.rs
│   ├── typing/                  # Typing metrics & safety ring-buffer
│   │   ├── mod.rs
│   │   ├── analytics.rs         # WPM & accuracy math
│   │   └── state.rs
│   ├── platform/                # OS-specific abstraction implementations
│   │   ├── mod.rs
│   │   ├── traits.rs            # Provider traits
│   │   ├── windows.rs           # Win32 APIs (windows-sys)
│   │   ├── macos.rs             # Cocoa / CoreGraphics bindings
│   │   └── linux.rs             # X11 / Wayland / D-Bus
│   ├── service/                 # OS background service wrappers
│   │   ├── mod.rs
│   │   ├── daemon.rs            # Core daemon event loop
│   │   ├── windows_task.rs
│   │   ├── macos_launchd.rs
│   │   └── linux_systemd.rs
│   ├── tui/                     # Ratatui dashboard
│   │   ├── mod.rs
│   │   ├── app.rs               # State & navigation
│   │   ├── ui.rs                # Layout & widgets
│   │   └── events.rs            # Terminal input pump
│   └── export/                  # CSV & JSON stream generators
│       └── mod.rs
└── tests/
    ├── aggregation_tests.rs
    ├── session_state_tests.rs
    ├── typing_math_tests.rs
    └── database_migration_tests.rs
```

---

## 31. Platform Abstraction

```rust
// src/platform/traits.rs
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveWindowInfo {
    pub process_id: u32,
    pub executable_name: String,
    pub window_title: Option<String>,
}

pub trait ActiveApplicationProvider: Send + Sync {
    fn get_active_window(&self) -> Option<ActiveWindowInfo>;
}

pub trait UserActivityProvider: Send + Sync {
    fn get_idle_time(&self) -> Result<Duration, PlatformError>;
    fn is_session_locked(&self) -> bool;
}

pub trait RawInputHookProvider: Send + Sync {
    fn start_hook(
        &self, 
        event_sender: crossbeam_channel::Sender<TypingInputEvent>
    ) -> Result<(), PlatformError>;
    fn stop_hook(&self);
}

#[derive(Debug, Clone, Copy)]
pub enum TypingInputEvent {
    StandardKey,
    Backspace,
    Enter,
    Modifier,
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Permission denied by operating system: {0}")]
    PermissionDenied(String),
    #[error("API not supported on this platform: {0}")]
    Unsupported(String),
    #[error("OS system call failed with error code: {0}")]
    SystemCallFailed(i32),
}
```

---

## 32. Windows Implementation

- **Foreground Window:** `GetForegroundWindow()` retrieves the active `HWND`. `GetWindowThreadProcessId` resolves the PID. `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ...)` and `QueryFullProcessImageNameW` extract the executable path.
- **Idle Detection:** `GetLastInputInfo(&LASTINPUTINFO)` computes idle time:
  $$\text{Idle Ms} = \text{GetTickCount64}() - \text{LASTINPUTINFO.dwTime}$$
- **Typing Hook:** Installs `SetWindowsHookExW(WH_KEYBOARD_LL, LowLevelKeyboardProc, ...)`. The callback inspects `vkCode` for `VK_BACK`, `VK_RETURN`, and modifiers, dispatching atomic events and invoking `CallNextHookEx`.
- **Service Strategy:** Standard Windows Services run in **Session 0** where desktop interaction and `GetForegroundWindow()` are prohibited. Therefore, `pcstats service install` registers a **Windows Scheduled Task** with an `AtLogon` trigger running in the interactive user session.

---

## 33. macOS Implementation

- **Foreground Window:** `NSWorkspace.sharedWorkspace.frontmostApplication` returns `NSRunningApplication` with `localizedName` and `bundleIdentifier`. Requires no special permissions.
- **Idle Detection:** `CGEventSourceSecondsSinceLastEventType(kCGEventSourceStateCombinedSessionState, kCGAnyInputEventType)` returns idle seconds without elevated privileges.
- **Typing Hook:** `CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, kCGEventTapOptionListenOnly, CGEventMaskBit(kCGEventKeyDown), ...)` requires **Input Monitoring** permission. If denied, the tap returns `NULL` and typing is gracefully disabled.
- **Service Strategy:** Deploys a LaunchAgent plist to `~/Library/LaunchAgents/com.pcstats.daemon.plist` managed via `launchctl`.

---

## 34. Linux Implementation

- **X11 Support:** `_NET_ACTIVE_WINDOW` on root window resolves active window; `_NET_WM_PID` resolves process ID; `XScreenSaverQueryInfo` reads idle time; `XRecord` or `/dev/input` handles key counts.
- **Wayland Support:** Wayland security isolates client windows and keystrokes.
  - Hyprland: Subscribes to `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`.
  - Sway: Communicates over `$SWAYSOCK` using IPC command `get_tree`.
  - Generic Wayland: Degrades gracefully if compositor IPC is unavailable.
  - Typing tracking on Wayland requires user membership in the `input` group to read `/dev/input/event*`.
- **Service Strategy:** Deploys a systemd user unit to `~/.config/systemd/user/pcstats.service` managed via `systemctl --user`.

---

## 35. Configuration

Stored in standard platform locations:
- **Windows:** `%APPDATA%\pcstats\config.toml`
- **macOS:** `~/Library/Application Support/pcstats/config.toml`
- **Linux:** `$XDG_CONFIG_HOME/pcstats/config.toml` (defaults to `~/.config/pcstats/config.toml`)

```toml
[general]
collection_interval_seconds = 5
active_window_poll_interval_seconds = 2

[activity]
idle_threshold_seconds = 300
anonymize_window_titles = true

[typing]
enabled = false
flush_interval_seconds = 60

[retention]
retention_raw_days = 7
retention_hourly_days = 90

[tui]
refresh_rate_ms = 1000
theme = "catppuccin"

[categories]
"code" = "Development"
"nvim" = "Development"
"alacritty" = "Terminal"
"zen" = "Browser"
"firefox" = "Browser"
"slack" = "Communication"
```

---

## 36. Background Service

- **Windows:** Scheduled task configured to launch at user logon with `RunLevel: LeastPrivilege`.
- **macOS:** LaunchAgent registered at `~/Library/LaunchAgents/com.pcstats.daemon.plist` with `RunAtLoad` and `KeepAlive`.
- **Linux:** Systemd user unit at `~/.config/systemd/user/pcstats.service` with `Restart=always` and `WantedBy=graphical-session.target`.

---

## 37. Permissions

| OS | Feature | Permission Required | Prompt Mechanism | Fallback if Denied |
|---|---|---|---|---|
| **macOS** | Typing Analytics | Input Monitoring | System Settings $\to$ Privacy $\to$ Input Monitoring | Disable typing hook; warn in `doctor` |
| **macOS** | Window Title | Accessibility | System Settings $\to$ Privacy $\to$ Accessibility | Fall back to process name via `NSWorkspace` |
| **Windows** | Typing Analytics | None (Desktop hook) | None (Standard Win32 hook) | N/A |
| **Windows** | Process Discovery | `PROCESS_QUERY_LIMITED_INFORMATION` | Unprivileged user token | Omit inaccessible system process names |
| **Linux** | Typing Analytics (Wayland) | Group Membership (`input`) | `sudo usermod -a -G input $USER` | Disable typing hook; notify in `doctor` |
| **Linux** | Active Window (Wayland) | Compositor IPC socket | Environment variable (`$SWAYSOCK`, etc.) | Record session without window attribution |

---

## 38. Error Handling

### Exit Codes Enum
```rust
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    InvalidArguments = 2,
    ConfigurationError = 3,
    DatabaseError = 4,
    ServiceManagerError = 5,
    PermissionDenied = 6,
    UnsupportedPlatform = 7,
}
```

### Fault Recovery Strategies
- **Database Corruption:** Moves `pcstats.db` to `pcstats.db.corrupt.<timestamp>`, logs critical alert, and initializes fresh database.
- **Database Locked (`SQLITE_BUSY`):** Handled via 5000ms busy timeout and 3-stage exponential backoff.
- **Clock Skew:** Uses monotonic `std::time::Instant` for session intervals; uses `SystemTime` strictly for recording wall-clock timestamps.

---

## 39. Logging

- Structured logging powered by `tracing` and `tracing-subscriber`.
- CLI writes to `stderr` only when `--verbose` is passed.
- Daemon writes to rotating daily log files (`daemon.log`) capped at 10 MB total size via `tracing-appender`.
- Sensitive strings (window titles, process paths) are sanitized prior to logging.

---

## 40. Diagnostics

`pcstats doctor` evaluates system health and permissions:
```text
PCStats System Doctor v0.1.0
OS Target:         linux (x86_64) [Wayland / Sway]
Database Path:     /home/alex/.local/share/pcstats/pcstats.db (WAL Mode, 4.2 MB)
Config Path:       /home/alex/.config/pcstats/config.toml

DIAGNOSTIC CHECKS:
[✓] SQLite Database Read/Write:         PASS (Latency: 0.42ms)
[✓] Hardware Metrics:
    ├─ CPU Utilization Sensors:         PASS (16 Cores detected)
    ├─ Memory Telemetry:                PASS (32,042 MB total)
    ├─ Disk Telemetry:                  PASS (2 Mount points)
    ├─ Network Interface Counters:      PASS (wlan0, eth0)
    ├─ Battery Sensors:                 PASS (94%, Discharging)
    └─ Thermal Sensors:                 PASS (k10temp: 52.4°C)
[!] GPU Telemetry:                      WARN (amdgpu sysfs found, utilization 0%)
[✓] Active Window Tracking:             PASS (Sway IPC connected via $SWAYSOCK)
[!] Typing Ergonomics:                  WARN (User not in 'input' group)
    └─ Resolution: Run 'sudo usermod -a -G input $USER' and log out/in.
[✓] Background Service:                 PASS (systemd user unit 'pcstats.service' active)

Status: Operational (with 1 non-fatal degradation).
```

---

## 41. Export

- **Formats:** JSON (streaming array of objects) and RFC 4180 CSV.
- **Command:** `pcstats export --format <json|csv> [--output <PATH>] [--table <NAME>]`.
- Supports streaming directly to stdout for piping into tools like `jq`, Python, or DuckDB.

---

## 42. Backup & Restore

- **Backup:** Post-MVP will support `pcstats backup --output <PATH>`, using SQLite Online Backup API (`sqlite3_backup_init`) to take consistent snapshots without stopping the daemon.
- **Manual Backup:** Users can safely back up `pcstats.db` while the daemon runs by executing:
  ```bash
  sqlite3 ~/.local/share/pcstats/pcstats.db ".backup 'pcstats-backup.db'"
  ```

---

## 43. Installation

- **Development Installation:**
  ```bash
  cargo build --release
  cp target/release/pcstats ~/.local/bin/
  pcstats init
  pcstats service install
  pcstats service start
  ```
- **Public Distribution (Post-MVP):** Pre-compiled standalone binaries on GitHub Releases; package definitions for Homebrew, WinGet, and Arch User Repository (AUR).

---

## 44. Packaging

- **Release Profile (`Cargo.toml`):**
  ```toml
  [profile.release]
  opt-level = 3
  lto = "fat"
  codegen-units = 1
  panic = "abort"
  strip = true
  ```
- **Cross-Compilation Matrix:** Targets `x86_64` and `aarch64` across Windows (MSVC), macOS (Apple Silicon & Intel), and Linux (GNU and musl static binaries).

---

## 45. Testing Strategy

- **Unit Tests:** Math calculations (Gross WPM, Behavioral Accuracy), idle state transitions, and configuration parsers.
- **Integration Tests:** SQLite migrations, WAL concurrency, repository aggregations, and CLI argument parsing.
- **Platform Mocking:** Abstract trait mocking for hardware collectors to enable headless CI testing.
- **Property-Based Testing:** Using `proptest` for time rollup math and day-boundary rollover handling.
- **Performance Benchmarks:** Criterion benchmarks for database write transactions and CLI query execution time.

---

## 46. Acceptance Criteria

- **AC-01 (Application Tracking):** Given VS Code is focused for 180s followed by Terminal for 60s, `pcstats apps --today` records approximately 3m for VS Code and 1m for Terminal ($\pm 4\text{s}$).
- **AC-02 (Idle Timeout):** Given an idle threshold of 300s, leaving inputs untouched for 360s causes active session accrual to freeze at 300s, categorizing the remaining 60s as Idle.
- **AC-03 (Typing Privacy):** Given typing tracking is active, typing a password string stores the numerical count of keystrokes without any character or string representation existing in SQLite.
- **AC-04 (Offline Operation):** Disconnecting all network interfaces results in continuous, error-free telemetry collection and CLI/TUI operation.
- **AC-05 (CLI Query Latency):** Querying `pcstats stats --week` against a 90-day database completes in $< 50\text{ ms}$.
- **AC-06 (Crash Recovery):** Terminating the daemon with `kill -9` leaves the database intact and allows clean startup with automatic WAL recovery.
- **AC-07 (Resource Overhead):** Steady-state background daemon execution consumes $\le 0.5\%$ CPU and $\le 20\text{ MB}$ RSS RAM.

---

## 47. Performance Requirements

- **CPU Overhead:** $\le 0.5\%$ single-core average during steady-state sampling.
- **Memory Footprint:** $\le 20\text{ MB}$ RSS during continuous execution.
- **Disk Write Rate:** Batched writes with `fsync` frequency capped at $\le 1$ flush per 30 seconds.
- **Database Size Growth:** $\le 5\text{ MB}$ per month of continuous tracking under 7-day raw retention.
- **CLI Startup Time:** Cold launch execution time $< 80\text{ ms}$.

---

## 48. Threat Model

- **Threat 1: Malicious Local App Reads Database:** Database permissions restricted via OS ACLs (`0600` on POSIX). Database contains no plain text, credentials, or URLs.
- **Threat 2: Keylogger Accusation:** Memory model contains zero character buffers. Ingested events only update numerical counters. Code is fully auditable and open-source.
- **Threat 3: Process Hijacking via Dynamic Libraries:** Binary statically links dependencies where possible and relies on bundled SQLite.
- **Threat 4: Log File Disk Exhaustion:** Rotating daily logs capped at 10 MB total size.

---

## 49. Documentation

The repository must provide:
- `README.md`: High-level overview, quickstart guide, and screenshots.
- `PRIVACY.md`: Comprehensive explanation of data invariants and typing privacy architecture.
- `CLI_REFERENCE.md`: Complete syntax and flag documentation for all subcommands.
- `ARCHITECTURE.md`: Detailed breakdown of module structure, thread model, and platform traits.
- `CONTRIBUTING.md`: Coding standards, clippy rules, and PR checklist.
- `TROUBLESHOOTING.md`: Resolution steps for platform permissions and sensor warnings.

---

## 50. Development Workflow

- **Toolchain:** Rust 1.82+ stable.
- **Formatting:** `cargo fmt -- --check`.
- **Linting:** `cargo clippy --all-targets -- -D warnings`.
- **Testing:** `cargo test --all-targets`.
- **CI Pipeline:** Automated GitHub Actions matrix testing Windows, macOS, and Linux builds on every push.

---

## 51. Implementation Phases

```text
Phase 0: Project Bootstrap & Foundations
  ├── Cargo workspace initialization & build profiles
  ├── Domain errors (thiserror) & configuration loader
  └── Embedded SQLite migration harness (rusqlite + V01 SQL)

Phase 1: Hardware Metrics Engine
  ├── Sysinfo integration (CPU, Memory, Disk, Network)
  ├── Platform power/battery & thermal providers
  └── Tier 1 raw snapshot persistence loop

Phase 2: Activity & Application Monitoring
  ├── Platform ActiveApplicationProvider (Win32, macOS, Linux)
  ├── UserIdleStatus providers (GetLastInputInfo, CoreGraphics, XScreenSaver)
  └── Session state machine & day-boundary rollover logic

Phase 3: Privacy-Preserving Typing Pipeline
  ├── Non-blocking OS keyboard event hooks
  ├── Atomic counters & typing slice aggregator
  └── Mathematical Gross WPM & accuracy calculators

Phase 4: Data Aggregation & Analytics Engine
  ├── Tier 2 (hourly) and Tier 3 (daily) rollup compaction workers
  ├── Personal records high-watermark evaluators
  └── Historical query repository

Phase 5: CLI Subsystems & Commands
  ├── Clap parser and subcommands implementation
  ├── Table, JSON, and CSV formatters
  └── Diagnostic doctor verification engine

Phase 6: Interactive Terminal User Interface (TUI)
  ├── Ratatui layout grid & theme engine
  ├── Real-time telemetry sparklines & gauges
  └── Keyboard navigation & event pump

Phase 7: Background Service Wrappers
  ├── Windows Task Scheduler registration
  ├── macOS LaunchAgent plist generator
  └── Linux systemd user unit manager

Phase 8: Hardening, Benchmarking & Release
  ├── Cross-platform CI automation
  ├── Latency benchmarks & memory leak profiling
  └── Binary release artifact packaging
```

---

## 52. Implementation Order

An autonomous coding agent must execute implementation in the following strict order:
1. Initialize Cargo workspace with dependencies specified in Section 44.
2. Implement `src/error.rs` using `thiserror`.
3. Implement `src/config/mod.rs` and `src/config/model.rs`.
4. Implement `src/database/connection.rs` (enforcing WAL and PRAGMAs) and `src/database/migrations.rs`.
5. Implement `src/collectors/` wrapping `sysinfo` for CPU, Memory, Disk, and Network.
6. Implement `src/platform/traits.rs`.
7. Implement `src/platform/windows.rs` using `windows-sys`.
8. Implement `src/platform/macos.rs` using Cocoa / CoreGraphics bindings.
9. Implement `src/platform/linux.rs` handling X11 (`x11rb`) and Wayland detection.
10. Implement `src/activity/session_manager.rs` managing sessions and idle transitions.
11. Implement `src/typing/state.rs` and `src/typing/analytics.rs`.
12. Implement `src/service/daemon.rs` connecting collectors and channels to SQLite writer.
13. Implement `src/database/repository.rs` containing aggregation rollup queries.
14. Implement `src/cli/args.rs` and subcommand handlers (`stats`, `apps`, `typing`, `records`, `export`, `reset`).
15. Implement `src/cli/handlers/doctor.rs`.
16. Implement `src/service/` background service wrappers for Windows, macOS, and Linux.
17. Implement `src/tui/` interactive dashboard using `ratatui`.
18. Implement integration tests validating AC-01 through AC-07.

---

## 53. Future Roadmap

- **v0.2.0:** Local opt-in Git repository activity monitoring.
- **v0.3.0:** Audio stream activity detection to prevent false idle timeouts during meetings.
- **v0.4.0:** "Computer Wrapped" annual visual summary generator.
- **v0.5.0:** Standalone local web dashboard (via local loopback HTTP server).
- **v1.0.0:** Production-hardened enterprise-ready personal release.

---

## 54. Open Questions & Resolved Decisions

- **Q1: Should Wayland compositor support be mandatory?**  
  *Resolution:* No. Wayland security restricts client introspection. PCStats guarantees support for Sway and Hyprland via their respective IPC protocols, degrading gracefully to active time without application attribution on unsupported Wayland compositors.
- **Q2: Should typing tracking be enabled by default?**  
  *Resolution:* No. Privacy and operating system permission friction dictate that typing analytics must be explicitly opt-in.
- **Q3: Why not use a standard Windows Service?**  
  *Resolution:* Windows Services run in Session 0, where `GetForegroundWindow()` returns `NULL` and `SetWindowsHookEx` fails. A Windows Scheduled Task running at user logon in the interactive session is technically required.

---

## 55. Technical Decisions Summary

| Decision | Selected Choice | Rejected Alternative | Rationale |
|---|---|---|---|
| **Architecture** | Synchronous Native Threads | Tokio Async Runtime | Avoids runtime bloat; simplifies synchronous OS hook event pumps. |
| **Database** | SQLite 3 (`rusqlite` bundled) | DuckDB / sled / PostgreSQL | Single-file reliability, zero external dependencies, robust WAL concurrency. |
| **TUI Library** | `ratatui` + `crossterm` | `cursive` / `crosstermion` | De facto industry standard in Rust, active maintenance, rich widget ecosystem. |
| **CLI Parser** | `clap` (derive macro) | `argh` / `lexopt` | Full-featured help generation, shell completions, and typed subcommand validation. |
| **Path Standards** | `directories` crate | Hardcoded paths | Strictly adheres to OS-specific conventions (XDG, AppData, Library). |

---

## 56. Final MVP Acceptance Checklist

- [ ] Binary compiles with zero warnings on stable Rust (`cargo check --all-targets`).
- [ ] `cargo clippy --all-targets -- -D warnings` passes with zero lints.
- [ ] Database migrations execute cleanly on a fresh initialization.
- [ ] `pcstats init` initializes directory structure and writes valid default `config.toml`.
- [ ] Daemon background memory usage is $\le 20\text{ MB}$ RSS after 12 hours of execution.
- [ ] Active application tracking attributes focus time to the foreground process.
- [ ] Idle state activates after 300s of inactivity, stopping application session accrual.
- [ ] Typing tracking calculates keystroke counts, WPM, and accuracy without storing text.
- [ ] `pcstats stats --today` executes in $< 50\text{ ms}$.
- [ ] `pcstats doctor` correctly verifies all sensors, permissions, and service states.
- [ ] `pcstats export --format json` produces valid, parseable JSON conforming to schema.
- [ ] `pcstats reset` completely purges historical tables upon typing `DELETE`.
- [ ] Hard process kill (`SIGKILL`) causes zero database corruption upon restart.
- [ ] TUI renders cleanly on an 80x24 terminal without wrapping or visual artifacts.

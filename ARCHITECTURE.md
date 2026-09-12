# ByteLog Architecture & Design Specification

This document describes the internal engineering architecture, threading model, database pragmas, and platform abstraction layers of ByteLog.

---

## 1. Concurrency & Threading Pipeline

ByteLog intentionally uses native OS threads (`std::thread`) and bounded channels (`crossbeam-channel`) rather than an asynchronous runtime like Tokio. This eliminates async executor bloat, reduces compilation overhead, and integrates cleanly with synchronous OS event hooks (`GetMessageW`, `CFRunLoopRun`, `x11rb`).

```text
┌──────────────────────────────────────────────────────────┐
│                      DAEMON PROCESS                      │
│                                                          │
│  ┌────────────────────┐          ┌────────────────────┐  │
│  │ MetricCollector    │          │ ActiveWindowHook   │  │
│  │ Thread (5s tick)   │          │ Thread (2s poll)   │  │
│  └─────────┬──────────┘          └─────────┬──────────┘  │
│            │ TelemetryMessage              │ ActivityMsg │
│            ▼                               ▼             │
│    ┌───────────────────────────────────────────────┐     │
│    │  Bounded Channel (crossbeam_channel::Sender)  │     │
│    └───────────────────────┬───────────────────────┘     │
│                            │                             │
│  ┌────────────────────┐    │                             │
│  │ TypingHook Thread  │    │                             │
│  │ (OS Event Tap/Hook)├────┤ EventMessage                │
│  └────────────────────┘    │                             │
│                            ▼                             │
│            ┌───────────────────────────────┐             │
│            │   Database Writer / Worker    │             │
│            │   Thread (Sole SQLite Writer) │             │
│            └───────────────┬───────────────┘             │
│                            │ Writes via WAL              │
│                            ▼                             │
│            ┌───────────────────────────────┐             │
│            │       SQLite Database         │             │
│            └───────────────▲───────────────┘             │
└────────────────────────────┼─────────────────────────────┘
                             │
                  Concurrent │ Read-Only Queries
                             │ (Fast, Non-blocking)
             ┌───────────────┴───────────────┐
             │       CLI / TUI Process       │
             └───────────────────────────────┘
```

---

## 2. Database Pragmas & Compaction

The embedded SQLite database is configured with the following PRAGMAs on every connection:
- `PRAGMA journal_mode = WAL;` (Enables simultaneous readers while the background daemon writes).
- `PRAGMA synchronous = NORMAL;` (Reduces disk I/O; `fsync` batched $\le 1$ per 30 seconds).
- `PRAGMA busy_timeout = 5000;` (Eliminates `SQLITE_BUSY` errors).
- `PRAGMA foreign_keys = ON;` (Enforces referential integrity).
- `PRAGMA cache_size = -2000;` (Caps memory page cache at 2 MB).
- `PRAGMA auto_vacuum = INCREMENTAL;` (Permits incremental space reclamation).

### Tiered Rollup Pipeline
- **Tier 1 (5s Raw Snapshots):** Retained for 7 days.
- **Tier 2 (Hourly Aggregates):** Retained for 90 days.
- **Tier 3 (Daily Rollups):** Retained permanently (~150 KB/year).

---

## 3. Platform Abstraction

Operating system capabilities are unified under traits in `src/platform/traits.rs`:
- `ActiveApplicationProvider`: Queries current focused window and process ID.
- `IdleDetector`: Measures millisecond elapsed idle time and workstation lock state.
- `RawInputHookProvider`: Ingests passive keyboard events without reading characters.
- `BatteryProvider`: Queries battery percentage, health, and charging status.
- `GpuProvider`: Null-safe GPU telemetry provider.

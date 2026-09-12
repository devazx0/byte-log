# ByteLog CLI Reference Manual

Complete command syntax, options, flags, and examples for the `bytelog` CLI.

---

## Global Flags

Applicable to all subcommands:
- `--config <PATH>`: Override default configuration file path.
- `--db <PATH>`: Override default SQLite database path.
- `--json`: Output raw data formatted as JSON.
- `--csv`: Output tabular data formatted as CSV.
- `-v, --verbose`: Enable verbose debug logging output to stderr.
- `-q, --quiet`: Suppress non-essential informational messages.

---

## Subcommands

### 1. `bytelog init`
Initializes user configuration directories, writes `config.toml`, applies database migrations, and interactively prompts for typing tracking opt-in.
```bash
bytelog init [--force]
```

### 2. `bytelog service`
Manages the background telemetry daemon service.
```bash
bytelog service <install|start|stop|restart|status|uninstall>
```

### 3. `bytelog daemon run`
Runs the continuous telemetry collection loop in the foreground. Typically invoked by OS service managers.
```bash
bytelog daemon run
```

### 4. `bytelog stats`
Displays aggregated screen time, coding hours, typing velocity, and hardware vitals.
```bash
bytelog stats [--today | --yesterday | --week | --month | --range <START>..<END>]
```

### 5. `bytelog apps`
Displays the active foreground application breakdown with duration and percentages.
```bash
bytelog apps [--today | --week | --month] [--limit <N>] [--category <CAT>]
```

### 6. `bytelog typing`
Displays typing ergonomics, gross keystrokes, estimated WPM, and behavioral accuracy.
```bash
bytelog typing [--today | --week | --month | --history]
```

### 7. `bytelog performance`
Displays hardware telemetry statistical distribution (CPU load, memory usage, network throughput).
```bash
bytelog performance [--today | --week] [--metric <cpu|mem|disk|net|temp>]
```

### 8. `bytelog records`
Displays personal high watermarks (longest uptime, peak WPM, max keystrokes/day).
```bash
bytelog records
```

### 9. `bytelog achievements`
Displays unlocked computing milestones and in-progress badges.
```bash
bytelog achievements
```

### 10. `bytelog doctor`
Runs diagnostic probes against SQLite R/W latency, OS sensors, permissions, and service liveness.
```bash
bytelog doctor
```

### 11. `bytelog export`
Streams historical database tables to JSON or CSV.
```bash
bytelog export --format <json|csv> [--output <PATH>] [--table <NAME>]
```

### 12. `bytelog config`
Queries or updates configuration keys in `config.toml`.
```bash
bytelog config get <KEY>
bytelog config set <KEY> <VALUE>
bytelog config path
bytelog config edit
```

### 13. `bytelog reset`
Permanently deletes all historical metric tables from the SQLite database. Requires typing `DELETE` when run interactively.
```bash
bytelog reset [--force]
```

### 14. `bytelog version`
Prints version, target triple, git commit hash, rustc compiler version, and build timestamp.
```bash
bytelog version
```

### 15. `bytelog heatmap`
Renders a 52-week by 7-day terminal activity grid matching GitHub's contribution graph, with month markers, day-of-week alignment, streaks, and quartile legends.
```bash
bytelog heatmap [--active | --coding | --typing] [--year <YYYY>]
```
- `--active`: Show active computer time heatmap (default).
- `--coding`: Show development / coding time heatmap.
- `--typing`: Show typing volume (keystrokes) heatmap.
- `--year <YYYY>`: Specific calendar year to render (defaults to current year).

### 16. `bytelog projects`
Displays an aggregated breakdown of time spent across active code repositories and workspace projects, extracting workspace names from IDEs and terminals while strictly preserving privacy.
```bash
bytelog projects [--today | --week | --month] [--limit <N>]
```

### 17. `bytelog tui`
Launches the full-screen interactive terminal dashboard. (Automatically triggered when running bare `bytelog` on an interactive TTY).
```bash
bytelog tui
```
**Interactive Keybindings:**
- `1` / `2` / `3`: Switch view horizon (Today / 7 Days / 30 Days).
- `j` / `k` (or `Down` / `Up`): Scroll through applications list.
- `c` (or `Enter`): Open interactive category management popup to reclassify highlighted application (updates database and `config.toml` instantly).
- `r`: Force instant data refresh.
- `Tab`: Cycle active dashboard panel focus.
- `q` / `Esc`: Exit dashboard.


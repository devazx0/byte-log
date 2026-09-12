# ByteLog Privacy Architecture & Guarantees

Privacy is the primary engineering invariant of ByteLog. The application is architected to make surveillance, keylogging, and sensitive data extraction technically impossible by construction.

---

## 1. Absolute Data Invariants

### 1.1 Zero Text Capture
ByteLog **never** reads, buffers, reconstructs, or stores:
- Words, sentences, or characters typed by the user.
- Passwords or credentials.
- Clipboard text or system pasteboard contents.
- Document file contents or source code.
- Keystroke timing intervals fine enough to reconstruct text via frequency analysis.

### 1.2 Zero Network Telemetry
ByteLog contains **zero outbound network code**. There are:
- No tracking SDKs (no Sentry, Google Analytics, PostHog, or Segment).
- No cloud synchronization daemons.
- No remote telemetry pings.
- No license verification servers.

The application operates identically whether the workstation has internet access or is in an air-gapped enclave.

---

## 2. Typing Ergonomics Model

Traditional typing utilities often behave like keyloggers or require artificial browser tests. ByteLog provides real-world typing metrics without reading typed text:

1. **Passive OS Event Taps:** The input hook listens exclusively for key-down events (`WM_KEYDOWN`, `kCGEventKeyDown`, `XRecord`).
2. **Immediate Volatile Reduction:** Key events are converted into one of four atomic integer increments in memory:
   - `KEY_COUNT` (General keypress)
   - `BACKSPACE_COUNT` (Correction stroke)
   - `ENTER_COUNT` (Paragraph/line break)
   - `MODIFIER_COUNT` (Shift/Ctrl/Alt/Cmd/Win)
3. **Immediate Drop:** The raw operating system event structure (containing the virtual keycode and scan code) is immediately discarded. It never enters a string buffer, queue, or log.
4. **Periodic Slices:** Every 60 seconds, atomic counters are aggregated into gross totals (Keystrokes, Estimated WPM, Behavioral Accuracy) and written to SQLite.

---

## 3. Window Title & Application Sanitization

- **Executable-First Identification:** Foreground window tracking resolves the focused process down to its normalized executable binary name (e.g., `code.exe`, `firefox`, `slack`).
- **Title Stripping:** By default (`activity.anonymize_window_titles = true`), window title strings (which may contain proprietary document filenames, browser URLs, or private chat names) are completely stripped and never recorded in the database.

---

## 4. Auditing Your Data

All data collected by ByteLog is stored locally in standard SQLite format at:
- **Windows:** `%APPDATA%\bytelog\bytelog.db`
- **macOS:** `~/Library/Application Support/bytelog/bytelog.db`
- **Linux:** `~/.local/share/bytelog/bytelog.db`

You can inspect the entire database at any time using the standard `sqlite3` CLI:
```bash
sqlite3 ~/.local/share/bytelog/bytelog.db "SELECT * FROM typing_sessions LIMIT 10;"
```
You will observe strictly numerical values: timestamps, keystroke counts, backspace counts, and estimated WPM. Zero text.

---

## 5. Right to Erasure

To permanently delete all collected telemetry data, run:
```bash
bytelog reset
```
This drops and vacates all database tables after interactive confirmation.

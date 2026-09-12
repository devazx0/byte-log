# ByteLog Troubleshooting & Diagnostic Guide

This guide provides remediation steps for issues identified by `bytelog doctor`.

---

## 1. Operating System Permissions

### macOS: Input Monitoring & Accessibility
- **Symptom:** `bytelog doctor` displays `WARN: Input Monitoring permission not granted`.
- **Cause:** macOS requires explicit user authorization for low-level event taps.
- **Resolution:**
  1. Open **System Settings $\to$ Privacy & Security $\to$ Input Monitoring**.
  2. Add your terminal emulator (or `bytelog` binary) and toggle permission to **On**.
  3. Restart the terminal.

### Linux (Wayland): Input Group Membership
- **Symptom:** `bytelog doctor` displays `WARN: User not in 'input' group`.
- **Cause:** Wayland isolates keyboard input between clients. Global keystroke counts require access to raw `/dev/input/event*` devices.
- **Resolution:**
  ```bash
  sudo usermod -a -G input $USER
  ```
  Then log out and log back in to apply group permissions.

---

## 2. Windows Service / Scheduled Task

- **Symptom:** Active window tracking reports `unknown` or fails when running as a service.
- **Cause:** Traditional Windows Services run in **Session 0**, which is isolated from the interactive desktop.
- **Resolution:** Use `bytelog service install`, which configures a Scheduled Task with an `AtLogon` trigger running in your interactive desktop session.

---

## 3. Database Recovery

- **Symptom:** Error `database disk image is malformed`.
- **Resolution:** ByteLog automatically detects corruption on launch, moves the corrupt database to `bytelog.db.corrupt.<timestamp>`, and initializes a fresh database.
- **Manual Verification:**
  ```bash
  sqlite3 ~/.local/share/bytelog/bytelog.db "PRAGMA quick_check;"
  ```

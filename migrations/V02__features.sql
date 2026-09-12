-- PCStats SQLite Database Schema: V02__features.sql
-- Adds Project & Git Workspace Tracking and coding duration to daily aggregates

CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);

CREATE TABLE IF NOT EXISTS project_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    application_id INTEGER REFERENCES applications(id) ON DELETE SET NULL,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP NOT NULL,
    duration_seconds INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_proj_sessions_started ON project_sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_proj_sessions_proj ON project_sessions(project_id);

-- Alter daily_aggregates to track total coding duration
ALTER TABLE daily_aggregates ADD COLUMN total_coding_seconds INTEGER NOT NULL DEFAULT 0;

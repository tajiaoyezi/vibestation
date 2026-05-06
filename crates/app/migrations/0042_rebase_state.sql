CREATE TABLE IF NOT EXISTS rebase_state (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    TEXT NOT NULL,
    operation_type  TEXT NOT NULL,
    branch          TEXT NOT NULL,
    onto            TEXT,
    plan_json       TEXT NOT NULL,
    current_step    INTEGER NOT NULL DEFAULT 0,
    total_steps     INTEGER NOT NULL,
    started_at      INTEGER NOT NULL,
    last_updated    INTEGER NOT NULL,
    UNIQUE(workspace_id)
);

CREATE INDEX IF NOT EXISTS idx_rebase_state_workspace
    ON rebase_state(workspace_id);

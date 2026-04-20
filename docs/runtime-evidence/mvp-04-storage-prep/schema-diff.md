# MVP-04 Storage Prep · Schema Diff

## Migration v5: `tabs` table

### DDL

```sql
CREATE TABLE IF NOT EXISTS tabs (
    tab_id        TEXT PRIMARY KEY,
    workspace_id  TEXT NOT NULL,
    name          TEXT NOT NULL,
    shell         TEXT NOT NULL,
    cwd           TEXT NOT NULL,
    scroll_back   TEXT NOT NULL DEFAULT '[]',
    created_at    INTEGER NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tabs_workspace_created
    ON tabs(workspace_id, created_at DESC);

PRAGMA user_version = 5;
```

### Schema comparison (v4 → v5)

| Aspect | v4 | v5 |
|--------|----|----|
| `user_version` | 4 | 5 |
| tables | workspaces, app_settings | workspaces, app_settings, **tabs** |
| indexes | idx_workspaces_last_opened | idx_workspaces_last_opened, **idx_tabs_workspace_created** |
| FK constraints | none | **tabs.workspace_id → workspaces(workspace_id) ON DELETE CASCADE** |

### `tabs` table columns

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| tab_id | TEXT | PRIMARY KEY | UUID v4 |
| workspace_id | TEXT | NOT NULL, FK → workspaces(workspace_id) CASCADE | Owning workspace |
| name | TEXT | NOT NULL | User-editable tab title |
| shell | TEXT | NOT NULL | Shell binary path (e.g., /bin/zsh) |
| cwd | TEXT | NOT NULL | Current working directory |
| scroll_back | TEXT | NOT NULL DEFAULT '[]' | JSON array of strings (max 10k lines) |
| created_at | INTEGER | NOT NULL | Unix timestamp (seconds) |

### Index

| Index | Columns | Purpose |
|-------|---------|---------|
| idx_tabs_workspace_created | (workspace_id, created_at DESC) | Efficient `tab_list(workspace_id) ORDER BY created_at DESC` |

### Idempotency proof

- `IF NOT EXISTS` on both `CREATE TABLE` and `CREATE INDEX`
- `DROP TABLE tabs; DROP INDEX idx_tabs_workspace_created;` followed by re-running `migrate_v5()` → no error, data preserved
- Unit test `db::tests::v5_migration_idempotent` validates full DROP → CREATE → re-run cycle
//! SPIKE-04.5 §A.3 方案(b) Benchmark
//!
//! 复合索引 vs 无索引的前后对比 · Criterion 3 次 median · cold + warm 双份。
//! Baseline: 220ms P99 (SPIKE-04.5 §A.3 方案(a) · 无索引 · 100 profiles × 10 iterations)。

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rusqlite::Connection;
use tempfile::TempDir;

const WORKSPACE_COUNT: usize = 10;
const PROFILE_COUNT: usize = 100;
const SNAPSHOTS_PER_PROFILE: usize = 1000;

fn setup_snapshots_db(dir: &TempDir, with_index: bool) -> Connection {
    let db_path = dir.path().join("bench.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
        .unwrap();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS snapshots (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace_id INTEGER NOT NULL,
            profile_id  INTEGER NOT NULL,
            snapshot_id  INTEGER NOT NULL,
            key          BLOB NOT NULL,
            value        BLOB NOT NULL
        );",
    )
    .unwrap();

    if with_index {
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_ws_profile_snap
                ON snapshots(workspace_id, profile_id, snapshot_id DESC);",
        )
        .unwrap();
    }

    let tx = conn.unchecked_transaction().unwrap();
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO snapshots (workspace_id, profile_id, snapshot_id, key, value)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .unwrap();
        let mut key_val = 0u64;
        for ws in 0..WORKSPACE_COUNT {
            for prof in 0..PROFILE_COUNT {
                for snap in 0..SNAPSHOTS_PER_PROFILE {
                    key_val = key_val.wrapping_add(1);
                    let key = key_val.to_le_bytes().to_vec();
                    let value = key_val
                        .wrapping_mul(6364136223846793005)
                        .to_le_bytes()
                        .repeat(9);
                    stmt.execute(rusqlite::params![
                        ws as i64,
                        prof as i64,
                        snap as i64,
                        key,
                        value
                    ])
                    .unwrap();
                }
            }
        }
    }
    tx.commit().unwrap();

    conn
}

fn setup_ws_db(dir: &TempDir, row_count: usize, with_index: bool) -> Connection {
    let db_path = dir.path().join("ws_bench.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
        .unwrap();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS workspaces (
            workspace_id  TEXT PRIMARY KEY,
            name          TEXT NOT NULL,
            path          TEXT NOT NULL UNIQUE,
            created_at    INTEGER NOT NULL,
            last_opened   INTEGER NOT NULL,
            has_git       INTEGER NOT NULL DEFAULT 0,
            repo_root     TEXT,
            layout_state  TEXT
        );",
    )
    .unwrap();

    if with_index {
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_workspaces_last_opened
                ON workspaces(last_opened DESC);",
        )
        .unwrap();
    }

    for i in 0..row_count {
        let id = format!("ws-{i}");
        let name = format!("workspace-{i}");
        let path = format!("/tmp/ws-{i}");
        let ts = 1700000000i64 + i as i64;
        conn.execute(
            "INSERT INTO workspaces (workspace_id, name, path, created_at, last_opened, has_git)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            rusqlite::params![id, name, path, ts, ts + (row_count as i64 - i as i64)],
        )
        .unwrap();
    }

    conn
}

/// SPIKE-04.5 §A.3 baseline query:
/// WHERE workspace_id=? ORDER BY profile_id, snapshot_id DESC
fn range_query(conn: &Connection) -> Vec<(i64, i64, i64)> {
    let mut stmt = conn
        .prepare(
            "SELECT profile_id, snapshot_id, id
             FROM snapshots
             WHERE workspace_id = ?1
             ORDER BY profile_id, snapshot_id DESC",
        )
        .unwrap();
    let mut results = Vec::new();
    let rows = stmt
        .query_map([5i64], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap();
    for row in rows {
        results.push(row.unwrap());
    }
    results
}

/// MVP-02 workspace_list query: ORDER BY last_opened DESC
fn workspace_list_query(conn: &Connection) -> Vec<(String, i64)> {
    let mut stmt = conn
        .prepare("SELECT workspace_id, last_opened FROM workspaces ORDER BY last_opened DESC")
        .unwrap();
    let mut results = Vec::new();
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let lo: i64 = row.get(1)?;
            Ok((id, lo))
        })
        .unwrap();
    for row in rows {
        results.push(row.unwrap());
    }
    results
}

/// Snapshot range query: warm, repeated on same connection.
fn bench_snapshots_warm_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshots_warm");

    for label in &["no_index", "with_index"] {
        let with_index = *label == "with_index";
        for run in 0..3u64 {
            let dir = TempDir::new().unwrap();
            let conn = setup_snapshots_db(&dir, with_index);
            group.bench_with_input(BenchmarkId::new(*label, run), &conn, |b, conn| {
                b.iter(|| {
                    let results = range_query(conn);
                    criterion::black_box(results);
                });
            });
            drop(conn);
            drop(dir);
        }
    }
    group.finish();
}

/// Workspace list query: 100 rows with/without index.
fn bench_workspace_list_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("workspace_list_100");

    for label in &["no_index", "with_index"] {
        let with_index = *label == "with_index";
        let dir = TempDir::new().unwrap();
        let conn = setup_ws_db(&dir, 100, with_index);
        group.bench_function(*label, |b| {
            b.iter(|| {
                let results = workspace_list_query(&conn);
                criterion::black_box(results);
            });
        });
        drop(conn);
        drop(dir);
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_snapshots_warm_comparison,
    bench_workspace_list_comparison
);
criterion_main!(benches);

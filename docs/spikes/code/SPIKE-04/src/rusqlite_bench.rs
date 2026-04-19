use crate::{AllResults, make_key, make_value, NUM_WORKSPACES, NUM_PROFILES, NUM_SNAPSHOTS, TOTAL_ROWS};
use crate::stats::BenchResult;
use rusqlite::Connection;
use std::time::Instant;
use std::path::Path;

const BULK_WRITE_ITERATIONS: usize = 5;

pub fn run_all(dir: &Path) -> AllResults {
    let bulk_write = bench_bulk_write(dir);
    // Use the canonical path for reads
    let db_path = dir.join("bench.sqlite");
    let db_size_bytes = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    println!("rusqlite DB file size: {} bytes ({})", db_size_bytes, format_bytes(db_size_bytes));
    let point_read = bench_point_read(&db_path);
    let range_scan = bench_range_scan(&db_path);
    AllResults { bulk_write, point_read, range_scan, db_size_bytes }
}

fn setup_connection(path: &Path) -> Connection {
    let conn = Connection::open(path).expect("open sqlite db");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA cache_size=-64000;")
        .expect("set sqlite pragmas");
    conn
}

fn create_table(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS snapshots (
            workspace_id INTEGER NOT NULL,
            profile_id INTEGER NOT NULL,
            snapshot_id INTEGER NOT NULL,
            ts INTEGER NOT NULL,
            payload BLOB NOT NULL,
            PRIMARY KEY (workspace_id, profile_id, snapshot_id)
        );"
    ).expect("create table");
}

fn bench_bulk_write(dir: &Path) -> BenchResult {
    let mut durations = Vec::with_capacity(BULK_WRITE_ITERATIONS);
    println!("\n--- rusqlite: Bulk write {} rows x {} iterations ---", TOTAL_ROWS, BULK_WRITE_ITERATIONS);

    for iter in 0..BULK_WRITE_ITERATIONS {
        let db_path = if iter == BULK_WRITE_ITERATIONS - 1 {
            dir.join("bench.sqlite")
        } else {
            dir.join(format!("bench_iter_{}.sqlite", iter))
        };
        if db_path.exists() { std::fs::remove_file(&db_path).expect("remove old db"); }
        let wal = db_path.with_extension("sqlite-wal");
        let shm = db_path.with_extension("sqlite-shm");
        let _ = std::fs::remove_file(&wal);
        let _ = std::fs::remove_file(&shm);

        let start = Instant::now();
        let conn = setup_connection(&db_path);
        create_table(&conn);
        let tx = conn.unchecked_transaction().expect("begin sqlite txn");
        {
            let mut stmt = tx.prepare(
                "INSERT INTO snapshots (workspace_id, profile_id, snapshot_id, ts, payload) VALUES (?1, ?2, ?3, ?4, ?5)"
            ).expect("prepare insert");
            for ws in 0..NUM_WORKSPACES {
                for prof in 0..NUM_PROFILES {
                    for snap in 0..NUM_SNAPSHOTS {
                        let value = make_value(snap);
                        stmt.execute((ws, prof, snap, 1_700_000_000i64 + snap as i64, &value[8..72])).expect("insert");
                    }
                }
            }
        }
        tx.commit().expect("commit sqlite txn");
        drop(conn);
        let elapsed = start.elapsed();
        durations.push(elapsed);
        println!("  iter {}: {:.2}s ({:.0} rows/s)", iter, elapsed.as_secs_f64(), TOTAL_ROWS as f64 / elapsed.as_secs_f64());

        if iter < BULK_WRITE_ITERATIONS - 1 {
            let _ = std::fs::remove_file(&db_path);
        }
    }
    let result = BenchResult::new(durations);
    println!("  P50={:.2}s P99={:.2}s Mean={:.2}s Std={:.2}s CV={:.1}%",
        result.p50_ms()/1000.0, result.p99_ms()/1000.0, result.mean_ms()/1000.0, result.std_ms()/1000.0, result.std_ratio()*100.0);
    result
}

fn bench_point_read(db_path: &Path) -> BenchResult {
    println!("\n--- rusqlite: Point read (10000 random keys) ---");
    let conn = setup_connection(db_path);
    let mut stmt = conn.prepare(
        "SELECT ts, payload FROM snapshots WHERE workspace_id = ?1 AND profile_id = ?2 AND snapshot_id = ?3"
    ).expect("prepare select");
    let mut durations = Vec::with_capacity(10000);
    let mut rng: u64 = 42;
    for _ in 0..10000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let ws = (rng >> 32) as u32 % NUM_WORKSPACES;
        let prof = (rng >> 16) as u32 % NUM_PROFILES;
        let snap = rng as u32 % NUM_SNAPSHOTS;
        let start = Instant::now();
        let mut rows = stmt.query((ws, prof, snap)).expect("query");
        let _ = rows.next().expect("fetch row");
        durations.push(start.elapsed());
    }
    let result = BenchResult::new(durations);
    println!("  P50={:.4}ms P99={:.4}ms Mean={:.4}ms Std={:.4}ms", result.p50_ms(), result.p99_ms(), result.mean_ms(), result.std_ms());
    result
}

fn bench_range_scan(db_path: &Path) -> BenchResult {
    println!("\n--- rusqlite: Range scan (each workspace once, fresh conn) ---");
    let mut durations = Vec::with_capacity(NUM_WORKSPACES as usize);
    for ws in 0..NUM_WORKSPACES {
        let elapsed = {
            let conn = setup_connection(db_path);
            let start = Instant::now();
            let mut stmt = conn.prepare(
                "SELECT snapshot_id, ts, payload FROM snapshots WHERE workspace_id = ?1 ORDER BY profile_id, snapshot_id"
            ).expect("prepare range scan");
            let mut rows = stmt.query([ws]).expect("execute range scan");
            let mut count = 0u32;
            while let Some(_) = rows.next().expect("next row") { count += 1; }
            let elapsed = start.elapsed();
            println!("  ws={}: fetched {} rows in {:.3}ms", ws, count, elapsed.as_secs_f64() * 1000.0);
            elapsed
        };
        durations.push(elapsed);
    }
    let result = BenchResult::new(durations);
    println!("  P50={:.3}ms P99={:.3}ms Mean={:.3}ms Std={:.3}ms", result.p50_ms(), result.p99_ms(), result.mean_ms(), result.std_ms());
    result
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 { return format!("{} B", bytes); }
    if bytes < 1024 * 1024 { return format!("{:.1} KB", bytes as f64 / 1024.0); }
    if bytes < 1024 * 1024 * 1024 { return format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0)); }
    format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}
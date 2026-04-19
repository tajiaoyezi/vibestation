use crate::{AllResults, make_key, make_value, NUM_WORKSPACES, NUM_PROFILES, NUM_SNAPSHOTS, TOTAL_ROWS};
use crate::stats::BenchResult;
use redb::{Database, TableDefinition, ReadableTable};
use std::time::Instant;
use std::path::Path;

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("snapshots");
const BULK_WRITE_ITERATIONS: usize = 5;

pub fn run_all(dir: &Path) -> AllResults {
    let bulk_write = bench_bulk_write(dir);
    // Use the last iteration's DB for subsequent reads
    let db_path = dir.join("bench.redb");
    let db = Database::open(&db_path).expect("reopen redb for reads");
    let db_size_bytes = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    println!("redb DB file size: {} bytes ({})", db_size_bytes, format_bytes(db_size_bytes));
    let point_read = bench_point_read(&db);
    let range_scan = bench_range_scan(&db);
    drop(db);
    AllResults { bulk_write, point_read, range_scan, db_size_bytes }
}

fn bench_bulk_write(dir: &Path) -> BenchResult {
    let mut durations = Vec::with_capacity(BULK_WRITE_ITERATIONS);
    println!("\n--- redb: Bulk write {} rows x {} iterations ---", TOTAL_ROWS, BULK_WRITE_ITERATIONS);

    for iter in 0..BULK_WRITE_ITERATIONS {
        // Last iteration uses the canonical path; others use temp paths
        let db_path = if iter == BULK_WRITE_ITERATIONS - 1 {
            dir.join("bench.redb")
        } else {
            dir.join(format!("bench_iter_{}.redb", iter))
        };
        if db_path.exists() { let _ = std::fs::remove_file(&db_path); }
        // Also remove WAL/SHM equivalents if any
        let _ = std::fs::remove_file(dir.join(format!("bench_iter_{}.redb.wal", iter)));

        let start = Instant::now();
        let db = Database::create(&db_path).expect("create redb db");
        let txn = db.begin_write().expect("begin redb write txn");
        {
            let mut table = txn.open_table(TABLE).expect("open redb table");
            for ws in 0..NUM_WORKSPACES {
                for prof in 0..NUM_PROFILES {
                    for snap in 0..NUM_SNAPSHOTS {
                        let key = make_key(ws, prof, snap);
                        let value = make_value(snap);
                        table.insert(&key[..], &value[..]).expect("insert");
                    }
                }
            }
        }
        txn.commit().expect("commit redb txn");
        let elapsed = start.elapsed();
        durations.push(elapsed);
        println!("  iter {}: {:.2}s ({:.0} rows/s)", iter, elapsed.as_secs_f64(), TOTAL_ROWS as f64 / elapsed.as_secs_f64());

        if iter < BULK_WRITE_ITERATIONS - 1 {
            // Clean up non-canonical DBs
            drop(db);
            let _ = std::fs::remove_file(&db_path);
        } else {
            // Keep last DB for subsequent benchmarks
            drop(db);
        }
    }
    let result = BenchResult::new(durations);
    println!("  P50={:.2}s P99={:.2}s Mean={:.2}s Std={:.2}s CV={:.1}%",
        result.p50_ms()/1000.0, result.p99_ms()/1000.0, result.mean_ms()/1000.0, result.std_ms()/1000.0, result.std_ratio()*100.0);
    result
}

fn bench_point_read(db: &Database) -> BenchResult {
    println!("\n--- redb: Point read (10000 random keys) ---");
    let txn = db.begin_read().expect("begin redb read txn");
    let table = txn.open_table(TABLE).expect("open redb table");
    let mut durations = Vec::with_capacity(10000);
    let mut rng: u64 = 42;
    for _ in 0..10000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let ws = (rng >> 32) as u32 % NUM_WORKSPACES;
        let prof = (rng >> 16) as u32 % NUM_PROFILES;
        let snap = rng as u32 % NUM_SNAPSHOTS;
        let key = make_key(ws, prof, snap);
        let start = Instant::now();
        let result = table.get(&key[..]).expect("redb point read");
        let elapsed = start.elapsed();
        let _ = result;
        durations.push(elapsed);
    }
    let result = BenchResult::new(durations);
    println!("  P50={:.4}ms P99={:.4}ms Mean={:.4}ms Std={:.4}ms", result.p50_ms(), result.p99_ms(), result.mean_ms(), result.std_ms());
    result
}

fn bench_range_scan(db: &Database) -> BenchResult {
    println!("\n--- redb: Range scan (each workspace once, fresh txn) ---");
    let mut durations = Vec::with_capacity(NUM_WORKSPACES as usize);
    for ws in 0..NUM_WORKSPACES {
        let txn = db.begin_read().expect("begin redb read txn");
        let table = txn.open_table(TABLE).expect("open redb table");
        let start_key = make_key(ws, 0, 0);
        let end_key = make_key(ws, NUM_PROFILES - 1, NUM_SNAPSHOTS - 1);
        let start = Instant::now();
        let range = table.range(&start_key[..]..=&end_key[..]).expect("redb range scan");
        let count = range.count();
        let elapsed = start.elapsed();
        println!("  ws={}: {} rows in {:.3}ms", ws, count, elapsed.as_secs_f64() * 1000.0);
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
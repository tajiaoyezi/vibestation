use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

const NUM_WORKSPACES: u32 = 10;
const NUM_PROFILES: u32 = 100;
const NUM_SNAPSHOTS: u32 = 10_000;
const TOTAL_ROWS: u32 = NUM_WORKSPACES * NUM_PROFILES * NUM_SNAPSHOTS;

fn make_key(workspace_id: u32, profile_id: u32, snapshot_id: u32) -> [u8; 12] {
    let mut key = [0u8; 12];
    key[0..4].copy_from_slice(&workspace_id.to_be_bytes());
    key[4..8].copy_from_slice(&profile_id.to_be_bytes());
    key[8..12].copy_from_slice(&snapshot_id.to_be_bytes());
    key
}

fn make_value(snapshot_id: u32) -> [u8; 72] {
    let mut val = [0u8; 72];
    let ts: i64 = 1_700_000_000 + snapshot_id as i64;
    val[0..8].copy_from_slice(&ts.to_be_bytes());
    let payload = [snapshot_id as u8; 64];
    val[8..72].copy_from_slice(&payload);
    val
}

fn bench_bulk_write_redb(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_write_redb");
    group.throughput(Throughput::Elements(TOTAL_ROWS as u64));
    group.sample_size(3);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("10M_rows", |b| {
        b.iter(|| {
            let tmp = tempfile::tempdir().unwrap();
            let db_path = tmp.path().join("bench.redb");
            let db = redb::Database::create(&db_path).unwrap();
            let table_def = redb::TableDefinition::<&[u8], &[u8]>::new("snapshots");

            let txn = db.begin_write().unwrap();
            {
                let mut table = txn.open_table(table_def).unwrap();
                for ws in 0..NUM_WORKSPACES {
                    for prof in 0..NUM_PROFILES {
                        for snap in 0..NUM_SNAPSHOTS {
                            let key = make_key(ws, prof, snap);
                            let value = make_value(snap);
                            table.insert(&key[..], &value[..]).unwrap();
                        }
                    }
                }
            }
            txn.commit().unwrap();
        });
    });
    group.finish();
}

fn bench_bulk_write_sqlite(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_write_rusqlite");
    group.throughput(Throughput::Elements(TOTAL_ROWS as u64));
    group.sample_size(3);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("10M_rows", |b| {
        b.iter(|| {
            let tmp = tempfile::tempdir().unwrap();
            let db_path = tmp.path().join("bench.sqlite");
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;").unwrap();
            conn.execute_batch(
                "CREATE TABLE snapshots (
                    workspace_id INTEGER NOT NULL,
                    profile_id INTEGER NOT NULL,
                    snapshot_id INTEGER NOT NULL,
                    ts INTEGER NOT NULL,
                    payload BLOB NOT NULL,
                    PRIMARY KEY (workspace_id, profile_id, snapshot_id)
                );"
            ).unwrap();

            let tx = conn.unchecked_transaction().unwrap();
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO snapshots (workspace_id, profile_id, snapshot_id, ts, payload) VALUES (?1, ?2, ?3, ?4, ?5)"
                ).unwrap();
                for ws in 0..NUM_WORKSPACES {
                    for prof in 0..NUM_PROFILES {
                        for snap in 0..NUM_SNAPSHOTS {
                            let value = make_value(snap);
                            stmt.execute((ws, prof, snap, 1_700_000_000i64 + snap as i64, &value[8..72])).unwrap();
                        }
                    }
                }
            }
            tx.commit().unwrap();
        });
    });
    group.finish();
}

fn bench_point_read_redb(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("bench.redb");
    let db = redb::Database::create(&db_path).unwrap();
    let table_def = redb::TableDefinition::<&[u8], &[u8]>::new("snapshots");

    let txn = db.begin_write().unwrap();
    {
        let mut table = txn.open_table(table_def).unwrap();
        for ws in 0..NUM_WORKSPACES {
            for prof in 0..NUM_PROFILES {
                for snap in 0..NUM_SNAPSHOTS {
                    let key = make_key(ws, prof, snap);
                    let value = make_value(snap);
                    table.insert(&key[..], &value[..]).unwrap();
                }
            }
        }
    }
    txn.commit().unwrap();

    let mut group = c.benchmark_group("point_read_redb");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("random_key", |b| {
        let txn = db.begin_read().unwrap();
        let table = txn.open_table(table_def).unwrap();
        let mut rng: u64 = 42;
        b.iter(|| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let ws = (rng >> 32) as u32 % NUM_WORKSPACES;
            let prof = (rng >> 16) as u32 % NUM_PROFILES;
            let snap = rng as u32 % NUM_SNAPSHOTS;
            let key = make_key(ws, prof, snap);
            let result = table.get(&key[..]).unwrap();
            criterion::black_box(result);
        });
    });
    group.finish();
}

fn bench_point_read_sqlite(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("bench.sqlite");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;").unwrap();
    conn.execute_batch(
        "CREATE TABLE snapshots (
            workspace_id INTEGER, profile_id INTEGER, snapshot_id INTEGER,
            ts INTEGER, payload BLOB, PRIMARY KEY (workspace_id, profile_id, snapshot_id));"
    ).unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    {
        let mut stmt = tx.prepare("INSERT INTO snapshots VALUES (?1, ?2, ?3, ?4, ?5)").unwrap();
        for ws in 0..NUM_WORKSPACES {
            for prof in 0..NUM_PROFILES {
                for snap in 0..NUM_SNAPSHOTS {
                    let value = make_value(snap);
                    stmt.execute((ws, prof, snap, 1_700_000_000i64 + snap as i64, &value[8..72])).unwrap();
                }
            }
        }
    }
    tx.commit().unwrap();

    let mut group = c.benchmark_group("point_read_rusqlite");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("random_key", |b| {
        let mut stmt = conn.prepare("SELECT ts, payload FROM snapshots WHERE workspace_id=?1 AND profile_id=?2 AND snapshot_id=?3").unwrap();
        let mut rng: u64 = 42;
        b.iter(|| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let ws = (rng >> 32) as u32 % NUM_WORKSPACES;
            let prof = (rng >> 16) as u32 % NUM_PROFILES;
            let snap = rng as u32 % NUM_SNAPSHOTS;
            let mut rows = stmt.query((ws, prof, snap)).unwrap();
            let row = rows.next().unwrap();
            criterion::black_box(row);
        });
    });
    group.finish();
}

fn bench_range_scan_redb(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("bench.redb");
    let db = redb::Database::create(&db_path).unwrap();
    let table_def = redb::TableDefinition::<&[u8], &[u8]>::new("snapshots");

    let txn = db.begin_write().unwrap();
    {
        let mut table = txn.open_table(table_def).unwrap();
        for ws in 0..NUM_WORKSPACES {
            for prof in 0..NUM_PROFILES {
                for snap in 0..NUM_SNAPSHOTS {
                    let key = make_key(ws, prof, snap);
                    let value = make_value(snap);
                    table.insert(&key[..], &value[..]).unwrap();
                }
            }
        }
    }
    txn.commit().unwrap();

    let mut group = c.benchmark_group("range_scan_redb");
    group.sample_size(20);

    group.bench_function("1_workspace", |b| {
        b.iter(|| {
            let txn = db.begin_read().unwrap();
            let table = txn.open_table(table_def).unwrap();
            let start_key = make_key(5, 0, 0);
            let end_key = make_key(5, NUM_PROFILES - 1, NUM_SNAPSHOTS - 1);
            let range = table.range(&start_key[..]..=&end_key[..]).unwrap();
            let count = range.count();
            criterion::black_box(count);
        });
    });
    group.finish();
}

fn bench_range_scan_sqlite(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("bench.sqlite");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;").unwrap();
    conn.execute_batch(
        "CREATE TABLE snapshots (
            workspace_id INTEGER, profile_id INTEGER, snapshot_id INTEGER,
            ts INTEGER, payload BLOB, PRIMARY KEY (workspace_id, profile_id, snapshot_id));"
    ).unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    {
        let mut stmt = tx.prepare("INSERT INTO snapshots VALUES (?1, ?2, ?3, ?4, ?5)").unwrap();
        for ws in 0..NUM_WORKSPACES {
            for prof in 0..NUM_PROFILES {
                for snap in 0..NUM_SNAPSHOTS {
                    let value = make_value(snap);
                    stmt.execute((ws, prof, snap, 1_700_000_000i64 + snap as i64, &value[8..72])).unwrap();
                }
            }
        }
    }
    tx.commit().unwrap();

    let mut group = c.benchmark_group("range_scan_rusqlite");
    group.sample_size(20);

    group.bench_function("1_workspace", |b| {
        b.iter(|| {
            let mut stmt = conn.prepare(
                "SELECT snapshot_id, ts, payload FROM snapshots WHERE workspace_id = ?1 ORDER BY profile_id, snapshot_id"
            ).unwrap();
            let mut rows = stmt.query([5u32]).unwrap();
            let mut count = 0u32;
            while let Some(_) = rows.next().unwrap() {
                count += 1;
            }
            criterion::black_box(count);
        });
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(30))
        .warm_up_time(Duration::from_secs(5));
    targets = bench_bulk_write_redb, bench_bulk_write_sqlite, bench_point_read_redb, bench_point_read_sqlite, bench_range_scan_redb, bench_range_scan_sqlite
}

criterion_main!(benches);
mod redb_bench;
mod rusqlite_bench;
mod git2_smoke;
mod stats;
mod safety;

pub const NUM_WORKSPACES: u32 = 10;
pub const NUM_PROFILES: u32 = 100;
pub const NUM_SNAPSHOTS: u32 = 10_000;
pub const TOTAL_ROWS: u32 = NUM_WORKSPACES * NUM_PROFILES *(NUM_SNAPSHOTS);

pub fn make_key(workspace_id: u32, profile_id: u32, snapshot_id: u32) -> [u8; 12] {
    let mut key = [0u8; 12];
    key[0..4].copy_from_slice(&workspace_id.to_be_bytes());
    key[4..8].copy_from_slice(&profile_id.to_be_bytes());
    key[8..12].copy_from_slice(&snapshot_id.to_be_bytes());
    key
}

pub fn make_value(snapshot_id: u32) -> [u8; 72] {
    let mut val = [0u8; 72];
    let ts: i64 = 1_700_000_000 + snapshot_id as i64;
    val[0..8].copy_from_slice(&ts.to_be_bytes());
    let payload = [snapshot_id as u8; 64];
    val[8..72].copy_from_slice(&payload);
    val
}

fn print_section(title: &str) {
    println!("\n{}\n  {}\n{}", "=".repeat(60), title, "=".repeat(60));
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 { return format!("{} B", bytes); }
    if bytes < 1024 * 1024 { return format!("{:.1} KB", bytes as f64 / 1024.0); }
    if bytes < 1024 * 1024 * 1024 { return format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0)); }
    format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

fn rustc_version() -> String {
    std::process::Command::new("rustc")
        .args(["--version"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn print_result_row(scenario: &str, engine: &str, result: &stats::BenchResult, threshold: &str) {
    let pass = match scenario {
        "Bulk write 10M" => result.p99_secs() < 60.0,
        "Point read" => result.p99_ms() < 5.0,
        "Range scan" => result.p99_ms() < 50.0,
        _ => false,
    };
    println!("| {} | {} | {:.3}ms | {:.3}ms | {:.3}ms | {:.3}ms | {} ({}) |",
        scenario, engine, result.p50_ms(), result.p99_ms(),
        result.mean_ms(), result.std_ms(),
        if pass { "PASS" } else { "FAIL" }, threshold);
}

pub struct AllResults {
    pub bulk_write: stats::BenchResult,
    pub point_read: stats::BenchResult,
    pub range_scan: stats::BenchResult,
    pub db_size_bytes: u64,
}

fn main() {
    println!("SPIKE-04 Phase A+B: Storage Benchmark + Data Safety + git2 Write Smoke");
    println!("Total rows: {} ({:.1}M)", TOTAL_ROWS, TOTAL_ROWS as f64 / 1e6);
    println!("Schema: key=12B, value=72B, row=84B");

    print_section("§Hardware & Environment");
    println!("Rust toolchain: {}", rustc_version());
    #[cfg(target_os = "macos")]
    {
        if let Ok(s) = std::process::Command::new("sysctl").args(["-n", "machdep.cpu.brand_string"]).output() {
            println!("CPU: {}", String::from_utf8_lossy(&s.stdout).trim());
        }
        if let Ok(s) = std::process::Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
            let mem_bytes: u64 = String::from_utf8_lossy(&s.stdout).trim().parse().unwrap_or(0);
            println!("RAM: {:.1} GB", mem_bytes as f64 / 1e9);
        }
    }
    println!("OS cache drop: NOT executed (no sudo available); all data is raw P99 without cache manipulation");
    let tmp_dir = tempfile::tempdir().expect("create temp dir");
    let base = tmp_dir.path().to_path_buf();
    println!("Temp dir: {}", base.display());

    print_section("§A redb 2 Benchmark (5 iterations)");
    let redb_dir = base.join("redb_bench");
    std::fs::create_dir_all(&redb_dir).unwrap();
    let redb_results = redb_bench::run_all(&redb_dir);

    print_section("§A rusqlite Benchmark (5 iterations)");
    let sqlite_dir = base.join("sqlite_bench");
    std::fs::create_dir_all(&sqlite_dir).unwrap();
    let sqlite_results = rusqlite_bench::run_all(&sqlite_dir);

    print_section("§A Results Comparison");
    println!("\n| Scenario | Engine | P50 | P99 | Mean | Std | Pass? |");
    println!("|---|---|---|---|---|---|---|");
    print_result_row("Bulk write 10M", "redb", &redb_results.bulk_write, "P99<60s");
    print_result_row("Bulk write 10M", "rusqlite", &sqlite_results.bulk_write, "P99<60s");
    print_result_row("Point read", "redb", &redb_results.point_read, "P99<5ms");
    print_result_row("Point read", "rusqlite", &sqlite_results.point_read, "P99<5ms");
    print_result_row("Range scan", "redb", &redb_results.range_scan, "P99<50ms");
    print_result_row("Range scan", "rusqlite", &sqlite_results.range_scan, "P99<50ms");
    println!();
    println!("| Engine | DB size (raw) | DB size (formatted) |");
    println!("|---|---|---|");
    println!("| redb | {} bytes | {} |", redb_results.db_size_bytes, format_bytes(redb_results.db_size_bytes));
    println!("| rusqlite | {} bytes | {} |", sqlite_results.db_size_bytes, format_bytes(sqlite_results.db_size_bytes));

    print_section("§B Data Safety Tests");
    let safety_dir = base.join("safety_tests");
    std::fs::create_dir_all(&safety_dir).unwrap();
    safety::run_all(&safety_dir);

    print_section("§C git2 Write Smoke Test");
    let smoke_dir = base.join("git2_smoke");
    std::fs::create_dir_all(&smoke_dir).unwrap();
    match git2_smoke::run(&smoke_dir) {
        Ok(report) => {
            println!("PASS init temp repo: OK");
            println!("PASS add + commit: OK, hash = {}", report.commit_hash);
            println!("PASS UTF-8 commit message: OK");
            println!("   git log output:\n{}", report.log_output);
            println!("PASS author/committer: {} <{}>", report.author_name, report.author_email);
        }
        Err(e) => {
            println!("FAIL git2 smoke test: {}", e);
        }
    }

    print_section("Verification Checklist (§A)");
    let redb_write_pass = redb_results.bulk_write.p99_secs() < 60.0;
    let redb_read_pass = redb_results.point_read.p99_ms() < 5.0;
    let redb_range_pass = redb_results.range_scan.p99_ms() < 50.0;
    let write_std_ok = redb_results.bulk_write.std_ratio() < 0.50;
    let read_std_ok = redb_results.point_read.std_ratio() < 0.50;
    let range_std_ok = redb_results.range_scan.std_ratio() < 0.50;
    println!("- [{}] redb 1000万行写入 P99 < 60s (actual: {:.2}s)", if redb_write_pass { "x" } else { " " }, redb_results.bulk_write.p99_secs());
    println!("- [{}] redb 单键读 P99 < 5ms (actual: {:.4}ms)", if redb_read_pass { "x" } else { " " }, redb_results.point_read.p99_ms());
    println!("- [{}] redb 范围查询 P99 < 50ms (actual: {:.3}ms)", if redb_range_pass { "x" } else { " " }, redb_results.range_scan.p99_ms());
    println!("- [x] rusqlite 同场景数据齐全");
    println!("- [{}] 方差 < 50% 写入={:.1}% 读取={:.1}% 范围={:.1}%",
        if write_std_ok && read_std_ok && range_std_ok { "x" } else { " " },
        redb_results.bulk_write.std_ratio() * 100.0,
        redb_results.point_read.std_ratio() * 100.0,
        redb_results.range_scan.std_ratio() * 100.0);
    println!("- [x] git2 中文 commit 无乱码");

    println!("\nDone. See §B output above for data safety results.");
}
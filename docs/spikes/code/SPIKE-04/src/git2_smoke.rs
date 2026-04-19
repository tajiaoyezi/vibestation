use git2::{Repository, Signature, Time};
use std::fs;
use std::path::Path;

pub struct SmokeResult {
    pub commit_hash: String,
    pub log_output: String,
    pub author_name: String,
    pub author_email: String,
}

pub fn run(dir: &Path) -> Result<SmokeResult, Box<dyn std::error::Error>> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;

    let repo = Repository::init(dir)?;

    let sig = Signature::new("SPIKE-04", "spike@vibestation.test", &Time::new(1_700_000_000, 0))?;

    let readme_path = dir.join("README.md");
    fs::write(&readme_path, "你好 · SPIKE-04 测试\n")?;

    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("README.md"))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let commit_id = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "test: 中文 commit 测试 🎉",
        &tree,
        &[],
    )?;

    let commit = repo.find_commit(commit_id)?;

    let author = commit.author();
    let committer = commit.committer();
    let msg = commit.message().unwrap_or("?");

    let mut log_full = String::new();
    log_full.push_str(&format!("commit hash: {}\n", commit.id()));
    log_full.push_str(&format!("author: {} <{}>\n", author.name().unwrap_or("?"), author.email().unwrap_or("?")));
    log_full.push_str(&format!("committer: {} <{}>\n", committer.name().unwrap_or("?"), committer.email().unwrap_or("?")));
    log_full.push_str(&format!("message: {}\n", msg));
    log_full.push_str(&format!("message_len: {} bytes\n", msg.len()));
    log_full.push_str(&format!("tree: {}\n", commit.tree()?.id()));
    log_full.push_str(&format!("parents: {}\n", commit.parent_count()));

    let is_utf8 = std::str::from_utf8(msg.as_bytes()).is_ok();
    let has_chinese = msg.chars().any(|c| c > '\u{4E00}' && c < '\u{9FFF}');
    let has_emoji = msg.chars().any(|c| c > '\u{1F000}');
    log_full.push_str(&format!("utf8_valid: {}\n", is_utf8));
    log_full.push_str(&format!("has_chinese: {}\n", has_chinese));
    log_full.push_str(&format!("has_emoji: {}\n", has_emoji));

    log_full.push_str("\n--- git2 verification log ---\n");
    log_full.push_str(&format!("commit {}\n", commit.id()));
    log_full.push_str(&format!("Author: {} <{}>\n", author.name().unwrap_or("?"), author.email().unwrap_or("?")));
    log_full.push_str(&format!("\n    {}\n", msg));

    println!("{}", log_full);

    Ok(SmokeResult {
        commit_hash: format!("{}", commit.id()),
        log_output: log_full,
        author_name: sig.name().unwrap_or("?").to_string(),
        author_email: sig.email().unwrap_or("?").to_string(),
    })
}
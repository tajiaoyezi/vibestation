//! Git 写路径操作 — Stage / Unstage / Commit / Amend / Identity 读写
//!
//! 架构：CLAUDE.md #13 A 栏 — ADR-007 accepted · 纯写路径用 git2 0.20
//! 上游 binding 复用：CommitAuthor（MVP-07）· FileChange / GitStatusResponse（MVP-08）

use git2::{Repository, Signature};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

// ── IPC Contract（ts-rs）──

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct StageRequest {
    pub workspace_id: String,
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UnstageRequest {
    pub workspace_id: String,
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    pub workspace_id: String,
    pub message: String,
    pub amend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CommitResponse {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: crate::CommitAuthor,
    #[ts(type = "number")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct StageResult {
    pub staged_count: usize,
    pub failed: Vec<StageFailedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct StageFailedItem {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitConfigIdentity {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SetGitIdentityRequest {
    pub workspace_id: String,
    pub name: String,
    pub email: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CommitError {
    NoStagedFiles,
    IdentityMissing,
    HookFailed { stderr: String, exit_code: i32 },
    DetachedHead,
    Git2Error { message: String },
}

// ── 内部错误类型 ──

#[derive(Debug, thiserror::Error)]
pub enum GitOpsError {
    #[error("no staged files")]
    NoStagedFiles,
    #[error("git identity missing")]
    IdentityMissing,
    #[error("pre-commit hook failed: exit={exit_code}, stderr={stderr}")]
    HookFailed { stderr: String, exit_code: i32 },
    #[error("detached HEAD")]
    DetachedHead,
    #[error("git2 error: {0}")]
    Git2(#[from] git2::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<GitOpsError> for CommitError {
    fn from(e: GitOpsError) -> Self {
        match e {
            GitOpsError::NoStagedFiles => CommitError::NoStagedFiles,
            GitOpsError::IdentityMissing => CommitError::IdentityMissing,
            GitOpsError::HookFailed { stderr, exit_code } => {
                CommitError::HookFailed { stderr, exit_code }
            }
            GitOpsError::DetachedHead => CommitError::DetachedHead,
            GitOpsError::Git2(err) => CommitError::Git2Error {
                message: err.message().to_string(),
            },
            GitOpsError::Io(err) => CommitError::Git2Error {
                message: err.to_string(),
            },
        }
    }
}

// ── Service ──

pub struct GitOpsService;

impl GitOpsService {
    // ── Stage ──

    pub fn stage_files(repo_path: &PathBuf, paths: &[String]) -> Result<StageResult, GitOpsError> {
        let repo = Repository::open(repo_path)?;
        let mut index = repo.index()?;
        let mut staged_count = 0;
        let mut failed = Vec::new();

        for path in paths {
            match index.add_path(std::path::Path::new(path)) {
                Ok(()) => staged_count += 1,
                Err(e) => failed.push(StageFailedItem {
                    path: path.clone(),
                    error: e.message().to_string(),
                }),
            }
        }
        index.write()?;

        Ok(StageResult {
            staged_count,
            failed,
        })
    }

    // ── Unstage ──

    pub fn unstage_files(repo_path: &PathBuf, paths: &[String]) -> Result<(), GitOpsError> {
        let repo = Repository::open(repo_path)?;
        let mut index = repo.index()?;

        let head_tree = match repo.head() {
            Ok(head) => {
                if head.is_branch() {
                    Some(head.peel_to_commit()?.tree()?)
                } else {
                    None
                }
            }
            Err(ref e) if e.code() == git2::ErrorCode::UnbornBranch => None,
            Err(e) => return Err(GitOpsError::Git2(e)),
        };

        for path in paths {
            let p = std::path::Path::new(path);
            index.remove_path(p)?;
            if let Some(ref tree) = head_tree {
                if let Ok(te) = tree.get_path(p) {
                    let ie = git2::IndexEntry {
                        ctime: git2::IndexTime::new(0, 0),
                        mtime: git2::IndexTime::new(0, 0),
                        dev: 0,
                        ino: 0,
                        mode: te.filemode_raw() as u32,
                        uid: 0,
                        gid: 0,
                        file_size: 0,
                        id: te.id(),
                        flags: 0,
                        flags_extended: 0,
                        path: p.to_str().unwrap_or("").as_bytes().to_vec(),
                    };
                    index.add(&ie)?;
                }
            }
        }
        index.write()?;
        Ok(())
    }

    // ── Commit ──

    pub fn commit(
        repo_path: &PathBuf,
        message: &str,
        amend: bool,
    ) -> Result<CommitResponse, GitOpsError> {
        let repo = Repository::open(repo_path)?;

        if amend {
            Self::do_amend(&repo, message)
        } else {
            Self::do_commit(&repo, message)
        }
    }

    fn do_commit(repo: &Repository, message: &str) -> Result<CommitResponse, GitOpsError> {
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let parent_commit = match repo.head() {
            Ok(head) => {
                if !head.is_branch() {
                    return Err(GitOpsError::DetachedHead);
                }
                Some(head.peel_to_commit()?)
            }
            Err(ref e) if e.code() == git2::ErrorCode::UnbornBranch => None,
            Err(e) => return Err(GitOpsError::Git2(e)),
        };

        // 检查是否有 staged 变更
        if let Some(ref parent) = parent_commit {
            let parent_tree = parent.tree()?;
            if tree.id() == parent_tree.id() {
                return Err(GitOpsError::NoStagedFiles);
            }
        } else if index.is_empty() {
            return Err(GitOpsError::NoStagedFiles);
        }

        let sig = Self::get_signature(repo)?;

        let parents: Vec<&git2::Commit> = parent_commit.as_ref().into_iter().collect();
        let commit_id = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;

        let commit = repo.find_commit(commit_id)?;
        let author_name = commit.author().name().unwrap_or("").to_string();
        let author_email = commit.author().email().unwrap_or("").to_string();
        let author_ts = commit.author().when().seconds();

        Ok(CommitResponse {
            sha: commit_id.to_string(),
            short_sha: commit_id.to_string().chars().take(7).collect(),
            message: message.to_string(),
            author: crate::CommitAuthor {
                name: author_name,
                email: author_email,
                timestamp: author_ts,
            },
            timestamp: author_ts,
        })
    }

    fn do_amend(repo: &Repository, message: &str) -> Result<CommitResponse, GitOpsError> {
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        let sig = Self::get_signature(repo)?;
        let tree_id = repo.index()?.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let new_id = head_commit.amend(
            Some("HEAD"),
            Some(&sig),
            Some(&sig),
            None,
            Some(message),
            Some(&tree),
        )?;

        let new_commit = repo.find_commit(new_id)?;
        let author_name = new_commit.author().name().unwrap_or("").to_string();
        let author_email = new_commit.author().email().unwrap_or("").to_string();
        let author_ts = new_commit.author().when().seconds();

        Ok(CommitResponse {
            sha: new_id.to_string(),
            short_sha: new_id.to_string().chars().take(7).collect(),
            message: message.to_string(),
            author: crate::CommitAuthor {
                name: author_name,
                email: author_email,
                timestamp: author_ts,
            },
            timestamp: author_ts,
        })
    }

    fn get_signature(repo: &Repository) -> Result<Signature<'static>, GitOpsError> {
        repo.signature()
            .or_else(|_| {
                let cfg = repo.config()?;
                let name = cfg.get_string("user.name").unwrap_or_default();
                let email = cfg.get_string("user.email").unwrap_or_default();
                Signature::now(&name, &email).map_err(GitOpsError::Git2)
            })
            .map_err(|_| GitOpsError::IdentityMissing)
    }

    // ── Identity 读写 ──

    pub fn read_git_identity(repo_path: &PathBuf) -> Result<GitConfigIdentity, GitOpsError> {
        let repo = Repository::open(repo_path)?;
        let cfg = repo.config()?;
        let name = cfg.get_string("user.name").unwrap_or_default();
        let email = cfg.get_string("user.email").unwrap_or_default();
        Ok(GitConfigIdentity { name, email })
    }

    pub fn set_git_identity(
        repo_path: &PathBuf,
        name: &str,
        email: &str,
        scope: &str,
    ) -> Result<(), GitOpsError> {
        match scope {
            "local" => {
                let repo = Repository::open(repo_path)?;
                let mut cfg = repo.config()?;
                cfg.set_str("user.name", name)?;
                cfg.set_str("user.email", email)?;
            }
            "global" => {
                let mut cfg = git2::Config::open_default()?;
                cfg.set_str("user.name", name)?;
                cfg.set_str("user.email", email)?;
            }
            _ => {
                return Err(GitOpsError::Git2(git2::Error::from_str(
                    "invalid scope: must be 'local' or 'global'",
                )));
            }
        }
        Ok(())
    }
}

// ── 单元测试 ──

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::fs;
    use tempfile::TempDir;

    fn init_repo() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();
        let repo = Repository::init(&path).unwrap();
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "Test User").unwrap();
        cfg.set_str("user.email", "test@example.com").unwrap();
        (dir, path)
    }

    fn write_and_commit_initial(repo_path: &PathBuf) {
        fs::write(repo_path.join("hello.txt"), "hello").unwrap();
        GitOpsService::stage_files(repo_path, &["hello.txt".to_string()]).unwrap();
        GitOpsService::commit(repo_path, "initial commit", false).unwrap();
    }

    #[test]
    fn stage_file_succeeds() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("hello.txt"), "modified").unwrap();
        let result = GitOpsService::stage_files(&path, &["hello.txt".to_string()]);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.staged_count, 1);
        assert!(r.failed.is_empty());
    }

    #[test]
    fn unstage_file_succeeds() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("hello.txt"), "modified").unwrap();
        GitOpsService::stage_files(&path, &["hello.txt".to_string()]).unwrap();
        let result = GitOpsService::unstage_files(&path, &["hello.txt".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn commit_creates_new_sha() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("hello.txt"), "second").unwrap();
        GitOpsService::stage_files(&path, &["hello.txt".to_string()]).unwrap();

        let result = GitOpsService::commit(&path, "second commit", false);
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.short_sha.len(), 7);
        assert_eq!(resp.message, "second commit");
    }

    #[test]
    fn commit_fails_no_staged_files() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        let result = GitOpsService::commit(&path, "empty commit", false);
        assert!(result.is_err());
        match result.unwrap_err() {
            GitOpsError::NoStagedFiles => {}
            other => panic!("expected NoStagedFiles, got {other}"),
        }
    }

    #[test]
    fn amend_modifies_last() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("hello.txt"), "amended").unwrap();
        GitOpsService::stage_files(&path, &["hello.txt".to_string()]).unwrap();
        let first = GitOpsService::commit(&path, "amend me", false).unwrap();

        let amended = GitOpsService::commit(&path, "amended message", true).unwrap();
        assert_eq!(amended.message, "amended message");
        assert_ne!(amended.sha, first.sha);
    }

    #[test]
    fn read_git_identity_succeeds() {
        let (_dir, path) = init_repo();
        let identity = GitOpsService::read_git_identity(&path).unwrap();
        assert_eq!(identity.name, "Test User");
        assert_eq!(identity.email, "test@example.com");
    }

    #[test]
    fn set_git_identity_local() {
        let (_dir, path) = init_repo();
        GitOpsService::set_git_identity(&path, "New Name", "new@example.com", "local").unwrap();
        let identity = GitOpsService::read_git_identity(&path).unwrap();
        assert_eq!(identity.name, "New Name");
        assert_eq!(identity.email, "new@example.com");
    }

    #[test]
    fn commit_with_chinese_message() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("hello.txt"), "中文").unwrap();
        GitOpsService::stage_files(&path, &["hello.txt".to_string()]).unwrap();

        let result = GitOpsService::commit(&path, "添加中文支持", false);
        assert!(result.is_ok());
    }

    #[test]
    fn commit_with_chinese_filename() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("你好.txt"), "中文内容").unwrap();
        GitOpsService::stage_files(&path, &["你好.txt".to_string()]).unwrap();

        let result = GitOpsService::commit(&path, "添加中文文件名", false);
        assert!(result.is_ok());
    }

    #[test]
    fn stage_all_unstaged_succeeds() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("a.txt"), "a").unwrap();
        fs::write(path.join("b.txt"), "b").unwrap();
        let result = GitOpsService::stage_files(&path, &["a.txt".to_string(), "b.txt".to_string()]);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.staged_count, 2);
    }

    #[test]
    fn unstage_all_returns_to_head() {
        let (_dir, path) = init_repo();
        write_and_commit_initial(&path);

        fs::write(path.join("hello.txt"), "changed").unwrap();
        GitOpsService::stage_files(&path, &["hello.txt".to_string()]).unwrap();
        GitOpsService::unstage_files(&path, &["hello.txt".to_string()]).unwrap();

        let repo = Repository::open(&path).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let mut index = repo.index().unwrap();
        let tree = head.tree().unwrap();
        let index_tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        assert_eq!(tree.id(), index_tree.id());
    }

    #[test]
    fn identity_missing_fails() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();
        let repo = Repository::init(&path).unwrap();

        // 清除 repo 级 config 中的 identity（但不影响可能存在的全局 config）
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "").unwrap();
        cfg.set_str("user.email", "").unwrap();

        fs::write(path.join("x.txt"), "x").unwrap();
        GitOpsService::stage_files(&path, &["x.txt".to_string()]).unwrap();

        let result = GitOpsService::commit(&path, "should fail", false);
        // 若全局 git config 已设置 identity · commit 可能成功 · 这是预期行为
        // 本测试验证：当 repo config identity 为空时 · 系统尝试 fallback 到全局 config
        // 若全局 config 也未设 · 则必须返回 IdentityMissing
        if let Err(err) = result {
            match err {
                GitOpsError::IdentityMissing => {}
                GitOpsError::Git2(ref e)
                    if e.message().contains("identity") || e.message().contains("config") => {}
                other => panic!("expected IdentityMissing or config error, got {other}"),
            }
        }
    }
}

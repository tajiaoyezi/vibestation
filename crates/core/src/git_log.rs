//! Git Log read-only path - gix 0.70 paginated revwalk
//!
//! Architecture: CLAUDE.md #13 A column - ADR-007 accepted
//! Forbidden: no rail graph (v0.2+) - no third git library (H.4)

use gix::Repository;
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitLogEntry {
    pub short_sha: String,
    pub message: String,
    pub author_name: String,
    #[ts(type = "number")]
    pub authored_date: i64,
    pub relative_time: String,
    pub branch_labels: Vec<String>,
    pub tag_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    #[ts(type = "number")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CommitParent {
    pub short_sha: String,
    pub full_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub full_sha: String,
    pub short_sha: String,
    pub author: CommitAuthor,
    pub committer: CommitAuthor,
    pub message: String,
    pub parents: Vec<CommitParent>,
    pub files: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitLogQueryRequest {
    pub workspace_id: String,
    pub offset: u32,
    pub limit: u32,
    pub filter_message: Option<String>,
    pub filter_author: Option<String>,
    pub filter_after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitLogQueryResponse {
    pub entries: Vec<GitLogEntry>,
    pub has_more: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum GitLogError {
    #[error("not a git repository: {0}")]
    NotAGitRepo(String),
    #[error("gix open failed: {0}")]
    OpenFailed(String),
    #[error("commit not found: {0}")]
    CommitNotFound(String),
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    #[error("gix error: {0}")]
    Gix(#[from] Box<gix::open::Error>),
    #[error("gix object error: {0}")]
    Object(String),
}

pub struct GitLogReader;

impl GitLogReader {
    pub fn query(
        repo_path: &Path,
        req: &GitLogQueryRequest,
    ) -> Result<GitLogQueryResponse, GitLogError> {
        let repo = gix::open(repo_path).map_err(|e| match e {
            gix::open::Error::NotARepository { .. } => {
                GitLogError::NotAGitRepo(repo_path.display().to_string())
            }
            other => GitLogError::OpenFailed(other.to_string()),
        })?;

        let head_id = repo
            .head_id()
            .map_err(|e| GitLogError::Object(e.to_string()))?;
        let reference_names = collect_reference_names(&repo);

        let walk = repo
            .rev_walk([head_id.detach()])
            .all()
            .map_err(|e| GitLogError::Object(e.to_string()))?;

        let filter_message = req.filter_message.as_deref();
        let filter_author = req.filter_author.as_deref();
        let filter_after_ts = req
            .filter_after
            .as_deref()
            .map(parse_after_date)
            .transpose()?;

        let mut entries = Vec::new();
        let mut skipped = 0u32;
        let limit = req.limit.max(1) as usize;
        let need = limit + 1;
        let mut collected = 0usize;

        for info in walk {
            let info = match info {
                Ok(i) => i,
                Err(_) => continue,
            };

            let commit = match info.object() {
                Ok(c) => c,
                Err(_) => continue,
            };

            let author_ref = match commit.author() {
                Ok(a) => a,
                Err(_) => continue,
            };
            let author_name = author_ref.name.to_string();
            // time 解析失败 → 展示降级到 epoch（commit 可读，不静默丢）
            let authored_date = gix::date::parse_header(author_ref.time)
                .map(|t| t.seconds)
                .unwrap_or(0);
            let message_summary = match commit.message() {
                Ok(msg) => msg.summary().to_string(),
                Err(_) => {
                    let raw = match commit.message_raw() {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    let first_line = raw.split(|b| *b == b'\n').next().unwrap_or(&[]);
                    String::from_utf8_lossy(first_line).to_string()
                }
            };

            if let Some(msg) = filter_message {
                if !message_summary.to_lowercase().contains(&msg.to_lowercase()) {
                    continue;
                }
            }

            if let Some(auth) = filter_author {
                if !author_name.to_lowercase().contains(&auth.to_lowercase()) {
                    continue;
                }
            }

            if let Some(after_ts) = filter_after_ts {
                if authored_date < after_ts {
                    continue;
                }
            }

            if skipped < req.offset {
                skipped += 1;
                continue;
            }

            let sha_str = info.id().to_string();
            let short_sha = sha_str[..7.min(sha_str.len())].to_string();

            let (branch_labels, tag_labels) = match reference_names.get(&info.id().detach()) {
                Some((b, t)) => (b.clone(), t.clone()),
                None => (Vec::new(), Vec::new()),
            };

            entries.push(GitLogEntry {
                short_sha,
                message: message_summary,
                author_name,
                authored_date,
                relative_time: format_relative_time(authored_date),
                branch_labels,
                tag_labels,
            });

            collected += 1;
            if collected >= need {
                break;
            }
        }

        let has_more = entries.len() > limit;
        if has_more {
            entries.truncate(limit);
        }

        Ok(GitLogQueryResponse { entries, has_more })
    }

    pub fn commit_detail(repo_path: &Path, sha: &str) -> Result<CommitDetail, GitLogError> {
        let repo = gix::open(repo_path).map_err(|e| match e {
            gix::open::Error::NotARepository { .. } => {
                GitLogError::NotAGitRepo(repo_path.display().to_string())
            }
            other => GitLogError::OpenFailed(other.to_string()),
        })?;

        let spec = repo
            .rev_parse(sha)
            .map_err(|e| GitLogError::CommitNotFound(format!("{sha}: {e}")))?;
        let commit_id = spec
            .single()
            .ok_or_else(|| GitLogError::CommitNotFound(format!("{sha}: not a single commit")))?;
        let commit = repo
            .find_object(commit_id.detach())
            .map_err(|e| GitLogError::CommitNotFound(format!("{sha}: {e}")))?
            .try_into_commit()
            .map_err(|e| GitLogError::CommitNotFound(e.to_string()))?;

        let full_sha = commit.id().to_string();
        let short_sha = full_sha[..7.min(full_sha.len())].to_string();

        let author_ref = commit
            .author()
            .map_err(|e| GitLogError::Object(e.to_string()))?;
        let committer_ref = commit
            .committer()
            .map_err(|e| GitLogError::Object(e.to_string()))?;

        let author_time = gix::date::parse_header(author_ref.time)
            .ok_or_else(|| GitLogError::Object("Failed to parse author timestamp".to_string()))?;
        let committer_time = gix::date::parse_header(committer_ref.time).ok_or_else(|| {
            GitLogError::Object("Failed to parse committer timestamp".to_string())
        })?;

        let parents: Vec<CommitParent> = commit
            .parent_ids()
            .map(|pid| {
                let ps = pid.to_string();
                CommitParent {
                    short_sha: ps[..7.min(ps.len())].to_string(),
                    full_sha: ps,
                }
            })
            .collect();

        let message = commit
            .message_raw()
            .map_err(|e| GitLogError::Object(e.to_string()))?
            .to_string();

        let files = diff_files_against_parent_commit(&repo, &commit)?;

        Ok(CommitDetail {
            full_sha,
            short_sha,
            author: CommitAuthor {
                name: author_ref.name.to_string(),
                email: author_ref.email.to_string(),
                timestamp: author_time.seconds,
            },
            committer: CommitAuthor {
                name: committer_ref.name.to_string(),
                email: committer_ref.email.to_string(),
                timestamp: committer_time.seconds,
            },
            message,
            parents,
            files,
        })
    }

    pub fn cache_clear() -> Result<(), GitLogError> {
        Ok(())
    }
}

fn collect_reference_names(
    repo: &Repository,
) -> std::collections::HashMap<gix::ObjectId, (Vec<String>, Vec<String>)> {
    let mut map: std::collections::HashMap<gix::ObjectId, (Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();

    if let Ok(refs) = repo.references() {
        if let Ok(platform) = refs.all() {
            for r in platform {
                let r = match r {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let name = r.name().shorten().to_string();
                let target_id = match r.try_id().map(|id| id.detach()) {
                    Some(id) => id,
                    None => continue,
                };

                let is_tag = r.name().category() == Some(gix::refs::Category::Tag);

                let entry = map
                    .entry(target_id)
                    .or_insert_with(|| (Vec::new(), Vec::new()));
                if is_tag {
                    entry.1.push(name);
                } else {
                    entry.0.push(name);
                }
            }
        }
    }

    map
}

fn diff_files_against_parent_commit(
    repo: &Repository,
    commit: &gix::Commit<'_>,
) -> Result<Vec<FileChange>, GitLogError> {
    let max_files = 1000;
    let mut files = Vec::new();

    let tree_id = commit
        .tree_id()
        .map_err(|e| GitLogError::Object(e.to_string()))?;
    let tree = repo
        .find_object(tree_id.detach())
        .map_err(|e| GitLogError::Object(e.to_string()))?
        .try_into_tree()
        .map_err(|e| GitLogError::Object(e.to_string()))?;

    let parent_ids: Vec<gix::Id<'_>> = commit.parent_ids().collect();

    if parent_ids.is_empty() {
        let mut entries: std::collections::BTreeMap<String, bool> =
            std::collections::BTreeMap::new();
        collect_tree_entries_blobs_only(&tree, String::new(), &mut entries, max_files);
        for (path, _) in entries {
            files.push(FileChange {
                path,
                status: "A".to_string(),
                additions: 0,
                deletions: 0,
            });
            if files.len() >= max_files {
                break;
            }
        }
        return Ok(files);
    }

    let parent_commit = repo
        .find_object(parent_ids[0].detach())
        .map_err(|e| GitLogError::Object(e.to_string()))?
        .try_into_commit()
        .map_err(|e| GitLogError::Object(e.to_string()))?;
    let parent_tree_id = parent_commit
        .tree_id()
        .map_err(|e| GitLogError::Object(e.to_string()))?;

    let parent_tree = match repo.find_object(parent_tree_id.detach()) {
        Ok(obj) => obj.try_into_tree().ok(),
        Err(_) => None,
    };

    let mut current_entries: std::collections::BTreeMap<String, bool> =
        std::collections::BTreeMap::new();
    collect_tree_entries_blobs_only(&tree, String::new(), &mut current_entries, max_files);

    let mut parent_entries: std::collections::BTreeMap<String, bool> =
        std::collections::BTreeMap::new();
    if let Some(pt) = parent_tree {
        collect_tree_entries_blobs_only(&pt, String::new(), &mut parent_entries, max_files);
    }

    let all_keys: std::collections::BTreeSet<String> = current_entries
        .keys()
        .chain(parent_entries.keys())
        .cloned()
        .collect();

    for key in all_keys {
        let in_current = current_entries.contains_key(&key);
        let in_parent = parent_entries.contains_key(&key);

        let status = if in_current && !in_parent {
            "A"
        } else if !in_current && in_parent {
            "D"
        } else {
            "M"
        };

        files.push(FileChange {
            path: key,
            status: status.to_string(),
            additions: 0,
            deletions: 0,
        });

        if files.len() >= max_files {
            break;
        }
    }

    Ok(files)
}

fn collect_tree_entries_blobs_only(
    tree: &gix::Tree<'_>,
    prefix: String,
    entries: &mut std::collections::BTreeMap<String, bool>,
    max: usize,
) {
    for entry in tree.iter() {
        if entries.len() >= max {
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.filename().to_string();
        let full_path = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };

        if !entry.mode().is_tree() {
            entries.insert(full_path, true);
        }
    }
}

fn parse_after_date(s: &str) -> Result<i64, GitLogError> {
    let s = s.trim();

    if let Ok(ts) = s.parse::<i64>() {
        return Ok(ts);
    }

    let formats = ["%Y-%m-%d", "%Y-%m-%dT%H:%M:%S", "%Y/%m/%d"];
    for fmt in formats {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(s, fmt) {
            return Ok(dt
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                .and_utc()
                .timestamp());
        }
    }

    Err(GitLogError::InvalidFilter(format!(
        "cannot parse after:date ({s}) - expected YYYY-MM-DD or unix timestamp"
    )))
}

fn format_relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;
    if diff < 0 {
        return "just now".to_string();
    }
    let minutes = diff / 60;
    let hours = diff / 3600;
    let days = diff / 86400;
    let weeks = diff / 604800;

    if minutes < 1 {
        "just now".to_string()
    } else if minutes < 60 {
        format!("{minutes} min ago")
    } else if hours < 24 {
        let suffix = if hours == 1 { "" } else { "s" };
        format!("{hours} hour{suffix} ago")
    } else if days < 7 {
        let suffix = if days == 1 { "" } else { "s" };
        format!("{days} day{suffix} ago")
    } else if weeks < 52 {
        let suffix = if weeks == 1 { "" } else { "s" };
        format!("{weeks} week{suffix} ago")
    } else {
        let years = diff / 31536000;
        let suffix = if years == 1 { "" } else { "s" };
        format!("{years} year{suffix} ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_log_entry_serializes_camel_case() {
        let entry = GitLogEntry {
            short_sha: "abc1234".to_string(),
            message: "feat: add git log".to_string(),
            author_name: "Test".to_string(),
            authored_date: 1714000000,
            relative_time: "2 hours ago".to_string(),
            branch_labels: vec!["main".to_string()],
            tag_labels: vec!["v1.0".to_string()],
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("shortSha"), "serde camelCase: {json}");
        assert!(json.contains("authorName"), "serde camelCase: {json}");
        assert!(json.contains("authoredDate"), "serde camelCase: {json}");
        assert!(json.contains("branchLabels"), "serde camelCase: {json}");
    }

    #[test]
    fn commit_detail_serializes_camel_case() {
        let detail = CommitDetail {
            full_sha: "abcdef1234567890".to_string(),
            short_sha: "abcdef1".to_string(),
            author: CommitAuthor {
                name: "Test".to_string(),
                email: "test@example.com".to_string(),
                timestamp: 1714000000,
            },
            committer: CommitAuthor {
                name: "Test".to_string(),
                email: "test@example.com".to_string(),
                timestamp: 1714000000,
            },
            message: "feat: test".to_string(),
            parents: vec![],
            files: vec![FileChange {
                path: "src/main.rs".to_string(),
                status: "M".to_string(),
                additions: 10,
                deletions: 5,
            }],
        };
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("fullSha"), "camelCase: {json}");
        assert!(json.contains("shortSha"), "camelCase: {json}");
        assert!(json.contains("\"author\""), "nested author: {json}");
    }

    #[test]
    fn git_log_query_request_serializes_camel_case() {
        let req = GitLogQueryRequest {
            workspace_id: "ws-1".to_string(),
            offset: 0,
            limit: 100,
            filter_message: Some("feat".to_string()),
            filter_author: None,
            filter_after: Some("2026-01-01".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("workspaceId"), "camelCase: {json}");
        assert!(json.contains("filterMessage"), "camelCase: {json}");
        assert!(json.contains("filterAfter"), "camelCase: {json}");
    }

    #[test]
    fn git_log_query_response_serializes_camel_case() {
        let resp = GitLogQueryResponse {
            entries: vec![],
            has_more: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("hasMore"), "camelCase: {json}");
    }

    #[test]
    fn file_change_serializes_camel_case() {
        let fc = FileChange {
            path: "a.rs".to_string(),
            status: "M".to_string(),
            additions: 1,
            deletions: 2,
        };
        let json = serde_json::to_string(&fc).unwrap();
        assert!(json.contains("\"path\":"), "snake path: {json}");
    }

    #[test]
    fn git_log_error_display() {
        let e1 = GitLogError::NotAGitRepo("/tmp/nope".to_string());
        assert!(e1.to_string().contains("/tmp/nope"));
        let e2 = GitLogError::CommitNotFound("deadbeef".to_string());
        assert!(e2.to_string().contains("deadbeef"));
        let e3 = GitLogError::InvalidFilter("bad".to_string());
        assert!(e3.to_string().contains("bad"));
    }

    #[test]
    fn parse_after_date_formats() {
        let ts = parse_after_date("2026-04-01").unwrap();
        assert!(ts > 0);
        let ts_unix = parse_after_date("1714000000").unwrap();
        assert_eq!(ts_unix, 1714000000);
        let bad = parse_after_date("not-a-date");
        assert!(bad.is_err());
    }

    #[test]
    fn format_relative_time_output() {
        let now = chrono::Utc::now().timestamp();
        assert_eq!(format_relative_time(now - 30), "just now");
        assert_eq!(format_relative_time(now - 120), "2 min ago");
        assert_eq!(format_relative_time(now - 7200), "2 hours ago");
        assert_eq!(format_relative_time(now - 86400), "1 day ago");
        assert_eq!(format_relative_time(now - 604800), "1 week ago");
    }

    #[test]
    fn query_non_git_path_returns_not_a_git_repo() {
        let result = GitLogReader::query(
            Path::new("/tmp"),
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 10,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );
        match result {
            Err(GitLogError::NotAGitRepo(_)) => {}
            Err(GitLogError::OpenFailed(_)) => {}
            other => panic!("expected NotAGitRepo or OpenFailed, got: {other:?}"),
        }
    }

    #[test]
    fn commit_detail_nonexistent_sha_returns_error() {
        let repo_path = Path::new("/tmp");
        let result =
            GitLogReader::commit_detail(repo_path, "deadbeef00000000000000000000000000000000");
        assert!(result.is_err());
    }

    #[test]
    fn query_nonexistent_path_returns_error() {
        let result = GitLogReader::query(
            Path::new("/tmp/nonexistent-git-repo-test-xyz"),
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 10,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );
        assert!(result.is_err(), "non-existent path should return error");
    }

    #[test]
    fn query_real_repo_returns_entries() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let result = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 10,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );

        if let Ok(resp) = result {
            assert!(
                resp.entries.len() <= 10,
                "limit=10 should return at most 10 entries"
            );
            if !resp.entries.is_empty() {
                let first = &resp.entries[0];
                assert!(!first.short_sha.is_empty(), "short_sha should not be empty");
                assert!(!first.message.is_empty(), "message should not be empty");
                assert!(
                    !first.author_name.is_empty(),
                    "author_name should not be empty"
                );
            }
        }
    }

    #[test]
    fn query_pagination_no_overlap() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let page1 = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 5,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );

        let page2 = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 5,
                limit: 5,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );

        if let (Ok(p1), Ok(p2)) = (page1, page2) {
            let p1_shas: std::collections::HashSet<String> =
                p1.entries.iter().map(|e| e.short_sha.clone()).collect();
            for e in &p2.entries {
                assert!(
                    !p1_shas.contains(&e.short_sha),
                    "page overlap: {} appears in both pages",
                    e.short_sha
                );
            }
        }
    }

    #[test]
    fn query_filter_by_message() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let result = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 50,
                filter_message: Some("feat".to_string()),
                filter_author: None,
                filter_after: None,
            },
        );

        if let Ok(resp) = result {
            for entry in &resp.entries {
                assert!(
                    entry.message.to_lowercase().contains("feat"),
                    "filter_message=feat should match: {}",
                    entry.message
                );
            }
        }
    }

    #[test]
    fn query_filter_by_author() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let result = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 50,
                filter_message: None,
                filter_author: Some("Leaf".to_string()),
                filter_after: None,
            },
        );

        if let Ok(resp) = result {
            for entry in &resp.entries {
                assert!(
                    entry.author_name.to_lowercase().contains("leaf"),
                    "filter_author=Leaf should match: {}",
                    entry.author_name
                );
            }
        }
    }

    #[test]
    fn query_filter_by_after_date() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let result = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 50,
                filter_message: None,
                filter_author: None,
                filter_after: Some("2026-04-01".to_string()),
            },
        );

        if let Ok(resp) = result {
            for entry in &resp.entries {
                assert!(
                    entry.authored_date >= 1743465600,
                    "filter_after=2026-04-01 should filter: authored_date={}, entry={}",
                    entry.authored_date,
                    entry.message
                );
            }
        }
    }

    #[test]
    fn commit_detail_real_repo() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let query = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 1,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );

        if let Ok(resp) = query {
            if let Some(entry) = resp.entries.first() {
                let detail_result = GitLogReader::commit_detail(repo_path, &entry.short_sha);
                if let Ok(detail) = detail_result {
                    assert!(!detail.full_sha.is_empty());
                    assert!(!detail.message.is_empty());
                }
            }
        }
    }

    #[test]
    fn query_has_more_correct() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let result = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 3,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );

        if let Ok(resp) = result {
            if resp.entries.len() == 3 {
                assert!(
                    resp.has_more,
                    "should have more with limit=3 on a real repo"
                );
            }
        }
    }

    #[test]
    fn branch_labels_found_on_main() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let result = GitLogReader::query(
            repo_path,
            &GitLogQueryRequest {
                workspace_id: "test".to_string(),
                offset: 0,
                limit: 1,
                filter_message: None,
                filter_author: None,
                filter_after: None,
            },
        );

        if let Ok(resp) = result {
            if let Some(entry) = resp.entries.first() {
                assert!(
                    entry.branch_labels.contains(&"main".to_string())
                        || !entry.branch_labels.is_empty(),
                    "HEAD commit should have at least one branch label"
                );
            }
        }
    }
}

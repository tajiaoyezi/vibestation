use std::path::Path;

use git2::{ErrorCode, Repository as Git2Repository};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use ts_rs::TS;

use crate::{
    app_settings::{AppSettingsStore, SettingsError},
    db::DbPool,
};

const DEFAULT_VIEW_MODE: &str = "split";
const LARGE_FILE_BYTES: usize = 1_000_000;
const MAX_TEXT_LINES: usize = 100_000;

type FileBytes = Option<Vec<u8>>;
type DiffSourcePair = (FileBytes, FileBytes);

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DiffRequest {
    pub workspace_id: String,
    pub source: String,
    pub file_path: String,
    #[serde(default)]
    pub allow_large_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DiffResponse {
    pub hunks: Vec<DiffHunk>,
    pub binary: bool,
    pub truncated: bool,
    pub truncated_reason: Option<String>,
    pub old_size_bytes: Option<u32>,
    pub new_size_bytes: Option<u32>,
    pub line_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_num: Option<u32>,
    pub new_line_num: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
}

#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error("not a git repository: {0}")]
    NotAGitRepo(String),
    #[error("git2 open failed: {0}")]
    GitOpen(String),
    #[error("invalid diff source: {0}")]
    InvalidSource(String),
    #[error("git2 error: {0}")]
    Git2(String),
    #[error("gix error: {0}")]
    Gix(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("setting error: {0}")]
    Settings(String),
    #[error("invalid view mode: {0:?}, expected \"split\" or \"unified\"")]
    InvalidViewMode(String),
}

impl From<std::io::Error> for DiffError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<git2::Error> for DiffError {
    fn from(error: git2::Error) -> Self {
        Self::Git2(error.to_string())
    }
}

impl From<SettingsError> for DiffError {
    fn from(error: SettingsError) -> Self {
        Self::Settings(error.to_string())
    }
}

pub struct DiffService;

impl DiffService {
    pub fn compute(repo_path: &Path, req: &DiffRequest) -> Result<DiffResponse, DiffError> {
        let (old_bytes, new_bytes) = load_source_pair(repo_path, req)?;
        build_response(
            old_bytes.as_deref(),
            new_bytes.as_deref(),
            req.allow_large_file,
        )
    }

    pub fn get_view_mode(pool: &DbPool, workspace_id: &str) -> Result<String, DiffError> {
        let key = format!("diff_view_mode:{workspace_id}");
        match AppSettingsStore::get(pool, &key) {
            Ok(value) => Ok(value),
            Err(SettingsError::NotFound(_)) => Ok(DEFAULT_VIEW_MODE.to_string()),
            Err(other) => Err(other.into()),
        }
    }

    pub fn set_view_mode(
        pool: &DbPool,
        workspace_id: &str,
        view_mode: &str,
    ) -> Result<(), DiffError> {
        match view_mode {
            "split" | "unified" => {}
            other => return Err(DiffError::InvalidViewMode(other.to_string())),
        }
        let key = format!("diff_view_mode:{workspace_id}");
        AppSettingsStore::set(pool, &key, view_mode).map_err(Into::into)
    }
}

fn load_source_pair(repo_path: &Path, req: &DiffRequest) -> Result<DiffSourcePair, DiffError> {
    match req.source.as_str() {
        "unstaged" => load_unstaged_pair(repo_path, &req.file_path),
        "staged" => load_staged_pair(repo_path, &req.file_path),
        commit_sha if !commit_sha.trim().is_empty() => {
            load_commit_pair(repo_path, commit_sha, &req.file_path)
        }
        _ => Err(DiffError::InvalidSource(req.source.clone())),
    }
}

fn load_unstaged_pair(repo_path: &Path, file_path: &str) -> Result<DiffSourcePair, DiffError> {
    let repo = open_git2_repo(repo_path)?;
    let path = Path::new(file_path);
    let index = repo.index()?;
    let old = read_index_blob(&repo, &index, path)?;
    let new = read_workdir_file(&repo, path)?;
    Ok((old, new))
}

fn load_staged_pair(repo_path: &Path, file_path: &str) -> Result<DiffSourcePair, DiffError> {
    let repo = open_git2_repo(repo_path)?;
    let path = Path::new(file_path);
    let index = repo.index()?;
    let head_tree = current_head_tree(&repo)?;
    let old = match head_tree.as_ref() {
        Some(tree) => read_tree_blob(&repo, tree, path)?,
        None => None,
    };
    let new = read_index_blob(&repo, &index, path)?;
    Ok((old, new))
}

fn load_commit_pair(
    repo_path: &Path,
    commit_sha: &str,
    file_path: &str,
) -> Result<DiffSourcePair, DiffError> {
    let repo = gix::open(repo_path).map_err(|error| DiffError::Gix(error.to_string()))?;
    let id = repo
        .rev_parse_single(commit_sha)
        .map_err(|error| DiffError::Gix(error.to_string()))?;
    let commit = id
        .object()
        .map_err(|error| DiffError::Gix(error.to_string()))?
        .try_into_commit()
        .map_err(|error| DiffError::Gix(error.to_string()))?;

    let mut tree = commit
        .tree()
        .map_err(|error| DiffError::Gix(error.to_string()))?;
    let new = read_gix_tree_blob(&mut tree, Path::new(file_path))?;

    let old = match commit.parent_ids().next() {
        Some(parent_id) => {
            let parent_commit = parent_id
                .object()
                .map_err(|error| DiffError::Gix(error.to_string()))?
                .try_into_commit()
                .map_err(|error| DiffError::Gix(error.to_string()))?;
            let mut parent_tree = parent_commit
                .tree()
                .map_err(|error| DiffError::Gix(error.to_string()))?;
            read_gix_tree_blob(&mut parent_tree, Path::new(file_path))?
        }
        None => None,
    };

    Ok((old, new))
}

fn open_git2_repo(repo_path: &Path) -> Result<Git2Repository, DiffError> {
    Git2Repository::open(repo_path).map_err(|error| match error.code() {
        ErrorCode::NotFound => DiffError::NotAGitRepo(repo_path.display().to_string()),
        _ => DiffError::GitOpen(error.to_string()),
    })
}

fn current_head_tree(repo: &Git2Repository) -> Result<Option<git2::Tree<'_>>, DiffError> {
    match repo.head() {
        Ok(head) => Ok(Some(head.peel_to_tree()?)),
        Err(error)
            if error.code() == ErrorCode::UnbornBranch || error.code() == ErrorCode::NotFound =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

fn read_workdir_file(repo: &Git2Repository, path: &Path) -> Result<Option<Vec<u8>>, DiffError> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| DiffError::GitOpen("workdir not available".to_string()))?;
    let full_path = workdir.join(path);
    match std::fs::read(full_path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn read_index_blob(
    repo: &Git2Repository,
    index: &git2::Index,
    path: &Path,
) -> Result<Option<Vec<u8>>, DiffError> {
    match index.get_path(path, 0) {
        Some(entry) => {
            let blob = repo.find_blob(entry.id)?;
            Ok(Some(blob.content().to_vec()))
        }
        None => Ok(None),
    }
}

fn read_tree_blob(
    repo: &Git2Repository,
    tree: &git2::Tree<'_>,
    path: &Path,
) -> Result<Option<Vec<u8>>, DiffError> {
    match tree.get_path(path) {
        Ok(entry) => {
            let blob = entry.to_object(repo)?.peel_to_blob()?;
            Ok(Some(blob.content().to_vec()))
        }
        Err(error) if error.code() == ErrorCode::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn read_gix_tree_blob(tree: &mut gix::Tree<'_>, path: &Path) -> Result<Option<Vec<u8>>, DiffError> {
    let Some(entry) = tree
        .peel_to_entry_by_path(path)
        .map_err(|error| DiffError::Gix(error.to_string()))?
    else {
        return Ok(None);
    };

    if entry.mode().is_tree() {
        return Ok(None);
    }

    let blob = entry
        .object()
        .map_err(|error| DiffError::Gix(error.to_string()))?
        .try_into_blob()
        .map_err(|error| DiffError::Gix(error.to_string()))?;
    Ok(Some(blob.data.clone()))
}

fn build_response(
    old_bytes: Option<&[u8]>,
    new_bytes: Option<&[u8]>,
    allow_large_file: bool,
) -> Result<DiffResponse, DiffError> {
    let old_size = old_bytes.map(|bytes| bytes.len() as u32);
    let new_size = new_bytes.map(|bytes| bytes.len() as u32);

    if is_binary(old_bytes) || is_binary(new_bytes) {
        return Ok(DiffResponse {
            hunks: Vec::new(),
            binary: true,
            truncated: false,
            truncated_reason: None,
            old_size_bytes: old_size,
            new_size_bytes: new_size,
            line_count: None,
        });
    }

    let max_size = old_bytes
        .map_or(0, |bytes| bytes.len())
        .max(new_bytes.map_or(0, |bytes| bytes.len()));

    if max_size > LARGE_FILE_BYTES && !allow_large_file {
        return Ok(DiffResponse {
            hunks: Vec::new(),
            binary: false,
            truncated: true,
            truncated_reason: Some("large_file".to_string()),
            old_size_bytes: old_size,
            new_size_bytes: new_size,
            line_count: None,
        });
    }

    let old_text = std::str::from_utf8(old_bytes.unwrap_or_default())
        .map_err(|error| DiffError::Io(error.to_string()))?;
    let new_text = std::str::from_utf8(new_bytes.unwrap_or_default())
        .map_err(|error| DiffError::Io(error.to_string()))?;

    let max_lines = old_text.lines().count().max(new_text.lines().count());
    if max_lines > MAX_TEXT_LINES {
        return Ok(DiffResponse {
            hunks: Vec::new(),
            binary: false,
            truncated: true,
            truncated_reason: Some("too_many_lines".to_string()),
            old_size_bytes: old_size,
            new_size_bytes: new_size,
            line_count: Some(max_lines as u32),
        });
    }

    let diff = TextDiff::from_lines(old_text, new_text);
    let mut hunks = Vec::new();

    for group in diff.grouped_ops(3) {
        let Some(first_op) = group.first() else {
            continue;
        };
        let old_start = first_op.old_range().start as u32 + 1;
        let new_start = first_op.new_range().start as u32 + 1;

        let mut lines = Vec::new();

        for op in group {
            let mut old_line = op.old_range().start as u32 + 1;
            let mut new_line = op.new_range().start as u32 + 1;

            for change in diff.iter_changes(&op) {
                let content = strip_trailing_newline(&change.to_string());
                match change.tag() {
                    ChangeTag::Delete => {
                        lines.push(DiffLine {
                            line_type: DiffLineType::Removed,
                            content,
                            old_line_num: Some(old_line),
                            new_line_num: None,
                        });
                        old_line += 1;
                    }
                    ChangeTag::Insert => {
                        lines.push(DiffLine {
                            line_type: DiffLineType::Added,
                            content,
                            old_line_num: None,
                            new_line_num: Some(new_line),
                        });
                        new_line += 1;
                    }
                    ChangeTag::Equal => {
                        lines.push(DiffLine {
                            line_type: DiffLineType::Context,
                            content,
                            old_line_num: Some(old_line),
                            new_line_num: Some(new_line),
                        });
                        old_line += 1;
                        new_line += 1;
                    }
                }
            }
        }

        hunks.push(DiffHunk {
            old_start,
            new_start,
            lines,
        });
    }

    Ok(DiffResponse {
        hunks,
        binary: false,
        truncated: false,
        truncated_reason: None,
        old_size_bytes: old_size,
        new_size_bytes: new_size,
        line_count: Some(max_lines as u32),
    })
}

fn is_binary(bytes: Option<&[u8]>) -> bool {
    match bytes {
        Some(bytes) => bytes.contains(&0),
        None => false,
    }
}

fn strip_trailing_newline(content: &str) -> String {
    content
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_diff_preserves_added_removed_and_context_lines() {
        let response = build_response(
            Some(b"line1\nline2\nline4\n"),
            Some(b"line1\nline3\nline4\n"),
            false,
        )
        .unwrap();
        assert!(!response.binary);
        assert!(!response.truncated);
        assert_eq!(response.truncated_reason, None);
        assert_eq!(response.line_count, Some(3));
        assert_eq!(response.hunks.len(), 1);
        let lines = &response.hunks[0].lines;
        assert!(matches!(lines[0].line_type, DiffLineType::Context));
        assert!(matches!(lines[1].line_type, DiffLineType::Removed));
        assert!(matches!(lines[2].line_type, DiffLineType::Added));
        assert!(matches!(lines[3].line_type, DiffLineType::Context));
        assert_eq!(lines[1].old_line_num, Some(2));
        assert_eq!(lines[2].new_line_num, Some(2));
    }

    #[test]
    fn binary_files_short_circuit() {
        let response = build_response(Some(&[1, 2, 0, 3]), Some(b"text"), false).unwrap();
        assert!(response.binary);
        assert!(!response.truncated);
        assert!(response.hunks.is_empty());
    }

    #[test]
    fn large_files_are_marked_truncated() {
        let old = vec![b'a'; LARGE_FILE_BYTES + 1];
        let response = build_response(Some(&old), Some(b"small"), false).unwrap();
        assert!(response.truncated);
        assert!(!response.binary);
        assert!(response.hunks.is_empty());
        assert_eq!(response.truncated_reason.as_deref(), Some("large_file"));
        assert_eq!(response.line_count, None);
    }

    #[test]
    fn large_files_can_be_loaded_when_explicitly_allowed() {
        let content = format!("{}\n", "a".repeat(LARGE_FILE_BYTES + 1));
        let response =
            build_response(Some(content.as_bytes()), Some(content.as_bytes()), true).unwrap();
        assert!(!response.binary);
        assert!(!response.truncated);
        assert_eq!(response.truncated_reason, None);
        assert_eq!(response.line_count, Some(1));
    }

    #[test]
    fn binary_detection_beats_large_file_truncation() {
        let mut old = vec![1_u8; LARGE_FILE_BYTES + 1];
        old[128] = 0;
        let response = build_response(Some(&old), None, false).unwrap();
        assert!(response.binary);
        assert!(!response.truncated);
        assert_eq!(response.truncated_reason, None);
    }

    #[test]
    fn too_many_lines_are_marked_truncated() {
        let text = "x\n".repeat(MAX_TEXT_LINES + 1);
        let response = build_response(Some(text.as_bytes()), Some(text.as_bytes()), true).unwrap();
        assert!(response.truncated);
        assert_eq!(response.truncated_reason.as_deref(), Some("too_many_lines"));
        assert_eq!(response.line_count, Some((MAX_TEXT_LINES + 1) as u32));
    }

    #[test]
    fn get_view_mode_defaults_to_split() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("diff_settings.db");
        let pool = crate::db::open_pool(&db_path).unwrap();
        assert_eq!(DiffService::get_view_mode(&pool, "ws-1").unwrap(), "split");
    }

    #[test]
    fn set_view_mode_roundtrip_unified() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("diff_settings.db");
        let pool = crate::db::open_pool(&db_path).unwrap();
        DiffService::set_view_mode(&pool, "ws-1", "unified").unwrap();
        assert_eq!(
            DiffService::get_view_mode(&pool, "ws-1").unwrap(),
            "unified"
        );
    }

    #[test]
    fn set_view_mode_overwrites_previous() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("diff_settings.db");
        let pool = crate::db::open_pool(&db_path).unwrap();
        DiffService::set_view_mode(&pool, "ws-1", "split").unwrap();
        DiffService::set_view_mode(&pool, "ws-1", "unified").unwrap();
        assert_eq!(
            DiffService::get_view_mode(&pool, "ws-1").unwrap(),
            "unified"
        );
    }

    #[test]
    fn set_view_mode_rejects_invalid_value() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("diff_settings.db");
        let pool = crate::db::open_pool(&db_path).unwrap();
        let err = DiffService::set_view_mode(&pool, "ws-1", "side-by-side").unwrap_err();
        assert!(matches!(err, DiffError::InvalidViewMode(ref v) if v == "side-by-side"));
        assert_eq!(DiffService::get_view_mode(&pool, "ws-1").unwrap(), "split");
    }
}

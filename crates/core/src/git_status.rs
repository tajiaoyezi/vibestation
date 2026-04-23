use std::{cell::RefCell, collections::HashMap, path::Path};

use git2::{DiffDelta, DiffOptions, ErrorCode, Repository as Git2Repository, Status};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    app_settings::{AppSettingsStore, SettingsError},
    db::DbPool,
    FileChange,
};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusRequest {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusResponse {
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
}

impl GitStatusResponse {
    #[must_use]
    pub fn to_event(&self, workspace_id: impl Into<String>) -> FileStatusEvent {
        FileStatusEvent {
            workspace_id: workspace_id.into(),
            staged: self.staged.clone(),
            unstaged: self.unstaged.clone(),
            untracked: self.untracked.clone(),
        }
    }

    #[must_use]
    pub fn equivalent(&self, other: &Self) -> bool {
        same_file_changes(&self.staged, &other.staged)
            && same_file_changes(&self.unstaged, &other.unstaged)
            && same_file_changes(&self.untracked, &other.untracked)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FileStatusEvent {
    pub workspace_id: String,
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusPanelSettings {
    pub staged_collapsed: bool,
    pub unstaged_collapsed: bool,
    pub untracked_collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusCollapseRequest {
    pub workspace_id: String,
    pub group: GitStatusGroup,
    pub collapsed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum GitStatusGroup {
    Staged,
    Unstaged,
    Untracked,
}

#[derive(Debug, thiserror::Error)]
pub enum GitStatusError {
    #[error("not a git repository: {0}")]
    NotAGitRepo(String),
    #[error("git2 open failed: {0}")]
    GitOpen(String),
    #[error("git2 error: {0}")]
    Git2(String),
    #[error("setting error: {0}")]
    Settings(String),
}

impl From<git2::Error> for GitStatusError {
    fn from(error: git2::Error) -> Self {
        Self::Git2(error.to_string())
    }
}

impl From<SettingsError> for GitStatusError {
    fn from(error: SettingsError) -> Self {
        Self::Settings(error.to_string())
    }
}

pub struct GitStatusService;

impl GitStatusService {
    pub fn query(
        repo_path: &Path,
        _req: &GitStatusRequest,
    ) -> Result<GitStatusResponse, GitStatusError> {
        let repo = open_repo(repo_path)?;
        let index = repo.index()?;

        let staged_stats = collect_staged_stats(&repo, &index)?;
        let unstaged_stats = collect_unstaged_stats(&repo, &index)?;

        let mut status_options = git2::StatusOptions::new();
        status_options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = repo.statuses(Some(&mut status_options))?;

        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = status_entry_path(&entry);

            if is_staged(status) {
                let (additions, deletions) = staged_stats.get(&path).copied().unwrap_or((0, 0));
                staged.push(FileChange {
                    path: path.clone(),
                    status: staged_status_code(status).to_string(),
                    additions,
                    deletions,
                });
            }

            if status.contains(Status::WT_NEW) {
                untracked.push(FileChange {
                    path: path.clone(),
                    status: "?".to_string(),
                    additions: 0,
                    deletions: 0,
                });
                continue;
            }

            if is_unstaged(status) {
                let (additions, deletions) = unstaged_stats.get(&path).copied().unwrap_or((0, 0));
                unstaged.push(FileChange {
                    path,
                    status: unstaged_status_code(status).to_string(),
                    additions,
                    deletions,
                });
            }
        }

        staged.sort_by(|left, right| left.path.cmp(&right.path));
        unstaged.sort_by(|left, right| left.path.cmp(&right.path));
        untracked.sort_by(|left, right| left.path.cmp(&right.path));

        Ok(GitStatusResponse {
            staged,
            unstaged,
            untracked,
        })
    }

    pub fn refresh(
        repo_path: &Path,
        req: &GitStatusRequest,
    ) -> Result<GitStatusResponse, GitStatusError> {
        Self::query(repo_path, req)
    }

    pub fn subscribe(_workspace_id: &str) {}

    pub fn unsubscribe(_workspace_id: &str) {}

    pub fn get_panel_settings(
        pool: &DbPool,
        workspace_id: &str,
    ) -> Result<GitStatusPanelSettings, GitStatusError> {
        Ok(GitStatusPanelSettings {
            staged_collapsed: get_collapsed(pool, workspace_id, GitStatusGroup::Staged)?,
            unstaged_collapsed: get_collapsed(pool, workspace_id, GitStatusGroup::Unstaged)?,
            untracked_collapsed: get_collapsed(pool, workspace_id, GitStatusGroup::Untracked)?,
        })
    }

    pub fn set_group_collapsed(
        pool: &DbPool,
        workspace_id: &str,
        group: GitStatusGroup,
        collapsed: bool,
    ) -> Result<(), GitStatusError> {
        let key = status_panel_setting_key(workspace_id, group);
        let value = if collapsed { "true" } else { "false" };
        AppSettingsStore::set(pool, &key, value)?;
        Ok(())
    }
}

fn get_collapsed(
    pool: &DbPool,
    workspace_id: &str,
    group: GitStatusGroup,
) -> Result<bool, GitStatusError> {
    let key = status_panel_setting_key(workspace_id, group);
    match AppSettingsStore::get(pool, &key) {
        Ok(value) => Ok(value == "true"),
        Err(SettingsError::NotFound(_)) => Ok(false),
        Err(other) => Err(other.into()),
    }
}

fn status_panel_setting_key(workspace_id: &str, group: GitStatusGroup) -> String {
    let suffix = match group {
        GitStatusGroup::Staged => "staged",
        GitStatusGroup::Unstaged => "unstaged",
        GitStatusGroup::Untracked => "untracked",
    };
    format!("status_panel_collapsed:{workspace_id}:{suffix}")
}

fn same_file_changes(left: &[FileChange], right: &[FileChange]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right.iter()).all(|(left, right)| {
            left.path == right.path
                && left.status == right.status
                && left.additions == right.additions
                && left.deletions == right.deletions
        })
}

fn open_repo(repo_path: &Path) -> Result<Git2Repository, GitStatusError> {
    Git2Repository::open(repo_path).map_err(|error| match error.code() {
        ErrorCode::NotFound => GitStatusError::NotAGitRepo(repo_path.display().to_string()),
        _ => GitStatusError::GitOpen(error.to_string()),
    })
}

fn collect_staged_stats(
    repo: &Git2Repository,
    index: &git2::Index,
) -> Result<HashMap<String, (u32, u32)>, GitStatusError> {
    let head_tree = current_head_tree(repo)?;
    let mut diff_options = DiffOptions::new();
    diff_options.include_typechange(true);
    let diff = repo.diff_tree_to_index(head_tree.as_ref(), Some(index), Some(&mut diff_options))?;
    collect_diff_stats(&diff)
}

fn collect_unstaged_stats(
    repo: &Git2Repository,
    index: &git2::Index,
) -> Result<HashMap<String, (u32, u32)>, GitStatusError> {
    let mut diff_options = DiffOptions::new();
    diff_options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_typechange(true);
    let diff = repo.diff_index_to_workdir(Some(index), Some(&mut diff_options))?;
    collect_diff_stats(&diff)
}

fn current_head_tree(repo: &Git2Repository) -> Result<Option<git2::Tree<'_>>, GitStatusError> {
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

fn collect_diff_stats(
    diff: &git2::Diff<'_>,
) -> Result<HashMap<String, (u32, u32)>, GitStatusError> {
    let counts: RefCell<HashMap<String, (u32, u32)>> = RefCell::new(HashMap::new());

    diff.foreach(
        &mut |delta, _| {
            counts
                .borrow_mut()
                .entry(diff_delta_path(delta))
                .or_insert((0, 0));
            true
        },
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            let mut counts = counts.borrow_mut();
            let entry = counts.entry(diff_delta_path(delta)).or_insert((0, 0));
            match line.origin() {
                '+' => entry.0 += 1,
                '-' => entry.1 += 1,
                _ => {}
            }
            true
        }),
    )?;

    Ok(counts.into_inner())
}

fn status_entry_path(entry: &git2::StatusEntry<'_>) -> String {
    entry
        .index_to_workdir()
        .map(diff_delta_path)
        .or_else(|| entry.head_to_index().map(diff_delta_path))
        .or_else(|| entry.path().map(|path| path.to_string()))
        .unwrap_or_default()
}

fn diff_delta_path(delta: DiffDelta<'_>) -> String {
    let old_path = delta
        .old_file()
        .path()
        .map(|path| path.to_string_lossy().to_string());
    let new_path = delta
        .new_file()
        .path()
        .map(|path| path.to_string_lossy().to_string());

    match (old_path, new_path) {
        (Some(old), Some(new)) if old != new => format!("{old} → {new}"),
        (_, Some(new)) => new,
        (Some(old), None) => old,
        _ => String::new(),
    }
}

fn is_staged(status: Status) -> bool {
    status.intersects(
        Status::INDEX_NEW
            | Status::INDEX_MODIFIED
            | Status::INDEX_DELETED
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE,
    )
}

fn is_unstaged(status: Status) -> bool {
    status.intersects(
        Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED | Status::WT_TYPECHANGE,
    )
}

fn staged_status_code(status: Status) -> &'static str {
    if status.contains(Status::INDEX_NEW) {
        "A"
    } else if status.contains(Status::INDEX_DELETED) {
        "D"
    } else if status.contains(Status::INDEX_RENAMED) {
        "R"
    } else {
        "M"
    }
}

fn unstaged_status_code(status: Status) -> &'static str {
    if status.contains(Status::WT_DELETED) {
        "D"
    } else if status.contains(Status::WT_RENAMED) {
        "R"
    } else {
        "M"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use git2::{IndexAddOption, Signature};
    use tempfile::TempDir;

    fn setup_repo() -> (TempDir, Git2Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = Git2Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    fn setup_settings_pool() -> (TempDir, DbPool) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("git_status_settings.db");
        let pool = db::open_pool(&db_path).unwrap();
        (dir, pool)
    }

    fn commit_all(repo: &Git2Repository, message: &str) {
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let parent = repo
            .head()
            .ok()
            .and_then(|head| head.target())
            .and_then(|oid| repo.find_commit(oid).ok());

        match parent {
            Some(parent) => {
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
                    .unwrap();
            }
            None => {
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                    .unwrap();
            }
        }
    }

    #[test]
    fn query_separates_staged_unstaged_and_untracked() {
        let (dir, repo) = setup_repo();
        std::fs::write(dir.path().join("tracked.txt"), "one\n").unwrap();
        commit_all(&repo, "initial");

        std::fs::write(dir.path().join("tracked.txt"), "one\ntwo\n").unwrap();
        std::fs::write(dir.path().join("added.txt"), "added\n").unwrap();
        std::fs::write(dir.path().join("new.txt"), "new\n").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("added.txt")).unwrap();
        index.write().unwrap();

        let response = GitStatusService::query(
            dir.path(),
            &GitStatusRequest {
                workspace_id: "ws-1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(response.staged.len(), 1);
        assert_eq!(response.staged[0].path, "added.txt");
        assert_eq!(response.staged[0].status, "A");
        assert_eq!(response.unstaged.len(), 1);
        assert_eq!(response.unstaged[0].path, "tracked.txt");
        assert_eq!(response.untracked.len(), 1);
        assert_eq!(response.untracked[0].path, "new.txt");
        assert_eq!(response.untracked[0].status, "?");
    }

    #[test]
    fn same_file_can_appear_in_staged_and_unstaged() {
        let (dir, repo) = setup_repo();
        std::fs::write(dir.path().join("combo.txt"), "one\n").unwrap();
        commit_all(&repo, "initial");

        std::fs::write(dir.path().join("combo.txt"), "two\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("combo.txt")).unwrap();
        index.write().unwrap();
        std::fs::write(dir.path().join("combo.txt"), "three\n").unwrap();

        let response = GitStatusService::query(
            dir.path(),
            &GitStatusRequest {
                workspace_id: "ws-1".to_string(),
            },
        )
        .unwrap();

        assert!(response.staged.iter().any(|file| file.path == "combo.txt"));
        assert!(response
            .unstaged
            .iter()
            .any(|file| file.path == "combo.txt"));
    }

    #[test]
    fn panel_settings_default_to_expanded() {
        let (_dir, pool) = setup_settings_pool();
        let settings = GitStatusService::get_panel_settings(&pool, "ws-1").unwrap();
        assert!(!settings.staged_collapsed);
        assert!(!settings.unstaged_collapsed);
        assert!(!settings.untracked_collapsed);
    }

    #[test]
    fn set_group_collapsed_round_trips() {
        let (_dir, pool) = setup_settings_pool();
        GitStatusService::set_group_collapsed(&pool, "ws-1", GitStatusGroup::Untracked, true)
            .unwrap();
        let settings = GitStatusService::get_panel_settings(&pool, "ws-1").unwrap();
        assert!(!settings.staged_collapsed);
        assert!(!settings.unstaged_collapsed);
        assert!(settings.untracked_collapsed);
    }

    #[test]
    fn response_equivalent_requires_matching_file_contents() {
        let base = GitStatusResponse {
            staged: vec![FileChange {
                path: "tracked.txt".to_string(),
                status: "M".to_string(),
                additions: 2,
                deletions: 1,
            }],
            unstaged: vec![],
            untracked: vec![FileChange {
                path: "new.txt".to_string(),
                status: "?".to_string(),
                additions: 0,
                deletions: 0,
            }],
        };

        let same = GitStatusResponse {
            staged: base.staged.clone(),
            unstaged: vec![],
            untracked: base.untracked.clone(),
        };
        let changed = GitStatusResponse {
            staged: vec![FileChange {
                path: "tracked.txt".to_string(),
                status: "M".to_string(),
                additions: 3,
                deletions: 1,
            }],
            unstaged: vec![],
            untracked: base.untracked.clone(),
        };

        assert!(base.equivalent(&same));
        assert!(!base.equivalent(&changed));
    }

    #[test]
    fn response_to_event_keeps_workspace_and_groups() {
        let response = GitStatusResponse {
            staged: vec![FileChange {
                path: "staged.txt".to_string(),
                status: "A".to_string(),
                additions: 4,
                deletions: 0,
            }],
            unstaged: vec![FileChange {
                path: "unstaged.txt".to_string(),
                status: "M".to_string(),
                additions: 1,
                deletions: 2,
            }],
            untracked: vec![],
        };

        let event = response.to_event("ws-1");
        assert_eq!(event.workspace_id, "ws-1");
        assert_eq!(event.staged.len(), 1);
        assert_eq!(event.staged[0].path, "staged.txt");
        assert_eq!(event.unstaged.len(), 1);
        assert_eq!(event.unstaged[0].path, "unstaged.txt");
        assert!(event.untracked.is_empty());
    }
}

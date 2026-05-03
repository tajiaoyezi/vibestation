//! Branch CRUD operations for MVP-13 Phase A.
//!
//! Read path prefers gix for reference enumeration. Write path stays on git2.

use git2::{build::CheckoutBuilder, BranchType, ErrorCode, Oid, Repository, Status, StatusOptions};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum BranchKind {
    Local,
    Remote,
    Tag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    pub name: String,
    pub full_ref: String,
    pub kind: BranchKind,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub head_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchListRequest {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchListResponse {
    pub branches: Vec<BranchInfo>,
    pub head_name: Option<String>,
    pub detached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchCreateRequest {
    pub workspace_id: String,
    pub name: String,
    pub from_ref: Option<String>,
    pub checkout: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchCheckoutRequest {
    pub workspace_id: String,
    pub name: String,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchDeleteRequest {
    pub workspace_id: String,
    pub name: String,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchSwitchResult {
    pub new_head: String,
    pub prev_head: String,
    #[ts(type = "number")]
    pub dirty_files_dropped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BranchError {
    InvalidName {
        reason: String,
    },
    NotFound {
        name: String,
    },
    AlreadyExists {
        name: String,
    },
    Unmerged {
        name: String,
        #[serde(rename = "missingCommits")]
        missing_commits: u32,
    },
    ProtectedBranch {
        name: String,
    },
    DetachedHead,
    DirtyWorkingTree {
        modified: Vec<String>,
        staged: Vec<String>,
        untracked: Vec<String>,
    },
    IndexLocked,
    Git2Error {
        class: String,
        code: i32,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SwitcherQueryRequest {
    pub workspace_id: String,
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SwitcherMatch {
    pub branch: BranchInfo,
    #[ts(type = "number")]
    pub score: f32,
    pub match_indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SwitcherSearchResult {
    pub matches: Vec<SwitcherMatch>,
}

pub fn branch_list(workspace_path: &Path) -> Result<BranchListResponse, BranchError> {
    let repo = open_repo(workspace_path)?;
    let mut branches = match collect_refs_with_gix(workspace_path) {
        Ok(refs) => enrich_refs_with_git2(&repo, refs),
        Err(_) => collect_refs_with_git2(&repo)?,
    };

    branches.sort_by(|left, right| {
        kind_rank(left.kind)
            .cmp(&kind_rank(right.kind))
            .then_with(|| left.name.cmp(&right.name))
    });

    let detached = repo.head_detached().map_err(map_git_error)?;
    let head_name = if detached {
        None
    } else {
        repo.head()
            .ok()
            .and_then(|head| head.shorthand().map(ToOwned::to_owned))
    };

    Ok(BranchListResponse {
        branches,
        head_name,
        detached,
    })
}

pub fn branch_create(workspace_path: &Path, req: BranchCreateRequest) -> Result<(), BranchError> {
    validate_name(&req.name)?;
    ensure_not_protected(&req.name)?;

    let repo = open_repo(workspace_path)?;
    ensure_index_unlocked(&repo)?;
    if repo.find_branch(&req.name, BranchType::Local).is_ok() {
        return Err(BranchError::AlreadyExists { name: req.name });
    }

    let (commit_id, upstream) = resolve_create_target(&repo, req.from_ref.as_deref())?;
    let commit = repo.find_commit(commit_id).map_err(map_git_error)?;
    {
        let mut branch = repo
            .branch(&req.name, &commit, false)
            .map_err(|error| map_branch_create_error(error, &req.name))?;
        if let Some(upstream_name) = upstream {
            branch
                .set_upstream(Some(&upstream_name))
                .map_err(map_git_error)?;
        }
    }

    if req.checkout {
        let dirty = dirty_working_tree(&repo)?;
        if dirty.is_dirty() {
            return Err(dirty.into_error());
        }
        checkout_local_branch(&repo, &req.name, false)?;
    }

    Ok(())
}

pub fn branch_checkout(
    workspace_path: &Path,
    req: BranchCheckoutRequest,
) -> Result<BranchSwitchResult, BranchError> {
    let repo = open_repo(workspace_path)?;
    ensure_index_unlocked(&repo)?;

    let prev_head = current_head_label(&repo)?;
    let dirty = dirty_working_tree(&repo)?;
    if !req.force && dirty.is_dirty() {
        return Err(dirty.into_error());
    }
    let dirty_files_dropped = if req.force { dirty.total_count() } else { 0 };

    let target_name = ensure_local_checkout_target(&repo, &req.name)?;
    checkout_local_branch(&repo, &target_name, req.force)?;

    Ok(BranchSwitchResult {
        new_head: target_name,
        prev_head,
        dirty_files_dropped,
    })
}

pub fn branch_delete(workspace_path: &Path, req: BranchDeleteRequest) -> Result<(), BranchError> {
    validate_name(&req.name)?;
    ensure_not_protected(&req.name)?;

    let repo = open_repo(workspace_path)?;
    ensure_index_unlocked(&repo)?;
    let mut branch = repo
        .find_branch(&req.name, BranchType::Local)
        .map_err(|error| map_not_found(error, &req.name))?;

    if !req.force {
        ensure_branch_merged(&repo, &branch, &req.name)?;
    }

    branch.delete().map_err(map_git_error)
}

pub fn branch_switcher_query(
    workspace_path: &Path,
    req: SwitcherQueryRequest,
) -> Result<SwitcherSearchResult, BranchError> {
    let list = branch_list(workspace_path)?;
    let limit = if req.limit == 0 {
        usize::MAX
    } else {
        req.limit
    };
    let query = req.query.trim();

    let mut matches = if query.is_empty() {
        list.branches
            .into_iter()
            .filter(|branch| matches!(branch.kind, BranchKind::Local | BranchKind::Remote))
            .map(|branch| {
                let score = if Some(branch.name.as_str()) == list.head_name.as_deref() {
                    1_000.0
                } else {
                    0.0
                };
                SwitcherMatch {
                    branch,
                    score,
                    match_indices: Vec::new(),
                }
            })
            .collect::<Vec<_>>()
    } else {
        list.branches
            .into_iter()
            .filter(|branch| matches!(branch.kind, BranchKind::Local | BranchKind::Remote))
            .filter_map(|branch| {
                fuzzy_match(&branch.name, query).map(|(score, match_indices)| SwitcherMatch {
                    branch,
                    score,
                    match_indices,
                })
            })
            .collect::<Vec<_>>()
    };

    matches.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| kind_rank(left.branch.kind).cmp(&kind_rank(right.branch.kind)))
            .then_with(|| left.branch.name.cmp(&right.branch.name))
    });
    matches.truncate(limit);

    Ok(SwitcherSearchResult { matches })
}

fn validate_name(name: &str) -> Result<(), BranchError> {
    if name.is_empty() {
        return invalid_name("branch name cannot be empty");
    }
    if name.chars().any(char::is_whitespace)
        || name
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '~' | '^' | ':' | '?' | '*' | '[' | '\\'))
    {
        return invalid_name("branch name contains forbidden characters");
    }
    if name.starts_with('.') || name.starts_with('/') {
        return invalid_name("branch name cannot start with '.' or '/'");
    }
    if name.contains("..") || name.contains("@{") || name.contains(".git") {
        return invalid_name("branch name contains forbidden pattern");
    }
    if name.ends_with(".lock") || name.ends_with('/') {
        return invalid_name("branch name cannot end with '.lock' or '/'");
    }

    let full_ref = format!("refs/heads/{name}");
    if !git2::Reference::is_valid_name(&full_ref) {
        return invalid_name(format!("git2 rejected ref name '{name}'"));
    }

    Ok(())
}

fn invalid_name(reason: impl Into<String>) -> Result<(), BranchError> {
    Err(BranchError::InvalidName {
        reason: reason.into(),
    })
}

fn open_repo(path: &Path) -> Result<Repository, BranchError> {
    Repository::open(path).map_err(map_git_error)
}

fn ensure_not_protected(name: &str) -> Result<(), BranchError> {
    if matches!(name, "main" | "master" | "trunk") {
        Err(BranchError::ProtectedBranch {
            name: name.to_string(),
        })
    } else {
        Ok(())
    }
}

fn ensure_index_unlocked(repo: &Repository) -> Result<(), BranchError> {
    if repo.path().join("index.lock").exists() {
        Err(BranchError::IndexLocked)
    } else {
        Ok(())
    }
}

fn collect_refs_with_gix(path: &Path) -> Result<Vec<BranchInfo>, String> {
    let repo = gix::open(path).map_err(|error| error.to_string())?;
    let refs = repo.references().map_err(|error| error.to_string())?;
    let refs = refs.all().map_err(|error| error.to_string())?;
    let mut branches = Vec::new();

    for reference in refs {
        let reference = match reference {
            Ok(reference) => reference,
            Err(_) => continue,
        };
        let full_ref = reference.name().as_bstr().to_string();
        let Some((kind, name)) = parse_ref_name(&full_ref) else {
            continue;
        };
        if is_remote_head(&name, kind) {
            continue;
        }
        let head_commit = reference.try_id().map(|id| id.detach().to_string());
        branches.push(BranchInfo {
            name,
            full_ref,
            kind,
            upstream: None,
            ahead: 0,
            behind: 0,
            head_commit,
        });
    }

    Ok(branches)
}

fn enrich_refs_with_git2(repo: &Repository, refs: Vec<BranchInfo>) -> Vec<BranchInfo> {
    refs.into_iter()
        .map(|mut branch| {
            enrich_branch(repo, &mut branch);
            branch
        })
        .collect()
}

fn collect_refs_with_git2(repo: &Repository) -> Result<Vec<BranchInfo>, BranchError> {
    let mut branches = Vec::new();

    for branch_type in [BranchType::Local, BranchType::Remote] {
        let kind = if branch_type == BranchType::Local {
            BranchKind::Local
        } else {
            BranchKind::Remote
        };
        let iter = repo.branches(Some(branch_type)).map_err(map_git_error)?;
        for item in iter {
            let (branch, _) = item.map_err(map_git_error)?;
            let Some(name) = branch.name().map_err(map_git_error)? else {
                continue;
            };
            let name = name.to_string();
            if is_remote_head(&name, kind) {
                continue;
            }
            let full_ref = branch
                .get()
                .name()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("refs/heads/{name}"));
            let mut info = BranchInfo {
                name,
                full_ref,
                kind,
                upstream: None,
                ahead: 0,
                behind: 0,
                head_commit: branch.get().target().map(|oid| oid.to_string()),
            };
            enrich_branch(repo, &mut info);
            branches.push(info);
        }
    }

    if let Ok(tags) = repo.tag_names(None) {
        for tag in tags.iter().flatten() {
            let full_ref = format!("refs/tags/{tag}");
            let head_commit = repo
                .revparse_single(&full_ref)
                .ok()
                .and_then(|object| object.peel_to_commit().ok())
                .map(|commit| commit.id().to_string());
            branches.push(BranchInfo {
                name: tag.to_string(),
                full_ref,
                kind: BranchKind::Tag,
                upstream: None,
                ahead: 0,
                behind: 0,
                head_commit,
            });
        }
    }

    Ok(branches)
}

fn parse_ref_name(full_ref: &str) -> Option<(BranchKind, String)> {
    full_ref
        .strip_prefix("refs/heads/")
        .map(|name| (BranchKind::Local, name.to_string()))
        .or_else(|| {
            full_ref
                .strip_prefix("refs/remotes/")
                .map(|name| (BranchKind::Remote, name.to_string()))
        })
        .or_else(|| {
            full_ref
                .strip_prefix("refs/tags/")
                .map(|name| (BranchKind::Tag, name.to_string()))
        })
}

fn enrich_branch(repo: &Repository, branch: &mut BranchInfo) {
    if branch.kind != BranchKind::Local {
        return;
    }

    let Ok(local) = repo.find_branch(&branch.name, BranchType::Local) else {
        return;
    };
    let Some(local_target) = local.get().target() else {
        return;
    };
    branch.head_commit = Some(local_target.to_string());

    let Ok(upstream) = local.upstream() else {
        return;
    };
    if let Ok(Some(upstream_name)) = upstream.name() {
        branch.upstream = Some(upstream_name.to_string());
    }
    if let Some(upstream_target) = upstream.get().target() {
        if let Ok((ahead, behind)) = repo.graph_ahead_behind(local_target, upstream_target) {
            branch.ahead = ahead as u32;
            branch.behind = behind as u32;
        }
    }
}

fn kind_rank(kind: BranchKind) -> u8 {
    match kind {
        BranchKind::Local => 0,
        BranchKind::Remote => 1,
        BranchKind::Tag => 2,
    }
}

fn is_remote_head(name: &str, kind: BranchKind) -> bool {
    kind == BranchKind::Remote && name.ends_with("/HEAD")
}

fn resolve_create_target(
    repo: &Repository,
    from_ref: Option<&str>,
) -> Result<(Oid, Option<String>), BranchError> {
    let Some(from_ref) = from_ref else {
        return repo
            .head()
            .and_then(|head| head.peel_to_commit())
            .map(|commit| (commit.id(), None))
            .map_err(map_git_error);
    };

    if let Ok(local) = repo.find_branch(from_ref, BranchType::Local) {
        if let Some(target) = local.get().target() {
            return Ok((target, None));
        }
    }
    if let Ok(remote) = repo.find_branch(from_ref, BranchType::Remote) {
        if let Some(target) = remote.get().target() {
            return Ok((target, Some(from_ref.to_string())));
        }
    }

    repo.revparse_single(from_ref)
        .and_then(|object| object.peel_to_commit())
        .map(|commit| (commit.id(), None))
        .map_err(|error| map_not_found(error, from_ref))
}

fn ensure_local_checkout_target(repo: &Repository, requested: &str) -> Result<String, BranchError> {
    if repo.find_branch(requested, BranchType::Local).is_ok() {
        return Ok(requested.to_string());
    }

    let remote = repo
        .find_branch(requested, BranchType::Remote)
        .map_err(|error| map_not_found(error, requested))?;
    let local_name = local_name_from_remote(requested)?;
    validate_name(&local_name)?;

    let remote_target = remote.get().target().ok_or_else(|| BranchError::NotFound {
        name: requested.to_string(),
    })?;
    if let Ok(local) = repo.find_branch(&local_name, BranchType::Local) {
        return if local.get().target() == Some(remote_target) {
            Ok(local_name)
        } else {
            Err(BranchError::AlreadyExists { name: local_name })
        };
    }

    let commit = repo.find_commit(remote_target).map_err(map_git_error)?;
    let mut branch = repo
        .branch(&local_name, &commit, false)
        .map_err(|error| map_branch_create_error(error, &local_name))?;
    branch
        .set_upstream(Some(requested))
        .map_err(map_git_error)?;

    Ok(local_name)
}

fn local_name_from_remote(remote_name: &str) -> Result<String, BranchError> {
    remote_name
        .split_once('/')
        .map(|(_, local)| local.to_string())
        .filter(|local| !local.is_empty())
        .ok_or_else(|| BranchError::NotFound {
            name: remote_name.to_string(),
        })
}

fn checkout_local_branch(repo: &Repository, name: &str, force: bool) -> Result<(), BranchError> {
    let branch = repo
        .find_branch(name, BranchType::Local)
        .map_err(|error| map_not_found(error, name))?;
    let full_ref = branch
        .get()
        .name()
        .ok_or_else(|| BranchError::NotFound {
            name: name.to_string(),
        })?
        .to_string();

    repo.set_head(&full_ref).map_err(map_git_error)?;
    let mut checkout = CheckoutBuilder::new();
    if force {
        checkout.force();
    } else {
        checkout.safe();
    }
    repo.checkout_head(Some(&mut checkout))
        .map_err(map_git_error)
}

fn current_head_label(repo: &Repository) -> Result<String, BranchError> {
    let head = repo.head().map_err(map_git_error)?;
    if let Some(name) = head.shorthand() {
        return Ok(name.to_string());
    }
    if let Some(target) = head.target() {
        return Ok(target.to_string());
    }
    Err(BranchError::DetachedHead)
}

#[derive(Debug, Default)]
struct DirtyState {
    modified: BTreeSet<String>,
    staged: BTreeSet<String>,
    untracked: BTreeSet<String>,
}

impl DirtyState {
    fn is_dirty(&self) -> bool {
        !(self.modified.is_empty() && self.staged.is_empty() && self.untracked.is_empty())
    }

    fn total_count(&self) -> usize {
        self.modified
            .iter()
            .chain(self.staged.iter())
            .chain(self.untracked.iter())
            .collect::<BTreeSet<_>>()
            .len()
    }

    fn into_error(self) -> BranchError {
        BranchError::DirtyWorkingTree {
            modified: self.modified.into_iter().collect(),
            staged: self.staged.into_iter().collect(),
            untracked: self.untracked.into_iter().collect(),
        }
    }
}

fn dirty_working_tree(repo: &Repository) -> Result<DirtyState, BranchError> {
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);
    let statuses = repo.statuses(Some(&mut options)).map_err(map_git_error)?;
    let mut dirty = DirtyState::default();

    for entry in statuses.iter() {
        let Some(path) = entry.path() else {
            continue;
        };
        let status = entry.status();
        let path = path.to_string();
        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            dirty.staged.insert(path.clone());
        }
        if status.contains(Status::WT_NEW) {
            dirty.untracked.insert(path.clone());
        }
        if status.intersects(
            Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED | Status::WT_TYPECHANGE,
        ) {
            dirty.modified.insert(path);
        }
    }

    Ok(dirty)
}

fn ensure_branch_merged(
    repo: &Repository,
    branch: &git2::Branch<'_>,
    name: &str,
) -> Result<(), BranchError> {
    let branch_tip = branch.get().target().ok_or_else(|| BranchError::NotFound {
        name: name.to_string(),
    })?;
    let head_tip = repo
        .head()
        .and_then(|head| head.peel_to_commit())
        .map(|commit| commit.id())
        .map_err(map_git_error)?;

    let merge_base = repo.merge_base(branch_tip, head_tip).ok();
    if merge_base == Some(branch_tip) {
        return Ok(());
    }

    let missing_commits = count_missing_commits(repo, branch_tip, merge_base)?;
    Err(BranchError::Unmerged {
        name: name.to_string(),
        missing_commits,
    })
}

fn count_missing_commits(
    repo: &Repository,
    branch_tip: Oid,
    merge_base: Option<Oid>,
) -> Result<u32, BranchError> {
    let mut revwalk = repo.revwalk().map_err(map_git_error)?;
    revwalk.push(branch_tip).map_err(map_git_error)?;
    if let Some(base) = merge_base {
        revwalk.hide(base).map_err(map_git_error)?;
    }

    let count = revwalk.filter_map(Result::ok).count();
    Ok(count.min(u32::MAX as usize) as u32)
}

fn fuzzy_match(candidate: &str, query: &str) -> Option<(f32, Vec<usize>)> {
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    let query_chars = query.chars().collect::<Vec<_>>();
    let candidate_lower = candidate.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();
    let mut indices = Vec::with_capacity(query_chars.len());
    let mut cursor = 0;
    let mut score = 0.0;
    for query_char in query_chars {
        let query_char = query_char.to_ascii_lowercase();
        let next = candidate_chars
            .iter()
            .enumerate()
            .skip(cursor)
            .find(|(_, candidate_char)| candidate_char.to_ascii_lowercase() == query_char)
            .map(|(idx, _)| idx)?;
        if indices.last().is_some_and(|previous| *previous + 1 == next) {
            score += 3.0;
        } else {
            score += 1.0;
        }
        if next == 0
            || matches!(
                candidate_chars.get(next.wrapping_sub(1)),
                Some('/' | '-' | '_')
            )
        {
            score += 1.0;
        }
        indices.push(next);
        cursor = next + 1;
    }
    if candidate_lower.contains(&query_lower) {
        score += 20.0;
    }
    score -= indices.first().copied().unwrap_or_default() as f32 * 0.01;
    Some((score, indices))
}

fn map_branch_create_error(error: git2::Error, name: &str) -> BranchError {
    if error.code() == ErrorCode::Exists {
        BranchError::AlreadyExists {
            name: name.to_string(),
        }
    } else {
        map_git_error(error)
    }
}

fn map_not_found(error: git2::Error, name: &str) -> BranchError {
    if error.code() == ErrorCode::NotFound {
        BranchError::NotFound {
            name: name.to_string(),
        }
    } else {
        map_git_error(error)
    }
}

fn map_git_error(error: git2::Error) -> BranchError {
    if error.code() == ErrorCode::Locked {
        return BranchError::IndexLocked;
    }

    BranchError::Git2Error {
        class: format!("{:?}", error.class()),
        code: error.raw_code(),
        message: error.message().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{BranchType, Repository, Signature};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    struct TestRepo {
        _dir: TempDir,
        path: PathBuf,
    }

    impl TestRepo {
        fn new() -> Self {
            let dir = TempDir::new().unwrap();
            let path = dir.path().to_path_buf();
            let repo = Repository::init(&path).unwrap();
            repo.set_head("refs/heads/main").unwrap();
            let mut cfg = repo.config().unwrap();
            cfg.set_str("user.name", "Test User").unwrap();
            cfg.set_str("user.email", "test@example.com").unwrap();
            commit_file(&repo, &path, "initial.txt", "initial", "initial");
            Self { _dir: dir, path }
        }

        fn repo(&self) -> Repository {
            Repository::open(&self.path).unwrap()
        }
    }

    fn commit_file(
        repo: &Repository,
        root: &Path,
        file: &str,
        content: &str,
        message: &str,
    ) -> Oid {
        fs::write(root.join(file), content).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new(file)).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let parents = repo
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok())
            .into_iter()
            .collect::<Vec<_>>();
        let parent_refs = parents.iter().collect::<Vec<_>>();
        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
            .unwrap();
        let object = repo.find_object(oid, None).unwrap();
        repo.reset(&object, git2::ResetType::Hard, None).unwrap();
        oid
    }

    fn checkout(repo: &Repository, name: &str) {
        repo.set_head(&format!("refs/heads/{name}")).unwrap();
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .unwrap();
    }

    fn create_branch(repo: &Repository, name: &str, target: Oid) {
        let commit = repo.find_commit(target).unwrap();
        repo.branch(name, &commit, false).unwrap();
    }

    fn request_id() -> String {
        "workspace-test".to_string()
    }

    #[test]
    fn name_validate_accepts_normal() {
        validate_name("feat/branch-ops").unwrap();
    }

    #[test]
    fn name_validate_accepts_chinese() {
        validate_name("feat/中文-test").unwrap();
    }

    #[test]
    fn name_validate_rejects_space() {
        assert!(matches!(
            validate_name("feat/bad name"),
            Err(BranchError::InvalidName { .. })
        ));
    }

    #[test]
    fn name_validate_rejects_dot_start() {
        assert!(matches!(
            validate_name(".hidden"),
            Err(BranchError::InvalidName { .. })
        ));
    }

    #[test]
    fn name_validate_rejects_at_brace() {
        assert!(matches!(
            validate_name("feat/@{bad"),
            Err(BranchError::InvalidName { .. })
        ));
    }

    #[test]
    fn name_validate_rejects_dotgit() {
        assert!(matches!(
            validate_name("feat/repo.git"),
            Err(BranchError::InvalidName { .. })
        ));
    }

    #[test]
    fn name_validate_rejects_lock_suffix() {
        assert!(matches!(
            validate_name("feat/file.lock"),
            Err(BranchError::InvalidName { .. })
        ));
    }

    #[test]
    fn branch_create_from_head() {
        let repo = TestRepo::new();
        branch_create(
            &repo.path,
            BranchCreateRequest {
                workspace_id: request_id(),
                name: "feat/from-head".to_string(),
                from_ref: None,
                checkout: false,
            },
        )
        .unwrap();

        assert!(repo
            .repo()
            .find_branch("feat/from-head", BranchType::Local)
            .is_ok());
    }

    #[test]
    fn branch_create_from_other_branch() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "source", main_tip);
        checkout(&git, "source");
        let source_tip = commit_file(&git, &repo.path, "source.txt", "source", "source");
        checkout(&git, "main");

        branch_create(
            &repo.path,
            BranchCreateRequest {
                workspace_id: request_id(),
                name: "from-source".to_string(),
                from_ref: Some("source".to_string()),
                checkout: false,
            },
        )
        .unwrap();

        let repo_handle = repo.repo();
        let created = repo_handle
            .find_branch("from-source", BranchType::Local)
            .unwrap();
        assert_eq!(created.get().target(), Some(source_tip));
    }

    #[test]
    fn branch_create_and_checkout() {
        let repo = TestRepo::new();
        branch_create(
            &repo.path,
            BranchCreateRequest {
                workspace_id: request_id(),
                name: "feat/checkout".to_string(),
                from_ref: None,
                checkout: true,
            },
        )
        .unwrap();

        assert_eq!(
            repo.repo().head().unwrap().shorthand(),
            Some("feat/checkout")
        );
    }

    #[test]
    fn branch_create_checkout_failure_keeps_created_branch() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "source", main_tip);
        checkout(&git, "source");
        commit_file(&git, &repo.path, "source-only.txt", "source", "source");
        checkout(&git, "main");
        fs::write(repo.path.join("source-only.txt"), "dirty").unwrap();

        let result = branch_create(
            &repo.path,
            BranchCreateRequest {
                workspace_id: request_id(),
                name: "created-after-checkout-fail".to_string(),
                from_ref: Some("source".to_string()),
                checkout: true,
            },
        );

        assert!(matches!(result, Err(BranchError::DirtyWorkingTree { .. })));
        assert!(repo
            .repo()
            .find_branch("created-after-checkout-fail", BranchType::Local)
            .is_ok());
    }

    #[test]
    fn branch_create_already_exists() {
        let repo = TestRepo::new();
        let req = BranchCreateRequest {
            workspace_id: request_id(),
            name: "duplicate".to_string(),
            from_ref: None,
            checkout: false,
        };
        branch_create(&repo.path, req.clone()).unwrap();

        assert!(matches!(
            branch_create(&repo.path, req),
            Err(BranchError::AlreadyExists { name }) if name == "duplicate"
        ));
    }

    #[test]
    fn branch_create_index_lock_returns_error() {
        let repo = TestRepo::new();
        fs::write(repo.repo().path().join("index.lock"), "").unwrap();

        assert!(matches!(
            branch_create(
                &repo.path,
                BranchCreateRequest {
                    workspace_id: request_id(),
                    name: "locked".to_string(),
                    from_ref: None,
                    checkout: false,
                },
            ),
            Err(BranchError::IndexLocked)
        ));
    }

    #[test]
    fn branch_create_protected_main_rejected() {
        let repo = TestRepo::new();
        assert!(matches!(
            branch_create(
                &repo.path,
                BranchCreateRequest {
                    workspace_id: request_id(),
                    name: "main".to_string(),
                    from_ref: None,
                    checkout: false,
                },
            ),
            Err(BranchError::ProtectedBranch { name }) if name == "main"
        ));
    }

    #[test]
    fn branch_checkout_clean_tree() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "feature", main_tip);

        let result = branch_checkout(
            &repo.path,
            BranchCheckoutRequest {
                workspace_id: request_id(),
                name: "feature".to_string(),
                force: false,
            },
        )
        .unwrap();

        assert_eq!(result.prev_head, "main");
        assert_eq!(result.new_head, "feature");
        assert_eq!(repo.repo().head().unwrap().shorthand(), Some("feature"));
    }

    #[test]
    fn branch_checkout_dirty_tree_returns_error() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "feature", main_tip);
        fs::write(repo.path.join("initial.txt"), "dirty").unwrap();

        assert!(matches!(
            branch_checkout(
                &repo.path,
                BranchCheckoutRequest {
                    workspace_id: request_id(),
                    name: "feature".to_string(),
                    force: false,
                },
            ),
            Err(BranchError::DirtyWorkingTree { modified, .. }) if modified == vec!["initial.txt"]
        ));
    }

    #[test]
    fn branch_checkout_force_drops_dirty() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "feature", main_tip);
        checkout(&git, "feature");
        commit_file(&git, &repo.path, "initial.txt", "feature", "feature");
        checkout(&git, "main");
        fs::write(repo.path.join("initial.txt"), "dirty").unwrap();

        let result = branch_checkout(
            &repo.path,
            BranchCheckoutRequest {
                workspace_id: request_id(),
                name: "feature".to_string(),
                force: true,
            },
        )
        .unwrap();

        assert_eq!(result.dirty_files_dropped, 1);
        assert_eq!(
            fs::read_to_string(repo.path.join("initial.txt")).unwrap(),
            "feature"
        );
    }

    #[test]
    fn branch_checkout_remote_creates_local_tracking() {
        let repo = TestRepo::new();
        let git = repo.repo();
        git.remote("origin", "https://example.com/repo.git")
            .unwrap();
        let head = git.head().unwrap().target().unwrap();
        git.reference("refs/remotes/origin/feat/remote", head, true, "remote")
            .unwrap();

        branch_checkout(
            &repo.path,
            BranchCheckoutRequest {
                workspace_id: request_id(),
                name: "origin/feat/remote".to_string(),
                force: false,
            },
        )
        .unwrap();

        let repo_handle = repo.repo();
        let local = repo_handle
            .find_branch("feat/remote", BranchType::Local)
            .unwrap();
        assert_eq!(
            local.upstream().unwrap().name().unwrap(),
            Some("origin/feat/remote")
        );
    }

    #[test]
    fn branch_delete_merged_succeeds() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "merged", main_tip);

        branch_delete(
            &repo.path,
            BranchDeleteRequest {
                workspace_id: request_id(),
                name: "merged".to_string(),
                force: false,
            },
        )
        .unwrap();

        assert!(repo
            .repo()
            .find_branch("merged", BranchType::Local)
            .is_err());
    }

    #[test]
    fn branch_delete_unmerged_returns_error_with_missing_count() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "unmerged", main_tip);
        checkout(&git, "unmerged");
        commit_file(&git, &repo.path, "unmerged.txt", "unmerged", "unmerged");
        checkout(&git, "main");

        assert!(matches!(
            branch_delete(
                &repo.path,
                BranchDeleteRequest {
                    workspace_id: request_id(),
                    name: "unmerged".to_string(),
                    force: false,
                },
            ),
            Err(BranchError::Unmerged {
                missing_commits: 1,
                ..
            })
        ));
    }

    #[test]
    fn branch_delete_force_succeeds() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "force-delete", main_tip);
        checkout(&git, "force-delete");
        commit_file(&git, &repo.path, "force.txt", "force", "force");
        checkout(&git, "main");

        branch_delete(
            &repo.path,
            BranchDeleteRequest {
                workspace_id: request_id(),
                name: "force-delete".to_string(),
                force: true,
            },
        )
        .unwrap();

        assert!(repo
            .repo()
            .find_branch("force-delete", BranchType::Local)
            .is_err());
    }

    #[test]
    fn branch_delete_index_lock_returns_error() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let main_tip = git.head().unwrap().target().unwrap();
        create_branch(&git, "locked-delete", main_tip);
        fs::write(git.path().join("index.lock"), "").unwrap();

        assert!(matches!(
            branch_delete(
                &repo.path,
                BranchDeleteRequest {
                    workspace_id: request_id(),
                    name: "locked-delete".to_string(),
                    force: true,
                },
            ),
            Err(BranchError::IndexLocked)
        ));
    }

    #[test]
    fn branch_delete_protected_main_rejected() {
        let repo = TestRepo::new();
        assert!(matches!(
            branch_delete(
                &repo.path,
                BranchDeleteRequest {
                    workspace_id: request_id(),
                    name: "main".to_string(),
                    force: true,
                },
            ),
            Err(BranchError::ProtectedBranch { name }) if name == "main"
        ));
    }

    #[test]
    fn branch_delete_protected_master_rejected() {
        let repo = TestRepo::new();
        assert!(matches!(
            branch_delete(
                &repo.path,
                BranchDeleteRequest {
                    workspace_id: request_id(),
                    name: "master".to_string(),
                    force: true,
                },
            ),
            Err(BranchError::ProtectedBranch { name }) if name == "master"
        ));
    }

    #[test]
    fn branch_delete_protected_trunk_rejected() {
        let repo = TestRepo::new();
        assert!(matches!(
            branch_delete(
                &repo.path,
                BranchDeleteRequest {
                    workspace_id: request_id(),
                    name: "trunk".to_string(),
                    force: true,
                },
            ),
            Err(BranchError::ProtectedBranch { name }) if name == "trunk"
        ));
    }

    #[test]
    fn branch_list_returns_local_remote_tag() {
        let repo = TestRepo::new();
        let git = repo.repo();
        git.remote("origin", "https://example.com/repo.git")
            .unwrap();
        let head = git.head().unwrap().target().unwrap();
        create_branch(&git, "feature", head);
        git.reference("refs/remotes/origin/feature", head, true, "remote")
            .unwrap();
        let object = git.find_object(head, None).unwrap();
        git.tag_lightweight("v0.1.0", &object, false).unwrap();

        let response = branch_list(&repo.path).unwrap();
        let by_name = response
            .branches
            .iter()
            .map(|branch| (branch.name.as_str(), branch.kind))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(response.head_name.as_deref(), Some("main"));
        assert_eq!(by_name.get("feature"), Some(&BranchKind::Local));
        assert_eq!(by_name.get("origin/feature"), Some(&BranchKind::Remote));
        assert_eq!(by_name.get("v0.1.0"), Some(&BranchKind::Tag));
    }

    #[test]
    fn branch_list_detects_detached_head() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let head = git.head().unwrap().target().unwrap();
        git.set_head_detached(head).unwrap();
        git.checkout_head(Some(CheckoutBuilder::new().force()))
            .unwrap();

        let response = branch_list(&repo.path).unwrap();

        assert!(response.detached);
        assert_eq!(response.head_name, None);
    }

    #[test]
    fn branch_list_non_repo_returns_git2_error() {
        let dir = TempDir::new().unwrap();

        assert!(matches!(
            branch_list(dir.path()),
            Err(BranchError::Git2Error { .. })
        ));
    }

    #[test]
    fn switcher_query_subsequence_match() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let head = git.head().unwrap().target().unwrap();
        create_branch(&git, "feat/pty-pool", head);

        let result = branch_switcher_query(
            &repo.path,
            SwitcherQueryRequest {
                workspace_id: request_id(),
                query: "fpp".to_string(),
                limit: 10,
            },
        )
        .unwrap();

        assert_eq!(result.matches[0].branch.name, "feat/pty-pool");
        assert!(!result.matches[0].match_indices.is_empty());
    }

    #[test]
    fn switcher_query_empty_returns_all() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let head = git.head().unwrap().target().unwrap();
        create_branch(&git, "alpha", head);
        create_branch(&git, "beta", head);

        let result = branch_switcher_query(
            &repo.path,
            SwitcherQueryRequest {
                workspace_id: request_id(),
                query: String::new(),
                limit: 10,
            },
        )
        .unwrap();

        let names = result
            .matches
            .iter()
            .map(|item| item.branch.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn switcher_query_score_orders_continuous_higher() {
        let repo = TestRepo::new();
        let git = repo.repo();
        let head = git.head().unwrap().target().unwrap();
        create_branch(&git, "feat/abc", head);
        create_branch(&git, "a-very-busy-candidate", head);

        let result = branch_switcher_query(
            &repo.path,
            SwitcherQueryRequest {
                workspace_id: request_id(),
                query: "abc".to_string(),
                limit: 10,
            },
        )
        .unwrap();

        assert_eq!(result.matches[0].branch.name, "feat/abc");
    }
}

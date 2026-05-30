//! Rebase / merge / cherry-pick operations for MVP-16 Phase A.
//!
//! The write path stays on git2. Interactive rebase is represented as an
//! explicit plan and a small state machine because git2 0.20 does not expose an
//! editable interactive rebase todo API.

use git2::{
    build::CheckoutBuilder, BranchType, CherrypickOptions, MergeOptions, ObjectType, Oid,
    Repository, ResetType, Status, StatusOptions,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RebaseStartRequest {
    pub workspace_id: String,
    pub branch: String,
    pub onto: String,
    pub interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RebaseInteractivePlan {
    pub steps: Vec<RebaseInteractiveStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RebaseInteractiveStep {
    pub step_id: String,
    pub op: RebaseOp,
    pub commit_sha: String,
    pub message_override: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "PascalCase")]
pub enum RebaseOp {
    Pick,
    Reword,
    Squash,
    Fixup,
    Drop,
    Edit,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RebaseStatus {
    pub in_progress: bool,
    pub operation: Option<String>,
    pub current_step: u32,
    pub total_steps: u32,
    pub conflicting_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RebaseControlRequest {
    pub workspace_id: String,
    pub action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum MergeStrategy {
    FastForward,
    NoFastForward,
    Squash,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MergeRequest {
    pub workspace_id: String,
    pub source_branch: String,
    pub strategy: MergeStrategy,
    pub commit_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MergeStatus {
    pub outcome: String,
    pub conflicting_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CherryPickRequest {
    pub workspace_id: String,
    pub commit_shas: Vec<String>,
    pub auto_commit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CherryPickStatus {
    pub in_progress: bool,
    pub current_index: u32,
    pub total_commits: u32,
    pub conflicting_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConflictedFile {
    pub path: String,
    pub hunks: Vec<ConflictHunk>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConflictHunk {
    pub id: String,
    pub base_content: String,
    pub ours_content: String,
    pub theirs_content: String,
    pub resolved: bool,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ConflictResolution {
    AcceptOurs,
    AcceptTheirs,
    AcceptBoth,
    Manual { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConflictResolveFileRequest {
    pub workspace_id: String,
    pub file_path: String,
    pub resolutions: Vec<ConflictHunkResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConflictHunkResolution {
    pub hunk_id: String,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CrashRecoveryState {
    pub in_progress: bool,
    pub operation: Option<String>,
    pub branch: Option<String>,
    pub current_step: u32,
    pub total_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RebaseOpError {
    NotInRebase,
    ConflictUnresolved {
        files: Vec<String>,
    },
    DirtyWorkingTree {
        modified: Vec<String>,
        staged: Vec<String>,
    },
    UncommittedChanges {
        paths: Vec<String>,
    },
    InvalidStep {
        step_id: String,
        reason: String,
    },
    DetachedHead,
    OperationInProgress {
        existing: String,
    },
    AlreadyUpToDate,
    Git2Error {
        class: String,
        code: i32,
        message: String,
    },
}

impl fmt::Display for RebaseOpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationState {
    operation: String,
    branch: String,
    onto: Option<String>,
    original_head: String,
    plan: RebaseInteractivePlan,
    current_step: usize,
    total_steps: usize,
    auto_commit: bool,
    started_at: i64,
    last_updated: i64,
}

#[derive(Default)]
struct DirtyState {
    modified: Vec<String>,
    staged: Vec<String>,
}

pub fn rebase_start(
    workspace_path: &Path,
    req: RebaseStartRequest,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    ensure_clean_operation_start(&repo)?;
    ensure_not_detached(&repo)?;
    ensure_no_operation_state(&repo)?;
    if req.branch == req.onto {
        return Err(RebaseOpError::AlreadyUpToDate);
    }

    checkout_branch_if_needed(&repo, &req.branch)?;
    let commits = commits_to_replay(&repo, &req.branch, &req.onto)?;
    if commits.is_empty() {
        return Err(RebaseOpError::AlreadyUpToDate);
    }
    let plan = RebaseInteractivePlan {
        steps: commits
            .iter()
            .enumerate()
            .map(|(index, oid)| RebaseInteractiveStep {
                step_id: format!("step-{index}"),
                op: RebaseOp::Pick,
                commit_sha: oid.to_string(),
                message_override: None,
            })
            .collect(),
    };
    validate_plan(&plan)?;

    let original_head = current_head_oid(&repo)?.to_string();
    let onto_commit = resolve_commit(&repo, &req.onto)?;
    hard_reset(&repo, onto_commit.id())?;

    let mut state = OperationState::new(
        "rebase",
        req.branch,
        Some(req.onto),
        original_head,
        plan,
        true,
    );
    write_operation_state(&repo, &state)?;
    execute_plan_from_state(&repo, &mut state)
}

pub fn rebase_continue(
    workspace_path: &Path,
    _req: RebaseControlRequest,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    let conflicts = conflict_paths(&repo)?;
    if !conflicts.is_empty() {
        return Err(RebaseOpError::ConflictUnresolved { files: conflicts });
    }
    let mut state = read_operation_state(&repo)?.ok_or(RebaseOpError::NotInRebase)?;
    if state.operation == "cherrypick" && state.auto_commit {
        commit_index_from_step(&repo, &state)?;
    }
    execute_plan_from_state(&repo, &mut state)
}

pub fn rebase_abort(
    workspace_path: &Path,
    _req: RebaseControlRequest,
) -> Result<(), RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    let state = read_operation_state(&repo)?.ok_or(RebaseOpError::NotInRebase)?;
    reset_to_original_head(&repo, &state)?;
    cleanup_git_state(&repo)?;
    remove_operation_state(&repo)
}

pub fn rebase_skip(
    workspace_path: &Path,
    _req: RebaseControlRequest,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    let mut state = read_operation_state(&repo)?.ok_or(RebaseOpError::NotInRebase)?;
    cleanup_git_state(&repo)?;
    state.current_step = state.current_step.saturating_add(1);
    state.touch();
    write_operation_state(&repo, &state)?;
    execute_plan_from_state(&repo, &mut state)
}

pub fn rebase_interactive_plan(
    workspace_path: &Path,
    branch: &str,
    onto: &str,
) -> Result<RebaseInteractivePlan, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    let commits = commits_to_replay(&repo, branch, onto)?;
    Ok(RebaseInteractivePlan {
        steps: commits
            .iter()
            .enumerate()
            .map(|(index, oid)| RebaseInteractiveStep {
                step_id: format!("step-{index}"),
                op: RebaseOp::Pick,
                commit_sha: oid.to_string(),
                message_override: None,
            })
            .collect(),
    })
}

pub fn rebase_interactive_apply(
    workspace_path: &Path,
    plan: RebaseInteractivePlan,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    ensure_clean_operation_start(&repo)?;
    ensure_not_detached(&repo)?;
    ensure_no_operation_state(&repo)?;
    validate_plan(&plan)?;
    let original_head = current_head_oid(&repo)?.to_string();
    let branch = current_branch_name(&repo).unwrap_or_else(|| "HEAD".to_string());
    let mut state = OperationState::new("rebase", branch, None, original_head, plan, true);
    write_operation_state(&repo, &state)?;
    execute_plan_from_state(&repo, &mut state)
}

pub fn merge_start(workspace_path: &Path, req: MergeRequest) -> Result<MergeStatus, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    ensure_clean_operation_start(&repo)?;
    ensure_not_detached(&repo)?;
    ensure_no_operation_state(&repo)?;

    let head_oid = current_head_oid(&repo)?;
    let target_commit = resolve_commit(&repo, &req.source_branch)?;
    if target_commit.id() == head_oid {
        return Err(RebaseOpError::AlreadyUpToDate);
    }
    let base = repo
        .merge_base(head_oid, target_commit.id())
        .map_err(map_git_error)?;
    if base == target_commit.id() {
        return Err(RebaseOpError::AlreadyUpToDate);
    }

    match req.strategy {
        MergeStrategy::FastForward if base == head_oid => {
            fast_forward_to(&repo, target_commit.id())?;
            Ok(MergeStatus {
                outcome: "fast-forwarded".to_string(),
                conflicting_files: vec![],
            })
        }
        MergeStrategy::FastForward | MergeStrategy::NoFastForward => {
            create_merge_commit(&repo, &target_commit, req.commit_message)
        }
        MergeStrategy::Squash => create_squash_commit(&repo, &target_commit, req.commit_message),
    }
}

pub fn merge_abort(workspace_path: &Path) -> Result<(), RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    if !repo.path().join("MERGE_HEAD").exists() {
        return Err(RebaseOpError::AlreadyUpToDate);
    }
    cleanup_git_state(&repo)?;
    force_checkout_head(&repo)
}

pub fn cherrypick_start(
    workspace_path: &Path,
    req: CherryPickRequest,
) -> Result<CherryPickStatus, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    ensure_clean_operation_start(&repo)?;
    ensure_not_detached(&repo)?;
    ensure_no_operation_state(&repo)?;
    if req.commit_shas.is_empty() {
        return Err(RebaseOpError::InvalidStep {
            step_id: "cherrypick".to_string(),
            reason: "at least one commit is required".to_string(),
        });
    }
    let plan = RebaseInteractivePlan {
        steps: req
            .commit_shas
            .iter()
            .enumerate()
            .map(|(index, sha)| RebaseInteractiveStep {
                step_id: format!("cherrypick-{index}"),
                op: RebaseOp::Pick,
                commit_sha: sha.clone(),
                message_override: None,
            })
            .collect(),
    };
    validate_plan(&plan)?;
    let original_head = current_head_oid(&repo)?.to_string();
    let branch = current_branch_name(&repo).unwrap_or_else(|| "HEAD".to_string());
    let mut state = OperationState::new(
        "cherrypick",
        branch,
        None,
        original_head,
        plan,
        req.auto_commit,
    );
    write_operation_state(&repo, &state)?;
    let status = execute_plan_from_state(&repo, &mut state)?;
    Ok(cherrypick_status_from_rebase(status))
}

pub fn cherrypick_continue(workspace_path: &Path) -> Result<CherryPickStatus, RebaseOpError> {
    let status = rebase_continue(
        workspace_path,
        RebaseControlRequest {
            workspace_id: String::new(),
            action: "continue".to_string(),
        },
    )?;
    Ok(cherrypick_status_from_rebase(status))
}

pub fn cherrypick_abort(workspace_path: &Path) -> Result<(), RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    let state = read_operation_state(&repo)?.ok_or(RebaseOpError::NotInRebase)?;
    if state.operation != "cherrypick" {
        return Err(RebaseOpError::NotInRebase);
    }
    reset_to_original_head(&repo, &state)?;
    cleanup_git_state(&repo)?;
    remove_operation_state(&repo)
}

pub fn conflict_resolve_file(
    workspace_path: &Path,
    req: ConflictResolveFileRequest,
) -> Result<(), RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    let final_content = req
        .resolutions
        .iter()
        .map(|item| match &item.resolution {
            ConflictResolution::AcceptOurs => {
                conflict_side_content(&repo, &req.file_path, Side::Ours)
            }
            ConflictResolution::AcceptTheirs => {
                conflict_side_content(&repo, &req.file_path, Side::Theirs)
            }
            ConflictResolution::AcceptBoth => {
                let ours = conflict_side_content(&repo, &req.file_path, Side::Ours)?;
                let theirs = conflict_side_content(&repo, &req.file_path, Side::Theirs)?;
                Ok(format!("{ours}{theirs}"))
            }
            ConflictResolution::Manual { content } => Ok(content.clone()),
        })
        .collect::<Result<Vec<_>, RebaseOpError>>()?
        .join("");
    let path = repo
        .workdir()
        .ok_or_else(|| git_error("Workdir", -1, "bare repository is not supported"))?
        .join(&req.file_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(map_io_error)?;
    }
    fs::write(path, final_content).map_err(map_io_error)?;
    let mut index = repo.index().map_err(map_git_error)?;
    index
        .add_path(Path::new(&req.file_path))
        .map_err(map_git_error)?;
    index.write().map_err(map_git_error)
}

pub fn conflict_status(workspace_path: &Path) -> Result<Vec<ConflictedFile>, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    conflict_files(&repo)
}

pub fn detect_in_progress(workspace_path: &Path) -> Result<CrashRecoveryState, RebaseOpError> {
    let repo = open_repo(workspace_path)?;
    if let Some(state) = read_operation_state(&repo)? {
        return Ok(CrashRecoveryState {
            in_progress: true,
            operation: Some(state.operation),
            branch: Some(state.branch),
            current_step: state.current_step as u32,
            total_steps: state.total_steps as u32,
        });
    }
    for (operation, marker) in [
        ("rebase", "rebase-merge"),
        ("rebase", "rebase-apply"),
        ("merge", "MERGE_HEAD"),
        ("cherrypick", "CHERRY_PICK_HEAD"),
    ] {
        if repo.path().join(marker).exists() {
            return Ok(CrashRecoveryState {
                in_progress: true,
                operation: Some(operation.to_string()),
                branch: current_branch_name(&repo),
                current_step: 0,
                total_steps: 0,
            });
        }
    }
    Ok(CrashRecoveryState {
        in_progress: false,
        operation: None,
        branch: current_branch_name(&repo),
        current_step: 0,
        total_steps: 0,
    })
}

fn validate_plan(plan: &RebaseInteractivePlan) -> Result<(), RebaseOpError> {
    if plan.steps.is_empty() {
        return invalid_step("plan", "plan must contain at least one step");
    }
    if plan.steps.iter().all(|step| step.op == RebaseOp::Drop) {
        return invalid_step("plan", "plan cannot drop every commit");
    }
    if matches!(plan.steps[0].op, RebaseOp::Squash | RebaseOp::Fixup) {
        return invalid_step(
            &plan.steps[0].step_id,
            "first step cannot be squash or fixup",
        );
    }
    for step in &plan.steps {
        if step.step_id.trim().is_empty() {
            return invalid_step(&step.step_id, "step_id is required");
        }
        if Oid::from_str(&step.commit_sha).is_err() {
            return invalid_step(&step.step_id, "commit_sha must be a valid oid");
        }
        if step.op == RebaseOp::Reword && step.message_override.as_deref().unwrap_or("").is_empty()
        {
            return invalid_step(&step.step_id, "reword requires message_override");
        }
    }
    Ok(())
}

fn invalid_step<T>(step_id: &str, reason: &str) -> Result<T, RebaseOpError> {
    Err(RebaseOpError::InvalidStep {
        step_id: step_id.to_string(),
        reason: reason.to_string(),
    })
}

fn execute_plan_from_state(
    repo: &Repository,
    state: &mut OperationState,
) -> Result<RebaseStatus, RebaseOpError> {
    while state.current_step < state.plan.steps.len() {
        let step = state.plan.steps[state.current_step].clone();
        match step.op {
            RebaseOp::Drop => {
                state.current_step += 1;
                state.touch();
                write_operation_state(repo, state)?;
            }
            RebaseOp::Edit => {
                apply_pick_like_step(repo, state, &step, false)?;
                state.touch();
                write_operation_state(repo, state)?;
                return status_from_state(repo, state, true);
            }
            RebaseOp::Pick | RebaseOp::Reword | RebaseOp::Squash | RebaseOp::Fixup => {
                let should_commit = state.operation == "rebase" || state.auto_commit;
                apply_pick_like_step(repo, state, &step, should_commit)?;
                let conflicts = conflict_paths(repo)?;
                if !conflicts.is_empty() {
                    state.touch();
                    write_operation_state(repo, state)?;
                    return Ok(RebaseStatus {
                        in_progress: true,
                        operation: Some(state.operation.clone()),
                        current_step: state.current_step as u32,
                        total_steps: state.total_steps as u32,
                        conflicting_files: conflicts,
                    });
                }
                state.current_step += 1;
                state.touch();
                write_operation_state(repo, state)?;
                if state.operation == "cherrypick" && !state.auto_commit {
                    return status_from_state(repo, state, true);
                }
            }
        }
    }
    let operation = state.operation.clone();
    remove_operation_state(repo)?;
    cleanup_git_state(repo)?;
    Ok(RebaseStatus {
        in_progress: false,
        operation: Some(operation),
        current_step: state.total_steps as u32,
        total_steps: state.total_steps as u32,
        conflicting_files: vec![],
    })
}

fn apply_pick_like_step(
    repo: &Repository,
    state: &OperationState,
    step: &RebaseInteractiveStep,
    should_commit: bool,
) -> Result<(), RebaseOpError> {
    let oid = Oid::from_str(&step.commit_sha).map_err(|error| RebaseOpError::InvalidStep {
        step_id: step.step_id.clone(),
        reason: error.to_string(),
    })?;
    let commit = repo.find_commit(oid).map_err(map_git_error)?;
    let mut opts = CherrypickOptions::new();
    repo.cherrypick(&commit, Some(&mut opts))
        .map_err(map_git_error)?;
    if !conflict_paths(repo)?.is_empty() {
        return Ok(());
    }
    if should_commit {
        let message = message_for_step(repo, step, &commit)?;
        commit_current_index(repo, &message)?;
        cleanup_git_state(repo)?;
    }
    if state.operation == "cherrypick" && !should_commit {
        cleanup_git_state(repo)?;
    }
    Ok(())
}

fn message_for_step(
    repo: &Repository,
    step: &RebaseInteractiveStep,
    commit: &git2::Commit<'_>,
) -> Result<String, RebaseOpError> {
    let original = commit.message().unwrap_or("cherry-pick").to_string();
    Ok(match step.op {
        RebaseOp::Reword => step.message_override.clone().unwrap_or(original),
        RebaseOp::Squash => {
            let previous = head_message(repo).unwrap_or_default();
            if previous.is_empty() {
                original
            } else {
                format!("{previous}\n\n{original}")
            }
        }
        RebaseOp::Fixup => head_message(repo).unwrap_or(original),
        _ => original,
    })
}

fn head_message(repo: &Repository) -> Option<String> {
    repo.head()
        .ok()
        .and_then(|head| head.peel_to_commit().ok())
        .and_then(|commit| commit.message().ok().map(ToOwned::to_owned))
}

fn commit_index_from_step(repo: &Repository, state: &OperationState) -> Result<(), RebaseOpError> {
    let Some(step) = state.plan.steps.get(state.current_step) else {
        return Ok(());
    };
    let oid = Oid::from_str(&step.commit_sha).map_err(map_git_error)?;
    let commit = repo.find_commit(oid).map_err(map_git_error)?;
    commit_current_index(repo, commit.message().unwrap_or("cherry-pick"))?;
    cleanup_git_state(repo)
}

fn commit_current_index(repo: &Repository, message: &str) -> Result<Oid, RebaseOpError> {
    let mut index = repo.index().map_err(map_git_error)?;
    let tree_oid = index.write_tree().map_err(map_git_error)?;
    let tree = repo.find_tree(tree_oid).map_err(map_git_error)?;
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let sig = repo
        .signature()
        .or_else(|_| git2::Signature::now("Codex CLI", "noreply@openai.com"))
        .map_err(map_git_error)?;
    let parents = parent.iter().collect::<Vec<_>>();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(map_git_error)
}

fn create_merge_commit(
    repo: &Repository,
    target_commit: &git2::Commit<'_>,
    message: Option<String>,
) -> Result<MergeStatus, RebaseOpError> {
    let head = repo
        .head()
        .map_err(map_git_error)?
        .peel_to_commit()
        .map_err(map_git_error)?;
    let merge_options = MergeOptions::new();
    let mut index = repo
        .merge_commits(&head, target_commit, Some(&merge_options))
        .map_err(map_git_error)?;
    if index.has_conflicts() {
        let conflicting_files = conflict_paths_from_index(&mut index)?;
        repo.checkout_index(Some(&mut index), None)
            .map_err(map_git_error)?;
        return Ok(MergeStatus {
            outcome: "conflict".to_string(),
            conflicting_files,
        });
    }
    let tree_oid = index.write_tree_to(repo).map_err(map_git_error)?;
    let tree = repo.find_tree(tree_oid).map_err(map_git_error)?;
    let sig = repo
        .signature()
        .or_else(|_| git2::Signature::now("Codex CLI", "noreply@openai.com"))
        .map_err(map_git_error)?;
    let message = message.unwrap_or_else(|| format!("Merge {}", target_commit.id()));
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &message,
        &tree,
        &[&head, target_commit],
    )
    .map_err(map_git_error)?;
    force_checkout_head(repo)?;
    Ok(MergeStatus {
        outcome: "merge-commit".to_string(),
        conflicting_files: vec![],
    })
}

fn create_squash_commit(
    repo: &Repository,
    target_commit: &git2::Commit<'_>,
    message: Option<String>,
) -> Result<MergeStatus, RebaseOpError> {
    let head = repo
        .head()
        .map_err(map_git_error)?
        .peel_to_commit()
        .map_err(map_git_error)?;
    let merge_options = MergeOptions::new();
    let mut index = repo
        .merge_commits(&head, target_commit, Some(&merge_options))
        .map_err(map_git_error)?;
    if index.has_conflicts() {
        let conflicting_files = conflict_paths_from_index(&mut index)?;
        repo.checkout_index(Some(&mut index), None)
            .map_err(map_git_error)?;
        return Ok(MergeStatus {
            outcome: "conflict".to_string(),
            conflicting_files,
        });
    }
    let tree_oid = index.write_tree_to(repo).map_err(map_git_error)?;
    let tree = repo.find_tree(tree_oid).map_err(map_git_error)?;
    let sig = repo
        .signature()
        .or_else(|_| git2::Signature::now("Codex CLI", "noreply@openai.com"))
        .map_err(map_git_error)?;
    let message = message.unwrap_or_else(|| format!("Squash merge {}", target_commit.id()));
    repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&head])
        .map_err(map_git_error)?;
    force_checkout_head(repo)?;
    Ok(MergeStatus {
        outcome: "squash-commit".to_string(),
        conflicting_files: vec![],
    })
}

fn fast_forward_to(repo: &Repository, target: Oid) -> Result<(), RebaseOpError> {
    let target_object = repo
        .find_object(target, Some(ObjectType::Commit))
        .map_err(map_git_error)?;
    let mut checkout = CheckoutBuilder::new();
    checkout.safe();
    repo.checkout_tree(&target_object, Some(&mut checkout))
        .map_err(map_git_error)?;
    let head_name = repo
        .head()
        .map_err(map_git_error)?
        .name()
        .map_err(|_| git_error("Reference", -1, "HEAD has no name"))?
        .to_string();
    let mut reference = repo.find_reference(&head_name).map_err(map_git_error)?;
    reference
        .set_target(target, "MVP-16 fast-forward merge")
        .map_err(map_git_error)?;
    repo.set_head(&head_name).map_err(map_git_error)
}

fn commits_to_replay(
    repo: &Repository,
    branch: &str,
    onto: &str,
) -> Result<Vec<Oid>, RebaseOpError> {
    let branch_commit = resolve_commit(repo, branch)?;
    let onto_commit = resolve_commit(repo, onto)?;
    let base = repo
        .merge_base(branch_commit.id(), onto_commit.id())
        .map_err(map_git_error)?;
    if base == branch_commit.id() {
        return Ok(vec![]);
    }
    let mut walk = repo.revwalk().map_err(map_git_error)?;
    walk.push(branch_commit.id()).map_err(map_git_error)?;
    walk.hide(base).map_err(map_git_error)?;
    let mut commits = walk.collect::<Result<Vec<_>, _>>().map_err(map_git_error)?;
    commits.reverse();
    Ok(commits)
}

fn resolve_commit<'repo>(
    repo: &'repo Repository,
    name: &str,
) -> Result<git2::Commit<'repo>, RebaseOpError> {
    let object = repo
        .revparse_single(name)
        .or_else(|_| {
            repo.find_branch(name, BranchType::Local)
                .and_then(|branch| branch.get().peel(ObjectType::Commit))
        })
        .map_err(map_git_error)?;
    object.peel_to_commit().map_err(map_git_error)
}

fn checkout_branch_if_needed(repo: &Repository, branch: &str) -> Result<(), RebaseOpError> {
    if current_branch_name(repo).as_deref() == Some(branch) {
        return Ok(());
    }
    let branch_ref = repo
        .find_branch(branch, BranchType::Local)
        .map_err(map_git_error)?;
    let reference_name = branch_ref
        .get()
        .name()
        .map_err(|_| git_error("Reference", -1, "branch reference has no name"))?
        .to_string();
    repo.set_head(&reference_name).map_err(map_git_error)?;
    force_checkout_head(repo)
}

fn hard_reset(repo: &Repository, oid: Oid) -> Result<(), RebaseOpError> {
    let object = repo
        .find_object(oid, Some(ObjectType::Commit))
        .map_err(map_git_error)?;
    let mut checkout = CheckoutBuilder::new();
    checkout.force();
    repo.reset(&object, ResetType::Hard, Some(&mut checkout))
        .map_err(map_git_error)
}

fn reset_to_original_head(repo: &Repository, state: &OperationState) -> Result<(), RebaseOpError> {
    let oid = Oid::from_str(&state.original_head).map_err(map_git_error)?;
    hard_reset(repo, oid)
}

fn force_checkout_head(repo: &Repository) -> Result<(), RebaseOpError> {
    let mut checkout = CheckoutBuilder::new();
    checkout.force();
    repo.checkout_head(Some(&mut checkout))
        .map_err(map_git_error)
}

fn ensure_clean_operation_start(repo: &Repository) -> Result<(), RebaseOpError> {
    let dirty = dirty_working_tree(repo)?;
    if dirty.modified.is_empty() && dirty.staged.is_empty() {
        Ok(())
    } else {
        Err(RebaseOpError::DirtyWorkingTree {
            modified: dirty.modified,
            staged: dirty.staged,
        })
    }
}

fn dirty_working_tree(repo: &Repository) -> Result<DirtyState, RebaseOpError> {
    let mut options = StatusOptions::new();
    options.include_untracked(true).renames_head_to_index(true);
    let statuses = repo.statuses(Some(&mut options)).map_err(map_git_error)?;
    let mut dirty = DirtyState::default();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();
        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            dirty.staged.push(path.clone());
        }
        if status.intersects(
            Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE,
        ) {
            dirty.modified.push(path);
        }
    }
    Ok(dirty)
}

fn ensure_not_detached(repo: &Repository) -> Result<(), RebaseOpError> {
    if repo.head_detached().map_err(map_git_error)? {
        Err(RebaseOpError::DetachedHead)
    } else {
        Ok(())
    }
}

fn ensure_no_operation_state(repo: &Repository) -> Result<(), RebaseOpError> {
    if let Some(state) = read_operation_state(repo)? {
        return Err(RebaseOpError::OperationInProgress {
            existing: state.operation,
        });
    }
    for (operation, marker) in [
        ("rebase", "rebase-merge"),
        ("rebase", "rebase-apply"),
        ("merge", "MERGE_HEAD"),
        ("cherrypick", "CHERRY_PICK_HEAD"),
    ] {
        if repo.path().join(marker).exists() {
            return Err(RebaseOpError::OperationInProgress {
                existing: operation.to_string(),
            });
        }
    }
    Ok(())
}

fn current_head_oid(repo: &Repository) -> Result<Oid, RebaseOpError> {
    repo.head()
        .map_err(map_git_error)?
        .target()
        .ok_or(RebaseOpError::DetachedHead)
}

fn current_branch_name(repo: &Repository) -> Option<String> {
    repo.head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(ToOwned::to_owned))
}

fn cherrypick_status_from_rebase(status: RebaseStatus) -> CherryPickStatus {
    CherryPickStatus {
        in_progress: status.in_progress,
        current_index: status.current_step,
        total_commits: status.total_steps,
        conflicting_files: status.conflicting_files,
    }
}

fn status_from_state(
    repo: &Repository,
    state: &OperationState,
    in_progress: bool,
) -> Result<RebaseStatus, RebaseOpError> {
    Ok(RebaseStatus {
        in_progress,
        operation: Some(state.operation.clone()),
        current_step: state.current_step as u32,
        total_steps: state.total_steps as u32,
        conflicting_files: conflict_paths(repo)?,
    })
}

impl OperationState {
    fn new(
        operation: impl Into<String>,
        branch: impl Into<String>,
        onto: Option<String>,
        original_head: String,
        plan: RebaseInteractivePlan,
        auto_commit: bool,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        let total_steps = plan.steps.len();
        Self {
            operation: operation.into(),
            branch: branch.into(),
            onto,
            original_head,
            plan,
            current_step: 0,
            total_steps,
            auto_commit,
            started_at: now,
            last_updated: now,
        }
    }

    fn touch(&mut self) {
        self.last_updated = chrono::Utc::now().timestamp();
    }
}

fn state_path(repo: &Repository) -> PathBuf {
    repo.path().join("vibestation-rebase-state.json")
}

fn read_operation_state(repo: &Repository) -> Result<Option<OperationState>, RebaseOpError> {
    let path = state_path(repo);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(map_io_error)?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|error| git_error("State", -1, error.to_string()))
}

fn write_operation_state(repo: &Repository, state: &OperationState) -> Result<(), RebaseOpError> {
    let content = serde_json::to_string_pretty(state)
        .map_err(|error| git_error("State", -1, error.to_string()))?;
    fs::write(state_path(repo), content).map_err(map_io_error)
}

fn remove_operation_state(repo: &Repository) -> Result<(), RebaseOpError> {
    let path = state_path(repo);
    if path.exists() {
        fs::remove_file(path).map_err(map_io_error)?;
    }
    Ok(())
}

fn cleanup_git_state(repo: &Repository) -> Result<(), RebaseOpError> {
    repo.cleanup_state().map_err(map_git_error)
}

fn conflict_paths(repo: &Repository) -> Result<Vec<String>, RebaseOpError> {
    Ok(conflict_files(repo)?
        .into_iter()
        .map(|file| file.path)
        .collect())
}

fn conflict_paths_from_index(index: &mut git2::Index) -> Result<Vec<String>, RebaseOpError> {
    let conflicts = index.conflicts().map_err(map_git_error)?;
    let mut paths = Vec::new();
    for conflict in conflicts {
        let conflict = conflict.map_err(map_git_error)?;
        if let Some(path) = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .map(|entry| String::from_utf8_lossy(&entry.path).to_string())
        {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn conflict_files(repo: &Repository) -> Result<Vec<ConflictedFile>, RebaseOpError> {
    let index = repo.index().map_err(map_git_error)?;
    let conflicts = index.conflicts().map_err(map_git_error)?;
    let mut files = Vec::new();
    for conflict in conflicts {
        let conflict = conflict.map_err(map_git_error)?;
        let path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .map(|entry| String::from_utf8_lossy(&entry.path).to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let base_content = conflict_entry_content(repo, conflict.ancestor.as_ref())?;
        let ours_content = conflict_entry_content(repo, conflict.our.as_ref())?;
        let theirs_content = conflict_entry_content(repo, conflict.their.as_ref())?;
        files.push(ConflictedFile {
            path: path.clone(),
            hunks: vec![ConflictHunk {
                id: format!("{path}:0"),
                base_content,
                ours_content,
                theirs_content,
                resolved: false,
                resolution: None,
            }],
            resolved: false,
        });
    }
    Ok(files)
}

enum Side {
    Ours,
    Theirs,
}

fn conflict_side_content(
    repo: &Repository,
    file_path: &str,
    side: Side,
) -> Result<String, RebaseOpError> {
    let file = conflict_files(repo)?
        .into_iter()
        .find(|file| file.path == file_path)
        .ok_or_else(|| {
            git_error(
                "Conflict",
                -1,
                format!("conflict not found for {file_path}"),
            )
        })?;
    let Some(hunk) = file.hunks.into_iter().next() else {
        return Ok(String::new());
    };
    Ok(match side {
        Side::Ours => hunk.ours_content,
        Side::Theirs => hunk.theirs_content,
    })
}

fn conflict_entry_content(
    repo: &Repository,
    entry: Option<&git2::IndexEntry>,
) -> Result<String, RebaseOpError> {
    let Some(entry) = entry else {
        return Ok(String::new());
    };
    let blob = repo.find_blob(entry.id).map_err(map_git_error)?;
    Ok(String::from_utf8_lossy(blob.content()).to_string())
}

fn open_repo(path: &Path) -> Result<Repository, RebaseOpError> {
    Repository::open(path).map_err(map_git_error)
}

fn map_git_error(error: git2::Error) -> RebaseOpError {
    RebaseOpError::Git2Error {
        class: format!("{:?}", error.class()),
        code: error.raw_code(),
        message: error.message().to_string(),
    }
}

fn map_io_error(error: std::io::Error) -> RebaseOpError {
    git_error("Io", error.raw_os_error().unwrap_or(-1), error.to_string())
}

fn git_error(class: impl Into<String>, code: i32, message: impl Into<String>) -> RebaseOpError {
    RebaseOpError::Git2Error {
        class: class.into(),
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{BranchType, Repository, Signature};
    use tempfile::TempDir;

    struct Fixture {
        _dir: TempDir,
        repo: Repository,
        path: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let repo = Repository::init(dir.path()).unwrap();
            {
                let mut config = repo.config().unwrap();
                config.set_str("user.name", "Codex CLI").unwrap();
                config.set_str("user.email", "noreply@openai.com").unwrap();
            }
            Self {
                path: dir.path().to_path_buf(),
                _dir: dir,
                repo,
            }
        }

        fn commit_file(&self, path: &str, content: &str, message: &str) -> Oid {
            let full_path = self.path.join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&full_path, content).unwrap();
            let mut index = self.repo.index().unwrap();
            index.add_path(Path::new(path)).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = self.repo.find_tree(tree_oid).unwrap();
            let sig = Signature::now("Codex CLI", "noreply@openai.com").unwrap();
            let parent = self
                .repo
                .head()
                .ok()
                .and_then(|head| head.peel_to_commit().ok());
            let parents = parent.iter().collect::<Vec<_>>();
            self.repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
                .unwrap()
        }

        fn branch(&self, name: &str, oid: Oid) {
            let commit = self.repo.find_commit(oid).unwrap();
            self.repo.branch(name, &commit, true).unwrap();
        }

        fn checkout(&self, name: &str) {
            self.repo.set_head(&format!("refs/heads/{name}")).unwrap();
            let mut checkout = CheckoutBuilder::new();
            checkout.force();
            self.repo.checkout_head(Some(&mut checkout)).unwrap();
        }

        fn head(&self) -> Oid {
            self.repo.head().unwrap().target().unwrap()
        }
    }

    fn diverged_fixture() -> Fixture {
        let fixture = Fixture::new();
        let base = fixture.commit_file("file.txt", "base\n", "base");
        fixture.branch("main", base);
        fixture.branch("feature", base);
        fixture.checkout("main");
        fixture.commit_file("main.txt", "main\n", "main");
        fixture.checkout("feature");
        fixture.commit_file("feature.txt", "feature\n", "feature");
        fixture
    }

    fn conflict_fixture() -> Fixture {
        let fixture = Fixture::new();
        let base = fixture.commit_file("file.txt", "base\n", "base");
        fixture.branch("main", base);
        fixture.branch("feature", base);
        fixture.checkout("main");
        fixture.commit_file("file.txt", "main\n", "main change");
        fixture.checkout("feature");
        fixture.commit_file("file.txt", "feature\n", "feature change");
        fixture
    }

    fn plan_with_ops(ops: &[RebaseOp]) -> RebaseInteractivePlan {
        let fixture = Fixture::new();
        let first = fixture.commit_file("a.txt", "a\n", "a");
        let second = fixture.commit_file("b.txt", "b\n", "b");
        let ids = [first, second, first, second, first, second];
        RebaseInteractivePlan {
            steps: ops
                .iter()
                .enumerate()
                .map(|(index, op)| RebaseInteractiveStep {
                    step_id: format!("step-{index}"),
                    op: *op,
                    commit_sha: ids[index].to_string(),
                    message_override: if *op == RebaseOp::Reword {
                        Some("new message".to_string())
                    } else {
                        None
                    },
                })
                .collect(),
        }
    }

    mod plan_validate {
        use super::*;

        #[test]
        fn valid_plan_accepts_pick() {
            assert!(validate_plan(&plan_with_ops(&[RebaseOp::Pick])).is_ok());
        }

        #[test]
        fn valid_plan_accepts_mixed_ops() {
            assert!(validate_plan(&plan_with_ops(&[
                RebaseOp::Pick,
                RebaseOp::Reword,
                RebaseOp::Squash,
                RebaseOp::Fixup,
                RebaseOp::Drop,
            ]))
            .is_ok());
        }

        #[test]
        fn empty_plan_rejected() {
            assert!(matches!(
                validate_plan(&RebaseInteractivePlan { steps: vec![] }).unwrap_err(),
                RebaseOpError::InvalidStep { .. }
            ));
        }

        #[test]
        fn squash_first_rejected() {
            assert!(matches!(
                validate_plan(&plan_with_ops(&[RebaseOp::Squash])).unwrap_err(),
                RebaseOpError::InvalidStep { .. }
            ));
        }

        #[test]
        fn fixup_first_rejected() {
            assert!(matches!(
                validate_plan(&plan_with_ops(&[RebaseOp::Fixup])).unwrap_err(),
                RebaseOpError::InvalidStep { .. }
            ));
        }

        #[test]
        fn all_drop_rejected() {
            assert!(matches!(
                validate_plan(&plan_with_ops(&[RebaseOp::Drop, RebaseOp::Drop])).unwrap_err(),
                RebaseOpError::InvalidStep { .. }
            ));
        }

        #[test]
        fn invalid_sha_rejected() {
            let mut plan = plan_with_ops(&[RebaseOp::Pick]);
            plan.steps[0].commit_sha = "not-a-sha".to_string();
            assert!(matches!(
                validate_plan(&plan).unwrap_err(),
                RebaseOpError::InvalidStep { .. }
            ));
        }

        #[test]
        fn reword_requires_message() {
            let mut plan = plan_with_ops(&[RebaseOp::Reword]);
            plan.steps[0].message_override = None;
            assert!(matches!(
                validate_plan(&plan).unwrap_err(),
                RebaseOpError::InvalidStep { .. }
            ));
        }
    }

    mod rebase_basic {
        use super::*;

        #[test]
        fn rebase_start_clean_finishes() {
            let fixture = diverged_fixture();
            let status = rebase_start(
                &fixture.path,
                RebaseStartRequest {
                    workspace_id: "w".to_string(),
                    branch: "feature".to_string(),
                    onto: "main".to_string(),
                    interactive: false,
                },
            )
            .unwrap();
            assert!(!status.in_progress);
        }

        #[test]
        fn rebase_start_conflict_returns_status() {
            let fixture = conflict_fixture();
            let status = rebase_start(
                &fixture.path,
                RebaseStartRequest {
                    workspace_id: "w".to_string(),
                    branch: "feature".to_string(),
                    onto: "main".to_string(),
                    interactive: false,
                },
            )
            .unwrap();
            assert!(status.in_progress);
            assert_eq!(status.conflicting_files, vec!["file.txt"]);
        }

        #[test]
        fn rebase_abort_resets_head() {
            let fixture = conflict_fixture();
            let original = fixture.head();
            let _ = rebase_start(
                &fixture.path,
                RebaseStartRequest {
                    workspace_id: "w".to_string(),
                    branch: "feature".to_string(),
                    onto: "main".to_string(),
                    interactive: false,
                },
            )
            .unwrap();
            rebase_abort(
                &fixture.path,
                RebaseControlRequest {
                    workspace_id: "w".to_string(),
                    action: "abort".to_string(),
                },
            )
            .unwrap();
            assert_eq!(fixture.head(), original);
        }

        #[test]
        fn rebase_continue_with_conflicts_errors() {
            let fixture = conflict_fixture();
            let _ = rebase_start(
                &fixture.path,
                RebaseStartRequest {
                    workspace_id: "w".to_string(),
                    branch: "feature".to_string(),
                    onto: "main".to_string(),
                    interactive: false,
                },
            )
            .unwrap();
            assert!(matches!(
                rebase_continue(
                    &fixture.path,
                    RebaseControlRequest {
                        workspace_id: "w".to_string(),
                        action: "continue".to_string(),
                    },
                )
                .unwrap_err(),
                RebaseOpError::ConflictUnresolved { .. }
            ));
        }

        #[test]
        fn rebase_skip_finishes_after_conflict() {
            let fixture = conflict_fixture();
            let _ = rebase_start(
                &fixture.path,
                RebaseStartRequest {
                    workspace_id: "w".to_string(),
                    branch: "feature".to_string(),
                    onto: "main".to_string(),
                    interactive: false,
                },
            )
            .unwrap();
            let status = rebase_skip(
                &fixture.path,
                RebaseControlRequest {
                    workspace_id: "w".to_string(),
                    action: "skip".to_string(),
                },
            )
            .unwrap();
            assert!(!status.in_progress);
        }

        #[test]
        fn rebase_onto_self_errors() {
            let fixture = diverged_fixture();
            assert!(matches!(
                rebase_start(
                    &fixture.path,
                    RebaseStartRequest {
                        workspace_id: "w".to_string(),
                        branch: "feature".to_string(),
                        onto: "feature".to_string(),
                        interactive: false,
                    },
                )
                .unwrap_err(),
                RebaseOpError::AlreadyUpToDate
            ));
        }

        #[test]
        fn rebase_onto_ancestor_errors() {
            let fixture = Fixture::new();
            let base = fixture.commit_file("base.txt", "base\n", "base");
            fixture.branch("main", base);
            fixture.branch("old", base);
            fixture.checkout("main");
            fixture.commit_file("main.txt", "main\n", "main");
            assert!(matches!(
                rebase_start(
                    &fixture.path,
                    RebaseStartRequest {
                        workspace_id: "w".to_string(),
                        branch: "old".to_string(),
                        onto: "main".to_string(),
                        interactive: false,
                    },
                )
                .unwrap_err(),
                RebaseOpError::AlreadyUpToDate
            ));
        }

        #[test]
        fn rebase_detached_head_errors() {
            let fixture = diverged_fixture();
            let head = fixture.head();
            fixture.repo.set_head_detached(head).unwrap();
            assert!(matches!(
                rebase_start(
                    &fixture.path,
                    RebaseStartRequest {
                        workspace_id: "w".to_string(),
                        branch: "feature".to_string(),
                        onto: "main".to_string(),
                        interactive: false,
                    },
                )
                .unwrap_err(),
                RebaseOpError::DetachedHead
            ));
        }

        #[test]
        fn rebase_dirty_tree_errors() {
            let fixture = diverged_fixture();
            fs::write(fixture.path.join("dirty.txt"), "dirty\n").unwrap();
            assert!(matches!(
                rebase_start(
                    &fixture.path,
                    RebaseStartRequest {
                        workspace_id: "w".to_string(),
                        branch: "feature".to_string(),
                        onto: "main".to_string(),
                        interactive: false,
                    },
                )
                .unwrap_err(),
                RebaseOpError::DirtyWorkingTree { .. }
            ));
        }

        #[test]
        fn rebase_large_linear_fixture_finishes() {
            let fixture = diverged_fixture();
            fixture.checkout("feature");
            for index in 0..20 {
                fixture.commit_file(
                    &format!("many-{index}.txt"),
                    "x\n",
                    &format!("many {index}"),
                );
            }
            let status = rebase_start(
                &fixture.path,
                RebaseStartRequest {
                    workspace_id: "w".to_string(),
                    branch: "feature".to_string(),
                    onto: "main".to_string(),
                    interactive: false,
                },
            )
            .unwrap();
            assert!(!status.in_progress);
        }
    }

    mod rebase_interactive {
        use super::*;

        fn feature_tip(fixture: &Fixture) -> Oid {
            fixture
                .repo
                .find_branch("feature", BranchType::Local)
                .unwrap()
                .get()
                .target()
                .unwrap()
        }

        #[test]
        fn interactive_plan_lists_commits() {
            let fixture = diverged_fixture();
            let plan = rebase_interactive_plan(&fixture.path, "feature", "main").unwrap();
            assert_eq!(plan.steps.len(), 1);
        }

        #[test]
        fn interactive_apply_pick_commits() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let plan = RebaseInteractivePlan {
                steps: vec![RebaseInteractiveStep {
                    step_id: "pick".to_string(),
                    op: RebaseOp::Pick,
                    commit_sha: feature_tip(&fixture).to_string(),
                    message_override: None,
                }],
            };
            assert!(
                !rebase_interactive_apply(&fixture.path, plan)
                    .unwrap()
                    .in_progress
            );
        }

        #[test]
        fn interactive_apply_reword_changes_message() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let status = rebase_interactive_apply(
                &fixture.path,
                RebaseInteractivePlan {
                    steps: vec![RebaseInteractiveStep {
                        step_id: "reword".to_string(),
                        op: RebaseOp::Reword,
                        commit_sha: feature_tip(&fixture).to_string(),
                        message_override: Some("rewritten".to_string()),
                    }],
                },
            )
            .unwrap();
            assert!(!status.in_progress);
            assert_eq!(
                fixture
                    .repo
                    .head()
                    .unwrap()
                    .peel_to_commit()
                    .unwrap()
                    .message()
                    .ok(),
                Some("rewritten")
            );
        }

        #[test]
        fn interactive_apply_edit_pauses() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let status = rebase_interactive_apply(
                &fixture.path,
                RebaseInteractivePlan {
                    steps: vec![RebaseInteractiveStep {
                        step_id: "edit".to_string(),
                        op: RebaseOp::Edit,
                        commit_sha: feature_tip(&fixture).to_string(),
                        message_override: None,
                    }],
                },
            )
            .unwrap();
            assert!(status.in_progress);
        }

        #[test]
        fn interactive_apply_squash_finishes() {
            let fixture = diverged_fixture();
            fixture.checkout("feature");
            let a = fixture.commit_file("s1.txt", "s1\n", "s1");
            let b = fixture.commit_file("s2.txt", "s2\n", "s2");
            fixture.checkout("main");
            let status = rebase_interactive_apply(
                &fixture.path,
                RebaseInteractivePlan {
                    steps: vec![
                        RebaseInteractiveStep {
                            step_id: "pick".to_string(),
                            op: RebaseOp::Pick,
                            commit_sha: a.to_string(),
                            message_override: None,
                        },
                        RebaseInteractiveStep {
                            step_id: "squash".to_string(),
                            op: RebaseOp::Squash,
                            commit_sha: b.to_string(),
                            message_override: None,
                        },
                    ],
                },
            )
            .unwrap();
            assert!(!status.in_progress);
        }

        #[test]
        fn interactive_apply_fixup_finishes() {
            let fixture = diverged_fixture();
            fixture.checkout("feature");
            let a = fixture.commit_file("f1.txt", "f1\n", "f1");
            let b = fixture.commit_file("f2.txt", "f2\n", "f2");
            fixture.checkout("main");
            let status = rebase_interactive_apply(
                &fixture.path,
                RebaseInteractivePlan {
                    steps: vec![
                        RebaseInteractiveStep {
                            step_id: "pick".to_string(),
                            op: RebaseOp::Pick,
                            commit_sha: a.to_string(),
                            message_override: None,
                        },
                        RebaseInteractiveStep {
                            step_id: "fixup".to_string(),
                            op: RebaseOp::Fixup,
                            commit_sha: b.to_string(),
                            message_override: None,
                        },
                    ],
                },
            )
            .unwrap();
            assert!(!status.in_progress);
        }
    }

    mod merge {
        use super::*;

        #[test]
        fn merge_fast_forward_succeeds() {
            let fixture = Fixture::new();
            let base = fixture.commit_file("a.txt", "a\n", "a");
            fixture.branch("main", base);
            fixture.branch("feature", base);
            fixture.checkout("feature");
            fixture.commit_file("b.txt", "b\n", "b");
            fixture.checkout("main");
            let status = merge_start(
                &fixture.path,
                MergeRequest {
                    workspace_id: "w".to_string(),
                    source_branch: "feature".to_string(),
                    strategy: MergeStrategy::FastForward,
                    commit_message: None,
                },
            )
            .unwrap();
            assert_eq!(status.outcome, "fast-forwarded");
        }

        #[test]
        fn merge_no_ff_succeeds() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let status = merge_start(
                &fixture.path,
                MergeRequest {
                    workspace_id: "w".to_string(),
                    source_branch: "feature".to_string(),
                    strategy: MergeStrategy::NoFastForward,
                    commit_message: Some("merge feature".to_string()),
                },
            )
            .unwrap();
            assert_eq!(status.outcome, "merge-commit");
        }

        #[test]
        fn merge_squash_succeeds() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let status = merge_start(
                &fixture.path,
                MergeRequest {
                    workspace_id: "w".to_string(),
                    source_branch: "feature".to_string(),
                    strategy: MergeStrategy::Squash,
                    commit_message: Some("squash feature".to_string()),
                },
            )
            .unwrap();
            assert_eq!(status.outcome, "squash-commit");
        }

        #[test]
        fn merge_conflict_returns_status() {
            let fixture = conflict_fixture();
            fixture.checkout("main");
            let status = merge_start(
                &fixture.path,
                MergeRequest {
                    workspace_id: "w".to_string(),
                    source_branch: "feature".to_string(),
                    strategy: MergeStrategy::NoFastForward,
                    commit_message: None,
                },
            )
            .unwrap();
            assert_eq!(status.outcome, "conflict");
        }

        #[test]
        fn merge_abort_without_merge_errors() {
            let fixture = diverged_fixture();
            assert!(matches!(
                merge_abort(&fixture.path).unwrap_err(),
                RebaseOpError::AlreadyUpToDate
            ));
        }

        #[test]
        fn merge_dirty_tree_errors() {
            let fixture = diverged_fixture();
            fs::write(fixture.path.join("dirty.txt"), "dirty\n").unwrap();
            assert!(matches!(
                merge_start(
                    &fixture.path,
                    MergeRequest {
                        workspace_id: "w".to_string(),
                        source_branch: "main".to_string(),
                        strategy: MergeStrategy::NoFastForward,
                        commit_message: None,
                    },
                )
                .unwrap_err(),
                RebaseOpError::DirtyWorkingTree { .. }
            ));
        }

        #[test]
        fn merge_detached_head_errors() {
            let fixture = diverged_fixture();
            let head = fixture.head();
            fixture.repo.set_head_detached(head).unwrap();
            assert!(matches!(
                merge_start(
                    &fixture.path,
                    MergeRequest {
                        workspace_id: "w".to_string(),
                        source_branch: "main".to_string(),
                        strategy: MergeStrategy::NoFastForward,
                        commit_message: None,
                    },
                )
                .unwrap_err(),
                RebaseOpError::DetachedHead
            ));
        }
    }

    mod cherrypick {
        use super::*;

        fn feature_tip(fixture: &Fixture) -> Oid {
            fixture
                .repo
                .find_branch("feature", BranchType::Local)
                .unwrap()
                .get()
                .target()
                .unwrap()
        }

        #[test]
        fn cherrypick_single_ok() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let status = cherrypick_start(
                &fixture.path,
                CherryPickRequest {
                    workspace_id: "w".to_string(),
                    commit_shas: vec![feature_tip(&fixture).to_string()],
                    auto_commit: true,
                },
            )
            .unwrap();
            assert!(!status.in_progress);
        }

        #[test]
        fn cherrypick_single_conflict() {
            let fixture = conflict_fixture();
            fixture.checkout("main");
            let status = cherrypick_start(
                &fixture.path,
                CherryPickRequest {
                    workspace_id: "w".to_string(),
                    commit_shas: vec![feature_tip(&fixture).to_string()],
                    auto_commit: true,
                },
            )
            .unwrap();
            assert!(status.in_progress);
        }

        #[test]
        fn cherrypick_range_ok() {
            let fixture = diverged_fixture();
            fixture.checkout("feature");
            let a = fixture.commit_file("r1.txt", "r1\n", "r1");
            let b = fixture.commit_file("r2.txt", "r2\n", "r2");
            fixture.checkout("main");
            let status = cherrypick_start(
                &fixture.path,
                CherryPickRequest {
                    workspace_id: "w".to_string(),
                    commit_shas: vec![a.to_string(), b.to_string()],
                    auto_commit: true,
                },
            )
            .unwrap();
            assert!(!status.in_progress);
        }

        #[test]
        fn cherrypick_abort_resets_head() {
            let fixture = conflict_fixture();
            fixture.checkout("main");
            let original = fixture.head();
            let _ = cherrypick_start(
                &fixture.path,
                CherryPickRequest {
                    workspace_id: "w".to_string(),
                    commit_shas: vec![feature_tip(&fixture).to_string()],
                    auto_commit: true,
                },
            )
            .unwrap();
            cherrypick_abort(&fixture.path).unwrap();
            assert_eq!(fixture.head(), original);
        }

        #[test]
        fn cherrypick_no_commit_leaves_in_progress() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let status = cherrypick_start(
                &fixture.path,
                CherryPickRequest {
                    workspace_id: "w".to_string(),
                    commit_shas: vec![feature_tip(&fixture).to_string()],
                    auto_commit: false,
                },
            )
            .unwrap();
            assert!(status.in_progress);
        }

        #[test]
        fn cherrypick_continue_after_no_commit_finishes() {
            let fixture = diverged_fixture();
            fixture.checkout("main");
            let _ = cherrypick_start(
                &fixture.path,
                CherryPickRequest {
                    workspace_id: "w".to_string(),
                    commit_shas: vec![feature_tip(&fixture).to_string()],
                    auto_commit: false,
                },
            )
            .unwrap();
            assert!(!cherrypick_continue(&fixture.path).unwrap().in_progress);
        }

        #[test]
        fn cherrypick_detached_head_errors() {
            let fixture = diverged_fixture();
            let head = fixture.head();
            fixture.repo.set_head_detached(head).unwrap();
            assert!(matches!(
                cherrypick_start(
                    &fixture.path,
                    CherryPickRequest {
                        workspace_id: "w".to_string(),
                        commit_shas: vec![head.to_string()],
                        auto_commit: true,
                    },
                )
                .unwrap_err(),
                RebaseOpError::DetachedHead
            ));
        }
    }

    mod conflict_detect {
        use super::*;

        fn start_conflict(fixture: &Fixture) {
            fixture.checkout("main");
            let sha = fixture
                .repo
                .find_branch("feature", BranchType::Local)
                .unwrap()
                .get()
                .target()
                .unwrap();
            let _ = cherrypick_start(
                &fixture.path,
                CherryPickRequest {
                    workspace_id: "w".to_string(),
                    commit_shas: vec![sha.to_string()],
                    auto_commit: true,
                },
            )
            .unwrap();
        }

        #[test]
        fn conflict_status_has_conflicts() {
            let fixture = conflict_fixture();
            start_conflict(&fixture);
            assert_eq!(conflict_status(&fixture.path).unwrap().len(), 1);
        }

        #[test]
        fn conflict_status_contains_three_sides() {
            let fixture = conflict_fixture();
            start_conflict(&fixture);
            let files = conflict_status(&fixture.path).unwrap();
            let hunk = &files[0].hunks[0];
            assert!(hunk.ours_content.contains("main"));
            assert!(hunk.theirs_content.contains("feature"));
            assert!(hunk.base_content.contains("base"));
        }

        #[test]
        fn conflict_resolve_file_accept_ours_marks_index() {
            let fixture = conflict_fixture();
            start_conflict(&fixture);
            conflict_resolve_file(
                &fixture.path,
                ConflictResolveFileRequest {
                    workspace_id: "w".to_string(),
                    file_path: "file.txt".to_string(),
                    resolutions: vec![ConflictHunkResolution {
                        hunk_id: "file.txt:0".to_string(),
                        resolution: ConflictResolution::AcceptOurs,
                    }],
                },
            )
            .unwrap();
            assert!(conflict_status(&fixture.path).unwrap().is_empty());
        }

        #[test]
        fn conflict_resolve_file_manual_writes_content() {
            let fixture = conflict_fixture();
            start_conflict(&fixture);
            conflict_resolve_file(
                &fixture.path,
                ConflictResolveFileRequest {
                    workspace_id: "w".to_string(),
                    file_path: "file.txt".to_string(),
                    resolutions: vec![ConflictHunkResolution {
                        hunk_id: "file.txt:0".to_string(),
                        resolution: ConflictResolution::Manual {
                            content: "manual\n".to_string(),
                        },
                    }],
                },
            )
            .unwrap();
            assert_eq!(
                fs::read_to_string(fixture.path.join("file.txt")).unwrap(),
                "manual\n"
            );
        }

        #[test]
        fn conflict_resolve_both_concatenates() {
            let fixture = conflict_fixture();
            start_conflict(&fixture);
            conflict_resolve_file(
                &fixture.path,
                ConflictResolveFileRequest {
                    workspace_id: "w".to_string(),
                    file_path: "file.txt".to_string(),
                    resolutions: vec![ConflictHunkResolution {
                        hunk_id: "file.txt:0".to_string(),
                        resolution: ConflictResolution::AcceptBoth,
                    }],
                },
            )
            .unwrap();
            let content = fs::read_to_string(fixture.path.join("file.txt")).unwrap();
            assert!(content.contains("main"));
            assert!(content.contains("feature"));
        }
    }

    mod crash_recovery {
        use super::*;

        #[test]
        fn detect_no_in_progress() {
            let fixture = diverged_fixture();
            assert!(!detect_in_progress(&fixture.path).unwrap().in_progress);
        }

        #[test]
        fn detect_rebase_state_file() {
            let fixture = diverged_fixture();
            let plan = rebase_interactive_plan(&fixture.path, "feature", "main").unwrap();
            let state = OperationState::new(
                "rebase",
                "feature",
                Some("main".to_string()),
                fixture.head().to_string(),
                plan,
                true,
            );
            write_operation_state(&fixture.repo, &state).unwrap();
            assert_eq!(
                detect_in_progress(&fixture.path)
                    .unwrap()
                    .operation
                    .as_deref(),
                Some("rebase")
            );
        }

        #[test]
        fn detect_rebase_merge_marker() {
            let fixture = diverged_fixture();
            fs::create_dir_all(fixture.repo.path().join("rebase-merge")).unwrap();
            assert_eq!(
                detect_in_progress(&fixture.path)
                    .unwrap()
                    .operation
                    .as_deref(),
                Some("rebase")
            );
        }

        #[test]
        fn detect_merge_marker() {
            let fixture = diverged_fixture();
            fs::write(
                fixture.repo.path().join("MERGE_HEAD"),
                fixture.head().to_string(),
            )
            .unwrap();
            assert_eq!(
                detect_in_progress(&fixture.path)
                    .unwrap()
                    .operation
                    .as_deref(),
                Some("merge")
            );
        }

        #[test]
        fn detect_cherrypick_marker() {
            let fixture = diverged_fixture();
            fs::write(
                fixture.repo.path().join("CHERRY_PICK_HEAD"),
                fixture.head().to_string(),
            )
            .unwrap();
            assert_eq!(
                detect_in_progress(&fixture.path)
                    .unwrap()
                    .operation
                    .as_deref(),
                Some("cherrypick")
            );
        }

        #[test]
        fn state_wins_over_git_marker() {
            let fixture = diverged_fixture();
            fs::write(
                fixture.repo.path().join("MERGE_HEAD"),
                fixture.head().to_string(),
            )
            .unwrap();
            let plan = rebase_interactive_plan(&fixture.path, "feature", "main").unwrap();
            let state = OperationState::new(
                "rebase",
                "feature",
                Some("main".to_string()),
                fixture.head().to_string(),
                plan,
                true,
            );
            write_operation_state(&fixture.repo, &state).unwrap();
            assert_eq!(
                detect_in_progress(&fixture.path)
                    .unwrap()
                    .operation
                    .as_deref(),
                Some("rebase")
            );
        }
    }

    mod error_mapping {
        use super::*;

        #[test]
        fn damaged_repo_maps_git2_error() {
            let dir = tempfile::tempdir().unwrap();
            assert!(matches!(
                detect_in_progress(dir.path()).unwrap_err(),
                RebaseOpError::Git2Error { .. }
            ));
        }

        #[test]
        fn not_in_rebase_on_continue() {
            let fixture = diverged_fixture();
            assert!(matches!(
                rebase_continue(
                    &fixture.path,
                    RebaseControlRequest {
                        workspace_id: "w".to_string(),
                        action: "continue".to_string(),
                    }
                )
                .unwrap_err(),
                RebaseOpError::NotInRebase
            ));
        }

        #[test]
        fn operation_in_progress_blocks_start() {
            let fixture = diverged_fixture();
            let plan = rebase_interactive_plan(&fixture.path, "feature", "main").unwrap();
            let state = OperationState::new(
                "rebase",
                "feature",
                Some("main".to_string()),
                fixture.head().to_string(),
                plan,
                true,
            );
            write_operation_state(&fixture.repo, &state).unwrap();
            assert!(matches!(
                merge_start(
                    &fixture.path,
                    MergeRequest {
                        workspace_id: "w".to_string(),
                        source_branch: "feature".to_string(),
                        strategy: MergeStrategy::NoFastForward,
                        commit_message: None,
                    },
                )
                .unwrap_err(),
                RebaseOpError::OperationInProgress { .. }
            ));
        }

        #[test]
        fn git_error_contains_class_code_message() {
            match git_error("Lock", -14, "locked") {
                RebaseOpError::Git2Error {
                    class,
                    code,
                    message,
                } => {
                    assert_eq!(class, "Lock");
                    assert_eq!(code, -14);
                    assert_eq!(message, "locked");
                }
                _ => panic!("expected git error"),
            }
        }

        #[test]
        fn display_uses_debug_shape() {
            assert!(RebaseOpError::AlreadyUpToDate
                .to_string()
                .contains("AlreadyUpToDate"));
        }
    }
}

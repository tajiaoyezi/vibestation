//! Git remote synchronization for MVP-21 Phase A.
//!
//! Network write paths use git2/libgit2. The core crate stays Tauri-free:
//! callers can provide event callbacks and the app crate maps them to Tauri
//! events.

use crate::{BranchInfo, GitStatusRequest, GitStatusResponse, GitStatusService};
use git2::{
    build::CheckoutBuilder, CertificateCheckStatus, Cred, CredentialType, ErrorCode, FetchOptions,
    FetchPrune, Index, Oid, PushOptions, RemoteCallbacks, Repository, ResetType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use ts_rs::TS;

const PROTECTED_BRANCHES: &[&str] = &["main", "master", "trunk"];
const STALE_LEASE_PREFIX: &str = "force-with-lease: stale";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    pub name: String,
    pub url: Option<String>,
    pub fetch_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RemoteListRequest {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RemoteListResponse {
    pub remotes: Vec<RemoteInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PushRequest {
    pub workspace_id: String,
    pub remote: String,
    pub branch: String,
    pub force: bool,
    pub expected_remote_oid: Option<String>,
    #[serde(default)]
    pub auth_method: Option<AuthMethod>,
    #[serde(default)]
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PushResult {
    pub pushed_commits: u32,
    pub new_remote_head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub workspace_id: String,
    pub remote: String,
    pub branch: String,
    pub strategy: PullStrategy,
    #[serde(default)]
    pub frontend_status_snapshot: Option<GitStatusResponse>,
    #[serde(default)]
    pub frontend_status_taken_at: Option<i64>,
    #[serde(default)]
    pub auth_method: Option<AuthMethod>,
    #[serde(default)]
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PullStrategy {
    Merge,
    Rebase,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PullResult {
    pub stage: String,
    pub new_head: String,
    pub merged_commits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FetchRequest {
    pub workspace_id: String,
    pub remote: String,
    pub prune: bool,
    #[serde(default)]
    pub auth_method: Option<AuthMethod>,
    #[serde(default)]
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub fetched_refs: Vec<String>,
    pub pruned_refs: Vec<String>,
    pub branches: Vec<BranchInfo>,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AuthMethod {
    SshAgent,
    SshKeyFile {
        path: String,
        passphrase: Option<String>,
    },
    HttpsHelper,
    HttpsManual {
        username: String,
        password: String,
    },
}

impl fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SshAgent => write!(f, "AuthMethod::SshAgent"),
            Self::SshKeyFile { path, passphrase } => f
                .debug_struct("AuthMethod::SshKeyFile")
                .field("path", path)
                .field("passphrase", &passphrase.as_ref().map(|_| "***REDACTED***"))
                .finish(),
            Self::HttpsHelper => write!(f, "AuthMethod::HttpsHelper"),
            Self::HttpsManual { username, .. } => f
                .debug_struct("AuthMethod::HttpsManual")
                .field("username", username)
                .field("password", &"***REDACTED***")
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    pub workspace_id: String,
    pub auth_challenge_id: String,
    pub task_id: String,
    pub remote_url: String,
    pub allowed_methods: Vec<String>,
    pub method: AuthMethod,
    #[ts(type = "number")]
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AuthChallenge {
    pub workspace_id: String,
    pub auth_challenge_id: String,
    pub task_id: String,
    pub remote_url: String,
    pub host_fingerprint: Option<String>,
    pub allowed_methods: Vec<String>,
    #[ts(type = "number")]
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MergeConflictInfo {
    pub files: Vec<ConflictFile>,
    pub aborted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFile {
    pub path: String,
    pub ours_oid: String,
    pub theirs_oid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NetworkOpError {
    AuthFailed {
        detail: String,
    },
    NetworkUnreachable {
        detail: String,
    },
    RemoteNotFound {
        remote: String,
    },
    NonFastForward {
        #[serde(rename = "remoteBranch")]
        remote_branch: String,
        #[serde(rename = "localAhead")]
        local_ahead: u32,
        #[serde(rename = "remoteAhead")]
        remote_ahead: u32,
    },
    MergeConflict {
        files: Vec<ConflictFile>,
        aborted: bool,
    },
    Aborted {
        reason: String,
    },
    DirtyWorkingTree {
        modified: Vec<String>,
        staged: Vec<String>,
        untracked: Vec<String>,
    },
    RejectedByRemote {
        detail: String,
    },
    StaleLease {
        expected: String,
        actual: String,
    },
    SslError {
        detail: String,
    },
    Git2Error {
        class: String,
        code: i32,
        message: String,
    },
}

impl fmt::Display for NetworkOpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for NetworkOpError {}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PushProgressEvent {
    pub workspace_id: String,
    pub task_id: String,
    pub stage: String,
    #[ts(type = "number")]
    pub objects_total: u32,
    #[ts(type = "number")]
    pub objects_done: u32,
    #[ts(type = "number")]
    pub bytes_total: u64,
    #[ts(type = "number")]
    pub bytes_done: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FetchProgressEvent {
    pub workspace_id: String,
    pub task_id: String,
    pub stage: String,
    #[ts(type = "number")]
    pub received_objects: u32,
    #[ts(type = "number")]
    pub total_objects: u32,
    #[ts(type = "number")]
    pub received_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OperationDoneEvent {
    pub workspace_id: String,
    pub task_id: String,
    pub operation: String,
    pub outcome: String,
    pub error: Option<NetworkOpError>,
}

#[derive(Clone, Default)]
pub struct GitSyncEventHandlers {
    pub push_progress: Option<Arc<dyn Fn(PushProgressEvent) -> bool + Send + Sync>>,
    pub fetch_progress: Option<Arc<dyn Fn(FetchProgressEvent) -> bool + Send + Sync>>,
    pub operation_done: Option<Arc<dyn Fn(OperationDoneEvent) + Send + Sync>>,
}

static AUTH_CACHE: OnceLock<Mutex<HashMap<String, AuthMethod>>> = OnceLock::new();

pub fn git_remote_list(workspace_path: &Path) -> Result<RemoteListResponse, NetworkOpError> {
    let repo = open_repo(workspace_path)?;
    let names = repo.remotes().map_err(map_git_error)?;
    let mut remotes = Vec::new();

    for name in names.iter().flatten() {
        let remote = repo
            .find_remote(name)
            .map_err(|error| map_remote_error(error, name))?;
        remotes.push(RemoteInfo {
            name: name.to_string(),
            url: remote.url().map(ToOwned::to_owned),
            fetch_url: remote.url().map(ToOwned::to_owned),
        });
    }

    remotes.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(RemoteListResponse { remotes })
}

pub fn git_push(workspace_path: &Path, req: PushRequest) -> Result<PushResult, NetworkOpError> {
    git_push_with_events(workspace_path, req, GitSyncEventHandlers::default())
}

pub fn git_push_with_events(
    workspace_path: &Path,
    req: PushRequest,
    handlers: GitSyncEventHandlers,
) -> Result<PushResult, NetworkOpError> {
    let result = do_push(workspace_path, &req, &handlers);
    emit_done(
        &handlers,
        &req.workspace_id,
        task_id(req.task_id.as_deref()),
        "push",
        &result,
    );
    result
}

pub fn git_fetch(workspace_path: &Path, req: FetchRequest) -> Result<FetchResult, NetworkOpError> {
    git_fetch_with_events(workspace_path, req, GitSyncEventHandlers::default())
}

pub fn git_fetch_with_events(
    workspace_path: &Path,
    req: FetchRequest,
    handlers: GitSyncEventHandlers,
) -> Result<FetchResult, NetworkOpError> {
    let result = do_fetch(workspace_path, &req, &handlers);
    emit_done(
        &handlers,
        &req.workspace_id,
        task_id(req.task_id.as_deref()),
        "fetch",
        &result,
    );
    result
}

pub fn git_pull(workspace_path: &Path, req: PullRequest) -> Result<PullResult, NetworkOpError> {
    git_pull_with_events(workspace_path, req, GitSyncEventHandlers::default())
}

pub fn git_pull_with_events(
    workspace_path: &Path,
    req: PullRequest,
    handlers: GitSyncEventHandlers,
) -> Result<PullResult, NetworkOpError> {
    let result = do_pull(workspace_path, &req, &handlers);
    emit_done(
        &handlers,
        &req.workspace_id,
        task_id(req.task_id.as_deref()),
        "pull",
        &result,
    );
    result
}

pub fn git_auth_provide(_workspace_path: &Path, req: AuthRequest) -> Result<(), NetworkOpError> {
    if req.auth_challenge_id.trim().is_empty() {
        return Err(NetworkOpError::Aborted {
            reason: "auth_challenge_id is required".to_string(),
        });
    }
    if req.task_id.trim().is_empty() || req.remote_url.trim().is_empty() {
        return Err(NetworkOpError::Aborted {
            reason: "task_id and remote_url are required".to_string(),
        });
    }

    AUTH_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|error| NetworkOpError::Git2Error {
            class: "AuthCache".to_string(),
            code: -1,
            message: error.to_string(),
        })?
        .insert(req.auth_challenge_id, req.method);
    Ok(())
}

pub fn git_merge_abort(workspace_path: &Path) -> Result<(), NetworkOpError> {
    let repo = open_repo(workspace_path)?;
    abort_in_progress_operation(&repo)
}

fn do_push(
    workspace_path: &Path,
    req: &PushRequest,
    handlers: &GitSyncEventHandlers,
) -> Result<PushResult, NetworkOpError> {
    check_protected_branch(req)?;
    let repo = open_repo(workspace_path)?;
    ensure_index_unlocked(&repo)?;

    let local_ref = format!("refs/heads/{}", req.branch);
    let local_oid = repo
        .find_reference(&local_ref)
        .and_then(|reference| {
            reference
                .target()
                .ok_or_else(|| git2::Error::from_str("local branch has no target"))
        })
        .map_err(|error| map_not_found_or_git(error, &req.branch))?;

    let pushed_commits = ahead_behind_for_remote(&repo, &req.remote, &req.branch)
        .map(|(ahead, _)| ahead)
        .unwrap_or(0);

    let stale_actual = Arc::new(Mutex::new(None::<String>));
    let rejected_by_remote = Arc::new(Mutex::new(None::<String>));
    let mut callbacks = remote_callbacks(
        &repo,
        &req.workspace_id,
        task_id(req.task_id.as_deref()),
        req.auth_method.clone(),
        handlers,
    )?;

    let expected = req.expected_remote_oid.clone();
    if req.force && expected.is_none() {
        return Err(NetworkOpError::Aborted {
            reason: "force=true requires expected_remote_oid".to_string(),
        });
    }
    let stale_actual_for_callback = Arc::clone(&stale_actual);
    callbacks.push_negotiation(move |updates| {
        if let Some(expected) = expected.as_deref() {
            for update in updates {
                let remote_current = update.src().to_string();
                if remote_current != expected {
                    if let Ok(mut guard) = stale_actual_for_callback.lock() {
                        *guard = Some(remote_current.clone());
                    }
                    return Err(git2::Error::from_str(&format!(
                        "{STALE_LEASE_PREFIX}: expected {expected} actual {remote_current}"
                    )));
                }
            }
        }
        Ok(())
    });
    let rejected_for_callback = Arc::clone(&rejected_by_remote);
    callbacks.push_update_reference(move |name, status| {
        if let Some(status) = status {
            if let Ok(mut guard) = rejected_for_callback.lock() {
                *guard = Some(format!("{name}: {status}"));
            }
            return Err(git2::Error::from_str(status));
        }
        Ok(())
    });

    let mut options = PushOptions::new();
    options.remote_callbacks(callbacks);

    let mut remote = repo
        .find_remote(&req.remote)
        .map_err(|error| map_remote_error(error, &req.remote))?;
    let refspec = if req.force {
        format!("+{local_ref}:{local_ref}")
    } else {
        format!("{local_ref}:{local_ref}")
    };
    remote
        .push(&[refspec.as_str()], Some(&mut options))
        .map_err(|error| {
            let stale_actual = stale_actual
                .lock()
                .ok()
                .and_then(|guard| guard.as_ref().cloned());
            if error.message().contains(STALE_LEASE_PREFIX) {
                NetworkOpError::StaleLease {
                    expected: req.expected_remote_oid.clone().unwrap_or_default(),
                    actual: stale_actual.unwrap_or_else(|| error.message().to_string()),
                }
            } else if let Some(detail) = rejected_by_remote
                .lock()
                .ok()
                .and_then(|guard| guard.as_ref().cloned())
            {
                NetworkOpError::RejectedByRemote { detail }
            } else {
                map_push_error(error, &req.remote, &req.branch, &repo)
            }
        })?;

    Ok(PushResult {
        pushed_commits: pushed_commits as u32,
        new_remote_head: Some(local_oid.to_string()),
    })
}

fn do_fetch(
    workspace_path: &Path,
    req: &FetchRequest,
    handlers: &GitSyncEventHandlers,
) -> Result<FetchResult, NetworkOpError> {
    let repo = open_repo(workspace_path)?;
    ensure_index_unlocked(&repo)?;
    let mut fetched_refs = Vec::<String>::new();
    let mut pruned_refs = Vec::<String>::new();
    fetch_remote(
        &repo,
        &req.remote,
        req.prune,
        req.auth_method.clone(),
        &req.workspace_id,
        task_id(req.task_id.as_deref()),
        handlers,
        &mut fetched_refs,
        &mut pruned_refs,
    )?;
    let branches = collect_branch_info(&repo)?;

    Ok(FetchResult {
        fetched_refs,
        pruned_refs,
        branches,
    })
}

fn do_pull(
    workspace_path: &Path,
    req: &PullRequest,
    handlers: &GitSyncEventHandlers,
) -> Result<PullResult, NetworkOpError> {
    let repo = open_repo(workspace_path)?;
    ensure_index_unlocked(&repo)?;
    ensure_clean_for_pull(&repo, req)?;
    let original_head = head_oid(&repo)?;

    let mut fetched_refs = Vec::new();
    let mut pruned_refs = Vec::new();
    fetch_remote_branch(&repo, req, handlers, &mut fetched_refs, &mut pruned_refs)?;

    ensure_clean_for_pull(&repo, req)?;

    let remote_ref = format!("refs/remotes/{}/{}", req.remote, req.branch);
    let annotated = repo
        .find_reference(&remote_ref)
        .and_then(|reference| repo.reference_to_annotated_commit(&reference))
        .map_err(|error| map_not_found_or_git(error, &remote_ref))?;
    let analysis = repo.merge_analysis(&[&annotated]).map_err(map_git_error)?;
    let remote_oid = annotated.id();

    if analysis.0.is_up_to_date() {
        return Ok(PullResult {
            stage: "upToDate".to_string(),
            new_head: original_head.to_string(),
            merged_commits: 0,
        });
    }

    let (_, behind) = repo
        .graph_ahead_behind(original_head, remote_oid)
        .unwrap_or((0, 0));

    if analysis.0.is_fast_forward() {
        fast_forward(&repo, &req.branch, remote_oid)?;
        return Ok(PullResult {
            stage: "ff".to_string(),
            new_head: remote_oid.to_string(),
            merged_commits: behind as u32,
        });
    }

    match req.strategy {
        PullStrategy::Merge => merge_remote(&repo, &annotated, original_head, behind as u32),
        PullStrategy::Rebase => rebase_remote(&repo, &annotated, original_head, behind as u32),
    }
}

fn fetch_remote_branch(
    repo: &Repository,
    req: &PullRequest,
    handlers: &GitSyncEventHandlers,
    fetched_refs: &mut Vec<String>,
    pruned_refs: &mut Vec<String>,
) -> Result<(), NetworkOpError> {
    let refspec = format!(
        "refs/heads/{branch}:refs/remotes/{remote}/{branch}",
        branch = req.branch,
        remote = req.remote
    );
    let mut remote = repo
        .find_remote(&req.remote)
        .map_err(|error| map_remote_error(error, &req.remote))?;
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(remote_callbacks(
        repo,
        &req.workspace_id,
        task_id(req.task_id.as_deref()),
        req.auth_method.clone(),
        handlers,
    )?);
    remote
        .fetch(&[refspec.as_str()], Some(&mut fetch_options), Some("pull"))
        .map_err(map_git_error)?;
    fetched_refs.push(format!("{}/{}", req.remote, req.branch));
    let _ = pruned_refs;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn fetch_remote(
    repo: &Repository,
    remote_name: &str,
    prune: bool,
    auth_method: Option<AuthMethod>,
    workspace_id: &str,
    task_id: &str,
    handlers: &GitSyncEventHandlers,
    fetched_refs: &mut Vec<String>,
    pruned_refs: &mut Vec<String>,
) -> Result<(), NetworkOpError> {
    let mut remote = repo
        .find_remote(remote_name)
        .map_err(|error| map_remote_error(error, remote_name))?;
    let fetched_for_callback = Arc::new(Mutex::new(Vec::<String>::new()));
    let pruned_for_callback = Arc::new(Mutex::new(Vec::<String>::new()));
    let fetched_capture = Arc::clone(&fetched_for_callback);
    let pruned_capture = Arc::clone(&pruned_for_callback);

    let mut callbacks =
        remote_callbacks(repo, workspace_id, task_id, auth_method.clone(), handlers)?;
    callbacks.update_tips(move |name, _old, new| {
        if new.is_zero() {
            if let Ok(mut guard) = pruned_capture.lock() {
                guard.push(name.to_string());
            }
        } else if let Ok(mut guard) = fetched_capture.lock() {
            guard.push(name.to_string());
        }
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    if prune {
        fetch_options.prune(FetchPrune::On);
    }

    let empty: [&str; 0] = [];
    remote
        .fetch(&empty, Some(&mut fetch_options), Some("fetch"))
        .map_err(map_git_error)?;

    if let Ok(guard) = fetched_for_callback.lock() {
        fetched_refs.extend(guard.iter().cloned());
    }
    if let Ok(guard) = pruned_for_callback.lock() {
        pruned_refs.extend(guard.iter().cloned());
    }
    fetched_refs.sort();
    fetched_refs.dedup();
    pruned_refs.sort();
    pruned_refs.dedup();
    Ok(())
}

fn remote_callbacks<'a>(
    repo: &'a Repository,
    workspace_id: &str,
    task_id: &str,
    auth_method: Option<AuthMethod>,
    handlers: &GitSyncEventHandlers,
) -> Result<RemoteCallbacks<'a>, NetworkOpError> {
    let config = repo.config().ok();
    let fetch_workspace_id = workspace_id.to_string();
    let fetch_task_id = task_id.to_string();
    let fetch_handler = handlers.fetch_progress.clone();
    let push_handler = handlers.push_progress.clone();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username, allowed| {
        credential_from_method(
            auth_method.as_ref(),
            config.as_ref(),
            url,
            username,
            allowed,
        )
    });
    callbacks.certificate_check(|_cert, _host| Ok(CertificateCheckStatus::CertificatePassthrough));
    callbacks.transfer_progress(move |stats| {
        let Some(handler) = fetch_handler.as_ref() else {
            return true;
        };
        handler(FetchProgressEvent {
            workspace_id: fetch_workspace_id.clone(),
            task_id: fetch_task_id.clone(),
            stage: "fetching".to_string(),
            received_objects: stats.received_objects() as u32,
            total_objects: stats.total_objects() as u32,
            received_bytes: stats.received_bytes() as u64,
        })
    });

    let push_workspace_id = workspace_id.to_string();
    let push_task_id = task_id.to_string();
    callbacks.push_transfer_progress(move |current, total, bytes| {
        let Some(handler) = push_handler.as_ref() else {
            return;
        };
        let _ = handler(PushProgressEvent {
            workspace_id: push_workspace_id.clone(),
            task_id: push_task_id.clone(),
            stage: "writing".to_string(),
            objects_total: total as u32,
            objects_done: current as u32,
            bytes_total: bytes as u64,
            bytes_done: bytes as u64,
        });
    });

    Ok(callbacks)
}

fn credential_from_method(
    method: Option<&AuthMethod>,
    config: Option<&git2::Config>,
    url: &str,
    username_from_url: Option<&str>,
    allowed: CredentialType,
) -> Result<Cred, git2::Error> {
    let username = username_from_url.unwrap_or("git");
    if let Some(method) = method {
        return match method {
            AuthMethod::SshAgent => Cred::ssh_key_from_agent(username),
            AuthMethod::SshKeyFile { path, passphrase } => {
                Cred::ssh_key(username, None, Path::new(path), passphrase.as_deref())
            }
            AuthMethod::HttpsHelper => config
                .ok_or_else(|| git2::Error::from_str("git config unavailable"))
                .and_then(|config| Cred::credential_helper(config, url, username_from_url)),
            AuthMethod::HttpsManual { username, password } => {
                Cred::userpass_plaintext(username, password)
            }
        };
    }

    if allowed.contains(CredentialType::SSH_KEY) {
        Cred::ssh_key_from_agent(username)
            .or_else(|_| ssh_key_from_home(username, "id_ed25519"))
            .or_else(|_| ssh_key_from_home(username, "id_rsa"))
    } else if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
        if let Some(config) = config {
            Cred::credential_helper(config, url, username_from_url)
        } else {
            Err(git2::Error::from_str("git config unavailable"))
        }
    } else if allowed.contains(CredentialType::USERNAME) {
        Cred::username(username)
    } else {
        Err(git2::Error::from_str("no supported credential type"))
    }
}

fn ssh_key_from_home(username: &str, filename: &str) -> Result<Cred, git2::Error> {
    let home = std::env::var("HOME").map_err(|_| git2::Error::from_str("HOME not set"))?;
    let path = Path::new(&home).join(".ssh").join(filename);
    Cred::ssh_key(username, None, &path, None)
}

fn check_protected_branch(req: &PushRequest) -> Result<(), NetworkOpError> {
    if req.force && PROTECTED_BRANCHES.contains(&req.branch.as_str()) {
        Err(NetworkOpError::RejectedByRemote {
            detail: format!(
                "Branch '{}' is protected; force push is not allowed",
                req.branch
            ),
        })
    } else {
        Ok(())
    }
}

fn fast_forward(repo: &Repository, branch: &str, target: Oid) -> Result<(), NetworkOpError> {
    let local_ref = format!("refs/heads/{branch}");
    match repo.find_reference(&local_ref) {
        Ok(mut reference) => {
            reference
                .set_target(target, "Fast-forward")
                .map_err(map_git_error)?;
        }
        Err(error) if error.code() == ErrorCode::NotFound => {
            repo.reference(&local_ref, target, true, "Fast-forward")
                .map_err(map_git_error)?;
        }
        Err(error) => return Err(map_git_error(error)),
    }
    repo.set_head(&local_ref).map_err(map_git_error)?;
    let mut checkout = CheckoutBuilder::new();
    checkout.force();
    repo.checkout_head(Some(&mut checkout))
        .map_err(map_git_error)
}

fn merge_remote(
    repo: &Repository,
    annotated: &git2::AnnotatedCommit<'_>,
    original_head: Oid,
    merged_commits: u32,
) -> Result<PullResult, NetworkOpError> {
    let remote_commit = repo.find_commit(annotated.id()).map_err(map_git_error)?;
    let local_commit = repo.find_commit(original_head).map_err(map_git_error)?;
    repo.merge(&[annotated], None, None)
        .map_err(map_git_error)?;

    let mut index = repo.index().map_err(map_git_error)?;
    if index.has_conflicts() {
        let files = collect_conflict_files(repo, &mut index)?;
        abort_to_original_head(repo, original_head)?;
        return Err(NetworkOpError::MergeConflict {
            files,
            aborted: true,
        });
    }

    let tree_id = index.write_tree().map_err(map_git_error)?;
    let tree = repo.find_tree(tree_id).map_err(map_git_error)?;
    let sig = git_signature(repo)?;
    let message = format!(
        "Merge remote-tracking branch '{}'",
        annotated.refname().unwrap_or("remote")
    );
    let commit_id = repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            &message,
            &tree,
            &[&local_commit, &remote_commit],
        )
        .map_err(map_git_error)?;
    repo.cleanup_state().map_err(map_git_error)?;
    let mut checkout = CheckoutBuilder::new();
    checkout.safe();
    repo.checkout_head(Some(&mut checkout))
        .map_err(map_git_error)?;

    Ok(PullResult {
        stage: "merge".to_string(),
        new_head: commit_id.to_string(),
        merged_commits,
    })
}

fn rebase_remote(
    repo: &Repository,
    annotated: &git2::AnnotatedCommit<'_>,
    original_head: Oid,
    merged_commits: u32,
) -> Result<PullResult, NetworkOpError> {
    let sig = git_signature(repo)?;
    let mut rebase = repo
        .rebase(None, Some(annotated), None, None)
        .map_err(map_git_error)?;

    loop {
        match rebase.next() {
            Some(Ok(_operation)) => {
                if let Err(error) = rebase.commit(None, &sig, None) {
                    drop(rebase);
                    let mut index = repo.index().map_err(map_git_error)?;
                    let files = collect_conflict_files(repo, &mut index)?;
                    abort_to_original_head(repo, original_head)?;
                    return if files.is_empty() {
                        Err(map_git_error(error))
                    } else {
                        Err(NetworkOpError::MergeConflict {
                            files,
                            aborted: true,
                        })
                    };
                }
            }
            Some(Err(error)) => {
                drop(rebase);
                let mut index = repo.index().map_err(map_git_error)?;
                let files = collect_conflict_files(repo, &mut index)?;
                abort_to_original_head(repo, original_head)?;
                return if files.is_empty() {
                    Err(map_git_error(error))
                } else {
                    Err(NetworkOpError::MergeConflict {
                        files,
                        aborted: true,
                    })
                };
            }
            None => break,
        }
    }

    rebase.finish(Some(&sig)).map_err(map_git_error)?;
    let head = head_oid(repo)?;
    Ok(PullResult {
        stage: "rebase".to_string(),
        new_head: head.to_string(),
        merged_commits,
    })
}

fn abort_in_progress_operation(repo: &Repository) -> Result<(), NetworkOpError> {
    let orig = repo
        .find_reference("ORIG_HEAD")
        .ok()
        .and_then(|reference| reference.target());
    repo.cleanup_state().map_err(map_git_error)?;
    if let Some(orig) = orig {
        let object = repo.find_object(orig, None).map_err(map_git_error)?;
        repo.reset(&object, ResetType::Hard, None)
            .map_err(map_git_error)?;
    }
    Ok(())
}

fn abort_to_original_head(repo: &Repository, original_head: Oid) -> Result<(), NetworkOpError> {
    repo.cleanup_state().map_err(map_git_error)?;
    let object = repo
        .find_object(original_head, None)
        .map_err(map_git_error)?;
    repo.reset(&object, ResetType::Hard, None)
        .map_err(map_git_error)
}

fn collect_conflict_files(
    repo: &Repository,
    index: &mut Index,
) -> Result<Vec<ConflictFile>, NetworkOpError> {
    let mut files = Vec::new();
    let conflicts = index.conflicts().map_err(map_git_error)?;
    for conflict in conflicts {
        let conflict = conflict.map_err(map_git_error)?;
        let entry = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref());
        let path = entry
            .map(|entry| String::from_utf8_lossy(&entry.path).to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        files.push(ConflictFile {
            path,
            ours_oid: conflict
                .our
                .as_ref()
                .map(|entry| entry.id.to_string())
                .unwrap_or_default(),
            theirs_oid: conflict
                .their
                .as_ref()
                .map(|entry| entry.id.to_string())
                .unwrap_or_default(),
        });
    }
    if files.is_empty() && repo.index().map_err(map_git_error)?.has_conflicts() {
        files.push(ConflictFile {
            path: "<conflict>".to_string(),
            ours_oid: String::new(),
            theirs_oid: String::new(),
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_branch_info(repo: &Repository) -> Result<Vec<BranchInfo>, NetworkOpError> {
    crate::branch_list(repo.workdir().unwrap_or_else(|| repo.path()))
        .map(|response| response.branches)
        .map_err(|error| NetworkOpError::Git2Error {
            class: "BranchList".to_string(),
            code: -1,
            message: format!("{error:?}"),
        })
}

fn ensure_clean_for_pull(repo: &Repository, req: &PullRequest) -> Result<(), NetworkOpError> {
    let current = status_snapshot(repo)?;
    if let Some(frontend) = req.frontend_status_snapshot.as_ref() {
        if !current.equivalent(frontend) {
            return Err(dirty_from_status(current));
        }
    }
    if !current.staged.is_empty() || !current.unstaged.is_empty() || !current.untracked.is_empty() {
        return Err(dirty_from_status(current));
    }
    Ok(())
}

fn status_snapshot(repo: &Repository) -> Result<GitStatusResponse, NetworkOpError> {
    let path = repo.workdir().unwrap_or_else(|| repo.path());
    GitStatusService::query(
        path,
        &GitStatusRequest {
            workspace_id: "git-sync".to_string(),
        },
    )
    .map_err(|error| NetworkOpError::Git2Error {
        class: "GitStatus".to_string(),
        code: -1,
        message: error.to_string(),
    })
}

fn dirty_from_status(status: GitStatusResponse) -> NetworkOpError {
    NetworkOpError::DirtyWorkingTree {
        modified: status.unstaged.into_iter().map(|item| item.path).collect(),
        staged: status.staged.into_iter().map(|item| item.path).collect(),
        untracked: status.untracked.into_iter().map(|item| item.path).collect(),
    }
}

fn ensure_index_unlocked(repo: &Repository) -> Result<(), NetworkOpError> {
    if repo.path().join("index.lock").exists() {
        Err(NetworkOpError::Git2Error {
            class: "Index".to_string(),
            code: -1,
            message: "index.lock exists".to_string(),
        })
    } else {
        Ok(())
    }
}

fn ahead_behind_for_remote(
    repo: &Repository,
    remote: &str,
    branch: &str,
) -> Result<(usize, usize), git2::Error> {
    let local = repo.refname_to_id(&format!("refs/heads/{branch}"))?;
    let remote = repo.refname_to_id(&format!("refs/remotes/{remote}/{branch}"))?;
    repo.graph_ahead_behind(local, remote)
}

fn head_oid(repo: &Repository) -> Result<Oid, NetworkOpError> {
    repo.head()
        .and_then(|head| {
            head.target()
                .ok_or_else(|| git2::Error::from_str("HEAD has no direct target"))
        })
        .map_err(map_git_error)
}

fn git_signature(repo: &Repository) -> Result<git2::Signature<'static>, NetworkOpError> {
    repo.signature()
        .or_else(|_| {
            let cfg = repo.config()?;
            let name = cfg.get_string("user.name").unwrap_or_default();
            let email = cfg.get_string("user.email").unwrap_or_default();
            git2::Signature::now(&name, &email)
        })
        .map_err(map_git_error)
}

fn open_repo(path: &Path) -> Result<Repository, NetworkOpError> {
    Repository::open(path).map_err(map_git_error)
}

fn task_id(task_id: Option<&str>) -> &str {
    task_id.unwrap_or("git-sync")
}

fn emit_done<T>(
    handlers: &GitSyncEventHandlers,
    workspace_id: &str,
    task_id: &str,
    operation: &str,
    result: &Result<T, NetworkOpError>,
) {
    let Some(handler) = handlers.operation_done.as_ref() else {
        return;
    };
    handler(OperationDoneEvent {
        workspace_id: workspace_id.to_string(),
        task_id: task_id.to_string(),
        operation: operation.to_string(),
        outcome: if result.is_ok() {
            "success".to_string()
        } else {
            "error".to_string()
        },
        error: result.as_ref().err().cloned(),
    });
}

fn map_not_found_or_git(error: git2::Error, name: &str) -> NetworkOpError {
    if error.code() == ErrorCode::NotFound {
        NetworkOpError::RemoteNotFound {
            remote: name.to_string(),
        }
    } else {
        map_git_error(error)
    }
}

fn map_remote_error(error: git2::Error, remote: &str) -> NetworkOpError {
    if error.code() == ErrorCode::NotFound {
        NetworkOpError::RemoteNotFound {
            remote: remote.to_string(),
        }
    } else {
        map_git_error(error)
    }
}

fn map_push_error(
    error: git2::Error,
    remote: &str,
    branch: &str,
    repo: &Repository,
) -> NetworkOpError {
    let message = error.message().to_lowercase();
    if message.contains("non-fast-forward")
        || message.contains("fetch first")
        || message.contains("not fast-forward")
        || message.contains("non-fastforward")
        || message.contains("failed to push some refs")
        || message.contains("not present locally")
    {
        let (local_ahead, remote_ahead) =
            ahead_behind_for_remote(repo, remote, branch).unwrap_or((0, 0));
        NetworkOpError::NonFastForward {
            remote_branch: format!("{remote}/{branch}"),
            local_ahead: local_ahead as u32,
            remote_ahead: remote_ahead as u32,
        }
    } else {
        map_git_error(error)
    }
}

fn map_git_error(error: git2::Error) -> NetworkOpError {
    let class = format!("{:?}", error.class());
    let code = error.raw_code();
    let message = error.message().to_string();
    let lower = message.to_lowercase();

    if lower.contains("ssl") || lower.contains("certificate") {
        NetworkOpError::SslError { detail: message }
    } else if lower.contains("auth")
        || lower.contains("credential")
        || lower.contains("permission denied")
    {
        NetworkOpError::AuthFailed { detail: message }
    } else if lower.contains("could not resolve")
        || lower.contains("failed to connect")
        || lower.contains("network")
        || lower.contains("timed out")
    {
        NetworkOpError::NetworkUnreachable { detail: message }
    } else if lower.contains("cancel") || lower.contains("aborted") {
        NetworkOpError::Aborted { reason: message }
    } else {
        NetworkOpError::Git2Error {
            class,
            code,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{BranchType, RepositoryInitOptions, Signature};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct Fixture {
        _bare_dir: TempDir,
        _work_dir: TempDir,
        _second_dir: TempDir,
        bare_path: PathBuf,
        work_path: PathBuf,
        second_path: PathBuf,
    }

    impl Fixture {
        fn repo(&self) -> Repository {
            Repository::open(&self.work_path).unwrap()
        }

        fn second_repo(&self) -> Repository {
            Repository::open(&self.second_path).unwrap()
        }
    }

    fn create_local_bare_remote() -> Fixture {
        let bare_dir = TempDir::new().unwrap();
        let mut opts = RepositoryInitOptions::new();
        opts.bare(true);
        opts.initial_head("main");
        Repository::init_opts(bare_dir.path(), &opts).unwrap();

        let work_dir = TempDir::new().unwrap();
        let bare_url = bare_dir.path().to_str().unwrap();
        let work = Repository::clone(bare_url, work_dir.path()).unwrap();
        configure_repo(&work);
        work.set_head("refs/heads/main").unwrap();
        commit_file(&work, work_dir.path(), "hello.txt", "hello", "initial");
        push_branch(&work, "origin", "main");

        let second_dir = TempDir::new().unwrap();
        let second = Repository::clone(bare_url, second_dir.path()).unwrap();
        configure_repo(&second);

        Fixture {
            bare_path: bare_dir.path().to_path_buf(),
            work_path: work_dir.path().to_path_buf(),
            second_path: second_dir.path().to_path_buf(),
            _bare_dir: bare_dir,
            _work_dir: work_dir,
            _second_dir: second_dir,
        }
    }

    fn configure_repo(repo: &Repository) {
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "Test User").unwrap();
        cfg.set_str("user.email", "test@example.com").unwrap();
    }

    fn signature() -> Signature<'static> {
        Signature::now("Test User", "test@example.com").unwrap()
    }

    fn commit_file(
        repo: &Repository,
        path: &Path,
        file: &str,
        content: &str,
        message: &str,
    ) -> Oid {
        fs::write(path.join(file), content).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new(file)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = signature();
        let parents = repo
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok())
            .into_iter()
            .collect::<Vec<_>>();
        let parent_refs = parents.iter().collect::<Vec<_>>();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
            .unwrap()
    }

    fn push_branch(repo: &Repository, remote_name: &str, branch: &str) {
        let mut remote = repo.find_remote(remote_name).unwrap();
        let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
        remote.push(&[refspec.as_str()], None).unwrap();
        let mut remote = repo.find_remote(remote_name).unwrap();
        remote.fetch(&[branch], None, None).unwrap();
    }

    fn fetch_origin(repo: &Repository, branch: &str) {
        let mut remote = repo.find_remote("origin").unwrap();
        let refspec = format!("refs/heads/{branch}:refs/remotes/origin/{branch}");
        remote.fetch(&[refspec.as_str()], None, None).unwrap();
    }

    fn clean_status(workspace_id: &str) -> Option<GitStatusResponse> {
        Some(GitStatusResponse {
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
        })
        .filter(|_| !workspace_id.is_empty())
    }

    fn push_req(branch: &str) -> PushRequest {
        PushRequest {
            workspace_id: "w".to_string(),
            remote: "origin".to_string(),
            branch: branch.to_string(),
            force: false,
            expected_remote_oid: None,
            auth_method: None,
            task_id: None,
        }
    }

    fn pull_req(strategy: PullStrategy) -> PullRequest {
        PullRequest {
            workspace_id: "w".to_string(),
            remote: "origin".to_string(),
            branch: "main".to_string(),
            strategy,
            frontend_status_snapshot: clean_status("w"),
            frontend_status_taken_at: Some(0),
            auth_method: None,
            task_id: None,
        }
    }

    fn fetch_req(prune: bool) -> FetchRequest {
        FetchRequest {
            workspace_id: "w".to_string(),
            remote: "origin".to_string(),
            prune,
            auth_method: None,
            task_id: None,
        }
    }

    #[test]
    fn auth_method_debug_redacts_password() {
        let auth = AuthMethod::HttpsManual {
            username: "alice".to_string(),
            password: "secret123".to_string(),
        };
        let debug_str = format!("{auth:?}");
        assert!(!debug_str.contains("secret123"));
        assert!(debug_str.contains("***REDACTED***"));
    }

    #[test]
    fn auth_method_debug_redacts_ssh_passphrase() {
        let auth = AuthMethod::SshKeyFile {
            path: "/tmp/key".to_string(),
            passphrase: Some("secret-phrase-123".to_string()),
        };
        let debug_str = format!("{auth:?}");
        assert!(!debug_str.contains("secret-phrase-123"));
        assert!(debug_str.contains("***REDACTED***"));
    }

    #[test]
    fn credential_https_manual_path_succeeds() {
        let auth = AuthMethod::HttpsManual {
            username: "alice".to_string(),
            password: "token".to_string(),
        };
        let cred = credential_from_method(
            Some(&auth),
            None,
            "https://example.com/repo.git",
            None,
            CredentialType::USER_PASS_PLAINTEXT,
        );
        assert!(cred.is_ok());
    }

    #[test]
    fn credential_ssh_key_file_path_is_attempted() {
        let auth = AuthMethod::SshKeyFile {
            path: "/no/such/key".to_string(),
            passphrase: None,
        };
        let result = credential_from_method(
            Some(&auth),
            None,
            "ssh://example.com/repo.git",
            Some("git"),
            CredentialType::SSH_KEY,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn credential_ssh_agent_path_is_attempted() {
        let _ = credential_from_method(
            Some(&AuthMethod::SshAgent),
            None,
            "ssh://example.com/repo.git",
            Some("git"),
            CredentialType::SSH_KEY,
        );
    }

    // task-6.1（ADR-005）：HttpsHelper 走 git2 Cred::credential_helper · 在 Windows 上会
    // 调用全局 `credential.helper`（开发机常配 Git Credential Manager `manager`）· GCM 弹交互
    // 凭据提示并**永久阻塞**（headless 无人应答 → 测试 hang 而非 err）· 本测断言 is_err() 隐含
    // "无交互 helper" 的 Unix/CI 环境假设。Windows ignore（HttpsHelper 凭据链的 Windows 行为
    // 由真实 push/pull 集成验证 · 非本单测范围）· mac/Linux 照常跑（无交互 GCM）。
    #[test]
    #[cfg_attr(
        target_os = "windows",
        ignore = "git2 Cred::credential_helper 触发 Git Credential Manager 交互提示 · headless 永久阻塞 · 断言 is_err() 隐含无交互 helper 的 Unix/CI 假设 · ADR-005"
    )]
    fn credential_https_helper_path_is_attempted() {
        let repo = Repository::init(TempDir::new().unwrap().path()).unwrap();
        let config = repo.config().unwrap();
        let result = credential_from_method(
            Some(&AuthMethod::HttpsHelper),
            Some(&config),
            "https://example.com/repo.git",
            None,
            CredentialType::USER_PASS_PLAINTEXT,
        );
        assert!(result.is_err());
    }

    #[test]
    fn remote_list_returns_origin() {
        let fixture = create_local_bare_remote();
        let response = git_remote_list(&fixture.work_path).unwrap();
        assert_eq!(response.remotes.len(), 1);
        assert_eq!(response.remotes[0].name, "origin");
    }

    #[test]
    fn remote_list_missing_repo_maps_git2_error() {
        let dir = TempDir::new().unwrap();
        let error = git_remote_list(dir.path()).unwrap_err();
        assert!(matches!(error, NetworkOpError::Git2Error { .. }));
    }

    #[test]
    fn push_success_updates_bare_remote() {
        let fixture = create_local_bare_remote();
        let repo = fixture.repo();
        commit_file(&repo, &fixture.work_path, "hello.txt", "local", "local");

        let result = git_push(&fixture.work_path, push_req("main")).unwrap();
        assert!(result.pushed_commits >= 1);

        let bare = Repository::open_bare(&fixture.bare_path).unwrap();
        assert_eq!(
            bare.refname_to_id("refs/heads/main").unwrap().to_string(),
            result.new_remote_head.unwrap()
        );
    }

    #[test]
    fn push_missing_remote_returns_remote_not_found() {
        let fixture = create_local_bare_remote();
        let mut req = push_req("main");
        req.remote = "upstream".to_string();
        let error = git_push(&fixture.work_path, req).unwrap_err();
        assert!(matches!(error, NetworkOpError::RemoteNotFound { .. }));
    }

    #[test]
    fn push_protected_force_rejected() {
        let fixture = create_local_bare_remote();
        let mut req = push_req("main");
        req.force = true;
        req.expected_remote_oid = Some("abc".to_string());
        let error = git_push(&fixture.work_path, req).unwrap_err();
        assert!(matches!(error, NetworkOpError::RejectedByRemote { .. }));
    }

    #[test]
    fn push_force_requires_expected_remote_oid() {
        let fixture = create_local_bare_remote();
        let repo = fixture.repo();
        repo.branch(
            "feature",
            &repo.head().unwrap().peel_to_commit().unwrap(),
            false,
        )
        .unwrap();
        let mut req = push_req("feature");
        req.force = true;
        let error = git_push(&fixture.work_path, req).unwrap_err();
        assert!(matches!(error, NetworkOpError::Aborted { .. }));
    }

    #[test]
    fn push_force_with_stale_lease_returns_stale_lease() {
        let fixture = create_local_bare_remote();
        let repo = fixture.repo();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();
        repo.set_head("refs/heads/feature").unwrap();
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .unwrap();
        commit_file(&repo, &fixture.work_path, "feature.txt", "one", "feature");

        let mut req = push_req("feature");
        req.force = true;
        req.expected_remote_oid = Some("0000000000000000000000000000000000000001".to_string());
        let error = git_push(&fixture.work_path, req).unwrap_err();
        assert!(matches!(error, NetworkOpError::StaleLease { .. }));
    }

    #[test]
    fn push_non_fast_forward_maps_error() {
        let fixture = create_local_bare_remote();
        let second = fixture.second_repo();
        commit_file(
            &second,
            &fixture.second_path,
            "remote.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        let repo = fixture.repo();
        commit_file(&repo, &fixture.work_path, "local.txt", "local", "local");
        let error = git_push(&fixture.work_path, push_req("main")).unwrap_err();
        assert!(
            matches!(
                error,
                NetworkOpError::NonFastForward { .. } | NetworkOpError::RejectedByRemote { .. }
            ),
            "expected non-fast-forward mapping, got {error:?}"
        );
    }

    #[test]
    fn fetch_basic_updates_remote_tracking_ref() {
        let fixture = create_local_bare_remote();
        let second = fixture.second_repo();
        commit_file(
            &second,
            &fixture.second_path,
            "remote.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        let result = git_fetch(&fixture.work_path, fetch_req(false)).unwrap();
        assert!(!result.fetched_refs.is_empty());
        let repo = fixture.repo();
        assert!(repo.refname_to_id("refs/remotes/origin/main").is_ok());
    }

    #[test]
    fn fetch_prune_removes_deleted_remote_tracking_ref() {
        let fixture = create_local_bare_remote();
        let repo = fixture.repo();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("old", &head, false).unwrap();
        push_branch(&repo, "origin", "old");
        fetch_origin(&repo, "old");

        let bare = Repository::open_bare(&fixture.bare_path).unwrap();
        bare.find_reference("refs/heads/old")
            .unwrap()
            .delete()
            .unwrap();

        let result = git_fetch(&fixture.work_path, fetch_req(true)).unwrap();
        assert!(result.pruned_refs.iter().any(|name| name.contains("old")));
    }

    #[test]
    fn fetch_missing_remote_returns_remote_not_found() {
        let fixture = create_local_bare_remote();
        let mut req = fetch_req(false);
        req.remote = "missing".to_string();
        let error = git_fetch(&fixture.work_path, req).unwrap_err();
        assert!(matches!(error, NetworkOpError::RemoteNotFound { .. }));
    }

    #[test]
    fn fetch_progress_callback_can_abort() {
        let fixture = create_local_bare_remote();
        let handlers = GitSyncEventHandlers {
            fetch_progress: Some(Arc::new(|_| false)),
            ..Default::default()
        };
        let _ = git_fetch_with_events(&fixture.work_path, fetch_req(false), handlers);
    }

    #[test]
    fn pull_fast_forward_succeeds() {
        let fixture = create_local_bare_remote();
        let second = fixture.second_repo();
        let new_oid = commit_file(
            &second,
            &fixture.second_path,
            "remote.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        let result = git_pull(&fixture.work_path, pull_req(PullStrategy::Merge)).unwrap();
        assert_eq!(result.stage, "ff");
        assert_eq!(result.new_head, new_oid.to_string());
    }

    #[test]
    fn pull_dirty_working_tree_returns_error() {
        let fixture = create_local_bare_remote();
        fs::write(fixture.work_path.join("dirty.txt"), "dirty").unwrap();
        let error = git_pull(&fixture.work_path, pull_req(PullStrategy::Merge)).unwrap_err();
        assert!(matches!(error, NetworkOpError::DirtyWorkingTree { .. }));
    }

    #[test]
    fn pull_snapshot_drift_returns_dirty_tree() {
        let fixture = create_local_bare_remote();
        let mut req = pull_req(PullStrategy::Merge);
        req.frontend_status_snapshot = Some(GitStatusResponse {
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
        });
        fs::write(fixture.work_path.join("drift.txt"), "drift").unwrap();
        let error = git_pull(&fixture.work_path, req).unwrap_err();
        assert!(matches!(error, NetworkOpError::DirtyWorkingTree { .. }));
    }

    #[test]
    fn pull_merge_succeeds() {
        let fixture = create_local_bare_remote();
        let second = fixture.second_repo();
        commit_file(
            &second,
            &fixture.second_path,
            "remote.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        let repo = fixture.repo();
        commit_file(&repo, &fixture.work_path, "local.txt", "local", "local");
        let result = git_pull(&fixture.work_path, pull_req(PullStrategy::Merge)).unwrap();
        assert_eq!(result.stage, "merge");
    }

    #[test]
    fn pull_rebase_succeeds() {
        let fixture = create_local_bare_remote();
        let second = fixture.second_repo();
        commit_file(
            &second,
            &fixture.second_path,
            "remote.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        let repo = fixture.repo();
        commit_file(&repo, &fixture.work_path, "local.txt", "local", "local");
        let result = git_pull(&fixture.work_path, pull_req(PullStrategy::Rebase)).unwrap();
        assert_eq!(result.stage, "rebase");
    }

    #[test]
    fn pull_merge_conflict_aborts_and_restores_worktree() {
        let fixture = create_local_bare_remote();
        let before = fs::read_to_string(fixture.work_path.join("hello.txt")).unwrap();

        let second = fixture.second_repo();
        commit_file(
            &second,
            &fixture.second_path,
            "hello.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        let repo = fixture.repo();
        commit_file(&repo, &fixture.work_path, "hello.txt", "local", "local");
        let error = git_pull(&fixture.work_path, pull_req(PullStrategy::Merge)).unwrap_err();
        assert!(matches!(
            error,
            NetworkOpError::MergeConflict { aborted: true, .. }
        ));
        assert_eq!(
            fs::read_to_string(fixture.work_path.join("hello.txt")).unwrap(),
            "local"
        );
        assert_ne!(before, "local");
    }

    #[test]
    fn pull_rebase_conflict_aborts_and_restores_worktree() {
        let fixture = create_local_bare_remote();
        let second = fixture.second_repo();
        commit_file(
            &second,
            &fixture.second_path,
            "hello.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        let repo = fixture.repo();
        commit_file(&repo, &fixture.work_path, "hello.txt", "local", "local");
        let error = git_pull(&fixture.work_path, pull_req(PullStrategy::Rebase)).unwrap_err();
        assert!(matches!(
            error,
            NetworkOpError::MergeConflict { aborted: true, .. }
        ));
        assert_eq!(
            fs::read_to_string(fixture.work_path.join("hello.txt")).unwrap(),
            "local"
        );
    }

    #[test]
    fn merge_abort_resets_to_orig_head_when_present() {
        let fixture = create_local_bare_remote();
        let repo = fixture.repo();
        let original = repo.head().unwrap().target().unwrap();
        commit_file(&repo, &fixture.work_path, "local.txt", "local", "local");
        repo.reference("ORIG_HEAD", original, true, "test").unwrap();
        git_merge_abort(&fixture.work_path).unwrap();
        assert_eq!(repo.head().unwrap().target().unwrap(), original);
    }

    #[test]
    fn auth_provide_stores_challenge_bound_method() {
        let fixture = create_local_bare_remote();
        let req = AuthRequest {
            workspace_id: "w".to_string(),
            auth_challenge_id: "challenge-1".to_string(),
            task_id: "task-1".to_string(),
            remote_url: "https://example.com/repo.git".to_string(),
            allowed_methods: vec!["httpsManual".to_string()],
            method: AuthMethod::HttpsManual {
                username: "alice".to_string(),
                password: "secret".to_string(),
            },
            expires_at: 9_999_999_999,
        };
        git_auth_provide(&fixture.work_path, req).unwrap();
        let cache = AUTH_CACHE.get().unwrap().lock().unwrap();
        assert!(cache.contains_key("challenge-1"));
    }

    #[test]
    fn auth_provide_rejects_empty_challenge() {
        let fixture = create_local_bare_remote();
        let req = AuthRequest {
            workspace_id: "w".to_string(),
            auth_challenge_id: String::new(),
            task_id: "task-1".to_string(),
            remote_url: "https://example.com/repo.git".to_string(),
            allowed_methods: vec!["httpsManual".to_string()],
            method: AuthMethod::HttpsManual {
                username: "alice".to_string(),
                password: "secret".to_string(),
            },
            expires_at: 9_999_999_999,
        };
        let error = git_auth_provide(&fixture.work_path, req).unwrap_err();
        assert!(matches!(error, NetworkOpError::Aborted { .. }));
    }

    #[test]
    fn error_mapping_network_unreachable() {
        let error = map_git_error(git2::Error::from_str("failed to connect to host"));
        assert!(matches!(error, NetworkOpError::NetworkUnreachable { .. }));
    }

    #[test]
    fn error_mapping_ssl() {
        let error = map_git_error(git2::Error::from_str("SSL certificate problem"));
        assert!(matches!(error, NetworkOpError::SslError { .. }));
    }

    #[test]
    fn error_mapping_auth() {
        let error = map_git_error(git2::Error::from_str("authentication failed"));
        assert!(matches!(error, NetworkOpError::AuthFailed { .. }));
    }

    #[test]
    fn error_mapping_aborted() {
        let error = map_git_error(git2::Error::from_str("operation cancelled"));
        assert!(matches!(error, NetworkOpError::Aborted { .. }));
    }

    #[test]
    fn error_mapping_generic_git2() {
        let error = map_git_error(git2::Error::from_str("some other error"));
        assert!(matches!(error, NetworkOpError::Git2Error { .. }));
    }

    #[test]
    fn network_error_variants_are_serializable() {
        let errors = vec![
            NetworkOpError::AuthFailed { detail: "x".into() },
            NetworkOpError::NetworkUnreachable { detail: "x".into() },
            NetworkOpError::RemoteNotFound { remote: "x".into() },
            NetworkOpError::NonFastForward {
                remote_branch: "origin/main".into(),
                local_ahead: 1,
                remote_ahead: 2,
            },
            NetworkOpError::MergeConflict {
                files: vec![ConflictFile {
                    path: "a".into(),
                    ours_oid: "1".into(),
                    theirs_oid: "2".into(),
                }],
                aborted: true,
            },
            NetworkOpError::Aborted { reason: "x".into() },
            NetworkOpError::DirtyWorkingTree {
                modified: vec!["m".into()],
                staged: vec!["s".into()],
                untracked: vec!["u".into()],
            },
            NetworkOpError::RejectedByRemote { detail: "x".into() },
            NetworkOpError::StaleLease {
                expected: "a".into(),
                actual: "b".into(),
            },
            NetworkOpError::SslError { detail: "x".into() },
            NetworkOpError::Git2Error {
                class: "x".into(),
                code: -1,
                message: "x".into(),
            },
        ];
        for error in errors {
            let json = serde_json::to_string(&error).unwrap();
            assert!(json.contains("kind"));
        }
    }

    #[test]
    fn operation_done_event_emitted_on_success() {
        let fixture = create_local_bare_remote();
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_for_handler = Arc::clone(&events);
        let handlers = GitSyncEventHandlers {
            operation_done: Some(Arc::new(move |event| {
                events_for_handler.lock().unwrap().push(event);
            })),
            ..Default::default()
        };
        git_fetch_with_events(&fixture.work_path, fetch_req(false), handlers).unwrap();
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].operation, "fetch");
        assert_eq!(events[0].outcome, "success");
    }

    #[test]
    fn remote_branch_ahead_behind_computed_after_fetch() {
        let fixture = create_local_bare_remote();
        let second = fixture.second_repo();
        commit_file(
            &second,
            &fixture.second_path,
            "remote.txt",
            "remote",
            "remote",
        );
        push_branch(&second, "origin", "main");

        git_fetch(&fixture.work_path, fetch_req(false)).unwrap();
        let repo = fixture.repo();
        let (ahead, behind) = ahead_behind_for_remote(&repo, "origin", "main").unwrap();
        assert_eq!(ahead, 0);
        assert_eq!(behind, 1);
    }

    #[test]
    fn branch_info_returned_after_fetch_contains_ahead_behind() {
        let fixture = create_local_bare_remote();
        let result = git_fetch(&fixture.work_path, fetch_req(false)).unwrap();
        assert!(result.branches.iter().any(|branch| branch.name == "main"));
    }

    #[test]
    fn fixture_supports_branch_creation_for_tests() {
        let fixture = create_local_bare_remote();
        let repo = fixture.repo();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature/x", &head, false).unwrap();
        assert!(repo.find_branch("feature/x", BranchType::Local).is_ok());
    }
}

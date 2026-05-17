//! MVP-19 W2-B · session service orchestration.
//!
//! This module is the DB-writing adapter between the pure lifecycle engine and
//! the canonical session DAO / IPC contracts. It deliberately imports W1/W2-A
//! types instead of redefining them.

use crate::db::{DbError, DbPool};
use crate::session_dao::{AiSessionDao, SessionCommitLinkDao};
use crate::session_ipc::{
    SessionBindCommitRequest, SessionBindCommitResult, SessionBindMode, SessionDetailRequest,
    SessionDetailResult, SessionEndRequest, SessionEndResult, SessionListRequest,
    SessionListResult, SessionRebindRequest, SessionRebindResult, SessionRecalcRequest,
    SessionRecalcResult, SessionStartRequest, SessionStartResult, SessionUnbindRequest,
    SessionUnbindResult,
};
use crate::session_lifecycle::{
    SessionInputEvent, SessionLifecycleDecision, SessionLifecycleService,
};
use crate::sessions::{AiSession, LinkState, SessionCommitLink, SessionError, SessionStatus};
use rusqlite::OptionalExtension;
use uuid::Uuid;

const STRATEGY_VERSION: &str = "v1";
const AUTO_CONFIRM_THRESHOLD: f32 = 0.7;
const TIME_WINDOW_STRONG_MS: i64 = 30 * 60 * 1000;
const TIME_WINDOW_NEAR_MS: i64 = 2 * 60 * 60 * 1000;

pub struct SessionService<'a> {
    pool: &'a DbPool,
    lifecycle: SessionLifecycleService,
}

impl<'a> SessionService<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self {
            pool,
            lifecycle: SessionLifecycleService::with_default_config(),
        }
    }

    pub fn with_lifecycle(pool: &'a DbPool, lifecycle: SessionLifecycleService) -> Self {
        Self { pool, lifecycle }
    }

    pub fn start(&self, req: SessionStartRequest) -> Result<SessionStartResult, SessionError> {
        let now = req.started_at.unwrap_or_else(now_ms);
        if let Some(active) = self.active_session(&req.workspace_id)? {
            if req.source == "manual" {
                let event = SessionInputEvent::ManualSplit {
                    pane_id: req.pane_id.unwrap_or_else(|| "manual".to_string()),
                    workspace_id: req.workspace_id.clone(),
                    at: millis_to_secs(now),
                };
                let session = self.process_lifecycle_event(Some(&active.id), event)?;
                return Ok(SessionStartResult {
                    session,
                    already_active: false,
                });
            }

            return Ok(SessionStartResult {
                session: active,
                already_active: true,
            });
        }

        let event = SessionInputEvent::ProcessStart {
            pane_id: req.pane_id.unwrap_or_else(|| "unknown".to_string()),
            workspace_id: req.workspace_id.clone(),
            cli_kind: req.cli_kind.clone(),
            at: millis_to_secs(now),
        };
        let decision =
            self.lifecycle
                .process_event(SessionStatus::Ended, &event, req.cli_kind.as_str())?;
        let mut session = self.apply_lifecycle_decision(&req.workspace_id, None, decision, now)?;
        if let Some(title) = req.title {
            session.title = title;
            session.updated_at = now_ms();
            AiSessionDao::update(self.pool, &session)?;
        }
        if session.source != req.source {
            session.source = req.source;
            session.updated_at = now_ms();
            AiSessionDao::update(self.pool, &session)?;
        }
        Ok(SessionStartResult {
            session,
            already_active: false,
        })
    }

    pub fn end(&self, req: SessionEndRequest) -> Result<SessionEndResult, SessionError> {
        let mut session = AiSessionDao::get_by_id(self.pool, &req.workspace_id, &req.session_id)?;
        let now = now_ms();
        match req.end_reason.as_deref() {
            Some("idle_cutoff") => {
                session.status = SessionStatus::IdleCutoff;
                session.end_reason = Some("idle_cutoff".to_string());
            }
            Some("process_exit") => {
                session.status = SessionStatus::Ended;
                session.end_reason = Some("process_exit".to_string());
            }
            Some(reason) => {
                session.status = SessionStatus::Ended;
                session.end_reason = Some(reason.to_string());
            }
            None => {
                session.status = SessionStatus::Ended;
                session.end_reason = Some("manual_end".to_string());
            }
        }
        session.ended_at = Some(now);
        session.updated_at = now;
        AiSessionDao::update(self.pool, &session)?;
        Ok(SessionEndResult { session })
    }

    pub fn bind_commit(
        &self,
        req: SessionBindCommitRequest,
    ) -> Result<SessionBindCommitResult, SessionError> {
        let session = self.resolve_bind_target(&req)?;
        let linked_at = now_ms();
        let (confidence, confidence_reason) = self.score_link_candidate(&session, linked_at, &req);
        let link_state = match req.mode {
            SessionBindMode::Manual => LinkState::ConfirmedManual,
            SessionBindMode::Auto if confidence >= AUTO_CONFIRM_THRESHOLD => {
                LinkState::ConfirmedAuto
            }
            SessionBindMode::Auto => LinkState::Pending,
        };
        let requires_confirmation = link_state == LinkState::Pending;
        let is_primary = !self.has_active_primary(&req.workspace_id, &req.commit_sha)?;
        let link = self.insert_link(LinkDraft {
            workspace_id: req.workspace_id,
            session_id: session.id,
            commit_sha: req.commit_sha,
            is_primary,
            link_state,
            auto_bound: req.mode == SessionBindMode::Auto,
            confidence,
            confidence_reason,
            linked_at,
            created_by: if req.mode == SessionBindMode::Manual {
                "user".to_string()
            } else {
                "system".to_string()
            },
            reviewed_by: if req.mode == SessionBindMode::Manual {
                Some("user".to_string())
            } else {
                None
            },
        })?;

        Ok(SessionBindCommitResult {
            link,
            requires_confirmation,
        })
    }

    pub fn unbind(&self, req: SessionUnbindRequest) -> Result<SessionUnbindResult, SessionError> {
        let unlinked = SessionCommitLinkDao::unlink(
            self.pool,
            &req.workspace_id,
            &req.link_id,
            req.reason.as_deref(),
        )?;
        Ok(SessionUnbindResult {
            link_id: req.link_id,
            unlinked,
        })
    }

    pub fn list(&self, req: SessionListRequest) -> Result<SessionListResult, SessionError> {
        let sessions = AiSessionDao::list_by_workspace(self.pool, &req.workspace_id)?;
        Ok(SessionListResult { sessions })
    }

    pub fn get_detail(
        &self,
        req: SessionDetailRequest,
    ) -> Result<SessionDetailResult, SessionError> {
        let session = AiSessionDao::get_by_id(self.pool, &req.workspace_id, &req.session_id)?;
        let links =
            SessionCommitLinkDao::list_by_session(self.pool, &req.workspace_id, &req.session_id)?;
        let active_links: Vec<_> = links
            .iter()
            .filter(|link| link.unlinked_at.is_none())
            .collect();
        let commit_count = active_links.len() as i64;
        let avg_confidence = if active_links.is_empty() {
            0.0
        } else {
            active_links.iter().map(|link| link.confidence).sum::<f32>() / active_links.len() as f32
        };
        Ok(SessionDetailResult {
            session,
            links,
            commit_count,
            avg_confidence,
        })
    }

    pub fn rebind(&self, req: SessionRebindRequest) -> Result<SessionRebindResult, SessionError> {
        let old = SessionCommitLinkDao::get(self.pool, &req.workspace_id, &req.link_id)?;
        let target =
            self.get_session_scoped_or_cross_workspace(&req.workspace_id, &req.target_session_id)?;

        if old.session_id == target.id {
            return Err(SessionError::InvalidStateTransition(format!(
                "link {} is already bound to session {}",
                old.id, target.id
            )));
        }

        if old.is_primary && old.unlinked_at.is_none() {
            SessionCommitLinkDao::unlink(
                self.pool,
                &req.workspace_id,
                &old.id,
                req.reason.as_deref(),
            )?;
        }

        let linked_at = now_ms();
        let new_link = self.insert_link(LinkDraft {
            workspace_id: req.workspace_id.clone(),
            session_id: target.id,
            commit_sha: old.commit_sha.clone(),
            is_primary: !self.has_active_primary(&req.workspace_id, &old.commit_sha)?,
            link_state: LinkState::ConfirmedManual,
            auto_bound: false,
            confidence: 1.0,
            confidence_reason: format!(
                "manual rebind from link {}{}",
                old.id,
                req.reason
                    .as_deref()
                    .map(|reason| format!(": {reason}"))
                    .unwrap_or_default()
            ),
            linked_at,
            created_by: "user".to_string(),
            reviewed_by: Some("user".to_string()),
        })?;
        SessionCommitLinkDao::supersede(self.pool, &req.workspace_id, &old.id, &new_link.id)?;

        Ok(SessionRebindResult {
            superseded_link_id: old.id,
            new_link,
        })
    }

    pub fn recalc(&self, req: SessionRecalcRequest) -> Result<SessionRecalcResult, SessionError> {
        let mut candidates =
            SessionCommitLinkDao::list_by_commit(self.pool, &req.workspace_id, &req.commit_sha)?;
        if candidates.is_empty() {
            if let Some(session) = self.best_session_for_auto_bind(&req.workspace_id, now_ms())? {
                let result = self.bind_commit(SessionBindCommitRequest {
                    workspace_id: req.workspace_id,
                    commit_sha: req.commit_sha,
                    session_id: Some(session.id),
                    mode: SessionBindMode::Auto,
                    reason: Some("recalc".to_string()),
                })?;
                candidates.push(result.link);
            }
        }
        Ok(SessionRecalcResult { candidates })
    }

    pub fn process_lifecycle_event(
        &self,
        current_session_id: Option<&str>,
        event: SessionInputEvent,
    ) -> Result<AiSession, SessionError> {
        let workspace_id = event_workspace_id(&event);
        let at_ms = event.at().saturating_mul(1000);
        let (current_status, current_cli_kind) = if let Some(id) = current_session_id {
            let current = AiSessionDao::get_by_id(self.pool, workspace_id, id)?;
            (current.status, current.cli_kind)
        } else {
            (SessionStatus::Ended, String::new())
        };
        let decision =
            self.lifecycle
                .process_event(current_status, &event, current_cli_kind.as_str())?;
        self.apply_lifecycle_decision(workspace_id, current_session_id, decision, at_ms)
    }

    pub fn apply_lifecycle_decision(
        &self,
        workspace_id: &str,
        current_session_id: Option<&str>,
        decision: SessionLifecycleDecision,
        at_ms: i64,
    ) -> Result<AiSession, SessionError> {
        match decision {
            SessionLifecycleDecision::Continue => {
                let id = current_session_id.ok_or_else(|| {
                    SessionError::InvalidStateTransition(
                        "Continue requires current session".to_string(),
                    )
                })?;
                AiSessionDao::get_by_id(self.pool, workspace_id, id)
            }
            SessionLifecycleDecision::StartNew { cli_kind, source } => {
                self.insert_session(workspace_id, &cli_kind, &source, at_ms, None)
            }
            SessionLifecycleDecision::SplitOnClear { cli_kind } => {
                self.end_current(workspace_id, current_session_id, at_ms, "clear", false)?;
                self.insert_session(workspace_id, &cli_kind, "auto", at_ms, Some("After /clear"))
            }
            SessionLifecycleDecision::SplitManual { cli_kind } => {
                self.end_current(
                    workspace_id,
                    current_session_id,
                    at_ms,
                    "manual_split",
                    false,
                )?;
                self.insert_session(
                    workspace_id,
                    &cli_kind,
                    "manual",
                    at_ms,
                    Some("Manual split"),
                )
            }
            SessionLifecycleDecision::SoftEndIdle => {
                self.end_current(workspace_id, current_session_id, at_ms, "idle_cutoff", true)
            }
            SessionLifecycleDecision::EndOnProcessExit { exit_code } => {
                let reason = exit_code
                    .map(|code| format!("process_exit:{code}"))
                    .unwrap_or_else(|| "process_exit".to_string());
                self.end_current(workspace_id, current_session_id, at_ms, &reason, false)
            }
        }
    }

    fn resolve_bind_target(
        &self,
        req: &SessionBindCommitRequest,
    ) -> Result<AiSession, SessionError> {
        if let Some(session_id) = &req.session_id {
            return self.get_session_scoped_or_cross_workspace(&req.workspace_id, session_id);
        }
        self.best_session_for_auto_bind(&req.workspace_id, now_ms())?
            .ok_or_else(|| SessionError::SessionNotFound("no session candidates".to_string()))
    }

    fn active_session(&self, workspace_id: &str) -> Result<Option<AiSession>, SessionError> {
        Ok(AiSessionDao::list_by_workspace(self.pool, workspace_id)?
            .into_iter()
            .find(|session| session.status == SessionStatus::Active))
    }

    fn best_session_for_auto_bind(
        &self,
        workspace_id: &str,
        commit_at: i64,
    ) -> Result<Option<AiSession>, SessionError> {
        let mut best: Option<(AiSession, f32)> = None;
        for session in AiSessionDao::list_by_workspace(self.pool, workspace_id)? {
            if session.status == SessionStatus::Archived {
                continue;
            }
            let req = SessionBindCommitRequest {
                workspace_id: workspace_id.to_string(),
                commit_sha: String::new(),
                session_id: Some(session.id.clone()),
                mode: SessionBindMode::Auto,
                reason: None,
            };
            let (score, _) = self.score_link_candidate(&session, commit_at, &req);
            if best
                .as_ref()
                .map(|(_, best_score)| score > *best_score)
                .unwrap_or(true)
            {
                best = Some((session, score));
            }
        }
        Ok(best.map(|(session, _)| session))
    }

    fn get_session_scoped_or_cross_workspace(
        &self,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<AiSession, SessionError> {
        match AiSessionDao::get_by_id(self.pool, workspace_id, session_id) {
            Ok(session) => Ok(session),
            Err(SessionError::SessionNotFound(_)) => {
                if self.session_exists_elsewhere(workspace_id, session_id)? {
                    Err(SessionError::CrossWorkspaceDenied)
                } else {
                    Err(SessionError::SessionNotFound(session_id.to_string()))
                }
            }
            Err(error) => Err(error),
        }
    }

    fn session_exists_elsewhere(
        &self,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<bool, SessionError> {
        let conn = self.pool.get().map_err(DbError::from).map_err(db_err)?;
        conn.query_row(
            "SELECT workspace_id FROM ai_sessions WHERE id = ?1",
            [session_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(db_err)
        .map(|found| {
            found
                .as_deref()
                .map(|found_workspace| found_workspace != workspace_id)
                .unwrap_or(false)
        })
    }

    fn has_active_primary(
        &self,
        workspace_id: &str,
        commit_sha: &str,
    ) -> Result<bool, SessionError> {
        Ok(
            SessionCommitLinkDao::list_by_commit(self.pool, workspace_id, commit_sha)?
                .into_iter()
                .any(|link| link.is_primary && link.unlinked_at.is_none()),
        )
    }

    fn insert_session(
        &self,
        workspace_id: &str,
        cli_kind: &str,
        source: &str,
        started_at: i64,
        title: Option<&str>,
    ) -> Result<AiSession, SessionError> {
        let session = AiSession {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.to_string(),
            cli_kind: cli_kind.to_string(),
            source: source.to_string(),
            title: title
                .map(str::to_string)
                .unwrap_or_else(|| format!("{cli_kind} session")),
            started_at,
            ended_at: None,
            end_reason: None,
            prompt_count: 0,
            token_count: None,
            event_count: 1,
            status: SessionStatus::Active,
            parser_version: None,
            strategy_version: Some(STRATEGY_VERSION.to_string()),
            // W2-B stores only bounded routing metadata. Phase E owns redacted summaries (§M).
            metadata_json: "{}".to_string(),
            created_at: started_at,
            updated_at: started_at,
        };
        AiSessionDao::insert(self.pool, &session)?;
        Ok(session)
    }

    fn end_current(
        &self,
        workspace_id: &str,
        current_session_id: Option<&str>,
        ended_at: i64,
        reason: &str,
        idle_cutoff: bool,
    ) -> Result<AiSession, SessionError> {
        let id = current_session_id.ok_or_else(|| {
            SessionError::InvalidStateTransition("ending requires current session".to_string())
        })?;
        let mut current = AiSessionDao::get_by_id(self.pool, workspace_id, id)?;
        current.status = if idle_cutoff {
            SessionStatus::IdleCutoff
        } else {
            SessionStatus::Ended
        };
        current.ended_at = Some(ended_at);
        current.end_reason = Some(reason.to_string());
        current.updated_at = ended_at;
        AiSessionDao::update(self.pool, &current)?;
        Ok(current)
    }

    fn score_link_candidate(
        &self,
        session: &AiSession,
        commit_at: i64,
        req: &SessionBindCommitRequest,
    ) -> (f32, String) {
        if req.mode == SessionBindMode::Manual {
            return (
                1.0,
                req.reason
                    .as_deref()
                    .map(|reason| format!("manual assignment: {reason}"))
                    .unwrap_or_else(|| "manual assignment".to_string()),
            );
        }

        let (time_score, time_reason) = time_window_score(session, commit_at);
        let (context_score, context_reason) = source_context_score(session, req);
        let confidence = (time_score + context_score).min(1.0);
        (
            confidence,
            format!(
                "v1 heuristic: time_window={time_score:.2} ({time_reason}); \
                 source_context={context_score:.2} ({context_reason}); threshold={AUTO_CONFIRM_THRESHOLD:.2}"
            ),
        )
    }

    fn insert_link(&self, draft: LinkDraft) -> Result<SessionCommitLink, SessionError> {
        let mut link = SessionCommitLink {
            id: Uuid::new_v4().to_string(),
            workspace_id: draft.workspace_id,
            session_id: draft.session_id,
            commit_sha: draft.commit_sha,
            is_primary: draft.is_primary,
            link_state: draft.link_state,
            auto_bound: draft.auto_bound,
            confidence: draft.confidence,
            confidence_reason: draft.confidence_reason,
            strategy_version: STRATEGY_VERSION.to_string(),
            source_event_id: None,
            linked_at: draft.linked_at,
            unlinked_at: None,
            unlinked_reason: None,
            superseded_by_link_id: None,
            created_by: draft.created_by,
            reviewed_by: draft.reviewed_by,
            created_at: draft.linked_at,
            updated_at: draft.linked_at,
        };
        match SessionCommitLinkDao::insert(self.pool, &link) {
            Ok(()) => Ok(link),
            Err(SessionError::DbError(message))
                if link.is_primary && message.contains("primary link conflict") =>
            {
                link.is_primary = false;
                SessionCommitLinkDao::insert(self.pool, &link)?;
                Ok(link)
            }
            Err(error) => Err(error),
        }
    }
}

struct LinkDraft {
    workspace_id: String,
    session_id: String,
    commit_sha: String,
    is_primary: bool,
    link_state: LinkState,
    auto_bound: bool,
    confidence: f32,
    confidence_reason: String,
    linked_at: i64,
    created_by: String,
    reviewed_by: Option<String>,
}

fn time_window_score(session: &AiSession, commit_at: i64) -> (f32, &'static str) {
    let ended_at = session.ended_at.unwrap_or(commit_at);
    if commit_at >= session.started_at && commit_at <= ended_at {
        return (0.65, "commit within session interval");
    }

    let distance = if commit_at < session.started_at {
        session.started_at - commit_at
    } else {
        commit_at - ended_at
    };
    if distance <= TIME_WINDOW_STRONG_MS {
        (0.55, "commit near session interval")
    } else if distance <= TIME_WINDOW_NEAR_MS {
        (0.35, "commit within fallback window")
    } else {
        (0.20, "commit outside fallback window")
    }
}

fn source_context_score(session: &AiSession, req: &SessionBindCommitRequest) -> (f32, String) {
    let Some(reason) = req.reason.as_deref() else {
        return (0.0, "no source hint".to_string());
    };
    let reason_lower = reason.to_ascii_lowercase();
    let mut matched = Vec::new();
    if !session.cli_kind.is_empty() && reason_lower.contains(&session.cli_kind.to_ascii_lowercase())
    {
        matched.push("cli_kind");
    }
    if !session.title.is_empty() && reason_lower.contains(&session.title.to_ascii_lowercase()) {
        matched.push("title");
    }
    if reason_lower.contains("pane=") || reason_lower.contains("pane_id=") {
        matched.push("pane_hint");
    }
    if matched.is_empty() {
        (0.05, "source hint present but weak".to_string())
    } else {
        (0.25, matched.join("+"))
    }
}

fn event_workspace_id(event: &SessionInputEvent) -> &str {
    match event {
        SessionInputEvent::ProcessStart { workspace_id, .. }
        | SessionInputEvent::ClearCommand { workspace_id, .. }
        | SessionInputEvent::ManualSplit { workspace_id, .. }
        | SessionInputEvent::Activity { workspace_id, .. }
        | SessionInputEvent::ProcessExit { workspace_id, .. } => workspace_id,
    }
}

fn millis_to_secs(ms: i64) -> i64 {
    ms / 1000
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn db_err(e: impl std::string::ToString) -> SessionError {
    SessionError::DbError(e.to_string())
}

use tempfile::TempDir;
use vibestation_core::{
    db, AiSession, AiSessionDao, LinkState, SessionBindCommitRequest, SessionBindMode,
    SessionCommitLinkDao, SessionDetailRequest, SessionError, SessionInputEvent,
    SessionLifecycleDecision, SessionListRequest, SessionRebindRequest, SessionRecalcRequest,
    SessionService, SessionStartRequest, SessionStatus, SessionUnbindRequest,
};

fn setup() -> (TempDir, db::DbPool) {
    let dir = TempDir::new().unwrap();
    let pool = db::open_pool(&dir.path().join("session_service_test.db")).unwrap();
    (dir, pool)
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn active_session(id: &str, workspace_id: &str, started_at: i64) -> AiSession {
    AiSession {
        id: id.to_string(),
        workspace_id: workspace_id.to_string(),
        cli_kind: "claude".to_string(),
        source: "auto".to_string(),
        title: format!("session {id}"),
        started_at,
        ended_at: None,
        end_reason: None,
        prompt_count: 0,
        token_count: None,
        event_count: 1,
        status: SessionStatus::Active,
        parser_version: None,
        strategy_version: Some("v1".to_string()),
        metadata_json: "{}".to_string(),
        created_at: started_at,
        updated_at: started_at,
    }
}

#[test]
fn session_service_start_new_inserts_active_session() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    let started_at = 1_800_000_000_000;

    let result = service
        .start(SessionStartRequest {
            workspace_id: "w1".into(),
            cli_kind: "claude".into(),
            source: "auto".into(),
            pane_id: Some("pane-1".into()),
            title: Some("implement feature".into()),
            started_at: Some(started_at),
        })
        .unwrap();

    assert!(!result.already_active);
    assert_eq!(result.session.workspace_id, "w1");
    assert_eq!(result.session.cli_kind, "claude");
    assert_eq!(result.session.title, "implement feature");
    assert_eq!(result.session.started_at, started_at);
    assert_eq!(result.session.status, SessionStatus::Active);
    assert_eq!(
        AiSessionDao::get_by_id(&pool, "w1", &result.session.id).unwrap(),
        result.session
    );
}

#[test]
fn session_service_lifecycle_decision_mapping_covers_all_decisions() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    AiSessionDao::insert(&pool, &active_session("s-active", "w1", 1_000)).unwrap();

    let continued = service
        .apply_lifecycle_decision(
            "w1",
            Some("s-active"),
            SessionLifecycleDecision::Continue,
            2_000,
        )
        .unwrap();
    assert_eq!(continued.id, "s-active");
    assert_eq!(continued.status, SessionStatus::Active);

    let started = service
        .apply_lifecycle_decision(
            "w1",
            None,
            SessionLifecycleDecision::StartNew {
                cli_kind: "codex".into(),
                source: "auto".into(),
            },
            3_000,
        )
        .unwrap();
    assert_eq!(started.cli_kind, "codex");
    assert_eq!(started.status, SessionStatus::Active);

    let split_clear = service
        .apply_lifecycle_decision(
            "w1",
            Some(&started.id),
            SessionLifecycleDecision::SplitOnClear {
                cli_kind: "codex".into(),
            },
            4_000,
        )
        .unwrap();
    assert_eq!(split_clear.status, SessionStatus::Active);
    assert_eq!(
        AiSessionDao::get_by_id(&pool, "w1", &started.id)
            .unwrap()
            .end_reason
            .as_deref(),
        Some("clear")
    );

    let split_manual = service
        .apply_lifecycle_decision(
            "w1",
            Some(&split_clear.id),
            SessionLifecycleDecision::SplitManual {
                cli_kind: "codex".into(),
            },
            5_000,
        )
        .unwrap();
    assert_eq!(
        AiSessionDao::get_by_id(&pool, "w1", &split_clear.id)
            .unwrap()
            .end_reason
            .as_deref(),
        Some("manual_split")
    );

    let idle = service
        .apply_lifecycle_decision(
            "w1",
            Some(&split_manual.id),
            SessionLifecycleDecision::SoftEndIdle,
            6_000,
        )
        .unwrap();
    assert_eq!(idle.status, SessionStatus::IdleCutoff);
    assert_eq!(idle.end_reason.as_deref(), Some("idle_cutoff"));

    AiSessionDao::insert(&pool, &active_session("s-exit", "w1", 7_000)).unwrap();
    let exited = service
        .apply_lifecycle_decision(
            "w1",
            Some("s-exit"),
            SessionLifecycleDecision::EndOnProcessExit { exit_code: Some(0) },
            8_000,
        )
        .unwrap();
    assert_eq!(exited.status, SessionStatus::Ended);
    assert_eq!(exited.end_reason.as_deref(), Some("process_exit:0"));
}

#[test]
fn session_service_can_process_lifecycle_events_against_dao_state() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    AiSessionDao::insert(&pool, &active_session("s1", "w1", 1_000)).unwrap();

    let new_session = service
        .process_lifecycle_event(
            Some("s1"),
            SessionInputEvent::ClearCommand {
                pane_id: "pane-1".into(),
                workspace_id: "w1".into(),
                at: 2,
            },
        )
        .unwrap();

    assert_eq!(new_session.status, SessionStatus::Active);
    let old = AiSessionDao::get_by_id(&pool, "w1", "s1").unwrap();
    assert_eq!(old.status, SessionStatus::Ended);
    assert_eq!(old.end_reason.as_deref(), Some("clear"));
}

#[test]
fn session_service_bind_commit_high_confidence_auto_confirms_primary() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    let started_at = now_ms() - 60_000;
    AiSessionDao::insert(&pool, &active_session("s1", "w1", started_at)).unwrap();

    let result = service
        .bind_commit(SessionBindCommitRequest {
            workspace_id: "w1".into(),
            commit_sha: "abc123".into(),
            session_id: Some("s1".into()),
            mode: SessionBindMode::Auto,
            reason: Some("pane=pane-1 cli=claude".into()),
        })
        .unwrap();

    assert!(!result.requires_confirmation);
    assert_eq!(result.link.session_id, "s1");
    assert_eq!(result.link.link_state, LinkState::ConfirmedAuto);
    assert!(result.link.is_primary);
    assert!(result.link.confidence >= 0.7, "{:?}", result.link);
    assert_eq!(result.link.strategy_version, "v1");
}

#[test]
fn session_service_bind_commit_low_confidence_stays_pending() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    let old_start = now_ms() - 7_200_000;
    let mut old = active_session("s-old", "w1", old_start);
    old.status = SessionStatus::Ended;
    old.ended_at = Some(old_start + 60_000);
    old.end_reason = Some("manual_end".into());
    AiSessionDao::insert(&pool, &old).unwrap();

    let result = service
        .bind_commit(SessionBindCommitRequest {
            workspace_id: "w1".into(),
            commit_sha: "lowconf".into(),
            session_id: Some("s-old".into()),
            mode: SessionBindMode::Auto,
            reason: None,
        })
        .unwrap();

    assert!(result.requires_confirmation);
    assert_eq!(result.link.link_state, LinkState::Pending);
    assert_ne!(result.link.link_state, LinkState::ConfirmedAuto);
    assert!(result.link.confidence < 0.7, "{:?}", result.link);
}

#[test]
fn session_service_rebind_supersedes_old_link_and_inserts_new_link() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    let now = now_ms();
    AiSessionDao::insert(&pool, &active_session("s1", "w1", now - 60_000)).unwrap();
    AiSessionDao::insert(&pool, &active_session("s2", "w1", now - 30_000)).unwrap();
    let old = service
        .bind_commit(SessionBindCommitRequest {
            workspace_id: "w1".into(),
            commit_sha: "rebind-sha".into(),
            session_id: Some("s1".into()),
            mode: SessionBindMode::Manual,
            reason: Some("initial".into()),
        })
        .unwrap()
        .link;

    let result = service
        .rebind(SessionRebindRequest {
            workspace_id: "w1".into(),
            link_id: old.id.clone(),
            target_session_id: "s2".into(),
            reason: Some("wrong session".into()),
        })
        .unwrap();

    assert_eq!(result.superseded_link_id, old.id);
    assert_eq!(result.new_link.session_id, "s2");
    assert_eq!(result.new_link.commit_sha, "rebind-sha");
    assert_eq!(result.new_link.link_state, LinkState::ConfirmedManual);

    let superseded = SessionCommitLinkDao::get(&pool, "w1", &old.id).unwrap();
    assert_eq!(superseded.link_state, LinkState::Superseded);
    assert_eq!(
        superseded.superseded_by_link_id.as_deref(),
        Some(result.new_link.id.as_str())
    );
}

#[test]
fn session_service_rejects_cross_workspace_bind_and_rebind() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    let now = now_ms();
    AiSessionDao::insert(&pool, &active_session("s1", "w1", now - 60_000)).unwrap();
    AiSessionDao::insert(&pool, &active_session("s2", "w2", now - 30_000)).unwrap();

    let bind_err = service
        .bind_commit(SessionBindCommitRequest {
            workspace_id: "w2".into(),
            commit_sha: "sha-cross".into(),
            session_id: Some("s1".into()),
            mode: SessionBindMode::Manual,
            reason: None,
        })
        .unwrap_err();
    assert_eq!(bind_err, SessionError::CrossWorkspaceDenied);

    let link = service
        .bind_commit(SessionBindCommitRequest {
            workspace_id: "w1".into(),
            commit_sha: "sha-ok".into(),
            session_id: Some("s1".into()),
            mode: SessionBindMode::Manual,
            reason: None,
        })
        .unwrap()
        .link;
    let rebind_err = service
        .rebind(SessionRebindRequest {
            workspace_id: "w1".into(),
            link_id: link.id,
            target_session_id: "s2".into(),
            reason: None,
        })
        .unwrap_err();
    assert_eq!(rebind_err, SessionError::CrossWorkspaceDenied);
}

#[test]
fn session_service_list_detail_unbind_and_recalc_roundtrip() {
    let (_dir, pool) = setup();
    let service = SessionService::new(&pool);
    let now = now_ms();
    AiSessionDao::insert(&pool, &active_session("s1", "w1", now - 60_000)).unwrap();
    AiSessionDao::insert(&pool, &active_session("s2", "w1", now - 30_000)).unwrap();

    let link = service
        .bind_commit(SessionBindCommitRequest {
            workspace_id: "w1".into(),
            commit_sha: "detail-sha".into(),
            session_id: Some("s1".into()),
            mode: SessionBindMode::Auto,
            reason: Some("cli=claude".into()),
        })
        .unwrap()
        .link;

    let list = service
        .list(SessionListRequest {
            workspace_id: "w1".into(),
        })
        .unwrap();
    assert_eq!(list.sessions.len(), 2);

    let detail = service
        .get_detail(SessionDetailRequest {
            workspace_id: "w1".into(),
            session_id: "s1".into(),
        })
        .unwrap();
    assert_eq!(detail.session.id, "s1");
    assert_eq!(detail.commit_count, 1);
    assert!(detail.avg_confidence > 0.0);

    let recalc = service
        .recalc(SessionRecalcRequest {
            workspace_id: "w1".into(),
            commit_sha: "detail-sha".into(),
        })
        .unwrap();
    assert!(!recalc.candidates.is_empty());
    assert!(
        recalc
            .candidates
            .iter()
            .filter(|link| link.is_primary && link.unlinked_at.is_none())
            .count()
            <= 1
    );

    let unbound = service
        .unbind(SessionUnbindRequest {
            workspace_id: "w1".into(),
            link_id: link.id.clone(),
            reason: Some("manual correction".into()),
        })
        .unwrap();
    assert!(unbound.unlinked);
    assert_eq!(
        SessionCommitLinkDao::get(&pool, "w1", &link.id)
            .unwrap()
            .link_state,
        LinkState::Unlinked
    );
}

use serde_json::json;
use ts_rs::{Config, TS};
use vibestation_core::{AiSession, LinkState, SessionCommitLink, SessionError, SessionStatus};

#[test]
fn session_types_preserve_canonical_serde_and_ts_contract() {
    let session = AiSession {
        id: "session-1".to_string(),
        workspace_id: "workspace-1".to_string(),
        cli_kind: "codex".to_string(),
        source: "auto".to_string(),
        title: "Implement session binding".to_string(),
        started_at: 1_789_000_000,
        ended_at: Some(1_789_000_100),
        end_reason: Some("userStopped".to_string()),
        prompt_count: 2,
        token_count: Some(4096),
        event_count: 8,
        status: SessionStatus::IdleCutoff,
        parser_version: Some("spike-07".to_string()),
        strategy_version: Some("v1".to_string()),
        metadata_json: "{}".to_string(),
        created_at: 1_789_000_000,
        updated_at: 1_789_000_100,
    };

    let session_json = serde_json::to_value(&session).expect("serialize AiSession");
    assert_eq!(session_json["workspaceId"], "workspace-1");
    assert_eq!(session_json["startedAt"], 1_789_000_000);
    assert_eq!(session_json["endReason"], "userStopped");
    assert_eq!(session_json["metadataJson"], "{}");
    assert_eq!(session_json["status"], "idleCutoff");

    assert_eq!(
        serde_json::to_value(SessionStatus::Active).unwrap(),
        json!("active")
    );
    assert_eq!(
        serde_json::to_value(SessionStatus::Ended).unwrap(),
        json!("ended")
    );
    assert_eq!(
        serde_json::to_value(SessionStatus::IdleCutoff).unwrap(),
        json!("idleCutoff")
    );
    assert_eq!(
        serde_json::to_value(SessionStatus::Archived).unwrap(),
        json!("archived")
    );

    let link = SessionCommitLink {
        id: "link-1".to_string(),
        workspace_id: "workspace-1".to_string(),
        session_id: "session-1".to_string(),
        commit_sha: "abc1234".to_string(),
        is_primary: true,
        link_state: LinkState::ConfirmedAuto,
        auto_bound: true,
        confidence: 0.87,
        confidence_reason: "temporal overlap".to_string(),
        strategy_version: "v1".to_string(),
        source_event_id: Some("event-1".to_string()),
        linked_at: 1_789_000_120,
        unlinked_at: None,
        unlinked_reason: None,
        superseded_by_link_id: None,
        created_by: "system".to_string(),
        reviewed_by: None,
        created_at: 1_789_000_120,
        updated_at: 1_789_000_120,
    };
    let link_json = serde_json::to_value(&link).expect("serialize SessionCommitLink");
    assert_eq!(link_json["commitSha"], "abc1234");
    assert_eq!(link_json["isPrimary"], true);
    assert_eq!(link_json["autoBound"], true);
    assert_eq!(link_json["linkState"], "confirmedAuto");
    assert_eq!(link_json["supersededByLinkId"], serde_json::Value::Null);

    assert_eq!(
        serde_json::to_value(LinkState::Pending).unwrap(),
        json!("pending")
    );
    assert_eq!(
        serde_json::to_value(LinkState::ConfirmedAuto).unwrap(),
        json!("confirmedAuto")
    );
    assert_eq!(
        serde_json::to_value(LinkState::ConfirmedManual).unwrap(),
        json!("confirmedManual")
    );
    assert_eq!(
        serde_json::to_value(LinkState::Unlinked).unwrap(),
        json!("unlinked")
    );
    assert_eq!(
        serde_json::to_value(LinkState::Superseded).unwrap(),
        json!("superseded")
    );
    assert_eq!(
        serde_json::to_value(LinkState::Stale).unwrap(),
        json!("stale")
    );

    assert_eq!(
        serde_json::to_value(SessionError::SessionNotFound("session-1".to_string())).unwrap(),
        json!({ "kind": "sessionNotFound", "detail": "session-1" })
    );
    assert_eq!(
        serde_json::to_value(SessionError::CrossWorkspaceDenied).unwrap(),
        json!({ "kind": "crossWorkspaceDenied" })
    );

    let cfg = Config::default();
    let session_decl = AiSession::decl(&cfg);
    assert!(session_decl.contains("workspaceId: string"));
    assert!(session_decl.contains("startedAt: number"));
    assert!(session_decl.contains("endedAt: number | null"));
    assert!(session_decl.contains("endReason: string | null"));
    assert!(session_decl.contains("metadataJson: string"));

    let link_decl = SessionCommitLink::decl(&cfg);
    assert!(link_decl.contains("commitSha: string"));
    assert!(link_decl.contains("isPrimary: boolean"));
    assert!(link_decl.contains("linkState: LinkState"));
    assert!(link_decl.contains("linkedAt: number"));
    assert!(link_decl.contains("unlinkedAt: number | null"));
}

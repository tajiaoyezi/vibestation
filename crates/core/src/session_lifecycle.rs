//! MVP-19 §I.1 Phase A · Session boundary detection + lifecycle state-machine engine.
//!
//! Pure logic layer — no DB writes, no background threads.
//! DAO (W1-A.1) and IPC wire (W2) consume the decisions produced here.
//!
//! Imports canonical types from `crate::sessions::*` (W1-A.0 single source of truth).
//!
//! § coverage:
//!  §C.1.1 / §C.3 — 4 boundary types: ProcessStart / ClearCommand / ManualSplit / IdleCutoff
//!  §I.1.1        — SessionBoundaryDetector
//!  §I.1.2        — SessionLifecycleService
//!  §I.1.3        — idle cutoff pure check (`is_idle_cutoff`)
//!  §I.1.4        — boundary fixture tests (see #[cfg(test)])
//!  §I.1.5        — SessionInputEvent pane/source event input interface
//!  §H7           — conservative idle default 1800 s (30 min)

use crate::sessions::{SessionError, SessionStatus};

// ── Input event interface ─────────────────────────────────────────────────────

/// §I.1.5 · Events the engine consumes from pane/source infrastructure.
///
/// W2 IPC wire adapts real pane events to this typed interface.
/// §C.1.1 / §C.3 boundary triggers represented here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionInputEvent {
    /// §C.1.1 boundary type 1: new CLI process started → new session.
    ProcessStart {
        pane_id: String,
        workspace_id: String,
        cli_kind: String,
        /// Unix timestamp (seconds).
        at: i64,
    },
    /// §C.1.1 boundary type 2: explicit `/clear` → old session end + new session start.
    ClearCommand {
        pane_id: String,
        workspace_id: String,
        at: i64,
    },
    /// §C.1.1 boundary type 3: user manually initiated a new session → explicit split.
    ManualSplit {
        pane_id: String,
        workspace_id: String,
        at: i64,
    },
    /// General activity — resets the idle timer without creating a boundary.
    Activity {
        pane_id: String,
        workspace_id: String,
        at: i64,
    },
    /// CLI process exited (normal or abnormal); may trigger session end.
    ProcessExit {
        pane_id: String,
        workspace_id: String,
        exit_code: Option<i32>,
        at: i64,
    },
}

impl SessionInputEvent {
    /// Wall-clock timestamp (Unix seconds) associated with this event.
    pub fn at(&self) -> i64 {
        match self {
            Self::ProcessStart { at, .. }
            | Self::ClearCommand { at, .. }
            | Self::ManualSplit { at, .. }
            | Self::Activity { at, .. }
            | Self::ProcessExit { at, .. } => *at,
        }
    }

    fn event_kind(&self) -> &'static str {
        match self {
            Self::ProcessStart { .. } => "ProcessStart",
            Self::ClearCommand { .. } => "ClearCommand",
            Self::ManualSplit { .. } => "ManualSplit",
            Self::Activity { .. } => "Activity",
            Self::ProcessExit { .. } => "ProcessExit",
        }
    }
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// §H7 · Idle cutoff configuration.
///
/// Conservative 30-min default. Arbiter+implementer tune after v1.0 launch
/// based on real usage patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdleCutoffConfig {
    /// Seconds of inactivity before an `Active` session soft-ends as `IdleCutoff`.
    /// §H7: Arbiter+implementer 后续按真实 usage 微调 · 此为保守初值.
    pub threshold_secs: u64,
}

impl Default for IdleCutoffConfig {
    fn default() -> Self {
        // §H7: 保守初值 30 min = 1800 s · Arbiter+implementer 后续按真实 usage 微调
        Self {
            threshold_secs: 1800,
        }
    }
}

// ── Decision / boundary types ─────────────────────────────────────────────────

/// Reason category for a detected boundary (used in `SessionBoundary`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundaryReason {
    ProcessStart,
    ClearCommand,
    ManualSplit,
    IdleCutoff,
    ProcessExit,
}

/// A detected session boundary in an ordered event stream.
///
/// Produced by `SessionBoundaryDetector::detect_boundaries`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionBoundary {
    pub reason: BoundaryReason,
    /// Timestamp at which the boundary was detected (Unix seconds).
    pub at: i64,
    pub decision: SessionLifecycleDecision,
}

/// The engine's output: what lifecycle action the DAO layer (W1-A.1) should perform.
///
/// This engine only *produces* decisions; DB persistence is W1-A.1's responsibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionLifecycleDecision {
    /// Start a brand-new session (no prior active session, or after idle/ended).
    /// `source`: `"auto"` for process-driven, `"manual"` for user-driven.
    StartNew { cli_kind: String, source: String },
    /// End the current session + start a new one, triggered by `/clear`.
    SplitOnClear { cli_kind: String },
    /// End the current session + start a new one, triggered by user manual action.
    SplitManual { cli_kind: String },
    /// Soft-end the current `Active` session due to idle threshold.
    /// DAO sets `status = IdleCutoff`, `end_reason = "idle_cutoff"`.
    /// §H7: §C.1.1 boundary type 4.
    SoftEndIdle,
    /// End the current session because the CLI process exited.
    EndOnProcessExit { exit_code: Option<i32> },
    /// No state change — session continues normally.
    Continue,
}

// ── Pure idle-cutoff check ────────────────────────────────────────────────────

/// §I.1.3 · Pure idle cutoff predicate — no side effects, no background threads.
///
/// Returns `true` when `now_at - last_activity_at >= config.threshold_secs`.
/// §H7: Arbiter+implementer 后续按真实 usage 微调 · 此为保守初值.
pub fn is_idle_cutoff(last_activity_at: i64, now_at: i64, config: &IdleCutoffConfig) -> bool {
    if now_at <= last_activity_at {
        return false;
    }
    (now_at - last_activity_at) as u64 >= config.threshold_secs
}

// ── Boundary detector ─────────────────────────────────────────────────────────

/// §I.1.1 · Batch session boundary detector over an ordered event stream.
///
/// Processes events in a single forward pass; stateless beyond `config`.
/// Idle cutoff is checked at each event arrival: a `SoftEndIdle` boundary is
/// emitted at the timestamp of the first event that arrives after the threshold.
pub struct SessionBoundaryDetector {
    config: IdleCutoffConfig,
}

impl SessionBoundaryDetector {
    pub fn new(config: IdleCutoffConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(IdleCutoffConfig::default())
    }

    /// Scan an ordered event stream and return all detected session boundaries.
    ///
    /// §C.1.1 / §C.3: recognises 4 boundary types plus `ProcessExit`.
    pub fn detect_boundaries(&self, events: &[SessionInputEvent]) -> Vec<SessionBoundary> {
        let mut boundaries = Vec::new();
        let mut active_cli_kind = String::new();
        let mut last_activity_at: Option<i64> = None;
        let mut has_active_session = false;

        for event in events {
            let at = event.at();

            // Before processing this event, check if idle elapsed since last activity.
            if has_active_session {
                if let Some(last) = last_activity_at {
                    if is_idle_cutoff(last, at, &self.config) {
                        boundaries.push(SessionBoundary {
                            reason: BoundaryReason::IdleCutoff,
                            at,
                            decision: SessionLifecycleDecision::SoftEndIdle,
                        });
                        has_active_session = false;
                        last_activity_at = None;
                    }
                }
            }

            match event {
                SessionInputEvent::ProcessStart { cli_kind, .. } => {
                    active_cli_kind = cli_kind.clone();
                    boundaries.push(SessionBoundary {
                        reason: BoundaryReason::ProcessStart,
                        at,
                        decision: SessionLifecycleDecision::StartNew {
                            cli_kind: cli_kind.clone(),
                            source: "auto".to_string(),
                        },
                    });
                    has_active_session = true;
                    last_activity_at = Some(at);
                }
                SessionInputEvent::ClearCommand { .. } => {
                    if has_active_session {
                        boundaries.push(SessionBoundary {
                            reason: BoundaryReason::ClearCommand,
                            at,
                            decision: SessionLifecycleDecision::SplitOnClear {
                                cli_kind: active_cli_kind.clone(),
                            },
                        });
                        // SplitOnClear ends old session and starts a new one; remains active.
                        last_activity_at = Some(at);
                    }
                }
                SessionInputEvent::ManualSplit { .. } => {
                    if has_active_session {
                        boundaries.push(SessionBoundary {
                            reason: BoundaryReason::ManualSplit,
                            at,
                            decision: SessionLifecycleDecision::SplitManual {
                                cli_kind: active_cli_kind.clone(),
                            },
                        });
                        last_activity_at = Some(at);
                    }
                }
                SessionInputEvent::Activity { .. } => {
                    if has_active_session {
                        last_activity_at = Some(at);
                    }
                }
                SessionInputEvent::ProcessExit { exit_code, .. } => {
                    if has_active_session {
                        boundaries.push(SessionBoundary {
                            reason: BoundaryReason::ProcessExit,
                            at,
                            decision: SessionLifecycleDecision::EndOnProcessExit {
                                exit_code: *exit_code,
                            },
                        });
                        has_active_session = false;
                        last_activity_at = None;
                    }
                }
            }
        }

        boundaries
    }
}

// ── Lifecycle service ─────────────────────────────────────────────────────────

/// §I.1.2 · Single-session lifecycle state machine.
///
/// Input: current `SessionStatus` + `SessionInputEvent` → `SessionLifecycleDecision` or error.
/// Does not write to DB; returns `Err(InvalidStateTransition)` for illegal state changes.
pub struct SessionLifecycleService {
    config: IdleCutoffConfig,
}

impl SessionLifecycleService {
    pub fn new(config: IdleCutoffConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(IdleCutoffConfig::default())
    }

    /// Process `event` against `current_status` and return a lifecycle decision.
    ///
    /// `current_cli_kind`: CLI type of the current session (needed for split decisions
    /// from events that do not carry cli_kind, e.g. `ClearCommand`/`ManualSplit`).
    ///
    /// Returns `Err(SessionError::InvalidStateTransition)` for illegal transitions.
    pub fn process_event(
        &self,
        current_status: SessionStatus,
        event: &SessionInputEvent,
        current_cli_kind: &str,
    ) -> Result<SessionLifecycleDecision, SessionError> {
        match (&current_status, event) {
            // Archived sessions are frozen — no further transitions allowed.
            (SessionStatus::Archived, _) => Err(SessionError::InvalidStateTransition(format!(
                "cannot process {} on Archived session",
                event.event_kind()
            ))),

            // Active: new process start → start a new session (implicit process boundary).
            (SessionStatus::Active, SessionInputEvent::ProcessStart { cli_kind, .. }) => {
                Ok(SessionLifecycleDecision::StartNew {
                    cli_kind: cli_kind.clone(),
                    source: "auto".to_string(),
                })
            }

            // Active: /clear → split (end current + start new).
            (SessionStatus::Active, SessionInputEvent::ClearCommand { .. }) => {
                Ok(SessionLifecycleDecision::SplitOnClear {
                    cli_kind: current_cli_kind.to_string(),
                })
            }

            // Active: manual split → split (end current + start new).
            (SessionStatus::Active, SessionInputEvent::ManualSplit { .. }) => {
                Ok(SessionLifecycleDecision::SplitManual {
                    cli_kind: current_cli_kind.to_string(),
                })
            }

            // Active: general activity → session continues, idle timer reset by caller.
            (SessionStatus::Active, SessionInputEvent::Activity { .. }) => {
                Ok(SessionLifecycleDecision::Continue)
            }

            // Active: process exited → end session.
            (SessionStatus::Active, SessionInputEvent::ProcessExit { exit_code, .. }) => {
                Ok(SessionLifecycleDecision::EndOnProcessExit {
                    exit_code: *exit_code,
                })
            }

            // Ended or IdleCutoff + new process → start fresh session.
            (
                SessionStatus::Ended | SessionStatus::IdleCutoff,
                SessionInputEvent::ProcessStart { cli_kind, .. },
            ) => Ok(SessionLifecycleDecision::StartNew {
                cli_kind: cli_kind.clone(),
                source: "auto".to_string(),
            }),

            // Ended + any non-process-start event → invalid (session is already closed).
            (SessionStatus::Ended, _) => Err(SessionError::InvalidStateTransition(format!(
                "cannot process {} on Ended session",
                event.event_kind()
            ))),

            // IdleCutoff: /clear → split (user is clearly active again).
            (SessionStatus::IdleCutoff, SessionInputEvent::ClearCommand { .. }) => {
                Ok(SessionLifecycleDecision::SplitOnClear {
                    cli_kind: current_cli_kind.to_string(),
                })
            }

            // IdleCutoff: manual split → split.
            (SessionStatus::IdleCutoff, SessionInputEvent::ManualSplit { .. }) => {
                Ok(SessionLifecycleDecision::SplitManual {
                    cli_kind: current_cli_kind.to_string(),
                })
            }

            // IdleCutoff: activity event → Continue (does not auto-reactivate the session).
            (SessionStatus::IdleCutoff, SessionInputEvent::Activity { .. }) => {
                Ok(SessionLifecycleDecision::Continue)
            }

            // IdleCutoff: process exited → mark ended.
            (SessionStatus::IdleCutoff, SessionInputEvent::ProcessExit { exit_code, .. }) => {
                Ok(SessionLifecycleDecision::EndOnProcessExit {
                    exit_code: *exit_code,
                })
            }
        }
    }

    /// Check whether an `Active` session has exceeded the idle threshold.
    ///
    /// Returns `Some(SoftEndIdle)` when threshold is exceeded, `None` otherwise.
    /// Only fires for `Active` sessions; already-idle sessions are ignored.
    pub fn check_idle_cutoff(
        &self,
        current_status: SessionStatus,
        last_activity_at: i64,
        now_at: i64,
    ) -> Option<SessionLifecycleDecision> {
        if current_status != SessionStatus::Active {
            return None;
        }
        if is_idle_cutoff(last_activity_at, now_at, &self.config) {
            Some(SessionLifecycleDecision::SoftEndIdle)
        } else {
            None
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fixture helpers ───────────────────────────────────────────────────────

    fn process_start(at: i64) -> SessionInputEvent {
        SessionInputEvent::ProcessStart {
            pane_id: "p1".into(),
            workspace_id: "ws1".into(),
            cli_kind: "claude".into(),
            at,
        }
    }

    fn clear_cmd(at: i64) -> SessionInputEvent {
        SessionInputEvent::ClearCommand {
            pane_id: "p1".into(),
            workspace_id: "ws1".into(),
            at,
        }
    }

    fn manual_split(at: i64) -> SessionInputEvent {
        SessionInputEvent::ManualSplit {
            pane_id: "p1".into(),
            workspace_id: "ws1".into(),
            at,
        }
    }

    fn activity(at: i64) -> SessionInputEvent {
        SessionInputEvent::Activity {
            pane_id: "p1".into(),
            workspace_id: "ws1".into(),
            at,
        }
    }

    fn process_exit(at: i64, code: Option<i32>) -> SessionInputEvent {
        SessionInputEvent::ProcessExit {
            pane_id: "p1".into(),
            workspace_id: "ws1".into(),
            exit_code: code,
            at,
        }
    }

    fn cfg_secs(s: u64) -> IdleCutoffConfig {
        IdleCutoffConfig { threshold_secs: s }
    }

    // ── is_idle_cutoff ────────────────────────────────────────────────────────

    /// §I.1.3 fixture: exactly at threshold → true.
    #[test]
    fn fixture_idle_cutoff_at_exact_threshold() {
        let cfg = cfg_secs(60);
        assert!(is_idle_cutoff(100, 160, &cfg));
    }

    /// §I.1.3 fixture: one second below threshold → false.
    #[test]
    fn fixture_idle_cutoff_below_threshold() {
        let cfg = cfg_secs(60);
        assert!(!is_idle_cutoff(100, 159, &cfg));
    }

    /// §I.1.3 fixture: now <= last_activity → false (clock guard).
    #[test]
    fn fixture_idle_cutoff_clock_guard() {
        let cfg = cfg_secs(60);
        assert!(!is_idle_cutoff(200, 200, &cfg));
        assert!(!is_idle_cutoff(200, 150, &cfg));
    }

    /// §H7: default config is 1800 s (30 min).
    #[test]
    fn fixture_idle_cutoff_default_config_is_1800s() {
        let cfg = IdleCutoffConfig::default();
        assert_eq!(cfg.threshold_secs, 1800);
        assert!(is_idle_cutoff(0, 1800, &cfg));
        assert!(!is_idle_cutoff(0, 1799, &cfg));
    }

    // ── SessionBoundaryDetector — boundary type 1: ProcessStart ──────────────

    /// §C.1.1 boundary type 1: new process → StartNew boundary.
    #[test]
    fn fixture_session_start_by_process_spawn() {
        let detector = SessionBoundaryDetector::with_default_config();
        let events = [process_start(100)];
        let boundaries = detector.detect_boundaries(&events);
        assert_eq!(boundaries.len(), 1);
        assert_eq!(boundaries[0].reason, BoundaryReason::ProcessStart);
        assert_eq!(boundaries[0].at, 100);
        assert_eq!(
            boundaries[0].decision,
            SessionLifecycleDecision::StartNew {
                cli_kind: "claude".into(),
                source: "auto".into(),
            }
        );
    }

    // ── SessionBoundaryDetector — boundary type 2: ClearCommand ──────────────

    /// §C.1.1 boundary type 2: /clear → SplitOnClear boundary.
    #[test]
    fn fixture_session_split_by_clear() {
        let detector = SessionBoundaryDetector::with_default_config();
        let events = [process_start(100), clear_cmd(200)];
        let boundaries = detector.detect_boundaries(&events);
        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0].reason, BoundaryReason::ProcessStart);
        assert_eq!(boundaries[1].reason, BoundaryReason::ClearCommand);
        assert_eq!(boundaries[1].at, 200);
        assert_eq!(
            boundaries[1].decision,
            SessionLifecycleDecision::SplitOnClear {
                cli_kind: "claude".into()
            }
        );
    }

    // ── SessionBoundaryDetector — boundary type 3: ManualSplit ───────────────

    /// §C.1.1 boundary type 3: manual split → SplitManual boundary.
    #[test]
    fn fixture_session_split_by_manual() {
        let detector = SessionBoundaryDetector::with_default_config();
        let events = [process_start(100), manual_split(300)];
        let boundaries = detector.detect_boundaries(&events);
        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[1].reason, BoundaryReason::ManualSplit);
        assert_eq!(
            boundaries[1].decision,
            SessionLifecycleDecision::SplitManual {
                cli_kind: "claude".into()
            }
        );
    }

    // ── SessionBoundaryDetector — boundary type 4: IdleCutoff ────────────────

    /// §C.1.1 boundary type 4: idle threshold exceeded → SoftEndIdle boundary.
    #[test]
    fn fixture_session_end_by_idle_cutoff() {
        let detector = SessionBoundaryDetector::new(cfg_secs(60));
        // Activity at t=100, then next event at t=161 (> threshold).
        let events = [process_start(100), activity(100), activity(161)];
        let boundaries = detector.detect_boundaries(&events);
        // Expect: StartNew at 100, SoftEndIdle at 161.
        let idle: Vec<_> = boundaries
            .iter()
            .filter(|b| b.reason == BoundaryReason::IdleCutoff)
            .collect();
        assert_eq!(idle.len(), 1, "expected exactly one idle boundary");
        assert_eq!(idle[0].at, 161);
        assert_eq!(idle[0].decision, SessionLifecycleDecision::SoftEndIdle);
    }

    /// Idle boundary is NOT emitted when activity arrives before threshold.
    #[test]
    fn fixture_no_idle_cutoff_within_threshold() {
        let detector = SessionBoundaryDetector::new(cfg_secs(60));
        let events = [process_start(0), activity(30), activity(59)];
        let boundaries = detector.detect_boundaries(&events);
        assert!(
            boundaries
                .iter()
                .all(|b| b.reason != BoundaryReason::IdleCutoff),
            "should not emit idle boundary if threshold not reached"
        );
    }

    // ── SessionBoundaryDetector — combination scenarios ───────────────────────

    /// Consecutive /clear events produce two SplitOnClear boundaries without overlap.
    #[test]
    fn fixture_consecutive_clears_no_overlap() {
        let detector = SessionBoundaryDetector::with_default_config();
        let events = [process_start(100), clear_cmd(200), clear_cmd(300)];
        let boundaries = detector.detect_boundaries(&events);
        let clears: Vec<_> = boundaries
            .iter()
            .filter(|b| b.reason == BoundaryReason::ClearCommand)
            .collect();
        assert_eq!(clears.len(), 2);
        assert_eq!(clears[0].at, 200);
        assert_eq!(clears[1].at, 300);
    }

    /// Idle soft-end followed by ProcessStart correctly starts a new session.
    #[test]
    fn fixture_idle_then_new_process_start() {
        let detector = SessionBoundaryDetector::new(cfg_secs(60));
        let events = [
            process_start(0),
            activity(10),
            // Gap of 70 s — idle fires at next event.
            process_start(80),
        ];
        let boundaries = detector.detect_boundaries(&events);
        let reasons: Vec<_> = boundaries.iter().map(|b| &b.reason).collect();
        // Sequence: StartNew, IdleCutoff, StartNew
        assert!(reasons.contains(&&BoundaryReason::IdleCutoff));
        let starts: Vec<_> = boundaries
            .iter()
            .filter(|b| b.reason == BoundaryReason::ProcessStart)
            .collect();
        assert_eq!(starts.len(), 2, "expected two ProcessStart boundaries");
    }

    /// Manual split nested after /clear — both produce boundaries.
    #[test]
    fn fixture_clear_then_manual_split() {
        let detector = SessionBoundaryDetector::with_default_config();
        let events = [process_start(100), clear_cmd(200), manual_split(300)];
        let boundaries = detector.detect_boundaries(&events);
        assert_eq!(boundaries.len(), 3);
        assert_eq!(boundaries[1].reason, BoundaryReason::ClearCommand);
        assert_eq!(boundaries[2].reason, BoundaryReason::ManualSplit);
    }

    /// ProcessExit ends the active session; subsequent events do not produce boundaries.
    #[test]
    fn fixture_process_exit_ends_session() {
        let detector = SessionBoundaryDetector::with_default_config();
        let events = [
            process_start(100),
            activity(150),
            process_exit(200, Some(0)),
            // These should not produce any boundary (no active session).
            activity(250),
            clear_cmd(300),
        ];
        let boundaries = detector.detect_boundaries(&events);
        let exit_b: Vec<_> = boundaries
            .iter()
            .filter(|b| b.reason == BoundaryReason::ProcessExit)
            .collect();
        assert_eq!(exit_b.len(), 1);
        assert_eq!(exit_b[0].at, 200);
        assert_eq!(
            exit_b[0].decision,
            SessionLifecycleDecision::EndOnProcessExit { exit_code: Some(0) }
        );
        // No clear or activity boundary after exit.
        assert_eq!(
            boundaries
                .iter()
                .filter(|b| b.reason == BoundaryReason::ClearCommand)
                .count(),
            0
        );
    }

    // ── SessionLifecycleService — state machine ───────────────────────────────

    fn svc() -> SessionLifecycleService {
        SessionLifecycleService::with_default_config()
    }

    fn svc_cfg(s: u64) -> SessionLifecycleService {
        SessionLifecycleService::new(cfg_secs(s))
    }

    /// Active + ClearCommand → SplitOnClear.
    #[test]
    fn fixture_svc_active_clear_splits() {
        let result = svc()
            .process_event(SessionStatus::Active, &clear_cmd(10), "claude")
            .unwrap();
        assert_eq!(
            result,
            SessionLifecycleDecision::SplitOnClear {
                cli_kind: "claude".into()
            }
        );
    }

    /// Active + ManualSplit → SplitManual.
    #[test]
    fn fixture_svc_active_manual_splits() {
        let result = svc()
            .process_event(SessionStatus::Active, &manual_split(10), "claude")
            .unwrap();
        assert_eq!(
            result,
            SessionLifecycleDecision::SplitManual {
                cli_kind: "claude".into()
            }
        );
    }

    /// Active + ProcessStart → StartNew.
    #[test]
    fn fixture_svc_active_process_start() {
        let result = svc()
            .process_event(SessionStatus::Active, &process_start(10), "claude")
            .unwrap();
        assert_eq!(
            result,
            SessionLifecycleDecision::StartNew {
                cli_kind: "claude".into(),
                source: "auto".into(),
            }
        );
    }

    /// Active + Activity → Continue.
    #[test]
    fn fixture_svc_active_activity_continues() {
        let result = svc()
            .process_event(SessionStatus::Active, &activity(10), "claude")
            .unwrap();
        assert_eq!(result, SessionLifecycleDecision::Continue);
    }

    /// Active + ProcessExit → EndOnProcessExit.
    #[test]
    fn fixture_svc_active_process_exit() {
        let result = svc()
            .process_event(SessionStatus::Active, &process_exit(10, Some(1)), "claude")
            .unwrap();
        assert_eq!(
            result,
            SessionLifecycleDecision::EndOnProcessExit { exit_code: Some(1) }
        );
    }

    /// Ended + ProcessStart → StartNew (valid).
    #[test]
    fn fixture_svc_ended_process_start_ok() {
        let result = svc()
            .process_event(SessionStatus::Ended, &process_start(10), "")
            .unwrap();
        assert!(matches!(result, SessionLifecycleDecision::StartNew { .. }));
    }

    /// Ended + ClearCommand → InvalidStateTransition.
    #[test]
    fn fixture_svc_ended_clear_invalid() {
        let err = svc()
            .process_event(SessionStatus::Ended, &clear_cmd(10), "claude")
            .unwrap_err();
        assert!(
            matches!(err, SessionError::InvalidStateTransition(_)),
            "expected InvalidStateTransition, got {err:?}"
        );
    }

    /// Ended + ManualSplit → InvalidStateTransition.
    #[test]
    fn fixture_svc_ended_manual_split_invalid() {
        let err = svc()
            .process_event(SessionStatus::Ended, &manual_split(10), "claude")
            .unwrap_err();
        assert!(matches!(err, SessionError::InvalidStateTransition(_)));
    }

    /// Archived + any event → InvalidStateTransition.
    #[test]
    fn fixture_svc_archived_any_invalid() {
        for event in [
            process_start(1),
            clear_cmd(1),
            manual_split(1),
            activity(1),
            process_exit(1, None),
        ] {
            let err = svc()
                .process_event(SessionStatus::Archived, &event, "claude")
                .unwrap_err();
            assert!(
                matches!(err, SessionError::InvalidStateTransition(_)),
                "Archived + {} should be InvalidStateTransition",
                event.event_kind()
            );
        }
    }

    /// IdleCutoff + ProcessStart → StartNew.
    #[test]
    fn fixture_svc_idle_cutoff_process_start() {
        let result = svc()
            .process_event(SessionStatus::IdleCutoff, &process_start(10), "")
            .unwrap();
        assert!(matches!(result, SessionLifecycleDecision::StartNew { .. }));
    }

    /// IdleCutoff + ClearCommand → SplitOnClear.
    #[test]
    fn fixture_svc_idle_cutoff_clear() {
        let result = svc()
            .process_event(SessionStatus::IdleCutoff, &clear_cmd(10), "claude")
            .unwrap();
        assert_eq!(
            result,
            SessionLifecycleDecision::SplitOnClear {
                cli_kind: "claude".into()
            }
        );
    }

    // ── SessionLifecycleService — check_idle_cutoff ───────────────────────────

    /// Active session exceeding threshold → SoftEndIdle.
    #[test]
    fn fixture_svc_check_idle_cutoff_fires() {
        let svc = svc_cfg(60);
        let decision = svc.check_idle_cutoff(SessionStatus::Active, 100, 161);
        assert_eq!(decision, Some(SessionLifecycleDecision::SoftEndIdle));
    }

    /// Active session below threshold → None.
    #[test]
    fn fixture_svc_check_idle_cutoff_no_fire() {
        let svc = svc_cfg(60);
        assert_eq!(svc.check_idle_cutoff(SessionStatus::Active, 100, 159), None);
    }

    /// Non-Active session → check_idle_cutoff always returns None.
    #[test]
    fn fixture_svc_check_idle_cutoff_only_for_active() {
        let svc = svc_cfg(1);
        for status in [
            SessionStatus::Ended,
            SessionStatus::IdleCutoff,
            SessionStatus::Archived,
        ] {
            // canonical SessionStatus has no Copy; clone for call, keep original for msg.
            assert_eq!(
                svc.check_idle_cutoff(status.clone(), 0, 9999),
                None,
                "check_idle_cutoff should return None for {status:?}"
            );
        }
    }
}

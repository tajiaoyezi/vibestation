//! MVP-19 W2-B · backend session IPC contract audit.
//!
//! This is a source-level guard for the W2-B/W2-C shared contract: Tauri invoke
//! keys are Rust fn names, events keep the colon style, and ACL identifiers are
//! wired into the default capability.

#[test]
fn session_command_names_are_registered_as_hc6_snake_case() {
    let lib_rs = include_str!("../src/lib.rs");
    for command in [
        "session_start",
        "session_end",
        "session_bind_commit",
        "session_unbind",
        "session_list",
        "session_get_detail",
        "session_rebind",
        "session_recalc",
    ] {
        assert!(
            lib_rs.contains(&format!("fn {command}(")),
            "missing #[tauri::command] function {command}"
        );
        assert!(
            lib_rs.contains(&format!("            {command},")),
            "missing generate_handler registration for {command}"
        );
        assert!(
            !command.contains(':'),
            "Tauri invoke key must be snake_case"
        );
    }
}

#[test]
fn session_event_names_match_hc6_colon_contract() {
    let lib_rs = include_str!("../src/lib.rs");
    for (constant, event_name) in [
        ("SESSION_STARTED_EVENT", "session:started"),
        ("SESSION_ENDED_EVENT", "session:ended"),
        ("SESSION_COMMIT_BOUND_EVENT", "session:commit-bound"),
        ("SESSION_COMMIT_UNBOUND_EVENT", "session:commit-unbound"),
        ("SESSION_LINK_UPDATED_EVENT", "session:link-updated"),
        ("SESSION_ERROR_EVENT", "session:error"),
    ] {
        assert!(
            lib_rs.contains(&format!("const {constant}: &str = \"{event_name}\";")),
            "missing event constant {constant}={event_name}"
        );
        assert!(
            lib_rs.contains(constant) && lib_rs.contains("app.emit("),
            "event constant {constant} must be emitted"
        );
        assert!(event_name.contains(':'), "event name must keep colon style");
    }
}

#[test]
fn session_acl_permissions_are_declared_and_enabled_by_default() {
    let session_toml = include_str!("../permissions/session.toml");
    let default_json = include_str!("../capabilities/default.json");
    for (identifier, command) in [
        ("allow-session-start", "session_start"),
        ("allow-session-end", "session_end"),
        ("allow-session-bind-commit", "session_bind_commit"),
        ("allow-session-unbind", "session_unbind"),
        ("allow-session-list", "session_list"),
        ("allow-session-get-detail", "session_get_detail"),
        ("allow-session-rebind", "session_rebind"),
        ("allow-session-recalc", "session_recalc"),
    ] {
        assert!(
            session_toml.contains(&format!("identifier = \"{identifier}\"")),
            "missing permission identifier {identifier}"
        );
        assert!(
            session_toml.contains(&format!("commands.allow = [\"{command}\"]")),
            "permission {identifier} must allow command {command}"
        );
        assert!(
            default_json.contains(&format!("\"{identifier}\"")),
            "default capability must include {identifier}"
        );
    }
}

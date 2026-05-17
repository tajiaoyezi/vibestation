//! MVP-20 Phase A M2 · rollback backend IPC/ACL contract audit.
//!
//! Tauri invoke keys are Rust snake_case function names. The product spec uses
//! `rollback:*` as human-readable namespace labels, while runtime permissions
//! must allow the concrete command names.

#[test]
fn rollback_command_names_are_registered_as_snake_case() {
    let lib_rs = include_str!("../src/lib.rs");
    for command in [
        "rollback_preview",
        "rollback_execute",
        "rollback_abort",
        "rollback_status",
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
fn rollback_acl_permissions_are_declared_and_enabled_by_default() {
    let rollback_toml = include_str!("../permissions/rollback_ops.toml");
    let default_json = include_str!("../capabilities/default.json");
    for (identifier, command) in [
        ("allow-rollback-preview", "rollback_preview"),
        ("allow-rollback-execute", "rollback_execute"),
        ("allow-rollback-abort", "rollback_abort"),
        ("allow-rollback-status", "rollback_status"),
    ] {
        assert!(
            rollback_toml.contains(&format!("identifier = \"{identifier}\"")),
            "missing permission identifier {identifier}"
        );
        assert!(
            rollback_toml.contains(&format!("commands.allow = [\"{command}\"]")),
            "permission {identifier} must allow command {command}"
        );
        assert!(
            default_json.contains(&format!("\"{identifier}\"")),
            "default capability must include {identifier}"
        );
    }
}

#[test]
fn rollback_bindings_are_exported_from_build_script_and_barrel() {
    let build_rs = include_str!("../build.rs");
    let expected = [
        "RollbackPreview",
        "RollbackProgress",
        "RollbackAbortResult",
        "RollbackStatus",
        "RollbackCommitEntry",
    ];
    for ty in expected {
        assert!(
            build_rs.contains(&format!("{ty}::export_all(&config)")),
            "build.rs must export {ty}"
        );
        assert!(
            build_rs.contains(&format!("export type {{ {ty} }} from \\\"./{ty}\\\";")),
            "bindings index barrel must export {ty}"
        );
    }
}

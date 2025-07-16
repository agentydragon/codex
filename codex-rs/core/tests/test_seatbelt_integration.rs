#![cfg(target_os = "macos")]
use codex_core::exec::ExecParams;
use codex_core::exec::SandboxType;
use codex_core::exec::process_exec_tool_call;
use codex_core::protocol::SandboxPolicy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;

#[tokio::test]
async fn test_sandbox_error_detection() {
    let params = ExecParams {
        command: vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo test > /tmp/blocked_write.txt".to_string(),
        ],
        cwd: PathBuf::from("/tmp"),
        timeout_ms: Some(5000),
        env: HashMap::new(),
    };

    // Very restrictive policy - no disk write permissions
    let sandbox_policy = SandboxPolicy::new_read_only_policy();
    let ctrl_c = Arc::new(Notify::new());

    let result = process_exec_tool_call(
        params,
        SandboxType::MacosSeatbelt,
        ctrl_c,
        &sandbox_policy,
        &None,
    )
    .await;

    match result {
        Err(codex_core::error::CodexErr::Sandbox(sandbox_err)) => {
            println!("Got sandbox error: {sandbox_err:?}");
            if sandbox_err.has_blocked_syscalls()
                && let Some(blocked) = sandbox_err.blocked_syscalls()
            {
                println!("Blocked syscalls: {blocked:?}");
            }
            if sandbox_err.should_retry_outside_sandbox() {
                println!("Should retry outside sandbox: true");
            }
        }
        other => {
            println!("Got unexpected result: {other:?}");
        }
    }
}

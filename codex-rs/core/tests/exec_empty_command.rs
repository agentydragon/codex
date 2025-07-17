use codex_core::exec::ExecParams;
use codex_core::exec::SandboxType;
use codex_core::exec::process_exec_tool_call;
use codex_core::protocol::SandboxPolicy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;

/// Ensure that an empty command returns immediate success without spawning.
#[tokio::test]
async fn test_empty_command_immediate_success() {
    let params = ExecParams {
        command: vec!["".to_string()],
        cwd: PathBuf::from("/tmp"),
        timeout_ms: Some(1000),
        env: HashMap::new(),
    };

    let sandbox_policy = SandboxPolicy::new_full_auto_policy();
    let ctrl_c = Arc::new(Notify::new());

    let result = process_exec_tool_call(params, SandboxType::None, ctrl_c, &sandbox_policy, &None)
        .await
        .expect("empty command should succeed");

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

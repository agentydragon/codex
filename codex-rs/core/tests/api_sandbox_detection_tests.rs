#![cfg(target_os = "macos")]
//! API-level unit tests for sandbox violation detection
//!
//! Tests the public codex API to ensure it correctly exposes sandbox violation
//! information through FunctionCallOutputPayload responses.

use codex_core::FunctionCallOutputPayload;
use codex_core::error::CodexErr;
use codex_core::exec::ExecParams;
use codex_core::exec::SandboxType;
use codex_core::protocol::SandboxPolicy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;

/// Helper function that simulates the API response creation from execution results
async fn simulate_api_response(
    command: Vec<String>,
    workdir: Option<String>,
    sandbox_policy: SandboxPolicy,
    sandbox_type: SandboxType,
) -> FunctionCallOutputPayload {
    // Convert to ExecParams (simulating the API's parameter conversion)
    let exec_params = ExecParams {
        command,
        cwd: workdir.map_or_else(|| PathBuf::from("/tmp"), PathBuf::from),
        timeout_ms: Some(5000),
        env: HashMap::new(),
    };

    let ctrl_c = Arc::new(Notify::new());

    // Execute the command (this is the core logic we're testing)
    match codex_core::exec::process_exec_tool_call(
        exec_params,
        sandbox_type,
        ctrl_c,
        &sandbox_policy,
        &None,
    )
    .await
    {
        Ok(output) => {
            // Successful execution - return success response
            FunctionCallOutputPayload {
                content: format!(
                    "Command completed successfully.\nExit code: {}\nStdout: {}\nStderr: {}",
                    output.exit_code, output.stdout, output.stderr
                ),
                success: Some(true),
            }
        }
        Err(error) => {
            // Error occurred - create appropriate API response
            let (content, success) = match &error {
                CodexErr::Sandbox(sandbox_err) => {
                    // This is where our sandbox detection logic is exposed to the API
                    if sandbox_err.has_blocked_syscalls() {
                        let blocked_ops = sandbox_err
                            .blocked_syscalls()
                            .map(|ops| ops.join(", "))
                            .unwrap_or_else(|| "unknown".to_string());

                        let should_retry = sandbox_err.should_retry_outside_sandbox();

                        (
                            format!(
                                "Sandbox blocked syscalls: {}\nRetry outside sandbox: {}\nError: {}",
                                blocked_ops, should_retry, error
                            ),
                            Some(false),
                        )
                    } else {
                        (
                            format!("Sandbox error (no specific syscalls detected): {}", error),
                            Some(false),
                        )
                    }
                }
                _ => (format!("Non-sandbox error: {}", error), Some(false)),
            };

            FunctionCallOutputPayload { content, success }
        }
    }
}

/// Test: API correctly reports normal command failure (no sandbox violation)
#[tokio::test]
async fn api_test_normal_command_failure_no_sandbox_violation() {
    let output = simulate_api_response(
        vec!["sh".to_string(), "-c".to_string(), "exit 42".to_string()],
        None,
        SandboxPolicy::new_full_auto_policy(),
        SandboxType::None, // No sandbox
    )
    .await;

    println!("API Response: {}", output.content);

    // Should be successful execution with exit code 42
    assert_eq!(output.success, Some(true));
    assert!(output.content.contains("Exit code: 42"));
    assert!(output.content.contains("Command completed successfully"));

    // Should NOT contain sandbox-related messages
    assert!(!output.content.contains("Sandbox blocked"));
    assert!(!output.content.contains("Retry outside sandbox"));
}

/// Test: API correctly exposes file write violation with specific syscall info
#[tokio::test]
async fn api_test_seatbelt_file_write_violation_exposure() {
    let output = simulate_api_response(
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo test > /tmp/blocked_api_test.txt".to_string(),
        ],
        None,
        SandboxPolicy::new_read_only_policy(), // Restrictive policy
        SandboxType::MacosSeatbelt,
    )
    .await;

    println!("API Response: {}", output.content);

    // Should be a failure
    assert_eq!(output.success, Some(false));

    // Should contain specific sandbox violation information
    assert!(output.content.contains("Sandbox blocked syscalls"));
    assert!(output.content.contains("file-write"));
    assert!(output.content.contains("/tmp/blocked_api_test.txt"));
    assert!(output.content.contains("Retry outside sandbox: true"));

    // Should NOT contain generic error messages
    assert!(!output.content.contains("no specific syscalls detected"));
}

/// Test: API correctly exposes process execution violation
#[tokio::test]
async fn api_test_seatbelt_process_exec_violation_exposure() {
    let output = simulate_api_response(
        vec!["nonexistent_blocked_command".to_string()],
        None,
        SandboxPolicy::new_read_only_policy(),
        SandboxType::MacosSeatbelt,
    )
    .await;

    println!("API Response: {}", output.content);

    // Should be a failure
    assert_eq!(output.success, Some(false));

    // Should contain specific sandbox violation information
    assert!(output.content.contains("Sandbox blocked syscalls"));
    assert!(output.content.contains("process-exec"));
    assert!(output.content.contains("nonexistent_blocked_command"));
    assert!(output.content.contains("Retry outside sandbox: true"));
}

/// Test: API correctly handles successful sandbox execution
#[tokio::test]
async fn api_test_successful_sandbox_execution() {
    let output = simulate_api_response(
        vec!["echo".to_string(), "hello from sandbox".to_string()],
        None,
        SandboxPolicy::new_full_auto_policy(), // Permissive policy
        SandboxType::MacosSeatbelt,
    )
    .await;

    println!("API Response: {}", output.content);

    // Should be successful
    assert_eq!(output.success, Some(true));
    assert!(output.content.contains("Command completed successfully"));
    assert!(output.content.contains("Exit code: 0"));
    assert!(output.content.contains("hello from sandbox"));

    // Should NOT contain error messages
    assert!(!output.content.contains("Sandbox blocked"));
    assert!(!output.content.contains("error"));
}

/// Test: API distinguishes between sandbox and non-sandbox errors
#[tokio::test]
async fn api_test_error_type_distinction() {
    // Test with a command that will fail due to sandbox (file write)
    let sandbox_output = simulate_api_response(
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo test > /tmp/blocked.txt".to_string(),
        ],
        None,
        SandboxPolicy::new_read_only_policy(),
        SandboxType::MacosSeatbelt,
    )
    .await;

    // Test with a command that will fail normally (command not found, no sandbox)
    let normal_output = simulate_api_response(
        vec!["definitely_nonexistent_command_12345".to_string()],
        None,
        SandboxPolicy::new_full_auto_policy(),
        SandboxType::None,
    )
    .await;

    println!("Sandbox error: {}", sandbox_output.content);
    println!("Normal error: {}", normal_output.content);

    // Sandbox error should contain specific sandbox information
    assert!(sandbox_output.content.contains("Sandbox blocked syscalls"));
    assert!(sandbox_output.content.contains("Retry outside sandbox"));

    // Normal error should NOT contain sandbox information
    assert!(!normal_output.content.contains("Sandbox blocked"));
    assert!(!normal_output.content.contains("Retry outside sandbox"));

    // Both should indicate their nature appropriately
    assert_eq!(sandbox_output.success, Some(false));
    // The normal error might succeed with non-zero exit code depending on implementation
}

/// Test: API payload serialization works correctly
#[tokio::test]
async fn api_test_payload_serialization() {
    let output = simulate_api_response(
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo blocked > /tmp/serialization_test.txt".to_string(),
        ],
        None,
        SandboxPolicy::new_read_only_policy(),
        SandboxType::MacosSeatbelt,
    )
    .await;

    // Test that the payload can be serialized to JSON (as the API would do)
    let json_result = serde_json::to_string(&output);
    assert!(json_result.is_ok(), "Should serialize to JSON");

    let json_str = json_result.unwrap();
    println!("Serialized payload: {}", json_str);

    // Should be a plain string (not nested object) due to custom Serialize impl
    assert!(json_str.starts_with('"'));
    assert!(json_str.ends_with('"'));

    // Should contain our sandbox information in the serialized string
    assert!(json_str.contains("Sandbox blocked syscalls"));
}

/// Test: API provides actionable information for retry logic
#[tokio::test]
async fn api_test_retry_logic_information() {
    let output = simulate_api_response(
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo test > /tmp/retry_test.txt".to_string(),
        ],
        None,
        SandboxPolicy::new_read_only_policy(),
        SandboxType::MacosSeatbelt,
    )
    .await;

    println!("API Response: {}", output.content);

    // Should provide clear guidance on whether retry is recommended
    assert!(output.content.contains("Retry outside sandbox: true"));

    // Should provide specific information about what was blocked
    assert!(output.content.contains("file-write"));
    assert!(output.content.contains("/tmp/retry_test.txt"));

    // Consumer can parse this information to make retry decisions
    let should_retry = output.content.contains("Retry outside sandbox: true");
    let blocked_file_op = output.content.contains("file-write");

    assert!(
        should_retry,
        "API should indicate retry is worth attempting"
    );
    assert!(
        blocked_file_op,
        "API should specify what type of operation was blocked"
    );
}

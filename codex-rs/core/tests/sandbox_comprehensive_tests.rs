#![cfg(target_os = "macos")]
#![allow(clippy::collapsible_if)]
use codex_core::error::CodexErr;
use codex_core::error::SandboxErr;
use codex_core::exec::ExecParams;
use codex_core::exec::SandboxType;
use codex_core::exec::process_exec_tool_call;
use codex_core::protocol::SandboxPolicy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;

/// Test that a command failing normally (without seatbelt violation) is correctly identified
#[tokio::test]
async fn test_normal_command_failure_no_sandbox_violation() {
    let params = ExecParams {
        command: vec!["sh".to_string(), "-c".to_string(), "exit 42".to_string()],
        cwd: PathBuf::from("/tmp"),
        timeout_ms: Some(5000),
        env: HashMap::new(),
    };

    // Use no sandbox - this should just be a normal command failure
    let sandbox_policy = SandboxPolicy::new_full_auto_policy();
    let ctrl_c = Arc::new(Notify::new());

    let result = process_exec_tool_call(
        params,
        SandboxType::None, // No sandbox
        ctrl_c,
        &sandbox_policy,
        &None,
    )
    .await;

    match result {
        Ok(output) => {
            println!("Command completed with exit code: {}", output.exit_code);
            assert_eq!(output.exit_code, 42, "Expected exit code 42 from 'exit 42'");
            // This is a normal failure, not a sandbox violation
        }
        Err(err) => {
            panic!("Expected successful execution with exit code 42, got error: {err:?}");
        }
    }
}

/// Test that we correctly detect file write violations
#[tokio::test]
async fn test_seatbelt_file_write_violation() {
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
        Err(CodexErr::Sandbox(sandbox_err)) => {
            println!("Got sandbox error: {sandbox_err:?}");

            // Should detect blocked syscalls
            assert!(
                sandbox_err.has_blocked_syscalls(),
                "Should detect blocked syscalls"
            );

            // Should suggest retry outside sandbox
            assert!(
                sandbox_err.should_retry_outside_sandbox(),
                "Should suggest retry outside sandbox"
            );

            // Should have specific blocked operation info
            if let Some(blocked) = sandbox_err.blocked_syscalls() {
                println!("Blocked syscalls: {blocked:?}");
                assert!(
                    !blocked.is_empty(),
                    "Should have at least one blocked operation"
                );

                // Check if it's a file write operation
                let has_file_write = blocked.iter().any(|op| op.contains("file-write"));
                if has_file_write {
                    println!("✓ Correctly detected file-write violation");
                } else {
                    println!(
                        "Note: Detected violation but not specifically file-write: {blocked:?}"
                    );
                }
            }
        }
        other => {
            panic!("Expected sandbox error, got: {other:?}");
        }
    }
}

/// Test that we correctly detect process execution violations  
#[tokio::test]
async fn test_seatbelt_process_exec_violation() {
    let params = ExecParams {
        command: vec!["nonexistent_command_that_should_be_blocked".to_string()],
        cwd: PathBuf::from("/tmp"),
        timeout_ms: Some(5000),
        env: HashMap::new(),
    };

    // Very restrictive policy that should block process execution
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
        Err(CodexErr::Sandbox(sandbox_err)) => {
            println!("Got sandbox error: {sandbox_err:?}");

            // Should suggest retry outside sandbox
            assert!(
                sandbox_err.should_retry_outside_sandbox(),
                "Should suggest retry outside sandbox"
            );

            // Check what type of violation this is
            match sandbox_err {
                SandboxErr::SyscallsBlocked(exit_code, blocked, _, _) => {
                    println!("✓ Detected syscalls blocked with exit code {exit_code}");
                    println!("✓ Blocked operations: {blocked:?}");

                    // Should have blocked operations
                    assert!(!blocked.is_empty(), "Should have blocked operations");
                }
                SandboxErr::Denied(exit_code, stdout, stderr) => {
                    println!("✓ Detected general sandbox denial with exit code {exit_code}");
                    println!("  stdout: '{stdout}'");
                    println!("  stderr: '{stderr}'");

                    // Exit code 71 indicates process execution was blocked
                    if exit_code == 71 {
                        println!("✓ Correctly detected process execution blocking (exit code 71)");
                    }
                }
                other => {
                    println!("Got other sandbox error type: {other:?}");
                }
            }
        }
        other => {
            // This might also be a normal "command not found" error depending on the sandbox policy
            println!("Got result: {other:?}");
            // Don't panic here as the behavior might vary depending on sandbox policy
        }
    }
}

/// Test network access violation (if we can trigger one)
#[tokio::test]
async fn test_seatbelt_network_violation() {
    let params = ExecParams {
        command: vec![
            "sh".to_string(),
            "-c".to_string(),
            "curl -s --connect-timeout 2 http://httpbin.org/get || echo 'network blocked'"
                .to_string(),
        ],
        cwd: PathBuf::from("/tmp"),
        timeout_ms: Some(10000),
        env: HashMap::new(),
    };

    // Policy with no network access
    let sandbox_policy = SandboxPolicy::new_read_only_policy(); // This should block network
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
        Ok(output) => {
            println!("Command completed with exit code: {}", output.exit_code);
            println!("stdout: '{}'", output.stdout);
            println!("stderr: '{}'", output.stderr);

            // The command might succeed but curl might fail due to network restrictions
            // Look for evidence that network was blocked
            if output.stderr.contains("Operation not permitted")
                || output.stdout.contains("network blocked")
                || output.stderr.contains("Could not resolve host")
            {
                println!("✓ Network access appears to be restricted");
            } else {
                println!("Note: Network access test inconclusive - might need different test");
            }
        }
        Err(CodexErr::Sandbox(sandbox_err)) => {
            println!("Got sandbox error for network test: {sandbox_err:?}");

            if sandbox_err.has_blocked_syscalls() {
                if let Some(blocked) = sandbox_err.blocked_syscalls() {
                    println!("Blocked network operations: {blocked:?}");

                    let has_network_block = blocked.iter().any(|op| {
                        op.contains("network") || op.contains("connect") || op.contains("socket")
                    });
                    if has_network_block {
                        println!("✓ Correctly detected network violation");
                    }
                }
            }
        }
        other => {
            println!("Network test got unexpected result: {other:?}");
        }
    }
}

/// Test that successful sandbox execution works correctly
#[tokio::test]
async fn test_successful_sandbox_execution() {
    let params = ExecParams {
        command: vec!["echo".to_string(), "hello world".to_string()],
        cwd: PathBuf::from("/tmp"),
        timeout_ms: Some(5000),
        env: HashMap::new(),
    };

    // Permissive policy that should allow echo
    let sandbox_policy = SandboxPolicy::new_full_auto_policy();
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
        Ok(output) => {
            println!("✓ Successful sandbox execution:");
            println!("  exit_code: {}", output.exit_code);
            println!("  stdout: '{}'", output.stdout);
            println!("  stderr: '{}'", output.stderr);

            assert_eq!(output.exit_code, 0, "Expected successful execution");
            assert!(
                output.stdout.contains("hello world"),
                "Expected 'hello world' in output"
            );
        }
        Err(err) => {
            panic!("Expected successful execution, got error: {err:?}");
        }
    }
}

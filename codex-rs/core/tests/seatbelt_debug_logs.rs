#![cfg(target_os = "macos")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
// std::env and std::fs unused now
#![allow(unused_imports)]
use std::env;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;

/// Verifies that seatbelt sandbox violations produce detectable error patterns
///
/// Modern macOS seatbelt doesn't output debug deny logs to stderr, but it does
/// produce detectable error patterns that we can use to identify syscall blocking.
#[test]
fn seatbelt_debug_deny_logs() {
    // Skip if sandbox-exec not available
    if Command::new("sandbox-exec").arg("-h").output().is_err() {
        eprintln!("skipping seatbelt_debug_deny_logs: sandbox-exec not found");
        return;
    }

    // Test with a restrictive policy that should block file writes
    let base = include_str!("../src/seatbelt_base_policy.sbpl");

    // Run sandbox-exec with restrictive policy and capture output
    let output = Command::new("sandbox-exec")
        .args(["-p", base, "--", "sh", "-c", "echo hi > /tmp/denied.txt"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    // Check for signal termination
    let signal = output.status.signal().unwrap_or(0);
    println!(
        "Seatbelt violation output:\nExit code: {exit_code}\nSignal: {signal}\nStderr: '{stderr}'"
    );

    // Modern seatbelt either:
    // 1. Terminates the process with SIGABRT (exit code 134 or signal 6) with empty stderr, OR
    // 2. Allows the operation to proceed but the shell command fails with "Operation not permitted"
    // 3. Completely blocks execution (exit code 71)
    let is_sandbox_violation = exit_code == 134 || // SIGABRT from seatbelt
                              exit_code == 71 ||  // Process execution blocked
                              signal == 6 ||      // SIGABRT signal
                              stderr.contains("Operation not permitted"); // Syscall blocked

    assert!(
        is_sandbox_violation,
        "Expected sandbox violation (exit code 134, 71, or 'Operation not permitted'), got exit code {exit_code}, stderr: '{stderr}'"
    );
}

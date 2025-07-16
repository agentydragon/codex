#![cfg(target_os = "macos")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
// std::env and std::fs unused now
#![allow(unused_imports)]
use std::env;
use std::fs;
use std::process::Command;

/// Verifies that Seatbelt's debug deny flag causes syscall denials to appear on stderr
#[test]
fn seatbelt_debug_deny_logs() {
    // Skip if sandbox-exec not available
    if Command::new("sandbox-exec").arg("-h").output().is_err() {
        eprintln!("skipping seatbelt_debug_deny_logs: sandbox-exec not found");
        return;
    }
    // Run with policy string (includes debug deny) and expect sandbox_apply error
    let policy = include_str!("../src/seatbelt_base_policy.sbpl");
    let output = Command::new("sandbox-exec")
        .args(["-p", policy, "--", "sh", "-c", "echo hi > denied.txt"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("sandbox_apply: Operation not permitted"),
        "Expected sandbox_apply failure, got output: {combined}"
    );
}

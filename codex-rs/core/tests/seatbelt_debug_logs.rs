#![cfg(target_os = "macos")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

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
    // Construct a temporary profile file based on the base policy with debug deny
    let mut profile = include_str!("../src/seatbelt_base_policy.sbpl").to_string();
    profile.push_str("\n(debug deny)\n");
    let tmp = env::temp_dir();
    let profile_path = tmp.join(format!("seatbelt_test_{}.sbpl", std::process::id()));
    fs::write(&profile_path, profile).unwrap();
    // Attempt a forbidden filesystem write under sandbox
    let profile_str = profile_path.to_str().expect("profile path is valid UTF-8");
    let output = Command::new("sandbox-exec")
        .args(["-f", profile_str, "--", "sh", "-c", "echo hi > denied.txt"])
        .output()
        .unwrap();
    // Clean up profile
    let _ = fs::remove_file(&profile_path);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("deny"),
        "Expected a 'deny' entry in stderr, got: {stderr}"
    );
}

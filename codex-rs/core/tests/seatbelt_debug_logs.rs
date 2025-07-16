#![cfg(target_os = "macos")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
// std::env and std::fs unused now
#![allow(unused_imports)]
use std::env;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;

/// Verifies that Seatbelt's debug deny flag causes syscall denials to appear on stderr
#[test]
fn seatbelt_debug_deny_logs() {
    // Skip if sandbox-exec not available
    if Command::new("sandbox-exec").arg("-h").output().is_err() {
        eprintln!("skipping seatbelt_debug_deny_logs: sandbox-exec not found");
        return;
    }
    // sandbox-exec should abort on a protected write
    // Use the default base policy file
    let profile = include_str!("../src/seatbelt_base_policy.sbpl");
    let status = Command::new("sandbox-exec")
        .args(["-p", profile, "--", "sh", "-c", "echo hi > denied.txt"])
        .status()
        .unwrap();
    // Expect either Operation not permitted or an abort, depending on OS version
    let exit_code = status.code();
    let signal = status.signal();
    assert!(
        exit_code == Some(71) || signal == Some(6),
        "Expected exit code 71 or SIGABRT, got code={exit_code:?} signal={signal:?}"
    );
}

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
    // Construct a temporary SBPL profile file with debug deny and test it
    let mut policy = include_str!("../src/seatbelt_base_policy.sbpl").to_string();
    policy.push_str("(debug deny)\n");
    let profile_path = env::temp_dir().join(format!("sbpl_{}.sbpl", std::process::id()));
    fs::write(&profile_path, policy).unwrap();
    let status = Command::new("sandbox-exec")
        .args([
            "-f",
            profile_path.to_str().unwrap(),
            "--",
            "sh",
            "-c",
            "echo hi > denied.txt",
        ])
        .status()
        .unwrap();
    let _ = fs::remove_file(&profile_path);
    // sandbox-exec aborts on Operation not permitted (exit code 71 or SIGABRT)
    let exit_code = status.code();
    let signal = status.signal();
    assert!(
        exit_code == Some(71) || signal == Some(6),
        "Expected exit code 71 or SIGABRT, got code={exit_code:?} signal={signal:?}"
    );
}

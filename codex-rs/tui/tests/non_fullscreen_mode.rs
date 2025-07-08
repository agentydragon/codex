#![allow(clippy::expect_used, clippy::unwrap_used)]
use expectrl::spawn;
use std::io::Read;

/// Verify that non_fullscreen_mode skips alternate-screen and appends output inline.
#[test]
fn non_fullscreen_scrollback_appended() {
    // Enforce a 30s timeout so hanging tests abort the process.
    use std::process;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;
    fn start_timeout(name: &str) -> Arc<AtomicBool> {
        let done = Arc::new(AtomicBool::new(false));
        let watcher = Arc::clone(&done);
        let name = name.to_string();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(30));
            if !watcher.load(Ordering::SeqCst) {
                eprintln!("test '{name}' timed out after 30 seconds");
                process::exit(1);
            }
        });
        done
    }
    let _timeout = start_timeout("non_fullscreen_scrollback_appended");
    // Spawn the TUI in non-fullscreen mode, skip git/login, send empty input to exit.
    let mut p = spawn(
        // Skip login and git repo check; set non_fullscreen_mode, then exit on newline.
        "bash -lc 'OPENAI_API_KEY=unused cargo run --quiet --bin codex-tui -- -c tui.non_fullscreen_mode=true --skip-git-repo-check --no-config-reload -- --'",
    )
    .expect("failed to spawn process");
    // send newline to exit prompt
    p.send_line("").unwrap();
    // read until EOF
    let mut output = String::new();
    p.read_to_string(&mut output).unwrap();
    // DEBUG: dump full output for inspection
    eprintln!("=== OUTPUT BEGIN ===\n{output}\n=== OUTPUT END ===");
    // should not enter alternate screen (CSI ?1049h)
    assert!(!output.contains("\x1B[?1049h"));
    // prompt header should appear inline
    assert!(output.contains("Messages (tab to focus)"));
    // internal scrollbar should be disabled (no track or thumb symbols)
    assert!(!output.contains('│'));
    assert!(!output.contains('█'));
    // Signal watcher to cancel timeout (test completed)
    _timeout.store(true, Ordering::SeqCst);
}

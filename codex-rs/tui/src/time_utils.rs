use std::time::Duration;

/// Format a duration in a short human-readable form.
///
/// Rules:
///   • <1s  → "{millis}ms"
///   • <60s → "{secs}s"
///   • otherwise → "{minutes}:{seconds:02}"
pub fn format_duration_short(duration: Duration) -> String {
    if duration < Duration::from_secs(1) {
        format!("{}ms", duration.as_millis())
    } else if duration < Duration::from_secs(60) {
        format!("{}s", duration.as_secs())
    } else {
        let secs = duration.as_secs();
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

/// Format an `elapsed` duration (i.e. "ago") in a short form.
///
/// Rules:
///   • <60s   → "{secs}s ago"
///   • <1h    → "{mins}m ago"
///   • else   → "{hours}h ago"
pub fn format_elapsed_short(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
    }
}

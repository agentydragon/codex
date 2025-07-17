use ratatui::style::Color;
use std::time::Duration;

/// Build the annotation ("✓ 123ms" or "✗ 5 123ms") for a finished command and
/// return the padded string together with its colour.
///
/// The padding width is dynamic so callers do not need to rely on the previous
/// magic constant of 9. The returned string is left-padded to `width`, where
/// `width` is the length of the annotation plus a single trailing space so
/// that multi-line commands can indent properly.
pub fn render_exec_annotation(exit_code: i32, duration: Duration) -> (String, Color) {
    let timing = crate::time_utils::format_duration_short(duration);

    let ann = if exit_code == 0 {
        format!("✓ {timing}")
    } else {
        format!("✗ {exit_code} {timing}")
    };

    // Pad with a trailing space to separate from "$" that follows.
    let padded = format!("{ann} ");

    let color = if exit_code == 0 {
        Color::Green
    } else {
        Color::Red
    };

    (padded, color)
}

use crate::parse_color;
use ratatui::style::Modifier;
use ratatui::style::Style;

/// Parse a style specification string into a `Style`.
/// Syntax: comma-separated parts, e.g. "bold,italic,fg=Red,bg=#00FF00".
pub fn parse_style(spec: &str) -> Style {
    let mut style = Style::default();
    for part in spec.split(',').map(str::trim) {
        let low = part.to_lowercase();
        if low == "bold" {
            style = style.add_modifier(Modifier::BOLD);
        } else if low == "italic" {
            style = style.add_modifier(Modifier::ITALIC);
        } else if low == "underline" {
            style = style.add_modifier(Modifier::UNDERLINED);
        } else if let Some(c) = part.strip_prefix("fg=") {
            style = style.fg(parse_color(c));
        } else if let Some(c) = part.strip_prefix("bg=") {
            style = style.bg(parse_color(c));
        }
    }
    style
}

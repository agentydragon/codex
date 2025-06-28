#![allow(clippy::collapsible_if)]
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use std::str::FromStr;

/// Parse a color specification string into a `Color`.
/// Supported formats:
/// - Named colors (case-insensitive), e.g. "Red", "LightBlue", "Reset"
/// - Hex codes `#RRGGBB` or `0xRRGGBB` for full 24-bit RGB
fn parse_color(spec: &str) -> Color {
    let s = spec.trim();
    if s.eq_ignore_ascii_case("reset") {
        return Color::Reset;
    }
    if let Some(hex) = s.strip_prefix('#').or_else(|| s.strip_prefix("0x")) {
        if hex.len() == 6 {
            if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                let r = ((rgb >> 16) & 0xff) as u8;
                let g = ((rgb >> 8) & 0xff) as u8;
                let b = (rgb & 0xff) as u8;
                return Color::Rgb(r, g, b);
            }
        }
    }
    Color::from_str(s).unwrap_or(Color::Reset)
}

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

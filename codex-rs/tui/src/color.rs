//! Helpers to parse config-specified color strings into `ratatui::style::Color`.
use ratatui::style::Color;
use std::str::FromStr;

/// Parse a color specification string into a `Color`.
///
/// Supported formats:
/// - Named colors (case-insensitive), e.g. "Red", "LightBlue", "Reset"
/// - Hex codes `#RRGGBB` or `0xRRGGBB` for full 24-bit RGB
pub fn parse_color(spec: &str) -> Color {
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
    // Fallback to named color parser
    Color::from_str(s).unwrap_or(Color::Reset)
}

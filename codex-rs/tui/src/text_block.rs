use crate::cell_widget::CellWidget;
use ratatui::prelude::*;

/// A simple widget that just displays a list of `Line`s via a `Paragraph`.
/// This is the default rendering backend for most `HistoryCell` variants.
/// A simple widget that displays a list of lines via a paragraph.
#[derive(Clone)]
pub struct TextBlock {
    /// The content lines to render.
    pub lines: Vec<Line<'static>>,
}

impl TextBlock {
    /// Create a new text block from preformatted lines.
    pub fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }
}

impl CellWidget for TextBlock {
    fn height(&self, width: u16) -> usize {
        // Calculate height by wrapping each line manually
        let mut total_height = 0;
        let wrap_cfg = crate::conversation_history_widget::wrap_cfg();

        for line in &self.lines {
            // For empty lines, count as 1 line
            let content: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            if content.trim().is_empty() {
                total_height += 1;
                continue;
            }

            // Calculate wrapped height for non-empty lines
            let line_height = calculate_wrapped_height(&content, width as usize, wrap_cfg.trim);
            total_height += line_height;
        }

        total_height
    }

    fn render_window(&self, first_visible_line: usize, area: Rect, buf: &mut Buffer) {
        // Render lines directly to buffer without using Paragraph widget
        let mut current_row = 0;
        let mut lines_skipped = 0;

        for line in &self.lines {
            // Skip lines until we reach the first visible line
            let content: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

            if content.trim().is_empty() {
                // Handle empty lines
                if lines_skipped >= first_visible_line {
                    if current_row >= area.height as usize {
                        break;
                    }
                    // Empty line - just advance the row (don't render anything)
                    current_row += 1;
                }
                lines_skipped += 1;
                continue;
            }

            // For most lines, render them directly without complex wrapping
            // since our markdown renderer produces reasonably sized lines
            if lines_skipped >= first_visible_line {
                if current_row >= area.height as usize {
                    break;
                }

                // Render the original spans directly to preserve all styling
                render_spans_to_buffer(
                    &line.spans,
                    line.style,
                    area.x,
                    area.y + current_row as u16,
                    area.width,
                    buf,
                );
                current_row += 1;
            }
            lines_skipped += 1;
        }
    }
}

/// Calculate the height a line will take when wrapped
fn calculate_wrapped_height(content: &str, width: usize, trim: bool) -> usize {
    if width == 0 {
        return 1;
    }

    let content = if trim { content.trim() } else { content };
    if content.is_empty() {
        return 1;
    }

    // Simple wrapping calculation - count how many lines we need
    // clippy suggests using `div_ceil`, but that is currently nightly-only for
    // `usize`. Keep the straightforward portable calculation.
    #[allow(clippy::manual_div_ceil)]
    {
        let content_len = content.chars().count();
        ((content_len + width - 1) / width).max(1)
    }
}

/// Render spans to buffer, preserving all styling
fn render_spans_to_buffer(
    spans: &[ratatui::text::Span<'static>],
    base_style: Style,
    x: u16,
    y: u16,
    width: u16,
    buf: &mut Buffer,
) {
    let mut current_x = 0;

    for span in spans {
        let span_style = base_style.patch(span.style);
        for ch in span.content.chars() {
            if current_x >= width as usize {
                break;
            }

            if let Some(cell) = buf.cell_mut((x + current_x as u16, y)) {
                cell.set_char(ch).set_style(span_style);
            }
            current_x += 1;
        }

        if current_x >= width as usize {
            break;
        }
    }
}

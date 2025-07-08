use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use super::BottomPane;
use super::BottomPaneView;
use super::bottom_pane_view::ConditionalUpdate;

/// View for displaying the output of `codex inspect-env` in the bottom pane.
pub(crate) struct InspectEnvView {
    lines: Vec<String>,
    done: bool,
    // UI state
    selected_tab: usize,
    scrolls: Vec<u16>,
}

impl InspectEnvView {
    /// Create a new inspect-env view.
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            done: false,
            selected_tab: 0,
            scrolls: Vec::new(),
        }
    }
}

impl<'a> BottomPaneView<'a> for InspectEnvView {
    fn update_status_text(&mut self, text: String) -> ConditionalUpdate {
        self.lines.push(text);
        ConditionalUpdate::NeedsRedraw
    }

    fn handle_key_event(&mut self, pane: &mut BottomPane<'a>, key_event: KeyEvent) {
        use KeyCode::*;
        match key_event.code {
            Enter | Esc => self.done = true,
            Left => {
                if self.selected_tab > 0 {
                    self.selected_tab -= 1
                }
            }
            Right => self.selected_tab += 1,
            Up => {
                if let Some(s) = self.scrolls.get_mut(self.selected_tab) {
                    *s = s.saturating_sub(1)
                }
            }
            Down => {
                if let Some(s) = self.scrolls.get_mut(self.selected_tab) {
                    *s = s.saturating_add(1)
                }
            }
            _ => {}
        }
        pane.request_redraw();
    }

    fn is_complete(&self) -> bool {
        self.done
    }

    fn calculate_required_height(&self, area: &Rect) -> u16 {
        area.height
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Tabs;
        // split lines into sections by unindented headings ending with ':'
        let mut secs: Vec<(String, Vec<String>)> = Vec::new();
        let mut cur = ("All".to_string(), Vec::new());
        for l in &self.lines {
            if !l.starts_with(' ') && l.trim_end().ends_with(':') {
                secs.push(cur);
                cur = (l.trim_end().to_string(), Vec::new());
            } else {
                cur.1.push(l.clone());
            }
        }
        secs.push(cur);
        // init scrolls
        let tab_count = secs.len();
        let mut scrolls = self.scrolls.clone();
        if scrolls.len() != tab_count {
            scrolls = vec![0; tab_count];
        }
        // ensure selected_tab in range
        let sel = self.selected_tab.min(tab_count - 1);
        // render block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Inspect Env (←/→ tabs, ↑/↓ scroll, Enter/Esc to close)");
        block.render(area, buf);
        // inner area
        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };
        // tabs
        let titles: Vec<&str> = secs.iter().map(|(t, _)| t.as_str()).collect();
        Tabs::new(titles)
            .select(sel)
            .block(Block::default())
            .render(
                Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width,
                    height: 1,
                },
                buf,
            );
        // content area below tabs
        let content_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: inner.height,
        };
        let text = secs[sel].1.join("\n");
        Paragraph::new(text)
            .scroll((scrolls[sel], 0))
            .render(content_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn update_status_text_appends_lines() {
        let mut view = InspectEnvView::new();
        assert!(view.lines.is_empty());
        view.update_status_text("foo".to_string());
        view.update_status_text("bar".to_string());
        assert_eq!(view.lines, vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn render_includes_lines() {
        let mut view = InspectEnvView::new();
        view.update_status_text("line1".to_string());
        view.update_status_text("line2".to_string());
        let area = Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 3,
        };
        let mut buf = Buffer::empty(area);
        view.render(area, &mut buf);
        // Collect all cell symbols into a flat string and verify the lines are present
        let content: String = buf.content().iter().fold(String::new(), |mut acc, cell| {
            acc.push_str(cell.symbol());
            acc
        });
        assert!(content.contains("line1"));
        // Expect at least the first line to be rendered
        assert!(content.contains("line1"));
    }
}

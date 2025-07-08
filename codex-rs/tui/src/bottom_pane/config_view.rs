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
use super::bottom_pane_view::BottomPaneView;
use super::bottom_pane_view::ConditionalUpdate;

/// View for displaying the effective configuration.
pub(crate) struct ConfigView {
    lines: Vec<String>,
    done: bool,
}

impl ConfigView {
    /// Create a new config view with the debug-formatted config string.
    pub fn new(config_str: String) -> Self {
        let lines = config_str.lines().map(|s| s.to_string()).collect();
        Self { lines, done: false }
    }
}

impl<'a> BottomPaneView<'a> for ConfigView {
    fn update_status_text(&mut self, _text: String) -> ConditionalUpdate {
        ConditionalUpdate::NoRedraw
    }

    fn handle_key_event(&mut self, pane: &mut BottomPane<'a>, key_event: KeyEvent) {
        if matches!(key_event.code, KeyCode::Esc | KeyCode::Enter) {
            self.done = true;
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
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Config (Enter/Esc to close)");
        Paragraph::new(self.lines.join("\n"))
            .block(block)
            .render(area, buf);
    }
}

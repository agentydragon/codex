use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use codex_core::exec_history::{ExecHistory, ExecHistoryEntry, ExecHistoryFilter};
use codex_core::protocol::ReviewDecision;

use crate::user_approval_widget::ApprovalRequest;

use super::bottom_pane_view::BottomPaneView;
use super::BottomPane;

pub struct ExecHistoryView {
    history: ExecHistory,
    entries: Vec<ExecHistoryEntry>,
    selected: usize,
    filter: ExecHistoryFilter,
    error_message: Option<String>,
    should_close: bool,
}

impl ExecHistoryView {
    pub fn new(codex_home: &PathBuf) -> Self {
        let history = ExecHistory::new(codex_home);
        let entries = match history.read_all() {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read exec history: {}", e);
                Vec::new()
            }
        };
        
        Self {
            history,
            entries,
            selected: 0,
            filter: ExecHistoryFilter::default(),
            error_message: None,
            should_close: false,
        }
    }
    
    fn reload_entries(&mut self) {
        match self.history.query(self.filter.clone()) {
            Ok(entries) => {
                self.entries = entries;
                self.error_message = None;
                if self.selected >= self.entries.len() && !self.entries.is_empty() {
                    self.selected = self.entries.len() - 1;
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to query exec history: {}", e));
            }
        }
    }
    
    fn format_decision(decision: &Option<ReviewDecision>) -> (&'static str, Style) {
        match decision {
            None => ("N/A", Style::default().fg(Color::DarkGray)),
            Some(ReviewDecision::Approved) => ("Approved", Style::default().fg(Color::Green)),
            Some(ReviewDecision::ApprovedForSession) => ("Session", Style::default().fg(Color::Cyan)),
            Some(ReviewDecision::Denied) => ("Denied", Style::default().fg(Color::Red)),
            Some(ReviewDecision::Abort) => ("Aborted", Style::default().fg(Color::Magenta)),
        }
    }
    
    fn format_status(entry: &ExecHistoryEntry) -> (&'static str, Style) {
        if !entry.execution_started {
            if entry.approval_requested && entry.approval_decision.is_none() {
                ("Pending", Style::default().fg(Color::Yellow))
            } else if matches!(entry.approval_decision, Some(ReviewDecision::Denied) | Some(ReviewDecision::Abort)) {
                ("Rejected", Style::default().fg(Color::Red))
            } else {
                ("Not Run", Style::default().fg(Color::DarkGray))
            }
        } else if let Some(result) = &entry.execution_result {
            if result.error.is_some() {
                ("Error", Style::default().fg(Color::Red))
            } else if let Some(exit_code) = result.exit_code {
                if exit_code == 0 {
                    ("Success", Style::default().fg(Color::Green))
                } else {
                    ("Failed", Style::default().fg(Color::Red))
                }
            } else {
                ("Unknown", Style::default().fg(Color::DarkGray))
            }
        } else {
            ("Running", Style::default().fg(Color::Blue))
        }
    }
}

impl<'a> BottomPaneView<'a> for ExecHistoryView {
    fn handle_key_event(&mut self, _pane: &mut BottomPane<'a>, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_close = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                // Redraw is automatic
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.entries.len() {
                    self.selected += 1;
                }
                // Redraw is automatic
            }
            KeyCode::Char('a') => {
                // Toggle approved filter
                self.filter.approved_only = match self.filter.approved_only {
                    None => Some(true),
                    Some(true) => Some(false),
                    Some(false) => None,
                };
                self.filter.denied_only = None;
                self.reload_entries();
                // Redraw is automatic
            }
            KeyCode::Char('d') => {
                // Toggle denied filter
                self.filter.denied_only = match self.filter.denied_only {
                    None => Some(true),
                    Some(true) => Some(false),
                    Some(false) => None,
                };
                self.filter.approved_only = None;
                self.reload_entries();
                // Redraw is automatic
            }
            KeyCode::Char('r') => {
                // Refresh
                self.reload_entries();
                // Redraw is automatic
            }
            _ => {}
        }
    }
    
    fn is_complete(&self) -> bool {
        self.should_close
    }
    
    fn calculate_required_height(&self, _area: &Rect) -> u16 {
        20 // Use a reasonable fixed height for the table
    }
    
    fn render(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let title = format!(
            " Command Execution History ({} entries) - Press 'q' to close, 'a' for approved, 'd' for denied, 'r' to refresh ",
            self.entries.len()
        );
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        
        // Create table headers
        let headers = vec![
            Cell::from("Time").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Command").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Approval").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Exit").style(Style::default().add_modifier(Modifier::BOLD)),
        ];
        
        // Create table rows
        let rows: Vec<Row> = self.entries.iter().enumerate().map(|(i, entry)| {
            let is_selected = i == self.selected;
            let base_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            
            // Convert SystemTime to local time string (just use relative time for simplicity)
            let time = match entry.timestamp.elapsed() {
                Ok(duration) => {
                    let secs = duration.as_secs();
                    if secs < 60 {
                        format!("{}s ago", secs)
                    } else if secs < 3600 {
                        format!("{}m ago", secs / 60)
                    } else {
                        format!("{}h ago", secs / 3600)
                    }
                }
                Err(_) => "future?".to_string(),
            };
            let command = entry.command.join(" ");
            let (decision_text, decision_style) = Self::format_decision(&entry.approval_decision);
            let (status_text, status_style) = Self::format_status(entry);
            let exit_code = entry.execution_result.as_ref()
                .and_then(|r| r.exit_code)
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());
            
            Row::new(vec![
                Cell::from(time).style(base_style),
                Cell::from(command).style(base_style),
                Cell::from(decision_text).style(base_style.patch(decision_style)),
                Cell::from(status_text).style(base_style.patch(status_style)),
                Cell::from(exit_code).style(base_style),
            ])
        }).collect();
        
        let table = Table::new(rows, vec![
            Constraint::Length(8),   // Time
            Constraint::Min(40),     // Command
            Constraint::Length(12),  // Approval
            Constraint::Length(10),  // Status
            Constraint::Length(6),   // Exit
        ])
        .header(Row::new(headers))
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        
        use ratatui::widgets::Widget;
        table.render(area, buf);
        
        // Show error message if any
        if let Some(error) = &self.error_message {
            let error_line = Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red)),
                Span::raw(error),
            ]);
            let error_area = ratatui::layout::Rect {
                x: area.x + 2,
                y: area.y + area.height - 2,
                width: area.width - 4,
                height: 1,
            };
            use ratatui::widgets::Widget;
            use ratatui::widgets::Paragraph;
            let para = Paragraph::new(error_line);
            para.render(error_area, buf);
        }
    }
    
    fn try_consume_approval_request(&mut self, request: ApprovalRequest) -> Option<ApprovalRequest> {
        // This view doesn't handle approval requests
        Some(request)
    }
}
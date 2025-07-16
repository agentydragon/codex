# Hardcoded TUI Styles

List of code locations where TUI styles or style manipulations are currently hardcoded (not loaded from `config.toml`).

## Ratatui `Style` hardcodes
```text
tui/src/conversation_history_widget.rs:333:                Style::default().fg(Color::LightYellow),
tui/src/conversation_history_widget.rs:336:            ("Messages (tab to focus)", Style::default().dim())
tui/src/conversation_history_widget.rs:462:                Style::reset().fg(Color::LightYellow)
tui/src/conversation_history_widget.rs:464:                Style::reset().fg(Color::Gray)
tui/src/conversation_history_widget.rs:471:                    .begin_style(Style::reset().fg(Color::DarkGray))
tui/src/conversation_history_widget.rs:472:                    .end_style(Style::reset().fg(Color::DarkGray))
tui/src/conversation_history_widget.rs:476:                    .track_style(Style::reset().fg(Color::DarkGray)),
tui/src/bottom_pane/chat_composer.rs:122:        textarea.set_cursor_line_style(ratatui::style::Style::default());
tui/src/bottom_pane/chat_composer.rs:459:                border_style: Style::default().fg(Color::Red),
tui/src/bottom_pane/chat_composer.rs:465:                border_style: Style::default(),
tui/src/bottom_pane/chat_composer.rs:470:                border_style: Style::default().dim(),
tui/src/bottom_pane/chat_composer.rs:529:                Style::default().fg(color),
tui/src/bottom_pane/command_popup.rs:187:            let default_style = Style::default();
tui/src/bottom_pane/command_popup.rs:188:            let command_style = Style::default().fg(Color::LightBlue);
tui/src/bottom_pane/exec_history_view.rs:77:            None => ("N/A", Style::default().fg(Color::DarkGray)),
tui/src/bottom_pane/exec_history_view.rs:78:            Some(ReviewDecision::Approved) => ("Approved", Style::default().fg(Color::Green)),
tui/src/bottom_pane/exec_history_view.rs:80:                ("Session", Style::default().fg(Color::Cyan))
tui/src/bottom_pane/exec_history_view.rs:82:            Some(ReviewDecision::Denied) => ("Denied", Style::default().fg(Color::Red)),
tui/src/bottom_pane/exec_history_view.rs:83:            Some(ReviewDecision::Abort) => ("Aborted", Style::default().fg(Color::Magenta)),
tui/src/bottom_pane/exec_history_view.rs:90:                ("Pending", Style::default().fg(Color::Yellow))
tui/src/bottom_pane/exec_history_view.rs:95:                ("Rejected", Style::default().fg(Color::Red))
tui/src/bottom_pane/exec_history_view.rs:97:                ("Not Run", Style::default().fg(Color::DarkGray))
tui/src/bottom_pane/exec_history_view.rs:101:                ("Error", Style::default().fg(Color::Red))
tui/src/bottom_pane/exec_history_view.rs:104:                    ("Success", Style::default().fg(Color::Green))
tui/src/bottom_pane/exec_history_view.rs:106:                    ("Failed", Style::default().fg(Color::Red))
tui/src/bottom_pane/exec_history_view.rs:109:                ("Unknown", Style::default().fg(Color::DarkGray))
tui/src/bottom_pane/exec_history_view.rs:112:            ("Running", Style::default().fg(Color::Blue))
tui/src/bottom_pane/exec_history_view.rs:183:            .border_style(Style::default().fg(Color::Cyan));
tui/src/bottom_pane/exec_history_view.rs:187:            Cell::from("Time").style(Style::default().add_modifier(Modifier::BOLD)),
tui/src/bottom_pane/exec_history_view.rs:188:            Cell::from("Command").style(Style::default().add_modifier(Modifier::BOLD)),
tui/src/bottom_pane/exec_history_view.rs:189:            Cell::from("Approval").style(Style::default().add_modifier(Modifier::BOLD)),
tui/src/bottom_pane/exec_history_view.rs:190:            Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
tui/src/bottom_pane/exec_history_view.rs:191:            Cell::from("Exit").style(Style::default().add_modifier(Modifier::BOLD)),
tui/src/bottom_pane/exec_history_view.rs:202:                    Style::default().bg(Color::DarkGray)
tui/src/bottom_pane/exec_history_view.rs:204:                    Style::default()
tui/src/bottom_pane/exec_history_view.rs:254:        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
tui/src/bottom_pane/exec_history_view.rs:262:                Span::styled("Error: ", Style::default().fg(Color::Red)),
tui/src/style.rs:32:    let mut style = Style::default();
tui/src/status_indicator_widget.rs:118:            Style::default().add_modifier(Modifier::BOLD),
tui/src/status_indicator_widget.rs:123:            Style::default().add_modifier(Modifier::BOLD),
tui/src/status_indicator_widget.rs:128:                Style::default()
tui/src/status_indicator_widget.rs:132:                Style::default().dim()
tui/src/status_indicator_widget.rs:139:            Style::default().add_modifier(Modifier::BOLD),
tui/src/status_indicator_widget.rs:180:        spans.push(Span::styled(sanitized_tail, Style::default().dim()));
tui/src/git_warning_screen.rs:88:                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
tui/src/git_warning_screen.rs:119:            .style(Style::default().add_modifier(Modifier::BOLD));
tui/src/history_cell.rs:231:            Style::default()
tui/src/history_cell.rs:274:            Style::default()
tui/src/history_cell.rs:314:            Style::default()
tui/src/history_cell.rs:402:        let ann_span = Span::styled(pad.clone(), Style::default().fg(ann_color));
tui/src/history_cell.rs:458:            Span::styled(server, Style::default().fg(Color::Blue)),
tui/src/history_cell.rs:460:            Span::styled(tool, Style::default().fg(Color::Blue)),
tui/src/history_cell.rs:462:            Span::styled(args_str, Style::default().fg(Color::Gray)),
tui/src/history_cell.rs:586:                        lines.push(Line::styled(line_text, Style::default().fg(Color::Gray)));
tui/src/history_cell.rs:596:                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
tui/src/history_cell.rs:664:                    let style_for = |fg| Style::default().fg(fg).add_modifier(Modifier::BOLD);
tui/src/history_cell.rs:680:            Style::default()
```

## `Span::styled`/`RtSpan::styled` hardcodes
```text
tui/src/markdown.rs:54:            let owned_span = Span::styled(span.content.to_string(), span.style);
tui/src/bottom_pane/exec_history_view.rs:262:                Span::styled("Error: ", Style::default().fg(Color::Red)),
tui/src/status_indicator_widget.rs:116:        header_spans.push(Span::styled(
tui/src/status_indicator_widget.rs:121:        header_spans.push(Span::styled(
tui/src/status_indicator_widget.rs:134:            header_spans.push(Span::styled(".", style));
tui/src/status_indicator_widget.rs:137:        header_spans.push(Span::styled(
tui/src/status_indicator_widget.rs:180:        spans.push(Span::styled(sanitized_tail, Style::default().dim()));
tui/src/history_cell.rs:229:        let label = RtSpan::styled(
tui/src/history_cell.rs:272:        let label = RtSpan::styled(
tui/src/history_cell.rs:312:        let label = RtSpan::styled(
tui/src/history_cell.rs:402:        let ann_span = Span::styled(pad.clone(), Style::default().fg(ann_color));
tui/src/history_cell.rs:458:            Span::styled(server, Style::default().fg(Color::Blue)),
tui/src/history_cell.rs:460:            Span::styled(tool, Style::default().fg(Color::Blue)),
tui/src/history_cell.rs:462:            Span::styled(args_str, Style::default().fg(Color::Gray)),
tui/src/history_cell.rs:594:                    Span::styled(
tui/src/history_cell.rs:666:                        "A" => RtSpan::styled(kind.clone(), style_for(Color::Green)),
tui/src/history_cell.rs:667:                        "D" => RtSpan::styled(kind.clone(), style_for(Color::Red)),
tui/src/history_cell.rs:668:                        "M" => RtSpan::styled(kind.clone(), style_for(Color::Yellow)),
tui/src/history_cell.rs:669:                        "R" | "C" => RtSpan::styled(kind.clone(), style_for(Color::Cyan)),
tui/src/history_cell.rs:678:        let label = RtSpan::styled(
tui/src/git_warning_screen.rs:86:            .title(Span::styled(
```

## ANSI/ANSI-like string style hardcodes
```text
tui/src/history_cell.rs:174:                    "codex session".magenta().bold(),
tui/src/history_cell.rs:212:                Line::from("model changed:".magenta().bold()),
tui/src/history_cell.rs:353:            Line::from(vec!["command".magenta(), " running...".dim()]),
tui/src/history_cell.rs:468:        let title_line = Line::from(vec!["tool".magenta(), " running...".dim()]);
tui/src/history_cell.rs:539:            "tool".magenta(),
tui/src/history_cell.rs:641:            let lines = vec![RtLine::from("patch applied".magenta().bold())];
tui/src/history_cell.rs:542:                status_str.green()
tui/src/history_cell.rs:658:                    RtLine::from(line).green()
tui/src/user_approval_widget.rs:555:                buf[(col, row)].set_bg(Color::Red);
```

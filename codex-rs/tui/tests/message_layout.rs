use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::config::ConfigToml;
use codex_tui::history_cell::HistoryCell;
use tempfile::TempDir;

/// Extract plain string content of each line for comparison.
fn lines_from_userprompt(view: &codex_tui::text_block::TextBlock) -> Vec<String> {
    view.lines
        .iter()
        .map(|line| line.spans.iter().map(|s| s.content.clone()).collect())
        .collect()
}

#[test]
#[allow(clippy::unwrap_used, clippy::uninlined_format_args)]
fn test_user_message_layout_combinations() {
    let tmp = TempDir::new().unwrap();
    let mut config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides::default(),
        tmp.path().to_path_buf(),
    )
    .unwrap();
    let message = "first line\nsecond line".to_string();
    for &sender_break in &[false, true] {
        config.tui.sender_break_line = sender_break;
        let cell = HistoryCell::new_user_prompt(&config, message.clone());
        let view = match cell {
            HistoryCell::UserPrompt { view } => view,
            _ => panic!("expected UserPrompt variant"),
        };
        let got = lines_from_userprompt(&view);
        let expected = if sender_break {
            vec![
                "user".to_string(),
                "first line".to_string(),
                "second line".to_string(),
            ]
        } else {
            vec!["user first line".to_string(), " second line".to_string()]
        };
        assert_eq!(
            got, expected,
            "Layout mismatch for sender_break_line={sender_break}"
        );
    }
}

#[test]
#[allow(clippy::unwrap_used, clippy::uninlined_format_args)]
fn test_agent_message_layout_combinations() {
    let tmp = TempDir::new().unwrap();
    let mut config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides::default(),
        tmp.path().to_path_buf(),
    )
    .unwrap();
    let message = "first line\nsecond line".to_string();
    for &sender_break in &[false, true] {
        config.tui.sender_break_line = sender_break;
        let cell = HistoryCell::new_agent_message(&config, message.clone());
        let view = match cell {
            HistoryCell::AgentMessage { view } => view,
            _ => panic!("expected AgentMessage variant"),
        };
        let got = lines_from_userprompt(&view);
        let expected = if sender_break {
            vec![
                "codex".to_string(),
                "first line".to_string(),
                "second line".to_string(),
            ]
        } else {
            vec!["codex first line".to_string(), " second line".to_string()]
        };
        assert_eq!(
            got, expected,
            "Agent layout mismatch for sender_break_line={sender_break}"
        );
    }
}

#[test]
#[allow(clippy::unwrap_used, clippy::uninlined_format_args)]
fn test_agent_reasoning_layout_combinations() {
    let tmp = TempDir::new().unwrap();
    let mut config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides::default(),
        tmp.path().to_path_buf(),
    )
    .unwrap();
    let message = "first line\nsecond line".to_string();
    for &sender_break in &[false, true] {
        config.tui.sender_break_line = sender_break;
        let cell = HistoryCell::new_agent_reasoning(&config, message.clone());
        let view = match cell {
            HistoryCell::AgentReasoning { view } => view,
            _ => panic!("expected AgentReasoning variant"),
        };
        let got = lines_from_userprompt(&view);
        let expected = if sender_break {
            vec![
                "thinking".to_string(),
                "first line".to_string(),
                "second line".to_string(),
            ]
        } else {
            vec![
                "thinking first line".to_string(),
                " second line".to_string(),
            ]
        };
        assert_eq!(
            got, expected,
            "Reasoning layout mismatch for sender_break_line={sender_break}"
        );
    }
}

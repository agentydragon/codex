//! Types used to define the fields of [`crate::config::Config`].

// Note this file should generally be restricted to simple struct/enum
// definitions that do not contain business logic.

use std::collections::HashMap;
use strum_macros::Display;
use wildmatch::WildMatchPattern;

use serde::Deserialize;
use serde::Serialize;

/// Configuration for an external hook process.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HookConfig {
    /// Command and arguments for the hook process.
    pub command: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct McpServerConfig {
    pub command: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum UriBasedFileOpener {
    #[serde(rename = "vscode")]
    VsCode,

    #[serde(rename = "vscode-insiders")]
    VsCodeInsiders,

    #[serde(rename = "windsurf")]
    Windsurf,

    #[serde(rename = "cursor")]
    Cursor,

    /// Option to disable the URI-based file opener.
    #[serde(rename = "none")]
    None,
}

impl UriBasedFileOpener {
    pub fn get_scheme(&self) -> Option<&str> {
        match self {
            UriBasedFileOpener::VsCode => Some("vscode"),
            UriBasedFileOpener::VsCodeInsiders => Some("vscode-insiders"),
            UriBasedFileOpener::Windsurf => Some("windsurf"),
            UriBasedFileOpener::Cursor => Some("cursor"),
            UriBasedFileOpener::None => None,
        }
    }
}

/// Settings that govern if and what will be written to `~/.codex/history.jsonl`.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct History {
    /// If true, history entries will not be written to disk.
    pub persistence: HistoryPersistence,

    /// If set, the maximum size of the history file in bytes.
    /// TODO(mbolin): Not currently honored.
    pub max_bytes: Option<usize>,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HistoryPersistence {
    /// Save all history entries to disk.
    #[default]
    SaveAll,
    /// Do not write history to disk.
    None,
}

/// Collection of settings that are specific to the TUI.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Tui {
    /// By default, mouse capture is enabled in the TUI so that it is possible
    /// to scroll the conversation history with a mouse. This comes at the cost
    /// of not being able to use the mouse to select text in the TUI.
    /// (Most terminals support a modifier key to allow this. For example,
    /// text selection works in iTerm if you hold down the `Option` key while
    /// clicking and dragging.)
    ///
    /// Setting this option to `true` disables mouse capture, so scrolling with
    /// the mouse is not possible, though the keyboard shortcuts e.g. `b` and
    /// `space` still work. This allows the user to select text in the TUI
    /// using the mouse without needing to hold down a modifier key.
    #[serde(default)]
    pub disable_mouse_capture: bool,
    /// When `true`, do not switch to the alternate-screen buffer so that all output
    /// remains in the normal terminal scrollback. The prompt and interactive widgets
    /// still render via ratatui at the bottom.
    #[serde(default)]
    pub non_fullscreen_mode: bool,

    /// When `true`, omit blank lines immediately following Markdown headings
    /// (levels 1–6) in TUI rendering for more compact vertical spacing.
    #[serde(default)]
    pub markdown_compact: bool,
    /// When true, collapse the header and first line of chat/prompt/patch events into one line
    #[serde(default)]
    pub header_compact: bool,

    /// When `true`, render the sender label on its own line above the message content.
    #[serde(default)]
    pub sender_break_line: bool,
    /// Per-element style overrides for the TUI.
    #[serde(default)]
    pub styles: Styles,

    /// Maximum number of visible lines in the chat input composer before scrolling.
    /// The composer will expand up to this many lines; additional content will enable
    /// an internal scrollbar.
    #[serde(default = "default_composer_max_rows")]
    pub composer_max_rows: usize,
    /// Command used to launch the external editor for editing the chat prompt.
    /// Defaults to the `VISUAL` or `EDITOR` environment variable, falling back to `nvim`.
    #[serde(default = "default_editor")]
    pub editor: String,
    /// Require two consecutive Ctrl+D keystrokes to exit the TUI when enabled.
    #[serde(default)]
    pub require_double_ctrl_d: bool,
    /// Timeout in seconds for requiring second Ctrl+D to confirm exit.
    #[serde(default = "default_double_ctrl_d_timeout_secs")]
    pub double_ctrl_d_timeout_secs: u64,
}

fn default_composer_max_rows() -> usize {
    10
}

/// Default editor: `$VISUAL`, then `$EDITOR`, falling back to `nvim`.
fn default_editor() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "nvim".into())
}

/// Default timeout in seconds for the second Ctrl+D confirmation to exit the TUI.
fn default_double_ctrl_d_timeout_secs() -> u64 {
    2
}

impl Default for Tui {
    fn default() -> Self {
        Self {
            disable_mouse_capture: Default::default(),
            non_fullscreen_mode: Default::default(),
            markdown_compact: Default::default(),
            header_compact: Default::default(),
            sender_break_line: Default::default(),
            styles: Styles::default(),
            composer_max_rows: default_composer_max_rows(),
            editor: default_editor(),
            require_double_ctrl_d: false,
            double_ctrl_d_timeout_secs: default_double_ctrl_d_timeout_secs(),
        }
    }
}

/// Style specifications for individual TUI elements. Override in `~/.codex/config.toml`
/// under `[tui.styles]` using kebab-case keys; values are comma-separated modifiers
/// (`bold`, `italic`, `underline`), `fg=<color>`, and `bg=<color>` (named or hex `#RRGGBB`).
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case", default)]
pub struct Styles {
    /// ">40% context left" indicator
    pub context_high: String,
    /// "25%–40% context left" indicator
    pub context_medium: String,
    /// "≤25% context left" indicator
    pub context_low: String,
    /// Focused scrollbar thumb
    pub scroll_thumb_active: String,
    /// Unfocused scrollbar thumb
    pub scroll_thumb_inactive: String,
    /// Scrollbar track
    pub scroll_track: String,
    /// Popup (approval/command) foreground
    pub popup_fg: String,
    /// Popup (approval/command) background
    pub popup_bg: String,
    /// Approval dialog selection style (comma-separated modifiers and colors, e.g. "bold,fg=Blue,underline")
    pub approval_select_style: String,
    /// Approval dialog default/plain style (comma-separated modifiers and colors, e.g. "fg=Gray")
    pub approval_plain_style: String,
    /// Approval dialog error style (comma-separated modifiers and colors, e.g. "fg=Red")
    pub approval_error_style: String,
    /// Composer error border
    pub composer_error_border: String,
    /// History border
    pub history_border: String,
    /// Git warning border
    pub git_warning_border: String,
    /// Status text
    pub status_text: String,
    /// Tool header
    pub tool_header: String,
    /// Tool arguments
    pub tool_args: String,
    /// History approved
    pub history_approved: String,
    /// History denied
    pub history_denied: String,
    /// History pending
    pub history_pending: String,
    /// History session
    pub history_session: String,
    /// History aborted
    pub history_aborted: String,
    /// History rejected
    pub history_rejected: String,
    /// History not run
    pub history_not_run: String,
    /// History error
    pub history_error: String,
    /// History success
    pub history_success: String,
    /// History failed
    pub history_failed: String,
    /// History unknown
    pub history_unknown: String,
    /// History running
    pub history_running: String,
    /// History N/A
    pub history_na: String,
    /// Dim text style
    pub dim_text: String,
    /// Magenta accent color
    pub magenta_accent: String,
    /// Bold text style
    pub bold_text: String,
    /// Red error text
    pub red_error: String,
    /// Green success text
    pub green_success: String,
    /// Session info text
    pub session_info: String,
    /// Command running text
    pub command_running: String,
    /// Tool running text
    pub tool_running: String,
    /// Event text
    pub event_text: String,
    /// Version text
    pub version_text: String,
    /// Research preview text
    pub research_preview_text: String,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            // Context indicators
            context_high: "fg=Green".to_string(),
            context_medium: "fg=Yellow".to_string(),
            context_low: "fg=Red".to_string(),

            // Scrollbar
            scroll_thumb_active: "fg=LightYellow".to_string(),
            scroll_thumb_inactive: "fg=Gray".to_string(),
            scroll_track: "fg=DarkGray".to_string(),

            // Popups
            popup_fg: "fg=LightBlue".to_string(),
            popup_bg: "bg=DarkGray".to_string(),

            // Approval dialog
            approval_select_style: "bold,fg=Blue".to_string(),
            approval_plain_style: "".to_string(),
            approval_error_style: "fg=Red".to_string(),

            // UI borders and elements
            composer_error_border: "fg=Red".to_string(),
            history_border: "fg=Cyan".to_string(),
            git_warning_border: "fg=Red".to_string(),
            status_text: "fg=White".to_string(),

            // Tool styles
            tool_header: "fg=Blue".to_string(),
            tool_args: "fg=Gray".to_string(),

            // History states
            history_approved: "fg=Green".to_string(),
            history_denied: "fg=Red".to_string(),
            history_pending: "fg=Yellow".to_string(),
            history_session: "fg=Cyan".to_string(),
            history_aborted: "fg=Magenta".to_string(),
            history_rejected: "fg=Red".to_string(),
            history_not_run: "fg=DarkGray".to_string(),
            history_error: "fg=Red".to_string(),
            history_success: "fg=Green".to_string(),
            history_failed: "fg=Red".to_string(),
            history_unknown: "fg=DarkGray".to_string(),
            history_running: "fg=Blue".to_string(),
            history_na: "fg=DarkGray".to_string(),

            // Common text styles
            dim_text: "dim".to_string(),
            magenta_accent: "fg=Magenta".to_string(),
            bold_text: "bold".to_string(),
            red_error: "fg=Red,bold".to_string(),
            green_success: "fg=Green".to_string(),
            session_info: "fg=Magenta,bold".to_string(),
            command_running: "fg=Magenta".to_string(),
            tool_running: "fg=Magenta".to_string(),
            event_text: "dim".to_string(),
            version_text: "bold".to_string(),
            research_preview_text: "dim".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ShellEnvironmentPolicyInherit {
    /// "Core" environment variables for the platform. On UNIX, this would
    /// include HOME, LOGNAME, PATH, SHELL, and USER, among others.
    #[default]
    Core,

    /// Inherits the full environment from the parent process.
    All,

    /// Do not inherit any environment variables from the parent process.
    None,
}

/// Policy for building the `env` when spawning a process via either the
/// `shell` or `local_shell` tool.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ShellEnvironmentPolicyToml {
    pub inherit: Option<ShellEnvironmentPolicyInherit>,

    pub ignore_default_excludes: Option<bool>,

    /// List of regular expressions.
    pub exclude: Option<Vec<String>>,

    pub r#set: Option<HashMap<String, String>>,

    /// List of regular expressions.
    pub include_only: Option<Vec<String>>,
}

pub type EnvironmentVariablePattern = WildMatchPattern<'*', '?'>;

/// Deriving the `env` based on this policy works as follows:
/// 1. Create an initial map based on the `inherit` policy.
/// 2. If `ignore_default_excludes` is false, filter the map using the default
///    exclude pattern(s), which are: `"*KEY*"` and `"*TOKEN*"`.
/// 3. If `exclude` is not empty, filter the map using the provided patterns.
/// 4. Insert any entries from `r#set` into the map.
/// 5. If non-empty, filter the map using the `include_only` patterns.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShellEnvironmentPolicy {
    /// Starting point when building the environment.
    pub inherit: ShellEnvironmentPolicyInherit,

    /// True to skip the check to exclude default environment variables that
    /// contain "KEY" or "TOKEN" in their name.
    pub ignore_default_excludes: bool,

    /// Environment variable names to exclude from the environment.
    pub exclude: Vec<EnvironmentVariablePattern>,

    /// (key, value) pairs to insert in the environment.
    pub r#set: HashMap<String, String>,

    /// Environment variable names to retain in the environment.
    pub include_only: Vec<EnvironmentVariablePattern>,
}

impl From<ShellEnvironmentPolicyToml> for ShellEnvironmentPolicy {
    fn from(toml: ShellEnvironmentPolicyToml) -> Self {
        let inherit = toml.inherit.unwrap_or(ShellEnvironmentPolicyInherit::Core);
        let ignore_default_excludes = toml.ignore_default_excludes.unwrap_or(false);
        let exclude = toml
            .exclude
            .unwrap_or_default()
            .into_iter()
            .map(|s| EnvironmentVariablePattern::new_case_insensitive(&s))
            .collect();
        let r#set = toml.r#set.unwrap_or_default();
        let include_only = toml
            .include_only
            .unwrap_or_default()
            .into_iter()
            .map(|s| EnvironmentVariablePattern::new_case_insensitive(&s))
            .collect();

        Self {
            inherit,
            ignore_default_excludes,
            exclude,
            r#set,
            include_only,
        }
    }
}

/// See https://platform.openai.com/docs/guides/reasoning?api-mode=responses#get-started-with-reasoning
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    #[default]
    Medium,
    High,
    /// Option to disable reasoning.
    None,
}

/// A summary of the reasoning performed by the model. This can be useful for
/// debugging and understanding the model's reasoning process.
/// See https://platform.openai.com/docs/guides/reasoning?api-mode=responses#reasoning-summaries
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ReasoningSummary {
    #[default]
    Auto,
    Concise,
    Detailed,
    /// Option to disable reasoning summaries.
    None,
}

/// How to emit ANSI color escapes.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ColorChoice {
    /// Always emit ANSI color codes.
    Always,
    /// Never emit ANSI color codes.
    Never,
    /// Emit ANSI color codes when output is a TTY (default).
    Auto,
}

impl Default for ColorChoice {
    fn default() -> Self {
        ColorChoice::Auto
    }
}

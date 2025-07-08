//! External hooks API for codex-core.
//!
//! Hooks are long-running external processes that receive session lifecycle
//! and model-agent events as newline-delimited JSON on stdin. This allows custom
//! integrations (logging, notifications, metrics) to monitor or respond to events.
//!
//! # Configuration
//!
//! Configure hooks in `Config.hooks` (via `~/.codex/config.toml`):
//!
//! ```toml
//! [[hooks]]
//! command = ["/path/to/hook", "--option"]
//! ```
//!
//! # Lifecycle
//!
//! 1. `HookManager::new` spawns each hook and sends `SessionStart`.
//! 2. On each Codex event, `handle_event` sends `CodexEvent`.
//! 3. On shutdown, `shutdown` sends `SessionEnd` and waits for exit.
//!
//! # Protocol
//!
//! Messages are JSON-serialized `HookEvent` values (snake_case tag):
//!
//! ```json
//! {"type":"session_start","session_id":"..."}
//! {"type":"codex_event","msg":{ /* Event fields */ }}
//! {"type":"session_end","session_id":"..."}
//! ```

use serde_json;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::Command;

use serde::Serialize;
use uuid::Uuid;

use crate::config_types::HookConfig;
use crate::protocol::Event;

/// Events delivered to external hooks.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookEvent {
    SessionStart {
        session_id: Uuid,
    },
    SessionEnd {
        session_id: Uuid,
    },
    CodexEvent(Event),
    /// User invoked a registered slash-command (invocation_id, name, text args).
    SlashCommandInvoked {
        invocation_id: String,
        name: String,
        args: String,
    },
}

/// Manager for external hook processes.
pub struct HookManager {
    procs: Vec<Child>,
}

impl HookManager {
    /// Spawn hook processes and send initial SessionStart event.
    pub fn new(hooks: &[HookConfig], session_id: Uuid) -> Self {
        let mut procs = Vec::new();
        for cfg in hooks {
            if let Some((cmd, args)) = cfg.command.split_first() {
                if let Ok(mut child) = Command::new(cmd)
                    .args(args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    // send session start
                    if let Some(mut stdin) = child.stdin.take() {
                        let evt = HookEvent::SessionStart { session_id };
                        let line = serde_json::to_string(&evt).unwrap() + "\n";
                        tokio::spawn(async move {
                            let _ = stdin.write_all(line.as_bytes()).await;
                        });
                    }
                    procs.push(child);
                }
            }
        }
        HookManager { procs }
    }

    /// Broadcast an event to all hook processes.
    pub fn handle_event(&mut self, event: &Event) {
        let evt = HookEvent::CodexEvent(event.clone());
        let line = match serde_json::to_string(&evt) {
            Ok(t) => t + "\n",
            Err(_) => return,
        };
        for child in &mut self.procs {
            if let Some(mut stdin) = child.stdin.take() {
                let data = line.clone();
                tokio::spawn(async move {
                    let _ = stdin.write_all(data.as_bytes()).await;
                });
            }
        }
    }

    /// Terminate hook processes by sending SessionEnd and waiting.
    pub fn shutdown(mut self, session_id: Uuid) {
        let evt = HookEvent::SessionEnd { session_id };
        let line = serde_json::to_string(&evt).unwrap() + "\n";
        for mut child in self.procs.drain(..) {
            if let Some(mut stdin) = child.stdin.take() {
                let data = line.clone();
                tokio::spawn(async move {
                    let _ = stdin.write_all(data.as_bytes()).await;
                });
            }
            #[allow(clippy::let_underscore_future)]
            let _ = child.wait();
        }
    }
}
impl HookManager {
    /// Read hook stdout lines as JSON `HookResponse` and invoke handler for each.
    pub async fn read_responses<F>(&mut self, handler: F)
    where
        F: Fn(HookResponse) + Send + Clone + 'static,
    {
        for child in &mut self.procs {
            if let Some(stdout) = child.stdout.take() {
                let mut lines = BufReader::new(stdout).lines();
                let handler = handler.clone();
                tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        if let Ok(resp) = serde_json::from_str::<HookResponse>(&line) {
                            handler(resp);
                        }
                    }
                });
            }
        }
    }
}
/// Responses emitted by hook processes via stdout.
/// These drive CLI actions or conversation injections.
#[derive(serde::Deserialize, Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookResponse {
    /// Approve or deny a pending approval request (command/patch).
    ApprovalDecision {
        id: String,
        decision: crate::protocol::ReviewDecision,
    },
    /// Register a new slash-command (name, description).
    RegisterCommand { name: String, description: String },
    /// Unregister a previously registered slash-command by name.
    UnregisterCommand { name: String },
    /// Insert a message visible only to the user (not sent to model).
    InsertUserMessage { content: String },
    /// Insert a message into the conversation (as if user-supplied) to drive the model.
    InsertConversationMessage { content: String },
}

//! Session-scoped recording of raw OpenAI API requests & responses (JSONL).
use std::fs;
use std::fs::File;
use std::io::Error as IoError;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::{self};
use uuid::Uuid;

use crate::config::Config;

/// Timestamp format for API log entries
pub const TS_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");

const SESSIONS_SUBDIR: &str = "sessions";

/// Records structured JSON entries to an API log file under ~/.codex/sessions
#[derive(Clone)]
pub struct ApiLogger {
    tx: Sender<String>,
}

impl ApiLogger {
    /// Initialize per-session API logger, writing to api-{timestamp}-{session_id}.jsonl
    pub async fn new(config: &Config, session_id: Uuid) -> std::io::Result<Self> {
        let mut dir = config.codex_home.clone();
        dir.push(SESSIONS_SUBDIR);
        fs::create_dir_all(&dir)?;
        let now = OffsetDateTime::now_utc();
        let fmt: &[FormatItem] =
            format_description!("[year]-[month]-[day]T[hour]-[minute]-[second]");
        let date = now
            .format(fmt)
            .map_err(|e| IoError::other(format!("failed to format date: {e}")))?;
        let name = format!("api-{}-{}.jsonl", date, session_id);
        let path = dir.join(name);
        let file = File::options().append(true).create(true).open(&path)?;
        let (tx, mut rx) = mpsc::channel::<String>(256);
        tokio::task::spawn(async move {
            let mut file = tokio::fs::File::from_std(file);
            while let Some(line) = rx.recv().await {
                if file.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if file.write_all(b"\n").await.is_err() {
                    break;
                }
                if file.flush().await.is_err() {
                    break;
                }
            }
        });
        Ok(ApiLogger { tx })
    }

    /// Log a pre-serialized JSON value as a line
    pub async fn log(&self, json: &serde_json::Value) -> std::io::Result<()> {
        let line = serde_json::to_string(json)
            .map_err(|e| IoError::other(format!("serialize api log: {e}")))?;
        self.tx
            .send(line)
            .await
            .map_err(|e| IoError::other(format!("queue api log: {e}")))
    }
}
